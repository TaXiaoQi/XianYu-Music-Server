import { ref } from 'vue'
import { banDevice, getUserDevices, unbanDevice } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'
import { fmtDateTime } from '@/utils/time'
import { formatOsVersion } from '@/utils/osVersion'
import { webConfirm, webPrompt } from '@/utils/webDialog'

// 用户设备信息弹窗
export function useUserDevices() {
  const showDeviceModal = ref(false)
  const deviceLoading = ref(false)
  const deviceData = ref<any>({})
  const userDevices = ref<any[]>([])

  function userDeviceIcon(dv: any): 'desktop' | 'mobile' | 'watch' {
    if (dv.platform === 'mobile' || dv.platform === 'watch' || dv.platform === 'desktop') return dv.platform
    if (/windows/i.test(dv.os_version || '')) return 'desktop'
    return 'mobile'
  }

  function userDevicePlatformLabel(dv: any): string {
    return userDeviceIcon(dv) === 'desktop' ? '桌面端' : userDeviceIcon(dv) === 'watch' ? '腕上端' : '移动端'
  }

  function userDeviceMeta(dv: any): string {
    const parts: string[] = []
    if (dv.os_version) parts.push(formatOsVersion(dv.os_version))
    if (dv.app_version) parts.push(`v${dv.app_version}`)
    if (dv.last_active) parts.push(`最后活跃 ${fmtDateTime(dv.last_active)}`)
    return parts.join(' · ') || '暂无活跃记录'
  }

  async function openDeviceModal(u: User) {
    showDeviceModal.value = true
    deviceLoading.value = true
    deviceData.value = { username: u.nickname || u.username, nickname: u.nickname || u.username }
    userDevices.value = []
    const res = await getUserDevices({ user_id: u.id })
    if (res.code === 200 && res.data) {
      deviceData.value = res.data
      userDevices.value = res.data.devices || []
    } else {
      showToast(res.msg || '加载设备信息失败')
    }
    deviceLoading.value = false
  }

  async function banUserDevice(deviceId: string, username: string) {
    const reason = await webPrompt(`请输入封禁用户 "${username}" 的设备 (${deviceId.substring(0, 16)}...) 的原因：`, '', { title: '封禁设备', placeholder: '封禁原因（必填）' })
    if (reason === null) return
    const reasonText = reason.trim()
    if (!reasonText) {
      showToast('封禁原因不能为空')
      return
    }
    const ok = await webConfirm(`确定封禁用户 "${username}" 的设备 (${deviceId.substring(0, 16)}...) 吗？封禁后该设备将无法登录。`, { title: '封禁设备', confirmText: '确认封禁' })
    if (!ok) return
    const res = await banDevice({ device_id: deviceId, reason: reasonText })
    if (res.code === 200) {
      showToast('设备已封禁', 'success')
      const it = userDevices.value.find(x => x.device_id === deviceId)
      if (it) it.is_banned = true
      if (deviceId === deviceData.value.last_device_id) deviceData.value.is_banned = true
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  async function unbanUserDevice(deviceId: string) {
    const res = await unbanDevice({ device_id: deviceId })
    if (res.code === 200) {
      showToast('设备已解封', 'success')
      const it = userDevices.value.find(x => x.device_id === deviceId)
      if (it) it.is_banned = false
      if (deviceId === deviceData.value.last_device_id) deviceData.value.is_banned = false
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    showDeviceModal, deviceLoading, deviceData, userDevices,
    userDeviceIcon, userDevicePlatformLabel, userDeviceMeta,
    openDeviceModal, banUserDevice, unbanUserDevice,
  }
}
