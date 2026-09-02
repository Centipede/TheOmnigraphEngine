<template>
  <div class="project-detail-page">
    <p v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </p>

    <p v-if="isLoading" class="muted">
      Loading project…
    </p>

    <template v-else-if="project && draft">
      <div class="page-header">
        <template v-if="mode === 'view'">
          <RouterLink to="/projects" class="back-link">
            ← Projects
          </RouterLink>
          <h2>{{ project.name }}</h2>
        </template>
        <template v-else>
          <button type="button" class="back-link button-link" @click="cancelEdit">
            ← {{ project.name }}
          </button>
          <h2>Edit Project</h2>
        </template>
      </div>

      <div v-if="blockingInfo" class="blocking-alert">
        <h3>Deletion Blocked</h3>
        <p>The following items cannot be removed because they are in use:</p>
        <ul>
          <li v-for="(pages, item) in blockingInfo" :key="item">
            <strong>{{ item }}</strong> used on: {{ pages.join(", ") }}
          </li>
        </ul>
      </div>

      <form @submit.prevent="acceptEdit">
        <div class="project-grid">
          <!-- Left Column: Metadata -->
          <section class="grid-column">
            <h3>Metadata</h3>
            <template v-if="mode === 'view'">
              <dl class="metadata-list">
                <dt>Machine name</dt>
                <dd>
                  <code>{{ project.machine_name }}</code>
                </dd>

                <template v-if="project.abbrev">
                  <dt>Abbreviation</dt>
                  <dd>{{ project.abbrev }}</dd>
                </template>

                <template v-if="project.description">
                  <dt>Description</dt>
                  <dd>{{ project.description }}</dd>
                </template>

                <template v-if="project.published">
                  <dt>Published</dt>
                  <dd>{{ formatDate(project.published) }}</dd>
                </template>

                <template v-if="project.ocr_language">
                  <dt>OCR Language</dt>
                  <dd>{{ project.ocr_language }}</dd>
                </template>

                <template v-if="project.authors.length > 0">
                  <dt>Authors</dt>
                  <dd>
                    <template
                        v-for="(author, index) in project.authors"
                        :key="`${author.full_name}-${index}`"
                    >
                      {{ author.full_name }}<template v-if="author.abbrev"> ({{ author.abbrev }})</template><template v-if="index < project.authors.length - 1">, </template>
                    </template>
                  </dd>
                </template>
              </dl>

              <sl-button variant="text" @click="mode = 'edit'">
                Edit Metadata
              </sl-button>
            </template>

            <template v-else>
              <div class="form-grid">
                <label for="name">Name</label>
                <input
                    id="name"
                    v-model="draft.name"
                    type="text"
                    name="name"
                    required
                >

                <label for="abbrev">Abbreviation</label>
                <input
                    id="abbrev"
                    v-model="draft.abbrev"
                    type="text"
                    name="abbrev"
                >

                <label for="description">Description</label>
                <textarea
                    id="description"
                    v-model="draft.description"
                    name="description"
                    rows="3"
                />

                <label for="published">Published</label>
                <input
                    id="published"
                    v-model="draft.published"
                    type="date"
                    name="published"
                >

                <label for="ocr_language">OCR Language</label>
                <input
                    id="ocr_language"
                    v-model="draft.ocr_language"
                    type="text"
                    name="ocr_language"
                    placeholder="e.g. eng, fra, deu"
                >

                <label>Authors</label>
                <div>
                  <div class="authors-list">
                    <div
                        v-for="(author, index) in draft.authors"
                        :key="index"
                        class="author-row"
                    >
                      <input
                          v-model="author.full_name"
                          type="text"
                          name="author_names"
                          placeholder="Full name"
                          class="author-name-input"
                      >

                      <input
                          v-model="author.abbrev"
                          type="text"
                          name="author_abbrevs"
                          placeholder="Abbrev"
                          class="author-abbrev-input"
                      >

                      <button
                          type="button"
                          class="remove-author"
                          title="Remove author"
                          @click="removeAuthor(index)"
                      >
                        ×
                      </button>
                    </div>
                  </div>

                  <sl-button type="button" variant="text" size="small" @click="addAuthor">
                    + Add author
                  </sl-button>
                </div>
              </div>
            </template>
          </section>

          <!-- Middle Column: Base Colors -->
          <section class="grid-column">
            <h3>Base Colors</h3>
            <div v-if="mode === 'view'">
              <ul class="schema-list">
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.careaOverlayColor }"></span>
                  CAREA Overlay
                </li>
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.blockOverlayColor }"></span>
                  Block Overlay
                </li>
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.lineOverlayColor }"></span>
                  Line Overlay
                </li>
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.wordOverlayColor }"></span>
                  Word Overlay
                </li>
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.keepColor }"></span>
                  Keep Color
                </li>
                <li>
                  <span class="color-swatch" :style="{ backgroundColor: project.editor_palette.discardColor }"></span>
                  Discard Color
                </li>
              </ul>
            </div>
            <div v-else>
              <div class="form-grid">
                <div class="color-field">
                  <label>CAREA Overlay</label>
                  <sl-color-picker :value="draft.editor_palette.careaOverlayColor" @sl-input="draft.editor_palette.careaOverlayColor = $event.target.value" label="CAREA Overlay Color"></sl-color-picker>
                </div>
                <div class="color-field">
                  <label>Block Overlay</label>
                  <sl-color-picker :value="draft.editor_palette.blockOverlayColor" @sl-input="draft.editor_palette.blockOverlayColor = $event.target.value" label="Block Overlay Color"></sl-color-picker>
                </div>
                <div class="color-field">
                  <label>Line Overlay</label>
                  <sl-color-picker :value="draft.editor_palette.lineOverlayColor" @sl-input="draft.editor_palette.lineOverlayColor = $event.target.value" label="Line Overlay Color"></sl-color-picker>
                </div>
                <div class="color-field">
                  <label>Word Overlay</label>
                  <sl-color-picker :value="draft.editor_palette.wordOverlayColor" @sl-input="draft.editor_palette.wordOverlayColor = $event.target.value" label="Word Overlay Color"></sl-color-picker>
                </div>
                <div class="color-field">
                  <label>Keep Color</label>
                  <sl-color-picker :value="draft.editor_palette.keepColor" @sl-input="draft.editor_palette.keepColor = $event.target.value" label="Keep Color" opacity></sl-color-picker>
                </div>
                <div class="color-field">
                  <label>Discard Color</label>
                  <sl-color-picker :value="draft.editor_palette.discardColor" @sl-input="draft.editor_palette.discardColor = $event.target.value" label="Discard Color" opacity></sl-color-picker>
                </div>
              </div>
            </div>
          </section>

          <!-- Column 3: Block Colors -->
          <section class="grid-column">
            <h3>Block Colors</h3>
            <div v-if="mode === 'view'">
              <ul class="schema-list">
                <li v-for="item in blockTypeItems" :key="item.key">
                  <span
                    class="color-swatch"
                    :style="{ backgroundColor: getEffectiveColor((project.editor_palette as any)[item.key], true) }"
                  ></span>
                  {{ item.label }}
                </li>
              </ul>
            </div>
            <div v-else>
              <div class="schema-edit-list">
                <div v-for="item in blockTypeItems" :key="item.key">
                  <div class="schema-row">
                    <span
                      class="color-swatch"
                      :style="{ backgroundColor: getEffectiveColor((draft.editor_palette as any)[item.key], true) }"
                    ></span>
                    <span class="flex-grow">{{ item.label }}</span>
                    <sl-color-picker
                      v-if="(draft.editor_palette as any)[item.key]"
                      :value="(draft.editor_palette as any)[item.key].base_color"
                      @sl-input="(draft.editor_palette as any)[item.key].base_color = $event.target.value"
                      label="Choose base color"
                    ></sl-color-picker>
                  </div>
                  <div v-if="(draft.editor_palette as any)[item.key]" class="color-details">
                    <div class="shift-group">
                      <label>Hue</label>
                      <input
                        type="range"
                        v-model.number="(draft.editor_palette as any)[item.key].hue_shift"
                        min="-50"
                        max="50"
                        step="5"
                      />
                      <div class="shift-value">{{ (draft.editor_palette as any)[item.key].hue_shift }}°</div>
                    </div>
                    <div class="shift-group">
                      <label>Sat</label>
                      <input
                        type="range"
                        v-model.number="(draft.editor_palette as any)[item.key].saturation_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ (draft.editor_palette as any)[item.key].saturation_shift }}%</div>
                    </div>
                    <div class="shift-group">
                      <label>Light</label>
                      <input
                        type="range"
                        v-model.number="(draft.editor_palette as any)[item.key].lightness_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ (draft.editor_palette as any)[item.key].lightness_shift }}%</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- Right Column: Layouts and Flows -->
          <section class="grid-column">
            <h3>Layouts & Flows</h3>
            <div v-if="mode === 'view'">
              <h4 class="sub-heading">Layouts</h4>
              <ul v-if="project.layouts.length > 0" class="schema-list">
                <li v-for="layout in project.layouts" :key="layout.name">
                  <span
                    class="color-swatch"
                    :style="{ backgroundColor: getEffectiveColor(layout.color) }"
                  ></span>
                  {{ layout.name }}
                </li>
              </ul>
              <p v-else class="muted">No layouts defined.</p>

              <h4 class="sub-heading">Flows</h4>
              <ul v-if="project.flows.length > 0" class="schema-list">
                <li v-for="flow in project.flows" :key="flow.name">
                  <span
                    class="color-swatch"
                    :style="{ backgroundColor: getEffectiveColor(flow.color) }"
                  ></span>
                  {{ flow.name }}
                </li>
              </ul>
              <p v-else class="muted">No flows defined.</p>
            </div>
            <div v-else>
              <h4 class="sub-heading">Layouts</h4>
              <div class="schema-edit-list">
                <div v-for="(layout, index) in draft.layouts" :key="index">
                  <div class="schema-row">
                    <span
                      class="color-swatch"
                      :style="{ backgroundColor: getEffectiveColor(layout.color) }"
                    ></span>
                    <input
                      v-model="layout.name"
                      type="text"
                      placeholder="Layout name"
                      required
                    />
                    <sl-color-picker
                      v-if="layout.color"
                      :value="layout.color.base_color"
                      @sl-input="layout.color.base_color = $event.target.value"
                      label="Choose base color"
                    ></sl-color-picker>
                    <button
                      type="button"
                      class="remove-btn"
                      title="Remove layout"
                      @click="removeLayout(index)"
                    >
                      ×
                    </button>
                  </div>
                  <div v-if="layout.color" class="color-details">
                    <div class="shift-group">
                      <label>Hue</label>
                      <input
                        type="range"
                        v-model.number="layout.color.hue_shift"
                        min="-50"
                        max="50"
                        step="5"
                      />
                      <div class="shift-value">{{ layout.color.hue_shift }}°</div>
                    </div>
                    <div class="shift-group">
                      <label>Sat</label>
                      <input
                        type="range"
                        v-model.number="layout.color.saturation_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ layout.color.saturation_shift }}%</div>
                    </div>
                    <div class="shift-group">
                      <label>Light</label>
                      <input
                        type="range"
                        v-model.number="layout.color.lightness_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ layout.color.lightness_shift }}%</div>
                    </div>
                  </div>
                </div>
              </div>
              <sl-button variant="text" size="small" @click="addLayout">
                + Add Layout
              </sl-button>

              <h4 class="sub-heading mt-4">Flows</h4>
              <div class="schema-edit-list">
                <div v-for="(flow, index) in draft.flows" :key="index">
                  <div class="schema-row">
                    <span
                      class="color-swatch"
                      :style="{ backgroundColor: getEffectiveColor(flow.color) }"
                    ></span>
                    <input
                      v-model="flow.name"
                      type="text"
                      placeholder="Flow name"
                      required
                    />
                    <sl-color-picker
                      v-if="flow.color"
                      :value="flow.color.base_color"
                      @sl-input="flow.color.base_color = $event.target.value"
                      label="Choose base color"
                    ></sl-color-picker>
                    <button
                      type="button"
                      class="remove-btn"
                      title="Remove flow"
                      @click="removeFlow(index)"
                    >
                      ×
                    </button>
                  </div>
                  <div v-if="flow.color" class="color-details">
                    <div class="shift-group">
                      <label>Hue</label>
                      <input
                        type="range"
                        v-model.number="flow.color.hue_shift"
                        min="-50"
                        max="50"
                        step="5"
                      />
                      <div class="shift-value">{{ flow.color.hue_shift }}°</div>
                    </div>
                    <div class="shift-group">
                      <label>Sat</label>
                      <input
                        type="range"
                        v-model.number="flow.color.saturation_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ flow.color.saturation_shift }}%</div>
                    </div>
                    <div class="shift-group">
                      <label>Light</label>
                      <input
                        type="range"
                        v-model.number="flow.color.lightness_shift"
                        min="-50"
                        max="50"
                        step="10"
                      />
                      <div class="shift-value">{{ flow.color.lightness_shift }}%</div>
                    </div>
                  </div>
                </div>
              </div>
              <sl-button variant="text" size="small" @click="addFlow">
                + Add Flow
              </sl-button>
            </div>
          </section>
        </div>

        <div v-if="mode === 'edit'" class="page-actions">
          <sl-button type="button" variant="default" :disabled="isSaving" @click="cancelEdit">
            Cancel
          </sl-button>

          <sl-button type="submit" variant="primary" :loading="isSaving" :disabled="isSaving">
            Save Changes
          </sl-button>
        </div>
      </form>
    </template>

    <p v-else class="muted">
      Project could not be loaded.
    </p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { Project, ColorSpecification } from "../types/project";
