<template>
  <div class="projects-page">
    <h2>Projects</h2>

    <p v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </p>

    <p v-if="isLoading" class="muted">
      Loading projects…
    </p>

    <ul v-else-if="projects.length > 0" class="project-list">
      <li
          v-for="project in projects"
          :key="project.machine_name"
          class="project-list-item"
      >
        <RouterLink
            :to="`/projects/${project.machine_name}`"
            class="project-link"
        >
          {{ project.name }}
        </RouterLink>

        <span class="project-machine-name">
          {{ project.machine_name }}
        </span>
      </li>
    </ul>

    <p v-else class="muted">
      No projects yet.
    </p>

    <h3>New project</h3>

    <form class="form-grid" @submit.prevent="createProject">
      <label for="proj-name">Name</label>
      <input
          id="proj-name"
          v-model="newProject.name"
          type="text"
          name="name"
          placeholder="My Book"
          required
          @input="syncMachineNameFromName"
      >

      <label for="machine_name">
        Machine name
        <small>Letters, numbers, - and _ only</small>
      </label>
      <input
          id="machine_name"
          v-model="newProject.machine_name"
          type="text"
          name="machine_name"
          placeholder="my-book"
          pattern="[a-zA-Z0-9_\-]+"
          required
          @input="userEditedMachineName = true"
      >

      <div class="form-actions">
        <sl-button
            type="submit"
            variant="primary"
            :loading="isCreating"
            :disabled="isCreating"
        >
          Create
        </sl-button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import type { Project } from '../types/project';

const projects = ref<Project[]>([]);
const isLoading = ref(false);
const isCreating = ref(false);
const errorMessage = ref('');

const userEditedMachineName = ref(false);

const newProject = reactive<Project>({
  name: '',
  machine_name: '',
});

function machineNameFromProjectName(name: string): string {
  return name
      .toLowerCase()
      .replace(/\s+/g, '-')
      .replace(/[^a-z0-9_-]/g, '');
}

function syncMachineNameFromName(): void {
  if (userEditedMachineName.value) {
    return;
  }

  newProject.machine_name = machineNameFromProjectName(newProject.name);
}

async function loadProjects(): Promise<void> {
  isLoading.value = true;
  errorMessage.value = '';

  try {
    const response = await fetch('/api/projects');

    if (!response.ok) {
      throw new Error(`Failed to load projects: ${response.status}`);
    }

    projects.value = await response.json() as Project[];
  } catch (error) {
    console.error(error);
    errorMessage.value = 'Could not load projects.';
  } finally {
    isLoading.value = false;
  }
}

async function createProject(): Promise<void> {
  isCreating.value = true;
  errorMessage.value = '';

  try {
    const response = await fetch('/api/projects', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(newProject),
    });

    if (!response.ok) {
      throw new Error(`Failed to create project: ${response.status}`);
    }

    const createdProject = await response.json() as Project;
    console.log('Created project:', createdProject);

    projects.value.push(createdProject);

    newProject.name = '';
    newProject.machine_name = '';
    userEditedMachineName.value = false;
  } catch (error) {
    console.error(error);
    errorMessage.value = 'Could not create project.';
  } finally {
    isCreating.value = false;
  }
}

onMounted(() => {
  void loadProjects();
});
</script>

<style scoped>
.projects-page {
  width: 100%;
  max-width: 48rem;
  box-sizing: border-box;
  margin: 2rem auto;
  padding: 0 1rem;
}

.project-list {
  list-style: none;
  padding: 0;
  margin: 1rem 0 2rem;
}

.project-list-item {
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--color-border);
}

.project-link {
  font-weight: 600;
  color: var(--color-accent);
  text-decoration: none;
}

.project-link:hover {
  color: var(--color-accent-hover);
  text-decoration: underline;
}

.project-machine-name {
  margin-left: 0.5rem;
  font-size: 0.8rem;
  color: var(--color-text-muted);
}

.muted {
  color: var(--color-text-muted);
}

.error-message {
  color: #dc2626;
}

.form-grid {
  display: grid;
  gap: 0.75rem;
}

.form-grid label {
  display: grid;
  gap: 0.25rem;
  font-weight: 600;
}

.form-grid small {
  font-weight: 400;
  color: var(--color-text-muted);
}

.form-grid input {
  padding: 0.5rem 0.625rem;
  color: var(--color-text);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font: inherit;
}

.form-actions {
  margin-top: 0.5rem;
}
</style>