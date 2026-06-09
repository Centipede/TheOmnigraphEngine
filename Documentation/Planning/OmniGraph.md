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