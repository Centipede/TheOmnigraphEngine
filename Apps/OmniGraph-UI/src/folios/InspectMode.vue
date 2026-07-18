<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      :show-crop-overlay="true"
  >
    <template #tools>

    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import PageWorkspace from "../components/PageWorkspace.vue";
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';

const props = defineProps<{
  machineName: string;
  projectName: string;
}>();

const panels = usePersistentPanels('panels.inspect', {
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
</script>

<style scoped>

</style>
