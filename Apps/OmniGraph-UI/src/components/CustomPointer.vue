<template>
  <div
      v-if="visible"
      class="custom-pointer"
      :style="pointerStyle"
  >
    <sl-icon class="custom-pointer-icon" :name="icon" :style="iconStyle"></sl-icon>
    <span v-if="label" class="custom-pointer-label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  visible: boolean;
  enabled: boolean;
  x: number;
  y: number;
  icon: string;
  color: string;
  label?: string;
}>();

const pointerStyle = computed(() => ({
  transform: `translate(${props.x}px, ${props.y}px)`,
  opacity: props.enabled ? 1 : 0.5,
}));

const iconStyle = computed(() => ({
  color: props.color,
}));

</script>

<style scoped>
.custom-pointer {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 9999;
  pointer-events: none;

  display: inline-flex;
  align-items: center;
  gap: 0.35rem;

  font-size: 1rem;
  font-weight: 700;
  color: white;

  transform-origin: top left;
}

.custom-pointer-icon {
  width: 2rem;
  height: 2rem;
  border-radius: 999px;

  display: inline-flex;
  align-items: center;
  justify-content: center;

  transform: translate(-50%, -50%);

  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.custom-pointer-label {
  transform: translate(-1rem, -50%);
  padding: 0.25rem 0.45rem;
  border-radius: 0.35rem;
  background: rgba(15, 23, 42, 0.9);
  font-size: 0.75rem;
  white-space: nowrap;
}

</style>