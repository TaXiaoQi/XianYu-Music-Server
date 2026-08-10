<template>
  <div class="mobile-dashboard">
    <section class="hero-card">
      <div>
        <p>移动端后台</p>
        <h2>数据概览</h2>
        <span>{{ today }}</span>
      </div>
    </section>

    <div v-if="loading" class="state-card">加载中...</div>
    <div v-else-if="loadError" class="state-card error">{{ loadError }}</div>

    <section v-if="!loading && !loadError" class="source-card">
      <div class="source-card-head">
        <div>
          <span>今日音源调用占比</span>
          <strong>{{ stats.today_source_calls ?? 0 }} 次</strong>
        </div>
      </div>
      <div class="source-card-body">
        <div class="source-donut" :style="{ background: sourceRingStyle }">
          <div class="source-donut-inner">{{ stats.today_source_calls ?? 0 }}</div>
        </div>
        <div class="source-list">
          <div v-if="sourceItems.length === 0" class="source-empty">暂无调用数据</div>
          <div v-for="item in sourceItems" :key="item.source_name" class="source-row">
            <i :style="{ background: item.color }"></i>
            <span>{{ item.source_name }}</span>
            <b>{{ item.percent }}%</b>
          </div>
        </div>
      </div>
    </section>

    <section v-if="!loading && !loadError" class="mobile-stats-grid">
      <div class="stat-card">
        <span>总用户</span>
        <strong>{{ stats.total_users ?? 0 }}</strong>
        <small>今日新增 {{ stats.today_users ?? 0 }}</small>
      </div>
      <div class="stat-card">
        <span>今日热搜</span>
        <strong class="hot-keyword">{{ stats.today_hot_search_keyword || '暂无' }}</strong>
        <small>今日搜索 {{ stats.today_hot_search_count ?? 0 }} 次</small>
      </div>
      <div class="stat-card">
        <span>播放过的用户</span>
        <strong>{{ stats.active_users ?? 0 }}</strong>
        <small>累计播放用户</small>
      </div>
    </section>

    <section class="mobile-section">
      <h3>常用操作</h3>
      <div class="quick-grid">
        <router-link to="/m/announcements" class="danger-link">公告管理</router-link>
        <router-link to="/m/version" class="danger-link">版本管理</router-link>
        <router-link to="/m/email-config">邮箱机设置</router-link>
        <router-link to="/m/turnstile-config">人机验证</router-link>
        <router-link to="/m/database">数据库管理</router-link>
      </div>
    </section>

    <section class="api-card">
      <span>客户端服务器 API</span>
      <div class="api-copy-row">
        <code>{{ publicApiUrl }}</code>
        <button class="copy-btn" @click="copyApiUrl">复制 API</button>
      </div>
    </section>

    <section v-if="!loading && !loadError" class="notice-card">
      <div class="notice-head">
        <div>
          <span>消息通知</span>
          <strong>{{ noticeTotal }} 条待处理</strong>
        </div>
      </div>
      <div class="notice-list">
        <router-link v-for="item in noticeItems" :key="item.label" :to="item.to" class="notice-row">
          <i :class="item.className"></i>
          <span>{{ item.label }}</span>
          <b>{{ item.count }}</b>
        </router-link>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface SourceDistributionItem {
  source_name: string
  count: number
}

interface DashboardStats {
  total_users?: number
  today_users?: number
  active_users?: number
  today_source_calls?: number
  source_distribution?: SourceDistributionItem[]
  today_hot_search_keyword?: string
  today_hot_search_count?: number
  pending_wallpapers?: number
  pending_avatars?: number
  pending_nicknames?: number
  pending_feedback?: number
}

const stats = ref<DashboardStats>({})
const loading = ref(true)
const loadError = ref('')
const publicApiUrl = ref('')
const today = new Date().toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'short' })
const sourceColors = ['#EC4141', '#f97316', '#facc15', '#22c55e', '#14b8a6', '#3b82f6', '#8b5cf6', '#ec4899']

const sourceItems = computed(() => {
  const rows = stats.value.source_distribution || []
  const total = rows.reduce((sum, item) => sum + Number(item.count || 0), 0)
  if (total <= 0) return []
  return rows.map((item, index) => ({
    ...item,
    color: sourceColors[index % sourceColors.length],
    percent: Math.round((Number(item.count || 0) / total) * 100),
  }))
})

