<template>
  <div class="mobile-page">
    <div class="mobile-tabs"><button class="mobile-btn" :class="{ primary: tab === 'op' }" @click="switchTab('op')">操作日志</button><button class="mobile-btn" :class="{ primary: tab === 'login' }" @click="switchTab('login')">登录日志</button></div>
    <input v-model="keyword" class="mobile-input" placeholder="搜索日志" @keyup.enter="load" />
    <select v-if="tab === 'login'" v-model="statusFilter" class="mobile-select" @change="load">
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
        <div class="mobile-item-title">{{ log.action || log.username || log.login_name || '日志记录' }}</div>
        <div class="mobile-item-sub">{{ log.detail || log.ip || '-' }}</div>
        <div class="mobile-item-sub">{{ log.created_at || log.login_time || '-' }}</div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi } from '@/api/client'
import './MobilePage.css'
const tab = ref<'op'|'login'>('op'), keyword = ref(''), statusFilter = ref(''), loading = ref(false), list = ref<any[]>([])
const page = ref(1), totalPages = ref(1)
function switchTab(t: 'op'|'login') { tab.value = t; page.value = 1; load() }
function prevPage() { if (page.value > 1) { page.value--; load() } }
function nextPage() { if (page.value < totalPages.value) { page.value++; load() } }
async function load() {
  loading.value = true
  const action = tab.value === 'op' ? 'list_operation_logs' : 'list_admin_login_logs'
  const res = await adminApi<any>(action, { page: page.value, page_size: 30, keyword: keyword.value, status_filter: statusFilter.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  totalPages.value = res.code === 200 && res.data ? Number(res.data.total_pages || 1) : 1
  loading.value = false
}
onMounted(load)
</script>
