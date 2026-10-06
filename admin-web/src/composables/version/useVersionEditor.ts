import { ref } from 'vue'
import type { Ref } from 'vue'
import { showToast } from '@/api/client'
import { saveDesktopVersion } from '@/api/version'
import { archOf, defaultArch, defaultSystem, platformOf, systemOf } from './versionMeta'
import type { PlatformKey } from './versionMeta'
import type { VersionDraft } from './useVersionDraft'
import type { VersionChannel } from './useVersionChannel'

interface VersionEditorDeps {
  desktopSaving: Ref<boolean>
  loadDesktop: () => Promise<void>
  platformFilter: Ref<PlatformKey>
  systemFilter: Ref<string>
}

// 新增/编辑版本配置弹窗：打开 / 关闭 / 保存
export function useVersionEditor(draft: VersionDraft, channel: VersionChannel, deps: VersionEditorDeps) {
  const {
    desktopDraft,
    desktopDraftEnabled,
    desktopDraftPlatform,
    desktopDraftSystem,
    desktopDraftArch,
    desktopDraftChannel,
    desktopDraftBetaNum,
    desktopEditingVersion,
    desktopEditingChannel,
    composedVersion,
  } = draft
  const { desktopPackageFile, desktopPackageFileDraft, desktopPackageDraft, desktopFileInputRef, abortDesktopUpload, waitDesktopUpload, desktopUploadError } = channel
  const { desktopSaving, loadDesktop, platformFilter, systemFilter } = deps

  const desktopModalVisible = ref(false)

  function openDesktopModal(item?: any) {
    if (item) {
      const version = item.version || ''
      const betaIdx = version.indexOf('-beta-')
      const isBeta = betaIdx >= 0
      desktopEditingVersion.value = version
      desktopEditingChannel.value = isBeta ? 'beta' : 'stable'
      desktopDraftPlatform.value = platformOf(item)
      desktopDraftSystem.value = systemOf(item)
      desktopDraftArch.value = archOf(item)
      desktopDraftChannel.value = isBeta ? 'beta' : 'stable'
      desktopDraft.value = {
        version: isBeta ? version.slice(0, betaIdx) : version,
        updateContent: item.updateContent || '',
        downloadUrl: item.downloadUrl || '',
        storeUrl: item.storeUrl || '',
      }
      desktopDraftBetaNum.value = isBeta ? version.slice(betaIdx + '-beta-'.length) : ''
      desktopDraftEnabled.value = !!item.enabled
    } else {
      desktopEditingVersion.value = ''
      desktopEditingChannel.value = 'stable'
      desktopDraftPlatform.value = platformFilter.value
      desktopDraftSystem.value = systemFilter.value || defaultSystem(platformFilter.value)
      desktopDraftArch.value = defaultArch(desktopDraftPlatform.value)
      desktopDraftChannel.value = 'stable'
      desktopDraft.value = { version: '', updateContent: '', downloadUrl: '', storeUrl: '' }
      desktopDraftBetaNum.value = ''
      desktopDraftEnabled.value = false
    }
    desktopPackageFile.value = null
    desktopPackageFileDraft.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    abortDesktopUpload()
    if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
    desktopModalVisible.value = true
  }

  function closeDesktopModal() {
    if (desktopSaving.value) return
    desktopModalVisible.value = false
  }

  async function saveDesktop() {
    const version = composedVersion.value
    if (!version) {
      showToast(desktopDraftChannel.value === 'beta' ? '请填写版本号和 beta 号（正整数）' : '请填写版本号')
      return
    }
    const hasPackage = !!desktopPackageFile.value
    const hasUrl = !!desktopDraft.value.downloadUrl?.trim()
    if (desktopDraftEnabled.value && !hasPackage && !hasUrl) {
      showToast('启用更新时，请填写下载链接或选择安装包')
      return
    }
    desktopSaving.value = true
    // 安装包在渠道弹窗确认时已后台上传；若仍在传输则等它完成，失败则中止保存
    if (hasPackage) {
      const uploadOk = await waitDesktopUpload()
      if (!uploadOk) {
        desktopSaving.value = false
        showToast(desktopUploadError.value || '安装包上传失败，请重新选择')
        return
      }
    }
    const res = await saveDesktopVersion({
      platform: desktopDraftPlatform.value,
      system: desktopDraftSystem.value,
      arch: desktopDraftArch.value,
      channel: desktopDraftChannel.value,
      version,
      original_version: desktopEditingVersion.value || '',
      download_url: desktopDraft.value.downloadUrl?.trim() || '',
      update_content: desktopDraft.value.updateContent.trim(),
      enabled: desktopDraftEnabled.value ? 1 : 0,
      store_url: desktopDraft.value.storeUrl,
      file_name: '',
      file_data: '',
    })
    desktopSaving.value = false
    if (res.code === 200) {
      const replaced = !!desktopEditingVersion.value
      loadDesktop()
      if (replaced) {
        showToast('修改成功', 'success')
        desktopModalVisible.value = false
      } else {
        showToast('保存成功，可继续新增版本', 'success')
        desktopEditingVersion.value = ''
        desktopEditingChannel.value = desktopDraftChannel.value
        desktopDraft.value = { version: '', updateContent: '', downloadUrl: '', storeUrl: '' }
        desktopDraftBetaNum.value = ''
        desktopDraftArch.value = defaultArch(desktopDraftPlatform.value)
        desktopDraftEnabled.value = false
        desktopPackageFile.value = null
        desktopPackageFileDraft.value = null
        desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
        abortDesktopUpload()
        if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
      }
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  return {
    desktopModalVisible,
    openDesktopModal,
    closeDesktopModal,
    saveDesktop,
  }
}
