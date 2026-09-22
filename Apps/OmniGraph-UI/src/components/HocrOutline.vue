<template>
  <div v-if="hocrPage" class="hocr-outline">

    <div v-for="carea in hocrPage.careas" :key="carea.id" class="hocr-item">

      <!-- Carea row -->
      <div class="hocr-row hocr-carea-row" :class="{ 'hocr-row-selected': selectedItemIds?.has(carea.id) }" @click="toggleCarea(carea.id)" @mouseenter="indicate(carea.id)" @mouseleave="indicate(null)">
        <span class="hocr-badge hocr-badge-c" :style="careaBadgeStyle(carea)" title="Select CAREA" @click.stop="selectNode('carea', carea.id, $event)">C</span>
        <span class="hocr-count">({{ carea.blocks.length }})</span>
        <span class="hocr-id" :title="carea.id">{{ carea.id }}</span>
        <span :class="getBadgeDetails('flow', carea.flow).class" :style="getBadgeDetails('flow', carea.flow).style">
          {{ getBadgeDetails('flow', carea.flow).text }}
        </span>
        <span :class="getBadgeDetails('layout', carea.layout).class" :style="getBadgeDetails('layout', carea.layout).style">
          {{ getBadgeDetails('layout', carea.layout).text }}
        </span>
        <span class="hocr-preview">{{ careaPreview(carea) }}</span>
        <sl-button variant="text" size="small" class="hocr-rescan-btn" @click.stop="rescan(carea.id)" title="Rescan OCR for this carea">
          <sl-icon name="arrow-repeat"></sl-icon>
        </sl-button>
        <span class="hocr-toggle">{{ collapsedCareas.has(carea.id) ? '▸' : '▾' }}</span>
      </div>

      <!-- Blocks -->
      <div v-if="!collapsedCareas.has(carea.id)" class="hocr-children">
        <div v-for="block in carea.blocks" :key="block.id" class="hocr-item">

          <!-- Block row -->
          <div class="hocr-row hocr-block-row" :class="{ 'hocr-row-selected': selectedItemIds?.has(block.id) }" @click="toggleBlock(block.id)" @mouseenter="indicate(block.id)" @mouseleave="indicate(null)">
            <span class="hocr-badge hocr-badge-p" :title="`Select ${blockBadge(block)}`" @click.stop="selectNode('block', block.id, $event)">{{ blockBadge(block) }}</span>
            <span class="hocr-count">({{ block.lines.length }})</span>
            <span class="hocr-id" :title="block.id">{{ block.id }}</span>
            <span v-if="block.lang" class="hocr-lang">{{ block.lang }}</span>
            <span class="hocr-preview">{{ blockPreview(block) }}</span>
            <span class="hocr-toggle">{{ expandedBlocks.has(block.id) ? '▾' : '▸' }}</span>
          </div>

          <!-- Hints -->
          <div v-if="block.hints && Object.keys(block.hints).length > 0" class="hocr-hints-list">
            <template v-for="(ev, key) in block.hints" :key="key">
              <div v-if="ev && ev !== 'untested' && ev !== 'undetermined'" class="hocr-hint-item" :class="getEvidenceClass(ev as Evidence)">
                <span class="hocr-hint-key">{{ key.replace('test_', '').replace('break_from_', '') }}:</span>
                <span class="hocr-hint-value">{{ getEvidenceText(ev as Evidence) }}</span>
              </div>
            </template>
          </div>

          <!-- Lines -->
          <div v-if="expandedBlocks.has(block.id)" class="hocr-children">
            <div v-for="line in block.lines" :key="line.id" class="hocr-item">

              <!-- Line row -->
              <div class="hocr-row hocr-line-row" :class="{ 'hocr-row-selected': selectedItemIds?.has(line.id) }" @click="toggleLine(line.id)" @mouseenter="indicate(line.id)" @mouseleave="indicate(null)">
                <span class="hocr-badge hocr-badge-l" title="Select LINE" @click.stop="selectNode('line', line.id, $event)">L</span>
                <span class="hocr-count">({{ line.words.length }})</span>
                <span class="hocr-id" :title="line.id">{{ line.id }}</span>
                <span v-if="line.lang" class="hocr-lang">{{ line.lang }}</span>
                <span class="hocr-preview">{{ lineText(line) }}</span>
                <span class="hocr-toggle">{{ expandedLines.has(line.id) ? '▾' : '▸' }}</span>
              </div>

              <!-- Words -->
              <div v-if="expandedLines.has(line.id)" class="hocr-children">
                <div
                    v-for="word in line.words"
                    :key="word.id"
                    class="hocr-row hocr-word-row"
                    :class="{ 'hocr-row-selected': selectedItemIds?.has(word.id) }"
                    @mouseenter="indicate(word.id)" @mouseleave="indicate(null)"
                >
                  <span class="hocr-badge hocr-badge-w" title="Select WORD" @click.stop="selectNode('word', word.id, $event)">W</span>
                  <span class="hocr-id" :title="word.id">{{ word.id }}</span>
                  <span v-if="word.lang" class="hocr-lang">{{ word.lang }}</span>
                  <span class="hocr-preview">{{ wordText(word) }}</span>
                  <sl-button variant="text" size="small" class="hocr-rescan-btn" @click.stop="rescanWordAction(word.id)" title="Rescan OCR for this word">
                    <sl-icon name="arrow-repeat"></sl-icon>
                  </sl-button>
                  <span class="hocr-conf">{{ word.wconf }}%</span>
                </div>
              </div>

            </div>
          </div>

        </div>
      </div>

    </div>
  </div>

  <div v-else class="hocr-empty">
    No hOCR loaded for this page
  </div>
