<template>
  <div class="layout">
    <div class="overlay" :class="{ show: sidebarOpen }" @click="sidebarOpen = false"></div>
    <!-- 侧边栏 -->
    <aside class="sidebar" :class="{ open: sidebarOpen }">
      <div class="sidebar-title">
        <img :src="siteLogoUrl" alt="弦予音乐" class="sidebar-logo" />
        弦予音乐
      </div>
      <ul class="sidebar-menu">
        <li><router-link to="/dashboard"><span class="menu-label"><span class="menu-icon" v-html="icons.dashboard"></span>仪表盘</span></router-link></li>
        <li><router-link to="/users"><span class="menu-label"><span class="menu-icon" v-html="icons.users"></span>用户管理</span></router-link></li>
        <li><router-link to="/device-banned"><span class="menu-label"><span class="menu-icon" v-html="icons.device"></span>设备管理</span></router-link></li>

        <!-- 内容管理 -->
        <li>
          <div class="menu-header" @click="toggleSubmenu('content')">
            <span class="menu-label"><span class="menu-icon" v-html="icons.content"></span>内容管理</span>
            <span class="menu-arrow" :style="{ transform: openMenu === 'content' ? 'rotate(90deg)' : '' }">&#9654;</span>
          </div>
          <ul class="submenu" :class="{ open: openMenu === 'content' }">
            <li><router-link to="/version">版本管理</router-link></li>
            <li><router-link to="/announcements">公告管理</router-link></li>
            <li><router-link to="/wallpapers">壁纸管理</router-link></li>
            <li><router-link to="/avatar-audit">头像/改名审核</router-link></li>
            <li><router-link to="/user-agreement">用户协议</router-link></li>
            <li><router-link to="/about-config">关于页配置</router-link></li>
          </ul>
        </li>

        <li><router-link to="/feedback"><span class="menu-label"><span class="menu-icon" v-html="icons.content"></span>反馈与建议</span></router-link></li>

        <!-- 系统管理 -->
        <li>
          <div class="menu-header" @click="toggleSubmenu('system')">
            <span class="menu-label"><span class="menu-icon" v-html="icons.system"></span>系统管理</span>
            <span class="menu-arrow" :style="{ transform: openMenu === 'system' ? 'rotate(90deg)' : '' }">&#9654;</span>
          </div>
          <ul class="submenu" :class="{ open: openMenu === 'system' }">
            <li><router-link to="/admin-account">后台账号管理</router-link></li>
            <li><router-link to="/external-notification">外部通知</router-link></li>
            <li><router-link to="/email-config">邮箱机设置</router-link></li>
            <li><router-link to="/turnstile-config">审核设置</router-link></li>
            <li><router-link to="/config-file">配置文件管理</router-link></li>
            <li><router-link to="/site-config">Logo 配置</router-link></li>
          </ul>
        </li>

        <!-- 数据日志 -->
        <li>
          <div class="menu-header" @click="toggleSubmenu('data')">
            <span class="menu-label"><span class="menu-icon" v-html="icons.data"></span>数据日志</span>
            <span class="menu-arrow" :style="{ transform: openMenu === 'data' ? 'rotate(90deg)' : '' }">&#9654;</span>
          </div>
          <ul class="submenu" :class="{ open: openMenu === 'data' }">
            <li><router-link to="/database">数据库管理</router-link></li>
            <li><router-link to="/api-test">接口测试</router-link></li>
            <li><router-link to="/logs">后台日志</router-link></li>
            <li><router-link to="/error-log">报错日志</router-link></li>
            <li><router-link to="/app-login-log">APP登录日志</router-link></li>
          </ul>
        </li>
      </ul>
      <div class="sidebar-switch">
        <button class="switch-mobile-btn" @click="goMobile">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="2" width="10" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
          切换移动版
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="main">
      <div class="topbar">
        <div class="topbar-left">
          <button class="menu-toggle" @click="sidebarOpen = true">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="3" y1="6" x2="21" y2="6"/>
              <line x1="3" y1="12" x2="21" y2="12"/>
              <line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          </button>
          <h1>{{ pageTitle }}</h1>
          <span v-if="isDebugMode" class="debug-chip">本地调试</span>
        </div>
        <div class="topbar-right">
          <div class="notify-wrap">
            <button class="notify-btn" :title="notifyLabel" @click="notifyOpen = !notifyOpen">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
                <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
              </svg>
              <span v-if="notify.pendingTotal > 0" class="notify-badge">{{ notify.pendingTotal }}</span>
            </button>

            <teleport to="body">
              <transition name="notify-pop">
                <div v-if="notifyOpen" class="notify-panel notify-panel--msg" :style="panelStyle">
                  <div class="notify-panel-head">
                    <span class="notify-title">消息通知</span>
                    <span class="notify-total">{{ noticeTotal }} 条待处理</span>
                  </div>

                  <div v-if="noticeTotal === 0" class="notify-empty">
                    <p>暂无待处理事项</p>
                    <span>有新反馈或待审核项时会显示在这里</span>
                  </div>

                  <div v-else class="notify-msg-list">
                    <button
                      v-for="item in noticeItems"
                      :key="item.label"
                      type="button"
                      class="notify-msg-item"
                      @click="goNotice(item.to)"
                    >
                      <span class="notify-msg-dot" :class="item.className"></span>
                      <span class="notify-msg-text">
                        <strong>{{ item.label }}</strong>
                        <small>{{ item.desc }}</small>
                      </span>
                      <b>{{ item.count }}</b>
                    </button>
                  </div>

                  <div class="notify-actions notify-actions--split">
                    <button type="button" class="notify-settings-link" @click="goNotice('/external-notification?tab=settings')">通知设置</button>
                    <button type="button" class="notify-close" @click="notifyOpen = false">关闭</button>
                  </div>
                </div>
              </transition>
            </teleport>
          </div>
          <button class="theme-toggle" :title="`当前：${theme.modeLabel}`" @click="theme.cycleMode">
            <svg v-if="theme.isDark" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z"/>
            </svg>
            <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="4"/>
              <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
            </svg>
            <span>{{ theme.modeLabel }}</span>
          </button>
          <span v-if="auth.user" class="topbar-admin">
            <img v-if="auth.user.avatar_url" :src="auth.user.avatar_url" alt="" class="topbar-avatar" />
            <span v-else class="topbar-avatar topbar-avatar-letter">{{ (auth.user.username || 'A').charAt(0).toUpperCase() }}</span>
            <span class="topbar-admin-name">{{ auth.user.username }} ({{ auth.user.role }})</span>
          </span>
          <button class="logout-btn" @click="handleLogout">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
              <polyline points="16 17 21 12 16 7"/>
              <line x1="21" y1="12" x2="9" y2="12"/>
            </svg>
            退出
          </button>
        </div>
      </div>
      <router-view v-slot="{ Component }">
        <transition name="route-fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useThemeStore } from '@/stores/theme'
