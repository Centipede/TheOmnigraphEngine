<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
      :show-crop-overlay="false"
      :hocr-level="ocrTool=='multi' ? null : ocrTool"
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
        <template v-if="ocrTool === 'multi'">
          <div class="ocr-info-row">
            <span class="ocr-info-label">Carea</span>
            <span class="ocr-info-value ocr-info-id">{{ multiSelect?.carea?.id ?? '—' }}</span>
          </div>
          <div class="ocr-info-row">
            <span class="ocr-info-label">Block</span>
            <span class="ocr-info-value ocr-info-id">{{ multiSelect?.block?.id ?? '—' }}</span>
          </div>
          <div class="ocr-info-row">
            <span class="ocr-info-label">Line</span>
            <span class="ocr-info-value ocr-info-id">{{ multiSelect?.line?.id ?? '—' }}</span>
          </div>
          <div class="ocr-info-row">
            <span class="ocr-info-label">Word</span>
            <span class="ocr-info-value">{{ multiSelect?.word?.text ?? '—' }}</span>
          </div>
        </template>
        <template v-else>
          <div class="ocr-info-row">
            <span class="ocr-info-label">Selected</span>
            <span class="ocr-info-value ocr-info-id">
              <template v-if="selectedTarget">{{ selectedTarget.id }} ({{ selectedTarget.level }})</template>
              <template v-else>—</template>
            </span>
          </div>
        </template>
      </div>

      <div class="tool-palette">
        <sl-button @click="restoreFromOriginal(currentPage)" size="small" >Restore</sl-button>

        <sl-button-group>
          <sl-button :variant="ocrTool==='multi' ? 'primary' : 'default'" size="small"  @click="setOcrTool('multi')"><sl-icon name="eyedropper"></sl-icon></sl-button>
          <sl-button :variant="ocrTool==='carea' ? 'primary' : 'default'" size="small"  @click="setOcrTool('carea')">Carea</sl-button>
          <sl-button :variant="ocrTool==='block' ? 'primary' : 'default'" size="small"  @click="setOcrTool('block')">Block</sl-button>
          <sl-button :variant="ocrTool==='line' ? 'primary' : 'default'"  size="small" @click="setOcrTool('line')">Line</sl-button>
          <sl-button :variant="ocrTool==='word' ? 'primary' : 'default'" size="small"  @click="setOcrTool('word')">Word</sl-button>
        </sl-button-group>

        <sl-button-group v-if="ocrTool === 'block'">
          <sl-button
              v-for="bk in BLOCK_KINDS"
              :key="bk.kind"
              size="small"
              :disabled="!selectedItemId"
              @click="changeBlockType(bk.kind)"
          >{{ bk.label }}<template v-if="bk.key"> <span class="kind-key">{{ bk.key }}</span></template></sl-button>
        </sl-button-group>

        <sl-button-group v-if="ocrTool!=='multi'">
          <sl-button :variant="ocrMode==='context' ? 'primary' : 'default'" size="small"  @click="setOcrMode('context')">By Context</sl-button>
          <sl-button :variant="effectiveOcrMode==='select' ? (ocrMode!=='context' ? 'primary' : 'secondary') : 'default'" size="small"  @click="setOcrMode('select')">Select</sl-button>
          <sl-button :variant="effectiveOcrMode==='join' ? (ocrMode!=='context' ? 'primary' : 'secondary') : 'default'" size="small"  @click="setOcrMode('join')">Join</sl-button>
          <sl-button :variant="effectiveOcrMode==='split' ? (ocrMode!=='context' ? 'primary' : 'secondary') : 'default'"  size="small" @click="setOcrMode('split')">Split</sl-button>
          <sl-button :variant="effectiveOcrMode==='remove' ? (ocrMode!=='context' ? 'primary' : 'secondary') : 'default'" size="small"  @click="setOcrMode('remove')">Remove</sl-button>
        </sl-button-group>
      </div>

    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import {computed, onMounted, onUnmounted, provide, type Ref, ref} from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import {
  type Page, type OverlayItem, type HocrNode,
  findItem, findMultiLevelItemByPoint, type MultiSelect,
} from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import {type HocrPage} from '../types/hocr';

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

