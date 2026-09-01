<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :page-list-columns="['name-or-scan', 'extras']"
      :show-crop-overlay="false"
      :hocr-level="ocrLevel=='multi' ? null : ocrLevel"
      :flows="flows"
      :layouts="layouts"
      :project="project"
      :pointer-settings="{ color: pointerColor, label: pointerLabel, icon: pointerIcon, enabled: pointerEnabled }"
      :show-layers="careaLayers"
      :show-blocks="true"
      :page-interaction-update="pageInteractionUpdate"
      :page-interaction-click="pageInteractionClick"
      :page-interaction-drag="pageInteractionDrag"
      :is-no-hocr-acceptable="false"
      @current-page-change="clearSelection"
  >
    <template #tools="{ currentPage }">

      <div class="tool-palette">
        <sl-button-group>
          <sl-button :variant="activeMasterTool === 'carea-flow' ? 'primary' : 'default'" size="small" @click="activeMasterTool = 'carea-flow'">
            Flow <span class="master-key">Q</span>
          </sl-button>
          <sl-button :variant="activeMasterTool === 'carea-layout' ? 'primary' : 'default'" size="small" @click="activeMasterTool = 'carea-layout'">
            Layout <span class="master-key">W</span>
          </sl-button>
          <sl-button :variant="activeMasterTool === 'edit' ? 'primary' : 'default'" size="small" @click="activeMasterTool = 'edit'">
            Edit <span class="master-key">E</span>
          </sl-button>
          <sl-button :variant="activeMasterTool === 'block-type' ? 'primary' : 'default'" size="small" @click="activeMasterTool = 'block-type'">
            Type <span class="master-key">R</span>
          </sl-button>
        </sl-button-group>

        <sl-checkbox
            v-if="['carea-flow', 'carea-layout', 'block-type'].includes(activeMasterTool)"
            :checked="mergeItems[activeMasterTool]"
            @sl-change="mergeItems[activeMasterTool] = ($event.target as HTMLInputElement).checked"
            size="small"
        >
          Merge items
        </sl-checkbox>

        <div v-if="activeMasterTool === 'carea-flow'" class="vertical-tool-list">
          <sl-button size="small" :disabled="!selectedItemId || ocrLevel !== 'carea'" @click="changeCareaOperation('change-flow', '')">
            No flow <span class="kind-key">0</span>
          </sl-button>
          <sl-button
              v-for="(flow, index) in flows"
              :key="flow.name"
              size="small"
              :disabled="!selectedItemId || ocrLevel !== 'carea'"
              @click="changeCareaOperation('change-flow', flow.name)"
          >
            {{ flow.name }} <span class="kind-key">{{ index + 1 }}</span>
          </sl-button>
        </div>

        <div v-if="activeMasterTool === 'carea-layout'" class="vertical-tool-list">
          <sl-button size="small" :disabled="!selectedItemId || ocrLevel !== 'carea'" @click="changeCareaOperation('change-layout', '')">
            No layout <span class="kind-key">0</span>
          </sl-button>
          <sl-button
              v-for="(layout, index) in layouts"
              :key="layout.name"
              size="small"
              :disabled="!selectedItemId || ocrLevel !== 'carea'"
              @click="changeCareaOperation('change-layout', layout.name)"
          >
            {{ layout.name }} <span class="kind-key">{{ index + 1 }}</span>
          </sl-button>
        </div>

        <sl-button-group v-if="activeMasterTool === 'edit'">
          <sl-button :variant="ocrLevel==='multi' ? 'primary' : 'default'" size="small"  @click="setOcrLevel('multi')"><sl-icon name="eyedropper"></sl-icon></sl-button>
          <sl-button :variant="ocrLevel==='page' ? 'primary' : 'default'" size="small"  @click="setOcrLevel('page')">Page <span class="kind-key">0</span></sl-button>
          <sl-button :variant="ocrLevel==='carea' ? 'primary' : 'default'" size="small"  @click="setOcrLevel('carea')">Carea <span class="kind-key">1</span></sl-button>
          <sl-button :variant="ocrLevel==='block' ? 'primary' : 'default'" size="small"  @click="setOcrLevel('block')">Block <span class="kind-key">2</span></sl-button>
          <sl-button :variant="ocrLevel==='line' ? 'primary' : 'default'"  size="small" @click="setOcrLevel('line')">Line <span class="kind-key">3</span></sl-button>
          <sl-button :variant="ocrLevel==='word' ? 'primary' : 'default'" size="small"  @click="setOcrLevel('word')">Word <span class="kind-key">4</span></sl-button>
        </sl-button-group>

        <sl-button-group v-if="activeMasterTool === 'block-type' && ocrLevel === 'block'">
          <sl-button
              v-for="bk in BLOCK_KINDS"
              :key="bk.kind"
              size="small"
              :disabled="!selectedItemId"
              @click="changeBlockType(bk.kind)"
          >{{ bk.label }} <span class="kind-key">{{ bk.key }}</span></sl-button>
        </sl-button-group>

        <sl-button-group v-if="activeMasterTool === 'edit' && ocrLevel!=='multi'">
          <sl-button :variant="effectiveOcrOperation==='add' ? (ocrOperation!=='context' ? 'primary' : 'secondary') : 'default'" size="small" @click="setOcrOperation('add')">Add <span class="kind-key">A</span></sl-button>
          <sl-button :variant="effectiveOcrOperation==='select' ? (ocrOperation!=='context' ? 'primary' : 'secondary') : 'default'" size="small" @click="setOcrOperation('select')">Sel <span class="kind-key">S</span></sl-button>
          <sl-button :variant="effectiveOcrOperation==='remove' ? (ocrOperation!=='context' ? 'primary' : 'secondary') : 'default'" size="small" @click="setOcrOperation('remove', $event)">Rem <span class="kind-key">D</span></sl-button>
          <sl-button :variant="ocrOperation==='context' ? 'primary' : 'default'" size="small" @click="setOcrOperation('context')">Auto <span class="kind-key">F</span></sl-button>
          <sl-button :variant="effectiveOcrOperation==='split' ? (ocrOperation!=='context' ? 'primary' : 'secondary') : 'default'" size="small" @click="setOcrOperation('split')">Split <span class="kind-key">H</span></sl-button>
          <sl-button :variant="effectiveOcrOperation==='join' ? (ocrOperation!=='context' ? 'primary' : 'secondary') : 'default'" size="small" @click="setOcrOperation('join', $event)">Join <span class="kind-key">J</span></sl-button>
        </sl-button-group>

        <sl-button-group v-if="activeMasterTool === 'edit' && ocrLevel==='page'">
          <sl-button @click="restoreFromOriginal(currentPage)" size="small">Restore</sl-button>
        </sl-button-group>

        <sl-button-group v-if="activeMasterTool === 'edit' && ocrLevel==='carea'">
          <sl-button @click="autoLayout(currentPage)" size="small">Auto layout</sl-button>
          <sl-button @click="autoFlow(currentPage)" size="small">Auto flow</sl-button>
        </sl-button-group>

        <sl-button-group v-if="activeMasterTool === 'edit' && (ocrLevel==='carea' || ocrLevel==='word')">
          <sl-button size="small" :disabled="!selectedItemId" @click="rescan(selectedItemId)">
            <sl-icon name="arrow-repeat" slot="prefix"></sl-icon>
            Rescan
          </sl-button>
          <sl-input
              size="small"
              :value="ocrLanguage"
              @sl-change="ocrLanguage = ($event.target as HTMLInputElement).value"
              placeholder="OCR Language"
              style="width: 80px;"
          ></sl-input>
        </sl-button-group>

        <template v-if="showAddForm">
          <form
              class="form-grid"
          >
            <template v-if="ocrLevel === 'block'">
              <sl-radio-group
                  size="small"
                  :value="addForm.blockType"
                  @sl-change="addForm.blockType = ($event.target as HTMLInputElement).value as AddBlockType">
                <sl-radio-button value="text">Text</sl-radio-button>
                <sl-radio-button value="image">Image</sl-radio-button>
              </sl-radio-group>
              <sl-checkbox :checked="addForm.shrinkWrapCarea" @sl-change="addForm.shrinkWrapCarea = ($event.target as HTMLInputElement).checked">Shrink wrap parent carea</sl-checkbox>
            </template>
            <div style="display: flex; flex-direction: row; align-items: center;">
            <sl-checkbox size="small" :checked="addForm.eraseUnderneath" @sl-change="addForm.eraseUnderneath = ($event.target as HTMLInputElement).checked">Erase overlapping ></sl-checkbox>
            <sl-input size="small" :value="addForm.eraseOverlapPercentage" @sl-change="addForm.eraseOverlapPercentage = parseInt(($event.target as HTMLInputElement).value)" type="number" min="0" max="100" step="1" placeholder="Overlap percentage"/>%
            </div>
          </form>
        </template>
      </div>

      <div class="ocr-info-panel">
        <div class="ocr-info-row" :class="{ active: ocrLevel === 'carea' }">
          <span class="ocr-info-label">Carea</span>
          <span class="ocr-info-value ocr-info-id">{{ multiSelect?.carea?.id ?? '—' }}</span>
          <span class="ocr-info-value ocr-info-id">({{ multiHover?.carea?.id ?? '—' }})</span>
        </div>
        <div class="ocr-info-row" :class="{ active: ocrLevel === 'block' }">
          <span class="ocr-info-label">Block</span>
          <span class="ocr-info-value ocr-info-id">{{ multiSelect?.block?.id ?? '—' }}</span>
          <span class="ocr-info-value ocr-info-id">({{ multiHover?.block?.id ?? '—' }})</span>
        </div>
        <div class="ocr-info-row" :class="{ active: ocrLevel === 'line' }">
          <span class="ocr-info-label">Line</span>
          <span class="ocr-info-value ocr-info-id">{{ multiSelect?.line?.id ?? '—' }}</span>
          <span class="ocr-info-value ocr-info-id">({{ multiHover?.line?.id ?? '—' }})</span>
        </div>
        <div class="ocr-info-row" :class="{ active: ocrLevel === 'word' }">
          <span class="ocr-info-label">Word</span>
          <span class="ocr-info-value ocr-info-id">{{ multiSelect?.word?.id ?? '—' }}</span>
          <span class="ocr-info-value ocr-info-id">({{ multiHover?.word?.id ?? '—' }})</span>
        </div>
      </div>

    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import {computed, onMounted, onUnmounted, provide, type Ref, ref, inject, watch} from 'vue';
