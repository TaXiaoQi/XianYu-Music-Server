import { computed, ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import {
  createBetaTester,
  deleteBetaTester,
  getBetaTesterDetail,
  listBetaTesters,
  updateBetaTesterNote,
} from '@/api/version'

// 内测名单：列表加载、添加/移除、设备详情与备注编辑
export function useVersionBeta() {
  const betaModalVisible = ref(false)
  const betaList = ref<any[]>([])
  const betaLoading = ref(false)
  const betaSaving = ref(false)
  const betaDeviceIdDraft = ref('')
  const betaNoteDraft = ref('')

  async function loadBeta() {
    betaLoading.value = true
    const res = await listBetaTesters()
    if (res.code === 200 && res.data) {
      betaList.value = Array.isArray(res.data.list) ? res.data.list : []
    }
    betaLoading.value = false
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
    if (!deviceId) {
      showToast('请输入设备ID')
      return
    }
    betaSaving.value = true
    const res = await createBetaTester({ device_id: deviceId, note: betaNoteDraft.value.trim() })
    betaSaving.value = false
    if (res.code === 200) {
      showToast('已添加到内测名单', 'success')
      betaDeviceIdDraft.value = ''
      betaNoteDraft.value = ''
      loadBeta()
    } else {
      showToast(res.msg || '添加失败')
    }
  }

  async function removeBetaTester(t: any) {
    const ok = await webConfirm(`确认将设备 ${t.device_id} 移出内测名单？`, { title: '移除内测设备', confirmText: '确认移除' })
    if (!ok) return
    const res = await deleteBetaTester({ device_id: t.device_id })
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
    betaNoteDraft,
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
