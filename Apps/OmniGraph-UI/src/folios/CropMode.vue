<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :strip-edge="edge"
      :strip-fraction="viewPercent / 100"
      :show-crop-overlay="true"
      :page-crops="pageCrops"
      :is-page-changed="isPageChanged"
      :keyboard-handler="handleCropKey"
      crop-color="rgba(0, 180, 0, 0.12)"
      discard-color="rgba(220, 0, 0, 0.35)"
      @pages-loaded="syncCropStateFromPageDb"
      @current-page-change="onPageChange"
  >
    <template #tools="workspace">

      <!-- Tools -->
      <sl-radio-group
          label="Tool"
          name="tool"
          :value="tool"
          @sl-change="tool = ($event.target as HTMLInputElement).value"
      >
        <sl-radio-button value="adjust">Adjust</sl-radio-button>
        <sl-radio-button value="assign">Assign</sl-radio-button>
      </sl-radio-group>

      <br>

      <template v-if="tool === 'adjust'">

        <!-- Adjust tool -->
        <sl-input
            label="Step small"
            type="number"
            min="1"
            :value="adjust_step_small"
            @sl-input="adjust_step_small = Math.max(1, parseInt(($event.target as HTMLInputElement).value) || 1)"
        />

        <sl-input
            label="Step large"
            type="number"
            min="1"
            :value="adjust_step_large"
            @sl-input="adjust_step_large = Math.max(1, parseInt(($event.target as HTMLInputElement).value) || 1)"
        />

        <div class="accumulator-display" v-if="accumulator !== 0">
          Δ {{ accumulator > 0 ? '+' : '' }}{{ accumulator }} px
        </div>

        <br>

        <!-- Magnet -->
        <sl-switch
            :checked="magnetEnabled"
            @sl-change="magnetEnabled = ($event.target as HTMLInputElement).checked; applyMagnet(workspace.filteredPages)"
        >
          Magnet
        </sl-switch>

        <template v-if="magnetEnabled">
          <br>
          <sl-radio-group
              label="Profile"
              name="profile"
              :value="magnetProfile"
              @sl-change="magnetProfile = ($event.target as HTMLInputElement).value as MagnetProfile; applyMagnet(workspace.filteredPages)"
          >
            <sl-radio-button value="bell" title="0 → peak → 0">Bell</sl-radio-button>
            <sl-radio-button value="rampup" title="0 → peak">Ramp ↑</sl-radio-button>
            <sl-radio-button value="rampdown" title="peak → 0">Ramp ↓</sl-radio-button>
          </sl-radio-group>
        </template>

        <br>

        <!-- Edge selector -->
        <sl-radio-group
            label="Edge"
            name="edge"
            size="small"
            :value="edge"
            @sl-change="onEdgeChange(($event.target as HTMLInputElement).value)"
        >
          <sl-radio-button value="none">None</sl-radio-button>
          <sl-radio-button value="left">Left</sl-radio-button>
          <sl-radio-button value="top">Top</sl-radio-button>
          <sl-radio-button value="bottom">Bottom</sl-radio-button>
          <sl-radio-button value="right">Right</sl-radio-button>
        </sl-radio-group>

        <template v-if="edge !== 'none'">
          <br>
          <sl-range
              :label="`Edge percent: ${viewPercent}%`"
              min="10"
              max="75"
              step="5"
              :value="viewPercent"
              @sl-input="viewPercent = parseInt(($event.target as HTMLInputElement).value)"
          />
        </template>
      </template>

      <template v-if="tool === 'assign'">
        <div class="diamond-inputs">
          <sl-input
              class="diamond-input top"
              size="small"
              pill
              type="number"
              :value="assignValues.top ?? ''"
              @sl-input="assignValues.top = parseOptionalNumber(($event.target as HTMLInputElement).value)"
          />

          <sl-input
              class="diamond-input left"
              size="small"
              pill
              type="number"
              :value="assignValues.left ?? ''"
              @sl-input="assignValues.left = parseOptionalNumber(($event.target as HTMLInputElement).value)"
          />

          <sl-input
              class="diamond-input right"
              size="small"
              pill
              type="number"
              :value="assignValues.right ?? ''"
              @sl-input="assignValues.right = parseOptionalNumber(($event.target as HTMLInputElement).value)"
          />

          <sl-input
              class="diamond-input bottom"
              size="small"
              pill
              type="number"
              :value="assignValues.bottom ?? ''"
              @sl-input="assignValues.bottom = parseOptionalNumber(($event.target as HTMLInputElement).value)"
          />
        </div>

        <br>

        <sl-button-group>
          <sl-button variant="default"
                     @click="assignBySetting(assignValues, workspace.filteredPages)">Set
          </sl-button>
          <sl-button variant="default"
                     @click="assignByAdding(assignValues, workspace.filteredPages)">Add
          </sl-button>
          <sl-button variant="default"
                     @click="assignBySubtracting(assignValues, workspace.filteredPages)">Sub
          </sl-button>
        </sl-button-group>

        <br>

        <sl-button-group>
          <sl-button variant="default" @click="assignReset(workspace.filteredPages)">Reset
          </sl-button>
          <sl-button variant="default" @click="assignAllEdges(100, workspace.filteredPages)">+100
          </sl-button>
          <sl-button variant="default" @click="assignAllEdges(-100, workspace.filteredPages)">
            −100
          </sl-button>
        </sl-button-group>
      </template>

      <br><br>

      <!-- Session buttons -->
      <sl-button-group>
        <sl-button
            variant="danger"
            :disabled="!workspace.hasChanges"
            @click="abandonCrop()"
        >
          Abandon
        </sl-button>

        <sl-button
            variant="primary"
            :disabled="!workspace.hasChanges"
            @click="commitCrops(workspace.pages, workspace.pageDbNextBatch)"
        >
          Commit
        </sl-button>
      </sl-button-group>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted, onUnmounted, inject } from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import type {CropEdges, Page, PageDb} from '../types';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const panels = usePersistentPanels('panels.crop', {
  'page-list': true,
  'page-strips': true,
  'page-preview': true,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();
const showError = inject<(msg: string) => void>('showError');

onMounted(() => setActivePanels(panels));
onUnmounted(() => setActivePanels(null));

// ── Tool / edge ─────────────────────────────────────────────────────────
const tool = ref('adjust');
const edge = ref('none');
const viewPercent = ref(25);

// ── Adjust tool data ────────────────────────────────────────────────────
const adjust_step_small = ref(10);
const adjust_step_large = ref(50);

// ── Assign tool data ────────────────────────────────────────────────────
type OptionalNumber = number | undefined;

type AssignValues = {
  top: OptionalNumber;
  left: OptionalNumber;
  right: OptionalNumber;
  bottom: OptionalNumber;
};

const assignValues = reactive<AssignValues>({
  top: undefined,
  left: undefined,
  right: undefined,
  bottom: undefined,
});

const pageCrops = reactive(new Map<number, CropEdges>());
const originalCrops = reactive(new Map<number, CropEdges>());

function syncCropStateFromPageDb(data: PageDb) {
  pageCrops.clear();
  originalCrops.clear();

  for (const page of data.pages) {
    const crop = { ...page.crop_edges };
    pageCrops.set(page.index, crop);
    originalCrops.set(page.index, { ...crop });
  }

  rebuildRoundBase();
}

function isPageChanged(page: Page): boolean {
  const orig = originalCrops.get(page.index);
  const curr = pageCrops.get(page.index);

  if (!orig || !curr) return false;

  return orig.left !== curr.left
      || orig.top !== curr.top
      || orig.right !== curr.right
      || orig.bottom !== curr.bottom;
}

function parseOptionalNumber(value: string): OptionalNumber {
  if (value.trim() === '') return undefined;

  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

// ── Accumulator & magnet ────────────────────────────────────────────────
const accumulator = ref(0);
const roundBaseCrops = new Map<number, CropEdges>();

type MagnetProfile = 'bell' | 'rampup' | 'rampdown';

const magnetEnabled = ref(false);
const magnetProfile = ref<MagnetProfile>('bell');

function getMagnetWeight(i: number, n: number, profile: MagnetProfile): number {
  if (n <= 1) return 1.0;

  const t = i / (n - 1);

  switch (profile) {
    case 'bell':
      return t <= 0.5 ? 2 * t : 2 * (1 - t);
    case 'rampup':
      return t;
    case 'rampdown':
      return 1 - t;
  }
}

function applyMagnet(
    filteredPages: Page[],
) {
  if (edge.value === 'none') return;

  const edgeKey = edge.value as keyof CropEdges;
  const n = filteredPages.length;

  for (let i = 0; i < n; i++) {
    const page = filteredPages[i];
    const base = roundBaseCrops.get(page.index);
    const curr = pageCrops.get(page.index);

    if (!base || !curr) continue;

    const weight = magnetEnabled.value
        ? getMagnetWeight(i, n, magnetProfile.value)
        : 1.0;

    curr[edgeKey] = Math.max(
        0,
        base[edgeKey] + Math.round(accumulator.value * weight),
    );
  }
}


function rebuildRoundBase() {
  roundBaseCrops.clear();
  for (const [index, crop] of pageCrops) {
    roundBaseCrops.set(index, { ...crop });
  }
}

function adjustRange(
    delta: number,
    filteredPages: Page[],
) {
  if (edge.value === 'none' || filteredPages.length === 0) return;

  accumulator.value += delta;
  applyMagnet(filteredPages);
}

function onEdgeChange(
    newEdge: string,
) {
  accumulator.value = 0;
  rebuildRoundBase();
  edge.value = newEdge;
}

function onPageChange() {
  accumulator.value = 0;
  rebuildRoundBase();
}

// ── Adjust tool functions ────────────────────────────────────────────

type CropKeyboardContext = {
  pages: Page[];
  filteredPages: Page[];
  visiblePages: Page[];
  selectionInfo: unknown;
  focusPage: (pageIndex: number) => void;
  navigatePage: (delta: number) => void;
};

function handleCropKey(e: KeyboardEvent, context: CropKeyboardContext): boolean {
  const shift = e.shiftKey;
  const alt = e.altKey;

  if (shift && alt) {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        onEdgeChange('top');
        return true;
      case 'ArrowDown':
        e.preventDefault();
        onEdgeChange('bottom');
        return true;
      case 'ArrowLeft':
        e.preventDefault();
        onEdgeChange('left');
        return true;
      case 'ArrowRight':
        e.preventDefault();
        onEdgeChange('right');
        return true;
    }
  }

  if (alt) return false;

  return adjustByKey(shift, e, context.filteredPages);
}

function adjustByKey(
    shift: boolean,
    e: KeyboardEvent,
    filteredPages: Page[],
): boolean {
  const step = shift ? adjust_step_large.value : adjust_step_small.value;

  if (edge.value === 'top' && e.key === 'ArrowDown') {
    e.preventDefault();
    adjustRange(step, filteredPages);
    return true;
  }

  if (edge.value === 'top' && e.key === 'ArrowUp') {
    e.preventDefault();
    adjustRange(-step, filteredPages);
    return true;
  }

  if (edge.value === 'bottom' && e.key === 'ArrowDown') {
    e.preventDefault();
    adjustRange(-step, filteredPages);
    return true;
  }

  if (edge.value === 'bottom' && e.key === 'ArrowUp') {
    e.preventDefault();
    adjustRange(step, filteredPages);
    return true;
  }

  if (edge.value === 'left' && e.key === 'ArrowRight') {
    e.preventDefault();
    adjustRange(step, filteredPages);
    return true;
  }

  if (edge.value === 'left' && e.key === 'ArrowLeft') {
    e.preventDefault();
    adjustRange(-step, filteredPages);
    return true;
  }

  if (edge.value === 'right' && e.key === 'ArrowRight') {
    e.preventDefault();
    adjustRange(-step, filteredPages);
    return true;
  }

  if (edge.value === 'right' && e.key === 'ArrowLeft') {
    e.preventDefault();
    adjustRange(step, filteredPages);
    return true;
  }

  return false;
}

// ── Assign tool functions ────────────────────────────────────────────

function forEachSelected(
    filteredPages: Page[],
    pageCrops: Map<number, CropEdges>,
    fn: (crop: CropEdges) => void,
) {
  for (const page of filteredPages) {
    const crop = pageCrops.get(page.index);
    if (crop) fn(crop);
  }
}

function assignBySetting(
    values: AssignValues,
    filteredPages: Page[]) {
  forEachSelected(filteredPages, pageCrops,crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, values.bottom);
  });
}

