<template>
  <PageWorkspace
      ref="workspaceRef"
      :machine-name="machineName"
      :project-name="projectName"
      :initial-page-stem="initialPageStem"
      :panels="panels"
      :page-list-columns="['index', 'batch', 'name', 'scan']"
      :can-pages-be-filtered="false"
  >
    <template #tools>
      <div class="process-controls">
        <h3>Image Processing Lab</h3>
        <p>This mode is for non-destructive image preprocessing settings.</p>

        <div v-if="project" class="settings-form">
          <sl-checkbox
              :checked="project.processing.desaturate"
              @sl-change="toggleDesaturate"
          >
            Desaturate (Grayscale)
          </sl-checkbox>
          <p class="help-text">Convert all book scans to grayscale for better OCR results.</p>

          <div class="control-group">
            <label class="label">Contrast</label>
            <sl-input
                type="number"
                step="0.1"
                :value="project.processing.contrast"
                @sl-input="updateContrast"
            ></sl-input>
            <p class="help-text">Adjust image contrast. 0.0 is neutral. Positive values increase contrast.</p>
          </div>

          <div class="control-group">
            <label class="label">Brightness</label>
            <sl-input
                type="number"
                step="1"
                :value="project.processing.brightness"
                @sl-input="updateBrightness"
            ></sl-input>
            <p class="help-text">Adjust image brightness. 0.0 is neutral.</p>
          </div>

          <sl-button
              variant="primary"
              :loading="isSaving"
              @click="saveSettings"
          >
            Save Settings
          </sl-button>
        </div>
        <p v-else>Loading settings...</p>
      </div>
    </template>
  </PageWorkspace>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue';
import PageWorkspace from "../components/PageWorkspace.vue";
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';
import type { Project } from "../types/project";

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const panels = usePersistentPanels('panels.process', {
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

const project = ref<Project | null>(null);
const isSaving = ref(false);

async function fetchProject() {
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`);
    if (resp.ok) {
      project.value = await resp.json();
    }
  } catch (e) {
    console.error('Failed to fetch project', e);
  }
}

function toggleDesaturate(e: any) {
  if (project.value) {
    project.value.processing.desaturate = e.target.checked;
  }
}

function updateContrast(e: any) {
  if (project.value) {
    project.value.processing.contrast = parseFloat(e.target.value) || 0;
  }
}

function updateBrightness(e: any) {
  if (project.value) {
    project.value.processing.brightness = parseFloat(e.target.value) || 0;
  }
}

async function saveSettings() {
  if (!project.value) return;

  isSaving.value = true;
  try {
    const resp = await fetch(`/api/projects/${props.machineName}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(project.value),
    });

    if (!resp.ok) {
      const text = await resp.text();
      showError?.(`Failed to save settings: ${text}`);
    }
  } catch (e) {
    console.error('Failed to save project settings', e);
    showError?.('Failed to save project settings');
  } finally {
    isSaving.value = false;
  }
}

onMounted(() => {
  setActivePanels(panels);
  void fetchProject();
});
onUnmounted(() => setActivePanels(null));

</script>

<style scoped>
.process-controls {
  padding: 0.5rem;
}

.settings-form {
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.help-text {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  margin-top: -0.5rem;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.label {
  font-size: 0.9rem;
  font-weight: 500;
}
</style>
