<template>
  <div class="asr-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">签名拦截</h2>
          <p class="page-desc">记录请求签名校验失败的流水，多见于使用旧版本客户端（密钥已轮换）或异常探测。同一 IP + 动作 10 秒内只记录一条。</p>
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
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.total) }}</span>
            <span class="stat-label">累计拦截</span>
          </div>
        </div>
        <div class="stat-chip stat-today">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.today) }}</span>
            <span class="stat-label">今日拦截</span>
          </div>
        </div>
        <div class="stat-chip stat-ip">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ formatNum(stats.distinct_ips) }}</span>
            <span class="stat-label">涉及 IP</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 筛选区 -->
    <Transition name="fade-up" appear>
      <div class="filter-bar">
        <div class="filter-left">
          <div class="search-input-wrap">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
            <input v-model="keyword" type="text" placeholder="搜索动作 / IP / UA" @keyup.enter="handleSearch" />
          </div>
          <button class="btn-search" @click="handleSearch">搜索</button>
          <button v-if="keyword" class="btn-reset" @click="clearFilter">重置</button>
        </div>
        <span class="filter-count">共 {{ filteredTotal }} 条记录</span>
      </div>
    </Transition>

    <!-- 表格区 -->
    <div class="table-card">
      <div v-if="loading" class="state-box">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <Transition name="fade-up" appear v-else-if="logList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          </div>
          <p class="empty-title">{{ keyword ? '未找到匹配条件的记录' : '暂无签名拦截记录' }}</p>
          <p v-if="keyword" class="empty-sub">尝试更换关键词或清除筛选条件</p>
        </div>
      </Transition>

      <TransitionGroup v-else name="row" tag="div" class="table-wrap">
        <div key="header" class="row row-header">
          <div class="col col-id">ID</div>
          <div class="col col-action">动作</div>
          <div class="col col-ip">IP</div>
          <div class="col col-ua">UA</div>
          <div class="col col-body">请求体</div>
          <div class="col col-iv">加密</div>
          <div class="col col-skew">时差</div>
          <div class="col col-time">拦截时间</div>
        </div>
        <div
          v-for="(row, idx) in logList"
          :key="row.id"
          class="row row-data"
          :style="{ animationDelay: `${idx * 40}ms` }"
        >
          <div class="col col-id">{{ row.id }}</div>
          <div class="col col-action"><span class="action-text">{{ row.action || '-' }}</span></div>
          <div class="col col-ip"><span class="ip-text" :title="row.ip">{{ row.ip || '-' }}</span></div>
          <div class="col col-ua"><span class="ua-text" :title="row.user_agent">{{ truncate(row.user_agent, 48) }}</span></div>
          <div class="col col-body">{{ formatBytes(row.body_len) }}</div>
          <div class="col col-iv">
            <span class="iv-badge" :class="row.has_iv == 1 ? 'iv-yes' : 'iv-no'">{{ row.has_iv == 1 ? 'AES' : '明文' }}</span>
          </div>
          <div class="col col-skew" :class="{ 'skew-warn': Math.abs(row.skew_s || 0) > 60 }">{{ row.skew_s ?? 0 }}s</div>
          <div class="col col-time">{{ fmtDateTime(row.created_at) }}</div>
        </div>
      </TransitionGroup>

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
import { fmtDateTime } from '@/utils/time'

interface SignRejectLog {
  id: number
  action: string
  ip: string
  user_agent: string
  body_len: number
  has_iv: number
  skew_s: number
  created_at: string
  [key: string]: any
}

interface RejectStats {
  total: number
  today: number
  distinct_ips: number
}

const loading = ref(true)
const logList = ref<SignRejectLog[]>([])
const page = ref(1)
const pageSize = 20
const filteredTotal = ref(0)
const totalPages = ref(0)
const total = ref(0)
const stats = ref<RejectStats>({ total: 0, today: 0, distinct_ips: 0 })

const keyword = ref('')

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

async function loadList() {
  loading.value = true
  const res = await adminApi<{
    total: number
    filtered_total: number
    total_pages: number
    list: SignRejectLog[]
    stats: RejectStats
  }>('list_sign_reject_logs', {
    page: page.value,
    page_size: pageSize,
    keyword: keyword.value,
  })
  if (res.code === 200 && res.data) {
    logList.value = res.data.list || []
    filteredTotal.value = res.data.filtered_total || 0
    totalPages.value = res.data.total_pages || 0
    total.value = res.data.total || 0
    if (res.data.stats) stats.value = res.data.stats
  } else {
    logList.value = []
    filteredTotal.value = 0
  }
  loading.value = false
}

function handleSearch() {
  page.value = 1
  loadList()
}

function clearFilter() {
  keyword.value = ''
  page.value = 1
  loadList()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadList()
}

function formatNum(n: number): string {
  return (n || 0).toLocaleString()
}

function formatBytes(n: number): string {
  if (!n) return '0 B'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
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
.asr-page {
  max-width: 1320px;
  margin: 0 auto;
}

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
  background: var(--card-solid);
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

.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
  margin-bottom: 16px;
}
.stat-chip {
  background: var(--card-solid);
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
.stat-total .stat-icon { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.stat-today .stat-icon { background: rgba(59, 130, 246, 0.12); color: #2563eb; }
.stat-ip .stat-icon { background: rgba(34, 197, 94, 0.14); color: #16a34a; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; }

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
  background: var(--card-solid);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-reset:hover { color: var(--text); border-color: #ccc; }
.filter-count { font-size: 13px; color: var(--text-muted); margin-left: auto; }

.table-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.table-wrap { overflow-x: auto; }

.row {
  display: grid;
  grid-template-columns: 70px 170px 140px minmax(200px, 1.4fr) 80px 64px 70px 160px;
  gap: 0;
  align-items: center;
  min-width: 1000px;
}
.row-header {
  background: var(--control-bg);
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
.row-data:hover { background: var(--control-bg); }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.col { padding-right: 8px; word-break: break-all; }
.col-id { font-weight: 600; color: var(--text-muted); }
.action-text {
  font-family: monospace;
  font-size: 12px;
  background: var(--track);
  padding: 2px 8px;
  border-radius: 6px;
}
.ip-text { font-family: monospace; font-size: 12.5px; }
.ua-text { font-size: 12px; color: var(--text-muted); }

.iv-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}
.iv-yes { background: rgba(34, 197, 94, 0.14); color: #16a34a; }
.iv-no { background: rgba(245, 158, 11, 0.15); color: #d97706; }

.skew-warn { color: #ef4444; font-weight: 600; }

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
  background: var(--card-solid);
  color: var(--text-light);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.page-btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.page-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.page-info { font-size: 12px; color: var(--text-muted); margin-left: auto; }

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

.spinner {
  width: 32px; height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

.row-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.row-enter-from { opacity: 0; transform: translateY(8px); }
.row-leave-active { transition: all 0.3s ease; }
.row-leave-to { opacity: 0; }
.row-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; }
  .stat-num { font-size: 22px; }
  .search-input-wrap input { width: 180px; }
  .filter-count { width: 100%; text-align: right; }
  .page-info { width: 100%; text-align: center; margin: 4px 0 0 0; }
}
</style>
