<template>
  <div class="mobile-page mobile-dashboard">
    <!-- 标题区 -->
    <div class="mobile-card dsh-head-card">
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
    </div>

    <div v-if="loading" class="state-text">加载中...</div>
    <div v-else-if="loadError" class="state-text error">{{ loadError }}</div>

    <!-- 今日音源调用占比 -->
    <div v-if="!loading && !loadError" class="mobile-card source-section">
      <div class="source-head">
        <div>
          <h3>今日音源调用占比</h3>
          <p>按不同音源的今日调用次数统计</p>
        </div>
        <span class="source-total">总调用 {{ stats.today_source_calls ?? 0 }} 次</span>
      </div>
      <div class="source-body">
        <div class="source-donut" :class="{ 'has-active': activeIndex >= 0 }" @click="onDonutClick" @mouseleave="activeIndex = -1">
          <svg viewBox="0 0 120 120" class="donut-svg">
            <circle class="donut-track" cx="60" cy="60" :r="DONUT_R" />
            <circle
              v-for="seg in donutSegments"
              :key="seg.source_name"
              class="donut-seg"
              :cx="60" :cy="60" :r="DONUT_R"
              :stroke="seg.color"
              :stroke-dasharray="seg.dasharray"
              :stroke-dashoffset="seg.dashoffset"
              :class="{ 'seg-active': activeIndex === seg.index }"
              @mouseenter="activeIndex = seg.index"
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
            @mouseenter="activeIndex = idx"
            @mouseleave="activeIndex = -1"
            @click="toggleActive(idx)"
          >
            <i :style="{ background: item.color }"></i>
            <span class="source-name">{{ item.source_name }}</span>
            <span class="source-count">{{ item.count }} 次</span>
            <span class="source-percent">{{ item.percent }}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 数据概览 -->
    <div v-if="!loading && !loadError" class="stats-section">
      <div class="stats-grid">
        <div v-for="(card, idx) in statCards" :key="card.label" class="stat-card" :style="{ animationDelay: `${idx * 80}ms` }">
          <div class="label">{{ card.label }}</div>
          <div class="value" :class="{ 'hot-keyword': card.hot }">{{ card.value() }}</div>
          <div class="sub">{{ card.sub() }}</div>
        </div>
      </div>
    </div>

    <!-- 常用操作 -->
    <div class="quick-section">
      <h3 class="quick-title">常用操作</h3>
      <div class="quick-grid">
        <router-link to="/m/announcements" class="mobile-btn primary">公告管理</router-link>
        <router-link to="/m/version" class="mobile-btn primary">版本管理</router-link>
        <router-link to="/m/email-config" class="mobile-btn">邮箱机设置</router-link>
        <router-link to="/m/turnstile-config" class="mobile-btn">审核设置</router-link>
        <router-link to="/m/about-config" class="mobile-btn">关于页设置</router-link>
        <router-link to="/m/database" class="mobile-btn">数据库管理</router-link>
      </div>
    </div>

    <!-- 服务器 API -->
    <div class="mobile-card api-section">
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

// 统计卡片配置（支持动态值，与桌面版一致，今日热搜在最后）
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
const DONUT_R = 48
const DONUT_STROKE = 20
const DONUT_C = 2 * Math.PI * DONUT_R
const activeIndex = ref(-1)
const legendRef = ref<HTMLElement | null>(null)

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

function toggleActive(index: number) {
  activeIndex.value = activeIndex.value === index ? -1 : index
  // 选中时把对应列表项滚动到滚动容器可见位置，避免被遮住
  if (activeIndex.value >= 0) {
    scrollLegendToActive(activeIndex.value)
  }
}

/** 直接选中指定索引对应的列表项（圆环点击用，避免移动端 mouseenter 先行设置 activeIndex 后被 toggle 取消的问题） */
function setActive(index: number) {
  activeIndex.value = index
  scrollLegendToActive(index)
}

/** 滚动右侧图例容器，使指定索引的列表项可见 */
function scrollLegendToActive(index: number) {
  const container = legendRef.value
  if (!container) return
  const item = container.querySelectorAll('.source-legend-item')[index] as HTMLElement | undefined
  if (!item) return
  // 用 getBoundingClientRect 计算列表项相对滚动容器的位置（offsetTop 相对定位祖先，基准不可靠）
  const cRect = container.getBoundingClientRect()
  const iRect = item.getBoundingClientRect()
  const cTop = container.scrollTop
  const cBottom = cTop + container.clientHeight
  const iTop = iRect.top - cRect.top + cTop
  const iBottom = iTop + iRect.height
  if (iTop < cTop) {
    container.scrollTop = iTop
  } else if (iBottom > cBottom) {
    container.scrollTop = iBottom - container.clientHeight
  }
}

