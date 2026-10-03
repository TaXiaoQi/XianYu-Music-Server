import { ref } from 'vue'

// 保存请求进行中的共享锁：编辑弹窗保存期间置位，列表的平台/系统切换据此暂停
export function useVersionSaving() {
  const desktopSaving = ref(false)
  return { desktopSaving }
}
