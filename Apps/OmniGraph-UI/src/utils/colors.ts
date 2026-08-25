
import type { ColorSpecification } from '../types';

export function makeVariedPalette(
    color: string,
    count = 8,
    options: {
        hueRange?: number;
        saturationRange?: number;
        lightnessRange?: number;
        alpha?: number;
    } = {},
): string[] {
    const {
        hueRange = 24,
        saturationRange = 20,
        lightnessRange = 20,
        alpha = 0.25,
    } = options;

    const rgb = parseCssColorToRgb(color);

    if (!rgb) {
        console.warn(`Invalid color: ${color}`);
        return [];
    }

    const base = rgbToHsl(rgb.r, rgb.g, rgb.b);

    const variationMode: 'fade'|'random' = 'fade'

    if (variationMode === 'fade') {

        const values = Array.from({ length: count }, (_,i) => i/(count-1));

        const hues = Array.from(values, (weight) => wrapHue(base.h + clamp(weight, -hueRange, hueRange)));
        const saturations = Array.from(values, (weight) => clamp(fadeBetween (base.s - saturationRange, base.s + saturationRange, weight), 0,100));
        const lightnesses = Array.from(values, (weight) =>clamp(fadeBetween (base.s - lightnessRange, base.s + lightnessRange, weight), 0,100));

        return Array.from({length:count}, (_,i) => `hsla(${Math.round(hues[i])}, ${Math.round(saturations[i])}%, ${Math.round(lightnesses[i])}%, ${alpha})`);
    }

    else if (variationMode === 'random') {
        return Array.from({ length: count }, () => {
            const hue = wrapHue(base.h + randomBetween(-hueRange, hueRange));
            const saturation = clamp(base.s + randomBetween(-saturationRange, saturationRange), 0, 100);
            const lightness = clamp(base.l + randomBetween(-lightnessRange, lightnessRange), 0, 100);

            return `hsla(${Math.round(hue)}, ${Math.round(saturation)}%, ${Math.round(lightness)}%, ${alpha})`;
        });
    }

    return [];
}

function parseCssColorToRgb(color: string): { r: number; g: number; b: number } | null {
    const trimmed = color.trim();

    if (trimmed.startsWith('#')) {
        return parseHexColorToRgb(trimmed);
    }

    const rgbaMatch = trimmed.match(
        /^rgba?\(\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)(?:\s*,\s*(\d+(?:\.\d+)?))?\s*\)$/i,
    );

    if (rgbaMatch) {
        return {
            r: clamp(Math.round(Number(rgbaMatch[1])), 0, 255),
            g: clamp(Math.round(Number(rgbaMatch[2])), 0, 255),
            b: clamp(Math.round(Number(rgbaMatch[3])), 0, 255),
        };
    }

    return null;
}

function parseHexColorToRgb(color: string): { r: number; g: number; b: number } | null {
    const hex = color.trim().replace(/^#/, '');

    const normalized =
        hex.length === 3
            ? hex.split('').map(char => char + char).join('')
            : hex;

    if (!/^[\da-f]{6}$/i.test(normalized)) {
        return null;
    }

    return {
        r: parseInt(normalized.slice(0, 2), 16),
        g: parseInt(normalized.slice(2, 4), 16),
        b: parseInt(normalized.slice(4, 6), 16),
    };
}

function rgbToHsl(r255: number, g255: number, b255: number): { h: number; s: number; l: number } {
    const r = r255 / 255;
    const g = g255 / 255;
    const b = b255 / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const delta = max - min;

    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (delta !== 0) {
        s = delta / (1 - Math.abs(2 * l - 1));

        switch (max) {
            case r:
                h = 60 * (((g - b) / delta) % 6);
                break;
            case g:
                h = 60 * ((b - r) / delta + 2);
                break;
            case b:
                h = 60 * ((r - g) / delta + 4);
                break;
        }
    }

    return {
        h: wrapHue(h),
        s: s * 100,
        l: l * 100,
    };
}

function randomBetween(min: number, max: number): number {
    return min + Math.random() * (max - min);
}

function fadeBetween(min: number, max: number, weight: number): number {
    return min + (max - min) * weight;
}
function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
}

function wrapHue(hue: number): number {
    return ((hue % 360) + 360) % 360;
}

export function applyColorSpecs(baseColor: string, specs: ColorSpecification[]): string {
    let currentBase = baseColor;
    let totalHueShift = 0;
    let totalSaturationShift = 0;
    let totalLightnessShift = 0;

    for (const spec of specs) {
        if (spec.base_color) {
            currentBase = spec.base_color;
        }
        totalHueShift += spec.hue_shift ?? 0;
        totalSaturationShift += spec.saturation_shift ?? 0;
        totalLightnessShift += spec.lightness_shift ?? 0;
    }

    const rgb = parseCssColorToRgb(currentBase);
    if (!rgb) {
        return currentBase;
    }

    const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b);

    const h = wrapHue(hsl.h + totalHueShift);
    const s = clamp(hsl.s + totalSaturationShift, 0, 100);
    const l = clamp(hsl.l + totalLightnessShift, 0, 100);

    return `hsl(${Math.round(h)}, ${Math.round(s)}%, ${Math.round(l)}%)`;
}
