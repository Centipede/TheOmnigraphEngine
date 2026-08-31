<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :show-crop-overlay="true"
      :flows="flows"
      :layouts="layouts"
      :hocr-level="'carea'"
      :show-layers="{ flow: true, layout: true }"
      :show-blocks="true"
      :palette="{ ...DEFAULT_PALETTE, keepColor: 'rgba(0, 0, 0, 0.0)', discardColor: 'rgba(50, 50, 50, 0.35)'}"
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
import { onMounted, onUnmounted, ref, computed } from 'vue';
import PageWorkspace from "../components/PageWorkspace.vue";
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';
import {DEFAULT_PALETTE, type Page, type Project} from '../types';

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

const project = ref<Project | null>(null);
const flows = computed(() => project.value?.flows || []);
const layouts = computed(() => project.value?.layouts || []);

async function fetchProjectMetadata(): Promise<void> {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      const data = await resp.json() as Project;
      project.value = data;
    }
  } catch (e) {
    console.error('Failed to fetch project metadata:', e);
  }
}

onMounted(() => {
  setActivePanels(panels);
  void fetchProjectMetadata();
});
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
