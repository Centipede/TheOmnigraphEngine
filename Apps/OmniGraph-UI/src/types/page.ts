export type HintType = 'dropcap' | 'image';

export interface Hint {
    type: HintType;
    area: CropEdges;
}

export interface Page {
    index: number;
    name: string;
    scan: string;
    scan_width: number;
    scan_height: number;
    thumb: string;
    thumb_width: number;
    thumb_height: number;
    batch: number;
    import_order: number;
    crop_edges: CropEdges;
    hints?: Hint[];
}

export interface PageDb {
    pages: Page[];
    next_batch: number;
}

export interface CropEdges {
    left:   number;  // scan pixels from left edge
    top:    number;  // scan pixels from top edge
    right:  number;  // scan pixels from right edge
    bottom: number;  // scan pixels from bottom edge
}

