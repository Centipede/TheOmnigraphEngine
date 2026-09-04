import { setBasePath } from '@shoelace-style/shoelace/dist/utilities/base-path.js';
import { createApp } from 'vue';
import { createHead } from '@unhead/vue/client'
import './shoelace'
import App from './App.vue';
import {router} from "./router/router.ts";

setBasePath('https://cdn.jsdelivr.net/npm/@shoelace-style/shoelace@2.20.1/dist');

const mountEl = document.getElementById('app');
const machineName = mountEl?.dataset.machineName ?? '';
const projectName = mountEl?.dataset.projectName ?? '';

// Vue now renders its own navbar, so the header height is always 56px.
document.documentElement.style.setProperty('--header-height', '56px');

const app = createApp(App, { machineName, projectName });
const head = createHead()

app.config.compilerOptions.isCustomElement = (tag) => tag.startsWith('sl-');
app.use(router);
app.use(head);
app.mount('#app');