function assignByAdding(
    values: AssignValues,
    filteredPages: Page[]) {
  forEachSelected(filteredPages, pageCrops,crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, crop.left   + values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, crop.top    + values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, crop.right  + values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, crop.bottom + values.bottom);
  });
}

function assignBySubtracting(
    values: AssignValues,
    filteredPages: Page[]) {
  forEachSelected(filteredPages, pageCrops,crop => {
    if (values.left   !== undefined) crop.left   = Math.max(0, crop.left   - values.left);
    if (values.top    !== undefined) crop.top    = Math.max(0, crop.top    - values.top);
    if (values.right  !== undefined) crop.right  = Math.max(0, crop.right  - values.right);
    if (values.bottom !== undefined) crop.bottom = Math.max(0, crop.bottom - values.bottom);
  });
}

function assignReset(
    filteredPages: Page[]) {
  forEachSelected(filteredPages, pageCrops,crop => { crop.left = crop.top = crop.right = crop.bottom = 0; });
}

function assignAllEdges(
    delta: number,
    filteredPages: Page[]) {
  forEachSelected(filteredPages, pageCrops,crop => {
    crop.left   = Math.max(0, crop.left   + delta);
    crop.top    = Math.max(0, crop.top    + delta);
    crop.right  = Math.max(0, crop.right  + delta);
    crop.bottom = Math.max(0, crop.bottom + delta);
  });
}

