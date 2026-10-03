import { ref } from 'vue'
import { getUserPlugins } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'

// 插件体积展示
export function formatScriptSize(bytes: number | undefined): string {
  const b = Number(bytes) || 0
  if (b < 1024) return `${b} B`
  return `${(b / 1024).toFixed(1)} KB`
}

// 查看用户插件弹窗
export function usePluginsModal() {
  const showPluginsModal = ref(false)
  const pluginsLoading = ref(false)
  const pluginsData = ref<any>({})

  async function viewPlugins(u: User) {
    showPluginsModal.value = true
    pluginsLoading.value = true
    pluginsData.value = {}
    const res = await getUserPlugins({ user_id: u.id })
    pluginsLoading.value = false
    if (res.code === 200 && res.data) {
      pluginsData.value = res.data
    } else {
      showToast(res.msg || '加载失败')
    }
  }

  return { showPluginsModal, pluginsLoading, pluginsData, viewPlugins, formatScriptSize }
}
