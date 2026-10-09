<template>
  <div class="email-log-wrap">
    <!-- 统计栏 -->
    <Transition name="fade-down" appear>
    <div class="stats-bar">
      <div class="stats-tags">
        <span class="stat-tag" @click="filterByStatus('')">总计: <b>{{ statsData.total }}</b></span>
        <span class="stat-tag stat-tag-success" @click="filterByStatus('success')">成功: <b>{{ statsData.success }}</b></span>
        <span class="stat-tag stat-tag-failed" @click="filterByStatus('failed')">失败: <b>{{ statsData.failed }}</b></span>
        <span class="stat-tag" @click="filterByStatus('pending')">待投递: <b>{{ statsData.pending }}</b></span>
        <span class="stat-tag stat-tag-total">今日: <b>{{ statsData.today }}</b></span>
      </div>
    </div>
    </Transition>

    <!-- 筛选区 -->
    <Transition name="fade-up" appear>
    <div class="filters">
      <select v-model="filterStatus" style="width:140px;">
        <option value="">全部状态</option>
        <option value="success">成功</option>
        <option value="failed">失败</option>
        <option value="pending">待投递</option>
      </select>
      <input v-model="filterKeyword" type="text" placeholder="搜索邮箱/主题/错误信息" style="width:240px;" @keyup.enter="handleFilter" />
      <button class="btn btn-primary" @click="handleFilter">筛选</button>
      <button v-if="hasFilter" class="btn" @click="clearFilter">清除</button>
      <span class="filter-count">共 {{ filteredTotal }} 条记录</span>
    </div>
    </Transition>

    <!-- 表格 -->
    <Transition name="fade-up" appear>
    <div class="card">
      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="logs.length === 0" class="empty">暂无数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>收件邮箱</th>
              <th>主题</th>
              <th>状态</th>
              <th>投递通道</th>
              <th>错误信息</th>
              <th>时间</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, idx) in logs" :key="row.id" class="table-row" :style="{ animationDelay: `${idx * 40}ms` }">
              <td>{{ row.id }}</td>
              <td class="ellipsis" :title="row.email">{{ row.email || '-' }}</td>
              <td class="ellipsis" :title="row.subject">{{ row.subject || '-' }}</td>
              <td><span class="badge" :class="statusClass(row.status)">{{ statusText(row.status) }}</span></td>
              <td>{{ channelText(row) }}</td>
              <td class="ellipsis err-msg" :title="row.error_msg">{{ row.error_msg || '-' }}</td>
              <td class="nowrap-time">{{ fmtDateTime(row.created_at) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && filteredTotal > 0" class="pagination">
        <button :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi } from '@/api/client'
import { fmtDateTime } from '@/utils/time'

interface EmailSendLog {
  id: number
  email: string
  subject: string
  interface_id: number
  template_id: number
  status: number
  error_msg: string
  ip: string
  created_at: string
  [key: string]: any
}

const logs = ref<EmailSendLog[]>([])
const loading = ref(true)
const loadError = ref('')
const page = ref(1)
const pageSize = 20
const filteredTotal = ref(0)
const totalPages = ref(0)

const filterStatus = ref('')
const filterKeyword = ref('')

const hasFilter = computed(() => filterStatus.value || filterKeyword.value)

const statsData = ref<{ total: number; success: number; failed: number; pending: number; today: number }>({
  total: 0, success: 0, failed: 0, pending: 0, today: 0,
})

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
  loadError.value = ''
  const res = await adminApi<{ total: number; filtered_total: number; total_pages: number; list: EmailSendLog[]; stats: typeof statsData.value }>('list_email_send_logs', {
    page: page.value,
    page_size: pageSize,
    status_filter: filterStatus.value,
    keyword: filterKeyword.value,
  })
  if (res.code === 200 && res.data) {
    logs.value = res.data.list || []
    filteredTotal.value = res.data.filtered_total
    totalPages.value = res.data.total_pages
    if (res.data.stats) statsData.value = res.data.stats
  } else {
    loadError.value = res.msg || '加载失败'
    logs.value = []
  }
  loading.value = false
}

function handleFilter() {
  page.value = 1
  loadList()
}

function clearFilter() {
  filterStatus.value = ''
  filterKeyword.value = ''
  page.value = 1
  loadList()
}

function filterByStatus(status: string) {
  filterStatus.value = status
  page.value = 1
  loadList()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadList()
}

function statusText(s: number) {
  if (s === 1) return '成功'
  if (s === 2) return '失败'
  return '待投递'
}

function statusClass(s: number) {
  if (s === 1) return 'badge-success'
  if (s === 2) return 'badge-error'
  return 'badge-warn'
}

function channelText(row: EmailSendLog) {
  if (row.ip === 'builtin') return '内置邮箱机'
  return row.ip || '-'
}

onMounted(loadList)
</script>

<style scoped>
.email-log-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

.stats-bar {
  margin-bottom: 16px;
}
.stats-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}
.stat-tag {
  background: var(--track);
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 12px;
  color: #6a6a7a;
  cursor: pointer;
  transition: background 0.15s;
}
.stat-tag:hover { background: var(--table-row-hover); }
.stat-tag b { font-weight: 700; }
.stat-tag-total { cursor: default; }
.stat-tag-success {
  background: rgba(46, 184, 92, 0.10);
  color: #2ebb5c;
}
.stat-tag-failed {
  background: rgba(236, 65, 65, 0.10);
  color: #e74c3c;
}

.filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
  align-items: center;
}
.filters select,
.filters input {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  background: var(--card-solid);
}
.filters select:focus,
.filters input:focus { border-color: var(--accent); }
.filter-count {
  font-size: 13px;
  color: var(--text-muted);
}

.table-wrapper {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
.ellipsis {
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.err-msg { color: #e74c3c; }
.nowrap-time { white-space: nowrap; font-size: 12px; }

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
}
.badge-success {
  background: rgba(46, 184, 92, 0.12);
  color: #2ebb5c;
}
.badge-error {
  background: rgba(236, 65, 65, 0.12);
  color: #e74c3c;
}
.badge-warn {
  background: rgba(240, 166, 35, 0.12);
  color: #f0a623;
}

.pagination {
  display: flex;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.pagination button.active {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }

tr.table-row {
  animation: rowIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes rowIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
