// 连接日志：HTTP 与 WS 日志合并展示、一键清空
import { computed } from 'vue'
import type { Ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { clearCommHttpLogs, clearCommWsLogs } from '@/api/commtool'
import type { HttpLog, WsLog } from '@/api/commtool'

interface MergedLog {
  source: string
  time: string
  method?: string
  path?: string
  direction?: string
  data?: string
  body?: string
}

export function useCommLogs(deps: { httpLogs: Ref<HttpLog[]>; wsLogs: Ref<WsLog[]> }) {
  const mergedLogs = computed<MergedLog[]>(() => {
    const httpArr: MergedLog[] = deps.httpLogs.value.map(l => ({ ...l, source: 'http' }))
    const wsArr: MergedLog[] = deps.wsLogs.value.map(l => ({ ...l, source: 'ws' }))
    return [...httpArr, ...wsArr].sort((a, b) => (b.time || '').localeCompare(a.time || ''))
  })

  async function clearAllLogs() {
    const ok = await webConfirm('确认清空所有连接日志？', { title: '清空日志', confirmText: '确认清空' })
    if (!ok) return
    await Promise.all([clearCommHttpLogs(), clearCommWsLogs()])
    deps.httpLogs.value = []
    deps.wsLogs.value = []
    showToast('已清空', 'success')
  }

  return { mergedLogs, clearAllLogs }
}
