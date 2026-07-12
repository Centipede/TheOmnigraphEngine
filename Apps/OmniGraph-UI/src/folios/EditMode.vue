<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      :show-crop-overlay="false"
      :hocr-level="ocrTool=='none' ? null : ocrTool"
      carea-overlay-color="rgba(249, 115, 22)"
      block-overlay-color="rgba(168, 85, 247)"
      line-overlay-color="rgba(59, 130, 246)"
      word-overlay-color="rgba(34, 197, 94)"
      :pointer-settings="{ color: pointerColor, label: pointerLabel, icon: pointerIcon, enabled: pointerEnabled }"
      @current-page-change="loadHocrPage"
      :page-interaction-update="pageInteractionUpdate"
  >
    <template #tools="{ currentPage }">

      <sl-button-group >
        <sl-button :variant="ocrMode==='none' ? 'primary' : 'default'" size="small"  @click="setOcrMode('none')">None</sl-button>
        <sl-button :variant="ocrMode==='select' ? 'primary' : 'default'" size="small"  @click="setOcrMode('select')">Select</sl-button>
        <sl-button :variant="ocrMode==='join' ? 'primary' : 'default'" size="small"  @click="setOcrMode('join')">Join</sl-button>
        <sl-button :variant="ocrMode==='split' ? 'primary' : 'default'"  size="small" @click="setOcrMode('split')">Split</sl-button>
        <sl-button :variant="ocrMode==='remove' ? 'primary' : 'default'" size="small"  @click="setOcrMode('remove')">Remove</sl-button>
      </sl-button-group>

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
import {computed, provide, type Ref, ref} from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import type {PanelVisibility, Page, OverlayItem, HocrSibling} from '../types';
import type {HocrPage} from '../types/hocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

const hocrPage = ref<HocrPage | null>(null);
provide('hocrPage', hocrPage);

type OcrTool = 'none' | 'carea' | 'block' | 'line' | 'word';
type OcrMode = 'none' | 'select' | 'join' | 'split' | 'remove';

const ocrTool:Ref<OcrTool> = ref('none');
const ocrMode:Ref<OcrMode> = ref('select');

const overTarget = ref<string | null>(null);
const betweenTargets = ref<[HocrSibling | null, HocrSibling | null]>([null,null]);
const betweenSubTargets = ref<[HocrSibling | null, HocrSibling | null]>([null,null]);

const pointerLabel = computed(() => {
  switch (ocrMode.value) {
    case 'none':
      return '';
    case 'select':
      return 'Select';
    case 'split':
      return 'Split';
    case 'join':
      return 'Join';
    case 'remove':
      return 'Remove';
  }
});
const pointerColor = computed(() => {
  switch (ocrMode.value) {
    case 'none':
      return '';
    case 'select':
      return '#2563eb'; // blue
    case 'split':
      return '#f97316'; // orange
    case 'join':
      return '#16a34a'; // green
    case 'remove':
      return '#dc2626'; // red
  }
});
const pointerIcon = computed(() => {
  switch (ocrMode.value) {
    case 'none':
      return '';
    case 'select':
      return 'crosshair';
    case 'split':
      return 'view-stacked';
    case 'join':
      return 'view-list';
    case 'remove':
      return 'x-square';
  }
});
const pointerEnabled = computed(() => {
  if(ocrMode.value === 'select' || ocrMode.value === 'remove')
    return overTarget.value != null;
  else if (ocrMode.value === 'split' || ocrMode.value === 'join') {
    return true;
  }
  return false;
})

function setOcrMode(mode: OcrMode) {
  ocrMode.value = mode;
}

function setOcrTool(tool: OcrTool) {
  ocrTool.value = tool;
}


function pageInteractionUpdate(
    x: number,
    y: number,
    overlappingOverlayItems: OverlayItem[],
    betweenOverlayItems: [HocrSibling | null, HocrSibling | null],
) {
  overTarget.value = null;
  betweenTargets.value = betweenOverlayItems

  for(const item of overlappingOverlayItems) {
    if (item.level == ocrTool.value) {
      overTarget.value = item.id
    }
  }
  //console.log('Page interaction update', x, y, overlappingOverlayItems);
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

window.addEventListener('keydown', refreshOperationalMode);
window.addEventListener('keyup', refreshOperationalMode);

function refreshOperationalMode(event: MouseEvent | KeyboardEvent) {
  if (event.altKey) {
    setOcrMode('remove');
    return;
  }

  if (event.ctrlKey) {
    return;
  }

  if (event.metaKey) {
    return;
  }

  if (event.shiftKey) {
    console.log(betweenTargets.value)
    if(betweenTargets.value[0] && betweenTargets.value[1]) {
      setOcrMode('join')
    }
    return;
  }

  setOcrMode('select');
}
</script>
