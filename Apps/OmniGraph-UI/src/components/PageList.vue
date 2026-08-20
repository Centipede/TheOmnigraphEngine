<template>
  <div class="page-list-panel">

    <sl-button-group>
      <sl-button size="small" title="Back (,)" @click="$emit('navigate', -1)">
        ←
      </sl-button>
      <sl-button size="small" title="Forward (.)" @click="$emit('navigate', 1)">
        →
      </sl-button>
      <sl-button size="small" title="Select all" @click="$emit('selectAll')">
        +
      </sl-button>
      <sl-button size="small" title="Select None" @click="$emit('selectNone')">
        -
      </sl-button>
    </sl-button-group>

    <ul
        ref="pageListRef"
        class="page-nav-list"
        @mousedown.shift.prevent
    >
      <li v-if="showHeader" class="page-nav-header">
        <b v-if="showIndexCol" class="page-nav-info">Idx</b>
        <b v-if="showBatchCol" class="page-nav-info">Bat</b>
        <b v-if="showNameCol || showNameOrScanCol" class="page-nav-main">Name</b>
        <b v-if="showScanCol" class="page-nav-info">Scan</b>
        <b v-if="showExtrasCol" class="page-nav-extra">Extras</b>
      </li>

      <li
          v-for="page in pages"
          :key="page.index"
          :data-page-idx="page.index"
          :class="{
            'page-nav-selected': selectedPageIndices.has(page.index),
            'page-nav-focused': page.index === currentPageIndex,
            'page-nav-named': !!page.name,
            'page-nav-filtered-out': !isPageInFilter(page),
          }"
          @click="$emit('page-click', page.index, $event)"
      >
        <span v-if="showIndexCol" class="page-nav-info">
          ({{ page.index }})
        </span>

        <span v-if="showBatchCol" class="page-nav-info">
          {{ page.batch }}
        </span>

        <span v-if="showNameCol" class="page-nav-main">
          <span v-if="page.name">{{ page.name }}</span>
          <em v-else class="page-nav-unnamed">[No name]</em>
        </span>

        <span v-if="showNameOrScanCol" class="page-nav-main">
          <span v-if="page.name">{{ page.name }}</span>
          <em v-else class="page-nav-unnamed">{{ page.scan }}</em>
        </span>

        <span v-if="showScanCol" class="page-nav-info">
          {{ page.scan }}
        </span>

        <span v-if="showExtrasCol && pageExtras?.get(page.index)" class="page-nav-extra">
          {{ pageExtras.get(page.index) }}
        </span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import {computed, nextTick, ref} from 'vue';
import type { Page } from '../types';

type Column = "index" | "batch" | "name" | "scan" | "name-or-scan" | "extras";


const props = defineProps<{
  pages: Page[];
  selectedPageIndices: Set<number>;
  currentPageIndex: number | null;
  isPageInFilter: (page: Page) => boolean;
  columns: Column[];
  pageExtras?: Map<number, string>;
}>();

defineEmits<{
  navigate: [delta: number];
  selectAll: [];
  selectNone: [];
  'page-click': [pageIndex: number, event: MouseEvent];
}>();

const pageListRef = ref<HTMLElement | null>(null);

const showIndexCol = computed(() => props.columns.includes("index"));
const showBatchCol = computed(() => props.columns.includes("batch"));
const showNameCol = computed(() => props.columns.includes("name"));
const showNameOrScanCol = computed(() => props.columns.includes("name-or-scan"));
const showScanCol = computed(() => props.columns.includes("scan"));
const showExtrasCol = computed(() => props.columns.includes("extras"));

const showHeader = computed(() => {
  return props.columns.some(column => column !== "name-or-scan");
});

async function scrollPageIntoView(pageIndex: number) {
  await nextTick();

  pageListRef.value
      ?.querySelector<HTMLElement>(`[data-page-idx="${pageIndex}"]`)
      ?.scrollIntoView({
        block: 'nearest',
        behavior: 'smooth',
      });
}

defineExpose({
  scrollPageIntoView,
});
</script>

<style scoped>

.page-list-panel {
  min-height: 0;
  overflow-y: auto;
}

.page-nav-empty {
  padding: 0.5rem 0.75rem;
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-text-muted, #6c757d);
}

.page-nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  font-size: 0.75rem;
  user-select: none;
  -webkit-user-select: none;
}

.page-nav-list li {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  overflow: hidden;
  color: var(--color-text-muted, #6c757d);
  border-left: 2px solid transparent;
}

.page-nav-header {
  cursor: default;
  color: var(--color-text, #212529);
  background: var(--color-surface, #fff);
  border-bottom: 1px solid var(--color-border, #dee2e6);
  position: sticky;
  top: 0;
  z-index: 1;
}

.page-nav-header:hover {
  background: var(--color-surface, #fff);
}

.page-nav-info {
  flex: 0 0 auto;
  min-width: 2.25rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.page-nav-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.page-nav-extra {
  flex-shrink: 0;
  font-size: 0.7em;
  font-weight: 700;
  color: var(--color-accent, #2563eb);
  opacity: 0.75;
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
}

.page-nav-focused {
  border-left-color: var(--color-accent, #2563eb) !important;
}

.page-nav-unnamed {
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.page-nav-filtered-out {
  opacity: 0.35;
  pointer-events: none;
}

</style>