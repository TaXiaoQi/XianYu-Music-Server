/**
 * 系统版本展示格式化。
 * 旧版移动端上报的系统版本是纯数字（如 13、16），分不清平台；
 * 展示时给纯数字开头的版本补 Android 前缀（→ Android 13），
 * 已带平台名的值（如 Android 13 (API 33)、Windows 11）原样返回。
 */
export function formatOsVersion(v?: string | null): string {
  const s = (v || '').trim()
  if (!s) return ''
  return /^\d/.test(s) ? `Android ${s}` : s
}
