<!-- src/folios/FoliosView.vue -->
<template>
  <RouterView v-slot="{ Component, route }">
    <component
        :is="Component"
        :machine-name="machineName"
        :project-name="projectName"
        :initial-page-stem="route.params.page ? String(route.params.page) : undefined"
    />
  </RouterView>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';

defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();
</script>
