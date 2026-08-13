<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">新增通知邮箱</h3>
      <input v-model="form.email" class="mobile-input" type="email" placeholder="notify@example.com" />
      <input v-model="form.remark" class="mobile-input" type="text" placeholder="备注（可选）" />
      <button class="mobile-btn primary" @click="add">新增</button>
      <p class="mobile-muted">启用后，服务端会将关键状态变更以邮件形式通知到以下邮箱。</p>
    </div>

    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="item in list" :key="item.id" class="mobile-item" :style="{ opacity: item.status == 1 ? 1 : 0.6 }">
        <div class="mobile-item-head">
          <div class="mobile-admin-left">
            <div class="notify-avatar" :class="item.status == 1 ? 'avatar-active' : 'avatar-disabled'">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
            </div>
            <div>
              <div class="mobile-item-title">{{ item.email }}</div>
              <div class="mobile-item-sub">{{ item.remark || '暂无备注' }}</div>
              <div class="mobile-item-sub muted-time">创建时间：{{ item.created_at || '-' }}</div>
            </div>
          </div>
          <span class="mobile-badge" :class="item.status == 1 ? 'green' : 'red'">{{ item.status == 1 ? '启用中' : '已停用' }}</span>
        </div>
        <div class="mobile-actions">
          <button class="mobile-btn" @click="sendTest(item)">发送测试</button>
          <button class="mobile-btn" @click="toggle(item)">{{ item.status == 1 ? '停用' : '启用' }}</button>
          <button class="mobile-btn danger" @click="remove(item)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const loading = ref(false), list = ref<any[]>([])
const form = ref({ email: '', remark: '' })

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}
async function load() { loading.value = true; const res = await adminApi<any>('list_notification_emails'); list.value = res.code === 200 && Array.isArray(res.data) ? res.data : []; loading.value = false }
async function add() {
  const email = form.value.email.trim()
  if (!email) { showToast('请输入邮箱地址'); return }
  if (!isValidEmail(email)) { showToast('邮箱格式不正确'); return }
  const res = await adminApi('add_notification_email', { email, remark: form.value.remark.trim() })
  if (res.code === 200) { showToast('添加成功', 'success'); form.value = { email: '', remark: '' }; load() } else showToast(res.msg || '添加失败')
}
async function toggle(item: any) { const res = await adminApi('toggle_notification_email', { id: item.id }); if (res.code === 200) { item.status = item.status == 1 ? 0 : 1; showToast('已更新', 'success') } else showToast(res.msg || '操作失败') }
async function remove(item: any) { if (!(await mobileConfirm(`确认删除通知邮箱 "${item.email}"？`))) return; const res = await adminApi('delete_notification_email', { id: item.id }); if (res.code === 200) { showToast('已删除', 'success'); load() } else showToast(res.msg || '删除失败') }
async function sendTest(item: any) { const res = await adminApi('test_notification_email', { email: item.email }); res.code === 200 ? showToast(res.msg || '测试通知已发送', 'success') : showToast(res.msg || '发送失败') }
onMounted(load)
</script>
<style scoped>
.mobile-admin-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.notify-avatar {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.avatar-active { background: #eff6ff; color: #3b82f6; }
.avatar-disabled { background: #f3f4f6; color: #9ca3af; }
</style>