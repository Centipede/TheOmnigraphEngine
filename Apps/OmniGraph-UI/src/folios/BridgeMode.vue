<template>
  <PageWorkspace
    ref="workspaceRef"
    :machine-name="machineName"
    :project-name="projectName"
    :initial-page-stem="initialPageStem"
    :panels="panels"
    :project="project"
    :palette="grayHintPalette"
    :keyboard-handler="onKeyDown"
    hocr-initial-collapse-mode="block"
    @current-page-change="onPageChange"
  >
    <template #page-canvas="{ pages, currentPageIndex, scanBaseUrl, palette }">
      <div class="bridge-slider-container">
        <div
          class="bridge-slider"
          :style="getSliderStyle(currentPageIndex)"
        >
          <div
            v-for="page in getBufferedPages(pages, currentPageIndex)"
            :key="page.index"
            class="bridge-page-wrapper"
            :style="getPageStyle(page.index, currentPageIndex)"
          >
            <BridgePage
              :page="page"
              :machine-name="machineName"
              :image-base-url="scanBaseUrl"
              :palette="palette"
              :minimal="page.index !== currentPageIndex"
              :show-block-hints="true"
              :reload-trigger="hocrReloadTrigger[page.scan.replace(/\.[^.]+$/, '')]"
              :interaction-update="makeInteractionUpdateHandler(page)"
              :interaction-click="() => onInteractionClick(page)"
            />
          </div>
        </div>
      </div>
    </template>

    <template #tools>
      <BridgeDetectionTools :selected-block-id="selectedBlockId" />
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, provide } from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import BridgePage from './BridgePage.vue';
import BridgeDetectionTools from '../components/BridgeDetectionTools.vue';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { provideHocrContext } from '../composables/useHocr';
import type {HocrNode, OverlayItem, Page, Project, PageInteractionUpdate} from '../types';
import { DEFAULT_PALETTE } from '../types';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

const currentPage = ref<Page | null>(null);
const project = ref<Project | null>(null);

const selectedBlockId = ref<string | null>(null);
const flowSet = ref<Set<string>>(new Set());
const selectedPageScan = ref<string | null>(null);
const hoveredItemId = ref<string | null>(null);

const selectedItemIds = computed(() => {
  const ids = new Set<string>();
  if (selectedBlockId.value) {
    ids.add(selectedBlockId.value);
  }
  return ids;
});

provide('selectedItemIds', selectedItemIds);
provide('selectedPageScan', selectedPageScan);
provide('indicatedItemId', ref(null));
provide('flowSet', flowSet);
provide('project', project);


function makeInteractionUpdateHandler(page: Page) {
  return (...args: Parameters<PageInteractionUpdate>) => {
    onInteractionUpdate(page, ...args);
  };
}

function onInteractionUpdate(_page: Page, _x: number, _y: number, _other: OverlayItem[], activeItem: HocrNode | null, _between1: [HocrNode | null, HocrNode | null], _between2: [HocrNode | null, HocrNode | null]) {
  if (activeItem) {
    hoveredItemId.value = activeItem.id;
  } else {
    hoveredItemId.value = null;
  }
}

function onInteractionClick(page: Page) {
  if (hoveredItemId.value) {
    selectedBlockId.value = hoveredItemId.value;
    selectedPageScan.value = page.scan;
  } else {
    selectedBlockId.value = null;
    selectedPageScan.value = null;
  }
}

const hocrReloadTrigger = ref<Record<string, number>>({});

