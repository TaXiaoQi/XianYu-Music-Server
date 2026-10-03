// 内置通信服务：状态加载 + 服务启停/端口配置
import { reactive, ref } from 'vue'
import type { Ref } from 'vue'
import { showToast } from '@/api/client'
import { getCommStatus, saveCommServiceConfig } from '@/api/commtool'
import type { CommStatus, WsClientConfigData } from '@/api/commtool'

export interface UseCommStatusDeps {
  wsClientConfig: WsClientConfigData
  wsClientConnected: Ref<boolean>
  authConfigLoaded: Ref<boolean>
}

export function useCommStatus(deps: UseCommStatusDeps) {
  const { wsClientConfig, wsClientConnected, authConfigLoaded } = deps

  const commStatus = ref<CommStatus>({
    server_running: false,
    server_port: 0,
    ws_server_count: 0,
    ws_client: null,
  })

  const commService = reactive({
    enabled: false,
    port: 8090,
  })
  const serviceSaving = ref(false)

  async function loadCommStatus() {
    const res = await getCommStatus()
    if (res.code === 200 && res.data) {
      commStatus.value = res.data
      wsClientConnected.value = !!res.data.ws_client
      if (res.data.server_enabled != null) {
        commService.enabled = !!res.data.server_enabled
      }
      if (res.data.server_port_config) {
        commService.port = res.data.server_port_config
      }
      if (res.data.ws_client_config) {
        const cfg = res.data.ws_client_config
        wsClientConfig.url = cfg.url || ''
        wsClientConfig.auto_reconnect = !!cfg.auto_reconnect
        wsClientConfig.reconnect_interval = cfg.reconnect_interval || '10'
        wsClientConfig.heartbeat_interval = cfg.heartbeat_interval || '30'
      }
      if (res.data.token_enabled != null) {
        authConfigLoaded.value = true
      }
    }
  }

  async function toggleService() {
    commService.enabled = !commService.enabled
    await saveServiceConfig()
  }

  async function saveServiceConfig() {
    if (commService.port < 1024 || commService.port > 65535) {
      showToast('端口必须在1024-65535之间')
      return
    }
    serviceSaving.value = true
    const res = await saveCommServiceConfig({
      enabled: commService.enabled,
      port: commService.port,
    })
    serviceSaving.value = false
    if (res.code === 200) {
      showToast('服务配置已保存', 'success')
      loadCommStatus()
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  return { commStatus, commService, serviceSaving, loadCommStatus, toggleService, saveServiceConfig }
}
