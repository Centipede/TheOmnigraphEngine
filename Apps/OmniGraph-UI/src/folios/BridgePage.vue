<template>
  <div class="bridge-page">
    <PageCanvas
      v-bind="$attrs"
      :page="page"
      :machine-name="machineName"
      :image-base-url="imageBaseUrl"
      :minimal="minimal"
      hocrLevel="block"
    />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted } from 'vue';
import PageCanvas from '../components/PageCanvas.vue';
import { provideHocrContext } from '../composables/useHocr';
import type { Page } from '../types';

const props = defineProps<{
  page: Page;
  machineName: string;
  imageBaseUrl: string;
  minimal?: boolean;
}>();

const { loadHocr } = provideHocrContext();

const reloadHocr = () => {
  if (props.page && props.machineName) {
    // page.scan is used as the stem for HOCR loading
    loadHocr(props.machineName, props.page.scan);
  }
};

watch(() => props.page?.scan, reloadHocr);
watch(() => props.machineName, reloadHocr);

onMounted(reloadHocr);
</script>

<style scoped>
.bridge-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
