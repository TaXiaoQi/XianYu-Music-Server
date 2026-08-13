<template>
  <div class="dashboard-wrap">
    <!-- 标题区 -->
    <Transition name="fade-down" appear>
    <div class="dsh-head">
      <div class="dsh-head-l">
        <div class="dsh-head-label">Dashboard Overview</div>
        <div class="dsh-head-title">数据<em>概览</em></div>
        <div class="dsh-head-date">{{ today }} · 弦予音乐后台</div>
      </div>
      <div class="dsh-head-r">
        <span class="dsh-live-dot"></span>
        <span>实时同步</span>
      </div>
    </div>
    </Transition>

    <!-- 今日音源调用占比 -->
    <Transition name="fade-up" appear>
    <div class="source-chart-card" v-if="!loading && !loadError">
      <div class="source-chart-head">
        <div>
          <h3>今日音源调用占比</h3>
          <p>按不同音源的今日调用次数统计</p>
        </div>
        <span class="source-total">总调用 {{ stats.today_source_calls ?? 0 }} 次</span>
      </div>
      <div class="source-chart-body">
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
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
    <div class="stats-grid" v-if="!loading && !loadError">
      <div v-for="(card, idx) in statCards" :key="card.label" class="stat-card" :style="{ animationDelay: `${idx * 80}ms` }">
        <div class="label">{{ card.label }}</div>
        <div class="value" :class="{ 'hot-keyword': card.hot }">{{ card.value() }}</div>
        <div class="sub">{{ card.sub() }}</div>
      </div>
    </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="empty">加载中...</div>
    <div v-if="!loading && loadError" class="empty">{{ loadError }}</div>

    <!-- 快捷操作 -->
    <Transition name="fade-up" appear>
    <div class="card" style="margin-top: 20px;">
      <h3>快捷操作</h3>
      <div style="display: flex; gap: 12px; flex-wrap: wrap;">
        <router-link to="/announcements" class="btn btn-primary">公告管理</router-link>
        <router-link to="/version" class="btn btn-primary">版本管理</router-link>
        <router-link to="/about-config" class="btn">关于页设置</router-link>
        <router-link to="/email-config" class="btn">邮箱机管理</router-link>
        <router-link to="/turnstile-config" class="btn">人机验证</router-link>
        <router-link to="/database" class="btn">数据库管理</router-link>
      </div>
    </div>
    </Transition>

    <!-- 消息通知 -->
    <Transition name="fade-up" appear>
    <div class="notice-card" v-if="!loading && !loadError">
      <div class="notice-head">
        <div>
          <h3>消息通知</h3>
          <p>待处理的审核与反馈会在这里汇总显示</p>
        </div>
        <span class="notice-total">{{ noticeTotal }} 条待处理</span>
      </div>
      <div class="notice-grid">
        <router-link v-for="(item, idx) in noticeItems" :key="item.label" :to="item.to" class="notice-item" :style="{ animationDelay: `${idx * 70}ms` }">
          <span class="notice-dot" :class="item.className"></span>
          <div class="notice-text">
            <strong>{{ item.label }}</strong>
            <small>{{ item.desc }}</small>
          </div>
          <b>{{ item.count }}</b>
        </router-link>
      </div>
    </div>
    </Transition>

    <!-- 服务器 API -->
    <Transition name="fade-up" appear>
    <div class="card api-card">
      <div class="api-card-head">
        <div>
          <h3>服务器 API</h3>
          <p>客户端“服务器 API / 接口地址”请填写下面这个地址，用于连接当前服务器。</p>
        </div>
        <span class="api-chip">客户端填写</span>
      </div>
      <div class="api-copy-row">
        <code>{{ publicApiUrl }}</code>
        <button class="btn btn-primary btn-sm" @click="copyApiUrl">复制 API</button>
      </div>
      <div class="api-copy-row api-secret-row">
        <code>{{ clientApiSecret || '未读取到客户端 API 签名密钥' }}</code>
        <button class="btn btn-primary btn-sm" :disabled="!clientApiSecret" @click="copyApiSecret">复制密钥</button>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface SourceDistributionItem {
  source_name: string
  count: number
}

