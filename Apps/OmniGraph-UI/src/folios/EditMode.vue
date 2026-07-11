<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      :show-crop-overlay="false"
      hocr-level="block"
      carea-overlay-color="rgba(249, 115, 22, 0.28)"
      block-overlay-color="rgba(168, 85, 247, 0.28)"
      line-overlay-color="rgba(59, 130, 246, 0.24)"
      word-overlay-color="rgba(34, 197, 94, 0.22)"
      @current-page-change="loadHocrPage"
  >
    <template #tools="{ currentPage }">
      <!-- Edit tools — to be implemented -->

      <sl-button @click="testEditPage(currentPage)">Test Edit</sl-button>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import {provide, ref} from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import type {PanelVisibility, Page} from '../types';
import type {HocrPage} from '../types/hocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

const hocrPage = ref<HocrPage | null>(null);
provide('hocrPage', hocrPage);

async function testEditPage(page: Page | null): Promise<void> {
  if (!page) {
    return
  }

  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/${page.scan}/test-edit`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({page}),
    });
    if (resp.ok) {
      const data = await resp.json() as { success: boolean };
      if (data.success) {
        alert('Page edited successfully!');
      } else {
      }
    }
  } catch (e) {
    console.error(e);
  }
}

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
