import { createRouter, createWebHistory } from 'vue-router';
import Projects from '../projects/Projects.vue';
import InspectMode from '../folios/InspectMode.vue';
import CropMode from '../folios/CropMode.vue';

export const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/projects',
            name: 'projects',
            component: Projects,
        },
        {
            path: '/projects/:machineName/folios',
            name: 'folios-inspect',
            component: InspectMode,
            props: route => ({
                machineName: String(route.params.machineName),
                projectName: String(route.params.machineName),
            }),
        },
        {
            path: '/projects/:machineName/folios/crop',
            name: 'folios-crop',
            component: CropMode,
            props: route => ({
                machineName: String(route.params.machineName),
                projectName: String(route.params.machineName),
            }),
        },
    ],
});
