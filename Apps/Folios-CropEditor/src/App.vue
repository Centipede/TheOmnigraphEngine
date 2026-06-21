<template>
  <div class="vue-app-root">
    <NavBar :machineName="machineName" :projectName="projectName"/>
    <RouterView/>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {RouterView, useRoute} from 'vue-router';
import NavBar from "./components/NavBar.vue";

const route = useRoute();

const machineName = computed(() => String(route.params.machineName ?? ''));
const projectName = computed(() => machineName.value);
</script>

<style>
.vue-app-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--color-bg, #f8f9fa);
  font-family: var(--sl-font-sans, sans-serif);
  color: var(--color-text, #212529);
}
</style>