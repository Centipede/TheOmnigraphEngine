<template>
  <PageWorkspace
      :machine-name="machineName"
      :project-name="projectName"
      :show-crop-overlay="true"
  >
    <template #tools>

    </template>

  </PageWorkspace>
</template>

<script setup lang="ts">
import PageWorkspace from "../components/PageWorkspace.vue";

/*const props = */defineProps<{ machineName: string; projectName: string }>();


</script>

<style scoped>

</style>