async function toggleHint(hintName: string) {
  if (!selectedBlockId.value || !selectedPageScan.value) return;

  const stem = selectedPageScan.value.replace(/\.[^.]+$/, '');
  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/blocks/${selectedBlockId.value}/toggle-hint/${hintName}`;

  try {
    const resp = await fetch(url, { method: 'POST' });
    if (resp.ok) {
      hocrReloadTrigger.value[stem] = (hocrReloadTrigger.value[stem] || 0) + 1;
    }
  } catch (e) {
    console.error('Failed to toggle hint:', e);
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    void toggleHint('continue_from_previous');
    return true;
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    void toggleHint('continue_to_following');
    return true;
  }
  return false;
}

const grayHintPalette = computed(() => {
  const basePalette = project.value?.editor_palette || DEFAULT_PALETTE;
  return {
    ...basePalette,
    hintDropcapColor: 'rgba(150, 150, 150, 1)',
    hintImageColor: 'rgba(150, 150, 150, 1)',
    hintCalloutColor: 'rgba(150, 150, 150, 1)',
    hintGarbageColor: 'rgba(150, 150, 150, 1)',
  };
});

async function fetchProjectMetadata(): Promise<void> {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      const data = await resp.json() as Project;
      project.value = data;
      if (data.flows && flowSet.value.size === 0) {
        flowSet.value = new Set(data.flows.map(f => f.name));
      }
    }
  } catch (e) {
    console.error('Failed to fetch project metadata:', e);
  }
}

const panels = usePersistentPanels('panels.bridge', {
  'page-list': true,
  'section-structure': false,

  'page-strips': false,
  'page-canvas': true,

  'tools': true,
  'ocr-structure': false,
  'structural-tree': false,
});
const { setActivePanels } = usePanelVisibilityContext();

// Provide HOCR context for PageWorkspace and general use. BridgeMode is special in that it works with multiple pages at once.
// But still there is a primary page relevant, namely the one in the center of the viewport, which is also the current page.
// Note though, that each inlined BridgePage uses its own HOCR context, which is loaded on demand.
provideHocrContext({ block_metrics: true });

onMounted(() => {
  setActivePanels(panels);
  void fetchProjectMetadata();
});

onUnmounted(() => {
  setActivePanels(null);
});

function onPageChange(page: Page | null) {
  currentPage.value = page;
}

function getBufferedPages(allPages: Page[], currentIdx: number | null) {
  if (currentIdx === null || !allPages.length) return [];
  
  const buffer = 3; // Keep a few pages around for smooth transition
  const centerPos = allPages.findIndex(p => p.index === currentIdx);
  if (centerPos === -1) return [];

  const start = Math.max(0, centerPos - buffer);
  const end = Math.min(allPages.length, centerPos + buffer + 1);
  
  return allPages.slice(start, end);
}

function getSliderStyle(currentIdx: number | null) {
  if (currentIdx === null) return {};
  
  // Each page wrapper is 33.333% wide.
  // We want the current page (at index currentIdx) to be centered in the viewport.
  // Viewport center is at 50%.
  // Page left edge is at currentIdx * 33.333%.
  // Page center is at (currentIdx + 0.5) * 33.333%.
  // Offset = 50% - (currentIdx + 0.5) * 33.333%
  
  const offset = 50 - (currentIdx + 0.5) * 33.333333;
  
  return {
    transform: `translateX(${offset}%)`,
    transition: 'transform 0.4s cubic-bezier(0.2, 0, 0.2, 1)',
    display: 'flex',
    height: '100%',
    width: '100%',
    position: 'relative' as const,
    willChange: 'transform'
  };
}

function getPageStyle(pageIndex: number, currentIdx: number | null) {
  if (currentIdx === null) return {};
  
  const diff = Math.abs(pageIndex - currentIdx);
  
  return {
    position: 'absolute' as const,
    left: `${pageIndex * 33.333333}%`,
    width: '33.333333%',
    height: '100%',
    zIndex: pageIndex === currentIdx ? 2 : 1,
    opacity: diff > 2 ? 0 : 1,
    transition: 'opacity 0.4s ease'
  };
}
</script>

<style scoped>
.bridge-slider-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: var(--color-bg-muted, #f1f3f5);
  display: flex;
  flex-direction: column;
}

.bridge-slider {
  flex: 1;
}

.bridge-page-wrapper {
  box-sizing: border-box;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
}
</style>
