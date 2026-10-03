import { ref, computed } from 'vue'
import type { ComputedRef } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { batchDeleteFeedback, listRecycleBin, restoreFeedback } from '@/api/feedback'
import type { Feedback } from '@/api/feedback'

// 批量管理 + 回收站
export function useFeedbackBatchRecycle(options: { filteredList: ComputedRef<Feedback[]>; loadList: () => Promise<void> }) {
  const { filteredList, loadList } = options

  // ===== 批量管理 =====
  const batchMode = ref(false)
  const selectedIds = ref<Set<number>>(new Set())

  const allSelected = computed(() => {
    return filteredList.value.length > 0 && filteredList.value.every(f => selectedIds.value.has(f.id))
  })

  function enterBatchMode() {
    batchMode.value = true
    selectedIds.value.clear()
  }

  function exitBatchMode() {
    batchMode.value = false
    selectedIds.value.clear()
  }

  function toggleSelect(id: number) {
    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id)
    } else {
      selectedIds.value.add(id)
    }
    // 重新创建 Set 以触发响应式
    selectedIds.value = new Set(selectedIds.value)
  }

  function toggleSelectAll() {
    if (allSelected.value) {
      filteredList.value.forEach(f => selectedIds.value.delete(f.id))
    } else {
      filteredList.value.forEach(f => selectedIds.value.add(f.id))
    }
    selectedIds.value = new Set(selectedIds.value)
  }

  async function confirmBatchDelete() {
    if (selectedIds.value.size === 0) return
    const ok = await webConfirm(`确认将选中的 ${selectedIds.value.size} 条记录移入回收站？14 天内可恢复。`, {
      title: '批量删除',
      confirmText: '删除',
    })
    if (!ok) return
    const ids = Array.from(selectedIds.value)
    const res = await batchDeleteFeedback(ids)
    if (res.code === 200) {
      showToast(`已删除 ${res.data?.deleted ?? ids.length} 条记录`, 'success')
      exitBatchMode()
      await loadList()
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  // ===== 回收站 =====
  const recycleModalVisible = ref(false)
  const recycleLoading = ref(false)
  const recycleList = ref<any[]>([])

  async function openRecycleBin() {
    recycleModalVisible.value = true
    await loadRecycleBin()
  }

  function closeRecycleBin() {
    if (recycleLoading.value) return
    recycleModalVisible.value = false
  }

  async function loadRecycleBin() {
    recycleLoading.value = true
    const res = await listRecycleBin()
    recycleLoading.value = false
    if (res.code === 200 && res.data) {
      recycleList.value = res.data.list || []
    } else {
      recycleList.value = []
      showToast(res.msg || '回收站加载失败')
    }
  }

  async function restoreItem(id: number) {
    const res = await restoreFeedback(id)
    if (res.code === 200) {
      showToast('恢复成功', 'success')
      recycleList.value = recycleList.value.filter(r => r.id !== id)
      await loadList()
    } else {
      showToast(res.msg || '恢复失败')
    }
  }

  return {
    batchMode,
    selectedIds,
    allSelected,
    enterBatchMode,
    exitBatchMode,
    toggleSelect,
    toggleSelectAll,
    confirmBatchDelete,
    recycleModalVisible,
    recycleLoading,
    recycleList,
    openRecycleBin,
    closeRecycleBin,
    loadRecycleBin,
    restoreItem,
  }
}
