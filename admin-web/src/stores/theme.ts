import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'
type ResolvedTheme = 'light' | 'dark'

const STORAGE_KEY = 'xy-admin-theme-mode'
const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

function readStoredMode(): ThemeMode {
  const value = localStorage.getItem(STORAGE_KEY)
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system'
}

function resolveMode(mode: ThemeMode): ResolvedTheme {
  return mode === 'system' ? (mediaQuery.matches ? 'dark' : 'light') : mode
}

function applyTheme(mode: ThemeMode) {
  const resolved = resolveMode(mode)
  document.documentElement.dataset.theme = resolved
  document.documentElement.dataset.themeMode = mode
  document.documentElement.style.colorScheme = resolved
}

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>(readStoredMode())
  const resolvedTheme = ref<ResolvedTheme>(resolveMode(mode.value))

  const modeLabel = computed(() => {
    if (mode.value === 'light') return '浅色'
    if (mode.value === 'dark') return '深色'
    return '跟随浏览器'
  })

  const isDark = computed(() => resolvedTheme.value === 'dark')

  function setMode(nextMode: ThemeMode) {
    mode.value = nextMode
    localStorage.setItem(STORAGE_KEY, nextMode)
    resolvedTheme.value = resolveMode(nextMode)
    applyTheme(nextMode)
  }

  function cycleMode() {
    if (mode.value === 'system') {
      setMode('light')
    } else if (mode.value === 'light') {
      setMode('dark')
    } else {
      setMode('system')
    }
  }

  function init() {
    applyTheme(mode.value)
    resolvedTheme.value = resolveMode(mode.value)
    mediaQuery.addEventListener('change', () => {
      if (mode.value !== 'system') return
      resolvedTheme.value = resolveMode(mode.value)
      applyTheme(mode.value)
    })
  }

  return {
    mode,
    modeLabel,
    resolvedTheme,
    isDark,
    setMode,
    cycleMode,
    init,
  }
})
