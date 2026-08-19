<template>
  <div class="email-home-page">
    <!-- Header -->
    <header class="page-header">
      <div class="header-brand">
        <div class="brand-icon">M</div>
        <span class="brand-name">弦予邮箱</span>
      </div>
      <button class="logout-btn" @click="handleLogout">退出登录</button>
    </header>

    <!-- Main -->
    <main class="page-main" v-if="profile">
      <!-- Profile Card -->
      <section class="profile-card" :style="{ animationDelay: '0ms' }">
        <div class="avatar" :style="{ background: avatarColor }">
          {{ avatarLetter }}
        </div>
        <div class="profile-info">
          <h2>{{ profile.nickname || '未设置昵称' }}</h2>
          <p class="email-text">{{ profile.email }}</p>
          <div class="profile-meta">
            <span class="meta-item">
              <span class="meta-label">注册时间</span>
              <span class="meta-value">{{ formatDate(profile.created_at) }}</span>
            </span>
            <span class="meta-item">
              <span class="meta-label">最后登录</span>
              <span class="meta-value">{{ profile.last_login ? formatDate(profile.last_login) : '—' }}</span>
            </span>
          </div>
        </div>
        <div class="status-badge" :class="{ active: profile.status === 1 }">
          {{ profile.status === 1 ? '正常' : '禁用' }}
        </div>
      </section>

      <!-- Activity Logs -->
      <section class="logs-card" :style="{ animationDelay: '80ms' }">
        <h3 class="section-title">最近活动</h3>
        <div v-if="profile.logs && profile.logs.length > 0" class="logs-list">
          <div
            v-for="(log, idx) in profile.logs"
            :key="idx"
            class="log-item"
            :style="{ animationDelay: `${120 + idx * 50}ms` }"
          >
            <div class="log-action" :class="actionClass(log.action)">
              {{ actionLabel(log.action) }}
            </div>
            <div class="log-detail">
              <span class="log-ip">{{ log.detail || '—' }}</span>
              <span class="log-time">{{ formatDate(log.created_at) }}</span>
            </div>
          </div>
        </div>
        <div v-else class="empty-state">暂无活动记录</div>
      </section>
    </main>

    <!-- Loading -->
    <main class="page-main" v-else>
      <div class="loading-state">
        <div class="loading-spinner"></div>
        <p>加载中...</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { emailGetProfile, getEmailToken, clearEmailToken, emailToast, type EmailProfile } from '@/api/email'
import { fmtDateTime } from '@/utils/time'

const router = useRouter()
const profile = ref<EmailProfile | null>(null)

const avatarLetter = computed(() => {
  if (!profile.value) return '?'
  const name = profile.value.nickname || profile.value.email
  return name.charAt(0).toUpperCase()
})

const avatarColor = computed(() => {
  if (!profile.value) return '#1a1a1a'
  const colors = ['#1a1a1a', '#2d4a22', '#1a3a5c', '#4a2d1a', '#3a1a4a']
  const hash = profile.value.email.split('').reduce((a, c) => a + c.charCodeAt(0), 0)
  return colors[hash % colors.length]
})

function actionLabel(action: string): string {
  const map: Record<string, string> = {
    register: '注册',
    login: '登录',
    reset_password: '重置密码',
  }
  return map[action] || action
}

function actionClass(action: string): string {
  const map: Record<string, string> = {
    register: 'tag-green',
    login: 'tag-blue',
    reset_password: 'tag-orange',
  }
  return map[action] || 'tag-gray'
}

function formatDate(s: string): string {
  if (!s) return '—'
  return fmtDateTime(s).substring(0, 16)
}

async function loadProfile() {
  if (!getEmailToken()) {
    router.replace('/email/login')
    return
  }
  const res = await emailGetProfile()
  if (res.code === 200 && res.data) {
    profile.value = res.data
  } else if (res.code === 401) {
    clearEmailToken()
    emailToast('登录已过期，请重新登录')
    setTimeout(() => router.replace('/email/login'), 1000)
  } else {
    emailToast(res.msg || '加载失败')
  }
}

function handleLogout() {
  clearEmailToken()
  emailToast('已退出登录', 'success')
  router.replace('/email/login')
}

onMounted(loadProfile)
</script>

<style scoped>
.email-home-page {
  min-height: 100vh;
  background: var(--track);
}

/* Header */
.page-header {
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  padding: 14px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: sticky;
  top: 0;
  z-index: 10;
}
.header-brand {
  display: flex;
  align-items: center;
  gap: 10px;
}
.brand-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #1a1a1a;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  font-weight: 700;
}
.brand-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-light);
}
.logout-btn {
  padding: 8px 18px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background: #fff;
  color: #666;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.logout-btn:hover {
  border-color: #c00;
  color: #c00;
  background: rgba(236, 65, 65, 0.10);
}

/* Main */
.page-main {
  max-width: 720px;
  margin: 0 auto;
  padding: 32px 20px;
}

/* Profile Card */
.profile-card {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 12px;
  padding: 28px;
  display: flex;
  align-items: center;
  gap: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  animation: cardIn 0.5s ease both;
  position: relative;
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}
.avatar {
  width: 60px;
  height: 60px;
  border-radius: 14px;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 26px;
  font-weight: 700;
  flex-shrink: 0;
}
.profile-info {
  flex: 1;
  min-width: 0;
}
.profile-info h2 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-light);
  margin: 0 0 4px;
}
.email-text {
  font-size: 13px;
  color: #999;
  margin: 0 0 12px;
  word-break: break-all;
}
.profile-meta {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}
.meta-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.meta-label {
  font-size: 11px;
  color: #bbb;
}
.meta-value {
  font-size: 13px;
  color: #666;
  font-weight: 500;
}
.status-badge {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 3px 10px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  background: var(--track);
  color: #999;
}
.status-badge.active {
  background: #f0faf3;
  color: #1a7a3a;
}

/* Logs Card */
.logs-card {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 12px;
  padding: 24px;
  margin-top: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  animation: cardIn 0.5s ease both;
}
.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-light);
  margin: 0 0 16px;
}
.logs-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}
.log-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 0;
  border-bottom: 1px solid #f5f5f5;
  animation: logIn 0.4s ease both;
}
.log-item:last-child {
  border-bottom: none;
}
@keyframes logIn {
  from { opacity: 0; transform: translateX(-10px); }
  to { opacity: 1; transform: translateX(0); }
}
.log-action {
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  min-width: 50px;
  text-align: center;
}
.tag-green { background: #f0faf3; color: #1a7a3a; }
.tag-blue { background: #f0f5ff; color: #1a4a8a; }
.tag-orange { background: rgba(245, 158, 11, 0.12); color: #b8651a; }
.tag-gray { background: #f5f5f5; color: #666; }

.log-detail {
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.log-ip {
  font-size: 13px;
  color: #999;
}
.log-time {
  font-size: 12px;
  color: #ccc;
}

.empty-state {
  text-align: center;
  color: #ccc;
  font-size: 14px;
  padding: 32px 0;
}

/* Loading */
.loading-state {
  text-align: center;
  padding: 60px 0;
  color: #999;
}
.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e0e0e0;
  border-top-color: var(--text-light);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  margin: 0 auto 12px;
}
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 600px) {
  .page-main { padding: 20px 14px; }
  .profile-card {
    flex-direction: column;
    text-align: center;
    padding: 24px 18px;
  }
  .profile-meta {
    justify-content: center;
  }
  .status-badge {
    position: static;
    margin-top: 8px;
  }
}
</style>