import { DEFAULT_PALETTE } from "../types/hocr_interaction";
import { applyColorSpecs } from "../utils/colors";
import { useActiveProjectContext } from "../composables/useActiveProject";

const props = defineProps<{ machineName: string; projectName: string }>();

const { activeProject } = useActiveProjectContext();

// ── Mode ─────────────────────────────────────────────────────────────
type Mode = 'view' | 'edit';
const mode = ref<Mode>('view');

const project = ref<Project>();
const draft = ref<Project>();

const blockTypeItems = [
  { key: 'partColor', label: 'Part' },
  { key: 'h1Color', label: 'Heading 1' },
  { key: 'h2Color', label: 'Heading 2' },
  { key: 'h3Color', label: 'Heading 3' },
  { key: 'h4Color', label: 'Heading 4' },
  { key: 'h5Color', label: 'Heading 5' },
  { key: 'h6Color', label: 'Heading 6' },
  { key: 'pColor', label: 'Paragraph' },
  { key: 'imgColor', label: 'Image' },
  { key: 'lstColor', label: 'List' },
  { key: 'tblColor', label: 'Table' },
] as const;

const isLoading = ref(false);
const isSaving = ref(false);
const errorMessage = ref('');
const blockingInfo = ref<Record<string, string[]>>();

function getEffectiveColor(spec: ColorSpecification | undefined, isBlock = false): string {
  const base = isBlock
    ? (draft.value?.editor_palette?.blockOverlayColor ?? DEFAULT_PALETTE.blockOverlayColor)
    : (draft.value?.editor_palette?.careaOverlayColor ?? DEFAULT_PALETTE.careaOverlayColor);
  if (!spec) return base;
  return applyColorSpecs(base, [spec]);
}