type OcrTool = 'multi' | 'carea' | 'block' | 'line' | 'word';
type OcrMode = 'context' | 'none' | 'select' | 'join' | 'split' | 'remove';

const ocrTool:Ref<OcrTool> = ref('multi');
const ocrMode:Ref<OcrMode> = ref('context');

const overItemId = ref<string | null>(null);
const selectedItemId = ref<string | null>(null);
provide('selectedItemId', selectedItemId);
const selectedTarget = computed(() => selectedItemId.value && hocrPage.value ? findItem(hocrPage.value, selectedItemId.value) : null);
const betweenTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);
const betweenSubTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);

// Multi-select: live hover stack (updated on mousemove) and committed selection (set on click).
const multiHover = ref<MultiSelect | null>(null);
const multiSelect = ref<MultiSelect | null>(null);

// ── Modifier key state ───────────────────────────────────────────────
const shiftDown = ref(false);
const altDown   = ref(false);
const metaDown  = ref(false);
const ctrlDown  = ref(false);
const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);

// ── Effective mode (modifier keys override manual ocrMode) ────────────
const effectiveOcrMode = computed<OcrMode>(() => {
  if (ocrMode.value !== 'context') return ocrMode.value;

  if (ocrTool.value === 'multi') return 'select';
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
  return '';
});

const pointerColor = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return '#000000';
    case 'select': return '#2563eb';
    case 'split':  return '#f97316';
    case 'join':   return '#16a34a';
    case 'remove': return '#dc2626';
  }
  return '';
});

