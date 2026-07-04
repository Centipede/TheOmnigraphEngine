<template>
  <PageWorkspace :machine-name="machineName"
                 :project-name="projectName"
                 :format-page-extras="formatPageExtras">

    <template #tools="{ selectionInfo, filteredPages }">
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

import {onMounted, onUnmounted, ref} from 'vue';
import type {Page} from '../types';

const props = defineProps<{ machineName: string; projectName: string }>();

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
const scanResults = ref<ScanPageResult[]>([]);
const scanError = ref('');


// ── Scan ─────────────────────────────────────────────────────────────
async function scanPages(pagesToScan: Page[], force = false): Promise<void> {
  const indices = pagesToScan.map(p => p.index);
  if (!indices.length) return;

  isScanning.value = true;
  scanError.value = '';
  if (!force) scanResults.value = [];

  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/scan`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({indices, force}),
    });

    if (resp.status === 409) {
      const conflict = await resp.json() as ScanConflict;
      isScanning.value = false;
      const msg = `${conflict.pages.length} page(s) have unsaved edits:\n${conflict.pages.join(', ')}\n\nRescanning will overwrite their originals. Continue?`;
      if (window.confirm(msg)) await scanPages(pagesToScan, true);
      return;
    }

    if (!resp.ok) {
      const err = await resp.json() as { error: string };
      scanError.value = err.error ?? 'Scan failed.';
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
  await fetchHocrStatus();
  hocrStatusInterval = setInterval(() => {
    void fetchHocrStatus();
  }, 30_000);
});

onUnmounted(() => {
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
</style>
