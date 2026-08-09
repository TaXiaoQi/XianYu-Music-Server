<template>
  <div class="mobile-dashboard">
    <section class="hero-card">
      <div>
        <p>移动端后台</p>
        <h2>数据概览</h2>
        <span>{{ today }}</span>
      </div>
      <button class="copy-btn" @click="copyApiUrl">复制 API</button>
    </section>

    <div v-if="loading" class="state-card">加载中...</div>
    <div v-else-if="loadError" class="state-card error">{{ loadError }}</div>

    <section v-else class="stats-grid">
      <div class="stat-card">
        <span>总用户</span>
        <strong>{{ stats.total_users ?? 0 }}</strong>
        <small>今日新增 {{ stats.today_users ?? 0 }}</small>
      </div>
      <div class="stat-card">
        <span>今日登录</span>
        <strong>{{ stats.today_logins ?? 0 }}</strong>
        <small>总计 {{ stats.total_logins ?? 0 }}</small>
      </div>
      <div class="stat-card">
        <span>今日报错</span>
        <strong>{{ stats.today_errors ?? 0 }}</strong>
        <small>总计 {{ stats.total_errors ?? 0 }}</small>
      </div>
      <div class="stat-card">
        <span>管理员</span>
        <strong>{{ stats.total_admins ?? 0 }}</strong>
        <small>系统管理</small>
      </div>
    </section>

    <section class="mobile-section">
      <h3>常用操作</h3>
      <div class="quick-grid">
        <router-link to="/m/avatar-audit">头像/改名审核</router-link>
        <router-link to="/m/wallpapers">壁纸管理</router-link>
        <router-link to="/m/announcements">公告管理</router-link>
        <router-link to="/m/version">版本管理</router-link>
        <router-link to="/m/email-config">邮箱机设置</router-link>
        <router-link to="/m/database">数据库管理</router-link>
      </div>
    </section>

    <section class="api-card">
      <span>客户端服务器 API</span>
      <code>{{ publicApiUrl }}</code>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface DashboardStats {
  total_users?: number
  today_users?: number
  total_admins?: number
  today_errors?: number
  total_errors?: number
  today_logins?: number
  total_logins?: number
}

const stats = ref<DashboardStats>({})
const loading = ref(true)
const loadError = ref('')
const publicApiUrl = ref('')
const today = new Date().toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'short' })

function resolvePublicApiUrl(): string {
  const { protocol, hostname, port } = window.location
  const apiPort = port === '3000' ? '8081' : port
  const host = apiPort ? `${hostname}:${apiPort}` : hostname
  return `${protocol}//${host}/api`
}

async function copyApiUrl() {
  try {
    await navigator.clipboard.writeText(publicApiUrl.value)
    showToast('服务器 API 已复制', 'success')
  } catch {
    showToast('复制失败')
  }
}

onMounted(async () => {
  publicApiUrl.value = resolvePublicApiUrl()
  const res = await adminApi<DashboardStats>('dashboard_stats')
  if (res.code === 200 && res.data) {
    stats.value = res.data
  } else {
    loadError.value = res.msg || '数据加载失败'
  }
  loading.value = false
})
</script>

<style scoped>
.mobile-dashboard {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.hero-card,
.state-card,
.stat-card,
.api-card,
.mobile-section {
  border: 1px solid var(--border);
  border-radius: 22px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
}
.hero-card {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 14px;
  padding: 20px;
  background:
    linear-gradient(135deg, rgba(236, 65, 65, 0.14), transparent 60%),
    var(--card);
}
.hero-card p {
  margin: 0 0 6px;
  color: var(--text-muted);
  font-size: 12px;
}
.hero-card h2 {
  margin: 0 0 8px;
  font-size: 28px;
  letter-spacing: -0.04em;
}
.hero-card span {
  color: var(--text-light);
  font-size: 12px;
}
.copy-btn {
  flex: 0 0 auto;
  border: none;
  border-radius: 999px;
  padding: 10px 14px;
  background: #EC4141;
  color: #fff;
  font-weight: 800;
}
.state-card {
  padding: 18px;
  color: var(--text-muted);
  text-align: center;
}
.state-card.error {
  color: #EC4141;
}
.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.stat-card {
  padding: 16px;
}
.stat-card span {
  display: block;
  color: var(--text-muted);
  font-size: 12px;
}
.stat-card strong {
  display: block;
  margin: 8px 0 4px;
  font-size: 28px;
  line-height: 1;
}
.stat-card small {
  color: var(--text-light);
  font-size: 11px;
}
.mobile-section {
  padding: 16px;
}
.mobile-section h3 {
  margin: 0 0 12px;
  font-size: 15px;
}
.quick-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.quick-grid a {
  display: flex;
  align-items: center;
  min-height: 48px;
  padding: 0 12px;
  border-radius: 16px;
  background: var(--control-bg);
  color: var(--text);
  text-decoration: none;
  font-size: 13px;
  font-weight: 800;
}
.api-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
}
.api-card span {
  color: var(--text-muted);
  font-size: 12px;
}
.api-card code {
  word-break: break-all;
  color: #EC4141;
  font-size: 12px;
}
</style>
