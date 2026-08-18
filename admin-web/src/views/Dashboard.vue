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
        <div class="source-donut" :class="{ 'has-active': activeIndex >= 0 }">
          <svg viewBox="0 0 200 200" class="donut-svg">
            <circle class="donut-track" cx="100" cy="100" :r="DONUT_R" />
            <circle
              v-for="seg in donutSegments"
              :key="seg.source_name"
              class="donut-seg"
              :cx="100" :cy="100" :r="DONUT_R"
              :stroke="seg.color"
              :stroke-dasharray="seg.dasharray"
              :stroke-dashoffset="seg.dashoffset"
              :class="{ 'seg-active': activeIndex === seg.index }"
              @mouseenter="onLegendHover(seg.index)"
              @mouseleave="activeIndex = -1"
            />
          </svg>
          <div class="source-donut-center">
            <template v-if="activeIndex >= 0 && activeItem">
              <strong class="center-name">{{ activeItem.source_name }}</strong>
              <span>{{ activeItem.count }} 次 · {{ activeItem.percent }}%</span>
            </template>
            <template v-else>
              <strong>{{ stats.today_source_calls ?? 0 }}</strong>
              <span>今日调用</span>
            </template>
          </div>
        </div>
        <div ref="legendRef" class="source-legend">
          <div v-if="sourceItems.length === 0" class="source-empty">今日暂无音源调用数据</div>
          <div
            v-for="(item, idx) in sourceItems"
            :key="item.source_name"
            class="source-legend-item"
            :class="{ 'legend-active': activeIndex === idx }"
            @mouseenter="onLegendHover(idx)"
            @mouseleave="activeIndex = -1"
          >
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
        <router-link to="/turnstile-config" class="btn">审核设置</router-link>
        <router-link to="/database" class="btn">数据库管理</router-link>
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
  {
    label: '今日热搜',
    key: 'today_hot_search_keyword',
    value: () => stats.value.today_hot_search_keyword || '暂无',
    sub: () => `今日搜索 ${stats.value.today_hot_search_count ?? 0} 次`,
    hot: true,
  },
]

// ─── SVG 圆环分段 ───────────────────────────────────────────
const DONUT_R = 84
const DONUT_C = 2 * Math.PI * DONUT_R
const activeIndex = ref(-1)
const legendRef = ref<HTMLElement | null>(null)

/** 悬停/点击联动：设置高亮并滚动列表项到可见位置 */
function onLegendHover(index: number) {
  activeIndex.value = index
  const container = legendRef.value
  if (!container) return
  const item = container.querySelectorAll('.source-legend-item')[index] as HTMLElement | undefined
  if (!item) return
  const cTop = container.scrollTop
  const cBottom = cTop + container.clientHeight
  const iTop = item.offsetTop
  const iBottom = iTop + item.offsetHeight
  if (iTop < cTop) {
    container.scrollTop = iTop
  } else if (iBottom > cBottom) {
    container.scrollTop = iBottom - container.clientHeight
  }
}

const donutSegments = computed(() => {
  const items = sourceItems.value
  const total = items.reduce((sum, item) => sum + Number(item.count || 0), 0)
  if (total <= 0) return []
  let acc = 0
  return items.map((item, index) => {
    const len = total > 0 ? (Number(item.count || 0) / total) * DONUT_C : 0
    const seg = {
      ...item,
      index,
      dasharray: `${len} ${DONUT_C - len}`,
      dashoffset: -acc,
    }
    acc += len
    return seg
  })
})

const activeItem = computed(() => {
  if (activeIndex.value < 0) return null
  return sourceItems.value[activeIndex.value] || null
})

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
  position: relative;
  width: 200px;
  height: 200px;
  display: grid;
  place-items: center;
}

.donut-svg {
  width: 100%;
  height: 100%;
  display: block;
  transform: rotate(-90deg);
}

.donut-track {
  fill: none;
  stroke: var(--track, var(--border));
  stroke-width: 22;
}

.donut-seg {
  fill: none;
  stroke-width: 22;
  cursor: pointer;
  transition: stroke-width 0.2s, opacity 0.2s;
  opacity: 0.9;
}

.donut-seg:hover,
.donut-seg.seg-active {
  stroke-width: 26;
  opacity: 1;
}

.source-donut.has-active .donut-seg:not(.seg-active) {
  opacity: 0.35;
}

.source-donut-center {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  align-content: center;
  pointer-events: none;
  text-align: center;
  padding: 8px;
}

.source-donut-center strong {
  font-size: 30px;
  line-height: 1.1;
  color: var(--text);
  display: block;
}

.source-donut-center .center-name {
  font-size: 14px;
  line-height: 1.2;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-donut-center span {
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 12px;
  display: block;
}

.source-legend {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  max-height: 200px;
  overflow-y: auto;
  padding-right: 4px;
}

.source-legend::-webkit-scrollbar {
  width: 4px;
}

.source-legend::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 4px;
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
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.source-legend-item:hover,
.source-legend-item.legend-active {
  border-color: var(--accent);
  background: var(--accent-soft);
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

@media (max-width: 640px) {
  .source-chart-head,
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
}

/* ===== 动效 ===== */
.stat-card {
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
