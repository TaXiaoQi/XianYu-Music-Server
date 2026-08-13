<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form mobile-toolbar">
      <input v-model="keyword" class="mobile-input" placeholder="搜索昵称 / 弦予号 / 邮箱" @keyup.enter="loadList" />
      <div class="mobile-actions mobile-toolbar-actions">
        <template v-if="!isBatchMode">
          <button class="mobile-btn primary" @click="loadList">搜索</button>
          <button class="mobile-btn" @click="openAdd = !openAdd">{{ openAdd ? '收起新增' : '新增用户' }}</button>
          <button class="mobile-btn primary" @click="enterBatchMode">批量管理</button>
        </template>
        <div v-else class="mobile-batch-bar">
          <button class="mobile-btn" @click="toggleSelectAll">{{ isAllSelected ? '取消全选' : '全选' }}</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchToggleSelected(0)">封禁</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchToggleSelected(1)">启用</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchBanDevice">封禁ID</button>
          <button class="mobile-btn danger" :disabled="selectedCount === 0" @click="batchDeleteSelected">删除</button>
          <button class="mobile-btn danger" @click="deleteEmptyFavorites">清空歌单</button>
          <button class="mobile-btn" @click="openBannedDevices">设备管理</button>
          <span class="mobile-batch-count">已选 {{ selectedCount }} 项</span>
          <button class="mobile-btn primary" @click="exitBatchMode">完成</button>
        </div>
      </div>
    </div>
    <transition name="expand">
      <div v-if="openAdd" class="mobile-card mobile-form">
        <h3 class="mobile-card-title">新增用户</h3>
        <input v-model="addForm.ciyuanxi_id" class="mobile-input" placeholder="弦予号（必填，字母开头）" />
        <input v-model="addForm.password" class="mobile-input" placeholder="密码（必填）" type="password" />
        <input v-model="addForm.nickname" class="mobile-input" placeholder="昵称（选填，留空默认弦予+号）" />
        <input v-model="addForm.email" class="mobile-input" placeholder="邮箱（选填）" />
        <button class="mobile-btn primary" :disabled="saving" @click="addUser">{{ saving ? '提交中...' : '确认新增' }}</button>
      </div>
    </transition>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="list.length === 0" class="mobile-empty">暂无用户</div>
    <div v-else class="mobile-list">
      <div v-for="u in list" :key="u.id" class="mobile-item" :class="{ 'batch-selected': isBatchMode && selected.has(u.id) }">
        <div class="mobile-item-head">
          <span v-if="isBatchMode" class="mobile-select-badge" :class="{ checked: selected.has(u.id) }" @click="toggleSelectOne(u.id)">
            <svg v-if="selected.has(u.id)" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </span>
          <img v-if="u.avatar_url" :src="u.avatar_url" class="mobile-avatar" alt="头像" @click="openAvatarPreview(u.avatar_url)" />
          <div v-else class="mobile-avatar mobile-avatar-ph">{{ (u.nickname || u.username || '?').charAt(0) }}</div>
          <div class="mobile-item-main">
            <div class="mobile-item-title-row">
              <span class="mobile-item-title">{{ u.ciyuanxi_id || '-' }}</span>
              <span class="mobile-badge" :class="u.status == 1 ? 'green' : 'red'">{{ u.status == 1 ? '正常' : '禁用' }}</span>
            </div>
            <div class="mobile-item-sub">{{ u.nickname || u.username || '-' }}</div>
            <div class="mobile-item-sub mobile-item-verify">
              <span class="mobile-email">{{ u.email || '未绑定邮箱' }}</span>
              <span class="mobile-badge" :class="u.email_verified == 1 ? 'green' : ''">{{ u.email_verified == 1 ? '已验证' : '未验证' }}</span>
            </div>
          </div>
        </div>
        <div v-if="u.status == 0 && u.ban_reason" class="mobile-item-reason">封禁原因：{{ u.ban_reason }}</div>
        <div class="mobile-item-foot">
          <span>听歌时长 {{ formatDuration(u.listen_duration) }}</span>
          <span>注册 {{ u.created_at || '-' }}</span>
        </div>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="openActionMenu(u)">操作</button>
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
        <h3 class="mobile-card-title">设备管理</h3>
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

    <!-- 头像预览 -->
    <transition name="expand">
      <div v-if="avatarPreview" class="mobile-avatar-preview" @click="avatarPreview = ''">
        <img :src="avatarPreview" alt="头像" />
      </div>
    </transition>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm, mobilePrompt } from '@/utils/mobileDialog'
