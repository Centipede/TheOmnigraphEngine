# The OmniGraph Engine

**Status June 18th 2026: Not ready for use yet! WIP**


An automatic system for the ingestion, transcription, and indexing of printed works.

This is the sister project of the actual study
tool: [The Palimpset Engine](https://github.com/Centipede/ThePalimpsestEngine)

## Prelude

OCR scanning books is a mixed blessing. For personal studies, I scan many books, but I have yet to achieve a
“hole-in-one” success. My requirements are meticulous:

* Structure must remain intact. Chapters, subchapters, main content, and footnotes must be kept separate.
* For search, AI, and MCP purposes, one needs control over how to handle footnotes, callouts, etc.
  For instance, search results should not be cluttered with index pages.
* Flexibility in search methods is essential—e.g., vector search, PostgreSQL FTS, and selective inclusion of whole
  chapters or subchapters.
* For reading and viewing purposes, the physical placement of everything—paragraphs, lines, and words—should be
  preserved as much as possible.

### OCR and AI – Future Promises, Present Realities

There is no doubt that, in the not-too-distant future, AI will solve most of these problems and OCR scanning will work
out of the box.

Today that is not the case, and I cannot wait.

As of April 2026:

* **OlmOCR** fails to correctly structure headlines and discards all physical layout. It does not even attempt to
  preserve it.
* **Docling** also misses headlines, and its physical layout handling is limited to paragraph-level bounding boxes.
* **Tesseract** struggles with many common book design elements—drop caps, callouts, and image captions being merged
  into adjacent columns. However, it does provide per-word and per-line bounding box data.

Each tool has its strengths, but none truly stands out. Even if one combined the best aspects of all of them, an
interactive tool would still be required to fix the remaining issues.

### This Project – The Background Story

I originally addressed these practical challenges the hard way—with a quick-and-dirty web application. It’s spaghetti
code, never meant for public consumption, but it has served me well for over a year.

Now, the time has come to share the useful parts publicly. To do that, I need to rewrite components that I can tolerate
personally but that are not acceptable in a public-facing application.

OCR is one part of the equation; the actual study tool is another.

This project is a sister project to my study tool, which currently runs on my personal server. It, too, requires
restructuring before it can be made public.

Starting today (commit #1), I will dedicate a steady—though limited—amount of spare time to incorporating the lessons
learned and migrating the useful components from the experimental project.

---

**Rene Jensen**
[rene@catatonic.dk](mailto:rene@catatonic.dk)

## The Names

No way around it. ChatGPT and I entertained each other with an enthusiastic joint venture of steampunk, faux victorian
naming fun.

## Planning

As I already have a test project working, I will take an approach that is unusual in the public sphere:
I will write **planning documents** that aim at summing up all the lessons I have learned.

[OmniGraph](Documentation/Planning/OmniGraph.md)

