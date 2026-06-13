<template>
  <div class="crop-area">

    <div class="crop-sidebar-pages">
      <div class="sidebar-lead">Pages</div>
      <p class="placeholder">Page list</p>
    </div>

    <div class="crop-sidebar-sections">
      <div class="sidebar-lead">Sections</div>
      <p class="placeholder">Section list</p>
    </div>

    <div class="crop-workarea">
      <p class="placeholder">Crop editor — {{ projectName }}</p>
    </div>

    <div class="crop-tools">
      <div class="sidebar-lead">Tools</div>
    </div>

  </div>
</template>

<script setup lang="ts">
defineProps<{ machineName: string; projectName: string }>();
</script>

<style>
.crop-area {
  width: 100%;
  display: grid;
  grid-template-columns: 10rem 18rem 1fr 12rem;
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

.placeholder {
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
