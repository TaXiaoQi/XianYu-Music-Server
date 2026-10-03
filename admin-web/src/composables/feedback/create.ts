import { ref, computed } from 'vue'
import { showToast } from '@/api/client'
import { createFeedback } from '@/api/feedback'

// 新建事项弹窗（拖拽/选择图片，最多 6 张，单张 ≤8MB）
export function useFeedbackCreateModal(options: { loadList: () => Promise<void> }) {
  const { loadList } = options

  const createModalVisible = ref(false)
  const createType = ref<'problem' | 'suggestion' | ''>('')
  const createPlatform = ref<'desktop' | 'mobile' | 'watch' | ''>('')
  const createTitle = computed(() => {
    if (createType.value === 'suggestion') return '功能建议'
    return '问题反馈'
  })
  const createContent = ref('')
  const createImages = ref<string[]>([])
  const createDragging = ref(false)
  const createSaving = ref(false)
  const createNotify = ref(false)
  const createFileInput = ref<HTMLInputElement | null>(null)

  function openCreateModal() {
    createType.value = ''
    createPlatform.value = ''
    createContent.value = ''
    createImages.value = []
    createSaving.value = false
    createDragging.value = false
    createNotify.value = false
    createModalVisible.value = true
  }
  function closeCreateModal() {
    if (createSaving.value) return
    createModalVisible.value = false
  }
  function onCreateDragOver(e: DragEvent) {
    e.preventDefault()
    createDragging.value = true
  }
  function onCreateDragLeave() {
    createDragging.value = false
  }
  function onCreateDrop(e: DragEvent) {
    e.preventDefault()
    createDragging.value = false
    const files = e.dataTransfer?.files
    if (files && files.length > 0) {
      handleCreateFiles(Array.from(files))
    }
  }
  function onCreateFileChange(e: Event) {
    const input = e.target as HTMLInputElement
    if (!input.files) return
    handleCreateFiles(Array.from(input.files))
    input.value = ''
  }
  function handleCreateFiles(files: File[]) {
    const remaining = 6 - createImages.value.length
    if (remaining <= 0) {
      showToast('最多上传 6 张图片')
      return
    }
    const accepted = files.slice(0, remaining)
    for (const file of accepted) {
      if (!file.type.startsWith('image/')) continue
      if (file.size > 8 * 1024 * 1024) {
        showToast(`图片 ${file.name} 超过 8MB，已跳过`)
        continue
      }
      const reader = new FileReader()
      reader.onload = () => {
        createImages.value.push(reader.result as string)
      }
      reader.onerror = () => showToast(`图片 ${file.name} 读取失败`)
      reader.readAsDataURL(file)
    }
  }
  function removeCreateImage(index: number) {
    createImages.value.splice(index, 1)
  }
  async function submitCreate() {
    if (createSaving.value) return
    if (!createType.value) { showToast('请选择事项类型'); return }
    if (!createPlatform.value) { showToast('请选择平台版本'); return }
    if (!createContent.value.trim()) {
      showToast('请填写内容')
      return
    }
    createSaving.value = true
    const res = await createFeedback({
      feedback_type: createType.value,
      platform: createPlatform.value,
      title: createTitle.value.trim(),
      content: createContent.value.trim(),
      images: createImages.value,
      notify_external: createNotify.value ? 1 : 0,
    })
    createSaving.value = false
    if (res.code === 200) {
      showToast('创建成功', 'success')
      closeCreateModal()
      await loadList()
    } else {
      showToast(res.msg || '创建失败')
    }
  }

  return {
    createModalVisible,
    createType,
    createPlatform,
    createTitle,
    createContent,
    createImages,
    createDragging,
    createSaving,
    createNotify,
    createFileInput,
    openCreateModal,
    closeCreateModal,
    onCreateDragOver,
    onCreateDragLeave,
    onCreateDrop,
    onCreateFileChange,
    handleCreateFiles,
    removeCreateImage,
    submitCreate,
  }
}
