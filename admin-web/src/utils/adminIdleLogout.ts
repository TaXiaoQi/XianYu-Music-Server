import type { Router } from 'vue-router'

export const CLEAR_LOGIN_FORM_KEY = 'admin_clear_login_form'

export function markAdminActivity(): void {
}

export function clearAdminActivity(): void {
}

export function isAdminSessionExpired(): boolean {
  return false
}

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
}