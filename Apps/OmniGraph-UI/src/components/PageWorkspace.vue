<template>
  <div class="three-column-grid">

    <!-- Left sidebar -->
    <div class="workspace-left-sidebar"
         :class="{ 'workspace-sidebar-hidden': !isLeftSidebarVisible }">
      <div
          class="workspace-page-list-pane"
          :class="{ 'workspace-pane-hidden': !(panels ? panels['page-list'] : false) }"
      >
        <PageList
            ref="pageListComponentRef"
            :pages="pages"
            :selected-page-indices="selectedPageSet"
            :current-page-index="currentPageIndex"
            :is-page-in-filter="isInFilter"
            :columns="pageListColumns"
            :page-extras="pageExtras"
            @navigate="navigatePage"
            @page-click="handleListClick"
            @select-all="selectAll"
            @select-none="selectNone"
        />
      </div>
      <div
          class="workspace-section-outline-pane"
          :class="{ 'workspace-pane-hidden': !(panels ? panels['section-structure'] : false) }"
      >
        <div class="sidebar-lead">Sections</div>
        <SectionOutline/>
      </div>
    </div>

    <!-- Central: workspace panes -->
    <div class="workspace-workarea">
      <div
          class="workspace-page-strips-pane"
          :class="{ 'workspace-pane-hidden': !(panels ? panels['page-strips'] : false) }"
          :ref="setStripWorkareaRef"
      >
        <slot
            name="page-strips"
            :pages="pages"
            :filtered-pages="filteredPages"
            :visible-pages="visiblePages"
            :selected-page-indices="selectedPageSet"
            :current-page-index="currentPageIndex"
            :current-page="currentPage"
            :current-page-crop="currentPageCrop"
            :focus-page="focusPage"
            :set-anchor="setAnchor"
            :extend-selection="extendSelection"
            :thumb-base-url="thumbBaseUrl"
            :scan-base-url="scanBaseUrl"
            :show-page-strips="!(panels ? panels['page-strips'] : false)"
            :show-page-preview="!(panels ? panels['page-preview'] : false)"
        >
          <div class="strip-grid">
            <PageStrip
                v-for="page in visiblePages"
                :key="page.index"
                :data-page-idx="page.index"
                :page="page"
                :edge="stripEdge ?? 'all'"
                :thumbBaseUrl="thumbBaseUrl"
                :fraction="stripFraction ?? 1"
                :showOverlay="showCropOverlay ?? false"
                :crop="pageCrops?.get(page.index) ?? page.crop_edges"
                :crop-color="cropColor ?? 'rgba(0, 0, 0, 0.0)'"
                :discard-color="discardColor ?? 'rgba(50, 50, 50, 0.35)'"
                :selected="selectedPageSet.has(page.index)"
                @click="handleStripClick(page.index, $event)"
            />
          </div>
        </slot>
      </div>

      <div
          class="workspace-page-preview-pane"
          :class="{ 'workspace-pane-hidden': !(panels ? panels['page-preview'] : false) }"
      >
        <slot
            name="page-preview"
            :pages="pages"
            :filtered-pages="filteredPages"
            :visible-pages="visiblePages"
            :selected-page-indices="selectedPageSet"
            :current-page-index="currentPageIndex"
            :current-page="currentPage"
            :current-page-crop="currentPageCrop"
            :focus-page="focusPage"
            :set-anchor="setAnchor"
            :extend-selection="extendSelection"
            :thumb-base-url="thumbBaseUrl"
            :scan-base-url="scanBaseUrl"
            :show-page-strips="!(panels ? panels['page-strips'] : false)"
            :show-page-preview="!(panels ? panels['page-preview'] : false)"
            :pointer-settings="pointerSettings"
        >
          <PagePreview
              v-if="currentPage && currentPageCrop"
              :page="currentPage"
              :image-base-url="scanBaseUrl"
              :crop="currentPageCrop"
              :show-crop-overlay="showCropOverlay ?? false"
              :crop-color="cropColor ?? 'rgba(0, 180, 0, 0.12)'"
              :discard-color="discardColor ?? 'rgba(220, 0, 0, 0.28)'"
              :hocr-level="hocrLevel"
              :carea-overlay-color="careaOverlayColor"
              :block-overlay-color="blockOverlayColor"
              :line-overlay-color="lineOverlayColor"
              :word-overlay-color="wordOverlayColor"
              :pointer-settings="pointerSettings"
              :interaction-update="pageInteractionUpdate"
              :interaction-click="pageInteractionClick"
          />
        </slot>
      </div>
    </div>

    <!-- Right sidebar -->
    <div class="workspace-right-sidebar"
         :class="{ 'workspace-sidebar-hidden': !isRightSidebarVisible }">
      <div class="workspace-tools"
           :class="{ 'workspace-pane-hidden': !(panels ? panels['tools'] : false) }">
        <div class="sidebar-lead">Tools</div>
        <div class="sidebar-content">

          <br>
          <sl-radio-group
              label="Even/Odd pages"
              size="small"
              :value="effectiveFilterMode"
              @sl-input="onFilterChange"
          >
            <sl-radio-button value="all" :disabled="!canPagesBeFiltered">All</sl-radio-button>
            <sl-radio-button value="even" :disabled="!canPagesBeFiltered">Even</sl-radio-button>
            <sl-radio-button value="odd" :disabled="!canPagesBeFiltered">Odd</sl-radio-button>
          </sl-radio-group>


          <!-- Selection info -->
          <template v-if="selectionInfo">
            <br>
            <div class="info-panel">
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

          <slot
              name="tools"
              :pages="pages"
              :filtered-pages="filteredPages"
              :visible-pages="visiblePages"
              :selected-pages="selectedPages"
              :selection-info="selectionInfo"
              :current-page-index="currentPageIndex"
              :current-page="currentPage"
              :focus-page="focusPage"
              :filter-mode="filterMode"
              :on-filter-change="onFilterChange"
              :page-db-next-batch="pageDbNextBatch"
              :has-changes="hasChanges"
          />

        </div>
      </div>
      <div
          class="workspace-hocr-outline-pane"
          :class="{ 'workspace-pane-hidden': !(panels ? panels['ocr-structure'] : false) }"
      >
        <div class="sidebar-lead">hOCR</div>
        <HocrOutline/>
      </div>
    </div><!-- end workspace-right-sidebar -->


  </div>
