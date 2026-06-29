<!-- src/folios/FoliosView.vue -->
<template>
  <RouterView />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';

defineProps<{ machineName: string; projectName: string }>();
</script>