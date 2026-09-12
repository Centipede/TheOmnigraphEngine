<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :palette="grayHintPalette"
      :is-no-hocr-acceptable="false"
  >
    <template #tools="{ currentPage }">
      <div class="codex-tools">
        <h3>Codex Edit Mode</h3>
        <p v-if="currentPage">Editing: {{ currentPage.scan }}</p>
        <p v-else>Select a page to edit.</p>
      </div>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from '../components/PageWorkspace.vue';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { provideHocrContext } from '../composables/useHocr';
import {computed, onMounted, onUnmounted, ref} from 'vue';
import {DEFAULT_PALETTE, type Project} from "../types";

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const project = ref<Project | null>(null);

async function fetchProject() {
  if (!props.machineName) return;
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      project.value = await resp.json();
    }
  } catch (e) {
    console.error('Failed to fetch project in Codex EditMode', e);
  }
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

const panels = usePersistentPanels('panels.codex-edit', {
  'page-list': true,
  'page-strips': true,
  'page-canvas': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();

onMounted(() => {
  setActivePanels(panels);
  fetchProject();
});

onUnmounted(() => {
  setActivePanels(null);
});
</script>

<style scoped>
.codex-tools {
  padding: 1rem;
}
</style>