import { useRoute } from 'vue-router';
import PageWorkspace from '../components/PageWorkspace.vue';
import {
  type Page, type Project, type OverlayItem, type HocrNode,
  findMultiLevelItemByPoint, type MultiSelect,
  sortIdsByDocumentOrder, findMultilevelById
} from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';
import { isTypingTarget } from '../utils/dom';
import {type HocrPage} from '../types';

interface AddRequest {
  to_carea?: string | null; to_block?: string | null; to_line?: string | null;
  bbox: [number, number, number, number];
  block_type: AddBlockType;
  text?: string;
  shrink_wrap_carea: boolean;
  erase_underneath: boolean;
  erase_overlap: number;
}

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
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
const showError = inject<(msg: string) => void>('showError');

const hocrContext = provideHocrContext();
const { hocrPage, rescanCarea, rescanWord, updateHocr, loadHocr } = hocrContext;
const route = useRoute();

const currentStem = computed(() => {
  if (route.params.page) return String(route.params.page);
  if (!hocrPage.value) return null;
  return hocrPage.value.page_id.replace(/\.[^.]+$/, '');
});

type MasterTool = 'carea-flow' | 'carea-layout' | 'edit' | 'block-type';
type OcrLevel = 'multi' | 'page' | 'carea' | 'block' | 'line' | 'word';
type OcrOperation = 'context' | 'none' | 'add' | 'select' | 'join' | 'split' | 'remove';