interface DashboardStats {
  total_users?: number
  today_users?: number
  yesterday_users?: number
  total_source_calls?: number
  today_source_calls?: number
  yesterday_source_calls?: number
  today_source_success?: number
  total_source_success?: number
  total_shares?: number
  today_shares?: number
  yesterday_shares?: number
  active_users?: number
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
const sourceColors = ['#EC4141', '#f97316', '#facc15', '#22c55e', '#14b8a6', '#3b82f6', '#8b5cf6', '#ec4899']

const today = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })

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

// 统计卡片配置（支持动态值）
const statCards = [
  {
    label: '总用户数',
    key: 'total_users',
    sub: () => `今日新增 ${stats.value.today_users ?? 0} · 昨日 ${stats.value.yesterday_users ?? 0}`,
    value: () => stats.value.total_users ?? 0,
  },
  {
    label: '今日热搜',
    key: 'today_hot_search_keyword',
    value: () => stats.value.today_hot_search_keyword || '暂无',
    sub: () => `今日搜索 ${stats.value.today_hot_search_count ?? 0} 次`,
    hot: true,
  },
  {
    label: '今日用户',
    key: 'active_users',
    value: () => stats.value.active_users ?? 0,
    sub: () => '今日活跃设备数',
  },
  {
    label: '今日分享',
    key: 'today_shares',
    value: () => stats.value.today_shares ?? 0,
    sub: () => `总计 ${stats.value.total_shares ?? 0} 次`,
  },
]

const sourceRingStyle = computed(() => {
  const items = sourceItems.value
  if (items.length === 0) {
    return 'conic-gradient(var(--track) 0deg 360deg)'
  }
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
  {
    label: '新壁纸审核',
    desc: '用户上传壁纸待审核',
    count: stats.value.pending_wallpapers ?? 0,
    to: '/wallpapers',
    className: 'wallpaper',
  },
  {
    label: '新头像审核',
    desc: '用户头像变更待审核',
    count: stats.value.pending_avatars ?? 0,
    to: '/avatar-audit',
    className: 'avatar',
  },
  {
    label: '新名称审核',
    desc: '用户改名申请待审核',
    count: stats.value.pending_nicknames ?? 0,
    to: '/avatar-audit',
    className: 'nickname',
  },
  {
    label: '新问题反馈',
    desc: '用户反馈待处理',
    count: stats.value.pending_feedback ?? 0,
    to: '/feedback',
    className: 'feedback',
  },
])

const noticeTotal = computed(() => noticeItems.value.reduce((sum, item) => sum + Number(item.count || 0), 0))

const clientApiSecret = computed(() => normalizeApiSecret(stats.value.api_secret || ''))

function resolvePublicApiUrl(): string {
  const { protocol, hostname, port } = window.location
  const apiPort = port === '3000' ? '8081' : port
  const host = apiPort ? `${hostname}:${apiPort}` : hostname
  return `${protocol}//${host}/api`
}

async function copyApiUrl() {
  await copyText(publicApiUrl.value, '服务器 API 已复制')
}

async function copyApiSecret() {
  if (!clientApiSecret.value) {
    showToast('未读取到客户端 API 签名密钥', 'error')
    return
  }
  await copyText(clientApiSecret.value, '客户端 API 签名密钥已复制')
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
.dashboard-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

/* 标题区 */
.dsh-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 28px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}
.dsh-head-label {
  font-size: 11px;
  letter-spacing: 0.25em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 500;
  margin-bottom: 4px;
}
.dsh-head-title {
  font-size: 32px;
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
  gap: 8px;
  font-size: 12px;
  color: var(--text-light);
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

.api-card {
  margin-top: 20px;
}

.source-chart-card {
  margin-bottom: 20px;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: 20px;
  background: var(--card, var(--white));
  box-shadow: var(--shadow-soft, 0 10px 30px rgba(0, 0, 0, 0.04));
}

.source-chart-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 18px;
}

