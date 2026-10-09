<template>
  <div class="mobile-page">
    <div class="mobile-grid">
      <div class="mobile-stat"><span>总计</span><strong>{{ stats.total }}</strong></div>
      <div class="mobile-stat"><span>成功</span><strong>{{ stats.success }}</strong></div>
      <div class="mobile-stat"><span>失败</span><strong>{{ stats.failed }}</strong></div>
      <div class="mobile-stat"><span>今日</span><strong>{{ stats.today }}</strong></div>
    </div>
    <div class="mobile-actions">
      <button class="mobile-btn primary" @click="load">刷新</button>
      <button class="mobile-btn" :disabled="page <= 1" @click="prevPage">上一页</button>
      <button class="mobile-btn" :disabled="page >= totalPages" @click="nextPage">下一页</button>
    </div>
    <div class="mobile-actions">
      <select v-model="statusFilter" class="mobile-select" @change="reload">
        <option value="">全部状态</option>
        <option value="success">成功</option>
        <option value="failed">失败</option>
        <option value="pending">待投递</option>
      </select>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="list.length === 0" class="mobile-empty">暂无数据</div>
    <div v-else class="mobile-list">
      <div v-for="e in list" :key="e.id" class="mobile-item">
        <div class="mobile-item-title">{{ e.email || '-' }}</div>
        <div class="mobile-item-sub">{{ e.subject || '-' }}</div>
        <div class="mobile-item-sub">
          <span class="badge" :class="statusClass(e.status)">{{ statusText(e.status) }}</span>
          · {{ e.ip === 'builtin' ? '内置邮箱机' : (e.ip || '-') }} · {{ fmtDateTime(e.created_at) || '-' }}
        </div>
        <pre v-if="e.error_msg" class="mobile-code">{{ e.error_msg }}</pre>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi } from '@/api/client'
import './MobilePage.css'
import { fmtDateTime } from '@/utils/time'
const loading = ref(false), list = ref<any[]>([])
const page = ref(1), totalPages = ref(1)
const statusFilter = ref('')
const stats = ref<any>({ total: 0, success: 0, failed: 0, pending: 0, today: 0 })
function prevPage() { if (page.value > 1) { page.value--; load() } }
function nextPage() { if (page.value < totalPages.value) { page.value++; load() } }
function reload() { page.value = 1; load() }
async function load() {
  loading.value = true
  const res = await adminApi<any>('list_email_send_logs', { page: page.value, page_size: 30, status_filter: statusFilter.value })
  if (res.code === 200 && res.data) {
    list.value = res.data.list || []
    totalPages.value = Number(res.data.total_pages || 1)
    if (res.data.stats) stats.value = res.data.stats
  } else {
    list.value = []
    totalPages.value = 1
  }
  loading.value = false
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
onMounted(load)
</script>
<style scoped>
.mobile-select {
  padding: 8px 12px;
  border: 1px solid var(--border, #ddd);
  border-radius: 8px;
  font-size: 13px;
  background: var(--card-solid, #fff);
  flex: 1;
}
.badge {
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 11px;
}
.badge-success { background: rgba(46, 184, 92, 0.12); color: #2ebb5c; }
.badge-error { background: rgba(236, 65, 65, 0.12); color: #e74c3c; }
.badge-warn { background: rgba(240, 166, 35, 0.12); color: #f0a623; }
</style>
