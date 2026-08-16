import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { adminApi } from '@/api/client'

export type NotifyPermission = 'default' | 'granted' | 'denied' | 'unsupported'

const STORAGE_KEY = 'xy-admin-notify'

export interface NotifyModuleMap {
  feedback: boolean
  wallpaper: boolean
  avatar: boolean
  nickname: boolean
}

interface PersistedState {
  /** 是否启用浏览器通知 */
  enabled: boolean
  /** 各模块开关 */
  modules: NotifyModuleMap
  /** 最近一次轮询到的各模块待处理数量（用于比对增量） */
  totals: Record<string, number>
  /** 是否已完成首次基线（避免开启瞬间批量弹通知） */
  baseline: boolean
}

const DEFAULT_MODULES: NotifyModuleMap = {
  feedback: true,
  wallpaper: true,
  avatar: true,
  nickname: true,
}

export const MODULE_META: { key: keyof NotifyModuleMap; label: string; desc: string }[] = [
  { key: 'feedback', label: '新反馈', desc: '用户反馈待处理' },
  { key: 'wallpaper', label: '新壁纸', desc: '壁纸上传待审核' },
  { key: 'avatar', label: '新头像', desc: '头像变更待审核' },
  { key: 'nickname', label: '新名称', desc: '改名申请待审核' },
]

/** window.NativeBridge：web-to-app 原生桥接接口（仅作兜底与增强）。
    正常路径优先使用标准 Web Notification API——web-to-app 的 polyfill 会把
    window.Notification 桥接成安卓系统通知，因此网页代码无需直接依赖原生方法；
    仅当标准 API 不可用，或需要"打开系统设置"等原生能力时，才降级使用本接口。 */
interface NativeBridgeApi {
  getNotificationPermissionState(): string
  requestNotificationPermission(): string
  showWebNotification(title: string, body?: string, tag?: string): boolean
  openNotificationSettings(): boolean
  areNotificationsEnabled(): boolean
}

function nativeBridge(): NativeBridgeApi | undefined {
  if (typeof window === 'undefined') return undefined
  return (window as unknown as { NativeBridge?: NativeBridgeApi }).NativeBridge
}

function isSupported(): boolean {
  return typeof window !== 'undefined' && ('Notification' in window || !!nativeBridge())
}

function readPermission(): NotifyPermission {
  if (!isSupported()) return 'unsupported'
  // 优先标准 Web Notification API：web-to-app 的 polyfill 会把 Notification.permission
  // 桥接成安卓系统通知权限状态，因此直接读 window.Notification 即可感知原生授权结果
  if ('Notification' in window) {
    try {
      const p = Notification.permission
      if (p === 'granted') return 'granted'
      if (p === 'denied') return 'denied'
      return 'default'
    } catch {
      /* 标准 API 不可用时回退到原生桥接 */
    }
  }
  const bridge = nativeBridge()
  if (bridge) {
    try {
      const s = bridge.getNotificationPermissionState()
      if (s === 'granted') return 'granted'
      if (s === 'denied') return 'denied'
      return 'default'
    } catch {
      return 'default'
    }
  }
  return Notification.permission as NotifyPermission
}

function loadState(): PersistedState {
  const fallback: PersistedState = {
    enabled: false,
    modules: { ...DEFAULT_MODULES },
    totals: {},
    baseline: false,
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return fallback
    const parsed = JSON.parse(raw)
    return {
      enabled: !!parsed.enabled,
      modules: { ...DEFAULT_MODULES, ...(parsed.modules || {}) },
      totals: parsed.totals || {},
      baseline: !!parsed.baseline,
    }
  } catch {
    return fallback
  }
}

function persist(state: PersistedState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch {
    /* 忽略存储失败 */
  }
}

