<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      @current-page-change="loadHocrPage"
  >
    <template #tools>
      <!-- Edit tools — to be implemented -->
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { provide, ref } from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import type { PanelVisibility, Page } from '../types';
import type { HocrPage } from '../types/hocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

const hocrPage = ref<HocrPage | null>(null);
provide('hocrPage', hocrPage);

async function loadHocrPage(page: Page | null): Promise<void> {
  if (!page) {
    hocrPage.value = null;
    return;
  }
  const stem = page.scan.replace(/\.[^.]+$/, '');
  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/${stem}/hocr-json`);
    hocrPage.value = resp.ok ? (await resp.json() as HocrPage) : null;
  } catch {
    hocrPage.value = null;
  }
}
</script>
