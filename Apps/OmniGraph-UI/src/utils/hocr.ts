import type { HocrPage, HocrBlock } from '../types/hocr';

/**
 * Augments an hOCR page by calculating and caching the bounding boxes of the
 * first and last words of each block. This is used for rendering indent/dedent
 * hints in the PageCanvas.
 */
export function augmentHocrPageWithWordCoords(page: HocrPage): HocrPage {
  if (!page || !page.careas) return page;

  for (const carea of page.careas) {
    if (!carea.blocks) continue;
    for (const block of carea.blocks) {
      augmentBlockWithWordCoords(block);
    }
  }
  return page;
}

function augmentBlockWithWordCoords(block: HocrBlock) {
  let firstWordBbox = null;
  let lastWordBbox = null;

  if (!block.lines) return;

  // Find first word of the first non-empty line
  for (const line of block.lines) {
    if (line.words && line.words.length > 0) {
      firstWordBbox = line.words[0].bbox;
      break;
    }
  }

  // Find last word of the last non-empty line
  for (let i = block.lines.length - 1; i >= 0; i--) {
    const line = block.lines[i];
    if (line.words && line.words.length > 0) {
      lastWordBbox = line.words[line.words.length - 1].bbox;
      break;
    }
  }

  block.firstWordBbox = firstWordBbox || undefined;
  block.lastWordBbox = lastWordBbox || undefined;
}
