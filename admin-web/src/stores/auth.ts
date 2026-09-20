import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminApi, getToken, setToken, clearToken, getAdminUser, setAdminUser } from '@/api/client'
import { clearAdminActivity, markAdminActivity } from '@/utils/adminIdleLogout'

export interface AdminUser {
  id: number
  username: string
  role: string
  avatar_url?: string
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(getToken())
  const user = ref<AdminUser | null>(getAdminUser())

  const isLoggedIn = computed(() => !!token.value)
  const isSuper = computed(() => user.value?.role === 'super_admin')
  const isGuest = computed(() => user.value?.role === 'guest')
  const canWrite = computed(() => user.value?.role !== 'guest')
  const roleLabel = computed(() => {
    switch (user.value?.role) {
      case 'super_admin': return '超级管理'
      case 'admin': return '一级管理'
      case 'admin2': return '二级管理'
      case 'guest': return '三级访客'
      default: return user.value?.role || ''
    }
  })

  async function login(username: string, password: string): Promise<{ success: boolean; msg: string; mustChangePassword?: boolean }> {
    const res = await adminApi<{ token: string; admin_id: number; username: string; role: string; avatar_url?: string; expires_in: number; must_change_password?: boolean }>('admin_login', {
      username,
      password,
    })
    if (res.code === 200 && res.data) {
      token.value = res.data.token
      setToken(res.data.token)
      markAdminActivity()
      const u: AdminUser = {
        id: res.data.admin_id,
        username: res.data.username,
        role: res.data.role,
        avatar_url: res.data.avatar_url || '',
      }
      user.value = u
      setAdminUser(u)
      return { success: true, msg: '登录成功', mustChangePassword: !!res.data.must_change_password }
    }
    return { success: false, msg: res.msg || '登录失败' }
  }

  async function logout(): Promise<void> {
    await adminApi('admin_logout')
    token.value = null
    user.value = null
    clearToken()
    clearAdminActivity()
  }

  function updateUser(patch: Partial<AdminUser> & { id: number }): void {
    if (!user.value) return
    user.value = { ...user.value, ...patch }
    setAdminUser(user.value)
  }

  return { token, user, isLoggedIn, isSuper, isGuest, canWrite, roleLabel, login, logout, updateUser }
})
