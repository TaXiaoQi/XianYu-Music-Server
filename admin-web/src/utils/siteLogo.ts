import { ref } from 'vue'

const DEFAULT_LOGO = '/logo.png'
export const siteLogoUrl = ref<string>(DEFAULT_LOGO)

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

export async function loadSiteLogo(): Promise<string> {
  try {
    const res = await fetch('/api?action=get_site_logo', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    })
    const json = await res.json()
    if (json && json.code === 200 && json.data && json.data.logo_url) {
      setSiteLogo(json.data.logo_url)
      return json.data.logo_url
    }
  } catch {
  }
  applyFavicon(siteLogoUrl.value)
  return siteLogoUrl.value
}