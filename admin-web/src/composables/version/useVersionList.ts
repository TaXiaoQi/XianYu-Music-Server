import { computed, ref } from 'vue'
import type { Ref } from 'vue'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { deleteDesktopVersion, fetchDesktopVersions, saveDesktopVersion } from '@/api/version'
import {
  SYSTEM_META,
  archOf,
  channelOf,
  pkgOf,
  platformLabelKey,
  platformLabelOf,
  platformOf,
  systemLabelKey,
  systemLabelOf,
  systemOf,
} from './versionMeta'
import type { PlatformKey } from './versionMeta'

interface VersionListDeps {
  desktopSaving: Ref<boolean>
}

// 版本列表与筛选：平台/系统切换、启用统计、卡片启用禁用与删除
export function useVersionList(deps: VersionListDeps) {
  const { desktopSaving } = deps

  const desktopList = ref<any[]>([])
  const desktopLoading = ref(true)
  const batchModalVisible = ref(false)
  const platformFilter = ref<PlatformKey>('desktop')
  const systemFilter = ref<string>('')

  const systemList = computed(() => SYSTEM_META[platformFilter.value] || [])
  const currentPlatformLabel = computed(() => platformLabelKey(platformFilter.value))
  const currentSubLabel = computed(() =>
    systemFilter.value
      ? `${systemLabelKey(platformFilter.value, systemFilter.value)}${currentPlatformLabel.value}`
      : currentPlatformLabel.value
  )
  const platformList = computed(() => desktopList.value.filter(v => platformOf(v) === platformFilter.value))
  const filteredList = computed(() => {
    if (!systemFilter.value) return platformList.value
    return platformList.value.filter(v => systemOf(v) === systemFilter.value)
  })
  const enabledCount = computed(() => filteredList.value.filter(v => v.enabled).length)
  const disabledCount = computed(() => filteredList.value.length - enabledCount.value)

  function switchPlatform(key: PlatformKey) {
    if (platformFilter.value === key || desktopSaving.value) return
    platformFilter.value = key
    systemFilter.value = ''
  }

  function switchSystem(key: string) {
    if (desktopSaving.value) return
    systemFilter.value = key
  }

  async function loadDesktop() {
    desktopLoading.value = true
    const res = await fetchDesktopVersions()
    if (res.code === 200 && res.data) {
      desktopList.value = Array.isArray(res.data.list) ? res.data.list : []
    }
    desktopLoading.value = false
  }

  async function toggleDesktop(e: Event, item: any) {
    const enabled = (e.target as HTMLInputElement).checked
    const res = await saveDesktopVersion({
      platform: platformOf(item),
      system: systemOf(item),
      arch: archOf(item),
      pkg: pkgOf(item),
      channel: channelOf(item),
      version: item.version,
      download_url: item.downloadUrl || '',
      update_content: item.updateContent || '',
      enabled: enabled ? 1 : 0,
      store_url: item.storeUrl || '',
      file_name: '',
      file_data: '',
    })
    if (res.code === 200) {
      showToast(enabled ? '已启用' : '已禁用', 'success')
    } else {
      showToast(res.msg || '操作失败')
    }
    loadDesktop()
  }

  async function deleteDesktop(item: any) {
    const ok = await webConfirm(`确认删除${systemLabelOf(item)}${platformLabelOf(item)} v${item.version} 的更新配置？`, { title: '删除配置', confirmText: '确认删除' })
    if (!ok) return
    const res = await deleteDesktopVersion({ platform: platformOf(item), system: systemOf(item), arch: archOf(item), pkg: pkgOf(item), version: item.version })
    if (res.code === 200) {
      showToast('删除成功', 'success')
      loadDesktop()
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  return {
    desktopList,
    desktopLoading,
    batchModalVisible,
    platformFilter,
    systemFilter,
    systemList,
    currentPlatformLabel,
    currentSubLabel,
    platformList,
    filteredList,
    enabledCount,
    disabledCount,
    switchPlatform,
    switchSystem,
    loadDesktop,
    toggleDesktop,
    deleteDesktop,
  }
}
