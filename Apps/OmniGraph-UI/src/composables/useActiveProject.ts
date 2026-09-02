import { computed, inject, provide, ref, watch, type ComputedRef, type InjectionKey, type Ref } from 'vue';
import type { Project } from '../types/project';

export interface ActiveProjectContext {
    activeProject: Ref<Project | null>;
    activeProjectName: ComputedRef<string>;
}

const activeProjectKey: InjectionKey<ActiveProjectContext> = Symbol('activeProject');

export function provideActiveProjectContext(machineName: Ref<string>) {
    const activeProject = ref<Project | null>(null);

    const fetchProject = async (id: string) => {
        if (!id) {
            activeProject.value = null;
            return;
        }

        try {
            const res = await fetch(`/api/projects/${id}`);
            if (res.ok) {
                activeProject.value = await res.json() as Project;
            } else {
                activeProject.value = null;
            }
        } catch (e) {
            console.error('Failed to fetch active project:', e);
            activeProject.value = null;
        }
    };

    watch(machineName, (newId) => {
        if (activeProject.value?.machine_name === newId) {
            return;
        }
        void fetchProject(newId);
    }, { immediate: true });

    const activeProjectName = computed(() => activeProject.value?.name || machineName.value || '');

    const context: ActiveProjectContext = {
        activeProject,
        activeProjectName,
    };

    provide(activeProjectKey, context);

    return context;
}

export function useActiveProjectContext() {
    const context = inject(activeProjectKey);
    if (!context) {
        throw new Error('Active project context was not provided');
    }
    return context;
}
