/** Bounding box in scan pixel coordinates: [x1, y1, x2, y2] */
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

export interface HocrPar {
    id: string;
    bbox: HocrBbox;
    lang?: string;
    lines: HocrLine[];
}

export interface HocrCarea {
    id: string;
    bbox: HocrBbox;
    pars: HocrPar[];
}

export interface HocrPage {
    page_id: string;
    bbox: HocrBbox;
    careas: HocrCarea[];
}
