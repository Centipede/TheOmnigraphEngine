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

    <template #tools="{ currentPageIndex, selectionInfo, selectedPages }">

      <!-- Rename tools -->

      <div class="form-actions">
        <sl-button v-if="selectionInfo!==null" @click="openRenameForm()">Rename</sl-button>
      </div>

      <template v-if="showRenameForm">
        <form
            class="form-grid"
            @submit.prevent="performRename(selectedPages)"
        >
          <sl-radio-group
              name="scheme"
              label="Scheme"
              help-text="Naming method"
              :value="renameForm.scheme"
              @sl-change="renameForm.scheme = ($event.target as HTMLInputElement).value as RenameScheme"
          >
            <sl-radio-button value="T">Text</sl-radio-button>
            <sl-radio-button value="1">1,2,3,...</sl-radio-button>
            <sl-radio-button value="i">i,ii,iii,iv,...</sl-radio-button>
            <sl-radio-button value="I">I,II,III,IV,...</sl-radio-button>
          </sl-radio-group>

          <sl-input
              name="page_name"
              :placeholder="renamePlaceholder"
              clearable
              :pattern="renamePattern"
              :value="renameForm.pageName"
              @sl-input="renameForm.pageName = ($event.target as HTMLInputElement).value"
          ></sl-input>

          <div class="form-actions">
            <sl-button
                type="submit"
                variant="primary"
            >
              Rename pages
            </sl-button>

            <sl-button type="button" @click="showRenameForm = false">
              Cancel
            </sl-button>
          </div>
        </form>
      </template>


      <!-- Append / Insert / Rename tools -->

      <div class="form-actions">
        <sl-button @click="openAppendForm()">Append</sl-button>
        <sl-button v-if="currentPageIndex!==null" @click="openInsertBeforeForm(currentPageIndex)">Insert before
          {{ currentPageIndex }}
        </sl-button>
        <sl-button v-if="currentPageIndex!==null" @click="openInsertAfterForm(currentPageIndex)">Insert after
          {{ currentPageIndex }}
        </sl-button>
        <sl-button v-if="selectionInfo!==null" @click="removePages(selectedPages)">Remove
          {{ selectionInfo.count }} pages
        </sl-button>
      </div>

      <template v-if="showUploadForm">
        <h3>
          {{
            uploadMode === 'append'
                ? 'Append images'
                : uploadMode === 'insert-before'
                    ? `Insert images before page ${targetPageIndex}`
                    : `Insert images after page ${targetPageIndex}`
          }}
        </h3>

        <form
            ref="uploadFormRef"
            enctype="multipart/form-data"
            class="form-grid"
            @submit.prevent="performUpload"
        >
          <label for="images">
            Image files
            <small>jpg, png, tif, webp</small>
          </label>

          <input
              type="file"
              id="images"
              name="images"
              multiple
              accept=".jpg,.jpeg,.png,.tif,.tiff,.webp"
          >

          <p v-if="uploadError" class="form-error">{{ uploadError }}</p>

          <div class="form-actions">
            <sl-button
                type="submit"
                variant="primary"
                :loading="isUploading"
                :disabled="isUploading"
            >
              Upload
            </sl-button>

            <sl-button
                type="button"
                :disabled="isUploading"
                @click="cancelUpload"
            >
              Cancel
            </sl-button>
          </div>
        </form>
      </template>


    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import {reactive, ref, onMounted, onUnmounted, computed, inject} from 'vue';
import type {Page} from "../types";
import PageWorkspace from "../components/PageWorkspace.vue";
import { usePanelVisibilityContext } from '../composables/usePanelVisibility';
import { usePersistentPanels } from '../composables/usePersistentPanels';
import { provideHocrContext } from '../composables/useHocr';

const props = defineProps<{
  machineName: string;
  projectName: string;
  initialPageStem?: string;
}>();

provideHocrContext();

