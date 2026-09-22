<template>
  <div class="detection-tools">
    <sl-details label="Detection Thresholds" open>
      <div class="threshold-group">
        <sl-checkbox :checked="thresholds.use_x_indent" @sl-change="thresholds.use_x_indent = $event.target.checked; onThresholdChange()">X Indent</sl-checkbox>
        <div class="input-row">
          <sl-input type="number" size="small" label="Min" :value="thresholds.x_indent_min" :disabled="!thresholds.use_x_indent" @sl-change="thresholds.x_indent_min = parseInt($event.target.value); onThresholdChange()" />
          <sl-input type="number" size="small" label="Max" :value="thresholds.x_indent_max" :disabled="!thresholds.use_x_indent" @sl-change="thresholds.x_indent_max = parseInt($event.target.value); onThresholdChange()" />
        </div>
      </div>

      <div class="threshold-group">
        <sl-checkbox :checked="thresholds.use_x_dedent" @sl-change="thresholds.use_x_dedent = $event.target.checked; onThresholdChange()">X Dedent</sl-checkbox>
        <div class="input-row">
          <sl-input type="number" size="small" label="Min" :value="thresholds.x_dedent_min" :disabled="!thresholds.use_x_dedent" @sl-change="thresholds.x_dedent_min = parseInt($event.target.value); onThresholdChange()" />
          <sl-input type="number" size="small" label="Max" :value="thresholds.x_dedent_max" :disabled="!thresholds.use_x_dedent" @sl-change="thresholds.x_dedent_max = parseInt($event.target.value); onThresholdChange()" />
        </div>
      </div>

      <div class="threshold-group">
        <sl-checkbox :checked="thresholds.use_y_advance" @sl-change="thresholds.use_y_advance = $event.target.checked; onThresholdChange()">Y Advance</sl-checkbox>
        <div class="input-row">
          <sl-input type="number" size="small" label="Min" :value="thresholds.y_advance_min" :disabled="!thresholds.use_y_advance" @sl-change="thresholds.y_advance_min = parseInt($event.target.value); onThresholdChange()" />
          <sl-input type="number" size="small" label="Max" :value="thresholds.y_advance_max" :disabled="!thresholds.use_y_advance" @sl-change="thresholds.y_advance_max = parseInt($event.target.value); onThresholdChange()" />
        </div>
      </div>

      <div class="threshold-group">
        <sl-checkbox :checked="thresholds.use_hyphenation" @sl-change="thresholds.use_hyphenation = $event.target.checked; onThresholdChange()">Use Hyphenation</sl-checkbox>
      </div>
    </sl-details>

    <div class="button-group">
      <sl-button variant="primary" size="small" @click="onAutoBridgePage" :loading="hocrContext.loading.value">
        Auto detect page
      </sl-button>
      <sl-button size="small" @click="onAutoBridgeBlock" :disabled="!selectedBlockId" :loading="hocrContext.loading.value">
        Auto detect selected
      </sl-button>
    </div>

    <div v-if="selectedBlock" class="ocr-info-panel">
      <div class="ocr-info-row">
        <span class="ocr-info-label">ID</span>
        <span class="ocr-info-value ocr-info-id">{{ selectedBlock.id }}</span>
      </div>
      <div class="ocr-info-row">
        <span class="ocr-info-label">x_indent</span>
        <span class="ocr-info-value">{{ selectedBlock.metrics?.x_indent ?? '—' }}</span>
      </div>
      <div class="ocr-info-row">
        <span class="ocr-info-label">x_dedent</span>
        <span class="ocr-info-value">{{ selectedBlock.metrics?.x_dedent ?? '—' }}</span>
      </div>
      <div class="ocr-info-row">
        <span class="ocr-info-label">y_advance</span>
        <span class="ocr-info-value">{{ selectedBlock.metrics?.y_advance ?? '—' }}</span>
      </div>
      <div class="ocr-info-row">
        <span class="ocr-info-label">y_reverse</span>
        <span class="ocr-info-value">{{ selectedBlock.metrics?.y_reverse ?? '—' }}</span>
      </div>
      <div class="ocr-info-row">
        <span class="ocr-info-label">has_hyphen</span>
        <span class="ocr-info-value">{{ selectedBlock.metrics ? (selectedBlock.metrics.has_final_hyphen ? 'Yes' : 'No') : '—' }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed } from 'vue';
import { useHocrContext } from '../composables/useHocr';
import type { DetectionThresholds, HocrBlock } from '../types/hocr';
import { findItem } from '../types/hocr';

const props = defineProps<{
  selectedBlockId: string | null;
}>();

const hocrContext = useHocrContext();

const selectedBlock = computed(() => {
  if (!props.selectedBlockId || !hocrContext.hocrPage.value) return null;
  const item = findItem(hocrContext.hocrPage.value, props.selectedBlockId);
  if (item && item.level === 'block') return item as HocrBlock;
  return null;
});

const thresholds = reactive<DetectionThresholds>({
  x_indent_min: 5,
  x_indent_max: 15,
  x_dedent_min: 0,
  x_dedent_max: 20,
  y_advance_min: 0,
  y_advance_max: 0,
  use_x_indent: true,
  use_x_dedent: true,
  use_y_advance: true,
  use_hyphenation: true,
});

// Load from local storage if available
const STORAGE_KEY = 'omnigraph-detection-thresholds';
const saved = localStorage.getItem(STORAGE_KEY);
if (saved) {
  try {
    Object.assign(thresholds, JSON.parse(saved));
  } catch (e) {
    console.error('Failed to parse saved thresholds', e);
  }
}

function onThresholdChange() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(thresholds));
}

async function onAutoBridgePage() {
  await hocrContext.autoBridgePage(thresholds);
}

async function onAutoBridgeBlock() {
  if (props.selectedBlockId) {
    await hocrContext.autoBridgeBlock(props.selectedBlockId, thresholds);
  }
}
</script>

<style scoped>
.detection-tools {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem;
}

.threshold-group {
  margin-bottom: 0.5rem;
  padding: 0.5rem;
  border: 1px solid var(--sl-color-neutral-200);
  border-radius: var(--sl-border-radius-medium);
}

.input-row {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.button-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ocr-info-panel {
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 0.375rem;
  padding: 0.5rem;
  font-size: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  margin-top: 0.5rem;
}

.ocr-info-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.ocr-info-label {
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.ocr-info-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ocr-info-id {
  font-family: ui-monospace, monospace;
  color: var(--color-text-dimmed, #a2acb6);
}

sl-input {
  flex: 1;
}
</style>
