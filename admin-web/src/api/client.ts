/**
 * 后台 API 客户端
 * 统一封装 fetch 请求，自动携带 JWT token
 */

const ADMIN_API = '/admin/api'

export interface ApiResponse<T = any> {
  code: number
  msg: string
  data: T | null
}

export function getToken(): string | null {
  return localStorage.getItem('admin_token')
}

export function setToken(token: string): void {
  localStorage.setItem('admin_token', token)
}

export function clearToken(): void {
  localStorage.removeItem('admin_token')
  localStorage.removeItem('admin_user')
}

export function getAdminUser(): { id: number; username: string; role: string } | null {
  const raw = localStorage.getItem('admin_user')
  if (!raw) return null
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

export function setAdminUser(user: { id: number; username: string; role: string }): void {
  localStorage.setItem('admin_user', JSON.stringify(user))
}

/**
 * 调用后台 API
 * @param action 后台 action 名称
 * @param data 请求体数据（会被 JSON 序列化）
 */
export async function adminApi<T = any>(action: string, data: Record<string, any> = {}): Promise<ApiResponse<T>> {
  const token = getToken()
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const url = `${ADMIN_API}?action=${encodeURIComponent(action)}`
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(data),
    })
    const json: ApiResponse<T> = await res.json()

    // 401 时清除 token 并跳转登录
    if (json.code === 401) {
      clearToken()
      if (window.location.pathname !== '/login') {
        window.location.href = '/login'
      }
    }

    return json
  } catch (err) {
    return { code: 500, msg: '网络错误，请检查服务是否启动', data: null }
  }
}

/**
 * Toast 提示
 */
export function showToast(msg: string, type: 'success' | 'error' = 'error'): void {
  const t = document.createElement('div')
  t.className = `toast ${type}`
  t.textContent = msg
  document.body.appendChild(t)
  setTimeout(() => {
    t.style.opacity = '0'
    setTimeout(() => t.remove(), 300)
  }, 3000)
}
