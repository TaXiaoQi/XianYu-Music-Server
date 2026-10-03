import { computed, ref } from 'vue'
import { showToast } from '@/api/client'
import { detectPackageMeta, packageMetaLabel, PACKAGE_ALLOWED_EXT } from '@/utils/packageDetect'
import type { VersionDraft } from './useVersionDraft'

// 下载渠道 / 安装包上传 / 商店分发：配置弹窗的状态与操作
export function useVersionChannel(draft: VersionDraft) {
  const { desktopDraft, desktopDraftEnabled, desktopDraftPlatform, desktopDraftSystem, desktopDraftArch, desktopEditingVersion } = draft

  const desktopChannelModalVisible = ref(false)
  const desktopChannelMode = ref<'link' | 'upload'>('link')
  const desktopChannelLinkDraft = ref('')
  const desktopFileInputRef = ref<HTMLInputElement | null>(null)
  const desktopPackageFile = ref<File | null>(null)
  const desktopPackageFileDraft = ref<File | null>(null)
  const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })
  const desktopPackageDragging = ref(false)

  const storeModalVisible = ref(false)
  const storeDraftEnabled = ref(false)
  const storeDraftUrl = ref('')

  const storeCardLabel = computed(() => {
    if (storeModalVisible.value) return storeDraftEnabled.value ? '微软商店' : '未设置'
    return desktopDraft.value.storeUrl ? '微软商店' : '未设置'
  })

  const storeCardDesc = computed(() => {
    if (!storeCardLabel.value.startsWith('微软商店')) {
      return desktopDraftPlatform.value === 'desktop' ? '点击设置商店分发渠道' : ''
    }
    return desktopDraft.value.storeUrl
  })

  const desktopChannelLabel = computed(() => {
    if (desktopPackageFile.value?.name) return '上传安装包'
    const url = desktopDraft.value.downloadUrl
    if (url) return url.startsWith('/uploads/packages/') ? '服务器安装包' : '下载链接'
    return '未选择下载渠道'
  })

  const desktopChannelDesc = computed(() => {
    if (desktopPackageFile.value?.name) return `已选择：${desktopPackageFile.value.name}（${formatFileSize(desktopPackageFile.value.size)}）`
    const url = desktopDraft.value.downloadUrl
    if (url) return url
    return desktopDraftEnabled.value ? '启用更新时，需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
  })

  function openStoreModal() {
    const current = desktopDraft.value.storeUrl
    storeDraftEnabled.value = !!current
    storeDraftUrl.value = current
    storeModalVisible.value = true
  }

  function confirmStore() {
    const url = storeDraftUrl.value.trim()
    if (storeDraftEnabled.value && !url) {
      showToast('请填写商店页链接')
      return
    }
    if (storeDraftEnabled.value && !url.startsWith('https://')) {
      showToast('商店页链接必须以 https:// 开头')
      return
    }
    desktopDraft.value.storeUrl = storeDraftEnabled.value ? url : ''
    storeModalVisible.value = false
  }

  function openDesktopChannelModal() {
    desktopChannelMode.value = desktopDraft.value.downloadUrl && !desktopPackageFile.value ? 'link' : 'upload'
    desktopChannelLinkDraft.value = desktopDraft.value.downloadUrl || ''
    desktopPackageFileDraft.value = desktopPackageFile.value
    desktopPackageDraft.value = desktopPackageFile.value
      ? { fileName: desktopPackageFile.value.name, fileSize: desktopPackageFile.value.size, fileBase64: '' }
      : { fileName: '', fileSize: 0, fileBase64: '' }
    desktopChannelModalVisible.value = true
  }

  function closeDesktopChannelModal() {
    desktopChannelModalVisible.value = false
  }

  function confirmDesktopChannel() {
    if (desktopChannelMode.value === 'link') {
      const url = desktopChannelLinkDraft.value.trim()
      if (!url) { showToast('请输入下载链接'); return }
      desktopDraft.value.downloadUrl = url
      desktopPackageFile.value = null
      desktopPackageFileDraft.value = null
      desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
      if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
    } else {
      if (!desktopPackageFileDraft.value) { showToast('请选择安装包'); return }
      desktopPackageFile.value = desktopPackageFileDraft.value
      desktopDraft.value.downloadUrl = ''
      desktopChannelLinkDraft.value = ''
      applyDetectedMeta(desktopPackageFile.value.name)
    }
    desktopChannelModalVisible.value = false
  }

  // 从安装包文件名识别平台/系统/架构，自动填入表单（识别结果仍可手动修正）
  function applyDetectedMeta(fileName: string) {
    const meta = detectPackageMeta(fileName)
    if (desktopEditingVersion.value) {
      // 编辑模式维度锁定：仅提示识别结果，便于核对是否传错包
      if (meta) showToast(`识别为 ${packageMetaLabel(meta)}，编辑模式不改变当前维度`)
      return
    }
    if (meta) {
      desktopDraftPlatform.value = meta.platform
      desktopDraftSystem.value = meta.system
      desktopDraftArch.value = meta.arch
      showToast(`已识别：${packageMetaLabel(meta)}，可手动修改`, 'success')
    } else {
      showToast('无法识别安装包平台，请手动选择平台 / 系统 / 架构')
    }
  }

  function onDesktopFileChange(e: Event) {
    const input = e.target as HTMLInputElement
    if (!input.files || input.files.length === 0) {
      desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
      desktopPackageFileDraft.value = null
      return
    }
    setDesktopPackageFile(input.files[0])
  }

  function triggerDesktopFileInput() {
    desktopFileInputRef.value?.click()
  }

  function onDesktopPackageDrop(e: DragEvent) {
    desktopPackageDragging.value = false
    const file = e.dataTransfer?.files?.[0]
    if (!file) return
    setDesktopPackageFile(file)
  }

  function setDesktopPackageFile(file: File) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    if (!PACKAGE_ALLOWED_EXT.includes(ext)) {
      showToast('不支持该安装包格式')
      if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
      return
    }
    desktopPackageFileDraft.value = file
    desktopPackageDraft.value = { fileName: file.name, fileSize: file.size, fileBase64: '' }
    desktopChannelLinkDraft.value = ''
  }

  function formatFileSize(bytes: number): string {
    if (!bytes) return '-'
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  function readFileAsBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
      reader.onerror = () => reject(new Error('文件读取失败'))
      reader.readAsDataURL(file)
    })
  }

  return {
    desktopChannelModalVisible,
    desktopChannelMode,
    desktopChannelLinkDraft,
    desktopFileInputRef,
    desktopPackageFile,
    desktopPackageFileDraft,
    desktopPackageDraft,
    desktopPackageDragging,
    storeModalVisible,
    storeDraftEnabled,
    storeDraftUrl,
    storeCardLabel,
    storeCardDesc,
    desktopChannelLabel,
    desktopChannelDesc,
    openStoreModal,
    confirmStore,
    openDesktopChannelModal,
    closeDesktopChannelModal,
    confirmDesktopChannel,
    applyDetectedMeta,
    onDesktopFileChange,
    triggerDesktopFileInput,
    onDesktopPackageDrop,
    setDesktopPackageFile,
    formatFileSize,
    readFileAsBase64,
  }
}

export type VersionChannel = ReturnType<typeof useVersionChannel>
