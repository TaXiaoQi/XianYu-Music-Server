import { ref } from 'vue'
import { showToast } from '@/api/client'
import { getFeedbackLimit, updateFeedbackLimit, getFeedbackAdminStats } from '@/api/feedback'
import type { FeedbackAdminStatsRow } from '@/api/feedback'

// 每日提交上限 + 管理员处理统计弹窗
export function useFeedbackLimitStats() {
  const limitLoading = ref(false)
  const limitSaving = ref(false)
  const limitModalVisible = ref(false)
  const feedbackDailyLimit = ref(20)
  const feedbackLimitInput = ref(20)

  function openLimitModal() {
    feedbackLimitInput.value = feedbackDailyLimit.value
    limitModalVisible.value = true
  }
  function closeLimitModal() {
    if (limitSaving.value) return
    limitModalVisible.value = false
  }

  async function loadFeedbackLimit() {
    limitLoading.value = true
    const res = await getFeedbackLimit()
    if (res.code === 200 && res.data) {
      const limit = Number(res.data.feedback_daily_limit ?? 20)
      feedbackDailyLimit.value = Number.isFinite(limit) ? limit : 20
      feedbackLimitInput.value = feedbackDailyLimit.value
    } else {
      showToast(res.msg || '反馈上限加载失败')
    }
    limitLoading.value = false
  }

  async function saveFeedbackLimit() {
    const limit = Number(feedbackLimitInput.value)
    if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
      showToast('每日上限需为 0 到 10000 的整数')
      return
    }
    limitSaving.value = true
    const res = await updateFeedbackLimit(limit)
    limitSaving.value = false
    if (res.code === 200) {
      feedbackDailyLimit.value = Number(res.data?.feedback_daily_limit ?? limit)
      feedbackLimitInput.value = feedbackDailyLimit.value
      limitModalVisible.value = false
      showToast('反馈提交上限已保存', 'success')
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  const statsModalVisible = ref(false)
  const statsLoading = ref(false)
  const statsList = ref<FeedbackAdminStatsRow[]>([])
  const statsGrandTotal = ref(0)

  async function openStats() {
    statsModalVisible.value = true
    await loadStats()
  }
  async function loadStats() {
    statsLoading.value = true
    const res = await getFeedbackAdminStats()
    statsLoading.value = false
    if (res.code === 200 && res.data) {
      statsList.value = res.data.list || []
      statsGrandTotal.value = Number(res.data.grand_total ?? 0)
    } else {
      statsList.value = []
      statsGrandTotal.value = 0
      showToast(res.msg || '统计加载失败')
    }
  }
  function closeStats() {
    if (statsLoading.value) return
    statsModalVisible.value = false
  }

  return {
    limitLoading,
    limitSaving,
    limitModalVisible,
    feedbackDailyLimit,
    feedbackLimitInput,
    openLimitModal,
    closeLimitModal,
    loadFeedbackLimit,
    saveFeedbackLimit,
    statsModalVisible,
    statsLoading,
    statsList,
    statsGrandTotal,
    openStats,
    loadStats,
    closeStats,
  }
}