function formatDate(value: string): string {
  return value.slice(0, 10);
}

function addAuthor(): void {
  if (!draft.value) {
    return;
  }

  draft.value.authors.push({
    full_name: '',
    abbrev: null,
  });
}

function removeAuthor(index: number): void {
  if (!draft.value) {
    return;
  }

  draft.value.authors.splice(index, 1);
}

function addLayout(): void {
  if (!draft.value) {
    return;
  }
  const base = draft.value.editor_palette.careaOverlayColor;
  draft.value.layouts.push({
    name: '',
    color: {
      base_color: base,
      hue_shift: 0,
      saturation_shift: 0,
      lightness_shift: 0,
    },
  });
}

function removeLayout(index: number): void {
  if (!draft.value) {
    return;
  }
  draft.value.layouts.splice(index, 1);
}

function addFlow(): void {
  if (!draft.value) {
    return;
  }
  const base = draft.value.editor_palette.careaOverlayColor;
  draft.value.flows.push({
    name: '',
    color: {
      base_color: base,
      hue_shift: 0,
      saturation_shift: 0,
      lightness_shift: 0,
    },
  });
}

function removeFlow(index: number): void {
  if (!draft.value) {
    return;
  }
  draft.value.flows.splice(index, 1);
}

function copyProject(source: Project): Project {
  const { editor_palette, ...rest } = source;
  const palette = editor_palette ? { ...editor_palette } : { ...DEFAULT_PALETTE };

  // Ensure all block color fields are copied/initialized properly as objects
  for (const item of blockTypeItems) {
    const key = item.key;
    const spec = (palette as any)[key];
    (palette as any)[key] = spec ? {
      base_color: spec.base_color || undefined,
      hue_shift: spec.hue_shift || 0,
      saturation_shift: spec.saturation_shift || 0,
      lightness_shift: spec.lightness_shift || 0,
    } : { hue_shift: 0, saturation_shift: 0, lightness_shift: 0 };
  }

  return {
    ...rest,
    authors: source.authors.map(author => ({ ...author })),
    editor_palette: palette,
    layouts: source.layouts.map(layout => ({
      ...layout,
      color: layout.color ? {
        base_color: layout.color.base_color || undefined,
        hue_shift: layout.color.hue_shift || 0,
        saturation_shift: layout.color.saturation_shift || 0,
        lightness_shift: layout.color.lightness_shift || 0,
      } : { base_color: undefined, hue_shift: 0, saturation_shift: 0, lightness_shift: 0 },
    })),
    flows: source.flows.map(flow => ({
      ...flow,
      color: flow.color ? {
        base_color: flow.color.base_color || undefined,
        hue_shift: flow.color.hue_shift || 0,
        saturation_shift: flow.color.saturation_shift || 0,
        lightness_shift: flow.color.lightness_shift || 0,
      } : { base_color: undefined, hue_shift: 0, saturation_shift: 0, lightness_shift: 0 },
    })),
  };
}

