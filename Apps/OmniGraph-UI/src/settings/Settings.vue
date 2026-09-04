<template>
  <main class="settings-page">
    <h2>Settings</h2>

    <p v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </p>

    <p v-if="successMessage" class="success-message">
      {{ successMessage }}
    </p>

    <p v-if="isLoading" class="muted">
      Loading settings…
    </p>

    <form class="form-grid" @submit.prevent="saveSettings">

      <h3>API Keys</h3>

      <label for="openai_api_key">
        OpenAI
        <small v-if="settingsStatus.openai_api_key_set">stored — leave blank to keep</small>
      </label>
      <input
          id="openai_api_key"
          v-model="settingsForm.openai_api_key"
          type="text"
          name="openai_api_key"
          :placeholder="settingsStatus.openai_api_key_set ? '••••••••' : 'sk-...'"
          autocomplete="off"
      >

      <label for="perplexity_api_key">
        Perplexity
        <small v-if="settingsStatus.perplexity_api_key_set">stored — leave blank to keep</small>
      </label>
      <input
          id="perplexity_api_key"
          v-model="settingsForm.perplexity_api_key"
          type="text"
          name="perplexity_api_key"
          :placeholder="settingsStatus.perplexity_api_key_set ? '••••••••' : 'pplx-...'"
          autocomplete="off"
      >

      <h3 class="section-heading">Tesseract OCR</h3>

      <label>Priority servers</label>
      <div class="ocr-servers">
        <div class="ocr-server-row">
          <span class="server-label">1st</span>
          <span class="status-dot" :class="settingsStatus.ocr_server_1_status" :title="statusLabel(settingsStatus.ocr_server_1_status)"></span>
          <input
              v-model="ocrForm.server_1_host"
              type="text"
              placeholder="hostname or IP"
              class="host-input"
              autocomplete="off"
          >
          <span class="port-sep">:</span>
          <input
              v-model="ocrForm.server_1_port"
              type="number"
              placeholder="3000"
              class="port-input"
              min="1"
              max="65535"
          >
        </div>
        <div class="ocr-server-row">
          <span class="server-label">2nd</span>
          <span class="status-dot" :class="settingsStatus.ocr_server_2_status" :title="statusLabel(settingsStatus.ocr_server_2_status)"></span>
          <input
              v-model="ocrForm.server_2_host"
              type="text"
              placeholder="hostname or IP"
              class="host-input"
              autocomplete="off"
          >
          <span class="port-sep">:</span>
          <input
              v-model="ocrForm.server_2_port"
              type="number"
              placeholder="3000"
              class="port-input"
              min="1"
              max="65535"
          >
        </div>
      </div>

      <label>Local command format</label>
      <div class="radio-group">
        <label class="radio-label">
          <input type="radio" v-model="ocrForm.command_format" value="native">
          Native tesseract
        </label>
        <label class="radio-label">
          <input type="radio" v-model="ocrForm.command_format" value="docker">
          Docker
        </label>
      </div>

      <div class="form-actions">
        <sl-button
            type="submit"
            variant="primary"
            :loading="isSaving"
            :disabled="isSaving"
        >
          Save
        </sl-button>
      </div>

    </form>
  </main>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref} from 'vue';
import { useHead } from '@unhead/vue';
import type {OcrCommandFormat, OcrServerStatus, ServiceStatus, SettingsStatus} from '../types/settings';

useHead({
  title: 'Settings',
});

const SETTINGS_ENDPOINT = '/api/settings';

const isLoading = ref(false);
const isSaving = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

const settingsStatus = reactive<SettingsStatus>({
  openai_api_key_set: false,
  perplexity_api_key_set: false,
  ocr_server_1: null,
  ocr_server_2: null,
  ocr_command_format: 'native',
  ocr_server_1_status: 'unconfigured',
  ocr_server_2_status: 'unconfigured',
});

const settingsForm = reactive({
  openai_api_key: '',
  perplexity_api_key: '',
});

const ocrForm = reactive({
  server_1_host: '',
  server_1_port: '',
  server_2_host: '',
  server_2_port: '',
  command_format: 'native' as OcrCommandFormat,
});

