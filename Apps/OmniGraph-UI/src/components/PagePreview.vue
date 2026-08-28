<template>
  <div class="page-preview">
    <div class="page-preview-toolbar">
      <sl-checkbox size="small" :checked="showConfidence" @sl-change="showConfidence = $event.target.checked">Confidence</sl-checkbox>
    </div>
    <div
        class="interactive-area"
        :class="pointerVisible ? 'cursor-mode-off' : ''"
        @mousemove="updatePointerAction"
        @mouseenter="changePointerState(true)"
        @mouseleave="changePointerState(false)"
        @mousedown="handleMouseDown"
        @mouseup="handleMouseUp"
        @click="performPendingAction"
    >
      <!-- page / overlays / workspace content -->
      <div
          ref="imageFrameRef"
          class="page-preview-image-frame"
      >
        <div
            ref="imageWrapRef"
            class="page-preview-image-wrap"
        >
          <img id="scan-image"
               :src="src"
               class="page-preview-image"
               :alt="label"
               :title="label"
               draggable="false"
               @dragstart.prevent
          />

          <template v-if="showCropOverlay && crop">
            <div class="preview-discard" :style="topDiscardStyle"/>
            <div class="preview-discard" :style="bottomDiscardStyle"/>
            <div class="preview-discard" :style="leftDiscardStyle"/>
            <div class="preview-discard" :style="rightDiscardStyle"/>
            <div class="preview-crop-area" :style="cropAreaStyle"/>
          </template>

          <div v-for="item in overlayItems"
               :key="item.id"
               class="hocr-overlay"
               :class="[`hocr-overlay--${item.role}`, { 'hocr-overlay--selected': selectedItemIds?.has(item.id), 'hocr-overlay--indicated': item.id === indicatedItemId }]"
               :style="overlayItemStyle(item)"

          >
            <div class="hocr-overlay-item-info"
                 v-if="item.role === 'active'">
              <span class="hocr-overlay-item-kind"
                    v-if="item.kind">{{ item.kind }}</span>
              <span class="hocr-overlay-item-lang" v-if="item.lang">[{{ item.lang }}]</span>
              <span class="hocr-overlay-item-index">#{{ item.index }}</span>
              <span class="hocr-overlay-item-wconf" v-if="item.wconf != null">{{ item.wconf }}%</span>
              <span class="hocr-overlay-item-id">{{ item.id }}</span>
            </div>
          </div>

          <div v-if="isDragging && dragRectStyle" class="drag-rect" :style="dragRectStyle" />
        </div>

      </div>

      <CustomPointer
          :visible="pointerVisible"
          :enabled="pointerSettings?.enabled ?? true"
          :x="pointerX"
          :y="pointerY"
          :color="pointerSettings?.color ?? '#000000'"
          :icon="pointerSettings?.icon ?? ''"
          :label="pointerSettings?.label ?? ''"
      />
    </div>

    <div class="page-preview-info">
      <span class="page-preview-hint">(Index: {{ page.index }})</span>
      <span :class="{ 'page-preview-unnamed': !page.name }">p. {{ label }}</span>
      <span class="page-preview-hint">(Scan: {{ page.scan }})</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref, type Ref} from 'vue';
import { useHocrContext } from '../composables/useHocr';
import {
  type CropEdges,
  findItem,
  getChildren,
  getParentLevel,
  bboxContainsPoint,
  type HocrLevel,
  type HocrPage,
  type OverlayItem,
  type OverlayRole,
  type Page,
  type PageInteractionUpdate,
  type PointerSettings, type HocrCarea, type HocrBlock, type HocrLine, type HocrWord, findSiblingsAroundCursor,
  type HocrNode,
  type FlowSchema,
  type LayoutSchema
} from '../types';
import {makeVariedPalette, applyColorSpecs} from '../utils/colors';
import CustomPointer from "./CustomPointer.vue";

const LEVELS: HocrLevel[] = ['page', 'carea', 'block', 'line', 'word'];

const CONFIDENCE_COLOR = '255, 0, 0';
const CONFIDENCE_MIN_ALPHA = 0.08;
const CONFIDENCE_MAX_ALPHA = 0.4;

