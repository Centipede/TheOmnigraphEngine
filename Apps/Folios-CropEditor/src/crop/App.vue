<template>
  <div class="crop-area">

    <div class="crop-sidebar-pages">
      <div class="sidebar-lead">Pages</div>
      <div class="sidebar-content">Page list</div>
    </div>

    <div class="crop-sidebar-sections">
      <div class="sidebar-lead">Sections</div>
      <div class="sidebar-content">Section list</div>
    </div>

    <div class="crop-workarea">
      <div class="strip-grid">
        <PageStrip
          v-for="page in pages"
          :key="page.index"
          :page="page"
          :edge="edge"
          :thumbBaseUrl="thumbBaseUrl"
          :fraction="VIEW_FRACTION"
        />
      </div>
    </div>

    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <sl-radio-group label="Mode" name="mode" :value="mode"
          @sl-change="mode = ($event.target as HTMLInputElement).value">
          <sl-radio-button value="none">None</sl-radio-button>
          <sl-radio-button value="crop">Crop</sl-radio-button>
        </sl-radio-group>

        <template v-if="mode === 'crop'">
          <br>

          <sl-radio-group label="Tool" name="tool" :value="tool"
            @sl-change="tool = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="singleadjust">Single Adjust</sl-radio-button>
            <sl-radio-button value="wideadjust">Wide Adjust</sl-radio-button>
          </sl-radio-group>

          <template v-if="tool === 'wideadjust'">
            <br>
            <sl-range
              :label="`Width: ${wide_width}`"
              min="1" max="200" step="1" :value="wide_width"
              @sl-input="wide_width = parseInt(($event.target as HTMLInputElement).value)"
            />
          </template>

          <br>

          <sl-radio-group label="Edge" name="edge" size="small" :value="edge"
            @sl-change="edge = ($event.target as HTMLInputElement).value">
            <sl-radio-button value="none">None</sl-radio-button>
            <sl-radio-button value="left">Left</sl-radio-button>
            <sl-radio-button value="top">Top</sl-radio-button>
            <sl-radio-button value="bottom">Bottom</sl-radio-button>
            <sl-radio-button value="right">Right</sl-radio-button>
          </sl-radio-group>
        </template>

      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import PageStrip from './PageStrip.vue';
import type { Page, PageDb } from './types';

const props = defineProps<{ machineName: string; projectName: string }>();

const mode       = ref('none');
const tool       = ref('singleadjust');
const edge       = ref('none');
const wide_width = ref(100);
const pages      = ref<Page[]>([]);

// 25% of the thumbnail shown in strip mode
const VIEW_FRACTION = 0.25;

const thumbBaseUrl = computed(
  () => `/projects/${props.machineName}/pages/thumbs/`
);

onMounted(async () => {
  try {
    const res  = await fetch(`/api/projects/${props.machineName}/pages`);
    const data = (await res.json()) as PageDb;
    pages.value = data.pages;
  } catch (e) {
    console.error('Failed to load pages:', e);
  }
});
</script>

<style>
.crop-area {
  width: 100%;
  display: grid;
  grid-template-columns: 10rem 18rem 1fr 20rem;
  height: calc(100vh - var(--header-height, 0px));
  overflow: hidden;
  font-family: var(--sl-font-sans, sans-serif);
  background: var(--color-bg, #f8f9fa);
  color: var(--color-text, #212529);
}

.crop-area > div {
  border-right: 1px solid var(--color-border, #dee2e6);
  overflow-y: auto;
  min-height: 0;
}
.crop-area > div:last-child { border-right: none; }

.sidebar-lead {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.6rem 0.75rem 0.4rem;
  border-bottom: 1px solid var(--color-border, #dee2e6);
  position: sticky;
  top: 0;
  background: var(--color-surface, #fff);
  z-index: 1;
}

.sidebar-content {
  padding: 0.75rem;
  font-size: 0.875rem;
  color: var(--color-text-muted, #6c757d);
}

.crop-workarea {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

.strip-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 6px;
  align-content: flex-start;
}

.crop-tools {
  padding: 0;
}
</style>
