// 数据库统一存 UTC，后台展示统一转北京时间（UTC+8，无夏令时）

/** 解析数据库时间字符串（如 2026-08-18 07:30:00 / 2026-08-18T07:30:00）为 UTC 毫秒时间戳 */
function parseUtcMs(v: any): number | null {
  if (!v) return null
  const s = String(v).trim()
  const m = s.match(/(\d{4})[-/](\d{1,2})[-/](\d{1,2})[T ]+(\d{1,2}):(\d{1,2})(?::(\d{1,2}))?/)
  if (!m) return null
  const [, y, mo, d, h, mi, sec] = m.map(Number)
  return Date.UTC(y, (mo || 1) - 1, d || 1, h || 0, mi || 0, sec || 0)
}

/** 北京时间格式化：2026年8月18日 15时30分 */
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

/** 北京时间完整格式化：2026-08-18 15:30:00 */
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

/** 北京时间日期：2026年8月18日 */
export function fmtDate(v: any): string {
  const ms = parseUtcMs(v)
  if (ms === null) return v ? String(v) : ''
  const dt = new Date(ms + 8 * 3600 * 1000)
  return `${dt.getUTCFullYear()}年${dt.getUTCMonth() + 1}月${dt.getUTCDate()}日`
}
