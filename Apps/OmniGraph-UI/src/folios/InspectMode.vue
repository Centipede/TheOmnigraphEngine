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
import { reactive, onMounted, onUnmounted } from 'vue';
import PageWorkspace from "../components/PageWorkspace.vue";
import type { PanelVisibility } from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';

const props = defineProps<{
  machineName: string;
  projectName: string;
}>();

const panels = reactive<PanelVisibility>({
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
