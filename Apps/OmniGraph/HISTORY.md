# History

All notable changes to this project will be documented in this file.

## Unreleased

### Features
- **Book Structure Database**: Initial implementation of the backend database to handle document structural hierarchy.
- **Codex Modes**: Added new Codex Edit and Script modes for advanced document processing.
- **Section Outline**: Introduced a structural SectionOutline widget to visualize and manage document sections.
- **Assist Mode**: Replaced Inspect Mode with Assist Mode, introducing stubs for auto-layout and flow.
- **hOCR Enhancements**: Added support for flow and layout hints in hOCR CAREA data.

### UI/UX
- **Keyboard-Aligned Layout**: Rearranged various buttons to better match their corresponding physical keyboard placement.
- **Contextual Tools**: Added a 'page' OCR tool that displays Restore options only when in page mode.
- **UI Consolidation**: Condensed several interface elements to improve workspace efficiency.

---

## v0.2.0 (2026-08-14)

### Features
- **Multi-Language OCR**: Support for multiple Tesseract languages and preservation of hOCR language attributes.
- **Bulk Operations**: Implemented bulk merge operations for careas and blocks in the OCR editor.
- **OCR Rescan**: Added ability to rescan selected careas while respecting page crops and blotting out images.
- **Automatic Carea Creation**: Improved OCR workflow with automatic carea creation and multi-item reordering.
- **Page Management**: Added functionality to rename and renumber pages within the Ingestor.
- **OCR Batching**: Support for batching OCR requests to improve processing efficiency.
- **New Edit Mode**: Introduced a comprehensive Edit Mode with panel toggles and a detailed hOCR outline.

### UI/UX
- **Grayscale Visualization**: Used grayscale colors for crop visualization in Inspect and Recognise modes.
- **Workflow Improvements**: Automatically select added careas/blocks; implemented hover indications and expand/collapse in hOCR outline.
- **Split-Screen Preview**: Refactored Page Workspace to include a dedicated Page Preview component with a split-screen layout.

### Keyboard Shortcuts
- **Immediate Actions**: Added Shift+J (Join) and Shift+D (Remove) shortcuts for immediate execution on active selections.
- **Shortcut Refresh**: Updated Edit Mode keyboard shortcuts: Q (flow), W (layout), E (levels 1-4 / tools A-J), and R (block types 0-7).

### Functional Changes
- **hOCR Parser Refactor**: Enhanced the parser to handle `<img>` blocks and preserve additional hOCR tags.
- **Backend Improvements**: Implemented backend logic for moving blocks (up/down) and changing block types.
- **Sort Order Preservation**: Corrected multi-item move operations to preserve document order.

---

## v0.1.0 (2026-06-21)

### Features
- **Initial Release**: The first public release of the OmniGraph Engine.
- **Project Management**: Full suite for creating projects, editing metadata, and managing project folders.
- **Page Ingestion**: Robust image ingestion with thumbnails, full-image zoom, and batch support.
- **Folios Crop Mode**: Dedicated tool for page cropping with edge strip display and adjustment tools.
- **OCR Integration**: Integration with a dedicated Tesseract OCR microservice for document transcription.
- **Secure Settings**: Implementation of secure API key storage for external services.

### UI/UX
- **Unified Interface**: Single-bundle Vue-based UI encompassing Projects, Settings, and Folios views.
- **Filtering & Tools**: Even/odd page filters in Crop Mode and an Assign tool for rapid page naming.
- **Performance**: Optimized page transitions to minimize flicker during reloads.

### Keyboard Shortcuts
- **Navigation**: Introduced `,` (comma) and `.` (period) for quick page navigation.
- **Selection**: Implemented ⌘/⌃-click for range selection in page views.

### Functional Changes
- **Architecture**: Established Rust/Axum backend architecture to serve the API and embedded static assets.
- **OCR Service**: Developed a decoupled OCR microservice with deployment scripts.
- **Storage Model**: Defined the initial project and hOCR storage formats.
