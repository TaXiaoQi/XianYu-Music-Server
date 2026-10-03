// 版本维度元数据与工具函数（平台 / 系统 / 架构 / 渠道）

export type PlatformKey = 'desktop' | 'mobile' | 'watch'

export const PLATFORMS: { key: PlatformKey; label: string }[] = [
  { key: 'desktop', label: '桌面端' },
  { key: 'mobile', label: '移动端' },
  { key: 'watch', label: '腕上端' },
]

export function platformOf(item: any): PlatformKey {
  const p = item?.platform
  return p === 'mobile' || p === 'watch' ? p : 'desktop'
}

export function channelOf(item: any): 'stable' | 'beta' {
  return item?.channel === 'beta' ? 'beta' : 'stable'
}

export function platformLabelKey(key: string): string {
  return PLATFORMS.find(p => p.key === key)?.label || '桌面端'
}

export const SYSTEM_META: Record<PlatformKey, { key: string; label: string }[]> = {
  desktop: [
    { key: 'windows', label: 'Windows' },
    { key: 'linux', label: 'Linux' },
    { key: 'macos', label: 'macOS' },
  ],
  mobile: [
    { key: 'android', label: 'Android' },
    { key: 'harmonyos', label: '鸿蒙' },
    { key: 'ios', label: 'iOS' },
  ],
  watch: [
    { key: 'wearos', label: 'WearOS' },
    { key: 'ohos', label: '鸿蒙' },
    { key: 'watchos', label: 'watchOS' },
  ],
}

export function defaultSystem(platform: PlatformKey): string {
  if (platform === 'mobile') return 'android'
  if (platform === 'watch') return 'wearos'
  return 'windows'
}

export function systemOf(item: any): string {
  const s = item?.system
  return s || defaultSystem(platformOf(item))
}

export function systemLabelKey(platform: PlatformKey, system: string): string {
  return SYSTEM_META[platform]?.find(s => s.key === system)?.label || '默认'
}

export const ARCH_META: Record<PlatformKey, { key: string; label: string }[]> = {
  desktop: [
    { key: 'x86', label: 'x86_64' },
    { key: 'arm64', label: 'ARM64' },
  ],
  mobile: [
    { key: 'arm64', label: 'ARM64' },
  ],
  watch: [
    { key: 'arm32', label: 'ARM32' },
    { key: 'arm64', label: 'ARM64' },
  ],
}

export function defaultArch(platform: PlatformKey): string {
  if (platform === 'desktop') return 'x86'
  return 'arm64'
}

export function archOf(item: any): string {
  const a = item?.arch
  if (a === 'x86' || a === 'arm32' || a === 'arm64') return a
  return defaultArch(platformOf(item))
}

export function archLabelOf(item: any): string {
  const a = archOf(item)
  return a === 'x86' ? 'x86_64' : a === 'arm32' ? 'ARM32' : 'ARM64'
}

export function platformLabelOf(item: any): string {
  return platformLabelKey(platformOf(item))
}

export function systemLabelOf(item: any): string {
  return systemLabelKey(platformOf(item), systemOf(item))
}
