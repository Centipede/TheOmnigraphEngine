<template>
  <div class="vue-app-root">
    <NavBar
        :machineName="machineName"
        :projectName="projectName"
        :panels="activePanels"
        @toggle-panel="togglePanel"
    />
    <RouterView v-slot="{ Component }">
      <component
          :is="Component"
          :panels="activePanels"
      />
    </RouterView>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {RouterView, useRoute} from 'vue-router';
import NavBar from "./components/NavBar.vue";
import type {PanelId} from './types';
import { providePanelVisibilityContext } from './composables/usePanelVisibility';

const route = useRoute();

const machineName = computed(() => String(route.params.machineName ?? ''));
const projectName = computed(() => machineName.value);

const { activePanels } = providePanelVisibilityContext();

function togglePanel(panelId: PanelId) {
  if (!activePanels.value) return;
  activePanels.value[panelId] = !activePanels.value[panelId];
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