function applyStatus(status: SettingsStatus): void {
  settingsStatus.openai_api_key_set = status.openai_api_key_set;
  settingsStatus.perplexity_api_key_set = status.perplexity_api_key_set;
  settingsStatus.ocr_server_1 = status.ocr_server_1;
  settingsStatus.ocr_server_2 = status.ocr_server_2;
  settingsStatus.ocr_command_format = status.ocr_command_format;
  settingsStatus.ocr_server_1_status = status.ocr_server_1_status;
  settingsStatus.ocr_server_2_status = status.ocr_server_2_status;

  ocrForm.server_1_host = status.ocr_server_1?.host ?? '';
  ocrForm.server_1_port = status.ocr_server_1?.port?.toString() ?? '';
  ocrForm.server_2_host = status.ocr_server_2?.host ?? '';
  ocrForm.server_2_port = status.ocr_server_2?.port?.toString() ?? '';
  ocrForm.command_format = status.ocr_command_format;
}

function statusLabel(s: OcrServerStatus): string {
  return s === 'online' ? 'Online' : s === 'offline' ? 'Offline' : 'Not configured';
}

async function fetchServiceStatus(): Promise<void> {
  try {
    const resp = await fetch(`/api/settings/service/status`);
    if (resp.ok) {
      const data = await resp.json() as ServiceStatus;
      settingsStatus.ocr_server_1_status = data.server_1;
      settingsStatus.ocr_server_2_status = data.server_2;
    }
  } catch (e) {
    console.error('Failed to fetch hOCR status:', e);
  }
}

async function loadSettingsStatus(): Promise<void> {
  isLoading.value = true;
  errorMessage.value = '';

  try {
    const response = await fetch(SETTINGS_ENDPOINT, { headers: { Accept: 'application/json' } });
    if (!response.ok) throw new Error(`${response.status}`);
    applyStatus(await response.json() as SettingsStatus);
  } catch (error) {
    console.error(error);
    errorMessage.value = 'Could not load settings.';
  } finally {
    isLoading.value = false;
  }
}

async function saveSettings(): Promise<void> {
  isSaving.value = true;
  errorMessage.value = '';
  successMessage.value = '';

  const s1Host = ocrForm.server_1_host.trim();
  const s2Host = ocrForm.server_2_host.trim();

  const body = {
    openai_api_key: settingsForm.openai_api_key,
    perplexity_api_key: settingsForm.perplexity_api_key,
    ocr: {
      server_1: s1Host ? { host: s1Host, port: parseInt(ocrForm.server_1_port) || 3000 } : null,
      server_2: s2Host ? { host: s2Host, port: parseInt(ocrForm.server_2_port) || 3000 } : null,
      command_format: ocrForm.command_format,
    },
  };

  try {
    const response = await fetch(SETTINGS_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
      body: JSON.stringify(body),
    });
    if (!response.ok) throw new Error(`${response.status}`);

    applyStatus(await response.json() as SettingsStatus);
    settingsForm.openai_api_key = '';
    settingsForm.perplexity_api_key = '';
    successMessage.value = 'Settings saved.';
  } catch (error) {
    console.error(error);
    errorMessage.value = 'Could not save settings.';
  } finally {
    isSaving.value = false;
  }
}

let fetchServiceInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  void loadSettingsStatus();
  await fetchServiceStatus();
  fetchServiceInterval = setInterval(() => {
    void fetchServiceStatus();
  }, 1_000);
});

onUnmounted(() => {
  if (fetchServiceInterval !== null) clearInterval(fetchServiceInterval);
});

</script>

<style scoped>
.settings-page {
  width: 100%;
  max-width: 48rem;
  box-sizing: border-box;
  margin: 2rem auto;
  padding: 0 1rem;
}

.form-grid {
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

/* OCR server rows */
.ocr-servers {
  display: grid;
  gap: 0.5rem;
}

.ocr-server-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.server-label {
  font-weight: 600;
  min-width: 2rem;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.host-input {
  flex: 1;
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
}

.port-sep {
  color: var(--color-text-muted);
}

.port-input {
  width: 5rem;
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
}

/* Status dot */
.status-dot {
  display: inline-block;
  width: 0.625rem;
  height: 0.625rem;
  border-radius: 50%;
  flex-shrink: 0;
}
.status-dot.online      { background: #16a34a; }
.status-dot.offline     { background: #dc2626; }
.status-dot.unconfigured { background: #9ca3af; }

/* Radio group */
.radio-group {
  display: flex;
  gap: 1.5rem;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-weight: 400;
  cursor: pointer;
}

.form-actions {
  margin-top: 0.5rem;
}

.error-message {
  color: #dc2626;
}

.success-message {
  color: #16a34a;
}
</style>
