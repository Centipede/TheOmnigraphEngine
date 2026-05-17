# Ingestor

**Status: Not implemented**

Once an empty book / document has been created, the work begins. This is done by workingin the Ingestor.

The test project (see main Readme.md) was built using regular python/django over postgresql.
BeautifulSoup for html processing (hOCR data).

Django+postgresql is fine for regular use, but my takeaway is that I would prefer something more speedy
at least for that part of the application.
Speedy and less memory hungry. I had to change my Proxmox server to using containers only, not VMs to save memory.

Considering:
- Rust for webserver, raw files for data repository and Vue for frontend.
- Go for webserver, postgresql for data repository and regular html+htmx+alpine.js for frontend.
- Sticking to Django but trying to optimize it.

## Configuration

Sensitive configuration (API keys etc.) is stored in a TOML file at the platform's standard config directory:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/omnigraph/secrets.toml` |
| Linux    | `~/.config/omnigraph/secrets.toml` |
| Windows  | `%APPDATA%\omnigraph\secrets.toml` |

The file is created with `0600` permissions on Unix (owner read/write only). In-memory, secret values are wrapped in `secrecy::Secret<String>` to prevent accidental exposure in logs or debug output.

## Functionality

- Ingesting pages to a book from scanned image files or PDF
- Naming pages. Books can have many strategies that make this nontrivial:
  - Blank pages, also in the middle of the book.
  - Prelude with roman numerals
  - Alternative naming schemes (A-1, A-2...)
- Preview:
  - Typographical areas (columns, floats)
  - Paragraph sectioning
  - Show temporary values assigned in a scripting environment.



## Workflow

### Section creation workflow

Personal tastes matter here, but I will exemplify with the differing needs I personally have. I think many can relate to them.


#### Example: Book with irregular division into parts, chapters, subchapters and sub-subchapters.

A four level partitioning is not unusual. Since a simple script cannot create the overall structure beforehand, I would probably prefer to manually mark all the headers on each page and then gather them into a structure in one go. 

#### Example: Modern textbook with predefined subchapters.

Some modern textbooks use an almost predefined chapter subdivision. E.g. Arabs and Israelis for each chapter have these subchapters: Main Developments, Narratives, References, ... and these intermingling flows: Main/body, Key Development, Key Figure, Key Document.

In this case, I need to be able to pre-create the section structure beforehand preferably using python scripting.



## UI


### Book editing page

- A 1st left sidebar with a page list.
- A 2nd left sidebar with a section list.
- A central view.
- Tool buttons.


#### Page list

- Select ranges.
- Assign page numbers / names to a selected range: Number of first.
- Drag selection up / down.

If importing pages that have names like ("scan001.jpg", "scan002.jpg", ...) page related work consists in:
- Assigning proper names (i, ii, iii, iv..., then 1, 2, 3...)
- Rearranging if scanning did not occur in order.
- Spotting missing pages and duplicates.



#### Section list.

If the workflow consists in going through the pages and marking elements as H1, H2, H3, somehow this
per-page operation must be linked to the process of naming sections.

#### Central view

The main stage for the action. Since a lot of interaction will occur here,
it will be broken down into smaller standalone apps.

- Page items. Many actions can be performed directly on the necessary pages:
  - Quickly mark images, tables, ...
  - Inspect correct splitting of paragraphs.

#### Keyboard shortcuts

| Shortcut | Function                                              |
|----------|-------------------------------------------------------|
| z        | Zoom in on the page hovered over in the central view. |
| <-       | Previous page                                         |
| ->       | Next page                                             |
|          |                                                       |
|          |                                                       |
|          |                                                       |

