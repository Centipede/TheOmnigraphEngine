# OmniGraph

OmniGraph is a tool implemented as a web service, but it runs as an executable directly on the computer used for work.

## Design

**Parts:**

- The backend tool with a web frontend. Language: Rust.
- The frontend UI. Language: HTML+JS and/or embedded Vue apps.
- Third-party tools: Tesseract primarily, plus olmOCR and Docling.
- Auxiliary microservices: I like to run Tesseract on a powerful computer and use a small laptop for OmniGraph. For that
  purpose, the system also provides a small executable that can be placed on a server and act as a bridge to OmniGraph.

**Data:**

A project is a folder with:

- A configuration file
- A number of page scan files
- A number of database JSON files: page database, structure database
- A number of OCR input files from external tools, such as Tesseract hOCR files, markdown from olmOCR, etc.

All data resides on the same computer on which OmniGraph is running.

## Concepts

- Section - a section of a book, which is a part of a chapter. Sections are nested inside chapters.
- Flow - a flow of content, which is a part of a section. Flows are nested inside sections and sections are nested inside flows.
- Page - a page of a book. Pages are nested inside chapters.
- hOCR careas - a rectangle on a page that belongs to a flow and a layout (unless there are no flows and/or layouts).
- hOCR blocks - either a textblock or an image. Blocks have type: image, paragraph, part, section, subsection, subsubsection ...

## Databases

### Page database

Contains information about page names, scanned image, dimensions etc.

### Structure database

Though the documentation uses the term 'section', it is really meant to cover a mix of two concepts: Flow and sections.

This is best illustrated with an example:

```
<root>
  +--(S)chapter1
  |   +----(F)main
  |   |     +---(S)section1
  |   +----(F)infoboxes
  +--(S)chapter2
  |   +----(F)main
  |   +----(F)infoboxes
```

This is a book with two chapters and a similar top-level structure for each chapter: A _main_ flow and an _infoboxes_ flow.
What is a flow? Flows are _treated_ as sections but act as parallel sections or containers for subsections and content.

In many textbooks you see this system where you have a main flow of content and a lot of margin boxes and inserted boxes with extra information.
Originally I simply discarded all those boxes which were not a strict part of the main flow. Later I moved them to top level flows.
But this design had the advantage of being able to bundle main-flow, footnotes, infoboxes, further reading, etc. into each chapter, which is also what a lot of books do.
This lowers the distance one has to jump to see infoboxes belonging to a chapter (remember, the section viewer doesn't adhere to a physical layout. The user is presented with the entire section at once).

The structure database really consists of two parts:
- Sections - the tree of sections/flows
- Headlines - pointers to all the pages that contain headlines which mark the beginning of new sections.

The reason one would want to maintain a list of headlines is to avoid having to parse hOCR for all pages on each operation.

### Sections & flows

Organised as a tree of section objects where each section object have these properties:
- path_id: Like machine name... unique within the scope of its parent
- level: Part, section... etc.
- title: Pure text, cleaned up... can be different from the OCR text in the associated headline
- is_linked: If yes, this is a free-floating section)
- is_orphaned: If yes, the headline is no longer valid. Only retained  for survival's sake.
- headline: The associated hOCR block element that causes this to exist
  - page
  - block id (hOCR block id)
- subsection_type (section or flow): How subsections are organised. Either they come in sequence (sections) or they are parallel (flows), which basically means there is no concept of a first or last subsection. Use parallel sections for footnotes, boxes etc.
  - A parent cannot have a mix of child sections and child flows. 
- subsections: Nested array of section objects.

### Headlines

Curated list of headlines:
- page
- block id
- is_linked

All operations that can cause changes to potentially involved (page, block-id) for these sections must load the structure database and update both sections and headlines.




## Workflow

1. Create a project and open it.
2. Fill in the necessary metadata: author, title, physical dimensions, etc.
3. Ingest scans in the [Ingestor](Ingestor.md) and set up page metadata.
4. Mark physical properties of the pages in [Folios](Folios.md).
   These include crop areas, images, headers, columns, etc. There are helper tools for those tasks.
5. Check and edit the overall structure in [Codex](Codex.md).
   This entails assigning machine names to sections, short codes, usage hints, etc.
6. Export the project in the [Bindery](Bindery.md).

## Configuration

Sensitive configuration, such as API keys, is stored in a TOML file in the platform's standard configuration directory:

| Platform | Path                                                   |
|----------|--------------------------------------------------------|
| macOS    | `~/Library/Application Support/omnigraph/secrets.toml` |
| Linux    | `~/.config/omnigraph/secrets.toml`                     |
| Windows  | `%APPDATA%\omnigraph\secrets.toml`                     |

The file is created with `0600` permissions on Unix, meaning owner read/write only. In memory, secret values are wrapped in
`secrecy::Secret<String>` to prevent accidental exposure in logs or debug output.

## Workflow considerations

### Section creation workflow

Personal tastes matter here, but different projects can require different section-creation workflows.

#### Example: Book with irregular division into parts, chapters, subchapters, and sub-subchapters

A four-level partitioning is not unusual. Since a simple script cannot create the overall structure beforehand, I would
probably prefer to manually mark all the headers on each page and then gather them into a structure in one step.

#### Example: Modern textbook with predefined subchapters

Some modern textbooks use an almost predefined chapter subdivision. For example, *Arabs and Israelis* has these subchapters
for each chapter: Main Developments, Narratives, References, etc., along with these intermingling flows: Main/body,
Key Development, Key Figure, and Key Document.

In this case, I need to be able to pre-create the section structure, preferably using Python scripting.