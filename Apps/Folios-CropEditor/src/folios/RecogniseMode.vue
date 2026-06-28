<template>
  <div class="inspect-area">

    <!-- Sidebar: page list -->
    <PageList
        ref="pageListComponentRef"
        :pages="pages"
        :selected-page-indices="selectedPageSet"
        :current-page-index="currentPageIndex"
        :is-page-in-filter="isInFilter"
        @navigate="navigatePage"
        @page-click="handleListClick"
    />

    <!-- Central: strip grid -->
    <div class="inspect-workarea" :ref="setStripWorkareaRef">
      <div class="strip-grid">
        <PageStrip
            v-for="page in visiblePages"
            :key="page.index"
            :data-page-idx="page.index"
            :page="page"
            :edge="`all`"
            :thumbBaseUrl="thumbBaseUrl"
            :fraction="1"
            :showOverlay="mode === 'crop'"
            :crop="pageCrops.get(page.index) ?? page.crop_edges"
            crop-color="rgba(0, 0, 0, 0.0)"
            discard-color="rgba(50, 50, 50, 0.35)"
            :selected="selectedPageSet.has(page.index)"
            @click="handleStripClick(page.index, $event)"
        />
      </div>
    </div>

    <!-- Tools -->
    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <br>
        <sl-radio-group label="Even/Odd pages" size="small" :value="filterMode" @sl-input="onFilterChange">
          <sl-radio-button value="all">All</sl-radio-button>
          <sl-radio-button value="even">Even</sl-radio-button>
          <sl-radio-button value="odd">Odd</sl-radio-button>
        </sl-radio-group>

        <template v-if="selectionInfo">
          <br>
          <div class="selection-info-panel">
            <div class="info-row">
              <span class="info-label">Range</span>
              <span class="info-value">{{ selectionInfo.firstName }} – {{ selectionInfo.lastName }}</span>
              <span class="info-count">({{ selectionInfo.count }})</span>
            </div>
            <div class="info-row">
              <span class="info-label">Center</span>
              <span class="info-value">{{ selectionInfo.centerName }}</span>
            </div>
            <sl-button-group>
              <sl-button size="small" @click="focusPage(selectionInfo.firstIdx)">First</sl-button>
              <sl-button size="small" @click="focusPage(selectionInfo.centerIdx)">Center</sl-button>
              <sl-button size="small" @click="focusPage(selectionInfo.lastIdx)">Last</sl-button>
            </sl-button-group>
          </div>
        </template>

        <br>
        <sl-button
            size="small"
            variant="primary"
            :loading="isScanning"
            :disabled="isScanning || !selectionInfo"
            @click="scanPages()"
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

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type {VNodeRef} from 'vue';
import {computed, nextTick, onMounted, onUnmounted, reactive, ref} from 'vue';
import { useFilteredPages, makeIsInFilter } from "../composables/useFilteredPages";
import { usePageFilterNavigation } from "../composables/usePageFilterNavigation";
import PageStrip from './PageStrip.vue';
import type {CropEdges, Page, PageDb} from '../types';
import PageList from "./PageList.vue";

const props = defineProps<{ machineName: string; projectName: string }>();

// ── Mode / tool / edge ───────────────────────────────────────────────
const mode = ref('recognise');
const filterMode = ref('all')

// ── Scan state ───────────────────────────────────────────────────────
interface ScanPageResult { scan: string; success: boolean; error?: string }
interface ScanConflict   { pages: string[] }

const isScanning  = ref(false);
const scanResults = ref<ScanPageResult[]>([]);
const scanError   = ref('');

// ── Pages & crop data ────────────────────────────────────────────────
const pages = ref<Page[]>([]);
const pageCrops = reactive(new Map<number, CropEdges>());
const originalCrops = reactive(new Map<number, CropEdges>());

// ── Selection ────────────────────────────────────────────────────────
// selectionAnchor = first-clicked page (plain click resets accumulator).
// currentPageIndex = free end of range (also used for zoom focus).
const selectionAnchor = ref<number | null>(null);
const currentPageIndex = ref<number | null>(null);

// selectedPages: ordered slice of pages[] between anchor and current.
const selectedPages = computed(() => {
  const all = pages.value;
  if (!all.length) return [];
  const anchor = selectionAnchor.value;
  const current = currentPageIndex.value;
  if (anchor === null && current === null) return [];
  const anchorPos = anchor !== null ? all.findIndex(p => p.index === anchor) : -1;
  const currentPos = current !== null ? all.findIndex(p => p.index === current) : -1;
  if (anchorPos < 0 && currentPos < 0) return [];
  if (anchorPos < 0) return current !== null ? [all[currentPos]] : [];
  if (currentPos < 0) return [all[anchorPos]];
  const lo = Math.min(anchorPos, currentPos);
  const hi = Math.max(anchorPos, currentPos);
  return all.slice(lo, hi + 1);
});

