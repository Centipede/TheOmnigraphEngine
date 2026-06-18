<template>
  <div class="crop-area">

    <div class="crop-sidebar-pages">
      <div class="sidebar-lead">Pages ({{ pages.length }})</div>
      <sl-button-group>
        <sl-button size="small" title="Back (,)" @click="navigatePage(-1)">←</sl-button>
        <sl-button size="small" title="Select all" @click="">All</sl-button>
        <sl-button size="small" title="Forward (.)" @click="navigatePage(1)">→</sl-button>
      </sl-button-group>
      <ul class="page-nav-list" ref="pageListRef">
        <li
            v-for="page in pages"
            :key="page.index"
            :data-page-idx="page.index"
            :class="{
            'page-nav-selected': page.index === currentPageIndex,
            'page-nav-named':    !!page.name,
          }"
            @click="currentPageIndex = page.index"
        >
          <span v-if="page.name">{{ page.name }}</span>
          <em v-else class="page-nav-unnamed">{{ page.scan }}</em>
        </li>
      </ul>
    </div>

<!--    <div class="crop-sidebar-sections">-->
<!--      <div class="sidebar-lead">Sections</div>-->
<!--      <div class="sidebar-content">Section list</div>-->
<!--    </div>-->

    <div class="crop-workarea">
      <div class="strip-grid">
        <PageStrip
            v-for="page in pages"
            :key="page.index"
            :page="page"
            :edge="edge"
            :thumbBaseUrl="thumbBaseUrl"
            :fraction="viewPercent / 100"
            :showOverlay="mode === 'crop'"
            :crop="pageCrops.get(page.index) ?? page.crop_edges"
            :selected="page.index === currentPageIndex"
            @click="currentPageIndex = page.index"
        />
      </div>
    </div>

    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <sl-button-group>
          <sl-button :disabled="hasChanges" :href="`/projects/${props.machineName}/folios`"  variant="default">OCR</sl-button>
          <sl-button disabled variant="primary">Crop</sl-button>
        </sl-button-group>

        <sl-range
            v-if="mode === 'crop'"
            :label="`View: ${viewPercent}%`"
            min="10" max="75" step="5" :value="viewPercent"
            @sl-input="viewPercent = parseInt(($event.target as HTMLInputElement).value)"
        />

        <br>

        <template v-if="mode === 'crop'">
          <br>

          <sl-radio-group label="Tool" name="tool" :value="tool"
                          @sl-change="tool = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="singleadjust">Single Adjust</sl-radio-button>
            <sl-radio-button value="wideadjust">Wide Adjust</sl-radio-button>
          </sl-radio-group>

          <template v-if="tool === 'singleadjust'">
            <br>

            <sl-input label="Step (small)" :value="adjust_step_small"
                      @sl-input="adjust_step_small = parseInt(($event.target as HTMLInputElement).value)"></sl-input>
            <sl-input label="Step (large)" :value="adjust_step_large"
                      @sl-input="adjust_step_large = parseInt(($event.target as HTMLInputElement).value)"></sl-input>

          </template>

          <template v-if="tool === 'wideadjust'">
            <br>
            <sl-range
                :label="`Width: ${wide_width} pages`"
                min="1" max="200" step="1" :value="wide_width"
                @sl-input="wide_width = parseInt(($event.target as HTMLInputElement).value)"
            />
            <br>
            <sl-input
                label="Adjust (scan px)"
                type="number"
                min="1"
                :value="wide_value"
                @sl-input="wide_value = Math.max(1, parseInt(($event.target as HTMLInputElement).value) || 1)"
            />
            <br>
            <sl-button-group>
              <sl-button @click="applyWideAdjust(1)">Apply</sl-button>
              <sl-button @click="applyWideAdjust(-1)">Unapply</sl-button>
            </sl-button-group>
          </template>

          <br>
          <br>

          <sl-radio-group label="Edge" name="edge" size="small" :value="edge"
                          @sl-change="edge = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="none">None</sl-radio-button>
            <sl-radio-button value="left">Left</sl-radio-button>
            <sl-radio-button value="top">Top</sl-radio-button>
            <sl-radio-button value="bottom">Bottom</sl-radio-button>
            <sl-radio-button value="right">Right</sl-radio-button>
          </sl-radio-group>
          <br><br>

          <div class="session-buttons">
            <sl-button
              variant="danger"
              :disabled="!hasChanges"
              @click="abandonCrop"
            >Abandon</sl-button>
            <sl-button
              variant="primary"
              :disabled="!hasChanges"
              @click="commitCrops"
            >Commit</sl-button>
          </div>

        </template>

      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import PageStrip from './PageStrip.vue';
