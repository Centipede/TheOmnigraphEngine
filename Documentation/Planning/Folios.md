# Folios Screen

**Status June 9th 2026: Not implemented**

## Functionality

### Cropping:

No scanned file is ever changed after upload. Cropping is logical only.
However, files sent to Tesseract **will** be cropped on the fly, so Tesseract never sees anything outside the crop edges.

First, mark the crop areas for each page. Page cropping is an underestimated but simple and effective way to discard page
headers, footers, and scanning-edge artifacts. With a proper tool, it is quick to do and often beats automation hands down.

## UI

### Generic layout

- A 1st left sidebar with a page list.
- A 2nd left sidebar with a section list.
- A central view.
- Tool buttons.

## Workflow

### Even/odd pages

In all modes, you can filter on **even/odd** pages, since pages from the same side of a page spread typically need to be
adjusted together.
I often make a rough ballpark adjustment of all pages first. Whether adjusting the top, bottom, left, or right edge, I still
need to adjust even pages differently from odd pages.
Very typically, the book “slides” away from the center position as the scanning process progresses. That is why I need a
tool for range-adjusting, for example, pages 200–400, with the strongest change around page 300.


### Mode: Crop regions

Once Crop mode begins, a session is started, and the current set of crop regions is captured.
Either cancel the session or accept the changes made to affected pages to write back the changes.

When entering Crop mode, the screen changes because the most important aspect of cropping is being able to quickly detect
whether you have come too close to an edge.

There are two view modes: full, showing all pages, and windowed, showing one row or column of pages that fit into a
horizontal or vertical view. In both cases, the view focuses on the edge being edited.

#### Tools

**Wide adjust tool** — This tool adjusts the edge of many pages at once.

If the range is ♾️, all pages are affected by the same amount.

If the range is a number, the change is applied from the currently selected page `N` and extends `range` pages backward and
forward.

The change tapers off according to the desired influence curve: none, linear, or Bézier.

**Single adjust tool** - This tool is meant for very quickly  

#### Keyboard shortcuts

| Shortcut | Function                          |
|----------|-----------------------------------|
|          | CROP MODE                         |
|          |                                   |
|          | ALL TOOLS:                        |
| ⇧⎇F      | Toggle full / windowed mode       |
| ⇧⎇↑      | Top edge edited                   |
| ⇧⎇↓      | Bottom edge edited                |
| ⇧⎇←      | Left edge edited                  |
| ⇧⎇→      | Right edge edited                 |
|          |                                   |
| ,        | Previous page                     |
| .        | Next page                         |
| ⇧,       | 10 pages back                     |
| ⇧.       | 10 pages forward                  |
|          |                                   |
|          | SINGLE ADJUST TOOL:               |
| ↑,↓      | Adjust top/bottom edge (if edited) |
| ←,→       | Adjust left/right edge (if edited) |



