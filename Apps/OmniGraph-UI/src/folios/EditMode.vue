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
    <template #tools="{ currentPage }">

      <div class="ocr-info-panel">
        <div class="ocr-info-row">
          <span class="ocr-info-label">Mode</span>
          <span class="ocr-info-value">{{ pointerLabel || 'None' }}</span>
        </div>
        <div class="ocr-info-row">
          <span class="ocr-info-label">Target</span>
          <span class="ocr-info-value">{{ ocrTool }}</span>
        </div>
        <div class="ocr-info-row">
          <span class="ocr-info-label">Keys</span>
          <span class="ocr-info-value">Shift join/split · {{ isMac ? 'Alt' : 'Ctrl' }} remove</span>
        </div>
        <div class="ocr-info-row">
          <span class="ocr-info-label">Selected</span>
          <span class="ocr-info-value">
            <template v-if="selectedTarget">{{ selectedTarget?.id }} ({{ selectedTarget?.level }})</template></span>
        </div>
      </div>

      <sl-button @click="restoreFromOriginal(currentPage)">Restore</sl-button>

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
import {type Page, type OverlayItem, type HocrNode, findItem} from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import type {HocrPage} from '../types/hocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
}>();

const panels = usePersistentPanels('panels.edit', {
  'page-list': true,
  'page-strips': true,
  'page-preview': false,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();

const hocrPage = ref<HocrPage | null>(null);
provide('hocrPage', hocrPage);

const currentStem = ref<string | null>(null);

type OcrTool = 'none' | 'carea' | 'block' | 'line' | 'word';
type OcrMode = 'none' | 'select' | 'join' | 'split' | 'remove';

const ocrTool:Ref<OcrTool> = ref('none');
const ocrMode:Ref<OcrMode> = ref('select');

const overItemId = ref<string | null>(null);
const selectedItemId = ref<string | null>(null);
const selectedTarget = computed(() => selectedItemId.value && hocrPage.value ? findItem(hocrPage.value, selectedItemId.value) : null);
const betweenTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);
const betweenSubTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);

// ── Modifier key state ───────────────────────────────────────────────
const shiftDown = ref(false);
const altDown   = ref(false);
const metaDown  = ref(false);
const ctrlDown  = ref(false);
const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);

// ── Effective mode (modifier keys override manual ocrMode) ────────────
const effectiveOcrMode = computed<OcrMode>(() => {
  if (ocrTool.value === 'none') return 'none';
  if (isMac ? altDown.value : ctrlDown.value) return 'remove';
  if (shiftDown.value) {
    if (betweenTargets.value[0] !== null && betweenTargets.value[1] !== null) return 'join';
    if (betweenSubTargets.value[0] !== null && betweenSubTargets.value[1] !== null) return 'split';
    return 'none';
  }
  return 'select';
});

const pointerLabel = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return 'No action';
    case 'select': return 'Select';
    case 'split':  return 'Split';
    case 'join':   return 'Join';
    case 'remove': return 'Remove';
  }
});

const pointerColor = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return '#000000';
    case 'select': return '#2563eb';
    case 'split':  return '#f97316';
    case 'join':   return '#16a34a';
    case 'remove': return '#dc2626';
  }
});

const pointerIcon = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return 'question-lg';
    case 'select': return 'crosshair';
    case 'split':  return 'view-stacked';
    case 'join':   return 'view-list';
    case 'remove': return 'x-square';
  }
});

const pointerEnabled = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'select':
    case 'remove': return overItemId.value !== null;
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
    _activeItem: HocrNode | null,
    betweenOverlayItems: [HocrNode | null, HocrNode | null],
    betweenOverlaySubItems: [HocrNode | null, HocrNode | null],
) {
  overItemId.value = null;
  betweenTargets.value = betweenOverlayItems;
  betweenSubTargets.value = betweenOverlaySubItems;

  for (const item of overlappingOverlayItems) {
    if (item.level === ocrTool.value) {
      overItemId.value = item.id;
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
  const resp = await fetch(url, body
      ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
      : { method: 'POST' });

  if (resp.ok) {
    hocrPage.value = resp.ok ? (await resp.json() as HocrPage) : null;
  }
}

async function pageInteractionClick(): Promise<void> {
  console.log('pageInteractionClick', ocrTool.value, effectiveOcrMode.value);
  const mode = effectiveOcrMode.value;
  if (ocrTool.value === 'none' || mode === 'none') return;

  if (mode === 'select') {
    selectedItemId.value = overItemId.value;
    return;
  }
  if (mode === 'remove' && overItemId.value) {
    await callHocrEndpoint(overItemId.value, 'remove');
    if (selectedItemId.value === overItemId.value) selectedItemId.value = null;
  } else if (mode === 'join' && betweenTargets.value[0] && betweenTargets.value[1]) {
    await callHocrEndpoint(betweenTargets.value[0].id, 'merge', { other_id: betweenTargets.value[1].id });
  } else if (mode === 'split' && overItemId.value && betweenSubTargets.value[0] && betweenSubTargets.value[1]) {
    await callHocrEndpoint(overItemId.value, 'split',
        { before_id: betweenSubTargets.value[0].id, after_id: betweenSubTargets.value[1].id });
  }
}

function isTypingTarget(): boolean {
  const el = document.activeElement as HTMLElement | null;
  if (!el) return false;
  return ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName) || el.isContentEditable;
}

async function handleKeyboardAction(e: KeyboardEvent): Promise<void> {
  if (!selectedItemId.value || isTypingTarget()) return;
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    await callHocrEndpoint(selectedItemId.value, 'move-up');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    await callHocrEndpoint(selectedItemId.value, 'move-down');
  } else if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault();
    await callHocrEndpoint(selectedItemId.value, 'remove');
    selectedItemId.value = null;
  }
}

async function restoreFromOriginal(page: Page | null): Promise<void> {
  if(! page) {
    return;
  }
  const resp = await fetch(`/api/projects/${props.machineName}/pages/${page.scan}/restore-original`, {
    method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({page}),
  });
  if (resp.ok) {
    hocrPage.value = resp.ok ? (await resp.json() as HocrPage) : null;
  }
}

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
  setActivePanels(panels);
  window.addEventListener('keydown', updateModifiers);
  window.addEventListener('keyup',   updateModifiers);
  window.addEventListener('keydown', handleKeyboardAction);
});

onUnmounted(() => {
  setActivePanels(null);
  window.removeEventListener('keydown', updateModifiers);
  window.removeEventListener('keyup',   updateModifiers);
  window.removeEventListener('keydown', handleKeyboardAction);
});

</script>

<style scoped>
.ocr-info-panel {
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 0.375rem;
  padding: 0.5rem;
  font-size: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  margin-bottom: 0.75rem;
}

.ocr-info-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.ocr-info-label {
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.ocr-info-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>