import type {Page, PageDb, CropEdges} from './types';

const props = defineProps<{ machineName: string; projectName: string }>();

const mode = ref('crop');
const tool = ref('singleadjust');
const edge = ref('none');
const adjust_step_small = ref(25);
const adjust_step_large = ref(100);
const wide_width        = ref(100);
const wide_value        = ref(5);
const viewPercent       = ref(25);
const viewMode          = ref<'windowed' | 'full'>('windowed');
const pages             = ref<Page[]>([]);
const currentPageIndex  = ref<number | null>(null);

// Per-page crop state — populated from API, modified client-side until saved.
const pageCrops     = reactive(new Map<number, CropEdges>());
// Snapshot of server state — reactive so hasChanges recomputes when it's updated.
const originalCrops = reactive(new Map<number, CropEdges>());
let   storedNextBatch = 0;

const pageListRef = ref<HTMLElement | null>(null);

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

// Keep the selected page visible in the sidebar when navigating by keyboard.
watch(currentPageIndex, async (idx) => {
  if (idx === null) return;
  await nextTick();
  pageListRef.value
      ?.querySelector<HTMLElement>(`[data-page-idx="${idx}"]`)
      ?.scrollIntoView({block: 'nearest', behavior: 'smooth'});
});

const thumbBaseUrl = computed(
    () => `/projects/${props.machineName}/pages/thumbs/`
);

// ── Page navigation ─────────────────────────────────────────────────

function navigatePage(delta: number) {
  if (!pages.value.length) return;
  const cur = currentPageIndex.value ?? 0;
  const next = Math.max(0, Math.min(pages.value.length - 1, cur + delta));
  currentPageIndex.value = next;
}

// ── Edge adjustment (Single Adjust — current page only) ─────────────

function adjustEdge(which: keyof CropEdges, delta: number) {
  if (currentPageIndex.value === null) return;
  const crop = pageCrops.get(currentPageIndex.value);
  if (crop) crop[which] = Math.max(0, crop[which] + delta);
}

// ── Wide Adjust — apply delta to all pages within ±wide_width ────────

function applyWideAdjust(sign: 1 | -1) {
  if (currentPageIndex.value === null || edge.value === 'none') return;
  const center  = currentPageIndex.value;
  const radius  = wide_width.value;
  const delta   = sign * wide_value.value;
  const edgeKey = edge.value as keyof CropEdges;
  for (const page of pages.value) {
    if (Math.abs(page.index - center) <= radius) {
      const crop = pageCrops.get(page.index);
      if (crop) crop[edgeKey] = Math.max(0, crop[edgeKey] + delta);
    }
  }
}

// ── Crop session: abandon / commit ──────────────────────────────────

function abandonCrop() {
  pageCrops.clear();
  for (const [idx, crop] of originalCrops) {
    pageCrops.set(idx, { ...crop });
  }
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
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updatedPageDb),
    });
    if (res.ok) {
      // Advance the snapshot so hasChanges resets to false.
      for (const [idx, crop] of pageCrops) {
        originalCrops.set(idx, { ...crop });
      }
    } else {
      console.error('Commit failed:', res.status, await res.text());
    }
  } catch (e) {
    console.error('Commit error:', e);
  }
}

// ── Keyboard shortcuts ───────────────────────────────────────────────
// Only active when not typing in a form element inside the tools panel.

