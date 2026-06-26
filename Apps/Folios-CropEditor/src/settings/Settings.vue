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

    <h3>API Keys</h3>

    <form class="form-grid" @submit.prevent="saveSettings">
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
import { onMounted, reactive, ref } from 'vue';
import type { SettingsForm, SettingsStatus } from '../types/settings';

const SETTINGS_ENDPOINT = '/api/settings';

const isLoading = ref(false);
const isSaving = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

const settingsForm = reactive<SettingsForm>({
  openai_api_key: '',
  perplexity_api_key: '',
});

const settingsStatus = reactive<SettingsStatus>({
  openai_api_key_set: false,
  perplexity_api_key_set: false,
});

async function loadSettingsStatus(): Promise<void> {
  isLoading.value = true;
  errorMessage.value = '';

  try {
    const response = await fetch(SETTINGS_ENDPOINT, {
      headers: {
        Accept: 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to load settings status: ${response.status}`);
    }

    const status = await response.json() as SettingsStatus;

    settingsStatus.openai_api_key_set = status.openai_api_key_set;
    settingsStatus.perplexity_api_key_set = status.perplexity_api_key_set;
    console.log('Settings status:', status);
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

  const formToSubmit: SettingsForm = {
    openai_api_key: settingsForm.openai_api_key,
    perplexity_api_key: settingsForm.perplexity_api_key,
  };

  try {
    const response = await fetch(SETTINGS_ENDPOINT, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
      body: JSON.stringify(formToSubmit),
    });

    if (!response.ok) {
      throw new Error(`Failed to save settings: ${response.status}`);
    }

    const status = await response.json() as SettingsStatus;

    settingsStatus.openai_api_key_set = status.openai_api_key_set;
    settingsStatus.perplexity_api_key_set = status.perplexity_api_key_set;

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

onMounted(() => {
  void loadSettingsStatus();
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

.form-grid label {
  display: grid;
  gap: 0.25rem;
  font-weight: 600;
}

.form-grid small {
  font-weight: 400;
  color: var(--color-text-muted);
}

.form-grid input {
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
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