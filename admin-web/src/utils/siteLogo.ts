import { ref } from 'vue'

/**
 * 站点 Logo 动态配置
 * 登录页、侧边栏、favicon 共用，从服务端 get_site_logo 拉取；
 * 后台 Logo 配置页上传后通过 setSiteLogo 即时同步。
 */
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

/** 拉取服务端配置的站点 Logo（失败回退默认 logo） */
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
    /* 网络异常时保留当前 logo */
  }
  applyFavicon(siteLogoUrl.value)
  return siteLogoUrl.value
}