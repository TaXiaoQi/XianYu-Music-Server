<template>
  <div class="mobile-layout">
    <header class="mobile-topbar">
      <div>
        <div class="mobile-kicker">弦予音乐</div>
      </div>
      <div class="top-actions">
        <button class="icon-btn" :title="`当前：${theme.modeLabel}`" @click="theme.cycleMode">
          <span v-if="theme.isDark">月</span>
          <span v-else>日</span>
        </button>
        <button class="icon-btn danger" title="退出登录" @click="handleLogout">退</button>
      </div>
    </header>

    <main class="mobile-main">
      <router-view v-slot="{ Component }">
        <transition name="mobile-page" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>

    <nav class="mobile-tabbar">
      <router-link v-for="item in primaryTabs" :key="item.to" :to="item.to">
        <span class="tab-icon">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </router-link>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useThemeStore } from '@/stores/theme'
import { showToast } from '@/api/client'

const router = useRouter()
const auth = useAuthStore()
const theme = useThemeStore()

const primaryTabs = [
  { to: '/m/dashboard', label: '首页', icon: '首' },
  { to: '/m/users', label: '用户', icon: '用' },
  { to: '/m/version', label: '版本', icon: '版' },
  { to: '/m/feedback', label: '反馈', icon: '反' },
  { to: '/m/more', label: '更多', icon: '更' },
]

async function handleLogout() {
  await auth.logout()
  showToast('已退出登录', 'success')
  router.push('/login')
}
</script>

<style scoped>
.mobile-layout {
  min-height: 100vh;
  background:
    radial-gradient(circle at 20% 0%, rgba(236, 65, 65, 0.10), transparent 32%),
    var(--page-bg);
  color: var(--text);
  padding-bottom: calc(74px + env(safe-area-inset-bottom));
}
.mobile-topbar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: calc(14px + env(safe-area-inset-top)) 16px 12px;
  background: color-mix(in srgb, var(--page-bg) 82%, transparent);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(18px);
}
.mobile-kicker {
  font-size: 20px;
  line-height: 1.2;
  color: var(--text);
  font-weight: 900;
  letter-spacing: -0.03em;
}
.top-actions {
  display: flex;
  gap: 8px;
}
.icon-btn {
  width: 38px;
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--card);
  color: var(--text);
  font-size: 13px;
  font-weight: 800;
}
.icon-btn.danger {
  color: #EC4141;
  background: rgba(236, 65, 65, 0.08);
}
.mobile-main {
  padding: 14px;
}
.mobile-main :deep(.card),
.mobile-main :deep(.section-card),
.mobile-main :deep(.panel) {
  border-radius: 18px;
}
.mobile-main :deep(.page-header),
.mobile-main :deep(.card-header),
.mobile-main :deep(.dsh-head) {
  flex-direction: column;
  align-items: stretch;
  gap: 12px;
}
.mobile-main :deep(.table-wrapper) {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
.mobile-main :deep(table) {
  min-width: 720px;
}
.mobile-main :deep(.form-grid),
.mobile-main :deep(.stats-grid),
.mobile-main :deep(.admin-grid),
.mobile-main :deep(.wallpaper-grid) {
  grid-template-columns: 1fr !important;
}
.mobile-main :deep(.modal),
.mobile-main :deep(.modal-dialog) {
  width: calc(100vw - 28px) !important;
  max-width: calc(100vw - 28px) !important;
}
.mobile-tabbar {
  position: fixed;
  left: 12px;
  right: 12px;
  bottom: max(10px, env(safe-area-inset-bottom));
  z-index: 30;
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 4px;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 24px;
  background: color-mix(in srgb, var(--card) 92%, transparent);
  box-shadow: var(--shadow-card);
  backdrop-filter: blur(22px);
}
.mobile-tabbar a {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 7px 2px;
  border-radius: 16px;
  color: var(--text-muted);
  text-decoration: none;
  font-size: 11px;
  font-weight: 700;
}
.mobile-tabbar a.router-link-active {
  color: #EC4141;
  background: rgba(236, 65, 65, 0.09);
}
.tab-icon {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 9px;
  background: var(--control-bg);
  font-size: 11px;
}
.mobile-page-enter-active,
.mobile-page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.mobile-page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}
.mobile-page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