/** 点击圆环区域，按角度计算落在哪个扇区 */
function onDonutClick(e: MouseEvent | TouchEvent) {
  const el = e.currentTarget as HTMLElement | null
  if (!el || donutSegments.value.length === 0) return
  const rect = el.getBoundingClientRect()
  const cx = rect.width / 2
  const cy = rect.height / 2
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY
  const dx = clientX - rect.left - cx
  const dy = clientY - rect.top - cy
  const radius = Math.sqrt(dx * dx + dy * dy)
  // 点击在圆环内孔（中心总调用区域）或超出外缘都忽略，避免误触
  // 坐标基于 CSS 像素，需按元素实际尺寸换算 viewBox 半径
  const scale = rect.width / 120
  const innerR = (DONUT_R - DONUT_STROKE / 2) * scale
  const outerR = (DONUT_R + DONUT_STROKE / 2) * scale
  if (radius < innerR || radius > outerR) return
  // 视觉角度：0° 在正上方，顺时针递增
  let angle = Math.atan2(dx, -dy) * (180 / Math.PI)
  if (angle < 0) angle += 360
  // 根据累计角度找扇区
  const items = sourceItems.value
  const total = items.reduce((s, it) => s + Number(it.count || 0), 0)
  let acc = 0
  for (let i = 0; i < items.length; i++) {
    const segAngle = (Number(items[i].count || 0) / total) * 360
    if (angle >= acc && angle < acc + segAngle) {
      setActive(i)
      return
    }
    acc += segAngle
  }
}

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
  gap: 14px;
}

/* 标题区 */
.dsh-head-card {
  padding: 18px 16px;
}
.dsh-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 12px;
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

/* 今日音源调用占比 */
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
  flex-direction: row;
  gap: 14px;
  align-items: stretch;
}

/* 左侧圆环 */
.source-donut {
  position: relative;
  width: 130px;
  min-width: 130px;
  height: 130px;
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
  stroke-width: 20;
}
.donut-seg {
  fill: none;
  stroke-width: 20;
  cursor: pointer;
  transition: stroke-width 0.2s, opacity 0.2s;
  opacity: 0.85;
}
.donut-seg:hover,
.donut-seg.seg-active {
  stroke-width: 24;
  opacity: 1;
}
.source-donut.has-active .donut-seg:not(.seg-active) {
  opacity: 0.4;
}
.source-donut-center {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  align-content: center;
  pointer-events: none;
  text-align: center;
  padding: 4px;
}
.source-donut-center strong {
  font-size: 18px;
  line-height: 1.1;
  color: var(--text);
  display: block;
}
.source-donut-center .center-name {
  font-size: 10px;
  line-height: 1.2;
  max-width: 70px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.source-donut-center span {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 9px;
  display: block;
}

/* 右侧滚动列表 */
.source-legend {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  max-height: 130px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-right: 2px;
}
.source-legend::-webkit-scrollbar {
  width: 3px;
}
.source-legend::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}
.source-legend-item {
  display: grid;
  grid-template-columns: 8px minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--control-bg);
  font-size: 11px;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}
.source-legend-item:hover,
.source-legend-item.legend-active {
  border-color: var(--accent);
  background: var(--accent-soft);
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
  font-size: 10px;
}
.source-percent {
  color: var(--accent);
  font-size: 10px;
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

/* 数据概览（4 个卡片单行排列，覆盖全局 2 列网格） */
.stats-section {
  display: flex;
  flex-direction: column;
  padding: 0 2px;
}
.stats-section .stats-grid {
  display: flex !important;
  flex-wrap: nowrap !important;
  flex-direction: row !important;
  gap: 8px;
  margin-bottom: 0;
}
.stats-section .stat-card {
  flex: 1 1 0;
  min-width: 0;
  padding: 12px 6px;
  text-align: center;
}
.stats-section .stat-card .label {
  margin-bottom: 6px;
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.stats-section .stat-card .value {
  font-size: 18px;
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.stats-section .stat-card .sub {
  margin-top: 5px;
  font-size: 9px;
  line-height: 1.3;
  word-break: break-all;
}
.stats-section .stat-card .hot-keyword {
  font-size: 15px;
}

/* 常用操作（去卡片化） */
.quick-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 2px;
}
.quick-title {
  margin: 0;
  font-size: 15px;
  color: var(--text);
  font-weight: 850;
}
.quick-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
}
.quick-grid .mobile-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 38px;
  padding: 8px 14px;
  text-decoration: none;
  flex: 0 0 auto;
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
