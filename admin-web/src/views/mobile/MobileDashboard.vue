<template>
  <div class="mobile-dashboard">
    <!-- 标题区（扁平，无卡片） -->
    <div class="dsh-head">
      <div class="dsh-head-l">
        <div class="dsh-head-label">Dashboard Overview</div>
        <div class="dsh-head-title">数据<em>概览</em></div>
        <div class="dsh-head-date">{{ today }}</div>
      </div>
      <div class="dsh-head-r">
        <span class="dsh-live-dot"></span>
        <span>实时同步</span>
      </div>
    </div>

    <div v-if="loading" class="state-text">加载中...</div>
    <div v-else-if="loadError" class="state-text error">{{ loadError }}</div>

    <!-- 今日音源调用占比（扁平，无卡片包裹） -->
    <div v-if="!loading && !loadError" class="source-section">
      <div class="source-head">
        <div>
          <h3>今日音源调用占比</h3>
          <p>按不同音源的今日调用次数统计</p>
        </div>
        <span class="source-total">总调用 {{ stats.today_source_calls ?? 0 }} 次</span>
      </div>
      <div class="source-body">
        <div class="source-donut" :style="{ background: sourceRingStyle }">
          <div class="source-donut-center">
            <strong>{{ stats.today_source_calls ?? 0 }}</strong>
            <span>今日调用</span>
          </div>
        </div>
        <div class="source-legend">
          <div v-if="sourceItems.length === 0" class="source-empty">今日暂无音源调用数据</div>
          <div v-for="item in sourceItems" :key="item.source_name" class="source-legend-item">
            <i :style="{ background: item.color }"></i>
            <span class="source-name">{{ item.source_name }}</span>
            <span class="source-count">{{ item.count }} 次</span>
            <span class="source-percent">{{ item.percent }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 统计卡片（扁平，无单独卡片包裹） -->
    <div v-if="!loading && !loadError" class="stats-grid">
      <div class="stat-item">
        <div class="stat-label">总用户数</div>
        <div class="stat-value">{{ stats.total_users ?? 0 }}</div>
        <div class="stat-sub">今日新增 {{ stats.today_users ?? 0 }} · 昨日 {{ stats.yesterday_users ?? 0 }}</div>
      </div>
      <div class="stat-item">
        <div class="stat-label">今日热搜</div>
        <div class="stat-value hot-keyword">{{ stats.today_hot_search_keyword || '暂无' }}</div>
        <div class="stat-sub">今日搜索 {{ stats.today_hot_search_count ?? 0 }} 次</div>
      </div>
      <div class="stat-item">
        <div class="stat-label">今日用户</div>
        <div class="stat-value">{{ stats.active_users ?? 0 }}</div>
        <div class="stat-sub">今日活跃设备数</div>
      </div>
      <div class="stat-item">
        <div class="stat-label">今日分享</div>
        <div class="stat-value">{{ stats.today_shares ?? 0 }}</div>
        <div class="stat-sub">总计 {{ stats.total_shares ?? 0 }} 次</div>
      </div>
    </div>

    <!-- 消息通知 -->
    <div v-if="!loading && !loadError" class="notice-section">
      <div class="notice-head">
        <div>
          <h3>消息通知</h3>
          <p>待处理的审核与反馈会在这里汇总显示</p>
        </div>
        <span class="notice-total">{{ noticeTotal }} 条待处理</span>
      </div>
      <div class="notice-list">
        <router-link v-for="item in noticeItems" :key="item.label" :to="item.to" class="notice-item">
          <span class="notice-dot" :class="item.className"></span>
          <div class="notice-text">
            <strong>{{ item.label }}</strong>
            <small>{{ item.desc }}</small>
          </div>
          <b>{{ item.count }}</b>
        </router-link>
      </div>
    </div>

    <!-- 常用操作 -->
    <div class="quick-section">
      <h3>常用操作</h3>
      <div class="quick-grid">
        <router-link to="/m/announcements" class="mobile-btn primary">公告管理</router-link>
        <router-link to="/m/version" class="mobile-btn primary">版本管理</router-link>
        <router-link to="/m/email-config" class="mobile-btn">邮箱机设置</router-link>
        <router-link to="/m/turnstile-config" class="mobile-btn">人机验证</router-link>
        <router-link to="/m/database" class="mobile-btn">数据库管理</router-link>
      </div>
    </div>

    <!-- 服务器 API -->
    <div class="api-section">
      <div class="api-head">
        <div>
          <h3>服务器 API</h3>
          <p>客户端"服务器 API / 接口地址"请填写下面这个地址</p>
        </div>
      </div>
      <div class="api-copy-row">
        <code>{{ publicApiUrl }}</code>
        <button class="mobile-btn primary" @click="copyApiUrl">复制 API</button>
      </div>
      <div class="api-copy-row api-secret-row">
        <code>{{ clientApiSecret || '未读取到客户端 API 签名密钥' }}</code>
        <button class="mobile-btn primary" :disabled="!clientApiSecret" @click="copyApiSecret">复制密钥</button>
      </div>
      <p class="api-hint">客户端会自动拼接 <code>?action=xxx</code>，所以服务器 API 只需要填到 <code>/api</code>。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

interface SourceDistributionItem {
  source_name: string
  count: number
}

interface DashboardStats {
  total_users?: number
  today_users?: number
  yesterday_users?: number
  active_users?: number
  today_source_calls?: number
  total_shares?: number
  today_shares?: number
  source_distribution?: SourceDistributionItem[]
  today_hot_search_keyword?: string
  today_hot_search_count?: number
  pending_wallpapers?: number
  pending_avatars?: number
  pending_nicknames?: number
  pending_feedback?: number
  api_secret?: string
}

const stats = ref<DashboardStats>({})
const loading = ref(true)
const loadError = ref('')
const publicApiUrl = ref('')
const today = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
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
  { label: '新壁纸审核', desc: '用户上传壁纸待审核', count: stats.value.pending_wallpapers ?? 0, to: '/m/wallpapers', className: 'wallpaper' },
  { label: '新头像审核', desc: '用户头像变更待审核', count: stats.value.pending_avatars ?? 0, to: '/m/avatar-audit', className: 'avatar' },
  { label: '新名称审核', desc: '用户改名申请待审核', count: stats.value.pending_nicknames ?? 0, to: '/m/avatar-audit', className: 'nickname' },
  { label: '新问题反馈', desc: '用户反馈待处理', count: stats.value.pending_feedback ?? 0, to: '/m/feedback', className: 'feedback' },
])