import { webActionMenu } from '@/utils/webDialog'
import './MobilePage.css'
const keyword = ref('')
const loading = ref(false)
const saving = ref(false)
const openAdd = ref(false)
const list = ref<any[]>([])
const pluginsUserId = ref(0)
const pluginsText = ref('')
const addForm = ref({ ciyuanxi_id: '', nickname: '', password: '', email: '' })
const deviceUserId = ref(0)
const deviceLoading = ref(false)
const deviceData = ref<any>({})
const showBannedPanel = ref(false)
const bannedLoading = ref(false)
const bannedDevices = ref<any[]>([])
const banDeviceInput = ref('')
const banReasonInput = ref('')
const avatarPreview = ref('')
const deviceRows = computed(() => normalizeDeviceRows(deviceData.value))

function formatDuration(seconds: number | undefined): string {
  const dur = Number(seconds) || 0
  const h = Math.floor(dur / 3600)
  const m = Math.floor((dur % 3600) / 60)
  const s = dur % 60
  return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}
function openAvatarPreview(url: string) {
  avatarPreview.value = url
}

// 批量选择模式（参考桌面端）
const isBatchMode = ref(false)
const selected = ref<Set<number>>(new Set())
const selectedCount = computed(() => selected.value.size)
const isAllSelected = computed(() => selected.value.size === list.value.length && list.value.length > 0)

function enterBatchMode() {
  isBatchMode.value = true
  selected.value.clear()
}
function exitBatchMode() {
  isBatchMode.value = false
  selected.value.clear()
}
function toggleSelectOne(id: number) {
  if (selected.value.has(id)) {
    selected.value.delete(id)
  } else {
    selected.value.add(id)
  }
}
function toggleSelectAll() {
  if (isAllSelected.value) {
    selected.value.clear()
  } else {
    list.value.forEach(u => selected.value.add(u.id))
  }
}
async function batchToggleSelected(status: number) {
  if (selected.value.size === 0) {
    showToast('请先选择要操作的用户', 'error')
    return
  }
  const label = status === 1 ? '启用' : '禁用'
  let reason = ''
  if (status !== 1) {
    const input = await mobilePrompt(`请输入对选中的 ${selected.value.size} 个用户执行禁用的原因`, '')
    if (input === null) return // 用户取消，静默退出
    reason = input.trim()
    if (!reason) return showToast('封禁原因不能为空')
  }
  if (!(await mobileConfirm(`确定${label}选中的 ${selected.value.size} 个用户吗？`))) return
  let successCount = 0
  let errorCount = 0
  for (const userId of selected.value) {
    const res = await adminApi('toggle_user_status', { id: userId, status, reason })
    if (res.code === 200) {
      const user = list.value.find(u => u.id === userId)
      if (user) user.status = status
      successCount++
    } else {
      errorCount++
    }
  }
  showToast(`批量操作完成：成功 ${successCount}，失败 ${errorCount}`, successCount > 0 ? 'success' : 'error')
  selected.value.clear()
  isBatchMode.value = false
  loadList()
}
async function batchBanDevice() {
  if (selected.value.size === 0) return
  const targets = list.value.filter(u => selected.value.has(u.id) && (u.last_device_id || '').trim())
  if (targets.length === 0) {
    showToast('选中的用户均无设备ID，无需封禁', 'error')
    return
  }
  const input = await mobilePrompt(`将封禁选中的 ${targets.length} 个用户最近登录的设备，请输入封禁原因`, '')
  if (input === null) return // 用户取消，静默退出
  const reason = input.trim()
  if (!reason) return showToast('封禁原因不能为空')
  if (!(await mobileConfirm(`确定封禁 ${targets.length} 个用户的设备ID吗？封禁后这些设备将无法登录。`))) return
  let successCount = 0
  let errorCount = 0
  for (const u of targets) {
    const res = await adminApi('ban_device', { device_id: (u.last_device_id || '').trim(), reason })
    if (res.code === 200) successCount++
    else errorCount++
  }
  showToast(`批量封禁设备完成：成功 ${successCount}，失败 ${errorCount}`, successCount > 0 ? 'success' : 'error')
  selected.value.clear()
  isBatchMode.value = false
}
async function batchDeleteSelected() {
  if (selected.value.size === 0) {
    showToast('请先选择要删除的用户', 'error')
    return
  }
  if (!(await mobileConfirm(`确认删除选中的 ${selected.value.size} 个用户？此操作不可撤销`))) return
  let successCount = 0
  let errorCount = 0
  for (const userId of selected.value) {
    const res = await adminApi('delete_user', { id: userId })
    if (res.code === 200) {
      successCount++
    } else {
      errorCount++
    }
  }
  showToast(`批量删除完成：成功 ${successCount}，失败 ${errorCount}`, successCount > 0 ? 'success' : 'error')
  selected.value.clear()
  isBatchMode.value = false
  loadList()
}

