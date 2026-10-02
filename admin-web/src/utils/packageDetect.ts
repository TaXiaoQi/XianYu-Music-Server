// 安装包文件名识别：从文件名推断 平台 / 系统 / 架构，识别结果仍可手动修正。
// 命名示例：
//   XianYu-Music_1.2.0_x64-setup.exe          → 桌面端 · Windows · x86_64
//   XianYu-Music_1.2.0_aarch64-setup.exe      → 桌面端 · Windows · ARM64
//   腕上端子v1.0.0-Watch-arm64.apk            → 腕上端 · WearOS · ARM64
//   app-armeabi-v7a-release.apk               → 移动端 · Android · ARM32(归一为平台支持项)
//   XianYu-Music_1.2.0_aarch64.dmg            → 桌面端 · macOS · ARM64

export type PlatformKey = 'desktop' | 'mobile' | 'watch'

export const PLATFORM_OPTIONS: { key: PlatformKey; label: string }[] = [
  { key: 'desktop', label: '桌面端' },
  { key: 'mobile', label: '移动端' },
  { key: 'watch', label: '腕上端' },
]

export const SYSTEM_OPTIONS: Record<PlatformKey, { key: string; label: string }[]> = {
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

export const ARCH_OPTIONS: Record<PlatformKey, { key: string; label: string }[]> = {
  desktop: [
    { key: 'x86', label: 'x86_64' },
    { key: 'arm64', label: 'ARM64' },
  ],
  mobile: [{ key: 'arm64', label: 'ARM64' }],
  watch: [
    { key: 'arm32', label: 'ARM32' },
    { key: 'arm64', label: 'ARM64' },
  ],
}

export function platformLabelOf(platform: PlatformKey | ''): string {
  return PLATFORM_OPTIONS.find(p => p.key === platform)?.label || ''
}

export function systemLabelOf(platform: PlatformKey | '', system: string): string {
  if (!platform) return system
  return SYSTEM_OPTIONS[platform]?.find(s => s.key === system)?.label || system
}

export function archLabelOf(platform: PlatformKey | '', arch: string): string {
  if (!platform) return arch
  const hit = ARCH_OPTIONS[platform].find(a => a.key === arch)
  return hit ? hit.label : arch
}

export function defaultSystemOf(platform: PlatformKey): string {
  if (platform === 'mobile') return 'android'
  if (platform === 'watch') return 'wearos'
  return 'windows'
}

export function defaultArchOf(platform: PlatformKey): string {
  if (platform === 'desktop') return 'x86'
  return 'arm64'
}

export interface PackageMeta {
  platform: PlatformKey
  system: string
  arch: string
}

// 支持的安装包扩展名（含压缩包壳与常见移动端格式）
export const PACKAGE_ALLOWED_EXT = ['exe', 'msi', 'zip', '7z', 'rar', 'dmg', 'pkg', 'apk', 'hap', 'ipa', 'deb', 'appimage']

export function packageMetaLabel(meta: PackageMeta): string {
  return `${platformLabelOf(meta.platform)} · ${systemLabelOf(meta.platform, meta.system)} · ${archLabelOf(meta.platform, meta.arch)}`
}

// 从文件名识别平台/系统/架构；无法识别平台时返回 null（由调用方提示手动选择）
export function detectPackageMeta(fileName: string): PackageMeta | null {
  const name = fileName.toLowerCase()
  const dot = name.lastIndexOf('.')
  const ext = dot >= 0 ? name.slice(dot + 1) : ''
  const has = (...kws: string[]) => kws.some(k => name.includes(k))

  // 架构关键词（先长后短，避免 x86 抢先命中 x86_64）
  let arch = ''
  if (has('aarch64', 'arm64', 'arm64-v8a', 'armv8')) arch = 'arm64'
  else if (has('x86_64', 'x64', 'win64', 'amd64')) arch = 'x86'
  else if (has('arm32', 'armv7', 'armeabi')) arch = 'arm32'
  else if (/(^|[^a-z0-9])x86([^a-z0-9]|$)/.test(name) || has('win32', 'ia32')) arch = 'x86'

  // 平台 + 系统
  let platform: PlatformKey | '' = ''
  let system = ''
  if (ext === 'dmg' || ext === 'pkg' || has('macos', 'darwin')) {
    platform = 'desktop'
    system = 'macos'
  } else if (ext === 'msi' || ext === 'exe' || has('windows')) {
    platform = 'desktop'
    system = 'windows'
  } else if (ext === 'deb' || ext === 'appimage' || has('linux')) {
    platform = 'desktop'
    system = 'linux'
  } else if (ext === 'apk' || ext === 'hap' || has('android', 'wearos')) {
    if (has('ohos', 'harmony')) {
      system = 'ohos'
      platform = has('watch', 'wear') ? 'watch' : 'mobile'
    } else if (has('watch', 'wear')) {
      platform = 'watch'
      system = 'wearos'
    } else {
      platform = 'mobile'
      system = 'android'
    }
  } else if (ext === 'ipa' || has('ios', 'iphone', 'ipad')) {
    platform = 'mobile'
    system = 'ios'
  } else if (has('watch', 'wear')) {
    platform = 'watch'
    system = 'wearos'
  } else if (has('setup', 'installer', 'install')) {
    platform = 'desktop'
    system = 'windows'
  } else {
    return null
  }

  // 归一化到各平台支持的架构
  if (platform === 'mobile') arch = 'arm64'
  else if (platform === 'watch') {
    if (arch === 'x86') arch = 'arm64'
  } else if (platform === 'desktop') {
    if (arch === 'arm32') arch = 'x86'
  }
  if (!arch) arch = defaultArchOf(platform)

  return { platform, system, arch }
}
