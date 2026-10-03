// HTTP 客户端：请求调试 + HTTP 请求日志
import { ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { clearCommHttpLogs, commHttpClient, getCommHttpLogs } from '@/api/commtool'
import type { HttpClientResult, HttpLog } from '@/api/commtool'

export function useCommHttpClient() {
  const httpLogs = ref<HttpLog[]>([])
  const httpClientUrl = ref('')
  const httpClientMethod = ref('GET')
  const httpClientHeaders = ref('')
  const httpClientBody = ref('')
  const httpClientSending = ref(false)
  const httpClientResult = ref<HttpClientResult | null>(null)

  async function loadHttpLogs() {
    const res = await getCommHttpLogs({ limit: 100 })
    if (res.code === 200 && Array.isArray(res.data)) {
      httpLogs.value = res.data
    }
  }

  async function clearHttpLogs() {
    const ok = await webConfirm('确认清空所有 HTTP 请求日志？', { title: '清空日志', confirmText: '确认清空' })
    if (!ok) return
    const res = await clearCommHttpLogs()
    if (res.code === 200) {
      httpLogs.value = []
      showToast('已清空', 'success')
    } else {
      showToast(res.msg || '清空失败')
    }
  }

  async function sendHttpClient() {
    if (!httpClientUrl.value.trim()) {
      showToast('请输入请求地址')
      return
    }
    httpClientSending.value = true
    const res = await commHttpClient({
      url: httpClientUrl.value.trim(),
      method: httpClientMethod.value,
      headers: httpClientHeaders.value,
      body: httpClientBody.value,
    })
    httpClientSending.value = false
    if (res.code === 200 && res.data) {
      httpClientResult.value = res.data
    } else {
      httpClientResult.value = null
      showToast(res.msg || '请求失败')
    }
  }

  function formatHeaders(headers: Record<string, string>): string {
    return Object.entries(headers || {}).map(([k, v]) => `${k}: ${v}`).join('\n')
  }

  return { httpLogs, httpClientUrl, httpClientMethod, httpClientHeaders, httpClientBody, httpClientSending, httpClientResult, loadHttpLogs, clearHttpLogs, sendHttpClient, formatHeaders }
}
