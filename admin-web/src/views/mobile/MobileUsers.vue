<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <input v-model="keyword" class="mobile-input" placeholder="搜索用户名 / 邮箱 / 弦予号" @keyup.enter="loadList" />
      <div class="mobile-actions">
        <button class="mobile-btn primary" @click="loadList">搜索</button>
        <button class="mobile-btn" @click="openAdd = !openAdd">{{ openAdd ? '收起新增' : '新增用户' }}</button>
        <button class="mobile-btn" @click="openBannedDevices">封禁设备</button>
        <button class="mobile-btn" @click="batchStatus(1)">批量启用</button>
        <button class="mobile-btn danger" @click="batchStatus(0)">批量禁用</button>
        <button class="mobile-btn danger" @click="deleteEmptyFavorites">清理空歌单</button>
      </div>
    </div>
    <transition name="expand">
      <div v-if="openAdd" class="mobile-card mobile-form">
        <h3 class="mobile-card-title">新增用户</h3>
        <input v-model="addForm.username" class="mobile-input" placeholder="用户名" />
        <input v-model="addForm.password" class="mobile-input" placeholder="密码" type="password" />
        <input v-model="addForm.email" class="mobile-input" placeholder="邮箱（选填）" />
        <button class="mobile-btn primary" :disabled="saving" @click="addUser">{{ saving ? '提交中...' : '确认新增' }}</button>
      </div>
    </transition>
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
          <button class="mobile-btn" @click="openDeviceInfo(u)">设备</button>
          <button class="mobile-btn" @click="resetDuration(u)">重置时长</button>
          <button class="mobile-btn danger" @click="deleteAvatar(u)">删头像</button>
          <button class="mobile-btn danger" @click="deleteUser(u)">删除</button>
        </div>
        <transition name="expand">
          <pre v-if="pluginsUserId === u.id" class="mobile-code">{{ pluginsText }}</pre>
        </transition>
        <transition name="expand">
          <div v-if="deviceUserId === u.id" class="device-panel">
            <div v-if="deviceLoading" class="mobile-empty">设备加载中...</div>
            <template v-else>
              <div class="mobile-item-sub">设备数量：{{ deviceRows.length || '-' }} · 封禁状态：{{ deviceData.is_banned ? '已封禁' : '正常' }}</div>
              <div v-if="deviceRows.length === 0" class="mobile-empty">暂无设备信息</div>
              <div v-for="d in deviceRows" :key="deviceIdOf(d)" class="device-row">
                <div>
                  <div class="mobile-item-title">{{ d.device_model || d.model || d.device_name || '未知设备' }}</div>
                  <div class="mobile-item-sub">{{ deviceIdOf(d) || '-' }}</div>
                  <div class="mobile-item-sub">{{ d.platform || d.os || '-' }} · {{ d.last_login_time || d.updated_at || d.created_at || '-' }}</div>
                  <div v-if="d.ban_reason || d.reason" class="mobile-item-sub">原因：{{ d.ban_reason || d.reason }}</div>
                </div>
                <div class="mobile-actions">
                  <button v-if="isDeviceBanned(d)" class="mobile-btn" @click="unbanUserDevice(deviceIdOf(d))">解封</button>
                  <button v-else class="mobile-btn danger" @click="banUserDevice(deviceIdOf(d), u.username)">封禁</button>
                </div>
              </div>
            </template>
          </div>
        </transition>
      </div>
    </div>
    <transition name="expand">
      <section v-if="showBannedPanel" class="mobile-card mobile-form">
        <h3 class="mobile-card-title">封禁设备管理</h3>
        <input v-model="banDeviceInput" class="mobile-input" placeholder="设备 ID" />
        <textarea v-model="banReasonInput" class="mobile-textarea" placeholder="封禁原因"></textarea>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="manualBanDevice">手动封禁</button>
          <button class="mobile-btn" @click="loadBannedDevices">刷新列表</button>
          <button class="mobile-btn" @click="showBannedPanel = false">收起</button>
        </div>
        <div v-if="bannedLoading" class="mobile-empty">加载中...</div>
        <div v-else-if="bannedDevices.length === 0" class="mobile-empty">暂无封禁设备</div>
        <div v-else class="mobile-list">
          <div v-for="d in bannedDevices" :key="d.id || deviceIdOf(d)" class="mobile-item">
            <div class="mobile-item-title">{{ deviceIdOf(d) || '未知设备' }}</div>
            <div class="mobile-item-sub">{{ d.username || d.user_id || '-' }} · {{ d.created_at || d.banned_at || '-' }}</div>
            <div class="mobile-item-sub">原因：{{ d.reason || d.ban_reason || '-' }}</div>
            <div class="mobile-actions">
              <button class="mobile-btn" @click="unbanDeviceById(d.id, deviceIdOf(d))">解封</button>
            </div>
          </div>
        </div>
      </section>
    </transition>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm, mobilePrompt } from '@/utils/mobileDialog'
