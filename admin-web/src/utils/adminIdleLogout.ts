import type { Router } from 'vue-router'

export const CLEAR_LOGIN_FORM_KEY = 'admin_clear_login_form'

/**
 * 已完全取消「闲置自动下线」。
 * 后台只在服务端 token 真实失效（接口返回 401）时由 client.ts 统一处理下线，
 * 不再根据前端本地闲置时长主动踢人。
 */

export function markAdminActivity(): void {
  // 无操作
}

export function clearAdminActivity(): void {
  // 无操作
}

export function isAdminSessionExpired(): boolean {
  return false
}

/** 保留函数签名，兼容历史上可能残留的调用点；实际不会再有闲置触发 */
export function logoutByIdleTimeout(router?: Router): void {
  if (window.location.pathname === '/login') return
  const target = { path: '/login', query: { timeout: '1' } }
  if (router) {
    router.replace(target)
    return
  }
  window.location.href = '/login?timeout=1'
}

export function initAdminIdleLogout(_router?: Router): void {
  // 无操作：不再注册闲置定时器与 activity 监听
}