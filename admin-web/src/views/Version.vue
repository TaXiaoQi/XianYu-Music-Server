<template>
  <div class="version-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">版本管理</h2>
          <p class="page-desc">
            按客户端平台管理在线更新配置。新增或编辑版本后，卡片实时刷新。启用的版本只对所选平台的客户端生效。
          </p>
        </div>
        <div class="header-actions">
          <button class="btn-beta" @click="openBetaModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
            内测名单
            <span v-if="betaList.length" class="beta-count">{{ betaList.length }}</span>
          </button>
          <button class="btn-add" @click="openDesktopModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            新增版本
          </button>
        </div>
      </div>
    </Transition>

    <!-- 平台切换 -->
    <div class="platform-tabs">
      <button
        v-for="p in PLATFORMS"
        :key="p.key"
        class="platform-tab"
        :class="{ active: platformFilter === p.key }"
        @click="switchPlatform(p.key)"
      >{{ p.label }}</button>
    </div>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ desktopList.length }}</span>
            <span class="stat-label">全部</span>
          </div>
        </div>
        <div class="stat-chip stat-on">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ enabledCount }}</span>
            <span class="stat-label">已启用</span>
          </div>
        </div>
        <div class="stat-chip stat-off">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ disabledCount }}</span>
            <span class="stat-label">已禁用</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 在线更新卡片（按当前平台过滤） -->
    <Transition name="fade-up" appear>
      <div class="desktop-section">
        <div class="section-label">
          <span class="section-dot" :class="`dot-${platformFilter}`"></span>
          <h3>{{ currentPlatformLabel }}在线更新</h3>
          <span class="section-hint">{{ currentPlatformLabel }}启动时自动比对版本号，低于此版本将弹窗提示更新</span>
        </div>

        <div v-if="desktopLoading" class="state-box"><div class="spinner"></div><span>加载中...</span></div>

        <div v-else-if="platformList.length === 0" class="desktop-empty">
          <p>暂未配置{{ currentPlatformLabel }}更新版本</p>
          <button class="btn-add-small" @click="openDesktopModal()">+ 新增配置</button>
        </div>

        <div v-else class="card-grid">
          <TransitionGroup name="card">
            <div
              v-for="(item, idx) in platformList"
              :key="`${item.platform || 'desktop'}-${item.version}`"
              class="ann-card"
              :class="{ disabled: !item.enabled }"
              :style="{ animationDelay: `${idx * 60}ms` }"
            >
              <div class="type-bar" :class="`bar-${item.platform || 'desktop'}`"></div>
              <div class="card-body">
                <div class="card-top">
                  <div class="card-badges">
                    <span class="type-badge" :class="`badge-${platformOf(item)}`">{{ platformLabelOf(item) }}</span>
                    <span v-if="channelOf(item) === 'beta'" class="type-badge badge-beta">测试版</span>
                  </div>
                  <label class="toggle-switch" :title="item.enabled ? '点击禁用' : '点击启用'">
                    <input type="checkbox" :checked="item.enabled" @change="toggleDesktop($event, item)" />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <h3 class="card-title">v{{ item.version || '-' }}</h3>
                <p class="card-content">{{ item.updateContent || '无更新说明' }}</p>
                <div v-if="item.downloadUrl" class="card-link">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                  <a :href="item.downloadUrl" target="_blank">{{ item.downloadUrl }}</a>
                </div>
                <div class="card-footer">
                  <span class="card-date">
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                    {{ item.updated_at || '-' }}
                  </span>
                  <div class="card-actions">
                    <button class="icon-btn" title="编辑" @click="openDesktopModal(item)">
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                    </button>
                    <button class="icon-btn icon-btn-danger" title="删除" @click="deleteDesktop(item)">
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </div>
    </Transition>

    <!-- 桌面端配置弹窗 -->
    <Transition name="modal">
      <div v-if="desktopModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ desktopEditingVersion ? `编辑${platformLabelKey(desktopDraftPlatform)}配置` : `新增${platformLabelKey(desktopDraftPlatform)}配置` }}</h3>
            <button class="modal-close" @click="closeDesktopModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field">
              <label class="required">客户端平台</label>
              <div class="type-picker platform-picker">
                <button
                  v-for="p in PLATFORMS"
                  :key="p.key"
                  class="type-option pick-platform"
                  :class="{ active: desktopDraftPlatform === p.key, locked: !!desktopEditingVersion }"
                  :disabled="!!desktopEditingVersion"
                  @click="desktopDraftPlatform = p.key"
                >
                  <span class="pick-dot"></span>{{ p.label }}
                </button>
              </div>
            </div>
            <div class="field">
              <label class="required">更新渠道</label>
              <div class="type-picker channel-picker">
                <button class="type-option pick-stable" :class="{ active: desktopDraftChannel === 'stable' }" @click="desktopDraftChannel = 'stable'">
                  <span class="pick-dot"></span>正式版
                </button>
                <button class="type-option pick-beta-opt" :class="{ active: desktopDraftChannel === 'beta' }" @click="desktopDraftChannel = 'beta'">
                  <span class="pick-dot"></span>测试版
                </button>
              </div>
              <p v-if="desktopDraftChannel === 'beta'" class="field-hint">测试版仅对「内测名单」中的设备ID下发，普通用户检测更新与官网下载均不受影响。</p>
            </div>
            <div class="field">
              <label class="required">版本号</label>
              <input v-if="desktopDraftChannel === 'stable'" v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
              <div v-else class="version-combo">
                <input v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
                <span class="version-combo-sep">-beta-</span>
                <input v-model="desktopDraftBetaNum" type="text" inputmode="numeric" placeholder="1" class="version-combo-num" />
              </div>
            </div>
            <div class="field">
              <label>下载渠道</label>
              <button type="button" class="channel-card" @click="openDesktopChannelModal">
                <div>
                  <strong>{{ desktopChannelLabel }}</strong>
                  <p>{{ desktopChannelDesc }}</p>
                </div>
                <span>选择渠道</span>
              </button>
            </div>
            <div class="field">
              <label>更新内容</label>
              <textarea v-model="desktopDraft.updateContent" rows="18" placeholder="本次更新内容"></textarea>
            </div>
            <div class="field">
              <label>启用状态</label>
              <div class="type-picker">
                <button class="type-option pick-enable" :class="{ active: desktopDraftEnabled }" @click="desktopDraftEnabled = true">
                  <span class="pick-dot"></span>启用
                </button>
                <button class="type-option pick-disable" :class="{ active: !desktopDraftEnabled }" @click="desktopDraftEnabled = false">
                  <span class="pick-dot"></span>禁用
                </button>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeDesktopModal">取消</button>
            <button class="btn-save" :disabled="desktopSaving" @click="saveDesktop">
              <span v-if="desktopSaving" class="btn-spinner"></span>
              {{ desktopSaving ? '保存中...' : '保存配置' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 桌面端下载渠道弹窗 -->
    <Transition name="modal">
      <div v-if="desktopChannelModalVisible" class="modal-backdrop channel-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>选择下载渠道</h3>
            <button class="modal-close" @click="closeDesktopChannelModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="channel-options">
              <button type="button" class="channel-option" :class="{ active: desktopChannelMode === 'upload' }" @click="desktopChannelMode = 'upload'">
                <strong>上传安装包</strong>
                <span>安装包保存到本服务端，并自动生成下发链接。</span>
              </button>
              <button type="button" class="channel-option" :class="{ active: desktopChannelMode === 'link' }" @click="desktopChannelMode = 'link'">
                <strong>填写下载链接</strong>
                <span>适合安装包已放在其它文件服务器或网盘直链。</span>
              </button>
            </div>
            <div v-if="desktopChannelMode === 'upload'" class="field">
              <label class="required">安装包</label>
              <div class="package-dropzone" :class="{ dragging: desktopPackageDragging, selected: !!desktopPackageDraft.fileName }" @click="triggerDesktopFileInput" @dragover.prevent="desktopPackageDragging = true" @dragleave.prevent="desktopPackageDragging = false" @drop.prevent="onDesktopPackageDrop">
                <input ref="desktopFileInputRef" type="file" accept=".exe,.msi,.zip,.7z,.rar,.dmg,.pkg,.apk" class="file-hidden" @change="onDesktopFileChange" />
                <div class="dropzone-icon">⬆</div>
                <strong>{{ desktopPackageDraft.fileName ? '已选择安装包' : '点击或拖拽安装包到此处' }}</strong>
                <span>支持 EXE / MSI / ZIP / 7Z / RAR / DMG / PKG / APK</span>
              </div>
              <div v-if="desktopPackageDraft.fileName" class="file-info">已选择：{{ desktopPackageDraft.fileName }}（{{ formatFileSize(desktopPackageDraft.fileSize) }}）</div>
            </div>
            <div v-else class="field">
              <label class="required">下载链接</label>
              <input v-model="desktopChannelLinkDraft" type="text" placeholder="https://..." />
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeDesktopChannelModal">取消</button>
            <button class="btn-save" @click="confirmDesktopChannel">确定</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 内测名单弹窗 -->
    <Transition name="modal">
      <div v-if="betaModalVisible" class="modal-backdrop beta-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>内测名单</h3>
            <button class="modal-close" @click="closeBetaModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <p class="field-hint">名单内的设备ID可收到「测试版」渠道的更新下发；正式版与官网下载对所有用户不变。设备ID可在客户端「关于/反馈」上报数据或设备管理中查询。</p>
            <div class="beta-add-row">
              <input v-model="betaDeviceIdDraft" type="text" placeholder="输入设备ID" @keyup.enter="addBetaTester" />
              <input v-model="betaNoteDraft" type="text" placeholder="备注（可选）" class="beta-note-input" @keyup.enter="addBetaTester" />
              <button class="btn-save" :disabled="betaSaving" @click="addBetaTester">{{ betaSaving ? '添加中...' : '添加' }}</button>
            </div>
            <div v-if="betaLoading" class="state-box"><div class="spinner"></div><span>加载中...</span></div>
            <div v-else-if="betaList.length === 0" class="beta-empty">暂无内测设备，添加后测试版将仅对这些设备下发</div>
            <div v-else class="beta-list">
              <TransitionGroup name="card">
                <div v-for="t in betaList" :key="t.id" class="beta-item" :style="{ animationDelay: '0ms' }">
                  <div class="beta-item-main">
                    <span class="beta-device-id" :title="t.device_id">{{ t.device_id }}</span>
                    <span v-if="t.note" class="beta-item-note">{{ t.note }}</span>
                  </div>
                  <div class="beta-item-side">
                    <span class="beta-item-date">{{ t.created_at || '-' }}</span>
                    <button class="icon-btn icon-btn-danger" title="移除" @click="removeBetaTester(t)">
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                    </button>
                  </div>
                </div>
              </TransitionGroup>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeBetaModal">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

type PlatformKey = 'desktop' | 'mobile' | 'watch'

const PLATFORMS: { key: PlatformKey; label: string }[] = [
  { key: 'desktop', label: '桌面端' },
  { key: 'mobile', label: '移动端' },
  { key: 'watch', label: '腕上端' },
]

function platformOf(item: any): PlatformKey {
  const p = item?.platform
  return p === 'mobile' || p === 'watch' ? p : 'desktop'
}

function channelOf(item: any): 'stable' | 'beta' {
  return item?.channel === 'beta' ? 'beta' : 'stable'
}

function platformLabelKey(key: string): string {
  return PLATFORMS.find(p => p.key === key)?.label || '桌面端'
}

// ===== 版本配置 =====
const desktopList = ref<any[]>([])
const desktopLoading = ref(true)
const platformFilter = ref<PlatformKey>('desktop')
const currentPlatformLabel = computed(() => platformLabelKey(platformFilter.value))
const platformList = computed(() => desktopList.value.filter(v => platformOf(v) === platformFilter.value))
const enabledCount = computed(() => platformList.value.filter(v => v.enabled).length)
const disabledCount = computed(() => platformList.value.length - enabledCount.value)

function platformLabelOf(item: any): string {
  return platformLabelKey(platformOf(item))
}

function switchPlatform(key: PlatformKey) {
  if (platformFilter.value === key || desktopSaving.value) return
  platformFilter.value = key
}

async function loadDesktop() {
  desktopLoading.value = true
  const res = await adminApi<any>('get_desktop_version')
  if (res.code === 200 && res.data) {
    desktopList.value = Array.isArray(res.data.list) ? res.data.list : []
  }
  desktopLoading.value = false
}

// 版本配置弹窗
const desktopModalVisible = ref(false)
const desktopDraft = ref<{ version: string; updateContent: string; downloadUrl: string }>({ version: '', updateContent: '', downloadUrl: '' })
const desktopDraftEnabled = ref(false)
const desktopDraftPlatform = ref<PlatformKey>('desktop')
const desktopDraftChannel = ref<'stable' | 'beta'>('stable')
const desktopDraftBetaNum = ref('')
const desktopEditingVersion = ref('')
const desktopEditingChannel = ref<'stable' | 'beta'>('stable')
const desktopSaving = ref(false)
const desktopChannelModalVisible = ref(false)
const desktopChannelMode = ref<'link' | 'upload'>('link')
const desktopChannelLinkDraft = ref('')
const desktopFileInputRef = ref<HTMLInputElement | null>(null)
const desktopPackageFile = ref<File | null>(null)
const desktopPackageFileDraft = ref<File | null>(null)
const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })
const desktopPackageDragging = ref(false)

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

