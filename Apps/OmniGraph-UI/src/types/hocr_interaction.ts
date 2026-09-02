import type {HocrBbox, HocrNode, HocrLevel} from "./hocr.ts";
import type {ColorSpecification} from "./project.ts";

export type OverlayRole = 'parent' | 'active' | 'child';

export interface EditorPalette {
    keepColor: string;
    discardColor: string;
    careaOverlayColor: string;
    blockOverlayColor: string;
    lineOverlayColor: string;
    wordOverlayColor: string;
    partColor: ColorSpecification;
    h1Color: ColorSpecification;
    h2Color: ColorSpecification;
    h3Color: ColorSpecification;
    h4Color: ColorSpecification;
    h5Color: ColorSpecification;
    h6Color: ColorSpecification;
    pColor: ColorSpecification;
    imgColor: ColorSpecification;
    lstColor: ColorSpecification;
    tblColor: ColorSpecification;
}

export const DEFAULT_PALETTE: EditorPalette = {
    keepColor: 'rgba(0, 180, 0, 0.12)',
    discardColor: 'rgba(220, 0, 0, 0.35)',
    careaOverlayColor: 'rgba(249, 115, 22, 1)',
    blockOverlayColor: 'rgba(168, 85, 247, 1)',
    lineOverlayColor: 'rgba(59, 130, 246, 1)',
    wordOverlayColor: 'rgba(34, 197, 94, 1)',
    partColor: { hue_shift: 0 },
    h1Color: { hue_shift: 30 },
    h2Color: { hue_shift: 60 },
    h3Color: { hue_shift: 90 },
    h4Color: { hue_shift: 120 },
    h5Color: { hue_shift: 150 },
    h6Color: { hue_shift: 180 },
    pColor: { hue_shift: 210 },
    imgColor: { hue_shift: 240 },
    lstColor: { hue_shift: 270 },
    tblColor: { hue_shift: 300 },
};

export interface OverlayItem {
    id: string;
    level: HocrLevel;
    index: number;
    bbox: HocrBbox;
    role: OverlayRole;
    color: string;
    kind: string | null;
    wconf?: number;
    lang?: string;
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