<template>
  <div class="strip-wrapper" :class="{ 'strip-selected': selected }" ref="el">
    <div class="strip-container" :style="containerStyle">
      <img :src="src" class="strip-img" :style="imgStyle" :alt="label" :title="label" />

      <template v-if="showHints && page.hints">
        <div v-for="(hint, index) in page.hints"
             :key="index"
             class="hint-overlay"
             :class="`hint-overlay--${hint.type}`"
             :style="getHintStyle(hint)"
        />
      </template>

      <!-- OCR Overlays -->
      <template v-if="hocrData">
        <!-- CAREAs -->
        <template v-if="showLayers?.flow || showLayers?.layout">
          <div v-for="carea in hocrData.careas" :key="'carea-' + carea.id" :style="getCareaStyle(carea)" />
        </template>

        <!-- BLOCKs -->
        <template v-if="showBlocks">
          <template v-for="carea in hocrData.careas" :key="'blocks-' + carea.id">
            <div v-for="block in carea.blocks" :key="'block-' + block.id" :style="getBlockStyle(block)" />
          </template>
        </template>
      </template>

      <template v-if="showOverlay">
        <div :style="topDiscardStyle"    />
        <div :style="bottomDiscardStyle" />
        <div :style="leftDiscardStyle"   />
        <div :style="rightDiscardStyle"  />
        <div :style="keepStyle"          />
      </template>
    </div>
    <div class="strip-info">
      <span class="strip-label" :class="{ 'strip-label-unnamed': !props.page.name }">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import type { Page, CropEdges, FlowSchema, LayoutSchema, Hint, EditorPalette } from '../types';
import { DEFAULT_PALETTE } from '../types';
import type { HocrPage, HocrCarea, HocrBlock } from '../types/hocr';
import { fetchHocrPage } from '../composables/useHocr';
import { applyColorSpecs } from '../utils/colors';

const props = withDefaults(defineProps<{
  page:          Page;
  edge:          string;
  selected?:     boolean;
  isCurrent?:    boolean;
  fraction:      number;
  crop:          CropEdges;
  palette?:      EditorPalette;
  machineName?:  string;
  thumbBaseUrl:  string;
  flows?:        Record<string, FlowSchema>;
  layouts?:      Record<string, LayoutSchema>;
  hocrSyncData?: HocrPage | null;
  showLayers?:   { flow: boolean; layout: boolean };
  showOverlay:   boolean;
  showBlocks?:   boolean;
  showHints?:    boolean;
}>(), {
  palette: () => DEFAULT_PALETTE,
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

const keepStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value + tl.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${iw.value}px`,
  height: `${ih.value}px`,
  background: props.palette.keepColor,
  outline: `2px solid ${props.palette.keepColor}`,
  outlineOffset: '-1px',
  pointerEvents: 'none' as const,
}));

const topDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value}px`,
  width:  `${tw.value}px`,
  height: `${tt.value}px`,
  background: props.palette.discardColor,
  pointerEvents: 'none' as const,
}));

const bottomDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value + th.value - tb.value}px`,
  width:  `${tw.value}px`,
  height: `${tb.value}px`,
  background: props.palette.discardColor,
  pointerEvents: 'none' as const,
}));

const leftDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${tl.value}px`,
  height: `${ih.value}px`,
  background: props.palette.discardColor,
  pointerEvents: 'none' as const,
}));

const rightDiscardStyle = computed(() => ({
  position: 'absolute' as const,
  left:   `${ox.value + tw.value - tr.value}px`,
  top:    `${oy.value + tt.value}px`,
  width:  `${tr.value}px`,
  height: `${ih.value}px`,
  background: props.palette.discardColor,
  pointerEvents: 'none' as const,
}));

// ── OCR Overlays ────────────────────────────────────────────────────

const hocrData = ref<HocrPage | null>(null);
const el = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

function setHocrData(page: HocrPage | null) {
  if (!page) {
    hocrData.value = null;
    return;
  }
  // Memory-efficient storage: filter out lines and words
  hocrData.value = {
    ...page,
    careas: page.careas.map(c => ({
      ...c,
      blocks: c.blocks.map(b => ({
        ...b,
        lines: [] // Clear lines to save memory
      }))
    }))
  };
}

