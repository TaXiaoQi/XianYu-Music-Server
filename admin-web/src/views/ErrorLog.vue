<template>
  <div class="error-log-wrap">
    <!-- 统计栏 -->
    <div class="stats-bar">
      <div class="stats-tags">
        <span
          v-for="s in statsData.stats"
          :key="s.error_type"
          class="stat-tag"
          @click="filterByType(s.error_type)"
        >
          {{ s.error_type || '未知' }}: <b>{{ s.count }}</b>
        </span>
        <span v-if="statsData.total > 0" class="stat-tag stat-tag-total">
          总计: <b>{{ statsData.total }}</b>
        </span>
        <button class="btn btn-danger btn-sm clear-btn" @click="clearAll">清空全部</button>
      </div>
    </div>

    <!-- 筛选区 -->
    <div class="filters">
      <select v-model="filterType" style="width:160px;">
        <option value="">全部类型</option>
        <option value="crash">崩溃(crash)</option>
        <option value="Exception">异常(Exception)</option>
        <option value="Error">错误(Error)</option>
        <option value="network">网络错误</option>
      </select>
      <select v-model="filterPlatform" style="width:120px;">
        <option value="">全部平台</option>
        <option value="android">Android</option>
        <option value="windows">Windows</option>
      </select>
      <input v-model="filterBrand" type="text" placeholder="手机品牌" style="width:120px;" />
      <input v-model="filterKeyword" type="text" placeholder="搜索关键词(型号/错误信息)" style="width:220px;" @keyup.enter="handleFilter" />
      <button class="btn btn-primary" @click="handleFilter">筛选</button>
      <button v-if="hasFilter" class="btn" @click="clearFilter">清除</button>
      <span class="filter-count">共 {{ total }} 条记录</span>
    </div>

    <!-- 表格 -->
    <div class="card">
      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="logs.length === 0" class="empty">暂无数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>设备型号</th>
              <th>品牌</th>
              <th>平台</th>
              <th>系统版本</th>
              <th>APP版本</th>
              <th>错误类型</th>
              <th>错误信息</th>
              <th>页面</th>
              <th>时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in logs" :key="row.id">
              <td>{{ row.id }}</td>
              <td>{{ row.device_model || '-' }}</td>
              <td>{{ row.device_brand || '-' }}</td>
              <td>{{ row.platform || '-' }}</td>
              <td>{{ row.os_version || '-' }}</td>
              <td>{{ row.app_version || '-' }}</td>
              <td><span class="badge badge-error">{{ row.error_type || '-' }}</span></td>
              <td class="ellipsis" :title="row.error_message">{{ row.error_message || '-' }}</td>
              <td>{{ row.page || '-' }}</td>
              <td class="nowrap-time">{{ row.error_time }}</td>
              <td class="nowrap">
                <button class="btn btn-primary btn-sm" @click="viewDetail(row.id)">详情</button>
                <button class="btn btn-danger btn-sm" @click="deleteRow(row.id)">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && total > 0" class="pagination">
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

    <!-- 详情弹窗 -->
    <Transition name="modal">
    <div v-if="showDetailModal" class="modal-overlay" @click.self="showDetailModal = false">
      <div class="modal modal-wide">
        <div class="modal-header">
          <span class="modal-title">崩溃日志详情</span>
          <button class="modal-close" @click="showDetailModal = false">&times;</button>
        </div>
        <div class="modal-body">
          <div v-if="detailLoading" class="empty">加载中...</div>
          <div v-else-if="detailData">
            <div class="detail-grid">
              <div class="detail-field"><span class="detail-label">记录ID</span><div>{{ detailData.id }}</div></div>
              <div class="detail-field"><span class="detail-label">崩溃时间</span><div>{{ detailData.error_time }}</div></div>
              <div class="detail-field"><span class="detail-label">设备型号</span><div>{{ detailData.device_model || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">手机品牌</span><div>{{ detailData.device_brand || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">出错平台</span><div>{{ detailData.platform || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">系统版本</span><div>{{ detailData.os_version || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">APP版本</span><div>{{ detailData.app_version || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">错误类型</span><div>{{ detailData.error_type || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">页面</span><div>{{ detailData.page || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">IP地址</span><div>{{ detailData.ip || '-' }}</div></div>
              <div class="detail-field"><span class="detail-label">设备ID</span><div>{{ detailData.device_id || '-' }}</div></div>
            </div>
            <div class="detail-section">
              <div class="detail-section-label">错误信息</div>
              <div class="error-message-box">{{ detailData.error_message || '无' }}</div>
            </div>
            <div class="detail-section">
              <div class="detail-section-label">错误堆栈</div>
              <pre class="error-stack-box">{{ detailData.error_stack || '无堆栈信息' }}</pre>
            </div>
            <details v-if="detailData.request_params" class="detail-section">
              <summary class="detail-section-label clickable">原始请求参数</summary>
              <pre class="request-params-box">{{ detailData.request_params }}</pre>
            </details>
          </div>
        </div>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

interface ErrorLog {
  id: number
  device_model: string
  device_brand: string
  platform: string
  os_version: string
  app_version: string
  error_type: string
  error_message: string
  error_stack: string
  page: string
  error_time: string
  ip: string
  device_id: string
  request_params: string
  [key: string]: any
}

interface StatsItem {
  error_type: string
  count: number
}

// 列表数据
const logs = ref<ErrorLog[]>([])
const loading = ref(true)
const loadError = ref('')
const page = ref(1)
const pageSize = 20
const total = ref(0)
const totalPages = ref(0)

// 筛选
const filterType = ref('')
const filterPlatform = ref('')
const filterBrand = ref('')
const filterKeyword = ref('')

const hasFilter = computed(() => filterType.value || filterPlatform.value || filterBrand.value || filterKeyword.value)

// 统计
const statsData = ref<{ stats: StatsItem[]; total: number }>({ stats: [], total: 0 })

// 详情弹窗
const showDetailModal = ref(false)
const detailLoading = ref(false)
const detailData = ref<ErrorLog | null>(null)

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

// 加载列表
async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<{ total: number; total_pages: number; list: ErrorLog[] }>('list_error_logs', {
    page: page.value,
    page_size: pageSize,
    error_type: filterType.value,
    platform: filterPlatform.value,
    device_brand: filterBrand.value,
    keyword: filterKeyword.value,
  })
  if (res.code === 200 && res.data) {
    logs.value = res.data.list || []
    total.value = res.data.total
    totalPages.value = res.data.total_pages
  } else {
    loadError.value = res.msg || '加载失败'
    logs.value = []
  }
  loading.value = false
}

// 加载统计
async function loadStats() {
  const res = await adminApi<{ stats: StatsItem[]; total: number }>('get_error_stats')
  if (res.code === 200 && res.data) {
    statsData.value = res.data
  }
}

function handleFilter() {
  page.value = 1
  loadList()
}

function clearFilter() {
  filterType.value = ''
  filterPlatform.value = ''
  filterBrand.value = ''
  filterKeyword.value = ''
  page.value = 1
  loadList()
}

function filterByType(type: string) {
  filterType.value = type
  page.value = 1
  loadList()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadList()
}

// 查看详情
async function viewDetail(id: number) {
  showDetailModal.value = true
  detailLoading.value = true
  detailData.value = null
  const res = await adminApi<ErrorLog>('get_error_detail', { id })
  detailLoading.value = false
  if (res.code === 200 && res.data) {
    detailData.value = res.data
  } else {
    showToast(res.msg || '加载失败')
    showDetailModal.value = false
  }
}

// 删除单条
async function deleteRow(id: number) {
  const ok = await webConfirm('确定删除这条崩溃日志吗？', { title: '删除日志', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_error', { id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
    loadStats()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// 清空全部
async function clearAll() {
  const ok = await webConfirm('确定清空所有崩溃日志吗？此操作不可恢复！', { title: '清空日志', confirmText: '确认清空' })
  if (!ok) return
  const res = await adminApi('clear_all_errors')
  if (res.code === 200) {
    showToast(res.msg || '已清空', 'success')
    page.value = 1
    loadList()
    loadStats()
  } else {
    showToast(res.msg || '操作失败')
  }
}

onMounted(() => {
  loadList()
  loadStats()
})
</script>

<style scoped>
.error-log-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

/* 统计栏 */
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
  background: #f0f0f5;
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 12px;
  color: #6a6a7a;
  cursor: pointer;
  transition: background 0.15s;
}
.stat-tag:hover { background: #e4e4ee; }
.stat-tag b { font-weight: 700; }
.stat-tag-total {
  background: #fff0f0;
  color: #e74c3c;
  cursor: default;
}
.stat-tag-total:hover { background: #fff0f0; }
.clear-btn {
  margin-left: auto;
  font-size: 12px;
  padding: 4px 12px;
}

/* 筛选区 */
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
  background: var(--white);
}
.filters select:focus,
.filters input:focus { border-color: var(--accent); }
.filter-count {
  font-size: 13px;
  color: var(--text-muted);
}

/* 表格 */
.ellipsis {
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.nowrap { white-space: nowrap; }
.nowrap-time { white-space: nowrap; font-size: 12px; }

/* 分页 */
.pagination {
  display: flex;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--white);
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

/* 详情弹窗 */
.modal-wide {
  width: 90%;
  max-width: 800px;
  max-height: 85vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 0;
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #eee;
}
.modal-title { font-weight: 700; font-size: 16px; }
.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
  line-height: 1;
}
.modal-close:hover { color: #333; }
.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

/* 详情内容 */
.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 24px;
  margin-bottom: 16px;
}
.detail-field .detail-label {
  font-size: 12px;
  color: var(--text-muted);
}
.detail-field > div {
  font-size: 14px;
  margin-top: 2px;
}
.detail-section {
  margin-bottom: 12px;
}
.detail-section-label {
  font-size: 13px;
  color: var(--text-light);
  margin-bottom: 4px;
  font-weight: 600;
}
.detail-section-label.clickable { cursor: pointer; }
.error-message-box {
  background: #fff5f5;
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
  color: #e74c3c;
  border: 1px solid #ffd6d6;
}
.error-stack-box {
  white-space: pre-wrap;
  font-size: 12px;
  color: #ff6b6b;
  max-height: 400px;
  overflow-y: auto;
  background: #fffafa;
  padding: 14px;
  border-radius: 6px;
  border: 1px solid #ffe0e0;
  line-height: 1.6;
  font-family: monospace;
}
.request-params-box {
  white-space: pre-wrap;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 8px;
  background: #f8f8fa;
  padding: 10px;
  border-radius: 6px;
  max-height: 200px;
  overflow-y: auto;
  font-family: monospace;
}

@media (max-width: 768px) {
  .detail-grid { grid-template-columns: 1fr; }
}

/* 弹窗淡进淡出 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.92) translateY(20px);
}
</style>
