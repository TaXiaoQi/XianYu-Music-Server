export function formatOsVersion(v?: string | null): string {
  const s = (v || '').trim()
  if (!s) return ''
  return /^\d/.test(s) ? `Android ${s}` : s
}
