<template>
  <header class="vue-header">
    <nav>
      <RouterLink to="/projects" custom v-slot="{ navigate, isExactActive }">
        <sl-button
            class="nav-brand"
            :variant="isExactActive ? 'primary' : 'text'"
            @click="navigate"
        >
          Home
        </sl-button>
      </RouterLink>

      <div class="nav-start">
        <RouterLink
            :to="`/projects/${machineName}`"
            custom
            v-slot="{ navigate, isExactActive }"
        >
          <sl-button
              :variant="isExactActive ? 'primary' : 'text'"
              :disabled="!machineName"
              @click="navigate"
          >
            Overview
          </sl-button>
        </RouterLink>

        <RouterLink
            :to="`/projects/${machineName}/ingestor`"
            custom
            v-slot="{ navigate, isActive }"
        >
          <sl-button
              :variant="isActive ? 'primary' : 'text'"
              :disabled="!machineName"
              @click="navigate"
          >
            Ingestor
          </sl-button>
        </RouterLink>


        <RouterLink
            :to="`/projects/${machineName}/folios`"
            custom
            v-slot="{ navigate, isActive }"
        >
          <sl-button
              :variant="isActive ? 'primary' : 'text'"
              :disabled="!machineName"
              @click="navigate"
          >
            Folios
          </sl-button>
        </RouterLink>
      </div>

      <div class="nav-end">
        <RouterLink to="/settings" custom v-slot="{ navigate, isActive }">
          <sl-button
              :variant="isActive ? 'primary' : 'text'"
              @click="navigate"
          >
            Settings
          </sl-button>
        </RouterLink>

        <sl-dropdown @sl-select="(e: Event) => setTheme((e as CustomEvent).detail.item.value)">
          <sl-icon-button slot="trigger" :name="themeIcon" label="Theme"></sl-icon-button>
          <sl-menu>
            <sl-menu-item value="light"><sl-icon slot="prefix" name="sun"></sl-icon>Light</sl-menu-item>
            <sl-menu-item value="system"><sl-icon slot="prefix" name="circle-half"></sl-icon>System</sl-menu-item>
            <sl-menu-item value="dark"><sl-icon slot="prefix" name="moon"></sl-icon>Dark</sl-menu-item>
          </sl-menu>
        </sl-dropdown>
      </div>
    </nav>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { RouterLink } from 'vue-router';

defineProps<{ machineName: string; projectName: string }>();

const ICONS: Record<string, string> = { light: 'sun', system: 'circle-half', dark: 'moon' };

const themeMode = ref(localStorage.getItem('theme') || 'system');
const themeIcon = computed(() => ICONS[themeMode.value] ?? 'circle-half');

function setTheme(mode: string) {
  const dark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
  document.documentElement.classList.toggle('sl-theme-dark', dark);
  themeMode.value = mode;
  localStorage.setItem('theme', mode);
}

function onSystemThemeChange() {
  if (themeMode.value === 'system') setTheme('system');
}

const mql = window.matchMedia('(prefers-color-scheme: dark)');

onMounted(() => {
  setTheme(themeMode.value);
  mql.addEventListener('change', onSystemThemeChange);
});

onUnmounted(() => {
  mql.removeEventListener('change', onSystemThemeChange);
});
</script>

<style scoped>
.vue-header {
  height: 56px;
  background: var(--color-surface, #ffffff);
  border-bottom: 1px solid var(--color-border, #dee2e6);
  flex-shrink: 0;
}

nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  height: 100%;
}

.nav-brand {
  font-weight: 600;
}

.nav-start,
.nav-end {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
</style>
