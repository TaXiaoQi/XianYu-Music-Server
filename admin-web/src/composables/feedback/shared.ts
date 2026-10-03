import { formatOsVersion } from '@/utils/osVersion'
import type { Feedback } from '@/api/feedback'

// collaborators / completed_by 等 JSON 数组字段的安全解析
export function parseJsonArray(v: string | null | undefined): any[] {
  if (!v) return []
  try {
    const arr = JSON.parse(v)
    return Array.isArray(arr) ? arr : []
  } catch {
    return []
  }
}

export function collaboratorsOf(item: Feedback): string[] {
  return parseJsonArray(item.collaborators).filter((c): c is string => typeof c === 'string')
}

export function completedOf(item: Feedback): Array<{ admin: string; note?: string }> {
  return parseJsonArray(item.completed_by).filter((c): c is { admin: string; note?: string } => !!c && typeof c.admin === 'string')
}

const statusMap: Record<string, string> = {
  pending: '待处理',
  processing: '处理中',
  resolved: '已解决',
  rejected: '已拒绝',
}

export function statusLabel(s: string): string {
  return statusMap[s] || s
}

const platformMap: Record<string, string> = {
  desktop: '桌面版',
  mobile: '移动版',
  watch: '腕上版',
}

export function platformLabel(p: string): string {
  return platformMap[p] || ''
}

export function deviceIcon(item: Feedback): 'desktop' | 'mobile' | 'watch' {
  if (item.platform === 'mobile') return 'mobile'
  if (item.platform === 'watch') return 'watch'
  return 'desktop'
}

export function formatLogSize(chars?: number | string): string {
  const n = Number(chars || 0)
  if (n <= 0) return ''
  if (n < 1024) return `${n} 字`
  return `${(n / 1024).toFixed(1)}K 字`
}

export function truthyFlag(value: unknown): boolean {
  return value === true || value === 1 || value === '1'
}

export function hasErrorLogs(item: Feedback): boolean {
  return truthyFlag(item.has_error_logs) || !!item.error_logs
}

export function hasAllLogs(item: Feedback): boolean {
  return truthyFlag(item.has_all_logs) || !!item.all_logs
}

export function deviceInfoText(item: Feedback): string {
  const brand = item.device_brand || ''
  const model = item.device_model || ''
  const os = item.os_version || ''
  const arch = item.architecture || ''
  const machine = item.machine_name || ''
  if (!brand && !model && !os && !arch && !machine) return ''
  const parts: string[] = []
  const dev = `${brand && model && brand !== model ? brand + ' · ' : ''}${model}`
  if (dev.trim()) parts.push(dev.trim())
  if (os) parts.push(formatOsVersion(os))
  if (arch) parts.push(arch)
  if (machine) parts.push('主机 ' + machine)
  return parts.join(' ｜ ')
}

// 内测申请（feedback_type='beta'）：仅有同意/拒绝两种处理
export function isBeta(item: Feedback): boolean {
  return item.category !== 'appeal' && item.feedback_type === 'beta'
}
