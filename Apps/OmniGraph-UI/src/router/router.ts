import { createRouter, createWebHistory } from 'vue-router';
import Projects from '../projects/Projects.vue';
import ProjectDetailView from '../projects/ProjectDetailView.vue';
import IngestorView from '../ingestor/IngestorView.vue';
import AssembleMode from '../ingestor/AssembleMode.vue';
import ProcessMode from '../ingestor/ProcessMode.vue';
import FoliosView from '../folios/FoliosView.vue';
import AssistMode from '../folios/AssistMode.vue';
import CropMode from '../folios/CropMode.vue';
import HintMode from '../folios/HintMode.vue';
import EditMode from '../folios/EditMode.vue';
import Settings from '../settings/Settings.vue';
import RecogniseMode from "../folios/RecogniseMode.vue";
import CodexView from '../codex/CodexView.vue';
import CodexEditMode from '../codex/EditMode.vue';
import CodexScriptMode from '../codex/ScriptMode.vue';

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
            path: '/projects/:machineName/ingestor',
            component: IngestorView,
            props: projectProps,
            children: [
                {
                    path: '',
                    redirect: { name: 'ingestor-assemble' },
                },
                {
                    path: 'assemble/:page?',
                    name: 'ingestor-assemble',
                    component: AssembleMode,
                    props: folioProps,
                },
                {
                    path: 'process/:page?',
                    name: 'ingestor-process',
                    component: ProcessMode,
                    props: folioProps,
                },
            ],
        },
        {
            path: '/projects/:machineName/folios',
            component: FoliosView,
            props: projectProps,
            children: [
                {
                    path: '',
                    redirect: { name: 'folios-assist' },
                },
                {
                    path: 'assist/:page?',
                    name: 'folios-assist',
                    component: AssistMode,
                    props: folioProps,
                },
                {
                    path: 'crop/:page?',
                    name: 'folios-crop',
                    component: CropMode,
                    props: folioProps,
                },
                {
                    path: 'hint/:page?',
                    name: 'folios-hint',
                    component: HintMode,
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
        {
            path: '/projects/:machineName/codex',
            component: CodexView,
            props: projectProps,
            children: [
                {
                    path: '',
                    redirect: { name: 'codex-edit' },
                },
                {
                    path: 'edit/:page?',
                    name: 'codex-edit',
                    component: CodexEditMode,
                    props: folioProps,
                },
                {
                    path: 'script/:page?',
                    name: 'codex-script',
                    component: CodexScriptMode,
                    props: folioProps,
                },
            ],
        },
    ],
});
