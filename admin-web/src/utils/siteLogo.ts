import { ref } from 'vue'

const DEFAULT_LOGO = '/logo.png'
export const siteLogoUrl = ref<string>(DEFAULT_LOGO)
// 站点备案信息（未配置时为空串，页脚回退占位样式）
export const siteIcp = ref<string>('')
export const sitePs = ref<string>('')

function applyFavicon(url: string): void {
  let link = document.querySelector<HTMLLinkElement>('link[rel="icon"]')
  if (!link) {
    link = document.createElement('link')
    link.rel = 'icon'
    link.type = 'image/png'
    document.head.appendChild(link)
  }
  link.href = url
}

export function setSiteLogo(url: string): void {
  if (!url) return
  siteLogoUrl.value = url
  applyFavicon(url)
}

export function setSiteBeian(icp: string, ps: string): void {
  siteIcp.value = icp || ''
  sitePs.value = ps || ''
}

export async function loadSiteLogo(): Promise<string> {
  try {
    const res = await fetch('/api?action=get_site_logo', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    })
    const json = await res.json()
    if (json && json.code === 200 && json.data) {
      setSiteBeian(json.data.icp, json.data.ps)
      if (json.data.logo_url) {
        setSiteLogo(json.data.logo_url)
        return json.data.logo_url
      }
    }
  } catch {
  }
  applyFavicon(siteLogoUrl.value)
  return siteLogoUrl.value
}