async function loadList() {
  loading.value = true
  const res = await adminApi<any>('get_users', { page: 1, page_size: 30, keyword: keyword.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  if (res.code !== 200) showToast(res.msg || '加载用户失败')
  loading.value = false
}
async function addUser() {
  const ciyuanxi = addForm.value.ciyuanxi_id.trim()
  if (!ciyuanxi) return showToast('请填写弦予号')
  if (!/^[a-zA-Z][a-zA-Z0-9_-]{5,19}$/.test(ciyuanxi)) return showToast('弦予号需 6-20 位，字母开头')
  if (!addForm.value.password) return showToast('请填写密码')
  saving.value = true
  const res = await adminApi('add_user', { username: ciyuanxi, nickname: addForm.value.nickname.trim(), password: addForm.value.password, email: addForm.value.email.trim() })
  saving.value = false
  if (res.code === 200) { showToast('新增成功', 'success'); openAdd.value = false; addForm.value = { ciyuanxi_id: '', nickname: '', password: '', email: '' }; loadList() } else showToast(res.msg || '新增失败')
}
async function openActionMenu(u: any) {
  const action = await webActionMenu(`用户操作 · ${u.nickname || u.username}`, [
    { key: 'toggle', label: u.status == 1 ? '禁用用户' : '启用用户', danger: u.status == 1, success: u.status != 1 },
    { key: 'ciyuanxi', label: '修改弦予号' },
    { key: 'email', label: '修改邮箱' },
    { key: 'reset', label: '重置听歌时长' },
    { key: 'plugins', label: '查看插件' },
    { key: 'device', label: '设备信息' },
    { key: 'avatar', label: '删除头像', danger: true, show: !!u.avatar_url },
    { key: 'delete', label: '删除用户', danger: true },
  ])
  if (!action) return
  switch (action) {
    case 'toggle': await toggleUser(u); break
    case 'ciyuanxi': await changeCiyuanxi(u); break
    case 'email': await changeEmail(u); break
    case 'reset': await resetDuration(u); break
    case 'plugins': await loadPlugins(u); break
    case 'device': await openDeviceInfo(u); break
    case 'avatar': await deleteAvatar(u); break
    case 'delete': await deleteUser(u); break
  }
}
async function toggleUser(u: any) {
  const status = u.status == 1 ? 0 : 1
  let reason = ''
  if (status === 0) {
    const input = await mobilePrompt(`请输入封禁用户 ${u.nickname || u.username} 的原因`, '')
    if (input === null) return // 用户取消，静默退出
    reason = input.trim()
    if (!reason) return showToast('封禁原因不能为空')
  }
  const res = await adminApi('toggle_user_status', { id: u.id, status, reason })
  if (res.code === 200) { u.status = status; u.ban_reason = status ? '' : reason; showToast(status ? '已启用' : '已禁用', 'success') } else showToast(res.msg || '操作失败')
}
async function changeCiyuanxi(u: any) {
  const input = await mobilePrompt(`请输入 ${u.nickname || u.username} 的新弦予号`, u.ciyuanxi_id || '')
  if (input === null) return
  const newId = input.trim()
  if (!newId) return showToast('请输入弦予号', 'error')
  if (!/^[a-zA-Z][a-zA-Z0-9_-]{5,19}$/.test(newId)) return showToast('弦予号需 6-20 位，字母开头', 'error')
  if (!(await mobileConfirm(`确认将 ${u.nickname || u.username} 的弦予号修改为 ${newId} 吗？`))) return
  const res = await adminApi('change_ciyuanxi_id', { user_id: u.id, new_ciyuanxi_id: newId })
  if (res.code === 200) {
    showToast('弦予号已修改', 'success')
    u.ciyuanxi_id = newId
  } else {
    showToast(res.msg || '修改失败', 'error')
  }
}
async function resetDuration(u: any) {
  if (!(await mobileConfirm(`确认重置 ${u.nickname || u.username} 的听歌时长？`))) return
  const res = await adminApi('reset_listen_duration', { user_id: u.id })
  if (res.code === 200) showToast('已重置', 'success'); else showToast(res.msg || '重置失败')
}
async function changeEmail(u: any) {
  const email = await mobilePrompt(`请输入 ${u.nickname || u.username} 的新邮箱，留空可解除绑定`, u.email || '')
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
  if (!(await mobileConfirm(`确认删除 ${u.nickname || u.username} 的头像？`))) return
  const res = await adminApi('delete_user_avatar', { user_id: u.id })
  if (res.code === 200) showToast('头像已删除', 'success'); else showToast(res.msg || '删除失败')
}
async function deleteEmptyFavorites() {
  if (!(await mobileConfirm('确认清理所有空收藏歌单？'))) return
  const res = await adminApi('delete_empty_favorite_playlists')
  if (res.code === 200) showToast('清理完成', 'success'); else showToast(res.msg || '清理失败')
}
async function deleteUser(u: any) {
  if (!(await mobileConfirm(`确认删除用户 ${u.nickname || u.username}？`))) return
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

/* 顶部工具栏：搜索 + 操作合并为一行 */
.mobile-toolbar {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.mobile-toolbar-actions {
  width: 100%;
  justify-content: flex-end;
  flex-wrap: wrap;
}
.mobile-toolbar-actions > .mobile-batch-bar {
  width: 100%;
}

/* 批量选择模式 */
.mobile-batch-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 10px 0 2px;
  border-top: 1px dashed var(--border);
  margin-top: 2px;
}
.mobile-batch-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--accent);
  white-space: nowrap;
}
.mobile-select-badge {
  width: 22px;
  height: 22px;
  flex: none;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  margin-right: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.mobile-select-badge.checked {
  background: var(--accent);
  border-color: var(--accent);
}
.mobile-item.batch-selected {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 6%, transparent);
}
.mobile-item-head .mobile-select-badge {
  align-self: flex-start;
  margin-top: 2px;
}

/* 头像 */
.mobile-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  object-fit: cover;
  flex: none;
  cursor: pointer;
}
.mobile-avatar-ph {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 18px;
  font-weight: 800;
}
.mobile-item-main {
  flex: 1;
  min-width: 0;
}
.mobile-item-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.mobile-item-verify {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.mobile-email {
  word-break: break-all;
}
.mobile-item-reason {
  margin-top: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(236, 65, 65, 0.08);
  color: #EC4141;
  font-size: 12px;
  line-height: 1.5;
  word-break: break-all;
}
.mobile-item-foot {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  margin-top: 8px;
  color: var(--text-muted);
  font-size: 12px;
}

/* 头像预览 */
.mobile-avatar-preview {
  position: fixed;
  inset: 0;
  z-index: 10001;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  padding: 40px;
}
.mobile-avatar-preview img {
  width: 240px;
  height: 240px;
  border-radius: 50%;
  object-fit: cover;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.4);
}
</style>
