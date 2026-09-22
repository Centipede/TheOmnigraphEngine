<template>
  <div class="bridge-page">
    <PageCanvas
      v-bind="$attrs"
      :page="page"
      :machine-name="machineName"
      :image-base-url="imageBaseUrl"
      :minimal="minimal"
      :show-block-hints="showBlockHints"
      hocrLevel="block"
      :interaction-update="interactionUpdate"
      :interaction-click="interactionClick"
    />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted, inject, type Ref } from 'vue';
import PageCanvas from '../components/PageCanvas.vue';
import { provideHocrContext, useHocrContext } from '../composables/useHocr';
import type { Page, PageInteractionUpdate, PageInteractionClick } from '../types';

const props = defineProps<{
  page: Page;
  machineName: string;
  imageBaseUrl: string;
  minimal?: boolean;
  showBlockHints?: boolean;
  reloadTrigger?: number;
  interactionUpdate?: PageInteractionUpdate;
  interactionClick?: PageInteractionClick;
}>();

// Inject parent context (shared by tools) before providing local context
const parentContext = useHocrContext();
const selectedPageScan = inject<Ref<string | null>>('selectedPageScan');
const { hocrPage, loadHocr } = provideHocrContext();

const reloadHocr = () => {
  if (props.page && props.machineName) {
    // page.scan is used as the stem for HOCR loading
    loadHocr(props.machineName, props.page.scan, { block_metrics: true });
  }
};

// Sync from parent context when it updates (e.g. after "Auto detect")
watch(() => parentContext.hocrPage.value, (newPage) => {
  if (newPage && props.page) {
    const currentStem = props.page.scan.replace(/\.[^.]+$/, '');
    if (parentContext.stem.value === currentStem) {
      hocrPage.value = newPage;
    }
  }
});

// Sync to parent context when this page is selected
watch([() => hocrPage.value, () => selectedPageScan?.value], ([newHocr, selectedScan]) => {
  if (selectedScan === props.page?.scan && newHocr) {
    parentContext.updateHocr(newHocr);
    parentContext.machineName.value = props.machineName;
    parentContext.stem.value = props.page.scan.replace(/\.[^.]+$/, '');
  }
}, { immediate: true });

watch(() => props.page?.scan, reloadHocr);
watch(() => props.machineName, reloadHocr);
watch(() => props.reloadTrigger, reloadHocr);

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