</template>

<script setup lang="ts">
import {inject, reactive, ref, watch} from 'vue';
import type { Ref } from 'vue';
import { useHocrContext } from '../composables/useHocr';
import type { HocrLevel, HocrCarea, HocrBlock, HocrLine, HocrWord, EditorPalette, Evidence } from '../types';
import { DEFAULT_PALETTE, findMultilevelById } from '../types';
import type { FlowSchema, LayoutSchema, ColorSpecification } from '../types';
import { applyColorSpecs } from '../utils/colors';

const props = withDefaults(defineProps<{
  palette?: EditorPalette;
  flows?: FlowSchema[];
  layouts?: LayoutSchema[];
  initialCollapseMode?: 'all' | 'block' | 'carea' | 'none';
}>(), {
  palette: () => DEFAULT_PALETTE,
  flows: () => [],
  layouts: () => [],
  initialCollapseMode: 'none',
});

const { hocrPage, machineName, stem, rescanCarea, rescanWord } = useHocrContext();
const selectedItemIds   = inject<Ref<Set<string>>>('selectedItemIds', ref(new Set()));
const indicatedItemId   = inject<Ref<string | null>>('indicatedItemId', ref(null));
const selectNodeCb      = inject<(level: string, id: string, e?: MouseEvent) => void>('selectNode', () => {});

function indicate(id: string | null) {
  if (indicatedItemId) indicatedItemId.value = id;
}

watch(hocrPage, (newPage) => {
  if (newPage) {
    applyInitialCollapse();
  }
}, { immediate: true });

function applyInitialCollapse() {
  if (!props.initialCollapseMode || props.initialCollapseMode === 'none') return;

  if (props.initialCollapseMode === 'all') {
    collapseAll();
  } else if (props.initialCollapseMode === 'carea') {
    collapseAll();
  } else if (props.initialCollapseMode === 'block') {
    collapseToBlock();
  }
}

function collapseToBlock() {
  collapsedCareas.clear();
  expandedBlocks.clear();
  expandedLines.clear();
}

watch(selectedItemIds, () => {
  if (! hocrPage || ! selectedItemIds.value.size) return;

  for (const id of selectedItemIds.value) {
    let chain = findMultilevelById(hocrPage.value!, id)
    if (chain) {
      if(chain.carea) collapsedCareas.delete(chain.carea.id);
      if(chain.block) expandedBlocks.add(chain.block.id);
      if(chain.line) expandedLines.add(chain.line.id);
    }
  }
}, { deep: true })

// ── Collapse / expand state ──────────────────────────────────────────
// Careas: empty = all expanded.  Blocks/lines: empty = all collapsed.
const collapsedCareas = reactive(new Set<string>());
const expandedBlocks  = reactive(new Set<string>());
const expandedLines   = reactive(new Set<string>());

function toggleCarea(id: string) {
  if (collapsedCareas.has(id)) collapsedCareas.delete(id);
  else collapsedCareas.add(id);
}
function toggleBlock(id: string) {
  if (expandedBlocks.has(id)) expandedBlocks.delete(id);
  else expandedBlocks.add(id);
}
function toggleLine(id: string) {
  if (expandedLines.has(id)) expandedLines.delete(id);
  else expandedLines.add(id);
}

