<template>
  <div class="crop-area">

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
    <div class="crop-workarea" :ref="setStripWorkareaRef">
      <div class="strip-grid">
        <PageStrip
            v-for="page in visiblePages"
            :key="page.index"
            :data-page-idx="page.index"
            :page="page"
            :edge="edge"
            :thumbBaseUrl="thumbBaseUrl"
            :fraction="viewPercent / 100"
            :showOverlay="mode === 'crop'"
            :crop="pageCrops.get(page.index) ?? page.crop_edges"
            crop-color="rgba(0, 180, 0, 0.12)"
            discard-color="rgba(220, 0, 0, 0.35)"
            :selected="selectedPageSet.has(page.index)"
            @click="handleStripClick(page.index, $event)"
        />
      </div>
    </div>

    <!-- Tools -->
    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <template v-if="mode === 'crop'">

          <br>
          <sl-radio-group label="Even/Odd pages" size="small" :value="filterMode" @sl-change="onFilterChange">
            <sl-radio-button value="all">All</sl-radio-button>
            <sl-radio-button value="even">Even</sl-radio-button>
            <sl-radio-button value="odd">Odd</sl-radio-button>
          </sl-radio-group>


          <!-- Selection info -->
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

          <!-- Tools -->
          <sl-radio-group label="Tool" name="tool" :value="tool"
                          @sl-change="tool = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="adjust">Adjust</sl-radio-button>
            <sl-radio-button value="assign">Assign</sl-radio-button>
          </sl-radio-group>

          <br>

          <template v-if="tool === 'adjust'">
            <!-- Adjust tool -->
            <sl-input label="Step small" type="number" min="1" :value="adjust_step_small"
                      @sl-input="adjust_step_small = Math.max(1, parseInt(($event.target as HTMLInputElement).value) || 1)"
            />
            <sl-input label="Step large" type="number" min="1" :value="adjust_step_large"
                      @sl-input="adjust_step_large = Math.max(1, parseInt(($event.target as HTMLInputElement).value) || 1)"
            />
            <div class="accumulator-display" v-if="accumulator !== 0">
              Δ {{ accumulator > 0 ? '+' : '' }}{{ accumulator }} px
            </div>

            <br>

            <!-- Magnet -->
            <sl-switch
                :checked="magnetEnabled"
                @sl-change="magnetEnabled = ($event.target as HTMLInputElement).checked; applyMagnet()"
            >Magnet
            </sl-switch>

            <br>

            <template v-if="magnetEnabled">
              <br>
              <sl-radio-group label="Profile" name="profile" :value="magnetProfile"
                              @sl-change="magnetProfile = ($event.target as HTMLInputElement).value as MagnetProfile; applyMagnet()">
                <sl-radio-button value="bell" title="0 → peak → 0">Bell</sl-radio-button>
                <sl-radio-button value="rampup" title="0 → peak">Ramp ↑</sl-radio-button>
                <sl-radio-button value="rampdown" title="peak → 0">Ramp ↓</sl-radio-button>
              </sl-radio-group>
            </template>

            <br>

            <!-- Edge selector -->
            <sl-radio-group label="Edge" name="edge" size="small" :value="edge"
                            @sl-change="onEdgeChange(($event.target as HTMLInputElement).value)">
              <sl-radio-button value="none">None</sl-radio-button>
              <sl-radio-button value="left">Left</sl-radio-button>
              <sl-radio-button value="top">Top</sl-radio-button>
              <sl-radio-button value="bottom">Bottom</sl-radio-button>
              <sl-radio-button value="right">Right</sl-radio-button>
            </sl-radio-group>

            <template v-if="edge !== 'none'">
              <br>
              <sl-range
                  :label="`Edge percent: ${viewPercent}%`"
                  min="10" max="75" step="5" :value="viewPercent"
                  @sl-input="viewPercent = parseInt(($event.target as HTMLInputElement).value)"
              />
            </template>

          </template>

          <template v-if="tool === 'assign'">
            <div class="diamond-inputs">
              <sl-input class="diamond-input top" size="small" pill type="number" :value="assignValues.top ?? ''"
                        @sl-input="assignValues.top = parseOptionalNumber(($event.target as HTMLInputElement).value)"></sl-input>
              <sl-input class="diamond-input left" size="small" pill type="number" :value="assignValues.left ?? ''"
                        @sl-input="assignValues.left = parseOptionalNumber(($event.target as HTMLInputElement).value)"></sl-input>
              <sl-input class="diamond-input right" size="small" pill type="number" :value="assignValues.right ?? ''"
                        @sl-input="assignValues.right = parseOptionalNumber(($event.target as HTMLInputElement).value)"></sl-input>
              <sl-input class="diamond-input bottom" size="small" pill type="number" :value="assignValues.bottom ?? ''"
                        @sl-input="assignValues.bottom = parseOptionalNumber(($event.target as HTMLInputElement).value)"></sl-input>
            </div>

            <br>

            <sl-button-group>
              <sl-button variant="default" @click="assignBySetting(assignValues)">Set</sl-button>
              <sl-button variant="default" @click="assignByAdding(assignValues)">Add</sl-button>
              <sl-button variant="default" @click="assignBySubtracting(assignValues)">Sub</sl-button>
            </sl-button-group>

            <br>

            <sl-button-group>
              <sl-button variant="default" @click="assignReset">Reset</sl-button>
              <sl-button variant="default" @click="assignAllEdges(100)">+100</sl-button>
              <sl-button variant="default" @click="assignAllEdges(-100)">−100</sl-button>
            </sl-button-group>
          </template>

          <br><br>

          <!-- Session buttons -->
          <sl-button-group>
            <sl-button variant="danger" :disabled="!hasChanges" @click="abandonCrop">Abandon</sl-button>
            <sl-button variant="primary" :disabled="!hasChanges" @click="commitCrops">Commit</sl-button>
          </sl-button-group>

        </template>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type {VNodeRef} from 'vue';
