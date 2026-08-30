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

      <div v-if="panels" class="panel-toggle-group">
        <sl-icon-button
            name="list"
            label="Toggle page selector"
            :class="{ active: panels['page-list'] }"
            @click="emit('togglePanel', 'page-list')"
        />
        <sl-icon-button
            name="list-nested"
            label="Toggle sections"
            :class="{ active: panels['section-structure'] }"
            @click="emit('togglePanel', 'section-structure')"
        />

        <span class="nav-separator" aria-hidden="true"></span>

        <sl-icon-button
            name="grid"
            label="Toggle page icons"
            :class="{ active: panels['page-strips'] }"
            @click="emit('togglePanel', 'page-strips')"
        />
        <sl-icon-button
            name="file-earmark-image"
            label="Toggle page preview"
            :class="{ active: panels['page-preview'] }"
            @click="emit('togglePanel', 'page-preview')"
        />

        <span class="nav-separator" aria-hidden="true"></span>

        <sl-icon-button
            name="wrench-adjustable"
            label="Toggle tools"
            :class="{ active: panels['tools'] }"
            @click="emit('togglePanel', 'tools')"
        />
        <sl-icon-button
            name="columns-gap"
            label="Toggle hOCR outline"
            :class="{ active: panels['ocr-structure'] }"
            @click="emit('togglePanel', 'ocr-structure')"
        />
      </div>

      <div class="nav-start">
        <RouterLink
            v-if="hasProject"
            :to="{ name: 'project-detail', params: { machineName } }"
            custom
            v-slot="{ navigate, isExactActive }"
        >
          <sl-button
              :variant="isExactActive ? 'primary' : 'text'"
              @click="navigate"
          >
            Overview
          </sl-button>
        </RouterLink>
        <sl-button v-else variant="text" disabled>
          Overview
        </sl-button>

        <RouterLink
            v-if="hasProject"
            :to="{ name: 'ingestor-assemble', params: { machineName, page: currentPageParam } }"
            custom
            v-slot="{ navigate }"
        >
          <sl-button
              :variant="isIngestorRoute ? 'primary' : 'text'"
              @click="navigate"
          >
            Ingestor
          </sl-button>
          <template v-if="isIngestorRoute">
            <!-- Mode selector -->
            <sl-button-group>
              <RouterLink
                  :to="{ name: 'ingestor-assemble', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Assemble
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'ingestor-process', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Process
                </sl-button>
              </RouterLink>
            </sl-button-group>
          </template>
        </RouterLink>
        <sl-button v-else variant="text" disabled>
          Ingestor
        </sl-button>

        <RouterLink
            v-if="hasProject"
            :to="{ name: 'folios-assist', params: { machineName, page: currentPageParam } }"
            custom
            v-slot="{ navigate }"
        >
          <sl-button
              :variant="isFoliosRoute ? 'primary' : 'text'"
              @click="navigate"
          >
            Folios
          </sl-button>
          <template v-if="isFoliosRoute">

            <!-- Mode selector -->
            <sl-button-group>
              <RouterLink
                  :to="{ name: 'folios-assist', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Assist
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'folios-crop', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Crop
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'folios-hint', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Hint
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'folios-recognise', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Recognise
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'folios-edit', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Edit
                </sl-button>
              </RouterLink>
            </sl-button-group>

          </template>
        </RouterLink>
        <sl-button v-else variant="text" disabled>
          Folios
        </sl-button>

        <RouterLink
            v-if="hasProject"
            :to="{ name: 'codex-edit', params: { machineName, page: currentPageParam } }"
            custom
            v-slot="{ navigate }"
        >
          <sl-button
              :variant="isCodexRoute ? 'primary' : 'text'"
              @click="navigate"
          >
            Codex
          </sl-button>
          <template v-if="isCodexRoute">

            <!-- Mode selector -->
            <sl-button-group>
              <RouterLink
                  :to="{ name: 'codex-edit', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Edit
                </sl-button>
              </RouterLink>

              <RouterLink
                  :to="{ name: 'codex-script', params: { machineName, page: currentPageParam } }"
                  custom
                  v-slot="{ navigate, isActive }"
              >
                <sl-button :variant="isActive ? 'primary' : 'text'" @click="navigate" size="small">
                  Script
                </sl-button>
              </RouterLink>
            </sl-button-group>

          </template>
        </RouterLink>
        <sl-button v-else variant="text" disabled>
          Codex
        </sl-button>
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
            <sl-menu-item value="light">
              <sl-icon slot="prefix" name="sun"></sl-icon>
              Light
            </sl-menu-item>
            <sl-menu-item value="system">
              <sl-icon slot="prefix" name="circle-half"></sl-icon>
              System
            </sl-menu-item>
            <sl-menu-item value="dark">
              <sl-icon slot="prefix" name="moon"></sl-icon>
              Dark
            </sl-menu-item>
          </sl-menu>
        </sl-dropdown>
      </div>
    </nav>
  </header>
</template>

<script setup lang="ts">
import {ref, computed, onMounted, onUnmounted} from 'vue';
import {RouterLink, useRoute} from 'vue-router';
import type {PanelVisibility, PanelId} from '../types';

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility | null;
}>();

const emit = defineEmits<{
  togglePanel: [panelId: PanelId];
}>();

const route = useRoute();
const hasProject = computed(() => props.machineName.length > 0);
const currentPageParam = computed(() => route.params.page ? String(route.params.page) : undefined);
const isIngestorRoute = computed(() => {
  return typeof route.name === 'string' && route.name.startsWith('ingestor-');
});
const isFoliosRoute = computed(() => {
  return typeof route.name === 'string' && route.name.startsWith('folios-');
});
const isCodexRoute = computed(() => {
  return typeof route.name === 'string' && route.name.startsWith('codex-');
});

const ICONS: Record<string, string> = {light: 'sun', system: 'circle-half', dark: 'moon'};

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

.nav-separator {
  align-self: stretch;
  width: 1px;
  margin: 0.15rem 0.55rem;
  border-radius: 999px;
  background: linear-gradient(
      to bottom,
      transparent,
      var(--color-border, #dee2e6) 18%,
      var(--color-border, #dee2e6) 82%,
      transparent
  );
}

.nav-start,
.nav-end {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.panel-toggle-group {
  align-self: stretch;
  display: flex;
  align-items: center;
  gap: 0.15rem;
}

sl-icon-button.active::part(base) {
  background: var(--sl-color-primary-200);
}

</style>