// ── Selection ────────────────────────────────────────────────────────
function selectNode(level: HocrLevel, id: string, e?: MouseEvent) {
  selectNodeCb(level, id, e);
}

function getEvidenceClass(ev: Evidence) {
  if (typeof ev === 'string') {
    return `ev-${ev}`;
  }
  const key = Object.keys(ev)[0];
  const val = (ev as any)[key];
  return `ev-${key} ev-val-${val}`;
}


function getEvidenceText(ev: Evidence): string {
  if (typeof ev === 'string') return ev;
  const key = Object.keys(ev)[0];
  const val = (ev as any)[key];
  return `${val}`;
}

function rescan(careaId: string) {
  if (!machineName.value || !stem.value) return;
  if (!window.confirm("Are you sure you want to rescan this carea? This will append new results to the existing ones.")) return;
  rescanCarea(machineName.value, stem.value, careaId);
}

function rescanWordAction(wordId: string) {
  if (!machineName.value || !stem.value) return;
  if (!window.confirm("Are you sure you want to rescan this word? This will replace the word with new results.")) return;
  rescanWord(machineName.value, stem.value, wordId);
}

// ── Display helpers ──────────────────────────────────────────────────
const blockAbbreviations: Record<string, string> = {
  part: 'Part', chapter: 'H1', section: 'H2', subsection: 'H3',
  subsubsection: 'H4', subsubsubsection: 'H5', subsubsubsubsection: 'H6',
  paragraph: 'P', image: 'IMG', table: 'TBL', list: 'LST',
};
const blockKinds = Object.keys(blockAbbreviations);

function blockBadge(block: HocrBlock): string {
  return blockAbbreviations[block.kind] ?? block.kind;
}

function wordText(word: HocrWord): string {
  if (word.dropcap) {
    return `[${word.dropcap}]${word.text}`;
  }
  return word.text;
}

function lineText(line: HocrLine): string {
  return line.words.map(wordText).join(' ');
}

function blockPreview(block: HocrBlock, maxLen = 60): string {
  const abbrev = blockBadge(block);
  const preview = abbrev === 'IMG' ? block.bbox : block.lines.map(lineText).join(' ')
  const text = blockKinds.includes(block.kind)
      ? abbrev + ': ' + preview
      : `--unknown: ${block.kind}`;
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
}

function expandAll() {
  if (!hocrPage.value) return;
  collapsedCareas.clear();
  expandedBlocks.clear();
  expandedLines.clear();
  for (const carea of hocrPage.value.careas) {
    for (const block of carea.blocks) {
      expandedBlocks.add(block.id);
      for (const line of block.lines) {
        expandedLines.add(line.id);
      }
    }
  }
}

function collapseAll() {
  if (!hocrPage.value) return;
  expandedBlocks.clear();
  expandedLines.clear();
  for (const carea of hocrPage.value.careas) {
    collapsedCareas.add(carea.id);
  }
}

defineExpose({ expandAll, collapseAll });

function careaBadgeStyle(carea: HocrCarea) {
  const specs: ColorSpecification[] = [];

  if (carea.flow) {
    const flow = props.flows.find(f => f.name === carea.flow);
    if (flow?.color) specs.push(flow.color);
  }

  if (carea.layout) {
    const layout = props.layouts.find(l => l.name === carea.layout);
    if (layout?.color) specs.push(layout.color);
  }

  return {
    backgroundColor: applyColorSpecs(props.palette.careaOverlayColor, specs),
    color: '#fff'
  };
}

function getBadgeDetails(type: 'flow' | 'layout', value?: string) {
  const schemas = type === 'flow' ? props.flows : props.layouts;
  const label = type === 'flow' ? 'flow' : 'layout';

  if (schemas.length === 0) {
    return {
      text: '',
      class: 'hocr-badge-pill hocr-badge-none',
      style: {}
    };
  }

  if (!value) {
    return {
      text: `No ${label}`,
      class: 'hocr-badge-pill hocr-badge-alarm',
      style: {}
    };
  }

  const schema = schemas.find(s => s.name === value);
  if (!schema) {
    return {
      text: `Bad ${label}`,
      class: 'hocr-badge-pill hocr-badge-alarm',
      style: {}
    };
  }

  const specs: ColorSpecification[] = [];
  if (schema.color) specs.push(schema.color);

  const color = applyColorSpecs(props.palette.careaOverlayColor, specs);

  return {
    text: value,
    class: `hocr-badge-pill hocr-badge-${type}`,
    style: {
      borderColor: color,
      color: color
    }
  };
}