const isInFilter = makeIsInFilter(filterMode);

// filteredPages: selectedPages narrowed to pages that pass the filter.
// ALL edit operations iterate this — pages outside the filter are never touched.
const filteredPages = useFilteredPages(filterMode, selectedPages);

// selectedPageSet: set of indices in filteredPages (for highlight/overlay).
const selectedPageSet = computed(() => new Set(filteredPages.value.map(p => p.index)));

// visiblePages: pages shown in the strip grid — only those passing the filter.
const visiblePages = useFilteredPages(filterMode, pages);


// ── Selection info (for tools panel) ────────────────────────────────
const selectionInfo = computed(() => {
  const fp = filteredPages.value;
  if (!fp.length) return null;
  const first = fp[0];
  const last = fp[fp.length - 1];
  const center = fp[Math.floor((fp.length - 1) / 2)];
  return {
    firstIdx: first.index,
    lastIdx: last.index,
    centerIdx: center.index,
    firstName: first.name || first.scan,
    lastName: last.name || last.scan,
    centerName: center.name || center.scan,
    count: fp.length,
  };
});

// ── Refs ─────────────────────────────────────────────────────────────
const pageListComponentRef = ref<InstanceType<typeof PageList> | null>(null);
const stripWorkareaRef = ref<HTMLElement | null>(null);

const thumbBaseUrl = computed(() => `/media/projects/${props.machineName}/pages/thumbs/`);

const setStripWorkareaRef: VNodeRef = el => {
  stripWorkareaRef.value = el instanceof HTMLElement ? el : null;
};

// ── Scroll helpers ───────────────────────────────────────────────────
async function scrollPageListItemIntoView(pageIndex: number) {
  await pageListComponentRef.value?.scrollPageIntoView(pageIndex);
}

async function scrollStripItemIntoView(pageIndex: number) {
  await nextTick();
  stripWorkareaRef.value
      ?.querySelector<HTMLElement>(`[data-page-idx="${pageIndex}"]`)
      ?.scrollIntoView({block: 'nearest', inline: 'nearest', behavior: 'smooth'});
}

function focusPage(pageIndex: number) {
  void scrollPageListItemIntoView(pageIndex);
  void scrollStripItemIntoView(pageIndex);
}

// ── Selection actions ────────────────────────────────────────────────
// Plain click: new anchor = apply+reset accumulator.
// click: start new selection and reset adjust-accumulator.
function setAnchor(pageIndex: number) {
  selectionAnchor.value = pageIndex;
  currentPageIndex.value = pageIndex;
  void scrollPageListItemIntoView(pageIndex);
  void scrollStripItemIntoView(pageIndex);
}

// ⇧-click: extend range without resetting accumulator.
function extendSelection(pageIndex: number) {
  currentPageIndex.value = pageIndex;
  void scrollPageListItemIntoView(pageIndex);
  void scrollStripItemIntoView(pageIndex);
}

function handleListClick(pageIndex: number, e: MouseEvent) {
  (e.shiftKey) ? extendSelection(pageIndex) : setAnchor(pageIndex);
}

function handleStripClick(pageIndex: number, e: MouseEvent) {
  (e.shiftKey) ? extendSelection(pageIndex) : setAnchor(pageIndex);
}

const { onFilterChange } = usePageFilterNavigation({
  filterMode,
  pages,
  visiblePages,
  selectionAnchor,
  currentPageIndex,
  setAnchor,
});

// ── Page navigation ──────────────────────────────────────────────────
function isTypingTarget(): boolean {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement)) return false;
  return active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' ||
      active.tagName === 'SELECT' || active.isContentEditable;
}

function navigatePage(delta: number) {
  const navPages = visiblePages.value;
  if (!navPages.length || isTypingTarget()) return;
  const anchor = selectionAnchor.value ?? navPages[0].index;
  const pos = navPages.findIndex(p => p.index === anchor);
  const next = pos < 0 ? 0 : Math.max(0, Math.min(navPages.length - 1, pos + delta));
  setAnchor(navPages[next].index);
}

// ── Keyboard shortcuts ───────────────────────────────────────────────
function onKeyDown(e: KeyboardEvent) {
  if (mode.value !== 'crop') return;
  const shift = e.shiftKey;
  const alt = e.altKey;

  // ⇧⎇ combos — edge selection and view toggle
  if (shift && alt) {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        return;
      case 'ArrowDown':
        e.preventDefault();
        return;
      case 'ArrowLeft':
        e.preventDefault();
        return;
      case 'ArrowRight':
        e.preventDefault();
        return;
      case 'F':
      case 'f':
        e.preventDefault();
        return;
    }
  }

  // Skip nav/adjust when focus is inside the tools panel
  const target = e.target as HTMLElement;
  if (target.closest?.('.crop-tools')) return;

  // Page navigation: , / . and < / > (Shift+, / Shift+.)
  switch (e.key) {
    case ',':
      e.preventDefault();
      navigatePage(-1);
      return;
    case '.':
      e.preventDefault();
      navigatePage(1);
      return;
    case '<':
      e.preventDefault();
      navigatePage(-10);
      return;
    case '>':
      e.preventDefault();
      navigatePage(10);
      return;
  }
}