const activeMasterTool = ref<MasterTool>('edit');
const ocrLevel:Ref<OcrLevel> = ref('multi');

const careaLayers = computed(() => ({
  flow: activeMasterTool.value === 'carea-flow' || activeMasterTool.value === 'edit',
  layout: activeMasterTool.value === 'carea-layout' || activeMasterTool.value === 'edit'
}));

const ocrOperation:Ref<OcrOperation> = ref('context');
const ocrLanguage = ref('eng');
const project = ref<Project | null>(null);
const flows = computed(() => project.value?.flows || []);
const layouts = computed(() => project.value?.layouts || []);

async function fetchProjectMetadata(): Promise<void> {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      const data = await resp.json() as Project;
      project.value = data;
      if (data.ocr_language) {
        ocrLanguage.value = data.ocr_language;
      }
    }
  } catch (e) {
    console.error('Failed to fetch project metadata:', e);
  }
}

const mergeItems = ref<Record<MasterTool, boolean>>({
  'carea-flow': false,
  'carea-layout': false,
  'edit': false,
  'block-type': true,
});

watch(activeMasterTool, (newVal) => {
  if (newVal === 'carea-flow' || newVal === 'carea-layout') {
    ocrLevel.value = 'carea';
  } else if (newVal === 'block-type') {
    ocrLevel.value = 'block';
  }
});


// ── Select ───────────────────────────────────────────────────────────

const overItemId = ref<string | null>(null);
const selectedItemIds = ref<Set<string>>(new Set());

// Multi-select: live hover stack (updated on mousemove) and committed selection (set on click).
const multiHover = ref<MultiSelect | null>(null);
const multiSelect = ref<MultiSelect | null>(null);

const selectedItemId = computed({
  get: () => {
    if (!multiSelect.value) return null;
    if (ocrLevel.value === 'multi') {
      return multiSelect.value.word?.id || multiSelect.value.line?.id || multiSelect.value.block?.id || multiSelect.value.carea?.id || null;
    }
    if (ocrLevel.value === 'page') return null;
    return multiSelect.value[ocrLevel.value as keyof MultiSelect]?.id ?? null;
  },
  set: (val) => {
    window.alert(`Forbidden direct assignment to selectedItemId.value = ${val}. Use newSelection(), addToSelection() or removeFromSelection() instead.`);
  }
});