import {computed, nextTick, onMounted, onUnmounted, reactive, ref} from 'vue';
import { onBeforeRouteLeave } from 'vue-router';
import { useFilteredPages, makeIsInFilter } from "../composables/useFilteredPages";
import { usePageFilterNavigation } from "../composables/usePageFilterNavigation";
import PageStrip from '../components/PageStrip.vue';
import type {CropEdges, Page, PageDb} from '../types';
import PageList from "../components/PageList.vue";

const props = defineProps<{ machineName: string; projectName: string }>();

// ── Mode / tool / edge ───────────────────────────────────────────────
const mode = ref('crop');
const tool = ref('adjust');
const edge = ref('none');
const viewPercent = ref(25);
const filterMode = ref('all')
const viewMode = ref<'windowed' | 'full'>('windowed');

// ── Adjust tool data ─────────────────────────────────────────────────
const adjust_step_small = ref(10);
const adjust_step_large = ref(50);

// ── Assign tool data ─────────────────────────────────────────────────
type OptionalNumber = number | undefined;

type AssignValues = {
  top: OptionalNumber;
  left: OptionalNumber;
  right: OptionalNumber;
  bottom: OptionalNumber;
};

const assignValues = reactive<AssignValues>({
  top: undefined,
  left: undefined,
  right: undefined,
  bottom: undefined,
});

function parseOptionalNumber(value: string): OptionalNumber {
  if (value.trim() === '') return undefined;

  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

// ── Pages & crop data ────────────────────────────────────────────────
const pages = ref<Page[]>([]);
const pageCrops = reactive(new Map<number, CropEdges>());
const originalCrops = reactive(new Map<number, CropEdges>());
let storedNextBatch = 0;

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

// ── Accumulator & magnet ─────────────────────────────────────────────
const accumulator = ref(0);
const roundBaseCrops = new Map<number, CropEdges>();

type MagnetProfile = 'bell' | 'rampup' | 'rampdown';
const magnetEnabled = ref(false);
const magnetProfile = ref<MagnetProfile>('bell');

function getMagnetWeight(i: number, n: number, profile: MagnetProfile): number {
  if (n <= 1) return 1.0;
  const t = i / (n - 1);
  switch (profile) {
    case 'bell':
      return t <= 0.5 ? 2 * t : 2 * (1 - t);
    case 'rampup':
      return t;
    case 'rampdown':
      return 1 - t;
  }
}

function applyMagnet() {
  if (edge.value === 'none') return;
  const edgeKey = edge.value as keyof CropEdges;
  const fp = filteredPages.value;
  const n = fp.length;
  for (let i = 0; i < n; i++) {
    const page = fp[i];
    const base = roundBaseCrops.get(page.index);
    const curr = pageCrops.get(page.index);
    if (!base || !curr) continue;
    const weight = magnetEnabled.value ? getMagnetWeight(i, n, magnetProfile.value) : 1.0;
    curr[edgeKey] = Math.max(0, base[edgeKey] + Math.round(accumulator.value * weight));
  }
}

function rebuildRoundBase() {
  roundBaseCrops.clear();
  for (const page of pages.value) {
    const crop = pageCrops.get(page.index);
    if (crop) roundBaseCrops.set(page.index, {...crop});
  }
}

function adjustRange(delta: number) {
  if (edge.value === 'none' || filteredPages.value.length === 0) return;
  accumulator.value += delta;
  applyMagnet();
}

function onEdgeChange(newEdge: string) {
  // Changing edge starts a new adjustment round.
  accumulator.value = 0;
  rebuildRoundBase();
  edge.value = newEdge;
}

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

// ── Has changes ──────────────────────────────────────────────────────
const hasChanges = computed(() => {
  for (const page of pages.value) {
    const orig = originalCrops.get(page.index);
    const curr = pageCrops.get(page.index);
    if (!orig || !curr) continue;
    if (orig.left !== curr.left || orig.top !== curr.top ||
        orig.right !== curr.right || orig.bottom !== curr.bottom) return true;
  }
  return false;
});

onBeforeRouteLeave(() => {
  if (!hasChanges.value) return true;

  return window.confirm(
      'You have uncommitted crop changes. Leave this page and discard them?'
  );
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
  accumulator.value = 0;
  rebuildRoundBase();
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

// ── Adjust tool functions ────────────────────────────────────────────

function adjustByKey(shift: boolean, e: KeyboardEvent) {
  const step = shift ? adjust_step_large.value : adjust_step_small.value;

  if (edge.value === 'top' && e.key === 'ArrowDown') {
    e.preventDefault();
    adjustRange(step);
    return;
  }
  if (edge.value === 'top' && e.key === 'ArrowUp') {
    e.preventDefault();
    adjustRange(-step);
    return;
  }
  if (edge.value === 'bottom' && e.key === 'ArrowDown') {
    e.preventDefault();
    adjustRange(-step);
    return;
  }
  if (edge.value === 'bottom' && e.key === 'ArrowUp') {
    e.preventDefault();
    adjustRange(step);
    return;
  }
  if (edge.value === 'left' && e.key === 'ArrowRight') {
    e.preventDefault();
    adjustRange(step);
    return;
  }
  if (edge.value === 'left' && e.key === 'ArrowLeft') {
    e.preventDefault();
    adjustRange(-step);
    return;
  }
  if (edge.value === 'right' && e.key === 'ArrowRight') {
    e.preventDefault();
    adjustRange(-step);
    return;
  }
  if (edge.value === 'right' && e.key === 'ArrowLeft') {
    e.preventDefault();
    adjustRange(step);
    return;
  }
}

// ── Assign tool functions ────────────────────────────────────────────

function forEachSelected(fn: (crop: CropEdges) => void) {
  for (const page of filteredPages.value) {
    const crop = pageCrops.get(page.index);
    if (crop) fn(crop);
  }
}

function assignBySetting(values: AssignValues) {
  forEachSelected(crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, values.bottom);
  });
}

function assignByAdding(values: AssignValues) {
  forEachSelected(crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, crop.left   + values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, crop.top    + values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, crop.right  + values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, crop.bottom + values.bottom);
  });
}

