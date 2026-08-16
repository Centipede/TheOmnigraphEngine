<script setup lang="ts">
import { computed } from 'vue';
import type { StructureDb, Section } from '../types';

const props = defineProps<{
  structure: StructureDb;
}>();

interface FlatSection extends Section {
  depth: number;
}

function flatten(sections: Section[], depth = 0): FlatSection[] {
  let result: FlatSection[] = [];
  for (const s of sections) {
    result.push({ ...s, depth });
    if (s.subsections && s.subsections.length > 0) {
      result = result.concat(flatten(s.subsections, depth + 1));
    }
  }
  return result;
}

const flatSections = computed(() => flatten(props.structure.sections));
</script>

<template>
  <div class="section-outline">
    <div class="outline-group">
      <div
          v-for="section in flatSections"
          :key="section.path_id"
          class="outline-item section-item"
          :class="{ 'is-linked': section.is_linked }"
          :style="{ paddingLeft: (section.depth * 0.75 + 0.5) + 'rem' }"
      >
        <span class="kind-label">{{ section.kind }}</span>
        <span class="title">{{ section.title }}</span>
      </div>
    </div>

    <div class="sidebar-lead" v-if="structure.headlines.length > 0">Captions</div>
    <div class="outline-group">
      <div
          v-for="(headline, index) in structure.headlines"
          :key="index"
          class="outline-item headline-item"
          :class="{ 'is-linked': headline.is_linked }"
          style="padding-left: 0.5rem;"
      >
        <span class="page-label">{{ headline.page }}</span>
        <span class="block-id">{{ headline.block_id }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.section-outline {
  display: flex;
  flex-direction: column;
}

.outline-group {
  padding: 0.25rem 0;
}

.outline-item {
  padding: 0.2rem 0.5rem;
  font-size: 0.85rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: default;
}

.outline-item:hover {
  background-color: var(--color-bg-muted, #f1f3f5);
}

.is-linked {
  color: var(--color-text-muted, #6c757d);
  font-style: italic;
}

.kind-label {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  margin-right: 0.5rem;
  color: var(--color-text-muted, #6c757d);
}

.page-label {
  font-weight: 600;
  margin-right: 0.5rem;
}

.block-id {
  color: var(--color-text-muted, #6c757d);
  font-size: 0.8rem;
}

.sidebar-lead {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted, #6c757d);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.6rem 0.75rem 0.4rem;
  border-top: 1px solid var(--color-border, #dee2e6);
  border-bottom: 1px solid var(--color-border, #dee2e6);
  background: var(--color-surface, #fff);
  position: sticky;
  top: 0;
  z-index: 1;
}
</style>