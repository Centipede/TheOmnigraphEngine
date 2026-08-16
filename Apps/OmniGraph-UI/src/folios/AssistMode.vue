<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :show-crop-overlay="true"
  >
    <template #tools="{ selectedPages, currentPage }">
      <sl-button-group>
        <sl-button
            size="small"
            :disabled="!currentPage"
            @click="autoLayout(selectedPages, currentPage)"
        >
          Auto layout
        </sl-button>
        <sl-button
            size="small"
            :disabled="!currentPage"
            @click="autoFlow(selectedPages, currentPage)"
        >
          Auto flow
        </sl-button>
      </sl-button-group>
    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import PageWorkspace from "../components/PageWorkspace.vue";
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';
import type { Page } from '../types';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const panels = usePersistentPanels('panels.assist', {
  'page-list': true,
  'page-strips': true,
  'page-preview': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();

onMounted(() => setActivePanels(panels));
onUnmounted(() => setActivePanels(null));

const getStem = (p: Page) => p.scan.replace(/\.[^.]+$/, '');

async function autoLayout(selectedPages: Page[], currentPage: Page | null) {
  if (!currentPage) return;
  const stems = selectedPages.map(getStem);
  const currentStem = getStem(currentPage);
  await fetch(`/api/projects/${props.machineName}/pages/${currentStem}/auto-layout`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ stems })
  });
}

async function autoFlow(selectedPages: Page[], currentPage: Page | null) {
  if (!currentPage) return;
  const stems = selectedPages.map(getStem);
  const currentStem = getStem(currentPage);
  await fetch(`/api/projects/${props.machineName}/pages/${currentStem}/auto-flow`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ stems })
  });
}
</script>

<style scoped>

</style>
