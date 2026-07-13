import type {HocrBbox, HocrNode, HocrLevel} from "./hocr.ts";

export type OverlayRole = 'parent' | 'active' | 'child';

export interface OverlayItem {
    id: string;
    level: HocrLevel;
    bbox: HocrBbox;
    role: OverlayRole;
    color: string;
}

export type PageInteractionUpdate = (
    x: number,
    y: number,
    overlappingOverlayItems: OverlayItem[],
    activeItem: HocrNode | null,
    betweenOverlayItems: [HocrNode | null, HocrNode | null],
    betweenOverlaySubItems: [HocrNode | null, HocrNode | null],
) => void;

export interface PointerSettings {
    enabled: boolean;
    color: string;
    icon: string;
    label: string;
}

export function getParentLevel(level: HocrLevel) {
    switch (level) {
        case 'word':
            return 'line';
        case 'line':
            return 'block';
        case 'block':
            return 'carea';
        case 'carea':
            return 'page';
        default:
            return null;
    }
}

export function getChildLevel(level: HocrLevel) {
    switch (level) {
        case 'page':
            return 'carea';
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