async function loadData() {
  if (!props.machineName) return;
  try {
    const stem = props.page.scan.replace(/\.[^/.]+$/, "");
    const fullPage = await fetchHocrPage(props.machineName, stem);
    setHocrData(fullPage);
  } catch (e) {
    console.error('Failed to load hOCR for strip:', e);
    hocrData.value = null;
  }
}

function clearData() {
  hocrData.value = null;
}

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting) {
      loadData();
    } else {
      clearData();
    }
  }, { rootMargin: '100% 0px' });

  if (el.value) {
    observer.observe(el.value);
  }
});

onUnmounted(() => {
  if (observer) {
    observer.disconnect();
  }
});

watch(() => props.isCurrent, (newVal) => {
  if (newVal) {
    loadData();
  }
});

watch(() => props.hocrSyncData, (newVal) => {
  if (props.isCurrent) {
    setHocrData(newVal ?? null);
  }
});

function getCareaStyle(carea: HocrCarea) {
  const [l, t, r, b] = carea.bbox;
  const wl = Math.round(l * sx.value);
  const wt = Math.round(t * sy.value);
  const wr = Math.round(r * sx.value);
  const wb = Math.round(b * sy.value);

  const specs = [];
  if (props.showLayers?.flow && carea.flow && props.flows?.[carea.flow]) {
    const f = props.flows[carea.flow];
    if (f.color) specs.push(f.color);
  }
  if (props.showLayers?.layout && carea.layout && props.layouts?.[carea.layout]) {
    const l = props.layouts[carea.layout];
    if (l.color) specs.push(l.color);
  }

  const background = applyColorSpecs(props.palette.careaOverlayColor, specs);

  return {
    position: 'absolute' as const,
    left: `${ox.value + wl}px`,
    top: `${oy.value + wt}px`,
    width: `${wr - wl}px`,
    height: `${wb - wt}px`,
    background,
    opacity: 0.5,
    mixBlendMode: 'multiply' as const,
    pointerEvents: 'none' as const,
  };
}

function getBlockStyle(block: HocrBlock) {
  const [l, t, r, b] = block.bbox;
  const wl = Math.round(l * sx.value);
  const wt = Math.round(t * sy.value);
  const wr = Math.round(r * sx.value);
  const wb = Math.round(b * sy.value);

  return {
    position: 'absolute' as const,
    left: `${ox.value + wl}px`,
    top: `${oy.value + wt}px`,
    width: `${wr - wl}px`,
    height: `${wb - wt}px`,
    outline: `2px solid ${props.palette.blockOverlayColor}`,
    opacity: 0.5,
    mixBlendMode: 'normal' as const,
    pointerEvents: 'none' as const,
  };
}

function getHintStyle(hint: Hint) {
  const wl = Math.round(hint.area.left * sx.value);
  const wt = Math.round(hint.area.top * sy.value);
  const wr = Math.round(hint.area.right * sx.value);
  const wb = Math.round(hint.area.bottom * sy.value);

  return {
    position: 'absolute' as const,
    left: `${ox.value + wl}px`,
    top: `${oy.value + wt}px`,
    width: `${wr - wl}px`,
    height: `${wb - wt}px`,
    pointerEvents: 'none' as const,
  };
}
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

.hint-overlay {
  box-sizing: border-box;
}

.hint-overlay--dropcap {
  background: rgba(255, 140, 0, 0.4);
  border: 1px solid rgb(255, 140, 0);
}

.hint-overlay--image {
  background: rgba(0, 191, 255, 0.4);
  border: 1px solid rgb(0, 191, 255);
}

.hint-overlay--callout {
  background: rgba(40, 167, 69, 0.4);
  border: 1px solid rgb(40, 167, 69);
}

.hint-overlay--garbage {
  background: rgba(220, 53, 69, 0.4);
  border: 1px solid rgb(220, 53, 69);
}
</style>
