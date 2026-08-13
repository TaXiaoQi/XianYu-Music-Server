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
        <span class="tab-icon" v-html="item.icon"></span>
        <span class="tab-label">{{ item.label }}</span>
      </router-link>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useThemeStore } from '@/stores/theme'
import { showToast } from '@/api/client'

const router = useRouter()
const route = useRoute()

// 切换路由时回到容器顶部
watch(
  () => route.fullPath,
  () => {
    window.scrollTo({ top: 0, left: 0, behavior: 'auto' })
  }
)
const auth = useAuthStore()
const theme = useThemeStore()

const primaryTabs = [
  {
    to: '/m/dashboard',
    label: '首页',
    icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V21h14V9.5"/><path d="M9 21v-6h6v6"/></svg>',
  },
  {
    to: '/m/feedback',
    label: '反馈',
    icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a4 4 0 0 1-4 4H8l-5 3V7a4 4 0 0 1 4-4h10a4 4 0 0 1 4 4z"/><path d="M8 9h8"/><path d="M8 13h5"/></svg>',
  },
  {
    to: '/m/version',
    label: '版本',
    icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/></svg>',
  },
  {
    to: '/m/users',
    label: '用户',
    icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>',
  },
  {
    to: '/m/more',
    label: '更多',
    icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="1.6"/><circle cx="19" cy="12" r="1.6"/><circle cx="5" cy="12" r="1.6"/></svg>',
  },
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
  justify-content: center;
  gap: 4px;
  min-width: 0;
  padding: 6px 2px 7px;
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
  width: 24px;
  height: 24px;
  color: currentColor;
}
.tab-icon :deep(svg) {
  width: 21px;
  height: 21px;
}
.tab-label {
  display: block;
  max-width: 100%;
  overflow: hidden;
  line-height: 1.1;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mobile-page-enter-active,
.mobile-page-leave-active {
  transition: opacity 0.3s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.mobile-page-enter-from {
  opacity: 0;
  transform: translateY(12px);
}
.mobile-page-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
