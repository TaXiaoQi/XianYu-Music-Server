<template>
  <div class="mobile-layout">
    <header class="mobile-topbar">
      <div>
        <div class="mobile-kicker">弦予音乐</div>
      </div>
      <div class="top-actions">
        <button class="icon-btn notify-btn" :title="notifyLabel" @click="notifyOpen = true">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
          <span v-if="notify.pendingTotal > 0" class="notify-badge">{{ notify.pendingTotal }}</span>
        </button>
        <button class="icon-btn theme-toggle" :title="`当前：${theme.modeLabel}`" @click="theme.cycleMode">
          <svg v-if="theme.isDark" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z" />
          </svg>
          <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
          </svg>
          <span class="theme-label">{{ theme.modeLabel }}</span>
        </button>
        <button class="logout-btn" @click="handleLogout">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
            <polyline points="16 17 21 12 16 7"/>
            <line x1="21" y1="12" x2="9" y2="12"/>
          </svg>
          退出
        </button>
      </div>
    </header>

    <main class="mobile-main">
      <router-view v-slot="{ Component, route: r }">
        <transition name="route-fade" mode="out-in">
          <component :is="Component" :key="r.path" />
        </transition>
      </router-view>
    </main>

    <nav class="mobile-tabbar">
      <router-link v-for="item in primaryTabs" :key="item.to" :to="item.to">
        <span class="tab-icon" v-html="item.icon"></span>
        <span class="tab-label">{{ item.label }}</span>
      </router-link>
    </nav>

    <!-- 浏览器通知设置弹窗（居中，仅内部按钮关闭） -->
    <transition name="mobile-fade">
      <div v-if="notifyOpen" class="notify-overlay">
        <div class="notify-dialog">
          <div class="notify-head">
            <span class="notify-title">{{ notify.nativeBridgeAvailable ? '系统通知' : '浏览器通知' }}</span>
            <span class="notify-status" :class="notify.permission">{{ notify.permissionLabel }}</span>
          </div>

          <div v-if="!notify.supportedByBrowser" class="notify-tip warn">
            当前环境不支持系统通知，请改用最新版 Chrome / Edge / Firefox 或系统浏览器打开。
          </div>

          <template v-else>
            <div v-if="notify.permission === 'default'" class="notify-perm-row">
              <button class="mobile-btn primary" @click="handleRequestPermission">授权通知</button>
              <span class="notify-hint">{{ notify.nativeBridgeAvailable ? '系统将询问是否允许发送通知' : '浏览器将询问是否允许本站发送通知' }}</span>
            </div>
            <div v-else-if="notify.permission === 'denied'" class="notify-tip warn">
              {{ notify.nativeBridgeAvailable ? '系统通知权限已被拒绝' : '浏览器已拒绝通知权限，请在浏览器设置中允许本站通知后重试。' }}
              <button v-if="notify.nativeBridgeAvailable" class="notify-settings-btn" @click="notify.openNotificationSettings()">打开系统设置</button>
            </div>
            <div v-else class="notify-perm-ok">
              <span class="notify-hint">已授权，有新事件时将在此提醒</span>
            </div>

            <div class="notify-divider"></div>

            <div class="notify-row">
              <span class="notify-row-label">启用通知</span>
              <span class="switch" :class="{ on: notify.enabled }" @click="notify.setEnabled(!notify.enabled)"><span class="switch-knob"></span></span>
            </div>

            <div class="notify-modules">
              <div v-for="m in notify.moduleList" :key="m.key" class="notify-row">
                <span class="notify-row-label">
                  {{ m.label }}
                  <span class="notify-row-desc">{{ m.desc }}</span>
                </span>
                <span class="switch" :class="{ on: notify.modules[m.key] }" @click="notify.toggleModule(m.key)"><span class="switch-knob"></span></span>
              </div>
            </div>

            <div class="notify-divider"></div>

            <button class="mobile-btn" :disabled="!notify.granted" @click="notify.testNotification()">发送测试通知</button>
            <div class="notify-tip">有新反馈或待审核项时，将弹出系统通知提醒。</div>
          </template>

          <div class="notify-actions">
            <button class="notify-close" @click="notifyOpen = false">关闭</button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useThemeStore } from '@/stores/theme'
import { useNotificationStore } from '@/stores/notification'
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
const notify = useNotificationStore()

const notifyOpen = ref(false)
const notifyLabel = computed(() => (notify.canNotify ? '通知已开启' : '通知未开启'))

watch(notifyOpen, (open) => {
  if (open) notify.refreshPermission()
})

onMounted(() => {
  notify.startPolling()
})
onUnmounted(() => {
  notify.stopPolling()
})

