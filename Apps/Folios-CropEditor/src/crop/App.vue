<template>
  <div class="crop-area">

    <div class="crop-sidebar-pages">
      <div class="sidebar-lead">Pages</div>
      <div class="sidebar-content">Page list</div>
    </div>

    <div class="crop-sidebar-sections">
      <div class="sidebar-lead">Sections</div>
      <div class="sidebar-content">Section list</div>
    </div>

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
          :crop="defaultCrop"
          :selected="page.index === currentPageIndex"
          @click="currentPageIndex = page.index"
        />
      </div>
    </div>

    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <sl-range
          :label="`View: ${viewPercent}%`"
          min="10" max="75" step="5" :value="viewPercent"
          @sl-input="viewPercent = parseInt(($event.target as HTMLInputElement).value)"
        />

        <br>

        <sl-radio-group label="Mode" name="mode" :value="mode"
          @sl-change="mode = ($event.target as HTMLInputElement).value">
          <sl-radio-button value="none">None</sl-radio-button>
          <sl-radio-button value="crop">Crop</sl-radio-button>
        </sl-radio-group>

        <template v-if="mode === 'crop'">
          <br>

          <sl-radio-group label="Tool" name="tool" :value="tool"
            @sl-change="tool = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="singleadjust">Single Adjust</sl-radio-button>
            <sl-radio-button value="wideadjust">Wide Adjust</sl-radio-button>
          </sl-radio-group>

          <template v-if="tool === 'wideadjust'">
            <br>
            <sl-range
              :label="`Width: ${wide_width}`"
              min="1" max="200" step="1" :value="wide_width"
              @sl-input="wide_width = parseInt(($event.target as HTMLInputElement).value)"
            />
          </template>

          <br>

          <sl-radio-group label="Edge" name="edge" size="small" :value="edge"
            @sl-change="edge = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="none">None</sl-radio-button>
            <sl-radio-button value="left">Left</sl-radio-button>
            <sl-radio-button value="top">Top</sl-radio-button>
            <sl-radio-button value="bottom">Bottom</sl-radio-button>
            <sl-radio-button value="right">Right</sl-radio-button>
          </sl-radio-group>
        </template>

      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import PageStrip from './PageStrip.vue';
import type { Page, PageDb, CropEdges } from './types';

const props = defineProps<{ machineName: string; projectName: string }>();

const mode             = ref('none');
const tool             = ref('singleadjust');
const edge             = ref('none');
const wide_width       = ref(100);
const viewPercent      = ref(25);
const viewMode         = ref<'windowed' | 'full'>('windowed');
const pages            = ref<Page[]>([]);
const currentPageIndex = ref<number | null>(null);

const defaultCrop = reactive<CropEdges>({ left: 20, top: 20, right: 20, bottom: 20 });

const thumbBaseUrl = computed(
  () => `/projects/${props.machineName}/pages/thumbs/`
);

// ── Page navigation ─────────────────────────────────────────────────

function navigatePage(delta: number) {
  if (!pages.value.length) return;
  const cur  = currentPageIndex.value ?? 0;
  const next = Math.max(0, Math.min(pages.value.length - 1, cur + delta));
  currentPageIndex.value = next;
}

// ── Edge adjustment (Single Adjust tool) ────────────────────────────

function adjustEdge(which: keyof CropEdges, delta: number) {
  defaultCrop[which] = Math.max(0, defaultCrop[which] + delta);
}

// ── Keyboard shortcuts ───────────────────────────────────────────────
// Only active when not typing in a form element inside the tools panel.

function onKeyDown(e: KeyboardEvent) {
  if (mode.value !== 'crop') return;

  const shift = e.shiftKey;
  const alt   = e.altKey;

  // ⇧⎇ combos — always active in crop mode
  if (shift && alt) {
    switch (e.key) {
      case 'ArrowUp':    e.preventDefault(); edge.value = 'top';    return;
      case 'ArrowDown':  e.preventDefault(); edge.value = 'bottom'; return;
      case 'ArrowLeft':  e.preventDefault(); edge.value = 'left';   return;
      case 'ArrowRight': e.preventDefault(); edge.value = 'right';  return;
      case 'F': case 'f':
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
    case ',': e.preventDefault(); navigatePage(-1);  return;
    case '.': e.preventDefault(); navigatePage(1);   return;
    case '<': e.preventDefault(); navigatePage(-10); return;  // Shift+,
    case '>': e.preventDefault(); navigatePage(10);  return;  // Shift+.
  }

  // Arrow key edge adjustment — Single Adjust tool only, no modifiers
  if (tool.value !== 'singleadjust' || shift || alt) return;

  if (edge.value === 'top'    && e.key === 'ArrowDown') { e.preventDefault(); adjustEdge('top',    1);  return; }
  if (edge.value === 'top'    && e.key === 'ArrowUp')   { e.preventDefault(); adjustEdge('top',   -1);  return; }
  if (edge.value === 'bottom' && e.key === 'ArrowDown') { e.preventDefault(); adjustEdge('bottom', 1);  return; }
  if (edge.value === 'bottom' && e.key === 'ArrowUp')   { e.preventDefault(); adjustEdge('bottom',-1);  return; }
  if (edge.value === 'left'   && e.key === 'ArrowRight'){ e.preventDefault(); adjustEdge('left',   1);  return; }
  if (edge.value === 'left'   && e.key === 'ArrowLeft') { e.preventDefault(); adjustEdge('left',  -1);  return; }
  if (edge.value === 'right'  && e.key === 'ArrowRight'){ e.preventDefault(); adjustEdge('right',  1);  return; }
  if (edge.value === 'right'  && e.key === 'ArrowLeft') { e.preventDefault(); adjustEdge('right', -1);  return; }
}

onMounted(async () => {
  document.addEventListener('keydown', onKeyDown);
  try {
    const res  = await fetch(`/api/projects/${props.machineName}/pages`);
    const data = (await res.json()) as PageDb;
    pages.value = data.pages;
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
  grid-template-columns: 10rem 18rem 1fr 20rem;
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
.crop-area > div:last-child { border-right: none; }

.sidebar-lead {
  font-size: 0.75rem;
  font-weight: 600;
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

.crop-tools { padding: 0; }
</style>
