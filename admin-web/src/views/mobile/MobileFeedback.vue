<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">反馈限制</h3>
      <input v-model="limit.feedback_daily_limit" class="mobile-input" type="number" placeholder="每日提交上限，0 表示不限" />
      <button class="mobile-btn primary" @click="saveLimit">保存限制</button>
    </section>
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: status === '' }" @click="setStatus('')">全部</button>
      <button class="mobile-btn" :class="{ primary: status === 'pending' }" @click="setStatus('pending')">待处理</button>
      <button class="mobile-btn" :class="{ primary: status === 'resolved' }" @click="setStatus('resolved')">已解决</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="list.length === 0" class="mobile-empty">暂无反馈</div>
    <div v-else class="mobile-list">
      <div v-for="f in list" :key="f.id" class="mobile-item">
        <div class="mobile-item-head"><div class="mobile-item-title">{{ f.title || f.feedback_type || '用户反馈' }}</div><span class="mobile-badge">{{ f.status }}</span></div>
        <div class="mobile-item-sub">{{ f.content || f.description || '-' }}</div>
        <div class="mobile-item-sub">{{ f.username || f.user_id || '-' }} · {{ f.created_at || '-' }}</div>
        <textarea v-model="replyMap[f.id]" class="mobile-textarea" placeholder="回复内容"></textarea>
        <div class="mobile-actions">
          <button class="mobile-btn" @click="loadDetail(f)">详情</button>
          <button class="mobile-btn" @click="reply(f)">回复</button>
          <button class="mobile-btn" @click="changeStatus(f, 'processing')">处理中</button>
          <button class="mobile-btn primary" @click="changeStatus(f, 'resolved')">解决</button>
        </div>
        <pre v-if="detailId === f.id" class="mobile-code">{{ detailText }}</pre>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const loading = ref(false), status = ref('')
const list = ref<any[]>([])
const detailId = ref(0)
const detailText = ref('')
const replyMap = ref<Record<number, string>>({})
const limit = ref({ feedback_daily_limit: 20 })
function setStatus(s: string) { status.value = s; loadList() }
async function loadList() {
  loading.value = true
  const res = await adminApi<any>('list_feedback', { page: 1, page_size: 30, status: status.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  loading.value = false
}
async function loadLimit() {
  const res = await adminApi<any>('get_feedback_limit')
  if (res.code === 200 && res.data) limit.value = { feedback_daily_limit: Number(res.data.feedback_daily_limit ?? 20) }
}
async function saveLimit() {
  const res = await adminApi('update_feedback_limit', { feedback_daily_limit: Number(limit.value.feedback_daily_limit) })
  if (res.code === 200) showToast('限制已保存', 'success'); else showToast(res.msg || '保存失败')
}
async function loadDetail(f: any) {
  if (detailId.value === f.id) { detailId.value = 0; detailText.value = ''; return }
  const res = await adminApi<any>('get_feedback_detail', { id: f.id })
  detailId.value = f.id
  detailText.value = res.code === 200 ? JSON.stringify(res.data || {}, null, 2) : (res.msg || '加载详情失败')
}
async function reply(f: any) {
  const content = (replyMap.value[f.id] || '').trim()
  if (!content) return showToast('请填写回复内容')
  const res = await adminApi('reply_feedback', { id: f.id, reply: content, reply_content: content })
  if (res.code === 200) { showToast('已回复', 'success'); replyMap.value[f.id] = ''; loadList() } else showToast(res.msg || '回复失败')
}
async function changeStatus(f: any, s: string) {
  const res = await adminApi('update_feedback_status', { id: f.id, status: s })
  if (res.code === 200) { f.status = s; showToast('状态已更新', 'success') } else showToast(res.msg || '操作失败')
}
onMounted(() => { loadLimit(); loadList() })
</script>
