<template>
  <div class="strip-wrapper" :class="{ 'strip-selected': selected }">
    <div class="strip-container" :style="containerStyle">
      <img :src="src" class="strip-img" :style="imgStyle" :alt="label" :title="label" />
      <template v-if="showOverlay">
        <div class="ov-red"   :style="topRedStyle"    />
        <div class="ov-red"   :style="bottomRedStyle" />
        <div class="ov-red"   :style="leftRedStyle"   />
        <div class="ov-red"   :style="rightRedStyle"  />
        <div class="ov-green" :style="greenStyle"     />
      </template>
    </div>
    <div class="strip-info">
      <span class="strip-label" :class="{ 'strip-label-unnamed': !props.page.name }">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Page, CropEdges } from '../types';

const props = withDefaults(defineProps<{
  page:        Page;
  edge:        string;
  thumbBaseUrl: string;
  fraction:    number;
  showOverlay: boolean;
  crop:        CropEdges;
  selected?:   boolean;
  cropColor?:  string;
  discardColor?: string;
}>(), {
  cropColor: 'rgba(0, 180, 0, 0.12)',
  discardColor: 'rgba(220, 0, 0, 0.35)',
});

const label = computed(() => props.page.name || props.page.scan);
const src   = computed(() => props.thumbBaseUrl + props.page.thumb);

// ── Container (the visible strip box) ──────────────────────────────

const containerStyle = computed(() => {
  const w = props.page.thumb_width;
  const h = props.page.thumb_height;
  const cw = (props.edge === 'left'  || props.edge === 'right')  ? Math.round(w * props.fraction) : w;
  const ch = (props.edge === 'top'   || props.edge === 'bottom') ? Math.round(h * props.fraction) : h;
  return { position: 'relative' as const, width: `${cw}px`, height: `${ch}px`, overflow: 'hidden', flexShrink: '0' };
});

// ── Image offset inside container ──────────────────────────────────
// Right/bottom edges: shift the image so the far side is visible.

const imgOffset = computed(() => {
  const w = props.page.thumb_width;
  const h = props.page.thumb_height;
  return {
    x: props.edge === 'right'  ? -Math.round(w * (1 - props.fraction)) : 0,
    y: props.edge === 'bottom' ? -Math.round(h * (1 - props.fraction)) : 0,
  };
});

const imgStyle = computed(() => ({
  position: 'absolute' as const,
  left: `${imgOffset.value.x}px`,
  top:  `${imgOffset.value.y}px`,
  width:  `${props.page.thumb_width}px`,
  height: `${props.page.thumb_height}px`,
  display: 'block',
}));

// ── Crop overlay ────────────────────────────────────────────────────
// Scale crop from scan pixels → thumbnail pixels, then offset by the
// same amount the image is offset so overlays stay aligned.

const sx = computed(() => props.page.scan_width  > 0 ? props.page.thumb_width  / props.page.scan_width  : 1);
const sy = computed(() => props.page.scan_height > 0 ? props.page.thumb_height / props.page.scan_height : 1);

const tl = computed(() => Math.round(props.crop.left   * sx.value));
const tt = computed(() => Math.round(props.crop.top    * sy.value));
const tr = computed(() => Math.round(props.crop.right  * sx.value));
const tb = computed(() => Math.round(props.crop.bottom * sy.value));

const iw = computed(() => props.page.thumb_width  - tl.value - tr.value);
const ih = computed(() => props.page.thumb_height - tt.value - tb.value);

const ox = computed(() => imgOffset.value.x);
const oy = computed(() => imgOffset.value.y);
const tw = computed(() => props.page.thumb_width);
const th = computed(() => props.page.thumb_height);

const greenStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value + tl.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${iw.value}px`,
  height: `${ih.value}px`,
  background: props.cropColor,
  outline: `2px solid ${props.cropColor}`,
  outlineOffset: '-1px',
  pointerEvents: 'none' as const,
}));

const topRedStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value}px`,
  width:  `${tw.value}px`,
  height: `${tt.value}px`,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));

const bottomRedStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value + th.value - tb.value}px`,
  width:  `${tw.value}px`,
  height: `${tb.value}px`,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));

const leftRedStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${tl.value}px`,
  height: `${ih.value}px`,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));

const rightRedStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value + tw.value - tr.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${tr.value}px`,
  height: `${ih.value}px`,
  background: props.discardColor,
  pointerEvents: 'none' as const,
}));
</script>

<style scoped>
.strip-wrapper {
  display: inline-flex;
  flex-direction: column;
  align-items: stretch;
  cursor: default;
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 2px;
  overflow: hidden;   /* clips the info section to the same width as the image strip */
}

.strip-wrapper:hover {
  border-color: var(--color-accent, #2563eb);
}

.strip-selected {
  outline: 2px solid var(--color-accent, #2563eb);
  outline-offset: 1px;
}

.strip-container {
  flex-shrink: 0;
}

/* Fixed-height info bar — same height on every card regardless of content.
   Designed to grow into a richer info section in future iterations. */
.strip-info {
  height: 1.4rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface, #fff);
  border-top: 1px solid var(--color-border, #dee2e6);
  padding: 0 3px;
  flex-shrink: 0;
}

.strip-label {
  font-size: 0.65rem;
  color: var(--color-text, #212529);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.strip-label-unnamed {
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.strip-img {
  user-select: none;
}
</style>