const sourceRingStyle = computed(() => {
  const items = sourceItems.value
  if (items.length === 0) return 'conic-gradient(var(--track) 0deg 360deg)'
  const total = items.reduce((sum, item) => sum + Number(item.count || 0), 0)
  let start = 0
  const parts = items.map((item) => {
    const angle = total > 0 ? (Number(item.count || 0) / total) * 360 : 0
    const end = start + angle
    const part = `${item.color} ${start}deg ${end}deg`
    start = end
    return part
  })
  return `conic-gradient(${parts.join(', ')})`
})

const noticeItems = computed(() => [
  { label: '新壁纸审核', count: stats.value.pending_wallpapers ?? 0, to: '/m/wallpapers', className: 'wallpaper' },
  { label: '新头像审核', count: stats.value.pending_avatars ?? 0, to: '/m/avatar-audit', className: 'avatar' },
  { label: '新名称审核', count: stats.value.pending_nicknames ?? 0, to: '/m/avatar-audit', className: 'nickname' },
  { label: '新问题反馈', count: stats.value.pending_feedback ?? 0, to: '/m/feedback', className: 'feedback' },
])

const noticeTotal = computed(() => noticeItems.value.reduce((sum, item) => sum + Number(item.count || 0), 0))

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
.source-card {
  border: 1px solid var(--border);
  border-radius: 22px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
  padding: 16px;
}
.source-card-head {
  display: flex;
  justify-content: space-between;
  margin-bottom: 14px;
}
.source-card-head span {
  display: block;
  color: var(--text-muted);
  font-size: 12px;
}
.source-card-head strong {
  display: block;
  margin-top: 5px;
  font-size: 20px;
}
.source-card-body {
  display: grid;
  grid-template-columns: 110px minmax(0, 1fr);
  gap: 14px;
  align-items: center;
}
.source-donut {
  width: 108px;
  height: 108px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  box-shadow: inset 0 0 0 1px var(--border);
}
.source-donut-inner {
  width: 66px;
  height: 66px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: var(--card-solid, var(--card));
  color: var(--text);
  font-size: 18px;
  font-weight: 900;
  box-shadow: var(--shadow-soft);
}
.source-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.source-row {
  display: grid;
  grid-template-columns: 8px minmax(0, 1fr) auto;
  align-items: center;
  gap: 7px;
  color: var(--text-light);
  font-size: 12px;
}
.source-row i {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.source-row span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.source-row b {
  color: #EC4141;
}
.source-empty {
  color: var(--text-muted);
  font-size: 12px;
}
.mobile-stats-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.stat-card {
  min-width: 0;
  padding: 12px 10px;
}
.stat-card span {
  display: block;
  color: var(--text-muted);
  font-size: 11px;
}
.stat-card strong {
  display: block;
  margin: 7px 0 4px;
  font-size: 22px;
  line-height: 1;
}
.stat-card small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-light);
  font-size: 10px;
}
.hot-keyword {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 18px;
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
.quick-grid a.danger-link {
  background: #EC4141;
  color: #fff;
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
.api-copy-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.api-card code {
  flex: 1;
  min-width: 0;
  word-break: break-all;
  color: #EC4141;
  font-size: 12px;
}
.notice-card {
  border: 1px solid var(--border);
  border-radius: 22px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
  padding: 16px;
}
.notice-head {
  margin-bottom: 12px;
}
.notice-head span {
  display: block;
  color: var(--text-muted);
  font-size: 12px;
}
.notice-head strong {
  display: block;
  margin-top: 5px;
  font-size: 20px;
}
.notice-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.notice-row {
  display: grid;
  grid-template-columns: 9px minmax(0, 1fr) auto;
  gap: 9px;
  align-items: center;
  min-height: 42px;
  padding: 0 12px;
  border-radius: 14px;
  background: var(--control-bg);
  color: var(--text);
  text-decoration: none;
  font-size: 13px;
  font-weight: 800;
}
.notice-row i {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #EC4141;
}
.notice-row i.wallpaper {
  background: #3b82f6;
}
.notice-row i.avatar {
  background: #22c55e;
}
.notice-row i.nickname {
  background: #f97316;
}
.notice-row i.feedback {
  background: #8b5cf6;
}
.notice-row b {
  color: #EC4141;
  font-size: 18px;
}
</style>
