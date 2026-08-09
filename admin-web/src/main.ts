import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/main.css'
import { useThemeStore } from '@/stores/theme'
import { initAdminIdleLogout } from '@/utils/adminIdleLogout'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
useThemeStore(pinia).init()
app.use(router)
initAdminIdleLogout(router)
app.mount('#app')