async function loadProject(): Promise<void> {
  // Optimization: use already fetched project from context if available
  if (activeProject.value?.machine_name === props.machineName) {
    project.value = copyProject(activeProject.value);
    draft.value = copyProject(activeProject.value);
    mode.value = 'view';
    return;
  }

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const res = await fetch(`/api/projects/${props.machineName}`);

    if (!res.ok) {
      throw new Error(`Failed to load project: ${res.status}`);
    }

    const data = await res.json() as Project;

    project.value = copyProject(data);
    draft.value = copyProject(data);
    mode.value = 'view';
  } catch (e) {
    console.error('Failed to load project:', e);
    errorMessage.value = 'Could not load project.';
  } finally {
    isLoading.value = false;
  }
}

async function acceptEdit(): Promise<void> {
  if (!draft.value) {
    return;
  }

  isSaving.value = true;
  errorMessage.value = '';
  blockingInfo.value = undefined;

  try {
    const res = await fetch(`/api/projects/${props.machineName}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(draft.value),
    });

    if (res.status === 409) {
      const data = await res.json();
      errorMessage.value = data.error || 'Conflict detected.';
      blockingInfo.value = data.blocking;
      return;
    }

    if (!res.ok) {
      throw new Error(`Failed to save project: ${res.status}`);
    }

    const savedProject = await res.json() as Project;

    // Sync with global state
    activeProject.value = savedProject;

    project.value = copyProject(savedProject);
    draft.value = copyProject(savedProject);
    mode.value = 'view';
  } catch (e) {
    console.error('Failed to accept edit:', e);
    errorMessage.value = 'Could not save project.';
  } finally {
    isSaving.value = false;
  }
}

function cancelEdit(): void {
  if (!project.value) {
    return;
  }

  draft.value = copyProject(project.value);
  mode.value = 'view';
}

onMounted(() => {
  void loadProject();
});

</script>

<style scoped>
.project-detail-page {
  flex: 1 1 auto;
  min-height: 0;
  width: 100%;
  max-width: 90rem;
  box-sizing: border-box;
  margin: 0 auto;
  padding: 2rem 1rem;
  overflow-y: auto;
}

.project-grid {
  display: grid;
  grid-template-columns: 3fr 3fr 5fr 5fr;
  gap: 2.5rem;
  margin-top: 2rem;
}

@media (max-width: 1024px) {
  .project-grid {
    grid-template-columns: 1fr;
  }
}

.grid-column h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);
  font-size: 1.125rem;
}

.page-header {
  margin-bottom: 1.5rem;
}

.page-header h2 {
  margin-top: 0.5rem;
}

.page-actions {
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
}

.back-link {
  color: var(--color-text-muted);
  text-decoration: none;
  font-size: 0.875rem;
}

.back-link:hover {
  text-decoration: underline;
}

.button-link {
  border: 0;
  padding: 0;
  background: transparent;
  cursor: pointer;
  font: inherit;
}

h2 {
  margin-top: 0.5rem;
}

.metadata-list {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 0.5rem 1rem;
  margin-bottom: 1.5rem;
}

.metadata-list dt {
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.metadata-list dd {
  margin: 0;
}

.muted {
  color: var(--color-text-muted);
}

.error-message {
  color: #dc2626;
}

.sub-heading {
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.025em;
  color: var(--color-text-muted);
  margin: 1.5rem 0 0.75rem;
}

.mt-4 {
  margin-top: 1.5rem;
}

.color-field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.color-field label {
  font-size: 0.875rem;
  font-weight: 500;
}

.form-grid {
  display: grid;
  gap: 0.75rem;
}

.form-grid label {
  font-weight: 600;
}

.form-grid input,
.form-grid textarea {
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
}

.form-grid textarea {
  resize: vertical;
}

.authors-list {
  display: grid;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.author-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.author-name-input {
  flex: 2;
}

.author-abbrev-input {
  flex: 1;
}

.flex-grow {
  flex: 1;
}

.remove-author {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted);
  font-size: 1.25rem;
  line-height: 1;
  padding: 0 0.25rem;
}

.remove-author:hover {
  color: var(--color-text);
}

.form-actions {
  margin-top: 0.5rem;
  display: flex;
  gap: 0.5rem;
}

.schema-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  gap: 0.5rem;
}

.schema-list li {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem;
  background: var(--color-surface-raised, #f8fafc);
  border-radius: 0.375rem;
}

.color-swatch {
  width: 1.25rem;
  height: 1.25rem;
  border-radius: 0.25rem;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.schema-edit-list {
  display: grid;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.schema-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.schema-row input[type="text"] {
  flex: 1;
}

.color-picker {
  width: 2.5rem;
  height: 2.5rem;
  padding: 0.25rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  cursor: pointer;
  background: var(--color-surface);
}

.remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted);
  font-size: 1.25rem;
  line-height: 1;
  padding: 0 0.25rem;
}

.remove-btn:hover {
  color: var(--color-text);
}

.blocking-alert {
  margin: 1.5rem 0;
  padding: 1.25rem;
  background: #fef2f2;
  border: 1px solid #fee2e2;
  border-radius: 0.5rem;
  color: #991b1b;
}

.blocking-alert h3 {
  margin-top: 0;
  margin-bottom: 0.5rem;
  font-size: 1rem;
  color: #b91c1c;
  border-bottom: none;
  padding-bottom: 0;
}

.blocking-alert p {
  margin: 0;
  font-size: 0.875rem;
}

.blocking-alert ul {
  margin: 0.75rem 0 0;
  padding-left: 1.25rem;
  font-size: 0.875rem;
}

.blocking-alert li {
  margin-bottom: 0.25rem;
}

.color-details {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  padding: 0.75rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  margin-top: -0.25rem;
  margin-bottom: 0.5rem;
}

.shift-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.shift-group label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
}

.shift-group input[type="range"] {
  width: 100%;
  padding: 0;
}

.shift-value {
  font-size: 0.75rem;
  text-align: right;
  font-family: monospace;
}
</style>