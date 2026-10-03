import { ref } from 'vue'
import { addUser, changeUserEmail, changeUserNickname, resetListenDuration } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'
import { formatDuration } from './useUserList'

interface AccountModalsOptions {
  loadUsers: () => Promise<void>
}

// 添加用户 / 修改昵称 / 修改邮箱 / 重置听歌时长 弹窗
export function useAccountModals({ loadUsers }: AccountModalsOptions) {
  // 添加用户
  const showAddModal = ref(false)
  const addLoading = ref(false)
  const addForm = ref({ username: '', nickname: '', password: '', email: '' })

  function openAddModal() {
    addForm.value = { username: '', nickname: '', password: '', email: '' }
    showAddModal.value = true
  }

  async function submitAddUser() {
    const ciyuanxi = addForm.value.username.trim()
    if (ciyuanxi.length < 6) {
      showToast('弦予号至少 6 位')
      return
    }
    if (!/^[a-zA-Z0-9]{6,20}$/.test(ciyuanxi)) {
      showToast('弦予号需 6-20 位，仅含字母或数字')
      return
    }
    if (addForm.value.password.length < 6) {
      showToast('密码至少 6 位')
      return
    }
    if (addForm.value.email && !isValidEmail(addForm.value.email)) {
      showToast('邮箱格式不正确')
      return
    }
    addLoading.value = true
    const res = await addUser({
      username: ciyuanxi,
      nickname: addForm.value.nickname.trim(),
      password: addForm.value.password,
      email: addForm.value.email.trim(),
    })
    addLoading.value = false
    if (res.code === 200) {
      showToast(`添加成功，弦予号: ${res.data ? res.data.ciyuanxi_id : ciyuanxi}`, 'success')
      showAddModal.value = false
      addForm.value = { username: '', nickname: '', password: '', email: '' }
      loadUsers()
    } else {
      showToast(res.msg || '添加失败')
    }
  }

  // 修改昵称
  const showNicknameModal = ref(false)
  const nicknameLoading = ref(false)
  const nicknameForm = ref({ userId: 0, oldNickname: '', ciyuanxiId: '', newNickname: '', reason: '' })

  function openNicknameModal(u: User) {
    nicknameForm.value = {
      userId: u.id,
      oldNickname: u.nickname || u.username || '',
      ciyuanxiId: u.ciyuanxi_id || '',
      newNickname: '',
      reason: '',
    }
    showNicknameModal.value = true
  }

  async function submitNicknameChange() {
    const newNickname = nicknameForm.value.newNickname.trim()
    const reason = nicknameForm.value.reason.trim()
    if (newNickname.length < 2 || newNickname.length > 32) {
      showToast('昵称需 2-32 个字符', 'error')
      return
    }
    if (!/^[a-zA-Z0-9\u4e00-\u9fa5]+$/.test(newNickname)) {
      showToast('昵称仅支持字母、数字、汉字', 'error')
      return
    }
    if (newNickname === nicknameForm.value.oldNickname) {
      showToast('新昵称与当前昵称相同', 'error')
      return
    }
    if (!reason) {
      showToast('修改原因不能为空', 'error')
      return
    }
    nicknameLoading.value = true
    const res = await changeUserNickname({
      id: nicknameForm.value.userId,
      new_nickname: newNickname,
      reason,
    })
    nicknameLoading.value = false
    if (res.code === 200) {
      showToast('昵称已修改，已下发客户端通知', 'success')
      showNicknameModal.value = false
      loadUsers()
    } else {
      showToast(res.msg || '修改失败', 'error')
    }
  }

  // 修改邮箱
  const showEmailModal = ref(false)
  const emailLoading = ref(false)
  const emailForm = ref({ userId: 0, username: '', nickname: '', currentEmail: '', newEmail: '' })

  function openEmailModal(u: User) {
    emailForm.value = { userId: u.id, username: u.nickname || u.username, nickname: u.nickname || u.username, currentEmail: u.email || '', newEmail: '' }
    showEmailModal.value = true
  }

  async function submitEmailChange() {
    if (emailForm.value.newEmail && !isValidEmail(emailForm.value.newEmail)) {
      showToast('邮箱格式不正确')
      return
    }
    emailLoading.value = true
    const res = await changeUserEmail({
      user_id: emailForm.value.userId,
      new_email: emailForm.value.newEmail.trim(),
    })
    emailLoading.value = false
    if (res.code === 200) {
      showToast('邮箱已更新', 'success')
      showEmailModal.value = false
      loadUsers()
    } else {
      showToast(res.msg || '修改失败')
    }
  }

  // 重置听歌时长
  const showResetModal = ref(false)
  const resetLoading = ref(false)
  const resetForm = ref({ userId: 0, username: '', nickname: '', duration: '', ciyuanxiId: '', reason: '' })

  function openResetModal(u: User) {
    resetForm.value = {
      userId: u.id,
      username: u.nickname || u.username,
      nickname: u.nickname || u.username,
      duration: formatDuration(u.listen_duration),
      ciyuanxiId: u.ciyuanxi_id || '',
      reason: '',
    }
    showResetModal.value = true
  }

  async function submitReset() {
    const reason = resetForm.value.reason.trim()
    if (!reason) {
      showToast('请填写清除原因')
      return
    }
    resetLoading.value = true
    const res = await resetListenDuration({
      user_id: resetForm.value.userId,
      ciyuanxi_id: resetForm.value.ciyuanxiId,
      reason,
    })
    resetLoading.value = false
    if (res.code === 200) {
      showToast('重置成功', 'success')
      showResetModal.value = false
      loadUsers()
    } else {
      showToast(res.msg || '重置失败')
    }
  }

  function isValidEmail(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
  }

  return {
    showAddModal, addLoading, addForm, openAddModal, submitAddUser,
    showNicknameModal, nicknameLoading, nicknameForm, openNicknameModal, submitNicknameChange,
    showEmailModal, emailLoading, emailForm, openEmailModal, submitEmailChange,
    showResetModal, resetLoading, resetForm, openResetModal, submitReset,
  }
}
