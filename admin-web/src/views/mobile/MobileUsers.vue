<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <input v-model="keyword" class="mobile-input" placeholder="搜索用户名 / 邮箱 / 弦予号" @keyup.enter="loadList" />
      <div class="mobile-actions">
        <button class="mobile-btn primary" @click="loadList">搜索</button>
        <button class="mobile-btn" @click="openAdd = !openAdd">{{ openAdd ? '收起新增' : '新增用户' }}</button>
        <button class="mobile-btn" @click="batchStatus(1)">批量启用</button>
        <button class="mobile-btn danger" @click="batchStatus(0)">批量禁用</button>
        <button class="mobile-btn danger" @click="deleteEmptyFavorites">清理空歌单</button>
      </div>
    </div>
    <div v-if="openAdd" class="mobile-card mobile-form">
      <h3 class="mobile-card-title">新增用户</h3>
      <input v-model="addForm.username" class="mobile-input" placeholder="用户名" />
      <input v-model="addForm.password" class="mobile-input" placeholder="密码" type="password" />
      <input v-model="addForm.email" class="mobile-input" placeholder="邮箱（选填）" />
      <button class="mobile-btn primary" :disabled="saving" @click="addUser">{{ saving ? '提交中...' : '确认新增' }}</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="list.length === 0" class="mobile-empty">暂无用户</div>
    <div v-else class="mobile-list">
      <div v-for="u in list" :key="u.id" class="mobile-item">
        <div class="mobile-item-head">
          <div>
            <div class="mobile-item-title">{{ u.username }}</div>
            <div class="mobile-item-sub">{{ u.email || '未绑定邮箱' }} · {{ u.ciyuanxi_id || '无弦予号' }}</div>
          </div>
          <span class="mobile-badge" :class="u.status == 1 ? 'green' : 'red'">{{ u.status == 1 ? '正常' : '禁用' }}</span>
        </div>
        <div class="mobile-item-sub">听歌时长：{{ u.listen_duration || 0 }} 分钟 · 注册：{{ u.created_at || '-' }}</div>
        <div class="mobile-actions">
          <button class="mobile-btn" @click="toggleUser(u)">{{ u.status == 1 ? '禁用' : '启用' }}</button>
          <button class="mobile-btn" @click="changeEmail(u)">改邮箱</button>
          <button class="mobile-btn" @click="loadPlugins(u)">插件</button>
          <button class="mobile-btn" @click="resetDuration(u)">重置时长</button>
          <button class="mobile-btn danger" @click="deleteAvatar(u)">删头像</button>
          <button class="mobile-btn danger" @click="deleteUser(u)">删除</button>
        </div>
        <pre v-if="pluginsUserId === u.id" class="mobile-code">{{ pluginsText }}</pre>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const keyword = ref('')
const loading = ref(false)
const saving = ref(false)
const openAdd = ref(false)
const list = ref<any[]>([])
const pluginsUserId = ref(0)
const pluginsText = ref('')
const addForm = ref({ username: '', password: '', email: '' })
async function loadList() {
  loading.value = true
  const res = await adminApi<any>('get_users', { page: 1, page_size: 30, keyword: keyword.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  if (res.code !== 200) showToast(res.msg || '加载用户失败')
  loading.value = false
}
async function addUser() {
  if (!addForm.value.username.trim() || !addForm.value.password.trim()) return showToast('请填写用户名和密码')
  saving.value = true
  const res = await adminApi('add_user', addForm.value)
  saving.value = false
  if (res.code === 200) { showToast('新增成功', 'success'); openAdd.value = false; addForm.value = { username: '', password: '', email: '' }; loadList() } else showToast(res.msg || '新增失败')
}
async function toggleUser(u: any) {
  const status = u.status == 1 ? 0 : 1
  const res = await adminApi('toggle_user_status', { id: u.id, status })
  if (res.code === 200) { u.status = status; showToast('操作成功', 'success') } else showToast(res.msg || '操作失败')
}
async function resetDuration(u: any) {
  if (!confirm(`确认重置 ${u.username} 的听歌时长？`)) return
  const res = await adminApi('reset_listen_duration', { user_id: u.id })
  if (res.code === 200) showToast('已重置', 'success'); else showToast(res.msg || '重置失败')
}
async function changeEmail(u: any) {
  const email = prompt(`请输入 ${u.username} 的新邮箱，留空可解除绑定`, u.email || '')
  if (email === null) return
  const res = await adminApi('change_user_email', { user_id: u.id, email: email.trim() })
  if (res.code === 200) { u.email = email.trim(); showToast('邮箱已更新', 'success') } else showToast(res.msg || '更新失败')
}
async function loadPlugins(u: any) {
  if (pluginsUserId.value === u.id) { pluginsUserId.value = 0; pluginsText.value = ''; return }
  const res = await adminApi<any>('get_user_plugins', { user_id: u.id })
  pluginsUserId.value = u.id
  pluginsText.value = res.code === 200 ? JSON.stringify(res.data || {}, null, 2) : (res.msg || '加载插件失败')
}
async function deleteAvatar(u: any) {
  if (!confirm(`确认删除 ${u.username} 的头像？`)) return
  const res = await adminApi('delete_user_avatar', { user_id: u.id })
  if (res.code === 200) showToast('头像已删除', 'success'); else showToast(res.msg || '删除失败')
}
async function batchStatus(status: number) {
  if (!confirm(`确认批量${status === 1 ? '启用' : '禁用'}所有用户？`)) return
  const res = await adminApi('batch_toggle_user_status', { status })
  if (res.code === 200) { showToast('批量操作完成', 'success'); loadList() } else showToast(res.msg || '批量操作失败')
}
async function deleteEmptyFavorites() {
  if (!confirm('确认清理所有空收藏歌单？')) return
  const res = await adminApi('delete_empty_favorite_playlists')
  if (res.code === 200) showToast('清理完成', 'success'); else showToast(res.msg || '清理失败')
}
async function deleteUser(u: any) {
  if (!confirm(`确认删除用户 ${u.username}？`)) return
  const res = await adminApi('delete_user', { id: u.id })
  if (res.code === 200) { showToast('已删除', 'success'); loadList() } else showToast(res.msg || '删除失败')
}
onMounted(loadList)
</script>
