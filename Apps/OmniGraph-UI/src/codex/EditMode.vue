<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
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
import { onMounted, onUnmounted } from 'vue';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const panels = usePersistentPanels('panels.codex-edit', {
  'page-list': true,
  'page-strips': true,
  'page-preview': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();

onMounted(() => {
  setActivePanels(panels);
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
