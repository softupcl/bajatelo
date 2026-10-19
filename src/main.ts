import { createApp } from 'vue'
import './styles.css'
import App from './App.vue'
import { installE2eBridge } from './services/e2e-fake'

installE2eBridge()
createApp(App).mount('#app')