.source-chart-head h3 {
  margin: 0 0 6px;
  font-size: 17px;
  color: var(--text);
}

.source-chart-head p {
  margin: 0;
  color: var(--text-light);
  font-size: 13px;
}

.source-total {
  flex-shrink: 0;
  padding: 6px 12px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
  font-weight: 800;
}

.source-chart-body {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 24px;
  align-items: center;
}

.source-donut {
  width: 200px;
  height: 200px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  box-shadow: inset 0 0 0 1px var(--border);
}

.source-donut-center {
  width: 122px;
  height: 122px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  align-content: center;
  background: var(--card-solid, var(--card));
  box-shadow: var(--shadow-soft);
}

.source-donut-center strong {
  font-size: 30px;
  line-height: 1;
  color: var(--text);
}

.source-donut-center span {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

.source-legend {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.source-legend-item {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 9px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg, #fafafa);
  font-size: 13px;
}

.source-legend-item i {
  width: 10px;
  height: 10px;
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

.source-count,
.source-percent,
.source-empty {
  color: var(--text-light);
  font-size: 12px;
}

.source-percent {
  color: var(--accent);
  font-weight: 800;
}

.source-empty {
  grid-column: 1 / -1;
  padding: 18px;
  border: 1px dashed var(--border);
  border-radius: 14px;
  text-align: center;
}

.hot-keyword {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 28px;
}

.api-card-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.api-card-head h3 {
  margin-bottom: 6px;
}

.api-card-head p,
.api-hint {
  color: var(--text-light);
  font-size: 13px;
  line-height: 1.7;
}

.api-chip {
  flex-shrink: 0;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
  font-weight: 700;
}

.api-copy-row {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px;
  border-radius: 14px;
  background: var(--control-bg);
  border: 1px solid var(--border);
}

.api-copy-row code {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text);
  font-size: 14px;
}

.api-secret-row {
  margin-top: 10px;
}

.api-hint {
  margin-top: 10px;
}

.api-hint code {
  color: var(--accent);
}

.notice-card {
  margin-top: 20px;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: 20px;
  background: var(--card, var(--white));
  box-shadow: var(--shadow-soft, 0 10px 30px rgba(0, 0, 0, 0.04));
}

.notice-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
}

.notice-head h3 {
  margin: 0 0 6px;
  color: var(--text);
  font-size: 17px;
}

.notice-head p {
  margin: 0;
  color: var(--text-light);
  font-size: 13px;
}

.notice-total {
  flex-shrink: 0;
  padding: 6px 12px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
  font-weight: 800;
}

.notice-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.notice-item {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--control-bg, #fafafa);
  color: var(--text);
  text-decoration: none;
  transition: transform 0.2s, border-color 0.2s;
}

.notice-item:hover {
  transform: translateY(-1px);
  border-color: var(--accent);
}

.notice-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--accent);
}

.notice-dot.wallpaper {
  background: #3b82f6;
}

.notice-dot.avatar {
  background: #22c55e;
}

.notice-dot.nickname {
  background: #f97316;
}

.notice-dot.feedback {
  background: #8b5cf6;
}

.notice-text {
  min-width: 0;
}

.notice-text strong,
.notice-text small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notice-text strong {
  font-size: 14px;
}

.notice-text small {
  margin-top: 4px;
  color: var(--text-light);
  font-size: 12px;
}

.notice-item b {
  color: var(--accent);
  font-size: 22px;
}

@media (max-width: 640px) {
  .source-chart-head,
  .notice-head,
  .api-card-head,
  .api-copy-row {
    flex-direction: column;
    align-items: stretch;
  }
  .source-chart-body {
    grid-template-columns: 1fr;
  }
  .source-donut {
    margin: 0 auto;
  }
  .source-legend {
    grid-template-columns: 1fr;
  }
  .notice-grid {
    grid-template-columns: 1fr;
  }
}

/* ===== 动效 ===== */
.stat-card, .notice-item {
  animation: dashIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes dashIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
