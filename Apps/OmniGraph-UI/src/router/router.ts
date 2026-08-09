import { createRouter, createWebHistory } from 'vue-router';
import Projects from '../projects/Projects.vue';
import ProjectDetailView from '../projects/ProjectDetailView.vue';
import IngestorView from '../ingestor/IngestorView.vue';
import FoliosView from '../folios/FoliosView.vue';
import InspectMode from '../folios/InspectMode.vue';
import CropMode from '../folios/CropMode.vue';
import EditMode from '../folios/EditMode.vue';
import Settings from '../settings/Settings.vue';
import RecogniseMode from "../folios/RecogniseMode.vue";

const projectProps = (route: { params: Record<string, unknown> }) => ({
    machineName: String(route.params.machineName),
    projectName: String(route.params.machineName),
});

const folioProps = (route: { params: Record<string, unknown> }) => ({
    ...projectProps(route),
    initialPageStem: route.params.page ? String(route.params.page) : undefined,
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
            path: '/projects/:machineName/ingestor/:page?',
            name: 'ingestor',
            component: IngestorView,
            props: folioProps,
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
                    path: 'inspect/:page?',
                    name: 'folios-inspect',
                    component: InspectMode,
                    props: folioProps,
                },
                {
                    path: 'crop/:page?',
                    name: 'folios-crop',
                    component: CropMode,
                    props: folioProps,
                },
                {
                    path: 'recognise/:page?',
                    name: 'folios-recognise',
                    component: RecogniseMode,
                    props: folioProps,
                },
                {
                    path: 'edit/:page?',
                    name: 'folios-edit',
                    component: EditMode,
                    props: folioProps,
                },
            ],
        },
    ],
});
