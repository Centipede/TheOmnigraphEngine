<template>
  <div class="project-detail-page">
    <p v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </p>

    <p v-if="isLoading" class="muted">
      Loading project…
    </p>

    <template v-else-if="project && draft">
      <template v-if="mode === 'view'">
        <RouterLink to="/projects" class="back-link">
          ← Projects
        </RouterLink>

        <h2>{{ project.name }}</h2>

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
        <button type="button" class="back-link button-link" @click="cancelEdit">
          ← {{ project.name }}
        </button>

        <h2>Metadata</h2>

        <form class="form-grid" @submit.prevent="acceptEdit">
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

          <div class="form-actions">
            <sl-button type="button" variant="default" :disabled="isSaving" @click="cancelEdit">
              Cancel
            </sl-button>

            <sl-button type="submit" variant="primary" :loading="isSaving" :disabled="isSaving">
              Save
            </sl-button>
          </div>
        </form>
      </template>
    </template>

    <p v-else class="muted">
      Project could not be loaded.
    </p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { Project } from "../types/project";

const props = defineProps<{ machineName: string; projectName: string }>();

// ── Mode ─────────────────────────────────────────────────────────────
type Mode = 'view' | 'edit';
const mode = ref<Mode>('view');

const project = ref<Project>();
const draft = ref<Project>();

const isLoading = ref(false);
const isSaving = ref(false);
const errorMessage = ref('');

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

function copyProject(source: Project): Project {
  return {
    ...source,
    authors: source.authors.map(author => ({ ...author })),
  };
}

async function loadProject(): Promise<void> {
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

  try {
    const res = await fetch(`/api/projects/${props.machineName}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(draft.value),
    });

    if (!res.ok) {
      throw new Error(`Failed to save project: ${res.status}`);
    }

    const savedProject = await res.json() as Project;

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
  width: 100%;
  max-width: 48rem;
  box-sizing: border-box;
  margin: 2rem auto;
  padding: 0 1rem;
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
  gap: 0.5rem 1.5rem;
  margin: 1.5rem 0;
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
</style>