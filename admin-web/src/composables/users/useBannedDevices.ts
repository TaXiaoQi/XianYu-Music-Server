import { ref } from 'vue'
import { banDevice, listBannedDevices, unbanDevice } from '@/api/users'
import { showToast } from '@/api/client'

// 封禁设备管理弹窗
export function useBannedDevices() {
  const showBannedModal = ref(false)
  const bannedLoading = ref(false)
  const bannedDevices = ref<any[]>([])
  const banDeviceInput = ref('')
  const banReasonInput = ref('')

  async function openBannedDevicesModal() {
    showBannedModal.value = true
    await loadBannedDevices()
  }

  async function loadBannedDevices() {
    bannedLoading.value = true
    const res = await listBannedDevices({ page: 1, page_size: 100 })
    if (res.code === 200 && res.data) {
      bannedDevices.value = res.data.list || []
    } else {
      bannedDevices.value = []
    }
    bannedLoading.value = false
  }

  async function manualBanDevice() {
    const deviceId = banDeviceInput.value.trim()
    if (!deviceId) {
      showToast('请输入设备ID')
      return
    }
    const reason = banReasonInput.value.trim()
    if (!reason) {
      showToast('封禁原因不能为空')
      return
    }
    const res = await banDevice({ device_id: deviceId, reason })
    if (res.code === 200) {
      showToast('设备已封禁', 'success')
      banDeviceInput.value = ''
      banReasonInput.value = ''
      await loadBannedDevices()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  async function unbanDeviceById(id: number, deviceId: string) {
    const res = await unbanDevice({ id, device_id: deviceId })
    if (res.code === 200) {
      showToast('设备已解封', 'success')
      await loadBannedDevices()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    showBannedModal, bannedLoading, bannedDevices, banDeviceInput, banReasonInput,
    openBannedDevicesModal, loadBannedDevices, manualBanDevice, unbanDeviceById,
  }
}