function getMinWconf(node: HocrNode): number {
  if ('words' in node) return node.words.length > 0 ? Math.min(...node.words.map(w => w.wconf)) : 100;
  if ('lines' in node) return node.lines.length > 0 ? Math.min(...node.lines.map(getMinWconf)) : 100;
  if ('blocks' in node) return node.blocks.length > 0 ? Math.min(...node.blocks.map(getMinWconf)) : 100;
  if ('wconf' in node) return node.wconf;
  return 100;
}

const props = withDefaults(defineProps<{
  page: Page;
  imageBaseUrl: string;
  crop?: CropEdges;
  showCropOverlay?: boolean;
  cropColor?: string;
  discardColor?: string;
  hocrLevel?: HocrLevel | null;
  careaOverlayColor?: string;
  blockOverlayColor?: string;
  lineOverlayColor?: string;
  wordOverlayColor?: string;
  pointerSettings?: PointerSettings;
  interactionUpdate?: PageInteractionUpdate;
  interactionClick?: () => void;
  interactionDrag?: (x1: number, y1: number, x2: number, y2: number) => void;
  flows?: Record<string, FlowSchema>;
  layouts?: Record<string, LayoutSchema>;
  careaLayers?: { flow: boolean; layout: boolean };
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

const { hocrPage } = useHocrContext();
const selectedItemIds = inject<Ref<Set<string>>>('selectedItemIds',   ref(new Set()));
const indicatedItemId = inject<Ref<string | null>>('indicatedItemId', ref(null));

const label = computed(() => props.page.name || props.page.scan);
const src = computed(() => props.imageBaseUrl + props.page.scan);

const imageFrameRef = ref<HTMLElement | null>(null);
const imageWrapRef = ref<HTMLElement | null>(null);

const pointerVisible = ref(false);
const pointerX = ref(0);
const pointerY = ref(0);
const showConfidence = ref(false);

// ── Drag-to-draw state ───────────────────────────────────────────────
const dragStart   = ref<{x: number; y: number} | null>(null);
const dragCurrent = ref<{x: number; y: number} | null>(null);
const isDragging    = ref(false);
const wasJustDragging = ref(false);

const dragRectStyle = computed(() => {
  if (!isDragging.value || !dragStart.value || !dragCurrent.value) return null;
  const x1 = Math.min(dragStart.value.x, dragCurrent.value.x);
  const y1 = Math.min(dragStart.value.y, dragCurrent.value.y);
  const x2 = Math.max(dragStart.value.x, dragCurrent.value.x);
  const y2 = Math.max(dragStart.value.y, dragCurrent.value.y);
  return {
    position: 'absolute' as const,
    left: scanXPct(x1),
    top: scanYPct(y1),
    width: scanXPct(x2 - x1),
    height: scanYPct(y2 - y1),
    pointerEvents: 'none' as const,
    zIndex: 10,
  };
});

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
  const items: OverlayItem[] = [];

  function colorFor(levelIdx: number, n: number, role: OverlayRole): string {
    if (role == 'child')
      return colorVars[levelIdx][n % 8];
    else
      return colorByLevel.value[levelIdx];
  }

  function roleFor(levelIdx: number): OverlayRole | null {
    const d = levelIdx - activeIdx;
    if (d === -1) return 'parent';
    if (d === 0) return 'active';
    if (d === 1) return 'child';
    if (levelIdx === 1 && (props.careaLayers?.flow || props.careaLayers?.layout)) return 'parent';
    return null;
  }

  function blockKindFor(block: HocrBlock): string {
    if(block.kind == 'part') return 'Part'
    if(block.kind == 'chapter') return 'H1'
    if(block.kind == 'section') return 'H2'
    if(block.kind == 'subsection') return 'H3'
    if(block.kind == 'subsubsection') return 'H4'
    if(block.kind == 'subsubsubsection') return 'H5'
    if(block.kind == 'subsubsubsubsection') return 'H6'
    if(block.kind == 'image') return 'IMG'
    if(block.kind == 'table') return 'TBL'
    if(block.kind == 'list') return 'LST'
    return 'P'
  }

  for (const [i, carea] of page.careas.entries()) {
    const cr = roleFor(1);
    if (cr) {
      let careaColor = colorFor(0, i, cr);
      if (props.careaLayers?.flow || props.careaLayers?.layout) {
        const specs = [];
        if (props.careaLayers?.flow && carea.flow && props.flows?.[carea.flow]) {
          const f = props.flows[carea.flow];
          if (f.color) specs.push(f.color);
        }
        if (props.careaLayers?.layout && carea.layout && props.layouts?.[carea.layout]) {
          const l = props.layouts[carea.layout];
          if (l.color) specs.push(l.color);
        }
        careaColor = applyColorSpecs(careaColor, specs);
      }

      items.push({
        id: carea.id,
        level: 'carea',
        index: i,
        bbox: carea.bbox,
        role: cr,
        color: careaColor,
        kind: null,
        wconf: getMinWconf(carea)
      });
    }

    for (const [j, block] of carea.blocks.entries()) {
      const br = roleFor(2);
      if (br) items.push({
        id: block.id,
        level: 'block',
        index: j,
        bbox: block.bbox,
        role: br,
        color: colorFor(1, j, br),
        kind: blockKindFor(block),
        wconf: getMinWconf(block),
      });

      for (const [k, line] of block.lines.entries()) {
        const lr = roleFor(3);
        if (lr) items.push({
          id: line.id,
          level: 'line',
          index: k,
          bbox: line.bbox,
          role: lr,
          color: colorFor(2, k, lr),
          kind: null,
          wconf: getMinWconf(line),
        });

        for (const [l, word] of line.words.entries()) {
          const wr = roleFor(4);
          if (wr) items.push({
            id: word.id,
            level: 'word',
            index: l,
            bbox: word.bbox,
            role: wr,
            color: colorFor(3, l, wr),
            kind: null,
            lang: word.lang,
            wconf: word.wconf,
          });
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

function changePointerState(inside: boolean) {
  pointerVisible.value = props.pointerSettings ? inside : false;
  if (!inside) {
    dragStart.value = null;
    dragCurrent.value = null;
    isDragging.value = false;
  }
}

function handleMouseDown(e: MouseEvent) {
  if (!props.interactionDrag) return;
  const point = getScanPointForEvent(e);
  if (point) {
    dragStart.value = point;
    dragCurrent.value = point;
    isDragging.value = false;
  }
}

function handleMouseUp(e: MouseEvent) {
  if (isDragging.value && dragStart.value && props.interactionDrag) {
    const point = getScanPointForEvent(e);
    if (point) {
      props.interactionDrag(dragStart.value.x, dragStart.value.y, point.x, point.y);
    }
    wasJustDragging.value = true;
  }
  dragStart.value = null;
  dragCurrent.value = null;
  isDragging.value = false;
}

function updateDragState(event: MouseEvent) {
  // Update drag rect while dragging
  if (dragStart.value) {
    const point = getScanPointForEvent(event);
    if (point) {
      dragCurrent.value = point;
      const dx = point.x - dragStart.value.x;
      const dy = point.y - dragStart.value.y;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) isDragging.value = true;
    }
  }
}

function updatePointerAction(event: MouseEvent) {

  pointerX.value = event.clientX;
  pointerY.value = event.clientY;

  updateDragState(event);

  if (!hocrPage?.value) return;
  if (!props.interactionUpdate) return;

  const page: HocrPage = hocrPage.value

  if (props.interactionUpdate) {
    const pagePoint = getScanPointForEvent(event);
    if (!pagePoint)
      return;

    if (props.hocrLevel == null) {

      if (props.interactionUpdate) {
        props.interactionUpdate(pagePoint.x, pagePoint.y, [], null, [null, null], [null, null]);
      }

    } else {

      const hitItems = overlayItems.value.filter(item =>
          item.level === props.hocrLevel && bboxContainsPoint(item.bbox, pagePoint.x, pagePoint.y)
      );

      let active: HocrNode | null = null;
      if (hitItems.length == 1)
        active = findItem(page, hitItems[0].id);

      let siblings: (HocrCarea | HocrBlock | HocrLine | HocrWord)[] = []

      // If in block, line or word mode, we should search inside the parent item.
      if (props.hocrLevel != null && props.hocrLevel !== 'carea') {

        const parentLevel = getParentLevel(props.hocrLevel);
        const inWhichParent = overlayItems.value.filter(item =>
            bboxContainsPoint(item.bbox, pagePoint.x, pagePoint.y) && item.level === parentLevel
        );

        if (inWhichParent.length > 0) {
          const parent = findItem(page, inWhichParent[0].id);
          if (parent) {
            siblings = getChildren(parent);
          }
        }
      } else {
        siblings = hocrPage.value.careas;
      }

      let betweenSiblings = findSiblingsAroundCursor(
          siblings,
          pagePoint.x,
          pagePoint.y,
          8,
      );

      let childrenElements = active ? getChildren(active) : [];

      let betweenSubSiblings = findSiblingsAroundCursor(
          childrenElements,
          pagePoint.x,
          pagePoint.y,
          8,
      )

      props.interactionUpdate(
          pagePoint.x,
          pagePoint.y,
          hitItems,
          active,
          betweenSiblings,
          betweenSubSiblings,
      );
    }
  }
}

function performPendingAction() {
  if (wasJustDragging.value) {
    wasJustDragging.value = false;
    return;
  }
  props.interactionClick?.();
}

function getScanPointForEvent(event: MouseEvent): { x: number; y: number } | null {
  const imageWrap = imageWrapRef.value;
  const imageFrame = imageFrameRef.value;

  if (!imageWrap || !imageFrame) return null;

  const frameRect = imageFrame.getBoundingClientRect();

  const clientX = clamp(event.clientX, frameRect.left, frameRect.right);
  const clientY = clamp(event.clientY, frameRect.top, frameRect.bottom);

  const imageRect = imageWrap.getBoundingClientRect();

  const imageX = clamp(clientX - imageRect.left, 0, imageRect.width);
  const imageY = clamp(clientY - imageRect.top, 0, imageRect.height);

  const scanX = imageRect.width > 0
      ? Math.round((imageX / imageRect.width) * props.page.scan_width)
      : 0;

  const scanY = imageRect.height > 0
      ? Math.round((imageY / imageRect.height) * props.page.scan_height)
      : 0;

  return {
    x: clamp(scanX, 0, props.page.scan_width),
    y: clamp(scanY, 0, props.page.scan_height),
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

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
  const style: any = {
    position: 'absolute' as const,
    left: scanXPct(l),
    top: scanYPct(t),
    width: scanXPct(r - l),
    height: scanYPct(b - t),
    '--hocr-color': item.color,
  };

  if (item.role === 'active') {
    const isHighLevel = item.level === 'carea' || item.level === 'block';
    if (isHighLevel) {
      style.background = `color-mix(in srgb, ${item.color} 15%, transparent)`;
      style.outline = `2px solid ${item.color}`;
      style['--info-display'] = 'inline-flex';
    } else {
      style.background = 'transparent';
      style.outline = `1px solid color-mix(in srgb, ${item.color} 25%, transparent)`;
      style['--info-display'] = 'none';
    }
  } else if (item.role === 'parent') {
    const isCareaHighlight = item.level === 'carea' && (props.careaLayers?.flow || props.careaLayers?.layout);
    if (isCareaHighlight) {
      const isLineOrWordMode = props.hocrLevel === 'line' || props.hocrLevel === 'word';
      style.background = isLineOrWordMode ? 'transparent' : `color-mix(in srgb, ${item.color} 15%, transparent)`;
      style.outline = `1px dotted color-mix(in srgb, ${item.color} 25%, transparent)`;
      const isImmediateParent = props.hocrLevel && LEVELS.indexOf(item.level) === LEVELS.indexOf(props.hocrLevel) - 1;
      if (!isImmediateParent) {
        style.pointerEvents = 'none';
      }
    } else {
      style.background = 'transparent';
      style.outline = `1px dotted color-mix(in srgb, ${item.color} 25%, transparent)`;
    }
  } else {
    style.background = 'transparent';
  }

  if (showConfidence.value && item.role === 'active' && item.wconf !== undefined) {
    const rawAlpha = (1 - item.wconf / 100) * CONFIDENCE_MAX_ALPHA;
    const alpha = item.wconf < 100 ? Math.max(rawAlpha, CONFIDENCE_MIN_ALPHA) : 0;
    style.background = `rgba(${CONFIDENCE_COLOR}, ${alpha})`;
  }

  return style;
}


</script>

<style scoped>

.interactive-area {
  position: relative;
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.cursor-mode-off {
  cursor: none;
}

.page-preview-toolbar {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  padding: 0.1rem 0.75rem;
  background: var(--color-surface, #fff);
  border-bottom: 1px solid var(--color-border, #dee2e6);
  font-size: 0.85rem;
  min-height: 1.5rem;
}

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
  min-height: 1.2rem;
  padding: 0.35rem 0.75rem;
  border-top: 1px solid var(--color-border, #dee2e6);
  background: var(--color-surface, #fff);
  color: var(--color-text, #212529);
  font-size: 0.85rem;
}

.page-preview-hint {
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

.hocr-overlay-item-info {
  position: absolute;
  right: 100%;
  top: -0.25rem;
  z-index: 2;
  max-width: calc(100% - 0.5rem);
  display: var(--info-display, none);
  align-items: center;
  gap: 0.25rem;
  padding: 0.15rem 0.4rem;
  border: 1px solid color-mix(in srgb, var(--hocr-color) 75%, white);
  border-radius: 999px;
  background: color-mix(in srgb, var(--hocr-color) 88%, black);
  box-shadow: 0 1px 4px rgba(15, 23, 42, 0.25);
  color: white;
  font-size: 0.65rem;
  font-weight: 700;
  line-height: 1.2;
  text-align: center;
  white-space: nowrap;
  pointer-events: none;
}

.hocr-overlay-item-kind {
  font-size: 0.65rem;
  font-weight: 800;
  line-height: 1.2;
}

.hocr-overlay-item-index {
  flex: 0 0 auto;
  opacity: 0.85;
  font-weight: 400;
}

.hocr-overlay-item-id {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  font-size: 0.55rem;
  font-weight: 400;
}

.hocr-overlay-item-lang {
  opacity: 0.7;
  font-weight: 400;
}

.hocr-overlay-item-wconf {
  color: #fbbf24;
}

/* N-1: parent context — faint dashed outline, no fill, interactive on hover */
.hocr-overlay--parent {
  outline: 2px dotted var(--hocr-color);
  outline-offset: 0.2rem;
  opacity: 1;
}

.hocr-overlay--parent:hover {
  outline: 2px dotted var(--hocr-color) !important;
  background: color-mix(in srgb, var(--hocr-color) 10%, transparent) !important;
  opacity: 1 !important;
}

/* N: active level — solid outline + translucent fill */
.hocr-overlay--active {
  outline: 2px solid var(--hocr-color);
  background: color-mix(in srgb, var(--hocr-color) 15%, transparent);
  opacity: 1;
}

/* N: active level — solid outline + translucent fill */
.hocr-overlay--active:hover {
  outline: 2px solid var(--hocr-color) !important;
  background: color-mix(in srgb, var(--hocr-color) 45%, transparent) !important;
}

.hocr-overlay:hover > .hocr-overlay-item-info,
.hocr-overlay--indicated > .hocr-overlay-item-info {
  display: inline-flex !important;
}

/* Selected item — stronger outline + hover-level fill, stays regardless of hover */
.hocr-overlay--selected {
  outline: 3px solid var(--hocr-color) !important;
  background: color-mix(in srgb, var(--hocr-color) 45%, transparent) !important;
  opacity: 1 !important;
}

/* Indicated item — outline-only highlight from outline hover, no fill */
.hocr-overlay--indicated {
  outline: 2px dashed var(--hocr-color) !important;
  opacity: 1 !important;
}

/* N+1: children — lighter fill, thin outline, selectable */
.hocr-overlay--child {
  pointer-events: none;
  outline: 1px solid var(--hocr-color);
  outline-offset: -0.2rem;
  opacity: 0.35;
  transition: opacity 120ms ease;
}


.drag-rect {
  box-sizing: border-box;
  border: 2px dashed rgba(120, 202, 61, 0.9);
  background: rgba(120, 202, 61, 0.12);
}

img {
  -webkit-user-select: none;
  user-select: none;
  -webkit-touch-callout: none;
}
</style>