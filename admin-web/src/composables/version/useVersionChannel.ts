import { computed, ref } from 'vue'
import { showToast } from '@/api/client'
import type { ApiResponse } from '@/api/client'
import { detectPackageMeta, detectPackageVersion, packageMetaLabel, PACKAGE_ALLOWED_EXT } from '@/utils/packageDetect'
import { uploadPackage } from '@/api/version'
import type { VersionDraft } from './useVersionDraft'

// 下载渠道 / 安装包上传 / 商店分发：配置弹窗的状态与操作
export function useVersionChannel(draft: VersionDraft) {
  const { desktopDraft, desktopDraftEnabled, desktopDraftPlatform, desktopDraftSystem, desktopDraftArch, desktopDraftChannel, desktopDraftBetaNum, desktopEditingVersion } = draft

  const desktopChannelModalVisible = ref(false)
  const desktopChannelMode = ref<'link' | 'upload'>('link')
  const desktopChannelLinkDraft = ref('')
  const desktopFileInputRef = ref<HTMLInputElement | null>(null)
  const desktopPackageFile = ref<File | null>(null)
  const desktopPackageFileDraft = ref<File | null>(null)
  const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })
  const desktopPackageDragging = ref(false)

  // 后台上传状态：渠道弹窗确认后立即上传，配置弹窗内可继续编辑公告等字段
  const desktopUploading = ref(false)
  const desktopUploadProgress = ref(0)
  const desktopUploadError = ref('')
  let uploadSeq = 0
  let uploadHandle: { abort: () => void } | null = null
  let uploadTask: Promise<ApiResponse> | null = null

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
    if (desktopPackageFile.value?.name) {
      if (desktopUploading.value) return '正在上传安装包…'
      return `已选择：${desktopPackageFile.value.name}（${formatFileSize(desktopPackageFile.value.size)}）`
    }
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
      abortDesktopUpload()
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
      // 确认即后台上传：不阻塞弹窗，保存配置前会自动等待上传完成
      startDesktopUpload(desktopPackageFile.value)
    }
    desktopChannelModalVisible.value = false
  }

  // ==================== 安装包后台上传 ====================

  function startDesktopUpload(file: File) {
    const seq = ++uploadSeq
    uploadHandle?.abort()
    uploadHandle = null
    uploadTask = null
    desktopUploading.value = true
    desktopUploadProgress.value = 0
    desktopUploadError.value = ''
    const task = (async (): Promise<ApiResponse> => {
      let fileData = ''
      try {
        fileData = await readFileAsBase64(file)
      } catch {
        if (seq !== uploadSeq) return { code: 499, msg: '上传已取消', data: null }
        desktopUploading.value = false
        desktopUploadError.value = '安装包读取失败'
        return { code: 500, msg: '安装包读取失败', data: null }
      }
      if (seq !== uploadSeq) return { code: 499, msg: '上传已取消', data: null }
      const handle = uploadPackage({ file_name: file.name, file_data: fileData }, (p) => {
        if (seq === uploadSeq) desktopUploadProgress.value = p
      })
      uploadHandle = handle
      const res = await handle.promise
      if (seq !== uploadSeq) return res
      desktopUploading.value = false
      if (res.code === 200 && res.data?.download_url) {
        desktopUploadProgress.value = 100
        desktopDraft.value.downloadUrl = res.data.download_url
        showToast('安装包上传完成', 'success')
      } else if (res.code !== 499) {
        desktopUploadError.value = res.msg || '安装包上传失败'
      }
      return res
    })()
    uploadTask = task
  }

  function abortDesktopUpload() {
    uploadSeq++
    uploadHandle?.abort()
    uploadHandle = null
    uploadTask = null
    desktopUploading.value = false
    desktopUploadProgress.value = 0
    desktopUploadError.value = ''
  }

  // 保存配置前调用：上传中则等待完成；失败返回 false
  async function waitDesktopUpload(): Promise<boolean> {
    if (uploadTask) {
      const res = await uploadTask
      return res.code === 200
    }
    return !desktopUploadError.value
  }

  // 从安装包文件名识别平台/系统/架构与版本号/渠道，自动填入表单（识别结果仍可手动修正）
  function applyDetectedMeta(fileName: string) {
    const meta = detectPackageMeta(fileName)
    const ver = detectPackageVersion(fileName)
    if (desktopEditingVersion.value) {
      // 编辑模式维度锁定：仅提示识别结果，便于核对是否传错包
      if (meta) showToast(`识别为 ${packageMetaLabel(meta)}，编辑模式不改变当前维度`)
      return
    }
    if (meta) {
      desktopDraftPlatform.value = meta.platform
      desktopDraftSystem.value = meta.system
      desktopDraftArch.value = meta.arch
    }
    if (ver) {
      desktopDraft.value.version = ver.main
      if (ver.betaNum > 0) {
        desktopDraftChannel.value = 'beta'
        desktopDraftBetaNum.value = String(ver.betaNum)
      } else {
        desktopDraftChannel.value = 'stable'
        desktopDraftBetaNum.value = ''
      }
    }
    if (meta && ver) showToast(`已识别：${packageMetaLabel(meta)}，版本 ${ver.version}，可手动修改`, 'success')
    else if (meta) showToast(`已识别：${packageMetaLabel(meta)}，可手动修改`, 'success')
    else if (ver) showToast(`已识别版本 ${ver.version}，可手动修改`, 'success')
    else showToast('无法识别安装包平台，请手动选择平台 / 系统 / 架构')
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
    desktopUploading,
    desktopUploadProgress,
    desktopUploadError,
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
    abortDesktopUpload,
    waitDesktopUpload,
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
