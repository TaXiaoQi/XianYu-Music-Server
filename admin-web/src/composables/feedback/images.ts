import { ref } from 'vue'
import type { Feedback } from '@/api/feedback'

// 外链图片改写为同源路径，避免混合内容/防盗链问题
function normalizeImgUrl(u: string): string {
  if (u.startsWith('http://') || u.startsWith('https://')) {
    try {
      const parsed = new URL(u, window.location.origin)
      if (parsed.origin !== window.location.origin) {
        return window.location.origin + parsed.pathname + parsed.search
      }
      return u
    } catch {
      return u
    }
  }
  return u
}

// 反馈图片缩略图 + 全屏图片查看器
export function useFeedbackImageViewer() {
  const imageViewerVisible = ref(false)
  const imageViewerList = ref<string[]>([])
  const imageViewerIndex = ref(0)
  const imageViewerReady = ref(false)

  function openImageViewer(imgs: string[], index: number) {
    imageViewerList.value = imgs
    imageViewerIndex.value = index
    imageViewerReady.value = false
    imageViewerVisible.value = true
  }
  function closeImageViewer() {
    imageViewerVisible.value = false
    imageViewerList.value = []
  }
  function nextImage() {
    if (imageViewerList.value.length === 0) return
    imageViewerIndex.value = (imageViewerIndex.value + 1) % imageViewerList.value.length
    imageViewerReady.value = false
  }
  function prevImage() {
    if (imageViewerList.value.length === 0) return
    imageViewerIndex.value = (imageViewerIndex.value - 1 + imageViewerList.value.length) % imageViewerList.value.length
    imageViewerReady.value = false
  }
  function onViewerKeydown(e: KeyboardEvent) {
    if (!imageViewerVisible.value) return
    if (e.key === 'Escape') closeImageViewer()
    else if (e.key === 'ArrowRight') nextImage()
    else if (e.key === 'ArrowLeft') prevImage()
  }

  function itemImages(item: Feedback): string[] {
    if (!item.images) return []
    try {
      const arr = JSON.parse(item.images)
      return Array.isArray(arr)
        ? arr.filter((u: string) => typeof u === 'string' && (u.startsWith('http') || u.startsWith('/'))).map(normalizeImgUrl)
        : []
    } catch {
      return []
    }
  }
  function resolveItemImages(item: Feedback): string[] {
    if (!item.resolve_images) return []
    try {
      const arr = JSON.parse(item.resolve_images)
      return Array.isArray(arr)
        ? arr.filter((u: string) => typeof u === 'string' && (u.startsWith('http') || u.startsWith('/'))).map(normalizeImgUrl)
        : []
    } catch {
      return []
    }
  }
  function stackThumbStyle(i: number, total: number): Record<string, string> {
    if (total <= 1) return {}
    const offset = Math.min(i, 3) * 5
    return {
      left: `${offset}px`,
      top: `${offset}px`,
      zIndex: String(total - i),
    }
  }

  return {
    imageViewerVisible,
    imageViewerList,
    imageViewerIndex,
    imageViewerReady,
    openImageViewer,
    closeImageViewer,
    nextImage,
    prevImage,
    onViewerKeydown,
    itemImages,
    resolveItemImages,
    stackThumbStyle,
  }
}