function newSelection(id: string | null) {
  if (!id) {
    selectedItemIds.value.clear();
    multiSelect.value = null;
    return;
  }
  const stack = findMultilevelById(hocrPage.value!, id);
  if (stack) {
    multiSelect.value = stack;
    const targetId = ocrLevel.value === 'multi'
        ? (stack.word?.id || stack.line?.id || stack.block?.id || stack.carea?.id || null)
        : (stack[ocrLevel.value as keyof MultiSelect]?.id || null);

    if (targetId) {
      selectedItemIds.value = new Set([targetId]);
    }
  }
}

function addToSelection(id: string) {
  const stack = findMultilevelById(hocrPage.value!, id);
  if (stack) {
    multiSelect.value = stack;
    const targetId = ocrLevel.value === 'multi'
        ? (stack.word?.id || stack.line?.id || stack.block?.id || stack.carea?.id || null)
        : (stack[ocrLevel.value as keyof MultiSelect]?.id || null);

    if (targetId) {
      selectedItemIds.value.add(targetId);
    }
  }
}

function removeFromSelection(id: string) {
  selectedItemIds.value.delete(id);
  if (selectedItemIds.value.size === 0) {
    multiSelect.value = null;
  } else {
    const lastId = Array.from(selectedItemIds.value).pop()!;
    const stack = findMultilevelById(hocrPage.value!, lastId);
    if (stack) multiSelect.value = stack;
  }
}

function clearSelection() {
  selectedItemIds.value.clear();
  multiSelect.value = null;
  overItemId.value = null;
  multiHover.value = null;
  indicatedItemId.value = null;
  betweenTargets.value = [null, null];
  betweenSubTargets.value = [null, null];
}

const indicatedItemId  = ref<string | null>(null);
provide('selectedItemIds', selectedItemIds);
provide('selectedItemId',  selectedItemId);
provide('indicatedItemId', indicatedItemId);
provide('selectNode', (level: OcrLevel, id: string, e?: MouseEvent) => {
  console.log("selectNode", level, id, e);
  setOcrLevel(level);
  if (e?.shiftKey) {
    if (selectedItemIds.value.has(id)) {
      removeFromSelection(id);
    } else {
      addToSelection(id);
    }
  } else {
    newSelection(id);
  }
});
const betweenTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);
const betweenSubTargets = ref<[HocrNode | null, HocrNode | null]>([null, null]);

// ── Modifier key state ───────────────────────────────────────────────

const shiftDown = ref(false);
const altDown   = ref(false);
const metaDown  = ref(false);
const ctrlDown  = ref(false);
const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);
const cmdDown = computed(() => isMac ? metaDown.value : ctrlDown.value);

// ── Effective mode (modifier keys override manual ocrMode) ────────────

const effectiveOcrOperation = computed<OcrOperation>(() => {
  if (activeMasterTool.value !== 'edit') return 'select';

  if (ocrOperation.value !== 'context') return ocrOperation.value;

  if (ocrLevel.value === 'multi') return 'select';

  if (cmdDown.value) {
    if (altDown.value && shiftDown.value) return 'remove';
    if (shiftDown.value) return 'add';
    if (betweenTargets.value[0] !== null && betweenTargets.value[1] !== null) return 'join';
    if (betweenSubTargets.value[0] !== null && betweenSubTargets.value[1] !== null) return 'split';
    return 'none';
  }

  return 'select';
});

// ── Add ──────────────────────────────────────────────────────────────

type AddBlockType = 'text' | 'image';
type AddForm = {
  blockType: AddBlockType
  text?: string
  shrinkWrapCarea: boolean
  eraseUnderneath: boolean
  eraseOverlapPercentage: number
}

const showAddForm = computed<boolean>( () => { return effectiveOcrOperation.value === 'add' });
const addForm: Ref<AddForm> = ref({
  blockType: 'text',
  shrinkWrapCarea: false,
  eraseUnderneath: true,
  eraseOverlapPercentage: 20,
});

// ── Custom pointer ───────────────────────────────────────────────────

