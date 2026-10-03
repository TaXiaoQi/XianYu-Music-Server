import { ref, computed } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm, webActionMenu } from '@/utils/webDialog'
import { listFeedback, updateFeedbackStatus } from '@/api/feedback'
import type { Feedback, FbStats } from '@/api/feedback'

// 列表加载、状态/类型筛选、搜索、排序、状态变更
export function useFeedbackList() {
  const loading = ref(true)
  const feedbackList = ref<Feedback[]>([])
  const activeFilter = ref('all')
  const stats = ref<FbStats>({ total: 0, pending: 0, processing: 0, resolved: 0, rejected: 0 })
  const searchKeyword = ref('')
  const appliedKeyword = ref('')
  const typeFilter = ref('all')
  const sortMode = ref('post_time_desc')

  const sortOptions = [
    { key: 'post_time_desc', label: '最新提交' },
    { key: 'post_time_asc', label: '最早提交' },
    { key: 'update_desc', label: '最近更新' },
  ]
  const sortLabel = computed(() => sortOptions.find(o => o.key === sortMode.value)?.label || '排序')

  const filteredList = computed(() => {
    let arr = feedbackList.value
    if (activeFilter.value !== 'all') {
      arr = arr.filter(f => f.status === activeFilter.value)
    }
    if (typeFilter.value === 'appeal') {
      arr = arr.filter(f => f.category === 'appeal')
    } else if (typeFilter.value === 'beta') {
      arr = arr.filter(f => f.feedback_type === 'beta' && f.category !== 'appeal')
    } else if (typeFilter.value !== 'all') {
      arr = arr.filter(f => f.feedback_type === typeFilter.value && f.category !== 'appeal')
    }
    const kw = appliedKeyword.value
    if (kw) {
      arr = arr.filter(f =>
        (f.content || '').toLowerCase().includes(kw) ||
        (f.nickname || '').toLowerCase().includes(kw) ||
        (f.title || '').toLowerCase().includes(kw)
      )
    }
    return arr
  })

  async function loadList() {
    loading.value = true
    const res = await listFeedback(activeFilter.value === 'all' ? '' : activeFilter.value, sortMode.value)
    if (res.code === 200 && res.data) {
      feedbackList.value = res.data.list || []
      if (res.data.stats) {
        stats.value = res.data.stats
      }
    } else {
      feedbackList.value = []
    }
    loading.value = false
  }

  function handleSearch() {
    appliedKeyword.value = searchKeyword.value.trim().toLowerCase()
    loadList()
  }
  function clearSearch() {
    searchKeyword.value = ''
    appliedKeyword.value = ''
    loadList()
  }
  async function openSortMenu() {
    const key = await webActionMenu('排序方式', sortOptions.map(o => ({ key: o.key, label: o.label })))
    if (key && key !== sortMode.value) {
      sortMode.value = key
      loadList()
    }
  }
  function setFilter(s: string) {
    if (activeFilter.value === s) return
    activeFilter.value = s
    loadList()
  }
  function setTypeFilter(t: string) {
    if (typeFilter.value === t) return
    typeFilter.value = t
    loadList()
  }

  // resolved/rejected 需二次确认，成功后本地同步 stats 计数
  async function changeStatus(id: number, status: string) {
    const tips: Record<string, string> = {
      resolved: '确认将此反馈标记为已解决？',
      rejected: '确认拒绝此反馈？',
    }
    if (tips[status]) {
      const ok = await webConfirm(tips[status], { title: '更新反馈状态', confirmText: '确认' })
      if (!ok) return
    }
    const res = await updateFeedbackStatus(id, status)
    if (res.code === 200) {
      showToast('状态已更新', 'success')
      const item = feedbackList.value.find(f => f.id === id)
      if (item) {
        const oldStatus = item.status
        item.status = status
        if (stats.value[oldStatus as keyof FbStats] !== undefined) {
          stats.value[oldStatus as keyof FbStats]--
        }
        if (stats.value[status as keyof FbStats] !== undefined) {
          stats.value[status as keyof FbStats]++
        }
      }
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    loading,
    feedbackList,
    activeFilter,
    stats,
    searchKeyword,
    appliedKeyword,
    typeFilter,
    sortMode,
    sortLabel,
    filteredList,
    handleSearch,
    clearSearch,
    openSortMenu,
    setFilter,
    setTypeFilter,
    loadList,
    changeStatus,
  }
}
