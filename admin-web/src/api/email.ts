/**
 * 邮箱注册登录测试 - API 客户端
 * 调用公共 API 端点（/api?action=xxx），免签名
 */

const PUBLIC_API = '/api'

export interface EmailApiResponse<T = any> {
  code: number
  msg: string
  data: T | null
}

export interface EmailUser {
  id: number
  email: string
  nickname: string
}

export interface EmailProfile {
  id: number
  email: string
  nickname: string
  status: number
  created_at: string
  last_login: string
  logs: Array<{ action: string; detail: string; created_at: string }>
}

export function getEmailToken(): string | null {
  return localStorage.getItem('email_token')
}

export function setEmailToken(token: string): void {
  localStorage.setItem('email_token', token)
}

export function clearEmailToken(): void {
  localStorage.removeItem('email_token')
  localStorage.removeItem('email_user')
}

export function getEmailUser(): EmailUser | null {
  const raw = localStorage.getItem('email_user')
  if (!raw) return null
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

export function setEmailUser(user: EmailUser): void {
  localStorage.setItem('email_user', JSON.stringify(user))
}

async function emailApi<T = any>(action: string, data: Record<string, any> = {}): Promise<EmailApiResponse<T>> {
  const url = `${PUBLIC_API}?action=${encodeURIComponent(action)}`
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    })
    return await res.json()
  } catch {
    return { code: 500, msg: '网络错误，请检查服务是否启动', data: null }
  }
}

/** 发送验证码 */
export async function sendCode(email: string): Promise<EmailApiResponse> {
  return emailApi('email_send_code', { email })
}

/** 注册 */
export async function emailRegister(email: string, code: string, password: string, password2: string, nickname: string): Promise<EmailApiResponse> {
  return emailApi('email_register', { email, code, password, password2, nickname })
}

/** 登录 */
export async function emailLogin(email: string, password: string): Promise<EmailApiResponse<{ token: string; user: EmailUser }>> {
  return emailApi('email_login', { email, password })
}

/** 重置密码 */
export async function emailResetPassword(email: string, code: string, password: string, password2: string): Promise<EmailApiResponse> {
  return emailApi('email_reset_password', { email, code, password, password2 })
}

/** 获取用户信息 */
export async function emailGetProfile(): Promise<EmailApiResponse<EmailProfile>> {
  const token = getEmailToken()
  return emailApi('email_get_profile', { token })
}

/** Toast 提示 */
export function emailToast(msg: string, type: 'success' | 'error' = 'error'): void {
  const t = document.createElement('div')
  t.className = `toast ${type}`
  t.textContent = msg
  document.body.appendChild(t)
  setTimeout(() => {
    t.style.opacity = '0'
    setTimeout(() => t.remove(), 300)
  }, 3000)
}
