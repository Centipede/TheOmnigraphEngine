/** Bounding box in scan pixel coordinates: [l, t, r, b] */
export type HocrBbox = [number, number, number, number];

export type HocrLevel = 'page' | 'carea' | 'block' | 'line' | 'word';

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


    if(above && below) {
        const aboveIndex = siblings.indexOf(above);
        const belowIndex = siblings.indexOf(below);
        const adjacentInInput = Math.abs(aboveIndex - belowIndex) === 1;

        if (! adjacentInInput)
            return [null, null]
    }


    return [above, below];
}