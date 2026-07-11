<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      :show-crop-overlay="false"
      :hocr-level="ocrTool=='none' ? null : ocrTool"
      carea-overlay-color="rgba(249, 115, 22, 0.28)"
      block-overlay-color="rgba(168, 85, 247, 0.28)"
      line-overlay-color="rgba(59, 130, 246, 0.24)"
      word-overlay-color="rgba(34, 197, 94, 0.22)"
      @current-page-change="loadHocrPage"
      :page-interaction-update="pageInteractionUpdate"
  >
    <template #tools="{ currentPage }">

      <sl-button-group >
        <sl-button :variant="ocrTool==='none' ? 'primary' : 'default'" size="small"  @click="setOcrTool('none')">None</sl-button>
        <sl-button :variant="ocrTool==='carea' ? 'primary' : 'default'" size="small"  @click="setOcrTool('carea')">Carea</sl-button>
        <sl-button :variant="ocrTool==='block' ? 'primary' : 'default'" size="small"  @click="setOcrTool('block')">Block</sl-button>
        <sl-button :variant="ocrTool==='line' ? 'primary' : 'default'"  size="small" @click="setOcrTool('line')">Line</sl-button>
        <sl-button :variant="ocrTool==='word' ? 'primary' : 'default'" size="small"  @click="setOcrTool('word')">Word</sl-button>
      </sl-button-group>


<!--      <sl-button @click="testEditPage(currentPage)">Test Edit</sl-button>-->
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import {provide, type Ref, ref} from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import type {PanelVisibility, Page, HocrBbox} from '../types';
import type {HocrPage} from '../types/hocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

type OverlayRole = 'parent' | 'active' | 'child';

interface OverlayItem {
  id: string;
  bbox: HocrBbox;
  role: OverlayRole;
  color: string;
}

function pageInteractionUpdate(
    x: number,
    y: number,
    overlappingOverlayItems: OverlayItem[],
) {
  console.log('Page interaction update', x, y, overlappingOverlayItems);
}

const hocrPage = ref<HocrPage | null>(null);
provide('hocrPage', hocrPage);

type OcrTool = 'none' | 'carea' | 'block' | 'line' | 'word';

const ocrTool:Ref<OcrTool> = ref('none');

function setOcrTool(tool: OcrTool) {
  ocrTool.value = tool;
}

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
