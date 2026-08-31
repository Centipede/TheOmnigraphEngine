<template>
  <PageWorkspace
      ref="workspaceRef"
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :page-list-columns="['name-or-scan', 'extras']"
      :pointer-settings="pointerSettings"
      :keyboard-handler="handleKeyDown"
      :page-interaction-drag="handleInteractionDrag"
      :show-crop-overlay="true"
      :show-hints="true"
      :palette="{ ...DEFAULT_PALETTE, keepColor: 'rgba(0, 0, 0, 0.0)', discardColor: 'rgba(50, 50, 50, 0.35)'}"
      @current-page-change="currentPage = $event"
      @pages-loaded="onPagesLoaded"
  >

    <template #tools>
      <div class="hint-tools">
        <sl-button-group>
          <sl-button
              :variant="activeTool === 'dropcap' ? 'primary' : 'default'"
              @click="activeTool = 'dropcap'"
              size="small"
          >
            Dropcap <span class="hint-key">D</span>
          </sl-button>
          <sl-button
              :variant="activeTool === 'image' ? 'primary' : 'default'"
              @click="activeTool = 'image'"
              size="small"
          >
            Image <span class="hint-key">I</span>
          </sl-button>
          <sl-button
              :variant="activeTool === 'callout' ? 'primary' : 'default'"
              @click="activeTool = 'callout'"
              size="small"
          >
            Callout <span class="hint-key">C</span>
          </sl-button>
          <sl-button
              :variant="activeTool === 'garbage' ? 'primary' : 'default'"
              @click="activeTool = 'garbage'"
              size="small"
          >
            Garbage <span class="hint-key">G</span>
          </sl-button>
        </sl-button-group>

        <br><br>

        <div class="hint-list-header">
          Hints on this page
        </div>

        <div class="hint-list" v-if="currentPage && currentPage.hints && currentPage.hints.length > 0">
          <div v-for="(hint, index) in currentPage.hints" :key="index" class="hint-item">
            <div class="hint-main-row">
              <sl-badge :variant="getBadgeVariant(hint.type)">
                {{ hint.type }}
              </sl-badge>
              <sl-input
                  v-if="hint.type === 'dropcap'"
                  size="small"
                  placeholder="Letter"
                  :value="hint.letter"
                  @sl-input="hint.letter = ($event.target as HTMLInputElement).value"
                  @sl-change="workspaceRef?.savePageDb()"
                  class="hint-letter-input"
              ></sl-input>
              <div style="flex: 1"></div>
              <sl-icon-button
                  name="x-lg"
                  label="Remove hint"
                  @click="removeHint(index)"
              ></sl-icon-button>
            </div>
            <div class="hint-area">
              {{ hint.area.left }},{{ hint.area.top }} - {{ hint.area.right }},{{ hint.area.bottom }}
            </div>
          </div>
        </div>
        <div v-else class="hint-list-empty">
          No hints marked.
        </div>
      </div>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import PageWorkspace from '../components/PageWorkspace.vue';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import {type Page, type Hint, type HintType, type PointerSettings, DEFAULT_PALETTE} from '../types';
import {provideHocrContext} from "../composables/useHocr.ts";

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

const workspaceRef = ref<InstanceType<typeof PageWorkspace> | null>(null);
const currentPage = ref<Page | null>(null);
const activeTool = ref<HintType | null>(null);

const panels = usePersistentPanels('panels.hint', {
  'page-list': true,
  'page-strips': false,
  'page-preview': true,
  'section-structure': false,
  'ocr-structure': false,
  tools: true,
  'structural-tree': false,
});

const { setActivePanels } = usePanelVisibilityContext();

provideHocrContext();

onMounted(() => setActivePanels(panels));
onUnmounted(() => setActivePanels(null));

const pointerSettings = computed((): PointerSettings => {
  if (!activeTool.value) return { enabled: false, color: '#808080', icon: 'crosshair', label: '' };
  
  const settings: Record<string, { label: string, color: string }> = {
    dropcap: { label: 'Dropcap', color: 'rgba(255, 140, 0, 1)' },
    image: { label: 'Image', color: 'rgba(0, 191, 255, 1)' },
    callout: { label: 'Callout', color: 'rgba(40, 167, 69, 1)' },
    garbage: { label: 'Garbage', color: 'rgba(220, 53, 69, 1)' },
  };

  const current = settings[activeTool.value] || { label: '', color: 'rgba(128, 128, 128, 1)' };

  return {
    enabled: true,
    label: current.label,
    color: current.color,
    icon: 'crosshair',
  };
});

function handleKeyDown(e: KeyboardEvent) {
  const key = e.key.toLowerCase();
  if (key === 'd') {
    activeTool.value = 'dropcap';
    return true;
  }
  if (key === 'i') {
    activeTool.value = 'image';
    return true;
  }
  if (key === 'c') {
    activeTool.value = 'callout';
    return true;
  }
  if (key === 'g') {
    activeTool.value = 'garbage';
    return true;
  }
  if (e.key === 'Escape') {
    activeTool.value = null;
    return true;
  }
  return false;
}

function getBadgeVariant(type: HintType) {
  if (type === 'dropcap') return 'warning';
  if (type === 'image') return 'info';
  if (type === 'callout') return 'success';
  if (type === 'garbage') return 'danger';
  return 'neutral';
}

async function handleInteractionDrag(x1: number, y1: number, x2: number, y2: number) {
  if (!activeTool.value || !currentPage.value) return;

  const hint: Hint = {
    type: activeTool.value,
    letter: activeTool.value === 'dropcap' ? '' : undefined,
    area: {
      left: Math.min(x1, x2),
      top: Math.min(y1, y2),
      right: Math.max(x1, x2),
      bottom: Math.max(y1, y2),
    },
  };

  if (!currentPage.value.hints) {
    currentPage.value.hints = [];
  }
  currentPage.value.hints.push(hint);

  try {
    await workspaceRef.value?.savePageDb();
  } catch (e) {
    console.error('Failed to save hint:', e);
  }
}

async function removeHint(index: number) {
  if (!currentPage.value || !currentPage.value.hints) return;
  currentPage.value.hints.splice(index, 1);
  try {
    await workspaceRef.value?.savePageDb();
  } catch (e) {
    console.error('Failed to save hint removal:', e);
  }
}

function onPagesLoaded() {
  // PageWorkspace handles initial page focus
}

</script>

<style scoped>
.hint-tools {
  display: flex;
  flex-direction: column;
}

.hint-key {
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  margin-left: 0.2rem;
  font-size: 0.8em;
  opacity: 0.7;
}

.hint-list-header {
  font-weight: bold;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}

.hint-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 400px;
  overflow-y: auto;
}

.hint-item {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.4rem;
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 4px;
  background: var(--color-bg-muted, #f8f9fa);
}

.hint-main-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.hint-area {
  font-family: monospace;
  font-size: 0.7rem;
  color: var(--color-text-muted, #6c757d);
}

.hint-letter-input {
  width: 4rem;
}

.hint-letter-input::part(base) {
  height: 1.5rem;
}

.hint-letter-input::part(input) {
  padding: 0 0.25rem;
}

.hint-list-empty {
  font-style: italic;
  color: var(--color-text-muted, #6c757d);
  font-size: 0.85rem;
}
</style>
