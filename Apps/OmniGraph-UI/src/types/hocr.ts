/** Bounding box in scan pixel coordinates: [l, t, r, b] */
export type HocrBbox = [number, number, number, number];

export type HocrLevel = 'page' | 'carea' | 'block' | 'line' | 'word';

export const HocrLevelOrder: HocrLevel[] = ['page', 'carea', 'block', 'line', 'word'];

export interface HocrWord {
    level: HocrLevel;
    id: string;
    bbox: HocrBbox;
    text: string;
    wconf: number;
}

export interface HocrLine {
    level: HocrLevel;
    id: string;
    bbox: HocrBbox;
    words: HocrWord[];
}

export interface HocrBlock {
    level: HocrLevel;
    id: string;
    bbox: HocrBbox;
    kind: string;
    lang?: string;
    lines: HocrLine[];
}

export interface HocrCarea {
    level: HocrLevel;
    id: string;
    bbox: HocrBbox;
    blocks: HocrBlock[];
}

export interface HocrPage {
    level: HocrLevel;
    page_id: string;
    bbox: HocrBbox;
    careas: HocrCarea[];
}

export type HocrNode = HocrCarea | HocrBlock | HocrLine | HocrWord;


// Items at every level that contain the cursor in multi-select mode.
export type MultiSelect = {
    carea: HocrCarea | null;
    block: HocrBlock | null;
    line: HocrLine | null;
    word: HocrWord | null;
};

export function getChildren(item: HocrNode): (HocrNode)[] {
    if ('blocks' in item) return item.blocks;
    if ('lines' in item) return item.lines;
    if ('words' in item) return item.words;
    return [];
}

export function findItem(page: HocrPage, id: string): HocrNode | null {
    for (const carea of page.careas) {
        if (carea.id === id) return carea;

        for (const block of carea.blocks) {
            if (block.id === id) return block;

            for (const line of block.lines) {
                if (line.id === id) return line;

                for (const word of line.words) {
                    if (word.id === id) return word;
                }
            }
        }
    }

    return null;
}

export function findMultiLevelItemByPoint(hocrPage: HocrPage, x: number, y: number): MultiSelect | null {
    const page = hocrPage;
    if (!page) return null;
    for (const carea of page.careas) {
        if (!bboxContainsPoint(carea.bbox, x, y)) continue;
        const result: MultiSelect = {carea, block: null, line: null, word: null};
        for (const block of carea.blocks) {
            if (!bboxContainsPoint(block.bbox, x, y)) continue;
            result.block = block;
            for (const line of block.lines) {
                if (!bboxContainsPoint(line.bbox, x, y)) continue;
                result.line = line;
                for (const word of line.words) {
                    if (!bboxContainsPoint(word.bbox, x, y)) {
                        continue;
                    }
                    result.word = word;
                    break;
                }
                break;
            }
            break;
        }
        return result;
    }
    return null;
}

export function findMultilevelById(page: HocrPage, id: string): MultiSelect | null {
    for (const carea of page.careas) {
        if (carea.id === id) {
            return {carea, block: null, line: null, word: null};
        }

        for (const block of carea.blocks) {
            if (block.id === id) {
                return {carea, block, line: null, word: null};
            }

            for (const line of block.lines) {
                if (line.id === id) {
                    return {carea, block, line, word: null};
                }

                for (const word of line.words) {
                    if (word.id === id) {
                        return {carea, block, line, word};
                    }
                }
            }
        }
    }

    return null;
}

export function sortBylevel(nodes: HocrNode[]): HocrNode[] {
    const cmp = (a: HocrNode, b: HocrNode) => {
        const alevel = HocrLevelOrder.indexOf(a.level);
        const blevel = HocrLevelOrder.indexOf(b.level);
        if (alevel === -1 || blevel === -1)
            return 0;

        return alevel - blevel;
    }

    return [...nodes].sort(cmp);
}

export function bboxContainsPoint(bbox: HocrBbox, x: number, y: number): boolean {
    const [left, top, right, bottom] = bbox;
    return x >= left && x <= right && y >= top && y <= bottom;
}

export function findSiblingsAroundCursor(
    siblings: HocrNode[],
    x: number,
    y: number,
    tolerance = 8,
): [HocrNode | null, HocrNode | null] {
    let above: HocrNode | null = null;
    let below: HocrNode | null = null;

    let bestAboveBottom = -Infinity;
    let bestBelowTop = Infinity;

    for (const sibling of siblings) {
        const [left, top, right, bottom] = sibling.bbox;

        const horizontallyRelevant =
            x >= left - tolerance &&
            x <= right + tolerance;

        if (!horizontallyRelevant) {
            continue;
        }

        /**
         * "Above" means the sibling's bottom edge is above the cursor,
         * with some tolerance allowing it to be slightly below the cursor.
         */
        if (bottom <= y + tolerance && bottom > bestAboveBottom) {
            above = sibling;
            bestAboveBottom = bottom;
        }

        /**
         * "Below" means the sibling's top edge is below the cursor,
         * with some tolerance allowing it to be slightly above the cursor.
         */
        if (top >= y - tolerance && top < bestBelowTop) {
            below = sibling;
            bestBelowTop = top;
        }
    }


    if (above && below) {
        const aboveIndex = siblings.indexOf(above);
        const belowIndex = siblings.indexOf(below);
        const adjacentInInput = Math.abs(aboveIndex - belowIndex) === 1;

        if (!adjacentInInput)
            return [null, null]
    }


    return [above, below];
}

export function sortIdsByDocumentOrder(page: HocrPage, ids: string[]): string[] {
    const indexMap = new Map<string, number>();
    let currentIndex = 0;

    for (const carea of page.careas) {
        indexMap.set(carea.id, currentIndex++);
        for (const block of carea.blocks) {
            indexMap.set(block.id, currentIndex++);
            for (const line of block.lines) {
                indexMap.set(line.id, currentIndex++);
                for (const word of line.words) {
                    indexMap.set(word.id, currentIndex++);
                }
            }
        }
    }

    return [...ids].sort((a, b) => {
        const indexA = indexMap.get(a) ?? Infinity;
        const indexB = indexMap.get(b) ?? Infinity;
        return indexA - indexB;
    });
}