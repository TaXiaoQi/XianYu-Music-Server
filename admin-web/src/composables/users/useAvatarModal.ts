import { ref } from 'vue'
import { deleteUserAvatar } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'

interface AvatarModalOptions {
  loadUsers: () => Promise<void>
}

// 头像大图查看与删除
export function useAvatarModal({ loadUsers }: AvatarModalOptions) {
  const showAvatarModal = ref(false)
  const avatarViewUser = ref<User | null>(null)
  const avatarDeleting = ref(false)

  function openAvatarView(u: User) {
    avatarViewUser.value = u
    showAvatarModal.value = true
  }

  async function confirmDeleteAvatar() {
    if (!avatarViewUser.value) return
    avatarDeleting.value = true
    const res = await deleteUserAvatar({ user_id: avatarViewUser.value.id })
    avatarDeleting.value = false
    if (res.code === 200) {
      showToast('头像已删除', 'success')
      avatarViewUser.value.avatar_url = ''
      showAvatarModal.value = false
      loadUsers()
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  return { showAvatarModal, avatarViewUser, avatarDeleting, openAvatarView, confirmDeleteAvatar }
}