const pointerLabel = computed(() => {
  switch (effectiveOcrOperation.value) {
    case 'none':   return 'No action';
    case 'add':    return 'Add';
    case 'select': return 'Select';
    case 'split':  return 'Split';
    case 'join':   return 'Join';
    case 'remove': return 'Remove';
  }
  return '';
});
const pointerColor = computed(() => {
  switch (effectiveOcrOperation.value) {
    case 'none':   return '#000000';
    case 'add':    return '#78ca3d';
    case 'select': return '#2563eb';
    case 'split':  return '#f97316';
    case 'join':   return '#d5a619';
    case 'remove': return '#dc2626';
  }
  return '';
});
const pointerIcon = computed(() => {
  switch (effectiveOcrOperation.value) {
    case 'none':   return 'question-lg';
    case 'add':    return 'plus';
    case 'select': return 'crosshair';
    case 'split':  return 'view-stacked';
    case 'join':   return 'view-list';
    case 'remove': return 'x-square';
  }
  return '';
});
const pointerEnabled = computed(() => {
  switch (effectiveOcrOperation.value) {
    case 'add':    return true;
    case 'select':
      return overItemId.value !== null;
    case 'remove':
      return overItemId.value !== null;
    case 'join':   return betweenTargets.value[0] !== null && betweenTargets.value[1] !== null;
    case 'split':  return betweenSubTargets.value[0] !== null && betweenSubTargets.value[1] !== null;
    default:       return false;
  }
});

// Can be called from UI with the intent of setting both level, master tool (and operation, depending on leftover setup).
function syncSelectionToLevel(level: OcrLevel) {
  if (level === 'multi' || level === 'page') return;

  const newIds = new Set<string>();
  for (const id of selectedItemIds.value) {
    const stack = findMultilevelById(hocrPage.value!, id);
    if (stack) {
      const target = stack[level as keyof MultiSelect];
      if (target) {
        newIds.add(target.id);
      }
    }
  }
  selectedItemIds.value = newIds;
  if (newIds.size > 0) {
    const lastId = Array.from(newIds).pop()!;
    multiSelect.value = findMultilevelById(hocrPage.value!, lastId);
  }
}

function setOcrLevel(level: OcrLevel) {
  ocrLevel.value = level;
  syncSelectionToLevel(level);

  if (level !== 'multi') {

    // Precaution: If we force the level to something that these master tools cannot handle: Change master tool.

    if (activeMasterTool.value === 'carea-flow' || activeMasterTool.value === 'carea-layout') {
      if (level !== 'carea') activeMasterTool.value = 'edit';
    } else if (activeMasterTool.value === 'block-type') {
      if (level !== 'block') activeMasterTool.value = 'edit';
    } else {
      activeMasterTool.value = 'edit';
    }
  }
  if (ocrLevel.value === 'multi') {
    if(['split', 'join', 'remove'].includes(ocrOperation.value)) {
      setOcrOperation('context');
    }
  }
  else {
    if (ocrOperation.value == 'none') {
      setOcrOperation('context');
    }
  }
}

function setOcrOperation(mode: OcrOperation, event?: MouseEvent | KeyboardEvent) {
  ocrOperation.value = mode;
  if (event?.shiftKey && selectedItemIds.value.size > 0) {
    if (mode === 'join') {
      void bulkJoin();
    } else if (mode === 'remove') {
      void bulkRemove();
    }
  }
}

async function bulkJoin() {
  if (selectedItemIds.value.size < 2 || ocrLevel.value === 'multi' || ocrLevel.value === 'page') return;
  const ids = sortIdsByDocumentOrder(hocrPage.value!, Array.from(selectedItemIds.value));
  await callBulkHocrEndpoint('merge', { item_ids: ids });
}

async function bulkRemove() {
  if (selectedItemIds.value.size === 0 || ocrLevel.value === 'multi' || ocrLevel.value === 'page') return;
  const ids = Array.from(selectedItemIds.value);
  for (const id of ids) {
    await callHocrEndpoint(id, 'remove');
  }
  newSelection(null);
}

// ── Change block type ────────────────────────────────────────────────

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
  '7': 'paragraph',           // P
  '8': 'image',               // IMG
  '9': 'table',               // TBL
  'l': 'list',                // LST
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
  { key: '7', kind: 'paragraph',            label: 'P'  },
  { key: '8', kind: 'image',                label: 'IMG' },
  { key: '9', kind: 'table',                label: 'TBL' },
  { key: 'L', kind: 'list',                 label: 'LST' },
] as const;

async function changeBlockType(turn_into: string): Promise<void> {
  if (selectedItemIds.value.size === 0 || ocrLevel.value !== 'block') return;
  activeMasterTool.value = 'block-type';
  const ids = Array.from(selectedItemIds.value);
  const merge = mergeItems.value['block-type'];

  if (ids.length > 1) {
    await callBulkHocrEndpoint('change-type', { item_ids: ids, turn_into, merge });
  } else if (ids.length === 1) {
    await callHocrEndpoint(ids[0], 'change-type', { turn_into, merge });
  }
}