function careaPreview(carea: HocrCarea, maxLen = 60): string {
  const text = carea.blocks
      .flatMap(p => blockKinds.includes(p.kind) ? p.lines : [])
      .map(lineText).join(' ');
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
}
</script>

<style scoped>
.hocr-outline {
  font-size: 0.72rem;
  font-family: ui-monospace, monospace;
}

.hocr-item { }

.hocr-children {
  padding-left: 1rem;
}

.hocr-row {
  display: flex;
  align-items: baseline;
  gap: 0.3rem;
  padding: 0.15rem 0.4rem;
  user-select: none;
  cursor: default;
  border-radius: 0.2rem;
}

.hocr-row:hover {
  background: var(--color-bg-muted, #f1f3f5);
}

.hocr-row-selected {
  background: color-mix(in srgb, var(--color-accent, #2563eb) 12%, transparent) !important;
  outline: 1px solid color-mix(in srgb, var(--color-accent, #2563eb) 40%, transparent);
}

.hocr-carea-row { cursor: pointer; }
.hocr-block-row { cursor: pointer; }
.hocr-line-row  { cursor: pointer; }

.hocr-badge {
  flex-shrink: 0;
  display: inline-block;
  min-width: 1.6em;
  height: 1.3em;
  line-height: 1.3em;
  text-align: center;
  border-radius: 2px;
  font-size: 0.65rem;
  font-weight: 700;
  cursor: pointer;
  padding: 0 0.2em;
}

.hocr-badge-c { background: #f97316; color: #fff; }
.hocr-badge-p { background: #a855f7; color: #fff; }
.hocr-badge-l { background: #2563eb; color: #fff; }
.hocr-badge-w { background: #16a34a; color: #fff; }

.hocr-badge-pill {
  flex-shrink: 0;
  display: inline-block;
  padding: 0 0.4rem;
  border-radius: 1rem;
  font-size: 0.6rem;
  font-weight: 600;
  border: 1px solid transparent;
  background: white;
  line-height: 1.2;
}

.hocr-badge-alarm {
  background: #fee2e2;
  color: #b91c1c;
  border-color: #f87171;
}

.hocr-badge-none {
  border-color: #d1d5db;
  min-width: 2rem;
  height: 0.8rem;
}

.hocr-count {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.7em;
}

.hocr-id {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  max-width: 7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hocr-lang {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.hocr-preview {
  flex: 1;
  min-width: 0;
  color: var(--color-text-muted, #6c757d);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hocr-rescan-btn {
  padding: 0;
  margin: 0;
  height: auto;
  --sl-input-height-small: 1.2rem;
  --sl-button-font-size-small: 0.8rem;
}

.hocr-rescan-btn::part(base) {
  padding: 0 0.2rem;
}

.hocr-toggle {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.55rem;
}

.hocr-conf {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.7em;
}

.hocr-empty {
  padding: 1rem 0.75rem;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.8rem;
  font-style: italic;
}

.hocr-hints-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  padding: 0.1rem 0.4rem 0.3rem 2rem;
}

.hocr-hint-item {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  font-size: 0.6rem;
  padding: 0.05rem 0.3rem;
  background: var(--color-bg-muted, #f8f9fa);
  border-radius: 4px;
  border: 1px solid var(--color-border-muted, #e9ecef);
}

.hocr-hint-key {
  color: var(--color-text-dimmed, #a2acb6);
  font-weight: 600;
  text-transform: capitalize;
}

.hocr-hint-value {
  font-weight: 700;
}

.ev-suggested {
  color: #16a34a;
  border-color: color-mix(in srgb, #16a34a 20%, transparent);
  background: color-mix(in srgb, #16a34a 5%, transparent);
}
.ev-suggested.ev-val-false {
  color: #a2acb6;
  opacity: 0.6;
}

.ev-determined {
  color: #16a34a;
  border-color: #16a34a;
  background: color-mix(in srgb, #16a34a 10%, transparent);
}

.ev-assigned {
  color: #2563eb;
  border-color: #2563eb;
  background: color-mix(in srgb, #2563eb 10%, transparent);
}

.ev-error {
  color: #dc2626;
  border-color: #dc2626;
  background: color-mix(in srgb, #dc2626 10%, transparent);
}
</style>