</template>

<script setup lang="ts">
import type {Ref, VNodeRef} from 'vue';
import {computed, nextTick, onMounted, onUnmounted, ref, watch} from 'vue';
import {onBeforeRouteLeave} from 'vue-router';
import {useFilteredPages, makeIsInFilter} from "../composables/useFilteredPages";
import {usePageFilterNavigation} from "../composables/usePageFilterNavigation";
import type {CropEdges, HocrLevel, Page, PageDb, PageInteractionUpdate, PointerSettings} from '../types';
import type {PanelVisibility} from '../types';
import PageStrip from '../components/PageStrip.vue';
import PagePreview from '../components/PagePreview.vue';
import PageList from "../components/PageList.vue";
import HocrOutline from "../components/HocrOutline.vue";
import SectionOutline from "../components/SectionOutline.vue";

type PageListColumn = "index" | "batch" | "name" | "scan" | "name-or-scan" | "extras";

type PageWorkspaceKeyboardContext = {
  pages: Page[];
  filteredPages: Page[];
  visiblePages: Page[];
  selectionInfo: {
    firstIdx: number;
    lastIdx: number;
    centerIdx: number;
    firstName: string;
    lastName: string;
    centerName: string;
    count: number;
  } | null;
  focusPage: (pageIndex: number) => void;
  navigatePage: (delta: number) => void;
};

type PageWorkspaceKeyboardHandler = (
    event: KeyboardEvent,
    context: PageWorkspaceKeyboardContext,
) => boolean | void;

