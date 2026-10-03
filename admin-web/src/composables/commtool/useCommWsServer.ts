// WS 服务端：客户端列表与消息下发
import { ref } from 'vue'
import { showToast } from '@/api/client'
import { commWsServerBroadcast, commWsServerSend, getCommWsServerClients } from '@/api/commtool'
import type { WsServerClient } from '@/api/commtool'

export function useCommWsServer(deps: { loadWsLogs: () => Promise<void> | void }) {
  const wsServerClients = ref<WsServerClient[]>([])
  const wsMessage = ref('')
  const wsBroadcastMessage = ref('')

  async function loadWsServerClients() {
    const res = await getCommWsServerClients()
    if (res.code === 200 && Array.isArray(res.data)) {
      wsServerClients.value = res.data
    }
  }

  async function sendWsServerMessage(clientId: string) {
    if (!wsMessage.value.trim()) {
      showToast('请输入消息内容')
      return
    }
    const res = await commWsServerSend({ id: clientId, message: wsMessage.value })
    if (res.code === 200) {
      showToast(res.msg || '已发送', 'success')
      wsMessage.value = ''
      deps.loadWsLogs()
    } else {
      showToast(res.msg || '发送失败')
    }
  }

  async function broadcastWsMessage() {
    if (!wsBroadcastMessage.value.trim()) {
      showToast('请输入广播内容')
      return
    }
    const res = await commWsServerBroadcast({ message: wsBroadcastMessage.value })
    if (res.code === 200) {
      showToast(res.msg || '已广播', 'success')
      wsBroadcastMessage.value = ''
      deps.loadWsLogs()
    } else {
      showToast(res.msg || '广播失败')
    }
  }

  return { wsServerClients, wsMessage, wsBroadcastMessage, loadWsServerClients, sendWsServerMessage, broadcastWsMessage }
}
