import { setBasePath } from '@shoelace-style/shoelace/dist/utilities/base-path.js';
import { createApp } from 'vue';
import App from './App.vue';

setBasePath('https://cdn.jsdelivr.net/npm/@shoelace-style/shoelace@2.20.1/dist');

const mountEl = document.getElementById('folios-crop-app');
const machineName = mountEl?.dataset.machineName ?? '';
const projectName = mountEl?.dataset.projectName ?? '';

const app = createApp(App, { machineName, projectName });
app.config.compilerOptions.isCustomElement = (tag) => tag.startsWith('sl-');
app.mount('#folios-crop-app');
