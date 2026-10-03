// SSE 推送
import { ref } from 'vue'
import { showToast } from '@/api/client'
import { commSsePush } from '@/api/commtool'

export function useCommSse() {
  const sseMessage = ref('')

  async function sendSsePush() {
    if (!sseMessage.value.trim()) {
      showToast('请输入推送内容')
      return
    }
    const res = await commSsePush({ message: sseMessage.value })
    if (res.code === 200) {
      showToast(res.msg || '已推送', 'success')
      sseMessage.value = ''
    } else {
      showToast(res.msg || '推送失败')
    }
  }

  return { sseMessage, sendSsePush }
}
