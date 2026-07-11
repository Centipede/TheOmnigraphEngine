<template>
  <div class="page-preview">
    <div class="page-preview-image-frame">
      <div class="page-preview-image-wrap">
        <img
            :src="src"
            class="page-preview-image"
            :alt="label"
            :title="label"
        />

        <template v-if="showCropOverlay && crop">
          <div class="preview-discard" :style="topDiscardStyle"/>
          <div class="preview-discard" :style="bottomDiscardStyle"/>
          <div class="preview-discard" :style="leftDiscardStyle"/>
          <div class="preview-discard" :style="rightDiscardStyle"/>
          <div class="preview-crop-area" :style="cropAreaStyle"/>
        </template>

        <div
            v-for="item in overlayItems"
            :key="item.id"
            class="hocr-overlay"
            :class="`hocr-overlay--${item.role}`"
            :style="overlayItemStyle(item)"
        />
      </div>
    </div>

    <div class="page-preview-info">
      <span class="page-preview-index">({{ page.index }})</span>
      <span :class="{ 'page-preview-unnamed': !page.name }">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, type Ref} from 'vue';
import type {CropEdges, HocrBbox, HocrPage, Page} from '../types';
import {makeVariedPalette} from '../utils/colors';
type HocrOverlayLevel = 'carea' | 'block' | 'line' | 'word';
type OverlayRole = 'parent' | 'active' | 'child';

interface OverlayItem {
  id: string;
  bbox: HocrBbox;
  role: OverlayRole;
  color: string;
}

const LEVELS: HocrOverlayLevel[] = ['carea', 'block', 'line', 'word'];

const props = withDefaults(defineProps<{
  page: Page;
  imageBaseUrl: string;
  crop?: CropEdges;
  showCropOverlay?: boolean;
  cropColor?: string;
  discardColor?: string;
  hocrLevel?: HocrOverlayLevel | null;
  careaOverlayColor?: string;
  blockOverlayColor?: string;
  lineOverlayColor?: string;
  wordOverlayColor?: string;
}>(), {
  showCropOverlay: true,
  cropColor: 'rgba(0, 180, 0, 0.12)',
  discardColor: 'rgba(220, 0, 0, 0.28)',
  hocrLevel: null,
  careaOverlayColor: 'rgba(249, 115, 22)',
  blockOverlayColor: 'rgba(168, 85, 247)',
  lineOverlayColor: 'rgba(34, 197, 94)',
  wordOverlayColor: 'rgba(59, 130, 246)',
});

const hocrPage = inject<Ref<HocrPage | null>>('hocrPage');

const label = computed(() => props.page.name || props.page.scan);
const src = computed(() => props.imageBaseUrl + props.page.scan);

const colorByLevel = computed(() => [
  props.careaOverlayColor!,
  props.blockOverlayColor!,
  props.lineOverlayColor!,
  props.wordOverlayColor!,
]);

const overlayItems = computed((): OverlayItem[] => {
  const page = hocrPage?.value;
  if (!page || !props.hocrLevel) return [];

  const activeIdx = LEVELS.indexOf(props.hocrLevel);
  const colors = colorByLevel.value;
  const colorVars = colors.map(color => makeVariedPalette(color));
  console.log(colorVars);
  const items: OverlayItem[] = [];

  function roleFor(levelIdx: number): OverlayRole | null {
    const d = levelIdx - activeIdx;
    if (d === -1) return 'parent';
    if (d === 0) return 'active';
    if (d === 1) return 'child';
    return null;
  }

  for (const [i, carea] of page.careas.entries()) {
    const cr = roleFor(0);
    if (cr) items.push({id: carea.id, bbox: carea.bbox, role: cr, color: colorVars[0][i%8]});

    for (const [j, block] of carea.blocks.entries()) {
      const br = roleFor(1);
      if (br) items.push({id: block.id, bbox: block.bbox, role: br, color: colorVars[1][j%8]});

      for (const [k, line] of block.lines.entries()) {
        const lr = roleFor(2);
        if (lr) items.push({id: line.id, bbox: line.bbox, role: lr, color: colorVars[2][k%8]});

        for (const [l, word] of line.words.entries()) {
          const wr = roleFor(3);
          if (wr) items.push({id: word.id, bbox: word.bbox, role: wr, color: colorVars[3][l%8]});
        }
      }
    }
  }

  return items;
});

