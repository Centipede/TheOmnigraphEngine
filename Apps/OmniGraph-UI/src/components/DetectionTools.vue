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
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue';
import { useHocrContext } from '../composables/useHocr';
import type { DetectionThresholds } from '../types/hocr';

const props = defineProps<{
  selectedBlockId: string | null;
}>();

const hocrContext = useHocrContext();

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

sl-input {
  flex: 1;
}
</style>
