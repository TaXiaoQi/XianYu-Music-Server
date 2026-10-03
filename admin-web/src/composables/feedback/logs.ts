import { ref, computed } from 'vue'
import { showToast } from '@/api/client'
import { getFeedbackDetail } from '@/api/feedback'
import type { Feedback } from '@/api/feedback'
import { hasErrorLogs } from './shared'

// 日志弹窗（错误日志 / 全量日志两个 tab）
export function useFeedbackLogs() {
  const logModalVisible = ref(false)
  const logTarget = ref<Feedback | null>(null)
  const logLoading = ref(false)
  const activeLogTab = ref<'error' | 'all'>('error')

  const currentLogText = computed(() => {
    if (!logTarget.value) return ''
    return activeLogTab.value === 'error'
      ? (logTarget.value.error_logs || '')
      : (logTarget.value.all_logs || '')
  })

  async function openLogModal(item: Feedback) {
    logModalVisible.value = true
    logTarget.value = item
    activeLogTab.value = hasErrorLogs(item) ? 'error' : 'all'
    logLoading.value = true
    const res = await getFeedbackDetail(item.id)
    logLoading.value = false
    if (res.code === 200 && res.data) {
      logTarget.value = res.data
      activeLogTab.value = res.data.error_logs ? 'error' : 'all'
    } else {
      showToast(res.msg || '日志加载失败')
    }
  }

  function closeLogModal() {
    if (logLoading.value) return
    logModalVisible.value = false
    logTarget.value = null
  }

  return {
    logModalVisible,
    logTarget,
    logLoading,
    activeLogTab,
    currentLogText,
    openLogModal,
    closeLogModal,
  }
}
