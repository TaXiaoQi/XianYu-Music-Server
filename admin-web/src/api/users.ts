import { adminApi } from './client'

// ===== 类型定义 =====
export interface User {
  id: number
  username: string
  email: string
  email_verified: number
  status: number
  listen_duration: number
  created_at: string
  avatar_url: string
  ciyuanxi_id: string
  master_quota: number
  [key: string]: any
}

export interface Plugin {
  name: string
  format: string
  version: string
  author: string
  description: string
  enabled: boolean
  scriptSize: number
}

export interface UserStats {
  total: number
  normal: number
  banned: number
}

export interface UserListData {
  total: number
  page: number
  total_pages: number
  list: User[]
}

// ===== 用户列表与统计 =====
export async function getUsers(params: { page: number; page_size: number; keyword: string }) {
  return adminApi<UserListData>('get_users', params)
}

export async function getUserStats() {
  return adminApi<UserStats>('get_user_stats')
}

// ===== 账号操作 =====
export async function addUser(params: { username: string; nickname: string; password: string; email: string }) {
  return adminApi<{ ciyuanxi_id?: string }>('add_user', params)
}

export async function deleteUser(params: { id: number }) {
  return adminApi('delete_user', params)
}

export async function deleteUserAvatar(params: { user_id: number }) {
  return adminApi('delete_user_avatar', params)
}

export async function toggleUserStatus(params: { id: number; status: number; reason: string }) {
  return adminApi('toggle_user_status', params)
}

export async function changeUserNickname(params: { id: number; new_nickname: string; reason: string }) {
  return adminApi('change_user_nickname', params)
}

export async function changeUserEmail(params: { user_id: number; new_email: string }) {
  return adminApi('change_user_email', params)
}

export async function changeCiyuanxiId(params: { user_id: number; new_ciyuanxi_id: string }) {
  return adminApi('change_ciyuanxi_id', params)
}

export async function resetListenDuration(params: { user_id: number; ciyuanxi_id: string; reason: string }) {
  return adminApi('reset_listen_duration', params)
}

export async function deleteEmptyFavoritePlaylists() {
  return adminApi<{ deleted_count?: number; total_scanned?: number }>('delete_empty_favorite_playlists')
}

// ===== 插件查看 =====
export async function getUserPlugins(params: { user_id: number }) {
  return adminApi<any>('get_user_plugins', params)
}

// ===== 设备管理 =====
export async function getUserDevices(params: { user_id: number }) {
  return adminApi<any>('get_user_devices', params)
}

export async function banDevice(params: { device_id: string; reason: string }) {
  return adminApi('ban_device', params)
}

export async function unbanDevice(params: { id?: number; device_id: string }) {
  return adminApi('unban_device', params)
}

export async function listBannedDevices(params: { page: number; page_size: number }) {
  return adminApi<any>('list_banned_devices', params)
}
