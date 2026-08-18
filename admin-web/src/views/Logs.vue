<template>
  <div class="logs-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">后台日志</h2>
          <p class="page-desc">查看后台管理员的操作记录与登录记录，便于行为审计与安全排查。</p>
        </div>
        <button class="btn-refresh" @click="loadCurrent" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 内容区 -->
    <Transition name="fade-up" appear>
      <div class="logs-content">
        <!-- Tab 切换 -->
        <div class="tab-bar">
          <div
            class="tab-item"
            :class="{ active: activeTab === 'operation' }"
            @click="switchTab('operation')"
          >操作日志</div>
          <div
            class="tab-item"
            :class="{ active: activeTab === 'login' }"
            @click="switchTab('login')"
          >登录日志</div>
        </div>

        <!-- 搜索 / 筛选 -->
        <div class="search-bar">
          <div class="search-left">
            <div class="search-input-wrap">
              <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
              <input
                v-model="keyword"
                type="text"
                :placeholder="activeTab === 'operation' ? '搜索管理员 / 操作 / IP' : '搜索管理员 / IP'"
                @keyup.enter="handleSearch"
              />
            </div>
            <button class="btn-search" @click="handleSearch">搜索</button>
            <button v-if="keyword || statusFilter" class="btn-reset" @click="clearSearch">清除</button>

            <!-- 登录日志状态筛选 -->
            <div v-if="activeTab === 'login'" class="status-chips">
              <span
                class="chip"
                :class="{ active: statusFilter === '' }"
                @click="setStatusFilter('')"
              >全部</span>
              <span
                class="chip chip-success"
                :class="{ active: statusFilter === 'success' }"
                @click="setStatusFilter('success')"
              >成功</span>
              <span
                class="chip chip-error"
                :class="{ active: statusFilter === 'failed' }"
                @click="setStatusFilter('failed')"
              >失败</span>
            </div>
          </div>
          <span class="search-count">共 {{ curTotal }} 条</span>
        </div>

        <!-- 表格卡片 -->
        <div class="table-card">
          <!-- 加载中 -->
          <div v-if="loading" class="state-box">
            <div class="spinner"></div>
            <span>加载中...</span>
          </div>

          <!-- 空状态 -->
          <Transition name="fade-up" appear v-else-if="curTotal === 0">
            <div class="state-box state-empty">
              <div class="empty-icon">
                <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="9" y1="17" x2="13" y2="17"/></svg>
              </div>
              <p class="empty-title">{{ hasFilter ? '未找到匹配条件的记录' : (activeTab === 'operation' ? '暂无操作日志' : '暂无登录日志') }}</p>
              <p v-if="hasFilter" class="empty-sub">尝试更换关键词或清除筛选条件</p>
            </div>
          </Transition>

          <!-- 操作日志表格 -->
          <div v-else-if="activeTab === 'operation'" class="table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>管理员</th>
                  <th>操作</th>
                  <th>目标</th>
                  <th>详情</th>
                  <th>IP</th>
                  <th>时间</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, idx) in opList"
                  :key="row.id"
                  class="table-row"
                  :style="{ animationDelay: `${idx * 40}ms` }"
                >
                  <td class="col-id">{{ row.id }}</td>
                  <td>
                    <div class="admin-name">{{ row.admin_username || '-' }}</div>
                    <div class="admin-sub">ID: {{ row.admin_id }}</div>
                  </td>
                  <td><span class="action-tag">{{ row.action || '-' }}</span></td>
                  <td class="col-target">{{ row.target || '-' }}</td>
                  <td class="col-detail" :title="row.detail || ''">{{ truncate(row.detail, 40) }}</td>
                  <td class="col-ip">{{ row.ip || '-' }}</td>
                  <td class="col-time">{{ fmtDateTime(row.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- 登录日志表格 -->
          <div v-else class="table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>管理员</th>
                  <th>IP</th>
                  <th>状态</th>
                  <th>时间</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, idx) in loginList"
                  :key="row.id"
                  class="table-row"
                  :class="{ 'is-failed': row.status == 0 }"
                  :style="{ animationDelay: `${idx * 40}ms` }"
                >
                  <td class="col-id">{{ row.id }}</td>
                  <td>
                    <div class="admin-name">{{ row.admin_username || '-' }}</div>
                    <div class="admin-sub">ID: {{ row.admin_id }}</div>
                  </td>
                  <td class="col-ip">{{ row.ip || '-' }}</td>
                  <td>
                    <span class="badge" :class="row.status == 1 ? 'badge-success' : 'badge-error'">
                      {{ row.status == 1 ? '成功' : '失败' }}
                    </span>
                  </td>
                  <td class="col-time">{{ fmtDateTime(row.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- 分页 -->
          <div v-if="!loading && curTotal > 0" class="pagination">
            <button class="page-btn" :disabled="curPage <= 1" @click="goPage(curPage - 1)">上一页</button>
            <button
              v-for="p in pageNumbers"
              :key="p"
              class="page-btn"
              :class="{ active: p === curPage }"
              @click="goPage(p)"
            >{{ p }}</button>
            <button class="page-btn" :disabled="curPage >= curTotalPages" @click="goPage(curPage + 1)">下一页</button>
            <span class="page-info">第 {{ curPage }} / {{ curTotalPages }} 页</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { fmtDateTime } from '@/utils/time'

interface OperationLog {
  id: number
  admin_id: number
  admin_username: string
  action: string
  target: string
  detail: string
  ip: string
  created_at: string
  [key: string]: any
}

interface LoginLog {
  id: number
  admin_id: number
  admin_username: string
  ip: string
  status: number
  created_at: string
  [key: string]: any
}

// ===== 状态 =====
const activeTab = ref<'operation' | 'login'>('operation')
const loading = ref(false)
const pageSize = 20

// 搜索 / 筛选
const keyword = ref('')
const statusFilter = ref('') // '' | 'success' | 'failed'

// 操作日志
const opList = ref<OperationLog[]>([])
const opPage = ref(1)
const opTotal = ref(0)
const opTotalPages = ref(0)

// 登录日志
const loginList = ref<LoginLog[]>([])
const loginPage = ref(1)
const loginTotal = ref(0)
const loginTotalPages = ref(0)

// 当前 tab 的派生值
const curTotal = computed(() => (activeTab.value === 'operation' ? opTotal.value : loginTotal.value))
const curPage = computed(() => (activeTab.value === 'operation' ? opPage.value : loginPage.value))
const curTotalPages = computed(() => (activeTab.value === 'operation' ? opTotalPages.value : loginTotalPages.value))

const hasFilter = computed(() => !!keyword.value || !!statusFilter.value)

// 分页页码
const pageNumbers = computed(() => {
  const tp = curTotalPages.value
  const cur = curPage.value
  const max = 7
  const pages: number[] = []
  if (tp <= max) {
    for (let i = 1; i <= tp; i++) pages.push(i)
  } else {
    let start = Math.max(1, cur - 3)
    let end = Math.min(tp, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

// ===== 加载数据 =====
async function loadOpLogs() {
  loading.value = true
  const res = await adminApi<{
    total: number
    page: number
    page_size: number
    total_pages: number
    list: OperationLog[]
  }>('list_operation_logs', {
    page: opPage.value,
    page_size: pageSize,
    keyword: keyword.value,
  })
  if (res.code === 200 && res.data) {
    opList.value = res.data.list || []
    opTotal.value = res.data.total || 0
    opTotalPages.value = res.data.total_pages || 0
  } else {
    opList.value = []
    opTotal.value = 0
    opTotalPages.value = 0
    if (res.code !== 200) showToast(res.msg || '加载操作日志失败')
  }
  loading.value = false
}

async function loadLoginLogs() {
  loading.value = true
  const res = await adminApi<{
    total: number
    page: number
    page_size: number
    total_pages: number
    list: LoginLog[]
  }>('list_admin_login_logs', {
    page: loginPage.value,
    page_size: pageSize,
    keyword: keyword.value,
    status_filter: statusFilter.value,
  })
  if (res.code === 200 && res.data) {
    loginList.value = res.data.list || []
    loginTotal.value = res.data.total || 0
    loginTotalPages.value = res.data.total_pages || 0
  } else {
    loginList.value = []
    loginTotal.value = 0
    loginTotalPages.value = 0
    if (res.code !== 200) showToast(res.msg || '加载登录日志失败')
  }
  loading.value = false
}

function loadCurrent() {
  if (activeTab.value === 'operation') loadOpLogs()
  else loadLoginLogs()
}

// ===== 操作 =====
function switchTab(tab: 'operation' | 'login') {
  if (activeTab.value === tab) return
  activeTab.value = tab
  // 切换 tab 时重新加载该 tab 数据
  loadCurrent()
}

function handleSearch() {
  if (activeTab.value === 'operation') {
    opPage.value = 1
    loadOpLogs()
  } else {
    loginPage.value = 1
    loadLoginLogs()
  }
}

function clearSearch() {
  keyword.value = ''
  statusFilter.value = ''
  if (activeTab.value === 'operation') {
    opPage.value = 1
    loadOpLogs()
  } else {
    loginPage.value = 1
    loadLoginLogs()
  }
}

function setStatusFilter(v: string) {
  if (statusFilter.value === v) return
  statusFilter.value = v
  loginPage.value = 1
  loadLoginLogs()
}

function goPage(p: number) {
  if (activeTab.value === 'operation') {
    if (p < 1 || p > opTotalPages.value || p === opPage.value) return
    opPage.value = p
    loadOpLogs()
  } else {
    if (p < 1 || p > loginTotalPages.value || p === loginPage.value) return
    loginPage.value = p
    loadLoginLogs()
  }
}

// ===== 工具 =====
function truncate(s: string, max: number): string {
  if (!s) return '-'
  return s.length > max ? s.substring(0, max) + '...' : s
}

onMounted(() => {
  loadOpLogs()
})
</script>

<style scoped>
.logs-page {
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
  max-width: 520px;
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

/* ===== Tab 切换 ===== */
.tab-bar {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 4px;
}
.tab-item {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-muted);
  user-select: none;
}
.tab-item:hover { color: var(--text); }
.tab-item.active {
  background: var(--accent);
  color: #fff;
}

/* ===== 搜索 / 筛选 ===== */
.search-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.search-left {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}
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
  background: var(--white);
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
  background: var(--white);
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-reset:hover { color: var(--text); border-color: #ccc; }

/* 状态筛选 chips */
.status-chips {
  display: inline-flex;
  gap: 6px;
  margin-left: 4px;
}
.chip {
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: #f5f5f5;
  color: var(--text-muted);
  border: 1px solid transparent;
  transition: all 0.2s;
  user-select: none;
}
.chip:hover { background: #ececec; color: var(--text); }
.chip-success.active { background: #f0fdf4; color: #16a34a; border-color: #bbf7d0; }
.chip-error.active { background: #fef2f2; color: #dc2626; border-color: #fecaca; }
.chip.active { background: var(--accent); color: #fff; }

.search-count {
  font-size: 13px;
  color: var(--text-muted);
  margin-left: auto;
}

/* ===== 表格卡片 ===== */
.table-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.table-wrap {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
.data-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--white);
  min-width: 720px;
}
.data-table th {
  padding: 12px 14px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  background: #fafafa;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.data-table td {
  padding: 12px 14px;
  font-size: 13px;
  color: var(--text);
  border-bottom: 1px solid #f5f5f5;
  vertical-align: middle;
}
.data-table tr.table-row {
  animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.data-table tr.table-row:hover td { background: #fafbfc; }
.data-table tr.table-row.is-failed td { background: #fffdfd; }
.data-table tr.table-row.is-failed:hover td { background: #fff8f8; }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.col-id { font-weight: 600; color: var(--text-muted); white-space: nowrap; }
.col-ip { white-space: nowrap; color: var(--text-light); }
.col-time { white-space: nowrap; font-size: 12px; color: var(--text-light); }
.col-target { white-space: nowrap; }
.col-detail {
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-light);
}
.admin-name { font-weight: 600; }
.admin-sub { font-size: 11px; color: #aaa; margin-top: 1px; }

.action-tag {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  background: #eff6ff;
  color: #2563eb;
}

/* 状态徽标 */
.badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
}
.badge-success { background: #f0fdf4; color: #16a34a; }
.badge-error { background: #fef2f2; color: #dc2626; }

/* ===== 分页 ===== */
.pagination {
  display: flex;
  gap: 6px;
  justify-content: center;
  align-items: center;
  margin-top: 16px;
  padding: 16px;
  border-top: 1px solid var(--border);
  flex-wrap: wrap;
}
.page-btn {
  min-width: 36px;
  height: 36px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.page-btn:hover:not(:disabled):not(.active) { border-color: var(--accent); color: var(--accent); }
.page-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }
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
  width: 32px;
  height: 32px;
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

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .search-input-wrap input { width: 180px; }
  .search-count { width: 100%; text-align: right; }
  .page-info { width: 100%; text-align: center; margin: 4px 0 0 0; }
  .status-chips { margin-left: 0; }
}
</style>
