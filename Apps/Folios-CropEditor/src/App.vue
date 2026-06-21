<template>
  <div class="vue-app-root">
    <NavBar :machineName="machineName" :projectName="projectName" />
    <CropMode :machine-name="machineName" :project-name="projectName" />
  </div>
</template>

<script setup lang="ts">
import NavBar from "./components/NavBar.vue";
import CropMode from "./folios/CropMode.vue";

defineProps<{ machineName: string; projectName: string }>();

</script>

<style>
.vue-app-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: var(--color-bg, #f8f9fa);
  font-family: var(--sl-font-sans, sans-serif);
  color: var(--color-text, #212529);
}
</style>