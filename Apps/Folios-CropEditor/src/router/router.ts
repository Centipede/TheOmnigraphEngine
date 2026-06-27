import { createRouter, createWebHistory } from 'vue-router';
import Projects from '../projects/Projects.vue';
import ProjectDetailView from '../projects/ProjectDetailView.vue';
import FoliosView from '../folios/FoliosView.vue';
import InspectMode from '../folios/InspectMode.vue';
import CropMode from '../folios/CropMode.vue';
import Settings from '../settings/Settings.vue';

const projectProps = (route: { params: Record<string, unknown> }) => ({
    machineName: String(route.params.machineName),
    projectName: String(route.params.machineName),
});

export const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/settings',
            name: 'settings',
            component: Settings,
        },
        {
            path: '/projects',
            name: 'projects',
            component: Projects,
        },
        {
            path: '/projects/:machineName',
            name: 'project-detail',
            component: ProjectDetailView,
            props: projectProps,
        },
        {
            path: '/projects/:machineName/folios',
            component: FoliosView,
            props: projectProps,
            children: [
                {
                    path: '',
                    redirect: { name: 'folios-inspect' },
                },
                {
                    path: 'inspect',
                    name: 'folios-inspect',
                    component: InspectMode,
                    props: projectProps,
                },
                {
                    path: 'crop',
                    name: 'folios-crop',
                    component: CropMode,
                    props: projectProps,
                },
            ],
        },
    ],
});
