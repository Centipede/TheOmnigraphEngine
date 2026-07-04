<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
  >
    <template #tools>
      <!-- Edit tools — to be implemented -->
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from '../components/PageWorkspace.vue';
import type { PanelVisibility } from '../types';

defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();
</script>
