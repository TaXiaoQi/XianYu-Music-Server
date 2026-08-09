<template>
  <div class="mobile-page">
    <div class="mobile-grid">
      <div class="mobile-stat"><span>总报错</span><strong>{{ stats.total || list.length }}</strong></div>
      <div class="mobile-stat"><span>统计项</span><strong>{{ statsItems.length }}</strong></div>
    </div>
    <div class="mobile-actions">
      <button class="mobile-btn primary" @click="loadAll">刷新</button>
      <button class="mobile-btn" :disabled="page <= 1" @click="prevPage">上一页</button>
      <button class="mobile-btn" :disabled="page >= totalPages" @click="nextPage">下一页</button>
      <button class="mobile-btn danger" @click="clearAll">清空</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="e in list" :key="e.id" class="mobile-item">
        <div class="mobile-item-title">{{ e.error_message || e.message || '错误日志' }}</div>
        <div class="mobile-item-sub">{{ e.device_model || '-' }} · {{ e.platform || '-' }} · {{ e.error_time || e.created_at || '-' }}</div>
        <pre v-if="e.error_stack" class="mobile-code">{{ e.error_stack }}</pre>
        <pre v-if="detailId === e.id" class="mobile-code">{{ detailText }}</pre>
        <div class="mobile-actions">
          <button class="mobile-btn" @click="detail(e)">详情</button>
          <button class="mobile-btn danger" @click="remove(e)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const loading = ref(false), list = ref<any[]>([])
const page = ref(1), totalPages = ref(1)
const stats = ref<any>({})
const statsItems = ref<any[]>([])
const detailId = ref(0)
const detailText = ref('')
function prevPage() { if (page.value > 1) { page.value--; load() } }
function nextPage() { if (page.value < totalPages.value) { page.value++; load() } }
async function load() {
  loading.value = true
  const res = await adminApi<any>('list_error_logs', { page: page.value, page_size: 30 })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  totalPages.value = res.code === 200 && res.data ? Number(res.data.total_pages || 1) : 1
  loading.value = false
}
async function loadStats() {
  const res = await adminApi<any>('get_error_stats')
  if (res.code === 200 && res.data) { stats.value = res.data; statsItems.value = res.data.stats || [] }
}
async function loadAll() { await Promise.all([load(), loadStats()]) }
async function detail(e: any) {
  if (detailId.value === e.id) { detailId.value = 0; detailText.value = ''; return }
  const res = await adminApi<any>('get_error_detail', { id: e.id })
  detailId.value = e.id
  detailText.value = res.code === 200 ? JSON.stringify(res.data || {}, null, 2) : (res.msg || '加载详情失败')
}
async function remove(e: any) { const res = await adminApi('delete_error', { id: e.id }); res.code === 200 ? (showToast('已删除', 'success'), load()) : showToast(res.msg || '删除失败') }
async function clearAll() { if (!confirm('确认清空所有报错日志？')) return; const res = await adminApi('clear_all_errors'); res.code === 200 ? (showToast('已清空', 'success'), load()) : showToast(res.msg || '清空失败') }
onMounted(loadAll)
</script>
