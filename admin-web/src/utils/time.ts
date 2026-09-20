function parseUtcMs(v: any): number | null {
  if (!v) return null
  const s = String(v).trim()
  const m = s.match(/(\d{4})[-/](\d{1,2})[-/](\d{1,2})[T ]+(\d{1,2}):(\d{1,2})(?::(\d{1,2}))?/)
  if (!m) return null
  const [, y, mo, d, h, mi, sec] = m.map(Number)
  return Date.UTC(y, (mo || 1) - 1, d || 1, h || 0, mi || 0, sec || 0)
}

export function fmtTime(v: any): string {
  const ms = parseUtcMs(v)
  if (ms === null) return v ? String(v) : ''
  const dt = new Date(ms + 8 * 3600 * 1000)
  const y = dt.getUTCFullYear()
  const mo = dt.getUTCMonth() + 1
  const d = dt.getUTCDate()
  const h = dt.getUTCHours()
  const mi = dt.getUTCMinutes()
  return `${y}年${mo}月${d}日 ${h}时${String(mi).padStart(2, '0')}分`
}

export function fmtDateTime(v: any): string {
  const ms = parseUtcMs(v)
  if (ms === null) return v ? String(v) : ''
  const dt = new Date(ms + 8 * 3600 * 1000)
  const y = dt.getUTCFullYear()
  const mo = String(dt.getUTCMonth() + 1).padStart(2, '0')
  const d = String(dt.getUTCDate()).padStart(2, '0')
  const h = String(dt.getUTCHours()).padStart(2, '0')
  const mi = String(dt.getUTCMinutes()).padStart(2, '0')
  const sec = String(dt.getUTCSeconds()).padStart(2, '0')
  return `${y}-${mo}-${d} ${h}:${mi}:${sec}`
}

export function fmtDate(v: any): string {
  const ms = parseUtcMs(v)
  if (ms === null) return v ? String(v) : ''
  const dt = new Date(ms + 8 * 3600 * 1000)
  return `${dt.getUTCFullYear()}年${dt.getUTCMonth() + 1}月${dt.getUTCDate()}日`
}
