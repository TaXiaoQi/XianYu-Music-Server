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

export function getAdminUser(): { id: number; username: string; role: string; avatar_url?: string } | null {
  const raw = localStorage.getItem('admin_user')
  if (!raw) return null
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

export function setAdminUser(user: { id: number; username: string; role: string; avatar_url?: string }): void {
  localStorage.setItem('admin_user', JSON.stringify(user))
}

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

// fetch 拿不到上传进度，大文件上传改用 XHR；返回 promise + abort 以支持后台取消
export interface UploadHandle {
  promise: Promise<ApiResponse>
  abort: () => void
}

export function adminApiUpload(
  action: string,
  data: Record<string, any>,
  onProgress?: (percent: number) => void,
): UploadHandle {
  const xhr = new XMLHttpRequest()
  const promise = new Promise<ApiResponse>((resolve) => {
    const token = getToken()
    xhr.open('POST', `${ADMIN_API}?action=${encodeURIComponent(action)}`)
    xhr.setRequestHeader('Content-Type', 'application/json')
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    }
    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100))
      }
    }
    xhr.onload = () => {
      try {
        const json: ApiResponse = JSON.parse(xhr.responseText)
        if (json.code === 401) {
          clearToken()
          if (window.location.pathname !== '/login') {
            window.location.href = '/login'
          }
        }
        resolve(json)
      } catch {
        resolve({ code: 500, msg: '网络错误，请检查服务是否启动', data: null })
      }
    }
    xhr.onerror = () => resolve({ code: 500, msg: '网络错误，请检查服务是否启动', data: null })
    xhr.onabort = () => resolve({ code: 499, msg: '上传已取消', data: null })
    xhr.send(JSON.stringify(data))
  })
  return { promise, abort: () => xhr.abort() }
}
