// WS 客户端：重连配置、连接/断开、消息发送与日志
import { reactive, ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import {
  clearCommWsLogs,
  commWsClientConnect,
  commWsClientDisconnect,
  commWsClientSend,
  getCommWsClientConfig,
  getCommWsLogs,
  saveCommWsClientConfig,
} from '@/api/commtool'
import type { WsLog } from '@/api/commtool'

export interface UseCommWsClientDeps {
  loadCommStatus: () => Promise<void> | void
}

export function useCommWsClient(deps: UseCommWsClientDeps) {
  const { loadCommStatus } = deps

  const wsClientConnected = ref(false)

  const wsClientConfig = reactive({
    url: '',
    auto_reconnect: false,
    reconnect_interval: '10',
    heartbeat_interval: '30',
  })
  const wsClientConfigSaving = ref(false)
  const wsClientUrl = ref('')
  const wsClientMessage = ref('')
  const wsLogs = ref<WsLog[]>([])

  async function loadWsClientConfig() {
    const res = await getCommWsClientConfig()
    if (res.code === 200 && res.data) {
      wsClientConfig.url = res.data.url || ''
      wsClientConfig.auto_reconnect = !!res.data.auto_reconnect
      wsClientConfig.reconnect_interval = res.data.reconnect_interval || '10'
      wsClientConfig.heartbeat_interval = res.data.heartbeat_interval || '30'
    }
  }

  async function saveWsClientConfig() {
    if (wsClientConfig.auto_reconnect && !wsClientConfig.url.trim()) {
      showToast('启用自动重连时请输入重连地址')
      return
    }
    wsClientConfigSaving.value = true
    const res = await saveCommWsClientConfig({
      url: wsClientConfig.url.trim(),
      auto_reconnect: wsClientConfig.auto_reconnect,
      reconnect_interval: wsClientConfig.reconnect_interval.trim() || '10',
      heartbeat_interval: wsClientConfig.heartbeat_interval.trim() || '30',
    })
    wsClientConfigSaving.value = false
    if (res.code === 200) {
      showToast(res.msg || '已保存', 'success')
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  async function connectWsClient() {
    if (!wsClientUrl.value.trim()) {
      showToast('请输入连接地址')
      return
    }
    const res = await commWsClientConnect({ url: wsClientUrl.value.trim() })
    if (res.code === 200) {
      wsClientConnected.value = true
      showToast(res.msg || '连接成功', 'success')
      loadCommStatus()
      loadWsLogs()
    } else {
      showToast(res.msg || '连接失败')
    }
  }

  async function disconnectWsClient() {
    const ok = await webConfirm('确认断开 WebSocket 客户端连接？', { title: '断开连接', confirmText: '确认断开' })
    if (!ok) return
    const res = await commWsClientDisconnect()
    if (res.code === 200) {
      wsClientConnected.value = false
      showToast(res.msg || '已断开', 'success')
      loadCommStatus()
    } else {
      showToast(res.msg || '断开失败')
    }
  }

  async function sendWsClientMessage() {
    if (!wsClientMessage.value.trim()) {
      showToast('请输入消息内容')
      return
    }
    const res = await commWsClientSend({ message: wsClientMessage.value })
    if (res.code === 200) {
      showToast(res.msg || '已发送', 'success')
      wsClientMessage.value = ''
      loadWsLogs()
    } else {
      showToast(res.msg || '发送失败')
    }
  }

  async function loadWsLogs() {
    const res = await getCommWsLogs({ limit: 100 })
    if (res.code === 200 && Array.isArray(res.data)) {
      wsLogs.value = res.data
    }
  }

  async function clearWsLogs() {
    const ok = await webConfirm('确认清空所有 WebSocket 消息日志？', { title: '清空日志', confirmText: '确认清空' })
    if (!ok) return
    const res = await clearCommWsLogs()
    if (res.code === 200) {
      wsLogs.value = []
      showToast('已清空', 'success')
    } else {
      showToast(res.msg || '清空失败')
    }
  }

  async function manualConnectWs() {
    if (!wsClientConfig.url.trim()) {
      showToast('请先填写重连地址')
      return
    }
    const res = await commWsClientConnect({ url: wsClientConfig.url.trim() })
    if (res.code === 200) {
      wsClientConnected.value = true
      showToast(res.msg || '连接成功', 'success')
      loadCommStatus()
    } else {
      showToast(res.msg || '连接失败')
    }
  }

  return { wsClientConfig, wsClientConfigSaving, wsClientConnected, wsClientUrl, wsClientMessage, wsLogs, loadWsClientConfig, saveWsClientConfig, connectWsClient, disconnectWsClient, sendWsClientMessage, loadWsLogs, clearWsLogs, manualConnectWs }
}