const props = withDefaults(defineProps<{
      machineName: string;
      projectName: string,
      canPagesBeFiltered?: boolean;
      formatPageExtras?: (pages: Page[]) => Map<number, string>;
      pageListColumns?: PageListColumn[];
      pageCrops?: Map<number, CropEdges>;
      isPageChanged?: (page: Page) => boolean;
      keyboardHandler?: PageWorkspaceKeyboardHandler;
      stripEdge?: string;
      stripFraction?: number;
      showCropOverlay?: boolean;
      cropColor?: string;
      discardColor?: string;
      hocrLevel?: HocrLevel | null;
      careaOverlayColor?: string;
      blockOverlayColor?: string;
      lineOverlayColor?: string;
      wordOverlayColor?: string;
      pointerSettings?: PointerSettings;
      panels: PanelVisibility | null;
      pageInteractionUpdate?: PageInteractionUpdate;
      pageInteractionClick?: () => void;
    }>(), {
      canPagesBeFiltered: true,
    }
);

const emit = defineEmits<{
  pagesLoaded: [data: PageDb];
  currentPageChange: [page: Page | null];
}>();

const canPagesBeFiltered = computed(() => props.canPagesBeFiltered ?? true);
const pageListColumns = computed(() => props.pageListColumns ?? ["name-or-scan"]) as Ref<PageListColumn[]>;


const isPanelVisible = (panelId: keyof PanelVisibility) => {
  return props.panels ? props.panels[panelId] : false;
};

const isLeftSidebarVisible = computed(() => {
  return isPanelVisible('page-list') || isPanelVisible('section-structure');
});

const isRightSidebarVisible = computed(() => {
  return isPanelVisible('tools') || isPanelVisible('ocr-structure');
});

// ── Mode / tool / edge ───────────────────────────────────────────────
const filterMode = ref('all')
const effectiveFilterMode = computed(() => canPagesBeFiltered.value ? filterMode.value : 'all');

// ── Pages & crop data ────────────────────────────────────────────────
const pages = ref<Page[]>([]);
let pageDbNextBatch = 0;

const thumbBaseUrl = computed(() => `/media/projects/${props.machineName}/pages/thumbs/`);
const scanBaseUrl = computed(() => `/media/projects/${props.machineName}/pages/scans/`);
const currentPage = computed(() => {
  const index = currentPageIndex.value;
  if (index === null) return null;
  return pages.value.find(page => page.index === index) ?? null;
});
const currentPageCrop = computed(() => {
  const page = currentPage.value;
  if (!page) return null;
  return props.pageCrops?.get(page.index) ?? page.crop_edges;
});
const pageExtras = computed(() => {
  return props.formatPageExtras?.(pages.value) ?? new Map<number, string>();
});

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

const isInFilter = makeIsInFilter(effectiveFilterMode);

// filteredPages: selectedPages narrowed to pages that pass the filter.
// ALL edit operations iterate this — pages outside the filter are never touched.
const filteredPages = useFilteredPages(effectiveFilterMode, selectedPages);

// selectedPageSet: set of indices in filteredPages (for highlight/overlay).
const selectedPageSet = computed(() => new Set(filteredPages.value.map(p => p.index)));

// visiblePages: pages shown in the strip grid — only those passing the filter.
const visiblePages = useFilteredPages(effectiveFilterMode, pages);


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
  return props.isPageChanged
      ? pages.value.some(props.isPageChanged)
      : false;
});

onBeforeRouteLeave(() => {
  if (!hasChanges.value) return true;

  return window.confirm(
      'You have uncommitted changes. Leave this page and discard them?'
  );
});


// ── Refs ─────────────────────────────────────────────────────────────

const pageListComponentRef = ref<InstanceType<typeof PageList> | null>(null);
const stripWorkareaRef = ref<HTMLElement | null>(null);

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

function selectAll() {
  if (pages.value.length) {
    setAnchor(pages.value[0].index);
    extendSelection(pages.value[pages.value.length - 1].index);
    console.log('Select all', filteredPages.value.map(p => p.name || p.scan));
  }
}

function selectNone() {
  selectionAnchor.value = null;
  currentPageIndex.value = null;
  console.log('Select none', filteredPages.value.map(p => p.name || p.scan));
}

