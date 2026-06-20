import { createApp } from 'vue'
import './style.css'
import './shoelace'
import App from './crop/App.vue'

createApp(App, {
    machineName: "test-project",
    projectName: 'Test Project',
}).mount('#app')
