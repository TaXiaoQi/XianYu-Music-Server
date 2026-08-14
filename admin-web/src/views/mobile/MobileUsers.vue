<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-toolbar">
      <div v-if="!isBatchMode" class="mobile-search-row">
        <input v-model="keyword" class="mobile-input" placeholder="搜索昵称 / 弦予号 / 邮箱" @keyup.enter="loadList" />
          <button class="mobile-btn primary" @click="loadList">搜索</button>
          <button class="mobile-btn" @click="openAdd = !openAdd">{{ openAdd ? '收起新增' : '新增用户' }}</button>
        <router-link to="/m/device-banned" custom v-slot="{ navigate }">
          <button class="mobile-btn" @click="navigate">设备管理</button>
        </router-link>
        <button class="mobile-btn primary" @click="enterBatchMode">批量管理</button>
      </div>
      <div v-else class="mobile-batch-bar">
          <button class="mobile-btn" @click="toggleSelectAll">{{ isAllSelected ? '取消全选' : '全选' }}</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchToggleSelected(0)">封禁</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchToggleSelected(1)">启用</button>
          <button class="mobile-btn" :disabled="selectedCount === 0" @click="batchBanDevice">封禁ID</button>
          <button class="mobile-btn danger" :disabled="selectedCount === 0" @click="batchDeleteSelected">删除</button>
          <button class="mobile-btn danger" @click="deleteEmptyFavorites">清空歌单</button>
          <span class="mobile-batch-count">已选 {{ selectedCount }} 项</span>
          <button class="mobile-btn primary" @click="exitBatchMode">完成</button>
        </div>
      </div>
    <transition name="user-expand">
      <div v-if="openAdd" class="mobile-card mobile-form user-expand-wrap">
        <div class="user-expand-inner">
          <h3 class="mobile-card-title">新增用户</h3>
          <input v-model="addForm.ciyuanxi_id" class="mobile-input" placeholder="弦予号（必填，仅含字母或数字）" />
          <input v-model="addForm.password" class="mobile-input" placeholder="密码（必填）" type="password" />
          <input v-model="addForm.nickname" class="mobile-input" placeholder="昵称（选填，留空默认弦予+号）" />
          <input v-model="addForm.email" class="mobile-input" placeholder="邮箱（选填）" />
          <button class="mobile-btn primary" :disabled="saving" @click="addUser">{{ saving ? '提交中...' : '确认新增' }}</button>
        </div>
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
              <span class="mobile-item-title">昵称：{{ u.nickname || u.username || '-' }}</span>
              <div class="mobile-title-right">
                <span class="mobile-badge" :class="u.status == 1 ? 'green' : 'red'">{{ u.status == 1 ? '正常' : '禁用' }}</span>
              </div>
            </div>
            <div class="mobile-item-sub">弦予号：{{ u.ciyuanxi_id || '-' }}</div>
            <div class="mobile-item-sub mobile-item-verify">
              <span class="mobile-email">邮箱：{{ u.email || '未绑定邮箱' }}</span>
              <span class="mobile-badge" :class="u.email_verified == 1 ? 'green' : ''">{{ u.email_verified == 1 ? '已验证' : '未验证' }}</span>
            </div>
          </div>
        </div>
        <div v-if="u.status == 0 && u.ban_reason" class="mobile-item-reason">封禁原因：{{ u.ban_reason }}</div>
        <div class="mobile-item-foot">
          <div class="mobile-item-foot-info">
            <span>听歌时长 {{ formatDuration(u.listen_duration) }}</span>
            <span>注册 {{ u.created_at || '-' }}</span>
          </div>
          <button class="mobile-op-btn" @click="openActionMenu(u)">操作</button>
        </div>
      </div>
    </div>
    <!-- 插件查看弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="showPluginsModal" class="mobile-dialog-overlay" @click.self="closePlugins">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:440px;max-height:88vh;">
        <div class="mobile-dialog-head">
          <span class="mobile-dialog-head-title">用户插件 - {{ pluginsData.nickname || pluginsData.username || '-' }}</span>
          <button class="mobile-dialog-close" @click="closePlugins">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="mobile-dialog-body popup-body">
          <div v-if="pluginsLoading" class="mobile-empty">加载中...</div>
          <template v-else>
            <div class="popup-meta">
              <span>插件数量：{{ pluginsData.plugin_count || 0 }}</span>
              <span v-if="pluginsData.uploaded_at">上传时间：{{ pluginsData.uploaded_at }}</span>
              <span v-if="pluginsData.ciyuanxi_id">弦予号：{{ pluginsData.ciyuanxi_id }}</span>
            </div>
            <div v-if="pluginsData.plugins && pluginsData.plugins.length > 0" class="popup-table-wrap">
              <table class="popup-table">
                <thead>
                  <tr><th>名称</th><th>格式</th><th>版本</th><th>作者</th><th>状态</th><th>大小</th></tr>
                </thead>
                <tbody>
                  <tr v-for="(p, i) in pluginsData.plugins" :key="i">
                    <td>
                      {{ p.name }}
                      <div v-if="p.description" class="td-desc">{{ p.description }}</div>
                    </td>
                    <td><span class="badge badge-info">{{ p.format }}</span></td>
                    <td>{{ p.version || '-' }}</td>
                    <td>{{ p.author || '-' }}</td>
                    <td>
                      <span :class="['badge', p.enabled ? 'badge-success' : 'badge-error']">{{ p.enabled ? '启用' : '禁用' }}</span>
                    </td>
                    <td>{{ formatScriptSize(p.scriptSize) }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="mobile-empty">该用户暂无插件数据</div>
          </template>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 设备信息弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="showDeviceModal" class="mobile-dialog-overlay" @click.self="closeDevice">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:460px;max-height:88vh;">
        <div class="mobile-dialog-head">
          <span class="mobile-dialog-head-title">设备信息 - {{ deviceData.nickname || deviceData.username || '-' }}</span>
          <button class="mobile-dialog-close" @click="closeDevice">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="mobile-dialog-body popup-body">
          <div v-if="deviceLoading" class="mobile-empty">加载中...</div>
          <template v-else>
            <div class="popup-meta">
              <span>弦予号：{{ deviceData.ciyuanxi_id || '-' }}</span>
              <span v-if="deviceData.last_device_id">设备ID：{{ deviceData.last_device_id }}</span>
              <span v-if="deviceData.is_banned" class="meta-banned">设备已封禁</span>
            </div>

            <div v-if="deviceData.last_device_id" class="popup-actions">
              <button v-if="!deviceData.is_banned" class="mobile-btn danger" @click="banUserDevice(deviceData.last_device_id, deviceData.nickname || deviceData.username)">封禁此设备</button>
              <button v-else class="mobile-btn" @click="unbanUserDevice(deviceData.last_device_id)">解封此设备</button>
            </div>

            <div v-if="deviceData.login_logs && deviceData.login_logs.length > 0" class="popup-section">
              <h4 class="popup-sec-title">登录记录</h4>
              <div class="popup-table-wrap">
                <table class="popup-table">
                  <thead><tr><th>设备ID</th><th>IP</th><th>时间</th></tr></thead>
                  <tbody>
                    <tr v-for="(log, i) in deviceData.login_logs" :key="'l'+i">
                      <td class="td-mono">{{ log.device_id }}</td>
                      <td>{{ log.ip }}</td>
                      <td>{{ log.created_at }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <div v-if="deviceData.open_logs && deviceData.open_logs.length > 0" class="popup-section">
              <h4 class="popup-sec-title">启动记录</h4>
              <div class="popup-table-wrap">
                <table class="popup-table">
                  <thead><tr><th>设备ID</th><th>IP</th><th>版本</th><th>时间</th></tr></thead>
                  <tbody>
                    <tr v-for="(log, i) in deviceData.open_logs" :key="'o'+i">
                      <td class="td-mono">{{ log.device_id }}</td>
                      <td>{{ log.ip }}</td>
                      <td>{{ log.app_version }}</td>
                      <td>{{ log.created_at }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <div v-if="!deviceData.last_device_id" class="mobile-empty">该用户暂无设备记录</div>
          </template>
        </div>
      </div>
    </div>
    </Transition>

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
import { mobileConfirm, mobilePrompt, mobileActionMenu } from '@/utils/mobileDialog'
import './MobilePage.css'
const keyword = ref('')
const loading = ref(false)
const saving = ref(false)
const openAdd = ref(false)
const list = ref<any[]>([])
const addForm = ref({ ciyuanxi_id: '', nickname: '', password: '', email: '' })
const showPluginsModal = ref(false)
const pluginsLoading = ref(false)
const pluginsData = ref<any>({})
const showDeviceModal = ref(false)
const deviceLoading = ref(false)
const deviceData = ref<any>({})
const avatarPreview = ref('')

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
  if (!/^[a-zA-Z0-9]{6,20}$/.test(ciyuanxi)) return showToast('弦予号需 6-20 位，仅含字母或数字')
  if (!addForm.value.password) return showToast('请填写密码')
  saving.value = true
  const res = await adminApi('add_user', { username: ciyuanxi, nickname: addForm.value.nickname.trim(), password: addForm.value.password, email: addForm.value.email.trim() })
  saving.value = false
  if (res.code === 200) { showToast('新增成功', 'success'); openAdd.value = false; addForm.value = { ciyuanxi_id: '', nickname: '', password: '', email: '' }; loadList() } else showToast(res.msg || '新增失败')
}
async function openActionMenu(u: any) {
  const action = await mobileActionMenu(`用户操作 · ${u.nickname || u.username}`, [
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
  if (!/^[a-zA-Z0-9]{6,20}$/.test(newId)) return showToast('弦予号需 6-20 位，仅含字母或数字', 'error')
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
  showPluginsModal.value = true
  pluginsLoading.value = true
  pluginsData.value = { nickname: u.nickname || u.username, username: u.username }
  const res = await adminApi<any>('get_user_plugins', { user_id: u.id })
  pluginsLoading.value = false
  if (res.code === 200 && res.data) {
    pluginsData.value = res.data
  } else {
    showToast(res.msg || '加载插件失败')
  }
}
function closePlugins() {
  if (!pluginsLoading.value) showPluginsModal.value = false
}
function formatScriptSize(size: number | string | undefined): string {
  const n = Number(size) || 0
  if (n <= 0) return '-'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / (1024 * 1024)).toFixed(2)} MB`
}
async function openDeviceInfo(u: any) {
  showDeviceModal.value = true
  await refreshDeviceInfo(u)
}
function closeDevice() {
  if (!deviceLoading.value) showDeviceModal.value = false
}
async function refreshDeviceInfo(u: any) {
  deviceLoading.value = true
  deviceData.value = { nickname: u.nickname || u.username, username: u.username }
  const res = await adminApi<any>('get_user_devices', { user_id: u.id })
  if (res.code === 200 && res.data) {
    deviceData.value = res.data
  } else {
    showToast(res.msg || '加载设备信息失败')
  }
  deviceLoading.value = false
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
    deviceData.value.is_banned = true
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function unbanUserDevice(deviceId: string) {
  if (!deviceId) return showToast('未读取到设备 ID')
  const res = await adminApi('unban_device', { device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    deviceData.value.is_banned = false
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
/* 新增用户面板展开/收起 - 平滑高度动画 */
.user-expand-wrap.mobile-card { padding: 0; }
.user-expand-inner {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 15px;
  overflow: hidden;
  min-height: 0;
}
.user-expand-enter-active,
.user-expand-leave-active {
  display: grid;
  grid-template-rows: 1fr;
  overflow: hidden;
  transition: grid-template-rows 0.3s cubic-bezier(0.16, 1, 0.3, 1),
              opacity 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.user-expand-enter-from,
.user-expand-leave-to {
  grid-template-rows: 0fr;
  opacity: 0;
}

/* 弹窗头部 */
.mobile-dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 16px 18px 0;
}
.mobile-dialog-head-title {
  font-size: 15px;
  font-weight: 850;
  color: var(--text);
  word-break: break-all;
}
.mobile-dialog-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.mobile-dialog-close:hover {
  background: var(--control-bg);
  color: #EC4141;
}

/* 弹窗主体 */
.popup-body {
  padding: 12px 0 0;
  overflow-y: auto;
  max-height: calc(88vh - 60px);
}
.popup-body .mobile-empty {
  padding: 24px 0;
}
.popup-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  padding: 0 18px 12px;
  font-size: 12px;
  color: var(--text-light);
}
.meta-banned {
  color: #EC4141;
  font-weight: 800;
}
.popup-actions {
  display: flex;
  gap: 8px;
  padding: 0 18px 12px;
}
.popup-section {
  padding: 0 18px 14px;
}
.popup-sec-title {
  margin: 0 0 8px;
  font-size: 13px;
  font-weight: 850;
  color: var(--text);
}
.popup-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: 12px;
}
.popup-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  min-width: 420px;
}
.popup-table th,
.popup-table td {
  padding: 9px 10px;
  text-align: left;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.popup-table thead th {
  background: var(--control-bg);
  color: var(--text-light);
  font-weight: 800;
  font-size: 11px;
}
.popup-table tbody tr:last-child td {
  border-bottom: 0;
}
.td-desc {
  font-size: 11px;
  color: var(--text-muted);
  white-space: normal;
}
.td-mono {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 11px;
}
.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 800;
}
.badge-info { background: rgba(59, 130, 246, 0.10); color: #3b82f6; }
.badge-success { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.badge-error { background: rgba(236, 65, 65, 0.10); color: #EC4141; }

/* 顶部工具栏：搜索框 + 搜索/新增/批量按钮同行，按钮在右侧 */
.mobile-toolbar {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.mobile-search-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.mobile-search-row .mobile-input {
  flex: 1;
  min-width: 0;
}
.mobile-search-row .mobile-btn {
  flex: 0 0 auto;
  white-space: nowrap;
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
  align-self: center;
  margin-top: 0;
}

/* 头像 */
.mobile-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  object-fit: cover;
  flex: none;
  cursor: pointer;
  align-self: center;
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
.mobile-title-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 0 0 auto;
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
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 10px;
  color: var(--text-muted);
  font-size: 12px;
}
.mobile-item-foot-info {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  min-width: 0;
}
.mobile-op-btn {
  flex: 0 0 auto;
  border: none;
  border-radius: 999px;
  padding: 8px 18px;
  background: #EC4141;
  color: #fff;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(236, 65, 65, 0.28);
  transition: transform 0.16s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)),
              opacity 0.16s, box-shadow 0.16s;
}
.mobile-op-btn:active {
  transform: scale(0.94);
  opacity: 0.85;
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