// ── Crop session: abandon / commit ───────────────────────────────────
function abandonCrop() {
  accumulator.value = 0;
  pageCrops.clear();
  for (const [idx, crop] of originalCrops) pageCrops.set(idx, {...crop});
  rebuildRoundBase();
}

async function commitCrops(pages: Page[],
                           pageDbNextBatch: number) {
  const updatedPageDb = {
    next_batch: pageDbNextBatch,
    pages: pages.map(page => ({
      ...page,
      crop_edges: pageCrops.get(page.index) ?? page.crop_edges,
    })),
  };
  try {
    const res = await fetch(`/api/projects/${props.machineName}/pages`, {
      method: 'PUT',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify(updatedPageDb),
    });
    if (res.ok) {
      for (const [idx, crop] of pageCrops) originalCrops.set(idx, {...crop});
    } else {
      const text = await res.text();
      let errorMsg = text;
      try {
        const json = JSON.parse(text);
        if (json.error) errorMsg = json.error;
      } catch (e) {
        // Not JSON
      }
      const finalMsg = errorMsg || `Commit failed: ${res.statusText}`;
      console.error('Commit failed:', res.status, text);
      showError?.(finalMsg);
    }
  } catch (e) {
    console.error('Commit error:', e);
    showError?.(e instanceof Error ? e.message : String(e));
  }
}

</script>

<style>

.accumulator-display {
  font-size: 0.8rem;
  font-variant-numeric: tabular-nums;
  color: var(--color-accent, #2563eb);
  padding: 0.15rem 0;
}

.diamond-inputs {
  display: grid;
  grid-template-columns: max-content max-content max-content;
  gap: 0.5rem;
  justify-content: center;
  align-items: center;
}

.diamond-input {
  width: 5rem;
}

.diamond-input.top {
  grid-column: 2;
  grid-row: 1;
}

.diamond-input.left {
  grid-column: 1;
  grid-row: 2;
}

.diamond-input.right {
  grid-column: 3;
  grid-row: 2;
}

.diamond-input.bottom {
  grid-column: 2;
  grid-row: 3;
}</style>
