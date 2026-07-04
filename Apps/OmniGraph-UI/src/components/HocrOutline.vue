<template>
  <div v-if="hocrPage" class="hocr-outline">
    <div v-for="carea in hocrPage.careas" :key="carea.id" class="hocr-carea">

      <div class="hocr-row hocr-carea-row" @click="toggleCarea(carea.id)">
        <span class="hocr-badge hocr-badge-c">C</span>
        <span class="hocr-id" :title="carea.id">{{ carea.id }}</span>
        <span class="hocr-preview">{{ careaPreview(carea) }}</span>
        <span class="hocr-toggle">{{ collapsed.has(carea.id) ? '▸' : '▾' }}</span>
      </div>

      <div v-if="!collapsed.has(carea.id)" class="hocr-pars">
        <div
            v-for="par in carea.pars"
            :key="par.id"
            class="hocr-row hocr-par-row"
        >
          <span class="hocr-badge hocr-badge-p">P</span>
          <span class="hocr-id" :title="par.id">{{ par.id }}</span>
          <span v-if="par.lang" class="hocr-lang">{{ par.lang }}</span>
          <span class="hocr-preview">{{ parPreview(par) }}</span>
        </div>
      </div>

    </div>
  </div>

  <div v-else class="hocr-empty">
    No hOCR loaded for this page
  </div>
</template>

<script setup lang="ts">
import { inject, reactive } from 'vue';
import type { Ref } from 'vue';
import type { HocrCarea, HocrLine, HocrPage, HocrPar } from '../types/hocr';

const hocrPage = inject<Ref<HocrPage | null>>('hocrPage');

const collapsed = reactive(new Set<string>());

function toggleCarea(id: string) {
  if (collapsed.has(id)) collapsed.delete(id);
  else collapsed.add(id);
}

function lineText(line: HocrLine): string {
  return line.words.map(w => w.text).join(' ');
}

function parPreview(par: HocrPar, maxLen = 80): string {
  const text = par.lines.map(lineText).join(' ');
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
}

function careaPreview(carea: HocrCarea, maxLen = 80): string {
  const text = carea.pars.flatMap(p => p.lines).map(lineText).join(' ');
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
}
</script>

<style scoped>
.hocr-outline {
  font-size: 0.72rem;
  font-family: ui-monospace, monospace;
}

.hocr-row {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  padding: 0.18rem 0.5rem;
  user-select: none;
}

.hocr-row:hover {
  background: var(--color-bg-muted, #f1f3f5);
}

.hocr-carea-row {
  cursor: pointer;
}

.hocr-par-row {
  cursor: default;
}

.hocr-pars {
  padding-left: 1.25rem;
}

.hocr-badge {
  flex-shrink: 0;
  display: inline-block;
  width: 1.3em;
  height: 1.3em;
  line-height: 1.3em;
  text-align: center;
  border-radius: 2px;
  font-size: 0.65rem;
  font-weight: 700;
}

.hocr-badge-c {
  background: #f97316;
  color: #fff;
}

.hocr-badge-p {
  background: #a855f7;
  color: #fff;
}

.hocr-id {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  max-width: 7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hocr-lang {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-style: italic;
}

.hocr-preview {
  flex: 1;
  min-width: 0;
  color: var(--color-text-muted, #6c757d);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hocr-toggle {
  flex-shrink: 0;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.55rem;
}

.hocr-empty {
  padding: 1rem 0.75rem;
  color: var(--color-text-dimmed, #a2acb6);
  font-size: 0.8rem;
  font-style: italic;
}
</style>
