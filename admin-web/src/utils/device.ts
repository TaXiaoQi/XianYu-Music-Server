export function isMobileBrowser(): boolean {
  if (typeof window === 'undefined') return false
  const ua = navigator.userAgent || ''
  const byUa = /Android|webOS|iPhone|iPod|BlackBerry|IEMobile|Opera Mini|Mobile/i.test(ua)
  const byWidth = window.matchMedia('(max-width: 768px)').matches
  return byUa || byWidth
}

export function toMobilePath(path: string): string {
  if (path.startsWith('/m')) return path
  if (path === '/' || path === '/dashboard') return '/m/dashboard'
  return `/m${path}`
}

export function toDesktopPath(path: string): string {
  if (!path.startsWith('/m')) return path
  const next = path.replace(/^\/m/, '') || '/dashboard'
  return next === '/' ? '/dashboard' : next
}