async function changeCareaOperation(action: 'change-flow' | 'change-layout', turn_into: string): Promise<void> {
  if (selectedItemIds.value.size === 0 || ocrLevel.value !== 'carea') return;
  const ids = Array.from(selectedItemIds.value);
  const merge = mergeItems.value[activeMasterTool.value as MasterTool];

  if (ids.length > 1) {
    await callBulkHocrEndpoint(action, { item_ids: ids, turn_into, merge });
  } else if (ids.length === 1) {
    await callHocrEndpoint(ids[0], action, { turn_into, merge });
  }
}

// ── Mouse and key interactions ───────────────────────────────────────

async function pageInteractionDrag(x1: number, y1: number, x2: number, y2: number): Promise<void> {
  if (effectiveOcrOperation.value !== 'add') return;
  await callAddEndpoint([
    Math.round(Math.min(x1, x2)), Math.round(Math.min(y1, y2)),
    Math.round(Math.max(x1, x2)), Math.round(Math.max(y1, y2)),
  ]);
}

async function pageInteractionClick(): Promise<void> {
  const mode = effectiveOcrOperation.value;
  if (mode === 'none') return;

  if (mode === 'select') {
    const targetId = (ocrLevel.value === 'multi' && multiHover.value)
        ? (multiHover.value.word?.id || multiHover.value.line?.id || multiHover.value.block?.id || multiHover.value.carea?.id || null)
        : overItemId.value;

    if (shiftDown.value) {
      if (targetId) {
        if (selectedItemIds.value.has(targetId)) {
          removeFromSelection(targetId);
        } else {
          addToSelection(targetId);
        }
      }
    } else {
      newSelection(targetId);
    }
    return;
  }
  else if (mode === 'remove') {
    const targetId = (ocrLevel.value === 'multi' && multiHover.value)
        ? (multiHover.value.word?.id || multiHover.value.line?.id || multiHover.value.block?.id || multiHover.value.carea?.id || null)
        : overItemId.value;
    if (targetId) {
      await callHocrEndpoint(targetId, 'remove');
      removeFromSelection(targetId);
    }
  }
  else if (mode === 'join' && betweenTargets.value[0] && betweenTargets.value[1]) {
    await callHocrEndpoint(betweenTargets.value[0].id, 'merge', { other_id: betweenTargets.value[1].id });
  }
  else if (mode === 'split' && overItemId.value && betweenSubTargets.value[0] && betweenSubTargets.value[1]) {
    await callHocrEndpoint(overItemId.value, 'split',
        { before_id: betweenSubTargets.value[0].id, after_id: betweenSubTargets.value[1].id });
  }
}

function pageInteractionUpdate(
    x: number,
    y: number,
    _: OverlayItem[],
    _activeItem: HocrNode | null,
    betweenOverlayItems: [HocrNode | null, HocrNode | null],
    betweenOverlaySubItems: [HocrNode | null, HocrNode | null],
) {
  betweenTargets.value = betweenOverlayItems;
  betweenSubTargets.value = betweenOverlaySubItems;

  // Always populate the multi-level stack
  multiHover.value = findMultiLevelItemByPoint(hocrPage.value!, x, y);

  // Derive overItemId from multiHover based on ocrLevel
  if (ocrLevel.value === 'multi' || ocrLevel.value === 'page') {
    overItemId.value = null;
  } else {
    overItemId.value = multiHover.value?.[ocrLevel.value as keyof MultiSelect]?.id ?? null;
  }
}

