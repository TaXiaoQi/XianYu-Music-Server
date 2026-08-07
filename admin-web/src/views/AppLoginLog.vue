<template>
  <div class="alg-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">APP 登录日志</h2>
          <p class="page-desc">记录管理员通过 APP 端登录的操作日志，包括设备信息、IP 地址和登录状态。</p>
        </div>
        <button class="btn-refresh" @click="loadList" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="M18.7 8l-5.1 5.2-2.8-2.7L7 14.3"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(total) }}</span>
            <span class="stat-label">累计登录</span>
          </div>
        </div>
        <div class="stat-chip stat-today">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.today_count) }}</span>
            <span class="stat-label">
              今日登录
              <span v-if="stats.today_trend > 0" class="trend trend-up">↑{{ stats.today_trend }}%</span>
              <span v-else-if="stats.today_trend < 0" class="trend trend-down">↓{{ Math.abs(stats.today_trend) }}%</span>
              <span v-else class="trend trend-flat">→0%</span>
            </span>
          </div>
        </div>
        <div class="stat-chip stat-ip">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.distinct_ips) }}</span>
            <span class="stat-label">独立 IP</span>
          </div>
        </div>
        <div class="stat-chip stat-device">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.distinct_devices) }}</span>
            <span class="stat-label">独立设备</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 今日成功/失败小条 -->
    <Transition name="fade-up" appear>
      <div v-if="stats.today_count > 0" class="today-bar">
        <div class="today-seg today-success" :style="{ width: successPct + '%' }">
          <span v-if="successPct > 15">成功 {{ stats.today_success }}</span>
        </div>
        <div class="today-seg today-failed" :style="{ width: failedPct + '%' }">
          <span v-if="failedPct > 15">失败 {{ stats.today_failed }}</span>
        </div>
      </div>
    </Transition>

    <!-- 筛选区 -->
    <Transition name="fade-up" appear>
      <div class="filter-bar">
        <div class="filter-left">
          <div class="search-input-wrap">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
            <input v-model="keyword" type="text" placeholder="搜索管理员 / IP / 设备" @keyup.enter="handleSearch" />
          </div>
          <select v-model="statusFilter" @change="handleSearch">
            <option value="">全部状态</option>
            <option value="success">成功</option>
            <option value="failed">失败</option>
          </select>
          <button class="btn-search" @click="handleSearch">搜索</button>
          <button v-if="hasFilter" class="btn-reset" @click="clearFilter">重置</button>
        </div>
        <span class="filter-count">共 {{ filteredTotal }} 条记录</span>
      </div>
    </Transition>

    <!-- 表格区 -->
    <div class="table-card">
      <!-- 加载中 -->
      <div v-if="loading" class="state-box">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <!-- 空状态 -->
      <Transition name="fade-up" appear v-else-if="logList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
          </div>
          <p class="empty-title">{{ hasFilter ? '未找到匹配条件的记录' : '暂无 APP 登录日志' }}</p>
          <p v-if="hasFilter" class="empty-sub">尝试更换关键词或清除筛选条件</p>
        </div>
      </Transition>

      <!-- 数据表格 -->
      <TransitionGroup v-else name="row" tag="div" class="table-wrap">
        <!-- 表头 -->
        <div key="header" class="row row-header">
          <div class="col col-id">ID</div>
          <div class="col col-admin">管理员</div>
          <div class="col col-ip">IP</div>
          <div class="col col-device">设备</div>
          <div class="col col-ver">APP版本</div>
          <div class="col col-os">系统</div>
          <div class="col col-status">状态</div>
          <div class="col col-extra">附加信息</div>
          <div class="col col-time">登录时间</div>
        </div>
        <!-- 数据行 -->
        <div
          v-for="(row, idx) in logList"
          :key="row.id"
          class="row row-data"
          :class="{ 'is-failed': row.status == 0 }"
          :style="{ animationDelay: `${idx * 40}ms` }"
        >
          <div class="col col-id">{{ row.id }}</div>
          <div class="col col-admin">
            <div class="admin-name">{{ row.admin_username || '-' }}</div>
            <div class="admin-id">ID: {{ row.admin_id }}</div>
          </div>
          <div class="col col-ip">{{ row.ip || '-' }}</div>
          <div class="col col-device">
            <div class="device-model">{{ row.device_model || '-' }}</div>
            <div class="device-id" :title="row.device_id">{{ truncate(row.device_id, 16) }}</div>
          </div>
          <div class="col col-ver">{{ row.app_version || '-' }}</div>
          <div class="col col-os">{{ row.os_version || '-' }}</div>
          <div class="col col-status">
            <span class="status-badge" :class="row.status == 1 ? 'badge-success' : 'badge-failed'">
              <svg v-if="row.status == 1" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
              <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              {{ row.status == 1 ? '成功' : '失败' }}
            </span>
          </div>
          <div class="col col-extra">
            <span v-if="row.extra" class="extra-text" :title="row.extra">{{ truncate(row.extra, 30) }}</span>
            <span v-else class="extra-none">-</span>
          </div>
          <div class="col col-time">{{ row.created_at }}</div>
        </div>
      </TransitionGroup>

      <!-- 分页 -->
      <div v-if="!loading && filteredTotal > 0" class="pagination">
        <button class="page-btn" :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          class="page-btn"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button class="page-btn" :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
        <span class="page-info">第 {{ page }} / {{ totalPages }} 页</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi } from '@/api/client'

