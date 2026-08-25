<template>
  <PageWorkspace :machine-name="machineName"
                 :project-name="projectName"
                 :initial-page-stem="initialPageStem"
                 :panels="panels"
                 hocr-level="block"
                 :page-list-columns="['name-or-scan', 'extras']"
                 :format-page-extras="formatPageExtras"
                 :show-crop-overlay="true"
                 :flows="flows"
                 :layouts="layouts"
  >

    <template #tools="{ selectionInfo, filteredPages, currentPage }">
      <div v-if="currentPage" class="crop-info-panel info-panel">
        <div class="info-row">
          <span class="info-label">Crop Top</span>
          <span class="info-value">{{ currentPage.crop_edges.top }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Crop Left</span>
          <span class="info-value">{{ currentPage.crop_edges.left }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Crop Right</span>
          <span class="info-value">{{ currentPage.crop_edges.right }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Crop Bottom</span>
          <span class="info-value">{{ currentPage.crop_edges.bottom }}</span>
        </div>
      </div>
      <sl-input
          label="OCR Language"
          size="small"
          :value="ocrLanguage"
          @sl-change="ocrLanguage = ($event.target as HTMLInputElement).value"
          style="margin-bottom: 0.5rem;"
      ></sl-input>
      <sl-button
          size="small"
          variant="primary"
          :loading="isScanning"
          :disabled="isScanning || !selectionInfo"
          @click="scanPages(filteredPages)"
      >
        Scan selected
      </sl-button>

      <p v-if="scanError" class="scan-error">{{ scanError }}</p>

      <div v-if="scanResults.length" class="scan-results">
        <div
            v-for="r in scanResults"
            :key="r.scan"
            class="scan-result-row"
            :class="r.success ? 'ok' : 'fail'"
        >
          <span class="scan-result-icon">{{ r.success ? '✓' : '✗' }}</span>
          <span class="scan-result-name">{{ r.scan }}</span>
          <span v-if="r.error" class="scan-result-error">{{ r.error }}</span>
        </div>
      </div>
    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from "../components/PageWorkspace.vue";

import {onMounted, onUnmounted, ref, inject, computed} from 'vue';
import type {Page, Project} from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

const panels = usePersistentPanels('panels.recognise', {
  'page-list': true,
  'page-strips': true,
  'page-preview': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();
const showError = inject<(msg: string) => void>('showError');

provideHocrContext();

// ── hOCR status ──────────────────────────────────────────────────────
const hocrScanned = ref<Set<string>>(new Set());

function formatPageExtras(pages: Page[]): Map<number, string> {
  const map = new Map<number, string>();
  for (const page of pages) {
    if (hocrScanned.value.has(page.scan)) map.set(page.index, 'hOCR');
  }
  return map;
}

async function fetchHocrStatus(): Promise<void> {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/hocr-status`);
    if (resp.ok) {
      const data = await resp.json() as { scanned: string[] };
      hocrScanned.value = new Set(data.scanned);
    }
  } catch (e) {
    console.error('Failed to fetch hOCR status:', e);
  }
}

// ── Scan state ───────────────────────────────────────────────────────
interface ScanPageResult {
  scan: string;
  success: boolean;
  error?: string
}

interface ScanConflict {
  pages: string[]
}

const isScanning = ref(false);
const ocrLanguage = ref('eng');
const scanResults = ref<ScanPageResult[]>([]);
const scanError = ref('');
const project = ref<Project | null>(null);
const flows = computed(() => project.value?.flows || []);
const layouts = computed(() => project.value?.layouts || []);

async function fetchProjectMetadata(): Promise<void> {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      const data = await resp.json() as Project;
      project.value = data;
      if (data.ocr_language) {
        ocrLanguage.value = data.ocr_language;
      }
    }
  } catch (e) {
    console.error('Failed to fetch project metadata:', e);
  }
}


// ── Scan ─────────────────────────────────────────────────────────────
async function scanPages(pagesToScan: Page[], force = false): Promise<void> {
  const indices = pagesToScan.map(p => p.index);
  if (!indices.length) return;

  const language = ocrLanguage.value;

  isScanning.value = true;
  scanError.value = '';
  if (!force) scanResults.value = [];

  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/scan`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({indices, force, language}),
    });

    if (resp.status === 409) {
      const conflict = await resp.json() as ScanConflict;
      isScanning.value = false;
      const msg = `${conflict.pages.length} page(s) have unsaved edits:\n${conflict.pages.join(', ')}\n\nRescanning will overwrite their originals. Continue?`;
      if (window.confirm(msg)) await scanPages(pagesToScan, true);
      return;
    }

    if (!resp.ok) {
      const text = await resp.text();
      let errorMsg = text;
      try {
        const json = JSON.parse(text);
        if (json.error) errorMsg = json.error;
      } catch (e) {
        // Not JSON
      }
      const finalMsg = errorMsg || `Scan failed: ${resp.statusText}`;
      scanError.value = finalMsg;
      showError?.(finalMsg);
      return;
    }

    const data = await resp.json() as { results: ScanPageResult[] };
    scanResults.value = data.results;
    void fetchHocrStatus();
  } catch (e) {
    console.error(e);
    scanError.value = 'Network error.';
  } finally {
    isScanning.value = false;
  }
}

let hocrStatusInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  setActivePanels(panels);
  void fetchProjectMetadata();
  await fetchHocrStatus();
  hocrStatusInterval = setInterval(() => {
    void fetchHocrStatus();
  }, 30_000);
});

onUnmounted(() => {
  setActivePanels(null);
  if (hocrStatusInterval !== null) clearInterval(hocrStatusInterval);
});
</script>

<style>

/* Scan results */
.scan-error {
  color: #dc2626;
  font-size: 0.8rem;
  margin: 0.5rem 0 0;
}

.scan-results {
  margin-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
}

.scan-result-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.scan-result-icon {
  font-weight: 700;
  flex-shrink: 0;
}

.scan-result-row.ok .scan-result-icon {
  color: #16a34a;
}

.scan-result-row.fail .scan-result-icon {
  color: #dc2626;
}

.scan-result-name {
  color: var(--color-text-muted, #6c757d);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scan-result-error {
  color: #dc2626;
  font-size: 0.75rem;
  width: 100%;
  padding-left: 1rem;
}

.crop-info-panel {
  margin-bottom: 0.75rem;
}
</style>
