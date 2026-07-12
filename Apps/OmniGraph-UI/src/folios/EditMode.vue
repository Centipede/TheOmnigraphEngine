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
      :page-interaction-click="pageInteractionClick"
  >
    <template #tools>

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
import {computed, onMounted, onUnmounted, provide, type Ref, ref} from 'vue';
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

const currentStem = ref<string | null>(null);

type OcrTool = 'none' | 'carea' | 'block' | 'line' | 'word';
type OcrMode = 'none' | 'select' | 'join' | 'split' | 'remove';

const ocrTool:Ref<OcrTool> = ref('none');
const ocrMode:Ref<OcrMode> = ref('select');

const overTarget = ref<string | null>(null);
const selectedTarget = ref<string | null>(null);
const betweenTargets = ref<[HocrSibling | null, HocrSibling | null]>([null, null]);
const betweenSubTargets = ref<[HocrSibling | null, HocrSibling | null]>([null, null]);

// ── Modifier key state ───────────────────────────────────────────────
const shiftDown = ref(false);
const altDown   = ref(false);
const metaDown  = ref(false);
const ctrlDown  = ref(false);

// ── Effective mode (modifier keys override manual ocrMode) ────────────
const effectiveOcrMode = computed<OcrMode>(() => {
  if (ocrTool.value === 'none') return 'none';
  if (altDown.value) return 'remove';
  if (shiftDown.value) {
    if (betweenTargets.value[0] !== null && betweenTargets.value[1] !== null) return 'join';
    if (betweenSubTargets.value[0] !== null && betweenSubTargets.value[1] !== null) return 'split';
    return 'none';
  }
  return 'select';
});

const pointerLabel = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return '';
    case 'select': return 'Select';
    case 'split':  return 'Split';
    case 'join':   return 'Join';
    case 'remove': return 'Remove';
  }
});

const pointerColor = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return '';
    case 'select': return '#2563eb';
    case 'split':  return '#f97316';
    case 'join':   return '#16a34a';
    case 'remove': return '#dc2626';
  }
});

const pointerIcon = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return '';
    case 'select': return 'crosshair';
    case 'split':  return 'view-stacked';
    case 'join':   return 'view-list';
    case 'remove': return 'x-square';
  }
});

const pointerEnabled = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'select':
    case 'remove': return overTarget.value !== null;
    case 'join':   return betweenTargets.value[0] !== null && betweenTargets.value[1] !== null;
    case 'split':  return betweenSubTargets.value[0] !== null && betweenSubTargets.value[1] !== null;
    default:       return false;
  }
});

function setOcrMode(mode: OcrMode) {
  ocrMode.value = mode;
}

function setOcrTool(tool: OcrTool) {
  ocrTool.value = tool;
}


function pageInteractionUpdate(
    _x: number,
    _y: number,
    overlappingOverlayItems: OverlayItem[],
    _activeItem: HocrSibling | null,
    betweenOverlayItems: [HocrSibling | null, HocrSibling | null],
    betweenOverlaySubItems: [HocrSibling | null, HocrSibling | null],
) {
  overTarget.value = null;
  betweenTargets.value = betweenOverlayItems;
  betweenSubTargets.value = betweenOverlaySubItems;

  for (const item of overlappingOverlayItems) {
    if (item.level === ocrTool.value) {
      overTarget.value = item.id;
    }
  }
}

const LEVEL_SEGMENT: Record<string, string> = {
  carea: 'careas', block: 'blocks', line: 'lines', word: 'words',
};

async function callHocrEndpoint(id: string, action: string, body?: object): Promise<void> {
  const stem = currentStem.value;
  const level = ocrTool.value;
  if (!stem || level === 'none') return;
  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/${LEVEL_SEGMENT[level]}/${id}/${action}`;
  await fetch(url, body
      ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
      : { method: 'POST' });
}

async function pageInteractionClick(): Promise<void> {
  const mode = effectiveOcrMode.value;
  if (ocrTool.value === 'none' || mode === 'none') return;

  if (mode === 'select') {
    selectedTarget.value = overTarget.value;
    return;
  }
  if (mode === 'remove' && overTarget.value) {
    await callHocrEndpoint(overTarget.value, 'remove');
    if (selectedTarget.value === overTarget.value) selectedTarget.value = null;
  } else if (mode === 'join' && betweenTargets.value[0] && betweenTargets.value[1]) {
    await callHocrEndpoint(betweenTargets.value[0].id, 'merge', { other_id: betweenTargets.value[1].id });
  } else if (mode === 'split' && overTarget.value && betweenSubTargets.value[0] && betweenSubTargets.value[1]) {
    await callHocrEndpoint(overTarget.value, 'split',
        { before_id: betweenSubTargets.value[0].id, after_id: betweenSubTargets.value[1].id });
  }
}

function isTypingTarget(): boolean {
  const el = document.activeElement as HTMLElement | null;
  if (!el) return false;
  return ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName) || el.isContentEditable;
}

async function handleKeyboardAction(e: KeyboardEvent): Promise<void> {
  if (!selectedTarget.value || isTypingTarget()) return;
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    await callHocrEndpoint(selectedTarget.value, 'move-up');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    await callHocrEndpoint(selectedTarget.value, 'move-down');
  } else if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault();
    await callHocrEndpoint(selectedTarget.value, 'remove');
    selectedTarget.value = null;
  }
}

// async function testEditPage(page: Page | null): Promise<void> {
//   const resp = await fetch(`/api/projects/${props.machineName}/pages/${page.scan}/test-edit`, {
//     method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({page}),
//   });
//   if (resp.ok && (await resp.json() as { success: boolean }).success) alert('Page edited!');
// }

async function loadHocrPage(page: Page | null): Promise<void> {
  if (!page) {
    hocrPage.value = null;
    currentStem.value = null;
    return;
  }
  const stem = page.scan.replace(/\.[^.]+$/, '');
  currentStem.value = stem;
  try {
    const resp = await fetch(`/api/projects/${props.machineName}/pages/${stem}/hocr-json`);
    hocrPage.value = resp.ok ? (await resp.json() as HocrPage) : null;
  } catch {
    hocrPage.value = null;
  }
}

function updateModifiers(e: KeyboardEvent) {
  shiftDown.value = e.shiftKey;
  altDown.value   = e.altKey;
  metaDown.value  = e.metaKey;
  ctrlDown.value  = e.ctrlKey;
}

onMounted(() => {
  window.addEventListener('keydown', updateModifiers);
  window.addEventListener('keyup',   updateModifiers);
  window.addEventListener('keydown', handleKeyboardAction);
});

onUnmounted(() => {
  window.removeEventListener('keydown', updateModifiers);
  window.removeEventListener('keyup',   updateModifiers);
  window.removeEventListener('keydown', handleKeyboardAction);
});
</script>