// ── Scan ─────────────────────────────────────────────────────────────
async function scanPages(force = false): Promise<void> {
  const indices = filteredPages.value.map(p => p.index);
  if (!indices.length) return;

  isScanning.value = true;
  scanError.value = '';
  if (!force) scanResults.value = [];

  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/scan`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ indices, force }),
    });

    if (resp.status === 409) {
      const conflict = await resp.json() as ScanConflict;
      isScanning.value = false;
      const msg = `${conflict.pages.length} page(s) have unsaved edits:\n${conflict.pages.join(', ')}\n\nRescanning will overwrite their originals. Continue?`;
      if (window.confirm(msg)) await scanPages(true);
      return;
    }

    if (!resp.ok) {
      const err = await resp.json() as { error: string };
      scanError.value = err.error ?? 'Scan failed.';
      return;
    }

    const data = await resp.json() as { results: ScanPageResult[] };
    scanResults.value = data.results;
  } catch (e) {
    console.error(e);
    scanError.value = 'Network error.';
  } finally {
    isScanning.value = false;
  }
}

onMounted(async () => {
  document.addEventListener('keydown', onKeyDown);
  try {
    const res = await fetch(`/api/projects/${props.machineName}/pages`);
    const data = (await res.json()) as PageDb;
    pages.value = data.pages;
    for (const page of data.pages) {
      const crop = {...page.crop_edges};
      pageCrops.set(page.index, crop);
      originalCrops.set(page.index, {...crop});
    }
    if (data.pages.length > 0) setAnchor(data.pages[0].index);
  } catch (e) {
    console.error('Failed to load pages:', e);
  }
});

onUnmounted(() => {
  document.removeEventListener('keydown', onKeyDown);
});
</script>

<style>

.inspect-area {
  flex: 1 1 auto;
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-columns: 8rem minmax(0, 1fr) 20rem;
  overflow: hidden;
}

.inspect-area > * {
  min-height: 0;
  overflow-y: auto;
  border-right: 1px solid var(--color-border, #dee2e6);
}

.inspect-area > *:last-child {
  border-right: none;
}

.sidebar-lead {
  font-size: 0.75rem;
  font-weight: 600;
  min-height: 1.2rem;
  color: var(--color-text-muted, #6c757d);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.6rem 0.75rem 0.4rem;
  border-bottom: 1px solid var(--color-border, #dee2e6);
  position: sticky;
  top: 0;
  background: var(--color-surface, #fff);
  z-index: 1;
}

.sidebar-content {
  padding: 0.75rem;
  font-size: 0.875rem;
  color: var(--color-text-muted, #6c757d);
}

.inspect-workarea {
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.strip-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 6px;
  align-content: flex-start;
  user-select: none;
  -webkit-user-select: none;
}

.crop-tools {
  padding: 0;
}

/* Page list */

.page-nav-list li {
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-muted, #6c757d);
  border-left: 2px solid transparent;
}

.page-nav-list li:hover {
  background: var(--color-bg-muted, #f1f3f5);
}

/* Selection info panel */
.selection-info-panel {
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 0.375rem;
  padding: 0.5rem;
  font-size: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  margin-bottom: 0.5rem;
}

.info-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.info-label {
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.info-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-count {
  color: var(--color-text-muted, #6c757d);
  flex-shrink: 0;
}

/* Accumulator display */
.accumulator-display {
  font-size: 0.8rem;
  font-variant-numeric: tabular-nums;
  color: var(--color-accent, #2563eb);
  padding: 0.15rem 0;
}


.diamond-inputs {
  display: grid;
  grid-template-columns: max-content max-content max-content;
  gap: 0.5rem;
  justify-content: center;
  align-items: center;
}

.diamond-input {
  width: 5rem;
}

.diamond-input.top {
  grid-column: 2;
  grid-row: 1;
}

.diamond-input.left {
  grid-column: 1;
  grid-row: 2;
}

.diamond-input.right {
  grid-column: 3;
  grid-row: 2;
}

.diamond-input.bottom {
  grid-column: 2;
  grid-row: 3;
}

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

.scan-result-row.ok  .scan-result-icon { color: #16a34a; }
.scan-result-row.fail .scan-result-icon { color: #dc2626; }

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
