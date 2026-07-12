<template>
  <div class="page-preview">
    <div
        class="interactive-area"
        :class="pointerVisible ? 'cursor-mode-off' : ''"
        @mousemove="updatePointerAction"
        @mouseenter="pointerVisible = true"
        @mouseleave="pointerVisible = false"
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
          />

          <template v-if="showCropOverlay && crop">
            <div class="preview-discard" :style="topDiscardStyle"/>
            <div class="preview-discard" :style="bottomDiscardStyle"/>
            <div class="preview-discard" :style="leftDiscardStyle"/>
            <div class="preview-discard" :style="rightDiscardStyle"/>
            <div class="preview-crop-area" :style="cropAreaStyle"/>
          </template>

          <div id="hocr-overlay-items"
               v-for="item in overlayItems"
               :key="item.id"
               class="hocr-overlay"
               :class="`hocr-overlay--${item.role}`"
               :style="overlayItemStyle(item)"
          />


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
      <span class="page-preview-index">({{ page.index }})</span>
      <span :class="{ 'page-preview-unnamed': !page.name }">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref, type Ref} from 'vue';
import {
  type CropEdges,
  findItem,
  getChildren,
  getParentLevel,
  bboxContainsPoint,
  type HocrOverlayLevel,
  type HocrPage,
  type OverlayItem,
  type OverlayRole,
  type Page,
  type PageInteractionUpdate,
  type PointerSettings, type HocrCarea, type HocrBlock, type HocrLine, type HocrWord, findSiblingsAroundCursor,
  type HocrSibling
} from '../types';
import {makeVariedPalette} from '../utils/colors';
import CustomPointer from "./CustomPointer.vue";


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
  pointerSettings?: PointerSettings;
  interactionUpdate?: PageInteractionUpdate;
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

const imageFrameRef = ref<HTMLElement | null>(null);
const imageWrapRef = ref<HTMLElement | null>(null);

const pointerVisible = ref(false);
const pointerX = ref(0);
const pointerY = ref(0);

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
    return null;
  }

  for (const [i, carea] of page.careas.entries()) {
    const cr = roleFor(0);
    if (cr) items.push({id: carea.id, level: 'carea', bbox: carea.bbox, role: cr, color: colorFor(0, i, cr)});

    for (const [j, block] of carea.blocks.entries()) {
      const br = roleFor(1);
      if (br) items.push({id: block.id, level: 'block', bbox: block.bbox, role: br, color: colorFor(1, j, br)});

      for (const [k, line] of block.lines.entries()) {
        const lr = roleFor(2);
        if (lr) items.push({id: line.id, level: 'line', bbox: line.bbox, role: lr, color: colorFor(2, k, lr)});

        for (const [l, word] of line.words.entries()) {
          const wr = roleFor(3);
          if (wr) items.push({id: word.id, level: 'word', bbox: word.bbox, role: wr, color: colorFor(3, l, wr)});
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

function updatePointerAction(event: MouseEvent) {
  if(hocrPage === undefined || hocrPage.value === null)
    return;

  const page:HocrPage = hocrPage.value

  pointerX.value = event.clientX;
  pointerY.value = event.clientY;

  if (props.interactionUpdate) {
    const pagePoint = getScanPointForEvent(event);

    if (pagePoint) {
      const overlappingOverlayItems = overlayItems.value.filter(item =>
          bboxContainsPoint(item.bbox, pagePoint.x, pagePoint.y) && item.level === props.hocrLevel
      );

      let active : HocrSibling | null = null;
      if (overlappingOverlayItems.length == 1)
        active = findItem(page, overlappingOverlayItems[0].id);

      let siblings : (HocrCarea | HocrBlock | HocrLine | HocrWord)[] = []

      // If in block, line or word mode, we should search inside the parent item.
      if (props.hocrLevel != null && props.hocrLevel !== 'carea') {

        const parentLevel = getParentLevel(props.hocrLevel);
        const inWhichParent = overlayItems.value.filter(item =>
            bboxContainsPoint(item.bbox, pagePoint.x, pagePoint.y) && item.level === parentLevel
        );

        if (inWhichParent.length > 0) {
          const parent = findItem(page, inWhichParent[0].id);
          if(parent) {
            siblings = getChildren(parent);
          }
        }
      }
      else {
        siblings = hocrPage.value.careas;
      }

      let betweenSiblings = findSiblingsAroundCursor(
          siblings,
          pagePoint.x,
          pagePoint.y,
          8,
      );

      let childrenElements = active? getChildren(active): [];

      let betweenSubSiblings = findSiblingsAroundCursor(
          childrenElements,
          pagePoint.x,
          pagePoint.y,
          8,
      )

      props.interactionUpdate(
          pagePoint.x,
          pagePoint.y,
          overlappingOverlayItems,
          active,
          betweenSiblings,
          betweenSubSiblings,
      );

    }
  }

  //refreshPendingClickState(event);
}


function performPendingAction() {
  console.log('performPendingAction');
  // switch (pointerMode.value) {
  //   case 'select':
  //     console.log('select');
  //     break;
  //   case 'add':
  //     console.log('add');
  //     break;
  //   case 'remove':
  //     console.log('remove');
  //     break;
  //   case 'disabled':
  //     console.log('disabled');
  //     break;
  // }
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
  return {
    position: 'absolute' as const,
    left: scanXPct(l),
    top: scanYPct(t),
    width: scanXPct(r - l),
    height: scanYPct(b - t),
    '--hocr-color': item.color,
    background: item.role !== 'active' ? 'transparent' : item.color,
  };
}


</script>

<style scoped>

.interactive-area {
  position: relative;
}

.cursor-mode-off {
  cursor: none;
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
  outline: 2px solid var(--hocr-color);
  opacity: 0.25;
}

/* N+1: children — lighter fill, thin outline, selectable */
.hocr-overlay--child {
  pointer-events: none;
  outline: 1px solid var(--hocr-color);
  outline-offset: -0.2rem;
  opacity: 0.35;
  transition: opacity 120ms ease;
}


img {
  -webkit-user-select: none;
  user-select: none;
  -webkit-touch-callout: none;
}
</style>