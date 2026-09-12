<template>
  <PageWorkspace
    ref="workspaceRef"
    :machine-name="machineName"
    :project-name="projectName"
    :initial-page-stem="initialPageStem"
    :panels="panels"
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
            />
          </div>
        </div>
      </div>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import BridgePage from './BridgePage.vue';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { provideHocrContext } from '../composables/useHocr';
import type { Page } from '../types';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

const workspaceRef = ref<InstanceType<typeof PageWorkspace> | null>(null);
const currentPage = ref<Page | null>(null);

const panels = usePersistentPanels('panels.bridge', {
  'page-list': true,
  'page-strips': false,
  'page-canvas': true,
  'section-structure': false,
  'ocr-structure': false,
  tools: false,
});

const { setActivePanels } = usePanelVisibilityContext();

// Provide HOCR context for PageWorkspace and general use
provideHocrContext();

onMounted(() => {
  setActivePanels(panels);
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