function assignBySubtracting(values: AssignValues) {
  forEachSelected(crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, crop.left   - values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, crop.top    - values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, crop.right  - values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, crop.bottom - values.bottom);
  });
}

function assignReset() {
  forEachSelected(crop => { crop.left = crop.top = crop.right = crop.bottom = 0; });
}

function assignAllEdges(delta: number) {
  forEachSelected(crop => {
    crop.left   = Math.max(0, crop.left   + delta);
    crop.top    = Math.max(0, crop.top    + delta);
    crop.right  = Math.max(0, crop.right  + delta);
    crop.bottom = Math.max(0, crop.bottom + delta);
  });
}

// ── Crop session: abandon / commit ───────────────────────────────────
function abandonCrop() {
  accumulator.value = 0;
  pageCrops.clear();
  for (const [idx, crop] of originalCrops) pageCrops.set(idx, {...crop});
  rebuildRoundBase();
}

async function commitCrops() {
  const updatedPageDb = {
    next_batch: storedNextBatch,
    pages: pages.value.map(page => ({
      ...page,
      crop_edges: pageCrops.get(page.index) ?? page.crop_edges,
    })),
  };
  try {
    const res = await fetch(`/api/projects/${props.machineName}/pages`, {
      method: 'PUT',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify(updatedPageDb),
    });
    if (res.ok) {
      for (const [idx, crop] of pageCrops) originalCrops.set(idx, {...crop});
    } else {
      console.error('Commit failed:', res.status, await res.text());
    }
  } catch (e) {
    console.error('Commit error:', e);
  }
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
        onEdgeChange('top');
        return;
      case 'ArrowDown':
        e.preventDefault();
        onEdgeChange('bottom');
        return;
      case 'ArrowLeft':
        e.preventDefault();
        onEdgeChange('left');
        return;
      case 'ArrowRight':
        e.preventDefault();
        onEdgeChange('right');
        return;
      case 'F':
      case 'f':
        e.preventDefault();
        viewMode.value = viewMode.value === 'full' ? 'windowed' : 'full';
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

  // Edge adjustment — not when Alt is held; Shift = large step
  if (alt) return;

  adjustByKey(shift, e);
}

onMounted(async () => {
  document.addEventListener('keydown', onKeyDown);
  try {
    const res = await fetch(`/api/projects/${props.machineName}/pages`);
    const data = (await res.json()) as PageDb;
    pages.value = data.pages;
    storedNextBatch = data.next_batch;
    for (const page of data.pages) {
      const crop = {...page.crop_edges};
      pageCrops.set(page.index, crop);
      originalCrops.set(page.index, {...crop});
    }
    rebuildRoundBase();
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

.crop-area {
  flex: 1 1 auto;
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-columns: 8rem minmax(0, 1fr) 20rem;
  overflow: hidden;
}

.crop-area > * {
  min-height: 0;
  overflow-y: auto;
  border-right: 1px solid var(--color-border, #dee2e6);
}

.crop-area > *:last-child {
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
</style>