const pointerIcon = computed(() => {
  switch (effectiveOcrMode.value) {
    case 'none':   return 'question-lg';
    case 'select': return 'crosshair';
    case 'split':  return 'view-stacked';
    case 'join':   return 'view-list';
    case 'remove': return 'x-square';
  }
  return '';
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

function setOcrTool(tool: OcrTool) {
  ocrTool.value = tool;
  if (ocrTool.value === 'multi') {
    if(['split', 'join', 'remove'].includes(ocrMode.value)) {
      setOcrMode('context');
    }
  }
  else {
    if (ocrMode.value == 'none') {
      setOcrMode('context');
    }
  }
}

function setOcrMode(mode: OcrMode) {
  ocrMode.value = mode;
}

function pageInteractionUpdate(
    x: number,
    y: number,
    overlappingOverlayItems: OverlayItem[],
    _activeItem: HocrNode | null,
    betweenOverlayItems: [HocrNode | null, HocrNode | null],
    betweenOverlaySubItems: [HocrNode | null, HocrNode | null],
) {
  overItemId.value = null;
  betweenTargets.value = betweenOverlayItems;
  betweenSubTargets.value = betweenOverlaySubItems;

  if (ocrTool.value === 'multi') {
    multiHover.value = findMultiLevelItemByPoint(hocrPage.value!, x, y);
  }
  else {
    for (const item of overlappingOverlayItems) {
      if (item.level === ocrTool.value) {
        overItemId.value = item.id;
      }
    }
  }
}

const LEVEL_SEGMENT: Record<string, string> = {
  carea: 'careas', block: 'blocks', line: 'lines', word: 'words',
};

// 0–6 → hOCR block kind strings. Keys match the task spec.
const BLOCK_KIND_KEYS: Record<string, string> = {
  '0': 'part',                // H1 + class='ocr_part'
  '1': 'chapter',             // H1
  '2': 'section',             // H2
  '3': 'subsection',          // H3
  '4': 'subsubsection',       // H4
  '5': 'subsubsubsection',    // H5
  '6': 'subsubsubsubsection', // H6
};

// Ordered list for the button palette, including paragraph.
const BLOCK_KINDS = [
  { key: '0', kind: 'part',                 label: 'Part' },
  { key: '1', kind: 'chapter',              label: 'H1' },
  { key: '2', kind: 'section',              label: 'H2' },
  { key: '3', kind: 'subsection',           label: 'H3' },
  { key: '4', kind: 'subsubsection',        label: 'H4' },
  { key: '5', kind: 'subsubsubsection',     label: 'H5' },
  { key: '6', kind: 'subsubsubsubsection',  label: 'H6' },
  { key: '',  kind: 'paragraph',            label: 'P'  },
] as const;

async function changeBlockType(kind: string): Promise<void> {
  if (!selectedItemId.value || ocrTool.value !== 'block') return;
  await callHocrEndpoint(selectedItemId.value, 'change-type', { kind });
}

async function callHocrEndpoint(id: string, action: string, body?: object): Promise<void> {
  const stem = currentStem.value;
  const level = ocrTool.value;
  if (!stem || level === 'multi') return;
  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/${LEVEL_SEGMENT[level]}/${id}/${action}`;
  const resp = await fetch(url, body
      ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
      : { method: 'POST' });

  if (resp.ok) {
    hocrPage.value = resp.ok ? (await resp.json() as HocrPage) : null;
  }
}

async function pageInteractionClick(): Promise<void> {
  const mode = effectiveOcrMode.value;
  if (mode === 'none') return;

  if (ocrTool.value === 'multi') {
    multiSelect.value = multiHover.value;
    return;
  }

  if (mode === 'select') {
    selectedItemId.value = overItemId.value;
    return;
  }

  if (mode === 'remove' && overItemId.value) {
    await callHocrEndpoint(overItemId.value, 'remove');
    if (selectedItemId.value === overItemId.value) selectedItemId.value = null;
  }
  else if (mode === 'join' && betweenTargets.value[0] && betweenTargets.value[1]) {
    await callHocrEndpoint(betweenTargets.value[0].id, 'merge', { other_id: betweenTargets.value[1].id });
  }
  else if (mode === 'split' && overItemId.value && betweenSubTargets.value[0] && betweenSubTargets.value[1]) {
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

  // Q = multi-level select tool
  // W = CAREA level tool
  // E = BLOCK level tool
  // R = LINE level tool
  // T = WORD level tool

  if (e.key === 'q') {
    setOcrTool('multi');
    return;
  }
  if (e.key === 'w') {
    setOcrTool('carea');
    return;
  }
  else if (e.key == 'e') {
    setOcrTool('block');
    return;
  }
  else if (e.key == 'r') {
    setOcrTool('line');
    return;
  }
  else if (e.key == 't') {
    setOcrTool('word');
    return;
  }

  // Block type change: 0–6, only when block tool is active and a block is selected.
  if (ocrTool.value === 'block' && selectedItemId.value && !isTypingTarget()) {
    const kind = BLOCK_KIND_KEYS[e.key];
    if (kind) {
      e.preventDefault();
      await changeBlockType(kind);
      return;
    }
  }

  if (!selectedItemId.value || isTypingTarget()) return;
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    await callHocrEndpoint(selectedItemId.value, 'move-up');
  }
  else if (e.key === 'ArrowDown') {
    e.preventDefault();
    await callHocrEndpoint(selectedItemId.value, 'move-down');
  }
  else if (e.key === 'Backspace' || e.key === 'Delete') {
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

.ocr-info-id {
  font-family: ui-monospace, monospace;
  color: var(--color-text-dimmed, #a2acb6);
}

.tool-palette {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  align-items: stretch;
}

.tool-palette > sl-button,
.tool-palette > sl-button-group {
  width: 100%;
}

.tool-palette > sl-button-group::part(base) {
  display: flex;
  width: 100%;
  max-width: 100%;
}

.tool-palette > sl-button-group sl-button {
  flex: 1 1 0;
}

.kind-key {
  opacity: 0.55;
  font-size: 0.7em;
}

</style>