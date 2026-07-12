import type {HocrBbox, HocrSibling} from "./hocr.ts";

export type HocrOverlayLevel = 'carea' | 'block' | 'line' | 'word';
export type OverlayRole = 'parent' | 'active' | 'child';

export interface OverlayItem {
    id: string;
    level: HocrOverlayLevel;
    bbox: HocrBbox;
    role: OverlayRole;
    color: string;
}

export type PageInteractionUpdate = (
    x: number,
    y: number,
    overlappingOverlayItems: OverlayItem[],
    betweenOverlayItems: [HocrSibling | null, HocrSibling | null],
) => void;

export interface PointerSettings {
    enabled: boolean;
    color: string;
    icon: string;
    label: string;
}

export function getParentLevel(level: HocrOverlayLevel) {
    switch (level) {
        case 'word':
            return 'line';
        case 'line':
            return 'block';
        case 'block':
            return 'carea';
        default:
            return null;
    }
}

export function getChildLevel(level: HocrOverlayLevel) {
    switch (level) {
        case 'carea':
            return 'block';
        case 'block':
            return 'line';
        case 'line':
            return 'word';
        default:
            return null;
    }
}