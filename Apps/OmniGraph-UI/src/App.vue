<template>
  <div class="vue-app-root">
    <NavBar
        :machineName="machineName"
        :projectName="projectName"
        :panels="activePanels"
        @toggle-panel="togglePanel"
    />
    <RouterView v-slot="{ Component, route }">
      <component
          :is="Component"
          :panels="activePanels"
          :machine-name="machineName"
          :project-name="projectName"
          :initial-page-stem="route.params.page ? String(route.params.page) : undefined"
      />
    </RouterView>

    <sl-alert ref="errorAlertRef" variant="danger" closable duration="5000">
      <sl-icon slot="icon" name="exclamation-octagon"></sl-icon>
      <strong>Error</strong><br />
      {{ errorMessage }}
    </sl-alert>
  </div>
</template>

<script setup lang="ts">
import {computed, ref, provide, nextTick, onMounted, onUnmounted} from 'vue';
import {RouterView, useRoute} from 'vue-router';
import NavBar from "./components/NavBar.vue";
import type {PanelId} from './types';
import { providePanelVisibilityContext } from './composables/usePanelVisibility';
import { isTypingElement } from './utils/dom';

const route = useRoute();

const machineName = computed(() => String(route.params.machineName ?? ''));
const projectName = computed(() => machineName.value);

const { activePanels } = providePanelVisibilityContext();

const errorAlertRef = ref<HTMLElement & { toast: () => void } | null>(null);
const errorMessage = ref('');

function showError(msg: string) {
  errorMessage.value = msg;
  nextTick(() => {
    errorAlertRef.value?.toast();
  });
}

provide('showError', showError);

function togglePanel(panelId: PanelId) {
  if (!activePanels.value) return;
  activePanels.value[panelId] = !activePanels.value[panelId];
}

function handleAutoBlur(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const button = target.closest?.('button, [role="button"], sl-button, sl-radio-button, sl-switch, sl-checkbox');
  if (button && !isTypingElement(button)) {
    (button as HTMLElement).blur();
  }
}

onMounted(() => {
  document.addEventListener('mouseup', handleAutoBlur);
});

onUnmounted(() => {
  document.removeEventListener('mouseup', handleAutoBlur);
});

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
