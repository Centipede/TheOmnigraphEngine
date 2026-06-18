# Ingestor Screen

**Status June 18th 2026: Partially implemented**

**Missing: Import from PDF**

Once an empty book or document has been created, raw scans need to be loaded into the project.

## Workflow

- Ingest pages into a book from scanned image files or PDFs.
- Name pages. Books can use many strategies that make this nontrivial:
  - Blank pages, including pages in the middle of the book.
  - Front matter with Roman numerals.
  - Alternative naming schemes, such as A-1, A-2, etc.
- Preview:
  - Typographical areas, such as columns and floats.
  - Paragraph sectioning.
  - Temporary values assigned in a scripting environment.

## UI

### Book editing page

- A left sidebar with a page list: page names, image names, and batch import.
- A central split-screen view: left = thumbnails, right = full page.
- Tool buttons.

#### Page list

- Select ranges.
- Assign page numbers or names to a selected range, starting from the first number/name.
- Insert, append, and remove pages.

If importing pages with names such as `scan001.jpg`, `scan002.jpg`, etc., page-related work consists of:

- Assigning proper names: i, ii, iii, iv, etc., followed by 1, 2, 3, etc.
- Rearranging pages if scanning did not occur in order.
- Spotting missing pages and duplicates.

#### Keyboard shortcuts

| Shortcut | Function                      |
|---------|-------------------------------|
| ,       | Previous page                 |
| .       | Next page                     |
| ⎇ ,     | 10 pages back                 |
| ⎇ .     | 10 pages forward              |
| ⇧        | Moves selected range endpoint |

## Preliminary lessons from the test project

The test project, see the main [README](../../README.md), was built using regular Python/Django over PostgreSQL, with BeautifulSoup for
HTML processing of hOCR data.

Django and PostgreSQL are fine for regular use, but my takeaway is that I would prefer something faster and less
memory-hungry, at least for this part of the application. I had to change my Proxmox server to use containers only, not VMs,
to save memory.

Considering:

- **Rust for the web server, raw files for the data repository, and Vue for the frontend.** Chosen for now.
- Go for the web server, PostgreSQL for the data repository, and regular HTML, htmx, and Alpine.js for the frontend.
- Sticking to Django but trying to optimize it.