function onKeyDown(e: KeyboardEvent) {
  if (mode.value !== 'crop') return;

  const shift = e.shiftKey;
  const alt = e.altKey;

  // ⇧⎇ combos — always active in crop mode
  if (shift && alt) {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        edge.value = 'top';
        return;
      case 'ArrowDown':
        e.preventDefault();
        edge.value = 'bottom';
        return;
      case 'ArrowLeft':
        e.preventDefault();
        edge.value = 'left';
        return;
      case 'ArrowRight':
        e.preventDefault();
        edge.value = 'right';
        return;
      case 'F':
      case 'f':
        e.preventDefault();
        viewMode.value = viewMode.value === 'full' ? 'windowed' : 'full';
        return;
    }
  }

  // Skip navigation/adjustment keys when focus is inside the tools panel
  const target = e.target as HTMLElement;
  if (target.closest?.('.crop-tools')) return;

  // Page navigation: , / .  and  < / >  (Shift+,  Shift+.)
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
      return;  // Shift+,
    case '>':
      e.preventDefault();
      navigatePage(10);
      return;  // Shift+.
  }

  // Arrow key edge adjustment — Single Adjust tool only, not when Alt is held.
  // Plain arrow = step small; Shift+arrow = step large.
  if (tool.value !== 'singleadjust' || alt) return;

  const step = shift ? adjust_step_large.value : adjust_step_small.value;

  if (edge.value === 'top'    && e.key === 'ArrowDown')  { e.preventDefault(); adjustEdge('top',    step);  return; }
  if (edge.value === 'top'    && e.key === 'ArrowUp')    { e.preventDefault(); adjustEdge('top',   -step);  return; }
  if (edge.value === 'bottom' && e.key === 'ArrowDown')  { e.preventDefault(); adjustEdge('bottom', step);  return; }
  if (edge.value === 'bottom' && e.key === 'ArrowUp')    { e.preventDefault(); adjustEdge('bottom',-step);  return; }
  if (edge.value === 'left'   && e.key === 'ArrowRight') { e.preventDefault(); adjustEdge('left',   step);  return; }
  if (edge.value === 'left'   && e.key === 'ArrowLeft')  { e.preventDefault(); adjustEdge('left',  -step);  return; }
  if (edge.value === 'right'  && e.key === 'ArrowRight') { e.preventDefault(); adjustEdge('right',  step);  return; }
  if (edge.value === 'right'  && e.key === 'ArrowLeft')  { e.preventDefault(); adjustEdge('right', -step);  return; }
}

onMounted(async () => {
  document.addEventListener('keydown', onKeyDown);
  try {
    const res = await fetch(`/api/projects/${props.machineName}/pages`);
    const data = (await res.json()) as PageDb;
    pages.value = data.pages;
    storedNextBatch = data.next_batch;
    for (const page of data.pages) {
      const crop = { ...page.crop_edges };
      pageCrops.set(page.index, crop);
      originalCrops.set(page.index, { ...crop });
    }
    if (data.pages.length > 0) currentPageIndex.value = 0;
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
  width: 100%;
  display: grid;
  grid-template-columns: 8rem 1fr 20rem;
  height: calc(100vh - var(--header-height, 0px));
  overflow: hidden;
  font-family: var(--sl-font-sans, sans-serif);
  background: var(--color-bg, #f8f9fa);
  color: var(--color-text, #212529);
}

.crop-area > div {
  border-right: 1px solid var(--color-border, #dee2e6);
  overflow-y: auto;
  min-height: 0;
}

.crop-area > div:last-child {
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

.crop-workarea {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

.strip-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 6px;
  align-content: flex-start;
}

.crop-tools {
  padding: 0;
}

.page-nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  font-size: 0.75rem;
}

.page-nav-list li {
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-muted, #6c757d);
  border-left: 2px solid transparent;
  user-select: none;
}

.page-nav-list li:hover {
  background: var(--color-bg-muted, #f1f3f5);
}

.page-nav-named {
  color: var(--color-text, #212529);
}

.page-nav-selected {
  background: var(--color-bg-selected, #8397aa) !important;
  color: var(--color-text, #212529) !important;
  border-left-color: var(--color-accent, #2563eb) !important;
}

.page-nav-unnamed {
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.session-buttons {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.25rem;
  border-top: 1px solid var(--color-border, #dee2e6);
}

.page-nav-buttons sl-button::part(base) {
  padding-inline: 0.25rem;
  min-height: 1.25rem;
  font-size: 0.7rem;
}
</style>
