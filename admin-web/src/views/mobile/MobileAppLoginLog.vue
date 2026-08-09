<template>
  <div class="mobile-page">
    <input v-model="keyword" class="mobile-input" placeholder="搜索用户 / 设备 / IP" @keyup.enter="load" />
    <select v-model="statusFilter" class="mobile-select" @change="load">
      <option value="">全部状态</option>
      <option value="success">成功</option>
      <option value="failed">失败</option>
    </select>
    <div class="mobile-actions">
      <button class="mobile-btn primary" @click="load">刷新</button>
      <button class="mobile-btn" :disabled="page <= 1" @click="prevPage">上一页</button>
      <button class="mobile-btn" :disabled="page >= totalPages" @click="nextPage">下一页</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="log in list" :key="log.id" class="mobile-item">
        <div class="mobile-item-head"><div class="mobile-item-title">{{ log.username || log.user_id || 'APP 登录' }}</div><span class="mobile-badge" :class="log.status === 'success' || log.success ? 'green' : 'red'">{{ log.status || (log.success ? 'success' : 'failed') }}</span></div>
        <div class="mobile-item-sub">{{ log.device_model || log.device_id || '-' }}</div>
        <div class="mobile-item-sub">{{ log.ip || '-' }} · {{ log.created_at || log.login_time || '-' }}</div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi } from '@/api/client'
import './MobilePage.css'
const keyword = ref(''), statusFilter = ref(''), loading = ref(false), list = ref<any[]>([])
const page = ref(1), totalPages = ref(1)
function prevPage() { if (page.value > 1) { page.value--; load() } }
function nextPage() { if (page.value < totalPages.value) { page.value++; load() } }
async function load() {
  loading.value = true
  const res = await adminApi<any>('list_app_login_log', { page: page.value, page_size: 30, keyword: keyword.value, status_filter: statusFilter.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  totalPages.value = res.code === 200 && res.data ? Number(res.data.total_pages || 1) : 1
  loading.value = false
}
onMounted(load)
</script>
