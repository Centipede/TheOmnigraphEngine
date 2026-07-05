<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :panels="panels"
  >

    <template v-slot:tools>
      <sl-button>Append</sl-button>
    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from "../components/PageWorkspace.vue";
import type {PanelVisibility} from "../types";

const props = defineProps<{
  machineName: string;
  projectName: string;
  panels: PanelVisibility;
}>();

</script>

<style scoped>

</style>

