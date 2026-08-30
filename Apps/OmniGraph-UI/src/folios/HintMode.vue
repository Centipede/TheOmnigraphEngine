<template>
  <PageWorkspace
      ref="workspaceRef"
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :pointer-settings="pointerSettings"
      :keyboard-handler="handleKeyDown"
      @current-page-change="currentPage = $event"
      @pages-loaded="onPagesLoaded"
  >
    <template #page-preview="workspace">
      <PagePreview
          v-if="workspace.currentPage && workspace.currentPageCrop"
          :page="workspace.currentPage"
          :image-base-url="workspace.scanBaseUrl"
          :crop="workspace.currentPageCrop"
          :show-crop-overlay="true"
          :crop-color="'rgba(0, 0, 0, 0.0)'"
          :discard-color="'rgba(50, 50, 50, 0.35)'"
          :pointer-settings="pointerSettings"
          :interaction-drag="handleInteractionDrag"
          :machine-name="machineName"
      />
    </template>

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
        </sl-button-group>

        <br><br>

        <div class="hint-list-header">
          Hints on this page
        </div>

        <div class="hint-list" v-if="currentPage && currentPage.hints && currentPage.hints.length > 0">
          <div v-for="(hint, index) in currentPage.hints" :key="index" class="hint-item">
            <div class="hint-main-row">
              <sl-badge :variant="hint.type === 'dropcap' ? 'warning' : 'info'">
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
import PagePreview from '../components/PagePreview.vue';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import type { Page, Hint, HintType, PointerSettings } from '../types';
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
  if (!activeTool.value) return { enabled: false, color: '', icon: '', label: '' };
  return {
    enabled: true,
    label: activeTool.value === 'dropcap' ? 'Dropcap' : 'Image',
    color: activeTool.value === 'dropcap' ? 'rgba(255, 140, 0, 1)' : 'rgba(0, 191, 255, 1)',
    icon: '',
  };
});

function handleKeyDown(e: KeyboardEvent) {
  if (e.key.toLowerCase() === 'd') {
    activeTool.value = 'dropcap';
    return true;
  }
  if (e.key.toLowerCase() === 'i') {
    activeTool.value = 'image';
    return true;
  }
  if (e.key === 'Escape') {
    activeTool.value = null;
    return true;
  }
  return false;
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
