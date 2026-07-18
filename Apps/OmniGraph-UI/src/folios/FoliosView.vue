<!-- src/folios/FoliosView.vue -->
<template>
  <RouterView v-slot="{ Component }">
    <component
        :is="Component"
        :machine-name="machineName"
        :project-name="projectName"
    />
  </RouterView>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';

defineProps<{
  machineName: string;
  projectName: string;
}>();
</script>
