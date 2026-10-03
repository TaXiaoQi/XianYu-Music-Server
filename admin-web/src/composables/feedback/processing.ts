import { ref } from 'vue'
import { showToast } from '@/api/client'
import { collaboratorComplete, resolveFeedback, updateFeedbackStatus, resolveBetaApplication } from '@/api/feedback'
import type { Feedback } from '@/api/feedback'
import { collaboratorsOf } from './shared'

// 处理类弹窗：完成 / 留言详情 / 拒绝 / 同意内测申请
export function useFeedbackProcessingModals(options: { loadList: () => Promise<void> }) {
  const { loadList } = options

  // ===== 完成弹窗 =====
  const resolveModalVisible = ref(false)
  const resolveTarget = ref<Feedback | null>(null)
  const resolveNote = ref('')
  const resolveImages = ref<string[]>([])
  const resolveDragging = ref(false)
  const resolveFileInput = ref<HTMLInputElement | null>(null)
  const resolveSaving = ref(false)

  function openResolveModal(item: Feedback) {
    resolveTarget.value = item
    resolveNote.value = ''
    resolveImages.value = []
    resolveDragging.value = false
    resolveSaving.value = false
    resolveModalVisible.value = true
  }

  function closeResolveModal() {
    if (resolveSaving.value) return
    resolveModalVisible.value = false
    resolveTarget.value = null
    resolveNote.value = ''
    resolveImages.value = []
  }

  function onResolveDragOver(e: DragEvent) {
    e.preventDefault()
    resolveDragging.value = true
  }
  function onResolveDragLeave() {
    resolveDragging.value = false
  }
  function onResolveDrop(e: DragEvent) {
    e.preventDefault()
    resolveDragging.value = false
    const files = e.dataTransfer?.files
    if (files && files.length > 0) {
      handleResolveFiles(Array.from(files))
    }
  }
  function onResolveFileChange(e: Event) {
    const input = e.target as HTMLInputElement
    if (!input.files) return
    handleResolveFiles(Array.from(input.files))
    input.value = ''
  }
  function handleResolveFiles(files: File[]) {
    const remaining = 6 - resolveImages.value.length
    if (remaining <= 0) {
      showToast('最多上传 6 张图片')
      return
    }
    const accepted = files.slice(0, remaining)
    for (const file of accepted) {
      if (!file.type.startsWith('image/')) continue
      if (file.size > 8 * 1024 * 1024) {
        showToast(`图片 ${file.name} 超过 8MB，已跳过`)
        continue
      }
      const reader = new FileReader()
      reader.onload = () => {
        resolveImages.value.push(reader.result as string)
      }
      reader.onerror = () => showToast(`图片 ${file.name} 读取失败`)
      reader.readAsDataURL(file)
    }
  }
  function removeResolveImage(index: number) {
    resolveImages.value.splice(index, 1)
  }

  // 有协同人时走 collaborator_complete，否则 resolve_feedback
  async function confirmResolve() {
    if (!resolveTarget.value || !resolveNote.value.trim()) return
    resolveSaving.value = true
    const item = resolveTarget.value
    const hasCollab = collaboratorsOf(item).length > 0
    const res = hasCollab
      ? await collaboratorComplete(item.id, resolveNote.value.trim(), resolveImages.value)
      : await resolveFeedback(item.id, resolveNote.value.trim(), resolveImages.value)
    resolveSaving.value = false
    if (res.code === 200) {
      if (res.data?.resolved) {
        showToast('协同反馈已全部完成', 'success')
      } else {
        showToast(`已确认完成（${res.data?.completed}/${res.data?.total}），等待其他参与人`, 'success')
      }
      closeResolveModal()
      await loadList()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  // ===== 留言详情弹窗 =====
  const contentModalVisible = ref(false)
  const contentModalItem = ref<Feedback | null>(null)

  function openContentModal(item: Feedback) {
    contentModalItem.value = item
    contentModalVisible.value = true
  }

  function closeContentModal() {
    contentModalVisible.value = false
    contentModalItem.value = null
  }

  // ===== 拒绝弹窗 =====
  const rejectModalVisible = ref(false)
  const rejectTarget = ref<Feedback | null>(null)
  const rejectNote = ref('')
  const rejectSaving = ref(false)

  function openRejectModal(item: Feedback) {
    rejectTarget.value = item
    rejectNote.value = ''
    rejectSaving.value = false
    rejectModalVisible.value = true
  }

  function closeRejectModal() {
    if (rejectSaving.value) return
    rejectModalVisible.value = false
    rejectTarget.value = null
    rejectNote.value = ''
  }

  async function confirmReject() {
    if (!rejectTarget.value || !rejectNote.value.trim()) return
    rejectSaving.value = true
    const res = await updateFeedbackStatus(rejectTarget.value.id, 'rejected', rejectNote.value.trim())
    rejectSaving.value = false
    if (res.code === 200) {
      showToast('已拒绝该反馈', 'success')
      closeRejectModal()
      await loadList()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  // ===== 内测申请：回执必填，同意后设备自动加入内测名单 =====
  const betaApproveModalVisible = ref(false)
  const betaApproveTarget = ref<Feedback | null>(null)
  const betaApproveNote = ref('')
  const betaApproveDeviceNote = ref('')
  const betaApproveSaving = ref(false)

  function openBetaApproveModal(item: Feedback) {
    betaApproveTarget.value = item
    betaApproveNote.value = ''
    betaApproveDeviceNote.value = ''
    betaApproveSaving.value = false
    betaApproveModalVisible.value = true
  }

  function closeBetaApproveModal() {
    if (betaApproveSaving.value) return
    betaApproveModalVisible.value = false
    betaApproveTarget.value = null
    betaApproveNote.value = ''
    betaApproveDeviceNote.value = ''
  }

  async function confirmBetaApprove() {
    if (!betaApproveTarget.value || !betaApproveNote.value.trim()) return
    betaApproveSaving.value = true
    const res = await resolveBetaApplication(
      betaApproveTarget.value.id,
      betaApproveNote.value.trim(),
      betaApproveDeviceNote.value.trim(),
    )
    betaApproveSaving.value = false
    if (res.code === 200) {
      showToast(res.data?.device_added ? '已同意，设备已自动加入内测名单' : '已同意该内测申请', 'success')
      closeBetaApproveModal()
      await loadList()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    resolveModalVisible,
    resolveTarget,
    resolveNote,
    resolveImages,
    resolveDragging,
    resolveFileInput,
    resolveSaving,
    openResolveModal,
    closeResolveModal,
    onResolveDragOver,
    onResolveDragLeave,
    onResolveDrop,
    onResolveFileChange,
    handleResolveFiles,
    removeResolveImage,
    confirmResolve,
    contentModalVisible,
    contentModalItem,
    openContentModal,
    closeContentModal,
    rejectModalVisible,
    rejectTarget,
    rejectNote,
    rejectSaving,
    openRejectModal,
    closeRejectModal,
    confirmReject,
    betaApproveModalVisible,
    betaApproveTarget,
    betaApproveNote,
    betaApproveDeviceNote,
    betaApproveSaving,
    openBetaApproveModal,
    closeBetaApproveModal,
    confirmBetaApprove,
  }
}