const panels = usePersistentPanels('panels.ingestor', {
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

onMounted(() => setActivePanels(panels));
onUnmounted(() => setActivePanels(null));

const workspaceRef = ref<InstanceType<typeof PageWorkspace> | null>(null);

// ── Rename ───────────────────────────────────────────────────────────

type RenameScheme = 'T' | '1' | 'i' | 'I';

type RenameForm = {
  pageName: string;
  scheme: RenameScheme;
}

const showRenameForm = ref(false);
const renameForm = reactive<RenameForm>({
  pageName: '',
  scheme: '1' as RenameScheme,
});

const renamePattern = computed(() => {
  if(renameForm.scheme === 'T') return '.+';
  else return '[0-9ivxlcdmIVXLCDM]+';
})

const renamePlaceholder = computed(() => {
  if(renameForm.scheme === 'T') return 'E.g.: front, back';
  else if(renameForm.scheme === 'i') return 'E.g.: i, ivmmc';
  else if(renameForm.scheme === 'I') return 'E.g.: II, XIII';
  else return 'E.g.: 1, 13, 100';
})

function openRenameForm() {
  showRenameForm.value = true;
}

async function performRename(selectedPages: Page[]): Promise<void> {
  const renamed = rename(selectedPages, renameForm.pageName, renameForm.scheme);

  if (!renamed) {
    return;
  }

  try {
    await workspaceRef.value?.savePageDb();
    showRenameForm.value = false;
  } catch (e) {
    console.error(e);
  }
}

function renameToNumbers(filteredPages: Page[], start: number) {
  if(filteredPages.length === 0) return [];
  const indexOffset = filteredPages[0].index;
  return filteredPages.map(page => page.index - indexOffset + start)
}

function rename(filteredPages: Page[], page_name: string, scheme: 'T' | '1' | 'i' | 'I'): boolean {
  if(filteredPages.length === 0) return false;

  if (scheme === 'i' || scheme === 'I') {
    let start = parseRoman(page_name);

    if (start === null) {
      start = parseInt(page_name, 10);
      if (Number.isNaN(start)) {
        return false;
      }
    }

    const names = renameToNumbers(filteredPages, start);
    for (let i = 0; i < filteredPages.length; i++) {
      const name = toRoman(names[i]);
      filteredPages[i].name = scheme === 'I' ? name.toUpperCase() : name.toLowerCase();
    }
  }
  else if (scheme === '1') {
    const start = parseInt(page_name, 10);

    if (Number.isNaN(start)) {
      return false;
    }

    const names = renameToNumbers(filteredPages, start);
    for (let i = 0; i < filteredPages.length; i++) {
      filteredPages[i].name = names[i].toString();
    }
  }
  else if (scheme === 'T') {
    filteredPages.forEach(page => {page.name = page_name})
  }

  return true;
}

const romanValues: Record<string, number> = {
  I: 1,
  V: 5,
  X: 10,
  L: 50,
  C: 100,
  D: 500,
  M: 1000,
};

const valuesRoman: readonly [number, string][] = [
  [1000, 'M'],
  [900, 'CM'],
  [500, 'D'],
  [400, 'CD'],
  [100, 'C'],
  [90, 'XC'],
  [50, 'L'],
  [40, 'XL'],
  [10, 'X'],
  [9, 'IX'],
  [5, 'V'],
  [4, 'IV'],
  [1, 'I'],
];

function parseRoman(s: string): number | null {
  const upper = s.toUpperCase();

  let total = 0;
  let prev = 0;

  for (let i = upper.length - 1; i >= 0; i--) {
    const ch = upper[i];
    const val = romanValues[ch];

    if (val === undefined) {
      return null;
    }

    total += val < prev ? -val : val;
    prev = val;
  }

  return total > 0 ? total : null;
}


function toRoman(n: number): string {
  let out = '';

  for (const [value, symbol] of valuesRoman) {
    while (n >= value) {
      out += symbol;
      n -= value;
    }
  }

  return out;
}


// ── Append/Insert/Remove ─────────────────────────────────────────────

const uploadFormRef = ref<HTMLFormElement | null>(null);
const showUploadForm = ref(false);
const isUploading = ref(false);
const uploadError = ref('');
type UploadMode = 'append' | 'insert-before' | 'insert-after';
const uploadMode = ref<UploadMode>('append');
const targetPageIndex = ref<number | null>(null);

function openAppendForm() {
  uploadMode.value = 'append';
  showUploadForm.value = true;
  targetPageIndex.value = null;
}

function openInsertBeforeForm(index: number) {
  uploadMode.value = 'insert-before';
  showUploadForm.value = true;
  targetPageIndex.value = index;
}

function openInsertAfterForm(index: number) {
  uploadMode.value = 'insert-after';
  showUploadForm.value = true;
  targetPageIndex.value = index;
}

function cancelUpload() {
  uploadMode.value = 'append';
  showUploadForm.value = false;
  targetPageIndex.value = null;
}

async function performUpload(): Promise<void> {
  if (!uploadFormRef.value) return;

  const formData = new FormData(uploadFormRef.value);
  const files = formData.getAll('images');

  if (!files.length || files.every(file => !(file instanceof File) || !file.name)) {
    uploadError.value = 'Choose at least one image file.';
    return;
  }

  isUploading.value = true;
  uploadError.value = '';

  var url = `/api/projects/${props.machineName}/pages/append`;
  if (uploadMode.value === 'insert-before') {
    url = `/api/projects/${props.machineName}/pages/insert?before=${targetPageIndex.value}`;
  }
  if (uploadMode.value === 'insert-after') {
    url = `/api/projects/${props.machineName}/pages/insert?after=${targetPageIndex.value}`;
  }

  try {
    const resp = await fetch(url, {
      method: 'POST',
      body: formData,
    });

    if (!resp.ok) {
      const text = await resp.text();
      let errorMsg = text;
      try {
        const json = JSON.parse(text);
        if (json.error) errorMsg = json.error;
      } catch (e) {
        // Not JSON
      }
      const finalMsg = errorMsg || `Upload failed: ${resp.statusText}`;
      uploadError.value = finalMsg;
      showError?.(finalMsg);
      return;
    }

    uploadFormRef.value.reset();
    showUploadForm.value = false;

    resp.json().then(data => {
      workspaceRef.value?.setPageDb(data);
    })

  } catch (e) {
    console.error(e);
    uploadError.value = 'Network error.';
  } finally {
    isUploading.value = false;
  }
}

async function removePages(pages: Page[]) {
  if (!pages.length) return;
  if (!window.confirm(`Remove ${pages.length} page(s)?`)) return;

  const pageIndices = pages.map(p => p.index);
  const url = `/api/projects/${props.machineName}/pages/remove`;
  try {
    const resp = await fetch(url, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({indices: pageIndices}),
    })

    if (!resp.ok) {
      const text = await resp.text();
      let errorMsg = text;
      try {
        const json = JSON.parse(text);
        if (json.error) errorMsg = json.error;
      } catch (e) {
        // Not JSON
      }
      const finalMsg = errorMsg || `Failed to remove pages: ${resp.statusText}`;
      console.error(finalMsg);
      showError?.(finalMsg);
      return;
    }

    resp.json().then(data => {
      workspaceRef.value?.setPageDb(data);
    })
  }
  catch(e) {
    console.error(e)
  }
}

</script>

<style scoped>
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

