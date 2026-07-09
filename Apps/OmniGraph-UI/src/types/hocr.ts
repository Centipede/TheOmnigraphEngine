/** Bounding box in scan pixel coordinates: [l, t, r, b] */
export type HocrBbox = [number, number, number, number];

export interface HocrWord {
    id: string;
    bbox: HocrBbox;
    text: string;
    wconf: number;
}

export interface HocrLine {
    id: string;
    bbox: HocrBbox;
    words: HocrWord[];
}

export interface HocrBlock {
    id: string;
    bbox: HocrBbox;
    kind: string;
    lang?: string;
    lines: HocrLine[];
}

export interface HocrCarea {
    id: string;
    bbox: HocrBbox;
    blocks: HocrBlock[];
}

export interface HocrPage {
    page_id: string;
    bbox: HocrBbox;
    careas: HocrCarea[];
}
