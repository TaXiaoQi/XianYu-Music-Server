import { computed, ref } from 'vue'
import { SYSTEM_META, defaultArch, defaultSystem } from './versionMeta'
import type { PlatformKey } from './versionMeta'

// 新增/编辑弹窗的草稿状态与维度（平台 / 系统 / 架构 / 渠道 / 版本号）
export function useVersionDraft() {
  const desktopDraft = ref<{ version: string; updateContent: string; downloadUrl: string; storeUrl: string }>({ version: '', updateContent: '', downloadUrl: '', storeUrl: '' })
  const desktopDraftEnabled = ref(false)
  const desktopDraftPlatform = ref<PlatformKey>('desktop')
  const desktopDraftSystem = ref<string>('windows')
  const desktopDraftArch = ref<string>('x86')
  const desktopDraftChannel = ref<'stable' | 'beta'>('stable')
  const desktopDraftBetaNum = ref('')
  const desktopEditingVersion = ref('')
  const desktopEditingChannel = ref<'stable' | 'beta'>('stable')

  const composedVersion = computed(() => {
    if (desktopDraftChannel.value !== 'beta') return desktopDraft.value.version.trim()
    const main = desktopDraft.value.version.trim()
    const num = (parseInt(desktopDraftBetaNum.value, 10) || 0)
    if (!main || num <= 0) return ''
    return `${main}-beta-${num}`
  })

  const watchArchLocked = computed(() =>
    desktopDraftPlatform.value === 'watch' && (desktopDraftSystem.value === 'ohos' || desktopDraftSystem.value === 'watchos')
  )

  function switchDraftPlatform(key: PlatformKey) {
    if (desktopEditingVersion.value) return
    desktopDraftPlatform.value = key
    if (!SYSTEM_META[key]?.some(s => s.key === desktopDraftSystem.value)) {
      desktopDraftSystem.value = defaultSystem(key)
    }
    desktopDraftArch.value = defaultArch(key)
  }

  function switchDraftSystem(key: string) {
    desktopDraftSystem.value = key
    if (desktopDraftPlatform.value === 'watch' && (key === 'ohos' || key === 'watchos')) {
      desktopDraftArch.value = 'arm64'
    }
  }

  return {
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
    watchArchLocked,
    switchDraftPlatform,
    switchDraftSystem,
  }
}

export type VersionDraft = ReturnType<typeof useVersionDraft>
