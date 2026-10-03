// 连接鉴权：Token 配置与弹窗
import { reactive, ref } from 'vue'
import { showToast } from '@/api/client'
import { getCommAuthConfig, saveCommAuthConfig } from '@/api/commtool'

export function useCommAuth(deps: { loadCommStatus: () => Promise<void> | void }) {
  const { loadCommStatus } = deps

  const authForm = reactive({ token: '', token_enabled: false })
  const authShowToken = ref(false)
  const authSaving = ref(false)
  const authConfigLoaded = ref(false)
  const authDialogVisible = ref(false)

  async function loadAuthConfig() {
    const res = await getCommAuthConfig()
    if (res.code === 200 && res.data) {
      authForm.token = res.data.token || ''
      authForm.token_enabled = !!res.data.token_enabled
      authConfigLoaded.value = true
    }
  }

  async function saveAuthConfig() {
    authSaving.value = true
    const res = await saveCommAuthConfig({ token: authForm.token.trim() })
    authSaving.value = false
    if (res.code === 200) {
      authForm.token_enabled = !!authForm.token.trim()
      showToast(res.msg || '已保存', 'success')
      loadCommStatus()
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  function openAuthDialog() {
    loadAuthConfig()
    authDialogVisible.value = true
  }

  function closeAuthDialog() {
    if (authSaving.value) return
    authDialogVisible.value = false
  }

  return { authForm, authShowToken, authSaving, authConfigLoaded, authDialogVisible, loadAuthConfig, saveAuthConfig, openAuthDialog, closeAuthDialog }
}