async function handleKeyboardAction(e: KeyboardEvent): Promise<void> {
  if (isTypingTarget(document.activeElement)) return;

  // Master tools (Row 2)
  if (e.key === 'q') {
    activeMasterTool.value = 'carea-flow';
    return;
  }
  if (e.key === 'w') {
    activeMasterTool.value = 'carea-layout';
    return;
  }
  if (e.key === 'e') {
    activeMasterTool.value = 'edit';
    return;
  }
  if (e.key === 'r') {
    activeMasterTool.value = 'block-type';
    return;
  }

  // Numeric keys (Row 1) - Context sensitive
  if (/^[0-9l]$/i.test(e.key)) {
    if (activeMasterTool.value === 'edit') {
      const toolMap: Record<string, OcrLevel> = {
        '0': 'page',
        '1': 'carea',
        '2': 'block',
        '3': 'line',
        '4': 'word',
      };
      if (toolMap[e.key]) {
        setOcrLevel(toolMap[e.key]);
        return;
      }
    } else if (activeMasterTool.value === 'block-type') {
      const kind = BLOCK_KIND_KEYS[e.key.toLowerCase()];
      if (kind && selectedItemIds.value.size > 0 && ocrLevel.value === 'block') {
        e.preventDefault();
        await changeBlockType(kind);
        return;
      }
    } else if (activeMasterTool.value === 'carea-flow') {
      if (selectedItemIds.value.size > 0 && ocrLevel.value === 'carea') {
        if (e.key === '0') {
          e.preventDefault();
          await changeCareaOperation('change-flow', '');
          return;
        }
        const index = parseInt(e.key) - 1;
        const flow = flows.value[index];
        if (flow) {
          e.preventDefault();
          await changeCareaOperation('change-flow', flow.name);
          return;
        }
      }
    } else if (activeMasterTool.value === 'carea-layout') {
      if (selectedItemIds.value.size > 0 && ocrLevel.value === 'carea') {
        if (e.key === '0') {
          e.preventDefault();
          await changeCareaOperation('change-layout', '');
          return;
        }
        const index = parseInt(e.key) - 1;
        const layout = layouts.value[index];
        if (layout) {
          e.preventDefault();
          await changeCareaOperation('change-layout', layout.name);
          return;
        }
      }
    }
  }

  // Operation keys (Row 3) - Only for Edit master tool
  if (activeMasterTool.value === 'edit') {
    const opMap: Record<string, OcrOperation> = {
      'a': 'add',
      's': 'select',
      'd': 'remove',
      'f': 'context',
      'h': 'split',
      'j': 'join',
    };
    const key = e.key.toLowerCase();
    if (opMap[key]) {
      setOcrOperation(opMap[key], e);
      return;
    }
  }

  // Common shortcuts (Arrows, Delete) - Require selection
  if (selectedItemIds.value.size === 0) return;

  if (e.key === 'ArrowUp') {
    e.preventDefault();
    const ids = sortIdsByDocumentOrder(hocrPage.value!, Array.from(selectedItemIds.value));
    for (const id of ids) {
      await callHocrEndpoint(id, 'move-up');
    }
  }
  else if (e.key === 'ArrowDown') {
    e.preventDefault();
    const ids = sortIdsByDocumentOrder(hocrPage.value!, Array.from(selectedItemIds.value)).reverse();
    for (const id of ids) {
      await callHocrEndpoint(id, 'move-down');
    }
  }
  else if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault();
    const ids = Array.from(selectedItemIds.value);
    for (const id of ids) {
      await callHocrEndpoint(id, 'remove');
    }
    newSelection(null);
  }
}

function updateModifiers(e: KeyboardEvent) {
  shiftDown.value = e.shiftKey;
  altDown.value   = e.altKey;
  metaDown.value  = e.metaKey;
  ctrlDown.value  = e.ctrlKey;
}

// ── API endpoints ────────────────────────────────────────────────────

async function handleHocrResponse(resp: Response, source: string): Promise<void> {
  if (resp.ok) {
    const data = await resp.json();
    if (data && typeof data === 'object' && 'page' in data) {
      updateHocr(data.page as HocrPage);
      if (data.new_id) {
        newSelection(data.new_id);
      }
    } else {
      updateHocr(data as HocrPage);
    }
  } else {
    const text = await resp.text();
    let errorMsg = text;
    try {
      const json = JSON.parse(text);
      if (json.error) errorMsg = json.error;
    } catch (e) {
      // Not JSON
    }
    const finalMsg = errorMsg || `${source} error: ${resp.statusText}`;
    console.error(`${source} error:`, resp.status, resp.statusText, text);
    showError?.(finalMsg);
  }
}

async function callHocrEndpoint(id: string, action: string, body?: object): Promise<void> {
  const stem = currentStem.value;
  const tool = ocrLevel.value;
  if (!stem || tool === 'multi' || tool === 'page') return;
  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/${LEVEL_SEGMENT[tool]}/${id}/${action}`;
  const resp = await fetch(url, body
      ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
      : { method: 'POST' });

  await handleHocrResponse(resp, 'callHocrEndpoint');
}

async function callBulkHocrEndpoint(action: string, body: object): Promise<void> {
  const stem = currentStem.value;
  const tool = ocrLevel.value;
  if (!stem || tool === 'multi' || tool === 'page') return;
  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/${LEVEL_SEGMENT[tool]}/${action}`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });

  await handleHocrResponse(resp, 'callBulkHocrEndpoint');
}

async function callAddEndpoint(bbox: [number, number, number, number]): Promise<void> {
  const stem = currentStem.value;
  const tool = ocrLevel.value;
  if (!stem || tool === 'multi' || tool === 'page') return;

  const form = addForm.value;
  const body: AddRequest = {
    bbox,
    block_type: form.blockType,
    text: form.text,
    shrink_wrap_carea: form.shrinkWrapCarea,
    erase_underneath: form.eraseUnderneath,
    erase_overlap: form.eraseOverlapPercentage,
  };

  if (multiSelect.value) {
    if (tool === 'block') body.to_carea = multiSelect.value.carea?.id;
    else if (tool === 'line') body.to_block = multiSelect.value.block?.id;
    else if (tool === 'word') body.to_line = multiSelect.value.line?.id;
  }
  console.log('callAddEndpoint', body, tool);

  const url = `/api/projects/${props.machineName}/pages/${stem}/hocr/${LEVEL_SEGMENT[tool]}/add`;
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });

  await handleHocrResponse(resp, 'callAddEndpoint');
}

