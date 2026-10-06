import { ref } from 'vue'
import { getUserFavorites } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'

// 收藏时间（epoch 毫秒）展示
export function formatFavTime(ms: number | undefined): string {
  const n = Number(ms) || 0
  if (n <= 0) return '-'
  const d = new Date(n)
  const p = (x: number) => String(x).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

// 查看用户收藏弹窗
export function useFavoritesModal() {
  const showFavoritesModal = ref(false)
  const favoritesLoading = ref(false)
  const favoritesData = ref<any>({})

  async function viewFavorites(u: User) {
    showFavoritesModal.value = true
    favoritesLoading.value = true
    favoritesData.value = {}
    const res = await getUserFavorites({ user_id: u.id })
    favoritesLoading.value = false
    if (res.code === 200 && res.data) {
      favoritesData.value = res.data
    } else {
      showToast(res.msg || '加载失败')
    }
  }

  return { showFavoritesModal, favoritesLoading, favoritesData, viewFavorites, formatFavTime }
}