interface LoginLog {
  id: number
  admin_id: number
  admin_username: string
  ip: string
  user_agent: string
  device_id: string
  device_model: string
  app_version: string
  os_version: string
  status: number
  extra: string
  created_at: string
  [key: string]: any
}

interface LogStats {
  today_count: number
  today_success: number
  today_failed: number
  yesterday_count: number
  today_trend: number
  distinct_ips: number
  distinct_devices: number
}

// ===== 状态 =====
const loading = ref(true)
const logList = ref<LoginLog[]>([])
const page = ref(1)
const pageSize = 20
const filteredTotal = ref(0)
const totalPages = ref(0)
const stats = ref<LogStats>({
  today_count: 0, today_success: 0, today_failed: 0,
  yesterday_count: 0, today_trend: 0,
  distinct_ips: 0, distinct_devices: 0,
})
const total = ref(0)

// 筛选
const keyword = ref('')
const statusFilter = ref('')
const hasFilter = computed(() => !!keyword.value || !!statusFilter.value)

// 分页页码
const pageNumbers = computed(() => {
  const max = 7
  const pages: number[] = []
  if (totalPages.value <= max) {
    for (let i = 1; i <= totalPages.value; i++) pages.push(i)
  } else {
    let start = Math.max(1, page.value - 3)
    let end = Math.min(totalPages.value, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

// 今日成功/失败百分比
const successPct = computed(() => {
  if (stats.value.today_count === 0) return 0
  return Math.round((stats.value.today_success / stats.value.today_count) * 100)
})
const failedPct = computed(() => 100 - successPct.value)

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const res = await adminApi<{
    total: number
    filtered_total: number
    total_pages: number
    list: LoginLog[]
    stats: LogStats
  }>('list_app_login_log', {
    page: page.value,
    page_size: pageSize,
    keyword: keyword.value,
    status_filter: statusFilter.value,
  })
  if (res.code === 200 && res.data) {
    logList.value = res.data.list || []
    filteredTotal.value = res.data.filtered_total || 0
    totalPages.value = res.data.total_pages || 0
    total.value = res.data.total || 0
    if (res.data.stats) {
      stats.value = res.data.stats
    }
  } else {
    logList.value = []
    filteredTotal.value = 0
  }
  loading.value = false
}

// ===== 操作 =====
function handleSearch() {
  page.value = 1
  loadList()
}

function clearFilter() {
  keyword.value = ''
  statusFilter.value = ''
  page.value = 1
  loadList()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadList()
}

// ===== 工具 =====
function formatNum(n: number): string {
  return n.toLocaleString()
}

function truncate(s: string, max: number): string {
  if (!s) return '-'
  return s.length > max ? s.substring(0, max) + '...' : s
}

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.alg-page {
  max-width: 1320px;
  margin: 0 auto;
}

/* ===== 页面头部 ===== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 20px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0 0 6px 0;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 620px;
}
.btn-refresh {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-refresh:hover { border-color: var(--accent); transform: translateY(-1px); }
.btn-refresh:active { transform: scale(0.96); }
.btn-refresh:disabled { opacity: 0.5; cursor: not-allowed; }
.spinning { animation: spin 0.8s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
  margin-bottom: 16px;
}
.stat-chip {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 14px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-3px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 44px; height: 44px;
  border-radius: 12px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-total .stat-icon { background: #f0f0f0; color: #1a1a1a; }
.stat-today .stat-icon { background: #eff6ff; color: #2563eb; }
.stat-ip .stat-icon { background: #f0fdf4; color: #16a34a; }
.stat-device .stat-icon { background: #faf5ff; color: #9333ea; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; display: flex; align-items: center; gap: 4px; }
.trend { font-size: 11px; font-weight: 600; }
.trend-up { color: #22c55e; }
.trend-down { color: #ef4444; }
.trend-flat { color: #8a8a9a; }

/* ===== 今日成功/失败条 ===== */
.today-bar {
  display: flex;
  height: 32px;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 16px;
  border: 1px solid var(--border);
}
.today-seg {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  transition: width 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}
.today-success { background: #16a34a; }
.today-failed { background: #dc2626; }

/* ===== 筛选区 ===== */
.filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.filter-left { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.search-input-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.search-icon {
  position: absolute;
  left: 10px;
  color: #aaa;
  pointer-events: none;
}
.search-input-wrap input {
  padding: 9px 12px 9px 34px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 13px;
  width: 260px;
  outline: none;
  transition: border-color 0.2s;
}
.search-input-wrap input:focus { border-color: var(--accent); }
.filter-left select {
  padding: 9px 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 13px;
  outline: none;
  cursor: pointer;
  background: var(--white);
}
.btn-search {
  padding: 9px 18px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-search:hover { opacity: 0.85; }
.btn-search:active { transform: scale(0.96); }
.btn-reset {
  padding: 9px 14px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-reset:hover { color: var(--text); border-color: #ccc; }
.filter-count { font-size: 13px; color: var(--text-muted); margin-left: auto; }

/* ===== 表格卡片 ===== */
.table-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.table-wrap { overflow-x: auto; }

/* 行 */
.row {
  display: grid;
  grid-template-columns: 60px 140px 130px 160px 90px 90px 80px 160px 150px;
  gap: 0;
  align-items: center;
  min-width: 1100px;
}
.row-header {
  background: #fafafa;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-light);
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 1;
}
.row-data {
  padding: 12px 16px;
  border-bottom: 1px solid #f5f5f5;
  font-size: 13px;
  color: var(--text);
  transition: background 0.15s;
  animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.row-data:hover { background: #fafafa; }
.row-data.is-failed { background: #fffdfd; }
.row-data.is-failed:hover { background: #fff8f8; }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.col { padding-right: 8px; word-break: break-all; }
.col-id { font-weight: 600; color: var(--text-muted); }
.admin-name { font-weight: 600; }
.admin-id { font-size: 11px; color: #aaa; margin-top: 1px; }
.device-model { font-weight: 500; }
.device-id { font-size: 11px; color: #aaa; margin-top: 1px; }

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}
.badge-success { background: #f0fdf4; color: #16a34a; }
.badge-failed { background: #fef2f2; color: #dc2626; }

.extra-text { font-size: 12px; color: #ef4444; }
.extra-none { color: #ccc; }

/* ===== 分页 ===== */
.pagination {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 16px;
  border-top: 1px solid var(--border);
  flex-wrap: wrap;
}
.page-btn {
  padding: 7px 14px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.page-btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.page-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.page-info { font-size: 12px; color: var(--text-muted); margin-left: auto; }

/* ===== 空状态 / 加载 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  gap: 12px;
  font-size: 14px;
}
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; margin-bottom: 4px; }
.empty-title { font-size: 15px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }

/* ===== Spinner ===== */
.spinner {
  width: 32px; height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

.row-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.row-enter-from { opacity: 0; transform: translateY(8px); }
.row-leave-active { transition: all 0.3s ease; }
.row-leave-to { opacity: 0; }
.row-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; }
  .stat-num { font-size: 22px; }
  .search-input-wrap input { width: 180px; }
  .filter-count { width: 100%; text-align: right; }
  .page-info { width: 100%; text-align: center; margin: 4px 0 0 0; }
}
</style>
