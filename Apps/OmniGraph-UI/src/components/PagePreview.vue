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
          <div class="preview-discard preview-discard-top" :style="topDiscardStyle" />
          <div class="preview-discard preview-discard-bottom" :style="bottomDiscardStyle" />
          <div class="preview-discard preview-discard-left" :style="leftDiscardStyle" />
          <div class="preview-discard preview-discard-right" :style="rightDiscardStyle" />
          <div class="preview-crop-area" :style="cropAreaStyle" />
        </template>
      </div>
    </div>

    <div class="page-preview-info">
      <span class="page-preview-index">({{ page.index }})</span>
      <span :class="{ 'page-preview-unnamed': !page.name }">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { CropEdges, Page } from '../types';

const props = withDefaults(defineProps<{
  page: Page;
  imageBaseUrl: string;
  crop?: CropEdges;
  showCropOverlay?: boolean;
  cropColor?: string;
  discardColor?: string;
}>(), {
  showCropOverlay: true,
  cropColor: 'rgba(0, 180, 0, 0.12)',
  discardColor: 'rgba(220, 0, 0, 0.28)',
});

const label = computed(() => props.page.name || props.page.scan);
const src = computed(() => props.imageBaseUrl + props.page.scan);

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
</style>