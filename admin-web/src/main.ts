import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/main.css'
import { useThemeStore } from '@/stores/theme'
import { initAdminIdleLogout } from '@/utils/adminIdleLogout'
import { initI18n } from './i18n'

initI18n()

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
useThemeStore(pinia).init()
app.use(router)
initAdminIdleLogout(router)
app.mount('#app')
