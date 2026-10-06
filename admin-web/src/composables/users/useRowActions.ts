import { changeCiyuanxiId, deleteUser as deleteUserApi, deleteUserAvatar, toggleUserStatus } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'
import { webActionMenu, webConfirm, webInfo, webPrompt } from '@/utils/webDialog'
import { useAuthStore } from '@/stores/auth'

interface RowActionOptions {
  loadUsers: () => Promise<void>
  openNicknameModal: (u: User) => void
  openEmailModal: (u: User) => void
  openResetModal: (u: User) => void
  viewPlugins: (u: User) => Promise<void>
  viewPlaylists: (u: User) => Promise<void>
  viewFavorites: (u: User) => Promise<void>
  openDeviceModal: (u: User) => Promise<void>
}

// 行内操作菜单与账号状态操作（跨子域分发由视图注入）
export function useRowActions(options: RowActionOptions) {
  const auth = useAuthStore()
  const { loadUsers, openNicknameModal, openEmailModal, openResetModal, viewPlugins, viewPlaylists, viewFavorites, openDeviceModal } = options

  async function openRowMenu(u: User) {
    if (auth.isGuest) {
      await webInfo('访客账号仅可查看，无法查看或操作详情。', { title: '权限不足', confirmText: '知道了' })
      return
    }
    const action = await webActionMenu(`用户操作 · ${u.nickname || u.username}`, [
      { key: 'toggle', label: u.status != 0 ? '禁用用户' : '启用用户', danger: u.status != 0, success: u.status == 0 },
      { key: 'nickname', label: '修改昵称' },
      { key: 'ciyuanxi', label: '修改弦予号' },
      { key: 'email', label: '修改邮箱' },
      { key: 'reset', label: '重置听歌时长' },
      { key: 'plugins', label: '查看插件' },
      { key: 'playlists', label: '查看歌单' },
      { key: 'favorites', label: '查看收藏' },
      { key: 'device', label: '设备信息' },
      { key: 'avatar', label: '删除头像', danger: true, show: !!u.avatar_url },
      { key: 'delete', label: '删除用户', danger: true },
    ])
    if (!action) return
    switch (action) {
      case 'toggle': await toggleStatus(u); break
      case 'nickname': openNicknameModal(u); break
      case 'ciyuanxi': await changeCiyuanxi(u); break
      case 'email': openEmailModal(u); break
      case 'reset': openResetModal(u); break
      case 'plugins': await viewPlugins(u); break
      case 'playlists': await viewPlaylists(u); break
      case 'favorites': await viewFavorites(u); break
      case 'device': await openDeviceModal(u); break
      case 'avatar': await deleteAvatar(u); break
      case 'delete': await deleteUser(u); break
    }
  }

  async function toggleStatus(u: User) {
    const newStatus = u.status != 0 ? 0 : 1
    let reason = ''
    if (newStatus === 0) {
      const input = await webPrompt(`请输入封禁用户 "${u.nickname || u.username}" 的原因：`, '', { title: '封禁用户', placeholder: '封禁原因（必填）' })
      if (input === null) return
      reason = input.trim()
      if (!reason) {
        showToast('封禁原因不能为空')
        return
      }
    }
    const res = await toggleUserStatus({ id: u.id, status: newStatus, reason })
    if (res.code === 200) {
      showToast(newStatus ? '已启用' : '已禁用', 'success')
      u.status = newStatus
      u.ban_reason = newStatus ? '' : reason
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  async function changeCiyuanxi(u: User) {
    const input = await webPrompt(`请输入 "${u.nickname || u.username}" 的新弦予号：`, u.ciyuanxi_id || '', { title: '修改弦予号', placeholder: '6-20 位，仅含字母或数字' })
    if (input === null) return
    const newId = input.trim()
    if (!newId) {
      showToast('请输入弦予号', 'error')
      return
    }
    if (!/^[a-zA-Z0-9]{6,20}$/.test(newId)) {
      showToast('弦予号需 6-20 位，仅含字母或数字', 'error')
      return
    }
    const ok = await webConfirm(`确定将 "${u.nickname || u.username}" 的弦予号由 "${u.ciyuanxi_id || '-'}" 修改为 "${newId}" 吗？`, { title: '修改弦予号', confirmText: '确认修改' })
    if (!ok) return
    const res = await changeCiyuanxiId({ user_id: u.id, new_ciyuanxi_id: newId })
    if (res.code === 200) {
      showToast('弦予号已修改', 'success')
      u.ciyuanxi_id = newId
    } else {
      showToast(res.msg || '修改失败', 'error')
    }
  }

  async function deleteUser(u: User) {
    const ok = await webConfirm(`确定删除用户 "${u.nickname || u.username}" 吗？此操作不可恢复。`, { title: '删除用户', confirmText: '确认删除' })
    if (!ok) return
    const res = await deleteUserApi({ id: u.id })
    if (res.code === 200) {
      showToast('删除成功', 'success')
      loadUsers()
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  async function deleteAvatar(u: User) {
    const ok = await webConfirm(`确定删除用户 "${u.nickname || u.username}" 的头像吗？`, { title: '删除头像', confirmText: '确认删除' })
    if (!ok) return
    const res = await deleteUserAvatar({ user_id: u.id })
    if (res.code === 200) {
      showToast('头像已删除', 'success')
      u.avatar_url = ''
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  return { openRowMenu, toggleStatus, changeCiyuanxi, deleteUser, deleteAvatar }
}