const cropLeft = computed(() => props.crop ? scanXPct(props.crop.left) : '0%');
const cropTop = computed(() => props.crop ? scanYPct(props.crop.top) : '0%');
const cropRight = computed(() => props.crop ? scanXPct(props.crop.right) : '0%');
const cropBottom = computed(() => props.crop ? scanYPct(props.crop.bottom) : '0%');

const cropAreaStyle = computed(() => ({
  position: 'absolute' as const,
  left: cropLeft.value,
  top: cropTop.value,
  right: cropRight.value,
  bottom: cropBottom.value,
  background: props.cropColor,
  outline: `2px solid ${props.cropColor}`,
  outlineOffset: '-1px',
  pointerEvents: 'none' as const,
}));
const topDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left: '0',
  top: '0',
  right: '0',
  height: cropTop.value,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));
const bottomDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left: '0',
  right: '0',
  bottom: '0',
  height: cropBottom.value,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));
const leftDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left: '0',
  top: cropTop.value,
  bottom: cropBottom.value,
  width: cropLeft.value,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));
const rightDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  right: '0',
  top: cropTop.value,
  bottom: cropBottom.value,
  width: cropRight.value,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));

function scanXPct(value: number): string {
  return props.page.scan_width > 0
      ? `${(value / props.page.scan_width) * 100}%`
      : '0%';
}

function scanYPct(value: number): string {
  return props.page.scan_height > 0
      ? `${(value / props.page.scan_height) * 100}%`
      : '0%';
}

function overlayItemStyle(item: OverlayItem) {
  const [l, t, r, b] = item.bbox;
  return {
    position: 'absolute' as const,
    left: scanXPct(l),
    top: scanYPct(t),
    width: scanXPct(r - l),
    height: scanYPct(b - t),
    '--hocr-color': item.color,
    background: item.role === 'parent' ? 'transparent' : item.color,
  };
}

</script>

<style scoped>
.page-preview {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg, #f8f9fa);
}

.page-preview-image-frame {
  min-width: 0;
  min-height: 0;
  flex: 1 1 auto;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  overflow: auto;
  padding: 1rem;
}

.page-preview-image-wrap {
  position: relative;
  max-width: 100%;
  flex: 0 0 auto;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.18);
  background: var(--color-surface, #fff);
}

.page-preview-image {
  display: block;
  max-width: 100%;
  height: auto;
  user-select: none;
}

.page-preview-info {
  flex: 0 0 auto;
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 0.35rem;
  min-height: 2rem;
  padding: 0.35rem 0.75rem;
  border-top: 1px solid var(--color-border, #dee2e6);
  background: var(--color-surface, #fff);
  color: var(--color-text, #212529);
  font-size: 0.85rem;
}

.page-preview-index {
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.8em;
}

.page-preview-unnamed {
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.preview-discard,
.preview-crop-area {
  box-sizing: border-box;
}

.hocr-overlay {
  box-sizing: border-box;
}

/* N-1: parent context — faint dashed outline, no fill, non-interactive */
.hocr-overlay--parent {
  pointer-events: none;
  outline: 2px dotted var(--hocr-color);
  outline-offset: 0.2rem;
  opacity: 0.85;
}

/* N: active level — solid outline + translucent fill */
.hocr-overlay--active {
  pointer-events: none;
  outline: 2px solid var(--hocr-color);
}

/* N+1: children — lighter fill, thin outline, selectable */
.hocr-overlay--child {
  pointer-events: auto;
  cursor: pointer;
  outline: 1px solid var(--hocr-color);
  outline-offset: -0.2rem;
  opacity: 0.75;
}

img {
  -webkit-user-select: none;
  user-select: none;
  -webkit-touch-callout: none;
}
</style>