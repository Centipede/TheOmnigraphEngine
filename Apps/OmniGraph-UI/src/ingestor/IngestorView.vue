<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :initial-show-page-preview="true"
  >

    <template v-slot:tools>
      <sl-button>Append</sl-button>
    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from "../components/PageWorkspace.vue";

/*const props = */defineProps<{ machineName: string; projectName: string }>();


</script>

<style scoped>

</style>

