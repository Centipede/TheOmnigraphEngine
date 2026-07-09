<template>
  <PageWorkspace
      ref="workspaceRef"
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
  >

    <template #tools="{ currentPageIndex, selectionInfo, selectedPages }">

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
import PageWorkspace from "../components/PageWorkspace.vue";
import type {Page, PanelVisibility} from "../types";
import {ref} from 'vue';


const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

const workspaceRef = ref<InstanceType<typeof PageWorkspace> | null>(null);

const uploadFormRef = ref<HTMLFormElement | null>(null);
const isUploading = ref(false);
const uploadError = ref('');

const showUploadForm = ref(false);
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
      const err = await resp.json().catch(() => null) as { error?: string } | null;
      uploadError.value = err?.error ?? 'Upload failed.';
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
      const err = await resp.json().catch(() => null) as { error?: string } | null;
      console.error(err?.error ?? 'Failed to remove pages');
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

</style>