import { useNotificationStore } from '@/stores/notification'
import { showToast } from '@/api/client'
import { loadSiteLogo, siteLogoUrl } from '@/utils/siteLogo'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const theme = useThemeStore()
const notify = useNotificationStore()

const sidebarOpen = ref(false)
const openMenu = ref<string | null>(null)
const notifyOpen = ref(false)
const panelStyle = ref<Record<string, string>>({})
const isDebugMode = import.meta.env.DEV

const pageTitle = computed(() => (route.meta.title as string) || '仪表盘')
const notifyLabel = computed(() => (notify.canNotify ? '通知已开启' : '通知未开启'))

/** 铃铛消息通知列表（与仪表盘「消息通知」一致） */
const noticeItems = computed(() => [
  { label: '新壁纸审核', desc: '用户上传壁纸待审核', count: notify.totals.wallpaper || 0, to: '/wallpapers', className: 'wallpaper' },
  { label: '新头像审核', desc: '用户头像变更待审核', count: notify.totals.avatar || 0, to: '/avatar-audit', className: 'avatar' },
  { label: '新名称审核', desc: '用户改名申请待审核', count: notify.totals.nickname || 0, to: '/avatar-audit', className: 'nickname' },
  { label: '新问题反馈', desc: '用户反馈待处理', count: notify.totals.feedback || 0, to: '/feedback', className: 'feedback' },
])
const noticeTotal = computed(() => noticeItems.value.reduce((sum, i) => sum + Number(i.count || 0), 0))

function goNotice(to: string) {
  notifyOpen.value = false
  router.push(to)
}

onMounted(() => {
  loadSiteLogo()
  document.addEventListener('click', onClickOutside)
  notify.startPolling()
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
  notify.stopPolling()
})