const {onFilterChange} = usePageFilterNavigation({
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

function makeKeyboardContext(): PageWorkspaceKeyboardContext {
  return {
    pages: pages.value,
    filteredPages: filteredPages.value,
    visiblePages: visiblePages.value,
    selectionInfo: selectionInfo.value,
    focusPage,
    navigatePage,
  };
}

function onKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLElement;

  if (target.closest?.('.workspace-tools')) return;

  if (props.keyboardHandler?.(e, makeKeyboardContext()) === true) {
    return;
  }

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

function setPageDb(data: PageDb) {
  pages.value = data.pages;
  pageDbNextBatch = data.next_batch;
  emit('pagesLoaded', data);
  if (data.pages.length > 0) setAnchor(data.pages[0].index);
}

async function loadPageDb(data?: PageDb) {
  if (data) {

  } else {
    try {
      const res = await fetch(`/api/projects/${props.machineName}/pages`);
      const data = (await res.json()) as PageDb;
      setPageDb(data);
    } catch (e) {
      console.error('Failed to load pages:', e);
    }
  }
}

async function savePageDb(): Promise<void> {
  const pageDb: PageDb = {
    pages: pages.value,
    next_batch: pageDbNextBatch,
  };

  const resp = await fetch(`/api/projects/${props.machineName}/pages`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(pageDb),
  });

  if (!resp.ok) {
    const err = await resp.json().catch(() => null) as { error?: string } | null;
    throw new Error(err?.error ?? 'Failed to save pages.');
  }
}


watch(currentPage, (page) => emit('currentPageChange', page));

onMounted(async () => {
  document.addEventListener('keydown', onKeyDown);
  await loadPageDb();
});

onUnmounted(() => {
  document.removeEventListener('keydown', onKeyDown);
});

defineExpose({
  setPageDb,
  savePageDb
});

</script>

<style>

.three-column-grid {
  flex: 1 1 auto;
  min-height: 0;
  height: 100%;
  display: flex;
  overflow: hidden;
}

/* Left sidebar — vertical flex column */
.workspace-left-sidebar {
  flex: 0 0 14rem;
  min-height: 0;
  max-height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border, #dee2e6);
}

/* Left sidebar — content items */

.workspace-page-list-pane {
  flex: 2 1 0;
  min-height: 0;
  overflow-y: auto;
}

.workspace-section-outline-pane {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  border-top: 1px solid var(--color-border, #dee2e6);
}

/* Centre */
.workspace-workarea {
  flex: 1 1 auto;
  border-right: 1px solid var(--color-border, #dee2e6);
}

/* Right sidebar — vertical flex column */
.workspace-right-sidebar {
  flex: 0 0 20rem;
  min-height: 0;
  max-height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Right sidebar — content items */

.workspace-tools {
  flex: 3 1 0;
  min-height: 0;
  max-height: 100%;
  overflow-y: auto;
  padding: 0;
  border-bottom: 1px solid var(--color-border, #dee2e6);
}

.workspace-hocr-outline-pane {
  flex: 2 1 0;
  min-height: 0;
  max-height: 100%;
  overflow-y: auto;
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

.workspace-workarea {
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  display: flex;
  overflow: hidden;
}

.workspace-page-strips-pane,
.workspace-page-preview-pane {
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow: hidden;
  transition: flex-basis 160ms ease,
  width 160ms ease,
  border-color 160ms ease;
}

.workspace-page-strips-pane {
  flex: 1 1 45%;
  overflow-y: auto;
}

.workspace-page-preview-pane {
  flex: 1 1 45%;
  border-left: 1px solid var(--color-border, #dee2e6);
  display: flex;
  min-width: 0;
  min-height: 0;;
}

.workspace-pane-hidden,
.workspace-sidebar-hidden {
  flex: 0 0 0;
  width: 0;
  min-width: 0;
  border-color: transparent;
  overflow: hidden;
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

/* Info panels */
.info-panel {
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

</style>