const noticeTotal = computed(() => noticeItems.value.reduce((sum, item) => sum + Number(item.count || 0), 0))

const clientApiSecret = computed(() => normalizeApiSecret(stats.value.api_secret || ''))

function resolvePublicApiUrl(): string {
  const { protocol, hostname, port } = window.location
  const apiPort = port === '3000' ? '8081' : port
  const host = apiPort ? `${hostname}:${apiPort}` : hostname
  return `${protocol}//${host}/api`
}

function normalizeApiSecret(value: string): string {
  const text = String(value || '').trim()
  if (!text) return ''
  const line = text.split(/\r?\n/).find((item) => /^api_secret\b/i.test(item.trim()))
  const source = line || text
  const match = source.match(/^api_secret\s*[:=]?\s*["']?([^"'\s]+)["']?/i)
  return (match?.[1] || source).trim()
}

async function copyText(text: string, successMessage: string) {
  try {
    await navigator.clipboard.writeText(text)
    showToast(successMessage, 'success')
  } catch {
    const input = document.createElement('input')
    input.value = text
    document.body.appendChild(input)
    input.select()
    document.execCommand('copy')
    input.remove()
    showToast(successMessage, 'success')
  }
}

async function copyApiUrl() {
  await copyText(publicApiUrl.value, '服务器 API 已复制')
}

async function copyApiSecret() {
  if (!clientApiSecret.value) {
    showToast('未读取到客户端 API 签名密钥')
    return
  }
  await copyText(clientApiSecret.value, '客户端 API 签名密钥已复制')
}

onMounted(async () => {
  publicApiUrl.value = resolvePublicApiUrl()
  const res = await adminApi<DashboardStats>('dashboard_stats')
  if (res.code === 200 && res.data) {
    stats.value = res.data
  } else {
    loadError.value = res.msg || '数据加载失败（数据库可能未连接）'
  }
  loading.value = false
})
</script>

