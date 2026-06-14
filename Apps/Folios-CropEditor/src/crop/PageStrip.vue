<template>
  <img
    :src="src"
    :width="w"
    :height="h"
    :alt="label"
    :title="label"
    class="page-strip"
    :style="{
      objectFit: 'none',
      objectPosition: objPos,
      display: 'block',
      flexShrink: '0',
    }"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Page } from './types';

const props = defineProps<{
  page: Page;
  edge: string;
  thumbBaseUrl: string;
  fraction: number;
}>();

const label = computed(() => props.page.name || props.page.scan);
const src   = computed(() => props.thumbBaseUrl + props.page.thumb);

const w = computed(() =>
  (props.edge === 'left' || props.edge === 'right')
    ? Math.round(props.page.thumb_width * props.fraction)
    : props.page.thumb_width
);

const h = computed(() =>
  (props.edge === 'top' || props.edge === 'bottom')
    ? Math.round(props.page.thumb_height * props.fraction)
    : props.page.thumb_height
);

const objPos = computed(() => ({
  left:   'left center',
  right:  'right center',
  top:    'center top',
  bottom: 'center bottom',
}[props.edge] ?? 'center center'));
</script>

<style scoped>
.page-strip {
  border: 1px solid var(--color-border, #dee2e6);
  border-radius: 1px;
}
.page-strip:hover {
  border-color: var(--color-accent, #2563eb);
}
</style>
