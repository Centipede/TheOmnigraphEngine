# Bridging / linking logic

(Based on github issue #140)

## Introduction

Page change as well as carea changes forces a paragraph to be cut into smaller blocks, which is why special care must be given to whether the author intended the split or not. The goal is to detect and mark as linked/bridged unintended splits, but not intended ones.

NB: We examine both the horizontal space from paragraph's left edge to the first word in the first line and the horizontal space after the last word on the last line to the paragraph's right edge. The first we call "indent" which is a bad choice of word. The second we call "dedent" which is an even worse word.

## UI:

In BridgeMode add a button "Auto detect".

It should have a couple of parameters:

**X indent** which is really two numbers: 
- X-indent-certainly-true: Above this number means we can be sure the typographer intended this as a new paragraph. Default 15
- X-indent-certainly-false: Below this number means we can be sure the typographer intended this as  not a new paragraph. Default 5


**X dedent** which is really two numbers: 
- X-dedent-certainly-true: Above this number means we can be sure the typographer intended this as a ending the paragraph. Default 20
- X-dedent-certainly-false: Below this number means we can be sure the typographer intended this as not ending the paragraph. Default 0. (Note that a paragraph can of course end in a long line and still end. Let the user look at the page and judge.) 

**Y advance** (default 0) which is the number of pixels between the preceding line's bottom and this line's top.

Both of these should also have a checkbox that determines if they are used during auto-detection.


## Nomenclature:

Some block, **B**, has a **flow** and is on a **page**. We are interested in all blocks within a specific flow. The flow of a block is the flow of its parent carea. "A flow" is the short way of speaking about all blocks of all careas belonging to a specific flow.
"The careas of a flow" is the well-determined document order of all careas with that flow.
"The blocks of a flow" is the well-determined document order of all blocks of the careas of a flow.

**preceding(B)** is the block in the blocks of a flow that precedes B.
**succeeding(B)** is the block in the blocks of a flow that succeeds B.
Pseudo code:
```
If the block is not the first block in its carea:
    preceding(B) is the first block in this page that come before block ID in document order and has the same flow as X.
Else if the block IS the first in its carea:
    predecessors = (previous(carea).blocks.filter(flow=B.flow)).reverse + ... next previous block ditto ... + next previous block ditto ...
    preceding(B) = predecessors[0] if length(predecessors)>0 else None
```

**Per-block properties that are hard calculations:**

**x_indent**(B) = B.line[0].bbox.left - B.bbox.left
**x_dedent**(B) = B.bbox.right - B.line[last].bbox.right

**Per-boundary properties that are hard calculations:**

**y_advance**(A,B) = B.bbox.top - A.bbox.bottom

Note that x_indent and x_dedent requires blocks of more than one line to calculate.
Note that y_advance requires two blocks.

Counting as input to the auto-detection algorithm are the criteria given by  the user. These are a small list of tests, whose truth-value should come into play:

test_x_indent
test_y_advance
test_hyphenation
(more to come in the future)

## The logic of auto-detection - all tests has true if pointing to a break

**What can be said with certainty (assuming a book that the user has identified as using x_indent and/or y_advance as visual indicators of paragraph break):**

I think we should use an enum to store the state of these as an Option is too limited:
- Untested ... simply not tested yet
- Undetermined ... cannot be confirmed true nor false
- Suggested(true) or Suggested(false) ... algorithmicly suggested but not fully determined
- Determined(true) or Determined(false) ... algorithmically determined
- Assigned(true) or Assigned(false) ... assigned by the user which must take precedence always
- Error ... used to communicate a conflict that the user must deal with

Each enabled test produced evidence. This evidence is then stored on the block and also gathered into a final formula, which the user can override at will.

### Low level evidence/indicators of change - per block:

**B.test_x_indent**
The first line seems to begin after the block's left edge.
A per-block calculation. Only processed if test_x_indent is enabled.
```
if B.test_x_indent == Assigned: 
  // do nothing. Never change user assignment
    
else:
  B.test_x_indent = 
     Determined(true) if x_indent(B) >= threshold.max
     Suggested(true) if value above min, below max
     Determined(false) if x_indent(B) < threshold.min
```

**B.test_x_dedent**
The last line seems to end before the block's right edge.
A per-block calculation / user manipulation. Only processed if test_x_dedent is enabled.
```
if B.test_x_dedent == Assigned: 
  // do nothing. Never change user assignment

else:
  B.test_x_dedent = 
     Determined(true) if x_dedent(B) >= threshold.max
     Suggested(true) if value above min, below max
     Determined(false) if x_dedent(B) < threshold.min
```


**B.test_hyphenation**
Does the block end i a hyphenation?
A per-block calculation
```
if B.test_hyphenation == Assigned: 
  // do nothing. Never change user assignment

else:
  B.test_hyphenation = 
   Suggested(true) if the last word in the block ends in a hyphen
```

### Low level evidence/indicators of change - per boundary:
The algorithm must also examine all boundary condition between A and B blocks.
This information is actually stored on both A and B to make it easy for the frontend to render what it needs to render.


**test_y_advance**
Is there space between blocks A and B
A per-boundary calculation
```
if test_y_advance and A,B ∈ same carea:
  A.test_y_advance = B.test_y_reverse =
     Determined(true)  if y_advance(A, B) >= threshold.max 
     Suggested(true) if value above min, below max
     Determined(false) if y_advance(A, B) < threshold.min
   else:
      Unknown
```

### Mid level calculations:

First of all, we still have the hint system:
B.break_from_preceding = Assigned(true/false)
B.break_from_following = Assigned(true/false)

The user can provide such hints which will take precedence and be preserved.

```
if B.break_from_following == Assigned: 
  // do nothing. Never change user assignment
    
else:
  tests = B.tests.filter(t in [B.test_x_dedent, B.test_y_advance]) // Relevant to B, B+1
  B.break_from_following = ...
  if tests are all Determined(x) then Determined(x)
  if tests contains some Determined(true) and some Determined(false) then Error
  if tests contains some Determined(x) and some Suggested(_) then Determined(x)
```

And

```
if B.break_from_preceding == Assigned: 
  // do nothing. Never change user assignment
    
else:
  tests = B.tests.filter(t in [B.test_x_indent, B.test_y_advance]) // Relevant to B, B+1
  B.break_from_preceding = ...
  if tests are all Determined(x) then Determined(x)
  if tests contains some Determined(true) and some Determined(false) then Error
  if tests contains some Determined(x) and some Suggested(_) then Determined(x)
```
### High level calculations:

Finally we should of course internally in memory handle the high level comparison between a block B and its preceding (A) and following (C) block to allow the frontend to visualize errors if A thinks it should continue into B but B thinks it starts a new block.

```
// In the following these count as true/false: Assigned(true/false), Determined(true/false), Suggested(true/false)

B.continued = 
   Determined(true) if A.break_from_following==false and B.break_from_preceding==false 
   Determined(false) if A.break_from_following==true and B.break_from_preceding==true 
   Error if A.break_from_following != B.break_from_preceding

B.continues = 
  ... similarly, just for B,C

```





