/** 测试版组合版本号：主版本 + -beta- + beta号（如 1.2.0-beta-3） */
const composedVersion = computed(() => {
  if (desktopDraftChannel.value !== 'beta') return desktopDraft.value.version.trim()
  const main = desktopDraft.value.version.trim()
  const num = (parseInt(desktopDraftBetaNum.value, 10) || 0)
  if (!main || num <= 0) return ''
  return `${main}-beta-${num}`
})

function openDesktopModal(item?: any) {
  if (item) {
    const version = item.version || ''
    const betaIdx = version.indexOf('-beta-')
    const isBeta = betaIdx >= 0
    desktopEditingVersion.value = version
    desktopEditingChannel.value = isBeta ? 'beta' : 'stable'
    desktopDraftPlatform.value = platformOf(item)
    desktopDraftChannel.value = isBeta ? 'beta' : 'stable'
    desktopDraft.value = {
      version: isBeta ? version.slice(0, betaIdx) : version,
      updateContent: item.updateContent || '',
      downloadUrl: item.downloadUrl || '',
    }
    desktopDraftBetaNum.value = isBeta ? version.slice(betaIdx + '-beta-'.length) : ''
    desktopDraftEnabled.value = !!item.enabled
  } else {
    desktopEditingVersion.value = ''
    desktopEditingChannel.value = 'stable'
    desktopDraftPlatform.value = platformFilter.value
    desktopDraftChannel.value = 'stable'
    desktopDraft.value = { version: '', updateContent: '', downloadUrl: '' }
    desktopDraftBetaNum.value = ''
    desktopDraftEnabled.value = false
  }
  desktopPackageFile.value = null
  desktopPackageFileDraft.value = null
  desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
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
  let fileData = ''
  if (desktopPackageFile.value) {
    try {
      fileData = await readFileAsBase64(desktopPackageFile.value)
    } catch {
      desktopSaving.value = false
      showToast('安装包读取失败')
      return
    }
  }
  const res = await adminApi('save_desktop_version', {
    platform: desktopDraftPlatform.value,
    channel: desktopDraftChannel.value,
    version,
    download_url: desktopDraft.value.downloadUrl?.trim() || '',
    update_content: desktopDraft.value.updateContent.trim(),
    enabled: desktopDraftEnabled.value ? 1 : 0,
    file_name: desktopPackageFile.value?.name || '',
    file_data: fileData,
  })
  desktopSaving.value = false
  if (res.code === 200) {
    const replaced = !!desktopEditingVersion.value && desktopEditingVersion.value === version && desktopEditingChannel.value === desktopDraftChannel.value
    loadDesktop()
    if (replaced) {
      showToast('修改成功', 'success')
      desktopModalVisible.value = false
    } else {
      // 新增成功：保持弹窗打开并清空表单（保留平台与渠道），方便继续新增多个版本
      showToast('保存成功，可继续新增版本', 'success')
      desktopEditingVersion.value = ''
      desktopEditingChannel.value = desktopDraftChannel.value
      desktopDraft.value = { version: '', updateContent: '', downloadUrl: '' }
      desktopDraftBetaNum.value = ''
      desktopDraftEnabled.value = false
      desktopPackageFile.value = null
      desktopPackageFileDraft.value = null
      desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
      if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
    }
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function toggleDesktop(e: Event, item: any) {
  const enabled = (e.target as HTMLInputElement).checked
  const res = await adminApi('save_desktop_version', {
    platform: platformOf(item),
    channel: channelOf(item),
    version: item.version,
    download_url: item.downloadUrl || '',
    update_content: item.updateContent || '',
    enabled: enabled ? 1 : 0,
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
  const ok = await webConfirm(`确认删除${platformLabelOf(item)} v${item.version} 的更新配置？`, { title: '删除配置', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_desktop_version', { platform: platformOf(item), version: item.version })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadDesktop()
  } else {
    showToast(res.msg || '删除失败')
  }
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
  }
  desktopChannelModalVisible.value = false
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
  const allowed = ['exe', 'msi', 'zip', '7z', 'rar', 'dmg', 'pkg', 'apk']
  if (!allowed.includes(ext)) {
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

// ===== 内测名单 =====
const betaModalVisible = ref(false)
const betaList = ref<any[]>([])
const betaLoading = ref(false)
const betaSaving = ref(false)
const betaDeviceIdDraft = ref('')
const betaNoteDraft = ref('')

async function loadBeta() {
  betaLoading.value = true
  const res = await adminApi<any>('list_beta_testers')
  if (res.code === 200 && res.data) {
    betaList.value = Array.isArray(res.data.list) ? res.data.list : []
  }
  betaLoading.value = false
}

function openBetaModal() {
  betaModalVisible.value = true
  if (!betaLoading.value && betaList.value.length === 0) loadBeta()
}

function closeBetaModal() {
  betaModalVisible.value = false
}

async function addBetaTester() {
  const deviceId = betaDeviceIdDraft.value.trim()
  if (!deviceId) {
    showToast('请输入设备ID')
    return
  }
  betaSaving.value = true
  const res = await adminApi('add_beta_tester', { device_id: deviceId, note: betaNoteDraft.value.trim() })
  betaSaving.value = false
  if (res.code === 200) {
    showToast('已添加到内测名单', 'success')
    betaDeviceIdDraft.value = ''
    betaNoteDraft.value = ''
    loadBeta()
  } else {
    showToast(res.msg || '添加失败')
  }
}

async function removeBetaTester(t: any) {
  const ok = await webConfirm(`确认将设备 ${t.device_id} 移出内测名单？`, { title: '移除内测设备', confirmText: '确认移除' })
  if (!ok) return
  const res = await adminApi('delete_beta_tester', { device_id: t.device_id })
  if (res.code === 200) {
    showToast('已移除', 'success')
    loadBeta()
  } else {
    showToast(res.msg || '移除失败')
  }
}

onMounted(() => {
  loadDesktop()
  loadBeta()
})
</script>

<style scoped>
.version-page {
  max-width: 1200px;
  margin: 0 auto;
}

/* 统计卡片 */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}
.stat-chip {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.2s;
}
.stat-chip:hover { transform: translateY(-3px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-total .stat-icon { background: var(--track); color: var(--text-light); }
.stat-on .stat-icon { background: rgba(34, 197, 94, 0.14); color: #16a34a; }
.stat-off .stat-icon { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; }
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; flex-direction: column; align-items: flex-start; gap: 8px; }
  .stat-num { font-size: 22px; }
}

/* ===== 页面头部 ===== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 20px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0 0 6px 0;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 580px;
}

/* 新增按钮 */
.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}
.btn-add:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
}
.btn-add:active { transform: scale(0.96); }

/* 内测名单按钮 */
.btn-beta {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: 1.5px solid rgba(245, 158, 11, 0.55);
  background: rgba(245, 158, 11, 0.1);
  color: #d97706;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-beta:hover {
  background: rgba(245, 158, 11, 0.18);
  transform: translateY(-2px);
  box-shadow: 0 4px 14px rgba(245, 158, 11, 0.22);
}
.btn-beta:active { transform: scale(0.96); }
.beta-count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 999px;
  background: #f59e0b;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.btn-add-small {
  display: inline-flex;
  align-items: center;
  padding: 8px 16px;
  border-radius: 8px;
  border: 1.5px dashed var(--accent);
  background: transparent;
  color: var(--accent);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-add-small:hover {
  background: var(--accent-soft);
}

/* ===== 区块标签 ===== */
.desktop-section, .app-section {
  margin-bottom: 28px;
}
.section-label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}
.section-label h3 {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
}
.section-hint {
  font-size: 12px;
  color: var(--text-muted);
}
.section-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-desktop { background: #3b82f6; }
.dot-mobile { background: #10b981; }
.dot-watch { background: #8b5cf6; }
.dot-app { background: #10b981; }

/* 平台切换 tab */
.platform-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 20px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 12px;
}
.platform-tab {
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-light);
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.platform-tab:hover { color: var(--text); }
.platform-tab.active {
  background: var(--accent-soft);
  color: var(--accent);
}

/* ===== 加载/错误/空状态 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 14px;
}
.state-error { color: #ef4444; }
.state-empty { padding: 80px 20px; }
.empty-icon {
  color: #d1d5db;
  margin-bottom: 4px;
}
.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-light);
  margin: 0;
}
.empty-sub {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ===== 桌面端空状态 ===== */
.desktop-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px 20px;
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  color: var(--text-muted);
  font-size: 14px;
}

/* ===== 桌面端卡片 ===== */
.desktop-card {
  display: flex;
  background: var(--card-solid);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  max-width: 480px;
}
.desktop-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
.desktop-card.disabled { opacity: 0.6; }
.desktop-card.disabled:hover { opacity: 0.85; }

/* ===== 卡片网格 ===== */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}

.ann-card {
  display: flex;
  background: var(--card-solid);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.ann-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
.ann-card.disabled { opacity: 0.6; }
.ann-card.disabled:hover { opacity: 0.85; }

@keyframes cardEnter {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 类型指示条 */
.type-bar {
  width: 4px;
  flex-shrink: 0;
}
.bar-desktop { background: #3b82f6; }
.bar-mobile { background: #10b981; }
.bar-watch { background: #8b5cf6; }
.bar-normal { background: #10b981; }
.bar-update { background: #3b82f6; }
.bar-force { background: #f59e0b; }
.bar-disabled { background: #9ca3af; }
.bar-crash { background: #ef4444; }
.bar-group { background: #8b5cf6; }

/* 卡片内容 */
.card-body {
  flex: 1;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* 类型徽章 */
.type-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
}
.badge-desktop { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.badge-mobile { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.badge-watch { background: rgba(139, 92, 246, 0.12); color: #8b5cf6; }
.badge-beta { background: rgba(245, 158, 11, 0.14); color: #d97706; }
.card-badges { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.badge-normal { background: rgba(34, 197, 94, 0.14); color: #10b981; }
.badge-update { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.badge-force { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.badge-disabled { background: var(--track); color: var(--text-muted); }
.badge-crash { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.badge-group { background: rgba(139, 92, 246, 0.12); color: #8b5cf6; }

/* Toggle 开关 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 38px;
  height: 22px;
  cursor: pointer;
  flex-shrink: 0;
}
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--track);
  border-radius: 22px;
  transition: background 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 3px;
  top: 3px;
  background: var(--card-solid);
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-switch input:checked + .toggle-slider { background: #10b981; }
.toggle-switch input:checked + .toggle-slider::before { transform: translateX(16px); }

/* 标题和正文 */
.card-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.ver-code {
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: 999px;
  margin-left: 4px;
}
.card-content {
  font-size: 13px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 链接 */
.card-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-muted);
}
.card-link a {
  color: #6366f1;
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
  transition: color 0.15s;
}
.card-link a:hover { color: #4f46e5; text-decoration: underline; }

/* 底部 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid #f5f5f5;
}
.card-date {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}
.card-actions {
  display: flex;
  gap: 4px;
}
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.icon-btn:hover {
  background: var(--track);
  color: var(--text);
}
.icon-btn-danger:hover {
  background: rgba(236, 65, 65, 0.12);
  color: #ef4444;
}

/* ===== 分页 ===== */
.pagination {
  display: flex;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.pagination button.active {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal-backdrop.channel-backdrop { z-index: 10010; }
.modal-dialog {
  background: var(--card-solid);
  border-radius: 18px;
  width: 100%;
  max-width: 560px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.16);
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
}
.modal-head h3 {
  font-size: 17px;
  font-weight: 700;
  margin: 0;
}
.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.modal-close:hover { background: var(--track); color: var(--text); }

.modal-form {
  padding: 0 24px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field input,
.field textarea {
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: var(--control-bg);
  font-family: inherit;
  resize: vertical;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus,
.field textarea:focus {
  border-color: var(--accent);
  background: var(--card-solid);
}

/* 渠道卡片 */
.channel-card {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--control-bg);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
}
.channel-card:hover {
  border-color: var(--accent);
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
  transform: translateY(-1px);
}
.channel-card strong { display: block; font-size: 14px; margin-bottom: 4px; }
.channel-card p { margin: 0; font-size: 12px; color: var(--text-muted); word-break: break-all; }
.channel-card > span { flex-shrink: 0; font-size: 12px; color: var(--accent); font-weight: 700; }

/* 类型选择器 */
.type-picker {
  display: flex;
  gap: 8px;
}
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: var(--control-bg);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.type-option:hover { border-color: var(--text-muted); }
.type-option.active {
  border-color: transparent;
  font-weight: 600;
}
.pick-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--track);
  transition: background 0.2s;
}
.pick-enable.active { background: rgba(34, 197, 94, 0.14); color: #10b981; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: var(--control-bg); color: #6b7280; }
.pick-disable.active .pick-dot { background: #6b7280; }

/* 平台选择器 */
.platform-picker .pick-platform.active { background: var(--accent-soft); color: var(--accent); }
.platform-picker .pick-platform.active .pick-dot { background: var(--accent); }
.platform-picker .pick-platform.locked { cursor: not-allowed; opacity: 0.7; }

/* 渠道选择器（正式版 / 测试版） */
.pick-stable.active { background: rgba(34, 197, 94, 0.14); color: #10b981; }
.pick-stable.active .pick-dot { background: #10b981; }
.pick-beta-opt.active { background: rgba(245, 158, 11, 0.14); color: #d97706; }
.pick-beta-opt.active .pick-dot { background: #f59e0b; }

.field-hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
  margin: 0;
}

/* 测试版组合版本号：版本号 -beta- beta号 */
.version-combo {
  display: flex;
  align-items: center;
  gap: 8px;
}
.version-combo input {
  flex: 1;
  min-width: 0;
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: var(--control-bg);
  font-family: inherit;
  transition: border-color 0.2s, background 0.2s;
}
.version-combo input:focus { border-color: var(--accent); background: var(--card-solid); }
.version-combo-sep {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 700;
  color: #d97706;
  background: rgba(245, 158, 11, 0.12);
  padding: 6px 10px;
  border-radius: 8px;
  user-select: none;
}
.version-combo-num { max-width: 90px !important; }

/* 内测名单弹窗 */
.beta-backdrop { z-index: 10010; }
.beta-add-row {
  display: flex;
  gap: 8px;
}
.beta-add-row input {
  flex: 1;
  min-width: 0;
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: var(--control-bg);
  font-family: inherit;
  transition: border-color 0.2s, background 0.2s;
}
.beta-add-row input:focus { border-color: var(--accent); background: var(--card-solid); }
.beta-add-row .beta-note-input { flex: 0.7; }
.beta-add-row .btn-save { flex-shrink: 0; }
.beta-empty {
  padding: 32px 16px;
  border: 1.5px dashed var(--border);
  border-radius: 12px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}
.beta-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 46vh;
  overflow-y: auto;
  padding-right: 2px;
}
.beta-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--control-bg);
}
.beta-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.beta-device-id {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.beta-item-note {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--track);
  padding: 2px 8px;
  border-radius: 999px;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.beta-item-side {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.beta-item-date { font-size: 11px; color: var(--text-muted); }

/* 渠道选项 */
.channel-options {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}
.channel-option {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  background: var(--control-bg);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.2s, background 0.2s, transform 0.2s;
}
.channel-option:hover,
.channel-option.active {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.channel-option:hover { transform: translateY(-1px); }
.channel-option strong { font-size: 14px; }
.channel-option span { font-size: 12px; color: var(--text-muted); line-height: 1.5; }

/* 拖拽上传区 */
.package-dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 150px;
  padding: 20px;
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  background: var(--control-bg);
  cursor: pointer;
  text-align: center;
  transition: border-color 0.2s, background 0.2s, transform 0.2s, box-shadow 0.2s;
}
.package-dropzone:hover,
.package-dropzone.dragging {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
  transform: translateY(-1px);
}
.package-dropzone.selected {
  border-style: solid;
  border-color: var(--accent);
}
.dropzone-icon {
  width: 42px;
  height: 42px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 20px;
  font-weight: 800;
}
.package-dropzone strong { font-size: 14px; color: var(--text); }
.package-dropzone span { font-size: 12px; color: var(--text-muted); }
.file-hidden { display: none; }
.file-info { font-size: 12px; color: var(--text-muted); }

/* 上传进度 */
.upload-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}
.progress-bar-track {
  flex: 1;
  height: 8px;
  background: var(--track);
  border-radius: 4px;
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  background: var(--accent);
  transition: width 0.3s;
}
.progress-text {
  font-size: 12px;
  color: var(--text-muted);
  min-width: 36px;
  text-align: right;
}

/* 弹窗底部 */
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 24px 20px;
  margin-top: 8px;
}
.btn-cancel {
  padding: 10px 20px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: var(--card-solid);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: var(--track); }
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-save:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.15); }
.btn-save:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-spinner {
  width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* ===== Transition 动画 ===== */
.fade-down-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(16px); }

/* 弹窗动画 */
.modal-enter-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active { transition: all 0.2s ease; }
.modal-enter-from,
.modal-leave-to { opacity: 0; }
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog {
  transform: scale(0.92) translateY(20px);
}
.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

/* 卡片列表过渡 */
.card-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.card-leave-active { transition: all 0.3s ease; }
.card-enter-from { opacity: 0; transform: translateY(20px); }
.card-leave-to { opacity: 0; transform: scale(0.9); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .page-header { flex-direction: column; }
  .header-actions { width: 100%; }
  .header-actions .btn-add,
  .header-actions .btn-beta { flex: 1; justify-content: center; }
  .beta-add-row { flex-wrap: wrap; }
  .beta-add-row .beta-note-input { flex: 1 0 100%; order: 3; }
  .page-desc { max-width: 100%; }
  .card-grid { grid-template-columns: 1fr; }
  .stats-row { flex-wrap: wrap; }
  .channel-options { grid-template-columns: 1fr; }
}
</style>
