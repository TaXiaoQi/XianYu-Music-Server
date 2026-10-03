import { computed, ref } from 'vue'
import type { Ref } from 'vue'
import { banDevice, deleteEmptyFavoritePlaylists, deleteUser, resetListenDuration, toggleUserStatus } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'
import { webConfirm, webPrompt } from '@/utils/webDialog'

interface BatchModeOptions {
  users: Ref<User[]>
  loadUsers: () => Promise<void>
}

// 批量管理模式：选择与批量操作
export function useBatchMode({ users, loadUsers }: BatchModeOptions) {
  const isBatchMode = ref(false)
  const selectedIds = ref<Set<number>>(new Set())
  const batchLoading = ref(false)
  const selectedCount = computed(() => selectedIds.value.size)
  const isAllSelected = computed(() => users.value.length > 0 && selectedIds.value.size === users.value.length)

  function enterBatchMode() {
    selectedIds.value = new Set()
    isBatchMode.value = true
  }

  function exitBatchMode() {
    isBatchMode.value = false
    selectedIds.value = new Set()
  }

  function toggleSelect(id: number) {
    const next = new Set(selectedIds.value)
    if (next.has(id)) next.delete(id)
    else next.add(id)
    selectedIds.value = next
  }

  function toggleSelectAll() {
    if (isAllSelected.value) {
      selectedIds.value = new Set()
    } else {
      selectedIds.value = new Set(users.value.map(u => u.id))
    }
  }

  async function batchToggleSelected(status: number) {
    const ids = [...selectedIds.value]
    if (ids.length === 0) return
    const label = status ? '启用' : '禁用'
    let reason = ''
    if (!status) {
      const input = await webPrompt(`请输入对选中的 ${ids.length} 个用户执行封禁的原因：`, '', { title: '批量禁用', placeholder: '封禁原因（必填）' })
      if (input === null) return
      reason = input.trim()
      if (!reason) {
        showToast('封禁原因不能为空')
        return
      }
    }
    const ok = await webConfirm(`确定${label}选中的 ${ids.length} 个用户吗？`, { title: `批量${label}`, confirmText: `确认${label}` })
    if (!ok) return
    batchLoading.value = true
    let fail = 0
    for (const id of ids) {
      const res = await toggleUserStatus({ id, status, reason })
      if (res.code !== 200) fail++
    }
    batchLoading.value = false
    if (fail === 0) {
      showToast(`已${label} ${ids.length} 个用户`, 'success')
    } else {
      showToast(`${ids.length - fail} 个成功，${fail} 个失败`, 'error')
    }
    selectedIds.value = new Set()
    loadUsers()
  }

  async function batchBanDevice() {
    const ids = [...selectedIds.value]
    if (ids.length === 0) return
    const targets: { id: number; name: string; deviceId: string }[] = []
    for (const u of users.value) {
      if (!ids.includes(u.id)) continue
      const deviceId = (u.last_device_id || '').trim()
      if (!deviceId) continue
      targets.push({ id: u.id, name: u.nickname || u.username, deviceId })
    }
    if (targets.length === 0) {
      showToast('选中的用户均无设备ID，无需封禁', 'error')
      return
    }
    const input = await webPrompt(`将封禁选中的 ${targets.length} 个用户最近登录的设备，请输入封禁原因：`, '', { title: '批量封禁设备ID', placeholder: '封禁原因（必填）' })
    if (input === null) return
    const reason = input.trim()
    if (!reason) {
      showToast('封禁原因不能为空')
      return
    }
    const ok = await webConfirm(`确定封禁 ${targets.length} 个用户的设备ID吗？封禁后这些设备将无法登录。`, { title: '批量封禁设备ID', confirmText: '确认封禁' })
    if (!ok) return
    batchLoading.value = true
    let success = 0
    let fail = 0
    for (const t of targets) {
      const res = await banDevice({ device_id: t.deviceId, reason })
      if (res.code === 200) success++
      else fail++
    }
    batchLoading.value = false
    if (fail === 0) {
      showToast(`已封禁 ${success} 个设备ID`, 'success')
    } else {
      showToast(`${success} 个成功，${fail} 个失败`, 'error')
    }
    selectedIds.value = new Set()
  }

  async function batchResetDuration() {
    const ids = [...selectedIds.value]
    if (ids.length === 0) return
    const input = await webPrompt(`将重置选中的 ${ids.length} 个用户的听歌时长，请输入清除原因：`, '', { title: '批量重置听歌时长', placeholder: '清除原因（必填）' })
    if (input === null) return
    const reason = input.trim()
    if (!reason) {
      showToast('清除原因不能为空')
      return
    }
    const ok = await webConfirm(`确定重置选中的 ${ids.length} 个用户的听歌时长吗？重置后时长与新增歌数将清零。`, { title: '批量重置听歌时长', confirmText: '确认重置' })
    if (!ok) return
    batchLoading.value = true
    let fail = 0
    for (const id of ids) {
      const u = users.value.find(x => x.id === id)
      const res = await resetListenDuration({ user_id: id, ciyuanxi_id: u?.ciyuanxi_id || '', reason })
      if (res.code !== 200) fail++
    }
    batchLoading.value = false
    if (fail === 0) {
      showToast(`已重置 ${ids.length} 个用户`, 'success')
    } else {
      showToast(`${ids.length - fail} 个成功，${fail} 个失败`, 'error')
    }
    selectedIds.value = new Set()
    loadUsers()
  }

  async function batchDeleteSelected() {
    const ids = [...selectedIds.value]
    if (ids.length === 0) return
    const ok = await webConfirm(`确定删除选中的 ${ids.length} 个用户吗？此操作不可恢复。`, { title: '批量删除', confirmText: '确认删除' })
    if (!ok) return
    batchLoading.value = true
    let fail = 0
    for (const id of ids) {
      const res = await deleteUser({ id })
      if (res.code !== 200) fail++
    }
    batchLoading.value = false
    if (fail === 0) {
      showToast(`已删除 ${ids.length} 个用户`, 'success')
    } else {
      showToast(`${ids.length - fail} 个成功，${fail} 个失败`, 'error')
    }
    selectedIds.value = new Set()
    loadUsers()
  }

  async function deleteEmptyPlaylists() {
    const ok = await webConfirm('确定删除所有空的"我喜欢的音乐"歌单吗？', { title: '清理空歌单', confirmText: '确认删除' })
    if (!ok) return
    batchLoading.value = true
    const res = await deleteEmptyFavoritePlaylists()
    batchLoading.value = false
    if (res.code === 200 && res.data) {
      const d = res.data
      showToast(`已删除 ${d.deleted_count || 0} 个空歌单（扫描 ${d.total_scanned || 0} 个）`, 'success')
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    isBatchMode, selectedIds, selectedCount, isAllSelected, batchLoading,
    enterBatchMode, exitBatchMode, toggleSelect, toggleSelectAll,
    batchToggleSelected, batchBanDevice, batchResetDuration, batchDeleteSelected, deleteEmptyPlaylists,
  }
}