async function restoreFromOriginal(page: Page | null): Promise<void> {
  if(! page) {
    return;
  }
  const resp = await fetch(`/api/projects/${props.machineName}/pages/${page.scan}/restore-original`, {
    method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({page}),
  });

  await handleHocrResponse(resp, 'restoreFromOriginal');
}

async function rescan(id: string | null) {
  if (!id || !hocrContext.machineName.value || !hocrContext.stem.value) return;
  const level = ocrLevel.value;
  if (level !== 'carea' && level !== 'word') return;

  const confirmMsg = level === 'carea'
    ? "Are you sure you want to rescan this carea? This will append new results to the existing ones."
    : "Are you sure you want to rescan this word? This will replace the word with new results.";

  if (!window.confirm(confirmMsg)) return;

  if (level === 'carea') {
    await rescanCarea(hocrContext.machineName.value, hocrContext.stem.value, id, ocrLanguage.value);
  } else {
    await rescanWord(hocrContext.machineName.value, hocrContext.stem.value, id, ocrLanguage.value);
  }
}

const getStem = (p: Page) => p.scan.replace(/\.[^.]+$/, '');

async function autoLayout(page: Page | null) {
  if (!page) return;
  const stem = getStem(page);
  try {
    let body = { stems: [stem], carea_ids: [] as string[] };
    if (ocrLevel.value === 'carea' && selectedItemIds.value.size > 0) {
      body.stems = [];
      body.carea_ids = Array.from(selectedItemIds.value);
    }

    const resp = await fetch(`/api/projects/${props.machineName}/pages/${stem}/auto-layout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    if (resp.ok) {
      await loadHocr(props.machineName, stem);
    } else {
      const text = await resp.text();
      showError?.(`Auto layout failed: ${text}`);
    }
  } catch (e) {
    showError?.(`Auto layout failed: ${e instanceof Error ? e.message : String(e)}`);
  }
}

async function autoFlow(page: Page | null) {
  if (!page) return;
  const stem = getStem(page);
  try {
    let body = { stems: [stem], carea_ids: [] as string[] };
    if (ocrLevel.value === 'carea' && selectedItemIds.value.size > 0) {
      body.stems = [];
      body.carea_ids = Array.from(selectedItemIds.value);
    }

    const resp = await fetch(`/api/projects/${props.machineName}/pages/${stem}/auto-flow`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    if (resp.ok) {
      await loadHocr(props.machineName, stem);
    } else {
      const text = await resp.text();
      showError?.(`Auto flow failed: ${text}`);
    }
  } catch (e) {
    showError?.(`Auto flow failed: ${e instanceof Error ? e.message : String(e)}`);
  }
}

onMounted(() => {
  setActivePanels(panels);
  void fetchProjectMetadata();
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
  min-height: 8rem;
}

.ocr-info-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  flex-wrap: wrap;
}

.ocr-info-row.active .ocr-info-label {
  font-weight: 800;
  color: var(--color-text, #212529);
}

.ocr-info-row.active .ocr-info-id {
  font-weight: 700;
  color: var(--color-text, #212529);
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

.vertical-tool-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.vertical-tool-list sl-button::part(base) {
  justify-content: flex-start;
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
  min-width: 0;
}

.tool-palette > sl-button-group::part(base) {
  display: flex;
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.tool-palette > sl-button-group sl-button {
  flex: 1 1 0;
  min-width: 0;
}

.tool-palette > sl-button-group sl-button::part(base) {
  width: 100%;
  min-width: 0;
  padding-inline: 0.25em;
}

.kind-key {
  font-size: 0.8em;
  opacity: 0.7;
}

.master-key {
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  margin-left: 0.2rem;
  font-size: 0.8em;
  opacity: 0.7;
}

sl-button[variant="primary"] .master-key {
  color: inherit;
  opacity: 0.8;
}

.form-grid {
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 0.375rem;
  padding: 0.5rem;
  font-size: 0.8rem;
  margin-bottom: 0.5rem;

  display: grid;
  gap: 0.75rem;
}

.form-grid h3 {
  margin: 1rem 0 0;
  font-size: 1rem;
}

.form-grid label {
  display: grid;
  gap: 0.25rem;
  font-weight: 600;
}

.form-grid small {
  font-weight: 400;
  color: var(--color-text-muted);
}

.form-grid input[type="text"],
.form-grid input[type="number"] {
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
}

</style>