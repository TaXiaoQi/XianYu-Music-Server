import { computed, ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import {
  createBetaTester,
  deleteBetaTester,
  getBetaTesterDetail,
  listBetaTesters,
  setBetaLockEnabled,
  updateBetaTesterNote,
} from '@/api/version'

// 内测名单：列表加载、添加/移除、设备详情与备注编辑；含内测锁开关
export function useVersionBeta() {
  const betaModalVisible = ref(false)
  const betaList = ref<any[]>([])
  const betaLoading = ref(false)
  const betaSaving = ref(false)
  const betaDeviceIdDraft = ref('')
  const betaCiyuanxiDraft = ref('')
  const betaNoteDraft = ref('')
  const betaLockEnabled = ref(true)
  const betaLockSaving = ref(false)
  // 每账号可登录设备数上限（1-10，默认 1）
  const betaMaxDevices = ref(1)
  const betaMaxSaving = ref(false)

  async function loadBeta() {
    betaLoading.value = true
    const res = await listBetaTesters()
    if (res.code === 200 && res.data) {
      betaList.value = Array.isArray(res.data.list) ? res.data.list : []
      if (typeof res.data.enabled === 'boolean') betaLockEnabled.value = res.data.enabled
      if (typeof (res.data as any).max_devices === 'number') betaMaxDevices.value = (res.data as any).max_devices
    }
    betaLoading.value = false
  }

  async function toggleBetaLock(event: Event) {
    const enabled = (event.target as HTMLInputElement).checked
    betaLockSaving.value = true
    const res = await setBetaLockEnabled({ enabled, max_devices: betaMaxDevices.value })
    betaLockSaving.value = false
    if (res.code === 200) {
      betaLockEnabled.value = enabled
      showToast(enabled ? '内测锁已开启' : '内测锁已关闭', 'success')
    } else {
      // 失败时回滚开关状态
      betaLockEnabled.value = !enabled
      showToast(res.msg || '设置失败')
    }
  }

  async function saveBetaMaxDevices() {
    const n = Math.floor(Number(betaMaxDevices.value))
    if (!Number.isFinite(n) || n < 1 || n > 10) {
      showToast('设备数需为 1-10 的整数')
      return
    }
    betaMaxSaving.value = true
    const res = await setBetaLockEnabled({ enabled: betaLockEnabled.value, max_devices: n })
    betaMaxSaving.value = false
    if (res.code === 200) {
      betaMaxDevices.value = n
      showToast(`每账号设备上限已设为 ${n} 台`, 'success')
    } else {
      showToast(res.msg || '设置失败')
    }
  }

  function openBetaModal() {
    betaModalVisible.value = true
    if (!betaLoading.value && betaList.value.length === 0) loadBeta()
  }

  function closeBetaModal() {
    betaModalVisible.value = false
  }

  async function addBetaTester() {
    const deviceId = betaDeviceIdDraft.value.trim()
    const ciyuanxiId = betaCiyuanxiDraft.value.trim()
    if (!deviceId && !ciyuanxiId) {
      showToast('请输入设备ID或账号ID')
      return
    }
    betaSaving.value = true
    const res = await createBetaTester({ device_id: deviceId, ciyuanxi_id: ciyuanxiId, note: betaNoteDraft.value.trim() })
    betaSaving.value = false
    if (res.code === 200) {
      showToast('已添加到内测名单', 'success')
      betaDeviceIdDraft.value = ''
      betaCiyuanxiDraft.value = ''
      betaNoteDraft.value = ''
      loadBeta()
    } else {
      showToast(res.msg || '添加失败')
    }
  }

  async function removeBetaTester(t: any) {
    const isAccount = !t.device_id && t.ciyuanxi_id
    const ok = await webConfirm(
      isAccount ? `确认将账号 ${t.ciyuanxi_id} 移出内测名单？` : `确认将设备 ${t.device_id} 移出内测名单？`,
      { title: '移除内测资格', confirmText: '确认移除' },
    )
    if (!ok) return
    const res = await deleteBetaTester({ device_id: t.device_id || '', ciyuanxi_id: t.ciyuanxi_id || '' })
    if (res.code === 200) {
      showToast('已移除', 'success')
      loadBeta()
    } else {
      showToast(res.msg || '移除失败')
    }
  }

  const betaDetailVisible = ref(false)
  const betaDetailLoading = ref(false)
  const betaDetail = ref<any>(null)
  const betaNoteEditDraft = ref('')
  const betaNoteSaving = ref(false)
  const hasBetaDetailDevice = computed(() => {
    const d = betaDetail.value?.device || {}
    return !!(d.brand || d.model || d.os_version || d.architecture || d.machine_name)
  })

  async function openBetaDetail(t: any) {
    betaDetailVisible.value = true
    betaDetail.value = null
    betaDetailLoading.value = true
    const res = await getBetaTesterDetail({ device_id: t.device_id })
    betaDetailLoading.value = false
    if (res.code === 200 && res.data) {
      betaDetail.value = res.data
      betaNoteEditDraft.value = res.data.tester?.note || ''
    } else {
      showToast(res.msg || '加载详情失败')
      betaDetailVisible.value = false
    }
  }

  async function saveBetaNote() {
    const tester = betaDetail.value?.tester
    if (!tester) return
    const note = betaNoteEditDraft.value.trim()
    betaNoteSaving.value = true
    const res = await updateBetaTesterNote({ device_id: tester.device_id, note })
    betaNoteSaving.value = false
    if (res.code === 200) {
      tester.note = note
      showToast('已更新设备备注', 'success')
      loadBeta()
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  function closeBetaDetail() {
    betaDetailVisible.value = false
    betaDetail.value = null
    betaNoteEditDraft.value = ''
  }

  return {
    betaModalVisible,
    betaList,
    betaLoading,
    betaSaving,
    betaDeviceIdDraft,
    betaCiyuanxiDraft,
    betaNoteDraft,
    betaLockEnabled,
    betaLockSaving,
    betaMaxDevices,
    betaMaxSaving,
    toggleBetaLock,
    saveBetaMaxDevices,
    loadBeta,
    openBetaModal,
    closeBetaModal,
    addBetaTester,
    removeBetaTester,
    betaDetailVisible,
    betaDetailLoading,
    betaDetail,
    betaNoteEditDraft,
    betaNoteSaving,
    hasBetaDetailDevice,
    openBetaDetail,
    saveBetaNote,
    closeBetaDetail,
  }
}