import './MobilePage.css'
const keyword = ref('')
const loading = ref(false)
const saving = ref(false)
const openAdd = ref(false)
const list = ref<any[]>([])
const pluginsUserId = ref(0)
const pluginsText = ref('')
const addForm = ref({ username: '', password: '', email: '' })
const deviceUserId = ref(0)
const deviceLoading = ref(false)
const deviceData = ref<any>({})
const showBannedPanel = ref(false)
const bannedLoading = ref(false)
const bannedDevices = ref<any[]>([])
const banDeviceInput = ref('')
const banReasonInput = ref('')
const deviceRows = computed(() => normalizeDeviceRows(deviceData.value))
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
  if (!(await mobileConfirm(`确认重置 ${u.username} 的听歌时长？`))) return
  const res = await adminApi('reset_listen_duration', { user_id: u.id })
  if (res.code === 200) showToast('已重置', 'success'); else showToast(res.msg || '重置失败')
}
async function changeEmail(u: any) {
  const email = await mobilePrompt(`请输入 ${u.username} 的新邮箱，留空可解除绑定`, u.email || '')
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
async function openDeviceInfo(u: any) {
  if (deviceUserId.value === u.id) {
    deviceUserId.value = 0
    deviceData.value = {}
    return
  }
  deviceUserId.value = u.id
  await refreshDeviceInfo(u)
}
async function refreshDeviceInfo(u: any) {
  deviceLoading.value = true
  deviceData.value = { username: u.username }
  const res = await adminApi<any>('get_user_devices', { user_id: u.id })
  if (res.code === 200 && res.data) {
    deviceData.value = res.data
  } else {
    showToast(res.msg || '加载设备信息失败')
  }
  deviceLoading.value = false
}
function normalizeDeviceRows(data: any): any[] {
  if (!data) return []
  if (Array.isArray(data.devices)) return data.devices
  if (Array.isArray(data.list)) return data.list
  if (Array.isArray(data.rows)) return data.rows
  if (data.device && typeof data.device === 'object') return [data.device]
  if (data.current_device && typeof data.current_device === 'object') return [data.current_device]
  if (data.device_id || data.deviceId) return [data]
  return []
}
function deviceIdOf(d: any): string {
  return String(d?.device_id || d?.deviceId || d?.id || '')
}
function isDeviceBanned(d: any): boolean {
  return Boolean(d?.is_banned || d?.banned || d?.status === 'banned' || d?.status === 'disabled')
}
async function banUserDevice(deviceId: string, username: string) {
  if (!deviceId) return showToast('未读取到设备 ID')
  const reason = await mobilePrompt(`请输入封禁用户 ${username} 的设备原因`, '')
  if (reason === null) return
  if (!reason.trim()) return showToast('封禁原因不能为空')
  if (!(await mobileConfirm(`确定封禁设备 ${deviceId.slice(0, 16)}... 吗？封禁后该设备将无法登录。`))) return
  const res = await adminApi('ban_device', { device_id: deviceId, reason: reason.trim() })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    if (deviceUserId.value) {
      const current = list.value.find((item) => item.id === deviceUserId.value)
      if (current) await refreshDeviceInfo(current)
    }
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function unbanUserDevice(deviceId: string) {
  if (!deviceId) return showToast('未读取到设备 ID')
  const res = await adminApi('unban_device', { device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    if (deviceUserId.value) {
      const current = list.value.find((item) => item.id === deviceUserId.value)
      if (current) await refreshDeviceInfo(current)
    }
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function openBannedDevices() {
  showBannedPanel.value = true
  await loadBannedDevices()
}
async function loadBannedDevices() {
  bannedLoading.value = true
  const res = await adminApi<any>('list_banned_devices', { page: 1, page_size: 100 })
  bannedDevices.value = res.code === 200 && res.data ? (res.data.list || []) : []
  bannedLoading.value = false
}
async function manualBanDevice() {
  const deviceId = banDeviceInput.value.trim()
  const reason = banReasonInput.value.trim()
  if (!deviceId) return showToast('请输入设备 ID')
  if (!reason) return showToast('封禁原因不能为空')
  const res = await adminApi('ban_device', { device_id: deviceId, reason })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    banDeviceInput.value = ''
    banReasonInput.value = ''
    await loadBannedDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function unbanDeviceById(id: number, deviceId: string) {
  if (!(await mobileConfirm(`确定解封设备 ${deviceId || id}？`))) return
  const res = await adminApi('unban_device', { id, device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    await loadBannedDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function deleteAvatar(u: any) {
  if (!(await mobileConfirm(`确认删除 ${u.username} 的头像？`))) return
  const res = await adminApi('delete_user_avatar', { user_id: u.id })
  if (res.code === 200) showToast('头像已删除', 'success'); else showToast(res.msg || '删除失败')
}
async function batchStatus(status: number) {
  if (!(await mobileConfirm(`确认批量${status === 1 ? '启用' : '禁用'}所有用户？`))) return
  const res = await adminApi('batch_toggle_user_status', { status })
  if (res.code === 200) { showToast('批量操作完成', 'success'); loadList() } else showToast(res.msg || '批量操作失败')
}
async function deleteEmptyFavorites() {
  if (!(await mobileConfirm('确认清理所有空收藏歌单？'))) return
  const res = await adminApi('delete_empty_favorite_playlists')
  if (res.code === 200) showToast('清理完成', 'success'); else showToast(res.msg || '清理失败')
}
async function deleteUser(u: any) {
  if (!(await mobileConfirm(`确认删除用户 ${u.username}？`))) return
  const res = await adminApi('delete_user', { id: u.id })
  if (res.code === 200) { showToast('已删除', 'success'); loadList() } else showToast(res.msg || '删除失败')
}
onMounted(loadList)
</script>
<style scoped>
.device-panel {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}
.device-row {
  padding: 10px 0;
  border-top: 1px solid var(--border);
  animation: mobileItemIn 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)) both;
}
.device-row:first-of-type {
  border-top: 0;
}
</style>