watch(notifyOpen, (open) => {
  if (open) {
    notify.refreshPermission()
    const btn = document.querySelector('.notify-btn') as HTMLElement | null
    if (btn) {
      const rect = btn.getBoundingClientRect()
      const panelW = 320
      const gap = 8
      const left = Math.max(8, Math.min(rect.right - panelW, window.innerWidth - panelW - 8))
      panelStyle.value = {
        top: `${rect.bottom + gap}px`,
        left: `${left}px`,
      }
    }
  }
})

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.notify-wrap')) {
    notifyOpen.value = false
  }
}

function toggleSubmenu(id: string) {
  openMenu.value = openMenu.value === id ? null : id
}

function goMobile() {
  router.push('/m/dashboard')
}

async function handleLogout() {
  await auth.logout()
  showToast('已退出登录', 'success')
  router.push('/login')
}

// SVG 图标
const icons = {
  dashboard: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>',
  users: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>',
  device: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>',
  data: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>',
  content: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>',
  system: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>',
}
</script>

<style scoped>
/* ===== 浏览器通知下拉面板 ===== */
.notify-wrap {
  position: relative;
}
.notify-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 34px;
  height: 34px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-light);
  cursor: pointer;
  transition: transform 0.2s var(--motion), background 0.2s, color 0.2s, box-shadow 0.2s;
}
.notify-btn:hover {
  transform: translateY(-1px);
  color: var(--accent);
  background: var(--accent-soft);
  box-shadow: var(--shadow-soft);
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
.notify-panel {
  position: fixed;
  width: 320px;
  max-height: calc(100vh - 90px);
  overflow-y: auto;
  padding: 18px 20px 0;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  z-index: 99999;
}
.notify-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.notify-title {
  font-size: 14px;
  font-weight: 800;
  color: var(--text);
}
.notify-total {
  padding: 2px 9px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
}
.notify-empty {
  padding: 26px 10px;
  text-align: center;
}
.notify-empty p {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  margin-bottom: 6px;
}
.notify-empty span {
  font-size: 12px;
  color: var(--text-muted);
}
.notify-msg-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.notify-msg-item {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  width: 100%;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg, #fafafa);
  color: var(--text);
  text-align: left;
  cursor: pointer;
  transition: transform 0.16s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)),
              border-color 0.16s, background 0.16s;
}
.notify-msg-item:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.notify-msg-item:active {
  transform: scale(0.97);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.notify-msg-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--accent);
}
.notify-msg-dot.wallpaper { background: #3b82f6; }
.notify-msg-dot.avatar { background: #22c55e; }
.notify-msg-dot.nickname { background: #f97316; }
.notify-msg-dot.feedback { background: #8b5cf6; }
.notify-msg-text {
  min-width: 0;
}
.notify-msg-text strong,
.notify-msg-text small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.notify-msg-text strong {
  font-size: 13px;
}
.notify-msg-text small {
  margin-top: 3px;
  color: var(--text-muted);
  font-size: 11px;
}
.notify-msg-item b {
  color: var(--accent);
  font-size: 18px;
}
.notify-actions {
  display: flex;
  margin: 14px -20px 0;
  border-top: 1px solid var(--border);
}
.notify-actions--split {
  position: sticky;
  bottom: 0;
  background: var(--card-solid, var(--card));
}
.notify-settings-link {
  flex: 1;
  border: none;
  padding: 15px 0;
  background: transparent;
  font-size: 14px;
  font-weight: 700;
  color: var(--accent);
  cursor: pointer;
  transition: background 0.18s;
}
.notify-settings-link:hover {
  background: var(--control-bg);
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
.notify-close:hover {
  background: var(--control-bg);
}
.notify-pop-enter-active,
.notify-pop-leave-active {
  transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  transform-origin: top right;
}
.notify-pop-enter-from,
.notify-pop-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-6px);
}
.debug-chip {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 9px;
  border-radius: 999px;
  background: rgba(236, 65, 65, 0.10);
  color: #EC4141;
  font-size: 12px;
  font-weight: 700;
  box-shadow: inset 0 0 0 1px rgba(236, 65, 65, 0.10);
}
.theme-toggle {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 34px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-light);
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  transition: transform 0.2s var(--motion), background 0.2s, color 0.2s, box-shadow 0.2s;
}
.theme-toggle:hover {
  transform: translateY(-1px);
  color: var(--accent);
  background: var(--accent-soft);
  box-shadow: var(--shadow-soft);
}
.topbar-admin {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  border-radius: 20px;
  background: var(--white);
  border: 1px solid var(--border);
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.topbar-avatar {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  object-fit: cover;
  flex-shrink: 0;
}
.topbar-avatar-letter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 13px;
  font-weight: 700;
}
.topbar-admin-name {
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
</style>