<style scoped>
.mobile-dashboard {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 标题区 — 扁平，无卡片 */
.dsh-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}
.dsh-head-label {
  font-size: 10px;
  letter-spacing: 0.25em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 500;
  margin-bottom: 4px;
}
.dsh-head-title {
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1;
  color: var(--accent);
}
.dsh-head-title em {
  font-style: italic;
  font-weight: 300;
  color: var(--accent);
}
.dsh-head-date {
  font-size: 12px;
  color: var(--text-soft);
  margin-top: 8px;
}
.dsh-head-r {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-light);
  white-space: nowrap;
}
.dsh-live-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
  position: relative;
}
.dsh-live-dot::after {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  background: var(--accent);
  opacity: 0.3;
  animation: dshPulse 2s ease-out infinite;
}
@keyframes dshPulse {
  0% { transform: scale(1); opacity: 0.4; }
  100% { transform: scale(2.5); opacity: 0; }
}

.state-text {
  padding: 24px 14px;
  color: var(--text-muted);
  text-align: center;
  font-size: 13px;
}
.state-text.error {
  color: #EC4141;
}

/* 今日音源调用占比 — 扁平，无卡片包裹 */
.source-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.source-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.source-head h3 {
  margin: 0 0 4px;
  font-size: 15px;
  color: var(--text);
}
.source-head p {
  margin: 0;
  color: var(--text-light);
  font-size: 12px;
}
.source-total {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 800;
}
.source-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  align-items: center;
}
.source-donut {
  width: 130px;
  height: 130px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  box-shadow: inset 0 0 0 1px var(--border);
}
.source-donut-center {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  align-content: center;
  background: var(--card-solid, var(--card));
  box-shadow: var(--shadow-soft);
}
.source-donut-center strong {
  font-size: 22px;
  line-height: 1;
  color: var(--text);
}
.source-donut-center span {
  margin-top: 3px;
  color: var(--text-muted);
  font-size: 10px;
}
.source-legend {
  display: flex;
  flex-direction: column;
  gap: 7px;
  width: 100%;
}
.source-legend-item {
  display: grid;
  grid-template-columns: 8px minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 7px;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  font-size: 12px;
}
.source-legend-item i {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.source-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text);
  font-weight: 700;
}
.source-count {
  color: var(--text-light);
  font-size: 11px;
}
.source-percent {
  color: var(--accent);
  font-size: 11px;
  font-weight: 800;
}
.source-empty {
  padding: 14px;
  border: 1px dashed var(--border);
  border-radius: 12px;
  text-align: center;
  color: var(--text-light);
  font-size: 12px;
}

/* 统计卡片 — 扁平，无单独卡片包裹 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1px;
  background: var(--border);
  border: 1px solid var(--border);
  border-radius: 16px;
  overflow: hidden;
}
.stat-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px 14px;
  background: var(--card);
  min-width: 0;
}
.stat-label {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 700;
}
.stat-value {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
  color: var(--accent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.hot-keyword {
  font-size: 20px;
}
.stat-sub {
  font-size: 10px;
  color: var(--text-light);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 消息通知 — 扁平列表 */
.notice-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.notice-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}
.notice-head h3 {
  margin: 0 0 4px;
  font-size: 15px;
  color: var(--text);
}
.notice-head p {
  margin: 0;
  color: var(--text-light);
  font-size: 12px;
}
.notice-total {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 800;
}
.notice-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: var(--border);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.notice-item {
  display: grid;
  grid-template-columns: 8px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  padding: 13px 14px;
  background: var(--card);
  color: var(--text);
  text-decoration: none;
}
.notice-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
}
.notice-dot.wallpaper { background: #3b82f6; }
.notice-dot.avatar { background: #22c55e; }
.notice-dot.nickname { background: #f97316; }
.notice-dot.feedback { background: #8b5cf6; }
.notice-text {
  min-width: 0;
}
.notice-text strong {
  display: block;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.notice-text small {
  display: block;
  margin-top: 3px;
  color: var(--text-light);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.notice-item b {
  color: var(--accent);
  font-size: 18px;
}

/* 常用操作 */
.quick-section h3 {
  margin: 0 0 12px;
  font-size: 15px;
  color: var(--text);
}
.quick-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.quick-grid .mobile-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 44px;
  text-decoration: none;
}

/* 服务器 API */
.api-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.api-head h3 {
  margin: 0 0 4px;
  font-size: 15px;
  color: var(--text);
}
.api-head p {
  margin: 0;
  color: var(--text-light);
  font-size: 12px;
}
.api-copy-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--control-bg);
}
.api-copy-row code {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text);
  font-size: 12px;
}
.api-secret-row {
  margin-top: 0;
}
.api-hint {
  color: var(--text-light);
  font-size: 11px;
  line-height: 1.6;
}
.api-hint code {
  color: var(--accent);
}
</style>
