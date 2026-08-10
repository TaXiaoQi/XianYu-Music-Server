import type { Router } from 'vue-router'
import { clearToken, getToken } from '@/api/client'

const ADMIN_IDLE_TIMEOUT_MS = 20 * 60 * 1000
const CHECK_INTERVAL_MS = 30 * 1000
const LAST_ACTIVE_KEY = 'admin_last_active_at'
export const CLEAR_LOGIN_FORM_KEY = 'admin_clear_login_form'
const ACTIVITY_EVENTS = ['mousedown', 'mousemove', 'keydown', 'scroll', 'touchstart', 'click', 'wheel']

let checkTimer: number | null = null
let lastMarkAt = 0

function now(): number {
  return Date.now()
}

export function markAdminActivity(): void {
  if (!getToken()) return
  const t = now()
  if (t - lastMarkAt < 1000) return
  lastMarkAt = t
  localStorage.setItem(LAST_ACTIVE_KEY, String(t))
}

export function clearAdminActivity(): void {
  localStorage.removeItem(LAST_ACTIVE_KEY)
  lastMarkAt = 0
}

export function isAdminSessionExpired(): boolean {
  if (!getToken()) return false
  const raw = localStorage.getItem(LAST_ACTIVE_KEY)
  if (!raw) {
    markAdminActivity()
    return false
  }
  const lastActiveAt = Number(raw)
  if (!Number.isFinite(lastActiveAt) || lastActiveAt <= 0) {
    clearAdminActivity()
    markAdminActivity()
    return false
  }
  return now() - lastActiveAt >= ADMIN_IDLE_TIMEOUT_MS
}

export function logoutByIdleTimeout(router?: Router): void {
  clearToken()
  clearAdminActivity()
  sessionStorage.setItem(CLEAR_LOGIN_FORM_KEY, String(Date.now()))
  window.dispatchEvent(new CustomEvent('admin-login-form-clear'))
  if (window.location.pathname === '/login') return
  const target = { path: '/login', query: { timeout: '1' } }
  if (router) {
    router.replace(target)
    return
  }
  window.location.href = '/login?timeout=1'
}

export function initAdminIdleLogout(router: Router): void {
  ACTIVITY_EVENTS.forEach(eventName => {
    window.addEventListener(eventName, markAdminActivity, { passive: true })
  })

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' && isAdminSessionExpired()) {
      logoutByIdleTimeout(router)
    }
  })

  window.addEventListener('focus', () => {
    if (isAdminSessionExpired()) {
      logoutByIdleTimeout(router)
    }
  })

  if (checkTimer !== null) {
    window.clearInterval(checkTimer)
  }
  checkTimer = window.setInterval(() => {
    if (isAdminSessionExpired()) {
      logoutByIdleTimeout(router)
    }
  }, CHECK_INTERVAL_MS)
}
