<!-- src/folios/FoliosView.vue -->
<template>
  <RouterView v-slot="{ Component }">
    <component
        :is="Component"
        :machine-name="machineName"
        :project-name="projectName"
        :panels="panels"
    />
  </RouterView>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';
import type { PanelVisibility } from '../types';

defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();
</script>
