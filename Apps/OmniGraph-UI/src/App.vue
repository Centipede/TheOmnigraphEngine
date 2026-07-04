<template>
  <div class="vue-app-root">
    <NavBar
        :machineName="machineName"
        :projectName="projectName"
        :panels="panels"
        @toggle-panel="togglePanel"
    />
    <RouterView v-slot="{ Component }">
      <component
          :is="Component"
          :panels="panels"
      />
    </RouterView>
  </div>
</template>

<script setup lang="ts">
import {computed, reactive} from 'vue';
import {RouterView, useRoute} from 'vue-router';
import NavBar from "./components/NavBar.vue";
import type {PanelId, PanelVisibility} from './types';

const route = useRoute();

const machineName = computed(() => String(route.params.machineName ?? ''));
const projectName = computed(() => machineName.value);


const panels = reactive<PanelVisibility>({
  'page-list': true,
  'page-strips': true,
  'page-preview': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

function togglePanel(panelId: PanelId) {
  panels[panelId] = !panels[panelId];
}

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