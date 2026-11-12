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

  // 恒取全量列表，状态/类型/关键词筛选全部走本地 computed——切换筛选即时
  // 生效，不再整屏转圈。仅首次进入（或列表为空时）显示加载态。
  // hard=true 强制显示整屏加载（首次挂载）。
  async function loadList(opts?: { hard?: boolean }) {
    const hard = opts?.hard === true
    if (hard || feedbackList.value.length === 0) {
      loading.value = true
    }
    const res = await listFeedback('', sortMode.value)
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
    // 纯本地筛选，无需请求
    appliedKeyword.value = searchKeyword.value.trim().toLowerCase()
  }
  function clearSearch() {
    searchKeyword.value = ''
    appliedKeyword.value = ''
  }
  async function openSortMenu() {
    const key = await webActionMenu('排序方式', sortOptions.map(o => ({ key: o.key, label: o.label })))
    if (key && key !== sortMode.value) {
      // 排序在服务端执行，需重新拉取（软加载，不清空当前列表）
      sortMode.value = key
      loadList()
    }
  }
  // 状态/类型筛选均为本地过滤，即时切换
  function setFilter(s: string) {
    activeFilter.value = s
  }
  function setTypeFilter(t: string) {
    typeFilter.value = t
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
