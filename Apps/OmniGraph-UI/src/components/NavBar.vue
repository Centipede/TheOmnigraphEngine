<template>
  <header class="vue-header">
    <nav>
      <div class="nav-start">
        <RouterLink to="/projects" class="nav-link">
          <img src="/icons/omnigraph-small.png" alt="Home" class="nav-brand-icon" />
        </RouterLink>

        <template v-if="hasProject">
          <RouterLink
              :to="{ name: 'project-detail', params: { machineName } }"
              class="nav-link"
          >
            <img src="/icons/overview.png" alt="Overview" class="nav-icon" />
            <span class="nav-label">Overview</span>
          </RouterLink>

          <div class="nav-group">
            <RouterLink
                :to="{ name: 'ingestor-assemble', params: { machineName, page: currentPageParam } }"
                class="nav-link"
                :class="{ active: isIngestorRoute }"
            >
              <img src="/icons/ingestor.png" alt="Ingestor" class="nav-icon" />
              <span class="nav-label">Ingestor</span>
            </RouterLink>
            <Transition name="subsection">
              <div v-if="isIngestorRoute" class="subpage-links">
                <RouterLink
                    :to="{ name: 'ingestor-assemble', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Assemble
                </RouterLink>
                <RouterLink
                    :to="{ name: 'ingestor-process', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Process
                </RouterLink>
              </div>
            </Transition>
          </div>

          <div class="nav-group">
            <RouterLink
                :to="{ name: 'folios-crop', params: { machineName, page: currentPageParam } }"
                class="nav-link"
                :class="{ active: isFoliosRoute }"
            >
              <img src="/icons/folios.png" alt="Folios" class="nav-icon" />
              <span class="nav-label">Folios</span>
            </RouterLink>
            <Transition name="subsection">
              <div v-if="isFoliosRoute" class="subpage-links">
                <RouterLink
                    :to="{ name: 'folios-crop', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Crop
                </RouterLink>
                <RouterLink
                    :to="{ name: 'folios-hint', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Hint
                </RouterLink>
                <RouterLink
                    :to="{ name: 'folios-recognise', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Recognise
                </RouterLink>
                <RouterLink
                    :to="{ name: 'folios-assist', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Assist
                </RouterLink>
                <RouterLink
                    :to="{ name: 'folios-edit', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Edit
                </RouterLink>
              </div>
            </Transition>
          </div>

          <div class="nav-group">
            <RouterLink
                :to="{ name: 'codex-edit', params: { machineName, page: currentPageParam } }"
                class="nav-link"
                :class="{ active: isCodexRoute }"
            >
              <img src="/icons/codex.png" alt="Codex" class="nav-icon" />
              <span class="nav-label">Codex</span>
            </RouterLink>
            <Transition name="subsection">
              <div v-if="isCodexRoute" class="subpage-links">
                <RouterLink
                    :to="{ name: 'codex-edit', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Edit
                </RouterLink>
                <RouterLink
                    :to="{ name: 'codex-script', params: { machineName, page: currentPageParam } }"
                    class="subpage-link"
                >
                  Script
                </RouterLink>
              </div>
            </Transition>
          </div>
        </template>

        <template v-else>
          <div class="nav-link disabled">
            <img src="/icons/overview.png" alt="Overview" class="nav-icon" />
            <span class="nav-label">Overview</span>
          </div>
          <div class="nav-link disabled">
            <img src="/icons/ingestor.png" alt="Ingestor" class="nav-icon" />
            <span class="nav-label">Ingestor</span>
          </div>
          <div class="nav-link disabled">
            <img src="/icons/folios.png" alt="Folios" class="nav-icon" />
            <span class="nav-label">Folios</span>
          </div>
          <div class="nav-link disabled">
            <img src="/icons/codex.png" alt="Codex" class="nav-icon" />
            <span class="nav-label">Codex</span>
          </div>
        </template>
      </div>

      <div class="nav-mid-title-portal"></div>

      <div class="nav-end-tools-portal">
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
      </div>

      <div class="nav-end">
        <RouterLink to="/settings" class="nav-link">
          <sl-icon name="gear" label="Settings"></sl-icon>
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
  padding: 0 1rem;
  height: 100%;
}

.nav-start,
.nav-end {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.nav-group {
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.nav-link {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 0.2rem 0.4rem;
  gap: 2px;
  border-radius: 6px;
  color: var(--color-text-muted);
  text-decoration: none;
  transition: all 0.2s ease;
  cursor: pointer;
  border: 1px solid transparent;
}

.nav-link:hover {
  background-color: var(--color-bg-muted);
  color: var(--color-text);
}

.nav-link.active,
.nav-link.router-link-active {
  background-color: var(--color-bg-selected);
  color: var(--color-accent);
}

.nav-link.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

.nav-icon {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.nav-label {
  font-size: 10px;
  text-transform: uppercase;
  font-weight: 600;
  line-height: 1;
}

.subpage-links {
  display: flex;
  align-items: center;
  gap: 0.1rem;
  padding-left: 0.4rem;
  margin-left: 0.1rem;
  border-left: 1px solid var(--color-border);
}

.subpage-link {
  font-size: 13px;
  color: var(--color-text-muted);
  text-decoration: none;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.subpage-link:hover {
  background-color: var(--color-bg-muted);
  color: var(--color-text);
}

.subpage-link.router-link-active {
  color: var(--color-accent);
  font-weight: 600;
}

.nav-brand-icon {
  width: 36px;
  height: 36px;
  object-fit: contain;
}

.nav-mid-title-portal {
  flex: 1;
  display: flex;
  justify-content: center;
  min-width: 0;
}

.nav-end-tools-portal {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-right: 1rem;
}

.panel-toggle-group {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  background: var(--color-bg-muted);
  padding: 2px;
  border-radius: 8px;
}

.nav-separator {
  width: 1px;
  height: 20px;
  background: var(--color-border);
  margin: 0 4px;
}

sl-icon-button::part(base) {
  transition: all 0.2s ease;
}

sl-icon-button.active::part(base) {
  background: var(--color-bg-selected);
  color: var(--color-accent);
}

.nav-link sl-icon {
  font-size: 20px;
}

/* Animation for subsections expansion */
.subsection-enter-active,
.subsection-leave-active {
  transition: all 0.3s ease-in-out;
  overflow: hidden;
  white-space: nowrap;
}

.subsection-enter-from,
.subsection-leave-to {
  max-width: 0;
  opacity: 0;
  margin-left: -0.5rem; /* Offsets the flex gap during transition */
}

.subsection-enter-to,
.subsection-leave-from {
  max-width: 500px; /* Large enough to accommodate all buttons */
  opacity: 1;
  margin-left: 0;
}
</style>
