<template>
  <div class="layout">
    <div class="overlay" :class="{ show: sidebarOpen }" @click="sidebarOpen = false"></div>
    <!-- 侧边栏 -->
    <aside class="sidebar" :class="{ open: sidebarOpen }">
      <div class="sidebar-title">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
        弦予音乐
      </div>
      <ul class="sidebar-menu">
        <li><router-link to="/dashboard"><span class="menu-label"><span class="menu-icon" v-html="icons.dashboard"></span>仪表盘</span></router-link></li>
        <li><router-link to="/users"><span class="menu-label"><span class="menu-icon" v-html="icons.users"></span>用户管理</span></router-link></li>

        <!-- 数据日志 -->
        <li>
          <div class="menu-header" @click="toggleSubmenu('data')">
            <span class="menu-label"><span class="menu-icon" v-html="icons.data"></span>数据日志</span>
            <span class="menu-arrow" :style="{ transform: openMenu === 'data' ? 'rotate(90deg)' : '' }">&#9654;</span>
          </div>
          <ul class="submenu" :class="{ open: openMenu === 'data' }">
            <li><router-link to="/error-log">报错日志</router-link></li>
            <li><router-link to="/app-login-log">APP登录日志</router-link></li>
          </ul>
        </li>

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
            <li><router-link to="/avatar-audit">头像审核</router-link></li>
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
            <li><router-link to="/admins">管理员管理</router-link></li>
            <li><router-link to="/account">账户管理</router-link></li>
            <li><router-link to="/password">修改密码</router-link></li>
            <li><router-link to="/logs">后台日志</router-link></li>
            <li><router-link to="/database">数据库管理</router-link></li>
            <li><router-link to="/api-test">接口测试</router-link></li>
          </ul>
        </li>
      </ul>
    </aside>

    <!-- 主内容区 -->
    <main class="main">
      <div class="topbar">
        <div class="topbar-left">
          <button class="menu-toggle" @click="sidebarOpen = true">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#1a1a1a" stroke-width="2" stroke-linecap="round">
              <line x1="3" y1="6" x2="21" y2="6"/>
              <line x1="3" y1="12" x2="21" y2="12"/>
              <line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          </button>
          <h1>{{ pageTitle }}</h1>
        </div>
        <div class="topbar-right">
          <span v-if="auth.user">{{ auth.user.username }} ({{ auth.user.role }})</span>
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
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { showToast } from '@/api/client'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const sidebarOpen = ref(false)
const openMenu = ref<string | null>(null)

const pageTitle = computed(() => (route.meta.title as string) || '仪表盘')

function toggleSubmenu(id: string) {
  openMenu.value = openMenu.value === id ? null : id
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
  data: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>',
  content: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>',
  system: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>',
}
</script>
