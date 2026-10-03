// 外部客户端管理：添加/删除/启停/连接 + 卡片展开
import { reactive, ref } from 'vue'
import type { Ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { addCommClient, commWsClientConnect, deleteCommClient, getCommClients, toggleCommClient } from '@/api/commtool'
import type { CommClient } from '@/api/commtool'

export interface UseCommClientsDeps {
  wsClientConnected: Ref<boolean>
  loadCommStatus: () => Promise<void> | void
}

export function useCommClients(deps: UseCommClientsDeps) {
  const { wsClientConnected, loadCommStatus } = deps

  const commClients = ref<CommClient[]>([])
  const addClientDialogVisible = ref(false)
  const addClientSaving = ref(false)
  const clientTypeOptions = [
    { value: 'ws', label: 'WebSocket' },
    { value: 'http', label: 'HTTP' },
    { value: 'sse', label: 'SSE' },
  ]
  const addClientForm = reactive({
    name: '',
    url: '',
    type: 'ws',
    events: '',
  })
  const expandedClient = ref<string | null>(null)

  function openAddClientModal() {
    addClientForm.name = ''
    addClientForm.url = ''
    addClientForm.type = 'ws'
    addClientForm.events = ''
    addClientDialogVisible.value = true
  }

  function closeAddClientDialog() {
    if (addClientSaving.value) return
    addClientDialogVisible.value = false
  }

  async function doAddClient() {
    const name = addClientForm.name.trim()
    const url = addClientForm.url.trim()
    if (!name) {
      showToast('请输入名称')
      return
    }
    if (!url) {
      showToast('请输入连接地址')
      return
    }
    addClientSaving.value = true
    const res = await addCommClient({
      name,
      url,
      type: addClientForm.type,
      events: addClientForm.events.trim(),
    })
    addClientSaving.value = false
    if (res.code === 200) {
      showToast('添加成功', 'success')
      addClientDialogVisible.value = false
      loadCommClients()
      loadCommStatus()
    } else {
      showToast(res.msg || '添加失败')
    }
  }

  async function loadCommClients() {
    const res = await getCommClients()
    if (res.code === 200 && Array.isArray(res.data)) {
      commClients.value = res.data
    }
  }

  async function deleteClient(item: CommClient) {
    const ok = await webConfirm(`确认删除客户端 "${item.name}"？`, { title: '删除客户端', confirmText: '确认删除' })
    if (!ok) return
    const res = await deleteCommClient({ id: item.id })
    if (res.code === 200) {
      showToast('已删除', 'success')
      loadCommClients()
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  async function toggleClientEnabled(item: CommClient) {
    const res = await toggleCommClient({ id: item.id, enabled: !item.enabled })
    if (res.code === 200) {
      item.enabled = !item.enabled
      showToast('已更新', 'success')
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  async function connectClient(item: CommClient) {
    if (item.type !== 'ws') return
    const res = await commWsClientConnect({ url: item.url })
    if (res.code === 200) {
      wsClientConnected.value = true
      showToast(res.msg || '连接成功', 'success')
      loadCommStatus()
    } else {
      showToast(res.msg || '连接失败')
    }
  }

  function toggleClient(key: string) {
    expandedClient.value = expandedClient.value === key ? null : key
  }

  return { commClients, addClientDialogVisible, addClientSaving, clientTypeOptions, addClientForm, expandedClient, openAddClientModal, closeAddClientDialog, doAddClient, loadCommClients, deleteClient, toggleClientEnabled, connectClient, toggleClient }
}