async function handleRequestPermission() {
  const p = await notify.requestPermission()
  if (p === 'granted') {
    showToast('已允许系统通知', 'success')
  } else if (p === 'denied') {
    showToast('通知权限被拒绝，请在系统设置中开启', 'error')
  }
}

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
/* 主题切换（参考桌面端设计） */
.theme-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: auto;
  height: 38px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-light);
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
  transition: transform 0.16s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)),
              color 0.16s, background 0.16s, border-color 0.16s;
}
.theme-toggle:active {
  transform: scale(0.94);
  color: var(--accent);
  background: var(--accent-soft);
  border-color: var(--accent);
}
.theme-toggle svg {
  flex-shrink: 0;
}
/* 退出登录（参考桌面端设计） */
.logout-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 38px;
  padding: 0 14px;
  border-radius: 12px;
  border: 1px solid rgba(236, 65, 65, 0.16);
  background: rgba(236, 65, 65, 0.08);
  color: var(--accent);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  transition: transform 0.16s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)),
              color 0.16s, background 0.16s, border-color 0.16s, box-shadow 0.16s;
}
.logout-btn:active {
  transform: scale(0.94);
  background: rgba(236, 65, 65, 0.12);
}
.logout-btn svg {
  flex-shrink: 0;
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
.route-fade-enter-active,
.route-fade-leave-active {
  transition: opacity 0.22s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}
.route-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}
.route-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

/* ===== 浏览器通知铃铛 ===== */
.notify-btn {
  position: relative;
}
.notify-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  background: #EC4141;
  color: #fff;
  font-size: 10px;
  font-weight: 800;
  line-height: 16px;
  text-align: center;
  box-shadow: 0 0 0 2px var(--card-solid, var(--white));
}

/* ===== 浏览器通知设置弹窗（居中，仅内部按钮关闭） ===== */
.notify-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  background: rgba(15, 23, 42, 0.38);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.notify-dialog {
  width: 100%;
  max-width: 340px;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
  padding: 18px 20px 0;
}
.notify-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
.notify-title {
  font-size: 16px;
  font-weight: 850;
  color: var(--text);
}
.notify-status {
  padding: 2px 9px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
}
.notify-status.granted {
  background: rgba(34, 197, 94, 0.12);
  color: #16a34a;
}
.notify-status.denied {
  background: rgba(236, 65, 65, 0.12);
  color: #EC4141;
}
.notify-status.default {
  background: var(--accent-soft);
  color: var(--accent);
}
.notify-status.unsupported {
  background: var(--control-bg);
  color: var(--text-muted);
}
.notify-perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.notify-perm-ok {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(34, 197, 94, 0.08);
}
.notify-hint {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip {
  margin-top: 10px;
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip.warn {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(236, 65, 65, 0.08);
  color: #EC4141;
}
.notify-settings-btn {
  display: inline-block;
  margin-top: 8px;
  padding: 6px 12px;
  border: none;
  border-radius: 8px;
  background: #EC4141;
  color: #fff;
  font-size: 12px;
  cursor: pointer;
}
.notify-settings-btn:active {
  opacity: 0.85;
}
.notify-divider {
  height: 1px;
  margin: 10px 0;
  background: var(--border);
}
.notify-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 5px 0;
}
.notify-row-label {
  display: inline-flex;
  flex-direction: column;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.notify-row-desc {
  font-size: 11px;
  font-weight: 400;
  color: var(--text-muted);
}
.notify-modules {
  display: flex;
  flex-direction: column;
}
.switch {
  width: 40px;
  height: 22px;
  border-radius: 12px;
  border: none;
  background: #d1d5db;
  cursor: pointer;
  position: relative;
  flex-shrink: 0;
  transition: background 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.switch.on {
  background: var(--accent);
}
.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.switch.on .switch-knob {
  transform: translateX(18px);
}
.notify-actions {
  display: flex;
  margin: 14px -20px 0;
  border-top: 1px solid var(--border);
}
.notify-close {
  flex: 1;
  border: none;
  padding: 15px 0;
  background: transparent;
  font-size: 15px;
  font-weight: 800;
  color: var(--text-muted);
  cursor: pointer;
  transition: background 0.18s;
}
.notify-close:active {
  background: var(--control-bg);
}

/* 弹窗居中缩放 + 透明度过渡（0.24s，符合移动端规范） */
.mobile-fade-enter-active,
.mobile-fade-leave-active {
  transition: opacity 0.24s cubic-bezier(0.16, 1, 0.3, 1);
}
.mobile-fade-enter-from,
.mobile-fade-leave-to {
  opacity: 0;
}
.mobile-fade-enter-active .notify-dialog,
.mobile-fade-leave-active .notify-dialog {
  transition: transform 0.24s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.24s;
}
.mobile-fade-enter-from .notify-dialog,
.mobile-fade-leave-to .notify-dialog {
  opacity: 0;
  transform: scale(0.94);
}
html[data-theme='dark'] .notify-dialog {
  background: var(--card-solid) !important;
}
html[data-theme='dark'] .notify-overlay {
  background: rgba(0, 0, 0, 0.56) !important;
}
</style>
