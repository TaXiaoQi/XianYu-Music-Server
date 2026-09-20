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

export async function sendCode(email: string, captchaToken = ''): Promise<EmailApiResponse> {
  return emailApi('email_send_code', { email, captcha_token: captchaToken, turnstile_token: captchaToken })
}

export interface CaptchaConfig {
  enabled: boolean
  provider: 'turnstile' | 'hcaptcha' | 'off' | string
  site_key: string
}

export async function getCaptchaConfig(): Promise<EmailApiResponse<CaptchaConfig>> {
  return emailApi('email_get_captcha_config', {})
}

export async function getTurnstileConfig(): Promise<EmailApiResponse<CaptchaConfig>> {
  return getCaptchaConfig()
}

export async function emailRegister(email: string, code: string, password: string, password2: string, nickname: string): Promise<EmailApiResponse> {
  return emailApi('email_register', { email, code, password, password2, nickname })
}

export async function emailLogin(email: string, password: string): Promise<EmailApiResponse<{ token: string; user: EmailUser }>> {
  return emailApi('email_login', { email, password })
}

export async function emailResetPassword(email: string, code: string, password: string, password2: string): Promise<EmailApiResponse> {
  return emailApi('email_reset_password', { email, code, password, password2 })
}

export async function emailGetProfile(): Promise<EmailApiResponse<EmailProfile>> {
  const token = getEmailToken()
  return emailApi('email_get_profile', { token })
}

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
