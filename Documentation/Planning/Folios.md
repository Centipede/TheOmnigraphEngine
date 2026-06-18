# Folios Screen

**Status June 18th 2026: Partially implemented**

## Functionality

### OCR:

For now, OCR is done by Tesseract. 2026-06-18: Further info coming.

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



