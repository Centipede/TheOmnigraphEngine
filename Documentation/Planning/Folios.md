# Folios Screen

**Status July 28th 2026: Many tools implemented**

## Functionality

### OCR:

2026-07-28: Tesseract will be used as foundation and hOCR as the core format.

### Cropping:

No scanned file is ever changed after upload. Cropping is logical only.
However, files sent to Tesseract **will** be cropped on the fly, so Tesseract never sees anything outside the crop edges.

First, mark the crop areas for each page. Page cropping is an underestimated but simple and effective way to discard page
headers, footers, and scanning-edge artifacts. With a proper tool, it is quick to do and often beats automation hands down.

## UI

### Generic layout

Most views will build on a shared workspace component into which they slot their own specific tools
and other views.

- Left sidebar:
  - Page list
  - Section overview
- A central view split between left/rigth.
  - Page thumbnail grid.
  - Full page preview
- Right sidebar:
  - Tool buttons.
  - hOCR outline.

## Workflow

### Even/odd pages

In all modes, you can filter on **even/odd** pages, since pages from the same side of a page spread typically need to be
adjusted together.
I often make a rough ballpark adjustment of all pages first. Whether adjusting the top, bottom, left, or right edge, I still
need to adjust even pages differently from odd pages.
Very typically, the book “slides” away from the center position as the scanning process progresses. That is why I need a
tool for range-adjusting, for example, pages 200–400, with the strongest change around page 300.

### Mode: Crop

Once Crop mode begins, a session is started, and the current set of crop regions is captured.
Either cancel the session or accept the changes made to affected pages to write back the changes.

When entering Crop mode, the screen changes because the most important aspect of cropping is being able to quickly detect
whether you have come too close to an edge.

~~There are two view modes: full, showing all pages, and windowed, showing one row or column of pages that fit into a
horizontal or vertical view. In both cases, the view focuses on the edge being edited.~~
EDIT: We are happy with a single view mode showing all pages in a grid.

#### Tools

Crop-region editing has two tools:

- **Adjust tool**
- **Assign tool**

Both tools operate on the currently selected set of pages, filtered by the active even/odd page filter if one is enabled.

##### Adjust tool

The Adjust tool is for nudging edges of the selected pages. It is used for gradual correction across single or ranges of pages.

The magnet controls whether an adjustment is applied evenly or as a weighted effect across the selection.

When the magnet is off, the adjustment is applied uniformly to the affected pages.

When the magnet is on, the adjustment follows a selected weighting curve. This makes it possible to express gradual changes,
such as drift through a scan run, without having to adjust each page individually.

The magnet controls include:

- **Taper profile** — the shape of the weighting curve. 

The tool is intended for correcting drift, skewed batches, and gradual scanning-position changes.

##### Assign tool

The Assign tool sets crop-edge values directly.
Unlike the Adjust tool, it does not add to the existing values. It overwrites them.

This tool is intended for quick fixed-value resets, batch normalization, and cases where a group of pages should share
exactly the same crop boundaries.
Good for a rough ballpark starting point.

#### Keyboard shortcuts

| Shortcut | Function                           |
|----------|------------------------------------|
| ,        | Previous page                      |
| .        | Next page                          |
| ⎇ ,     | 10 pages back                      |
| ⎇ .     | 10 pages forward                   |
| ⇧        | Moves selected range endpoint      |
|          |                                    |
|          | CROP MODE                          |
|          |                                    |
|          | ALL TOOLS:                         |
| ⇧⎇↑      | Top edge edited                    |
| ⇧⎇↓      | Bottom edge edited                 |
| ⇧⎇←      | Left edge edited                   |
| ⇧⎇→      | Right edge edited                  |
|          |                                    |
|          | SINGLE ADJUST TOOL:                |
| ↑,↓      | Adjust top/bottom edge (if edited) |
| ←,→       | Adjust left/right edge (if edited) |



On PC use ^ instead of ⎇
⌘ currently not used

### Mode: Recognise regions

Select pages and send to Tesseract.
Set up ocr services and command line tools in Settings.

### Mode: Assist - intelligent editing

A toolset for working on multiple pages at a time.

Tools:
- Auto-assign layout to CAREAs
- Auto-assign flow to CAREAs

These are essential to get out of the way before manually editing single pages.

Go to Overview to set up the book's layouts and flows.
Note: These have not been planned yet.


### Mode: Edit - manual editing

A toolset for working on one page at a time.

This mode will be documented later. It has already evolved substantially and further development
happens live.

Tools available:
Add/remove/join/split CAREAS, BLOCKS (paragraphs, images), LINES, WORDS.
Change block type: Part, H1, ... H6, P



#### Roadmap for the immediate future:

As a more comprehensive picture is formed, I can better see where the pieces fit in.
This is a suggestion:

Master tools and keyboard:

(Q) - Change carea flow:
- (1), (2), ... Change selected CAREAs to the flow preset that one must set up in the project Overview.
  - If 'merge' is ✔️then all selected will be merged into one and changed.
  
(W) - Change carea layout:
- (1), (2), ... Change selected CAREAs to the layout preset that one must set up in the project Overview.
  - If 'merge' is ✔️then all selected will be merged into one and changed.

(E) - Edit

Primary tool:
- (1) - Edit CAREA
- (2) - Edit BLOCK
- (3) - Edit LINE
- (4) - Edit WORD

Secondary tool:
- (A) - Add
- (S) - Select
- (D) - Delete / remove
- (F) - Join/Split
- (G) - Context tool

(R) - Change block type:
- (1) - H1
- (2) - H2
- ... 
- (6) - H6
- (7) - P
- (0) - Part
  - If 'merge' is ✔️then all selected will be merged into one and changed. Note that parent careas may need to merge too.

The Q, W and R tools all select something (either a CAREA or a BLOCK) and change the given attribute.
If merging is desired, the backend will first merge them together into one and then assign their new attribute and do
whatever changes are required in hOCR and databases to accomplish the result.




#### Section workflow

The project contains a single section database like it contains a page database (both json files).

As blocks are converted to headlines (Part, H1, ... H6), the section view in the left sidebar
should indicate that new subsections can be added to the database.
Similarly when a section is deleted, the section view in the left sidebar
should indicate that existing subsections should be removed from the database.

Section interaction on a per-page workflow should be limited. Go to Codex to work on sections.


