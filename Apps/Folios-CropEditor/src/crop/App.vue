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
      <div class="sidebar-content">Crop editor — {{ projectName }}</div>
    </div>

    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
      <div class="sidebar-content">

        <sl-radio-group label="Mode" name="mode" :value="mode" @sl-change="mode = ($event.target as HTMLInputElement).value">
          <sl-radio-button value="none">None</sl-radio-button>
          <sl-radio-button value="crop">Crop</sl-radio-button>
        </sl-radio-group>

        <br>

        <sl-radio-group label="Tool" name="tool" :value="tool" v-show="mode === 'crop'"  @sl-change="tool = ($event.target as HTMLInputElement).value">
          <sl-radio-button value="singleadjust">Single Adjust</sl-radio-button>
          <sl-radio-button value="wideadjust">Wide Adjust</sl-radio-button>
        </sl-radio-group>

        <br>

        <sl-range :label="`Width: ${wide_width}`" min="1" max="200" step="1" :value="wide_width" v-show="mode === 'crop' && tool === 'wideadjust'" @sl-input="wide_width = parseInt(($event.target as HTMLInputElement).value)"></sl-range>


        <br>

        <sl-radio-group label="Edge" name="edge" size="small" :value="edge" v-show="mode === 'crop'"  @sl-change="edge = ($event.target as HTMLInputElement).value">
          <sl-radio-button value="none">None</sl-radio-button>
          <sl-radio-button value="left">Left</sl-radio-button>
          <sl-radio-button value="top">Top</sl-radio-button>
          <sl-radio-button value="bottom">Bottom</sl-radio-button>
          <sl-radio-button value="right">Right</sl-radio-button>
        </sl-radio-group>

      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
  import {ref} from "vue";

  defineProps<{ machineName: string; projectName: string }>();
  const mode = ref('none')
  const tool = ref('singleadjust')
  const edge = ref('none')
  const wide_width = ref(100)

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

.crop-area > div:last-child {
  border-right: none;
}

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
}

.crop-tools {
  padding: 0;
}
</style>