export const useNotificationStore = defineStore('notification', () => {
  const state = ref<PersistedState>(loadState())
  const permission = ref<NotifyPermission>(readPermission())

  const enabled = computed(() => state.value.enabled)
  const modules = computed(() => state.value.modules)
  const granted = computed(() => permission.value === 'granted')
  const canNotify = computed(() => granted.value && state.value.enabled)
  const nativeBridgeAvailable = computed(() => !!nativeBridge())
  const supportedByBrowser = computed(() => isSupported())

  /** 铃铛角标：最近一次轮询到的待处理总数 */
  const pendingTotal = computed(() =>
    Object.values(state.value.totals).reduce((sum, n) => sum + Number(n || 0), 0),
  )

  const permissionLabel = computed(() => {
    switch (permission.value) {
      case 'granted':
        return '已允许'
      case 'denied':
        return '已拒绝'
      case 'unsupported':
        return '浏览器不支持'
      default:
        return '未授权'
    }
  })

  async function requestPermission(): Promise<NotifyPermission> {
    if (!isSupported()) {
      permission.value = 'unsupported'
      return 'unsupported'
    }
    // 标准 Web Notification API：web-to-app 的 polyfill 会把 Notification.requestPermission()
    // 转发到原生，拉起安卓系统授权弹窗（实现了"网页通知 → 系统通知"的联动）
    if ('Notification' in window && typeof Notification.requestPermission === 'function') {
      try {
        const p = await Notification.requestPermission()
        permission.value = (['granted', 'denied', 'default'].includes(p)
          ? p
          : readPermission()) as NotifyPermission
        if (permission.value !== 'default') return permission.value
        // Android 13+ 系统权限弹窗为异步结果，轮询等待用户响应
        for (let i = 0; i < 10; i++) {
          await new Promise((r) => setTimeout(r, 500))
          permission.value = readPermission()
          if (permission.value !== 'default') return permission.value
        }
        return permission.value
      } catch {
        /* 标准 API 失败时回退到原生桥接 */
      }
    }
    const bridge = nativeBridge()
    if (bridge) {
      try {
        bridge.requestNotificationPermission()
      } catch {
        /* 忽略 */
      }
      for (let i = 0; i < 10; i++) {
        await new Promise((r) => setTimeout(r, 500))
        permission.value = readPermission()
        if (permission.value !== 'default') return permission.value
      }
      return permission.value
    }
    try {
      const p = await Notification.requestPermission()
      permission.value = p as NotifyPermission
      return permission.value
    } catch {
      return permission.value
    }
  }

  function refreshPermission() {
    permission.value = readPermission()
  }

  function openNotificationSettings(): boolean {
    const bridge = nativeBridge()
    if (!bridge) return false
    try {
      return bridge.openNotificationSettings()
    } catch {
      return false
    }
  }

  function setEnabled(value: boolean) {
    state.value.enabled = value
    persist(state.value)
  }

  function toggleModule(key: keyof NotifyModuleMap) {
    state.value.modules[key] = !state.value.modules[key]
    persist(state.value)
  }

  function showNotification(title: string, options: NotificationOptions = {}) {
    // 标准 Web Notification API：web-to-app 的 polyfill 会把 new Notification() 路由到
    // 安卓系统通知，从而实现"网页审核通知 → 系统通知"
    if ('Notification' in window && Notification.permission === 'granted') {
      try {
        const n = new Notification(title, {
          icon: '/logo.png',
          badge: '/logo.png',
          ...options,
        })
        n.onclick = () => {
          window.focus()
          n.close()
        }
        return true
      } catch {
        /* 标准 API 失败时回退到原生桥接 */
      }
    }
    const bridge = nativeBridge()
    if (bridge) {
      if (readPermission() !== 'granted') return false
      try {
        return bridge.showWebNotification(title, (options.body as string) || '', title)
      } catch {
        return false
      }
    }
    return false
  }

  function testNotification(): boolean {
    return showNotification('测试通知', {
      body: '弦予音乐后台通知设置成功，有新事件时将在此提醒。',
    })
  }

  /** 依据 dashboard_stats 的 pending_* 字段，比对增量并弹通知 */
  function checkStats(stats: any) {
    // 主开关未开启则不弹通知（但仍更新计数，供铃铛角标展示）
    const notifying = state.value.enabled

    const mapping: Record<string, number> = {
      feedback: Number(stats?.pending_feedback ?? 0),
      wallpaper: Number(stats?.pending_wallpapers ?? 0),
      avatar: Number(stats?.pending_avatars ?? 0),
      nickname: Number(stats?.pending_nicknames ?? 0),
    }

    // 首次轮询建立基线，不弹通知，避免开启瞬间批量弹出
    if (!state.value.baseline) {
      state.value.totals = { ...mapping }
      state.value.baseline = true
      persist(state.value)
      return
    }

    if (notifying) {
      for (const meta of MODULE_META) {
        if (!state.value.modules[meta.key]) continue
        const now = mapping[meta.key]
        const prev = state.value.totals[meta.key] ?? 0
        if (now > prev) {
          showNotification(meta.label, {
            body: `${meta.desc}，当前 ${now} 条待处理。`,
          })
        }
      }
    }

    state.value.totals = { ...mapping }
    persist(state.value)
  }

  let timer: number | undefined

  async function pollOnce() {
    const res = await adminApi<any>('dashboard_stats')
    if (res.code === 200 && res.data) {
      checkStats(res.data)
    }
  }

  function startPolling(intervalMs = 30000) {
    stopPolling()
    pollOnce()
    timer = window.setInterval(pollOnce, intervalMs)
  }

  function stopPolling() {
    if (timer !== undefined) {
      window.clearInterval(timer)
      timer = undefined
    }
  }

  return {
    enabled,
    modules,
    permission,
    granted,
    canNotify,
    nativeBridgeAvailable,
    supportedByBrowser,
    pendingTotal,
    permissionLabel,
    moduleList: MODULE_META,
    requestPermission,
    refreshPermission,
    setEnabled,
    toggleModule,
    showNotification,
    openNotificationSettings,
    testNotification,
    checkStats,
    startPolling,
    stopPolling,
  }
})