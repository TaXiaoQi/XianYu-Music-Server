import { computed, ref } from 'vue'
import { getUserStats, getUsers } from '@/api/users'
import type { User } from '@/api/users'

// 听歌时长展示格式化（h:mm:ss）
export function formatDuration(seconds: number | undefined): string {
  const dur = Number(seconds) || 0
  const h = Math.floor(dur / 3600)
  const m = Math.floor((dur % 3600) / 60)
  const s = dur % 60
  return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

// 用户列表 / 统计 / 分页 / 搜索
export function useUserList() {
  const users = ref<User[]>([])
  const loading = ref(true)
  const loadError = ref('')
  const keyword = ref('')
  const page = ref(1)
  const pageSize = 20
  const total = ref(0)
  const totalPages = ref(0)
  const stats = ref({ total: 0, normal: 0, banned: 0 })

  const pageNumbers = computed(() => {
    const max = 7
    const pages: number[] = []
    if (totalPages.value <= max) {
      for (let i = 1; i <= totalPages.value; i++) pages.push(i)
    } else {
      let start = Math.max(1, page.value - 3)
      let end = Math.min(totalPages.value, start + max - 1)
      if (end - start < max - 1) start = Math.max(1, end - max + 1)
      for (let i = start; i <= end; i++) pages.push(i)
    }
    return pages
  })

  async function loadUsers() {
    loading.value = true
    loadError.value = ''
    const res = await getUsers({ page: page.value, page_size: pageSize, keyword: keyword.value })
    if (res.code === 200 && res.data) {
      users.value = res.data.list || []
      total.value = res.data.total
      totalPages.value = res.data.total_pages
    } else {
      loadError.value = res.msg || '加载失败'
      users.value = []
    }
    loading.value = false
    loadUserStats()
  }

  async function loadUserStats() {
    const res = await getUserStats()
    if (res.code === 200 && res.data) {
      stats.value = {
        total: Number(res.data.total) || 0,
        normal: Number(res.data.normal) || 0,
        banned: Number(res.data.banned) || 0,
      }
    }
  }

  function handleSearch() {
    page.value = 1
    loadUsers()
  }

  function clearSearch() {
    keyword.value = ''
    page.value = 1
    loadUsers()
  }

  function goPage(p: number) {
    if (p < 1 || p > totalPages.value || p === page.value) return
    page.value = p
    loadUsers()
  }

  return {
    users, loading, loadError, keyword, page, pageSize, total, totalPages, stats, pageNumbers,
    loadUsers, loadUserStats, handleSearch, clearSearch, goPage, formatDuration,
  }
}
