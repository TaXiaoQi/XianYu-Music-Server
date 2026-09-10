<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="ver-header">
      <div class="ver-header-info">
        <h2 class="ver-title">版本管理</h2>
        <p class="ver-desc">按客户端平台管理在线更新配置，启用的版本只对所选平台生效。</p>
      </div>
      <div class="ver-header-actions">
        <button class="mobile-btn beta-btn" @click="openBetaModal">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
          内测名单<span v-if="betaList.length" class="beta-count">{{ betaList.length }}</span>
        </button>
        <button class="mobile-btn primary" @click="openDesktopModal()">+ 新增版本</button>
      </div>
    </div>

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
    <div class="mobile-grid" style="grid-template-columns: repeat(3, minmax(0, 1fr));">
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-total">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </span>
          <strong>{{ platformList.length }}</strong>
        </div>
        <span class="stat-label">全部</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-on">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </span>
          <strong>{{ enabledCount }}</strong>
        </div>
        <span class="stat-label">已启用</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-off">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </span>
          <strong>{{ platformList.length - enabledCount }}</strong>
        </div>
        <span class="stat-label">已禁用</span>
      </div>
    </div>

    <!-- 在线更新（按当前平台过滤） -->
    <div class="ver-section">
      <div class="ver-section-label">
        <span class="ver-section-dot" :class="`dot-${platformFilter}`"></span>
        <h3>{{ currentPlatformLabel }}在线更新</h3>
      </div>

      <div v-if="desktopLoading" class="ver-empty">加载中...</div>

      <div v-else-if="platformList.length === 0" class="ver-desktop-empty">
        <p>暂未配置{{ currentPlatformLabel }}更新版本</p>
        <button class="mobile-btn outline" @click="openDesktopModal()">+ 新增配置</button>
      </div>

      <div v-else class="ver-card-list">
        <div
          v-for="item in platformList"
          :key="`${platformOf(item)}-${item.version}`"
          class="ver-card"
          :class="{ disabled: !item.enabled }"
        >
          <div class="ver-card-bar" :class="`bar-${platformOf(item)}`"></div>
          <div class="ver-card-body">
            <div class="ver-card-top">
              <div class="ver-badges">
                <span class="ver-badge" :class="`badge-${platformOf(item)}`">{{ platformLabelOf(item) }}</span>
                <span v-if="channelOf(item) === 'beta'" class="ver-badge badge-beta">测试版</span>
              </div>
              <label class="ver-toggle" :title="item.enabled ? '点击禁用' : '点击启用'">
                <input type="checkbox" :checked="item.enabled" @change="toggleDesktop($event, item)" />
                <span class="ver-toggle-slider"></span>
              </label>
            </div>
            <div class="ver-card-title">v{{ item.version || '-' }}</div>
            <div class="ver-card-content">{{ item.updateContent || '无更新说明' }}</div>
            <div v-if="item.downloadUrl" class="ver-card-link" @click="openUrl(item.downloadUrl)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
              <span>下载安装包</span>
            </div>
            <div class="ver-card-footer">
              <span class="ver-card-date">{{ fmtDateTime(item.updated_at) || '-' }}</span>
              <div class="ver-card-actions">
                <button class="ver-icon-btn" title="编辑" @click="openDesktopModal(item)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="ver-icon-btn ver-icon-danger" title="删除" @click="deleteDesktop(item)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 版本配置弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="desktopModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ desktopEditingVersion ? `编辑${platformLabelKey(desktopDraftPlatform)}配置` : `新增${platformLabelKey(desktopDraftPlatform)}配置` }}</h3>
            <button class="modal-close" @click="closeDesktopModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="ver-form">
              <div class="field">
                <span class="required">客户端平台</span>
                <div class="enable-row platform-row">
                  <button
                    v-for="p in PLATFORMS"
                    :key="p.key"
                    type="button"
                    class="enable-btn platform-btn"
                    :class="{ on: desktopDraftPlatform === p.key, locked: !!desktopEditingVersion }"
                    :disabled="!!desktopEditingVersion"
                    @click="desktopDraftPlatform = p.key"
                  >{{ p.label }}</button>
                </div>
              </div>
              <div class="field">
                <span class="required">更新渠道</span>
                <div class="enable-row channel-row">
                  <button type="button" class="enable-btn channel-stable" :class="{ on: desktopDraftChannel === 'stable' }" @click="desktopDraftChannel = 'stable'">正式版</button>
                  <button type="button" class="enable-btn channel-beta" :class="{ on: desktopDraftChannel === 'beta' }" @click="desktopDraftChannel = 'beta'">测试版</button>
                </div>
                <p v-if="desktopDraftChannel === 'beta'" class="field-hint">测试版仅对「内测名单」中的设备ID下发，普通用户检测更新与官网下载均不受影响。</p>
              </div>
              <label class="field">
                <span class="required">版本号</span>
                <input v-if="desktopDraftChannel === 'stable'" v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
                <div v-else class="ver-combo">
                  <input v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
                  <span class="ver-combo-sep">-beta-</span>
                  <input v-model="desktopDraftBetaNum" type="text" inputmode="numeric" placeholder="1" class="ver-combo-num" />
                </div>
              </label>
              <label class="field">
                <span>下载渠道</span>
                <button type="button" class="channel-card" @click="openDesktopChannelModal">
                  <div>
                    <strong>{{ desktopChannelLabel }}</strong>
                    <p>{{ desktopChannelDesc }}</p>
                  </div>
                  <span>选择</span>
                </button>
              </label>
              <label class="field">
                <span>更新内容</span>
                <textarea v-model="desktopDraft.updateContent" rows="18" placeholder="本次更新内容"></textarea>
              </label>
              <label class="field">
                <span>启用状态</span>
                <div class="enable-row">
                  <button type="button" class="enable-btn" :class="{ on: desktopDraftEnabled }" @click="desktopDraftEnabled = true">启用</button>
                  <button type="button" class="enable-btn" :class="{ on: !desktopDraftEnabled }" @click="desktopDraftEnabled = false">禁用</button>
                </div>
              </label>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeDesktopModal">取消</button>
            <button class="modal-btn save" :disabled="desktopSaving" @click="saveDesktop">{{ desktopSaving ? '保存中...' : '保存配置' }}</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 桌面端下载渠道弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="desktopChannelModalVisible" class="modal-backdrop channel-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>选择下载渠道</h3>
            <button class="modal-close" @click="closeDesktopChannelModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
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
              <span class="required">安装包</span>
              <div class="package-dropzone" :class="{ selected: !!desktopPackageDraft.fileName }" @click="triggerDesktopFileInput">
                <input ref="desktopFileInputRef" type="file" accept=".exe,.msi,.zip,.7z,.rar,.dmg,.pkg,.apk" class="file-hidden" @change="onDesktopFileChange" />
                <div class="dropzone-icon">⬆</div>
                <strong>{{ desktopPackageDraft.fileName ? '已选择安装包' : '点击选择安装包' }}</strong>
                <span>支持 EXE / MSI / ZIP / 7Z / RAR / DMG / PKG / APK</span>
              </div>
              <div v-if="desktopPackageDraft.fileName" class="file-info">已选择：{{ desktopPackageDraft.fileName }}（{{ formatFileSize(desktopPackageDraft.fileSize) }}）</div>
            </div>
            <div v-else class="field">
              <span class="required">下载链接</span>
              <input v-model="desktopChannelLinkDraft" type="text" placeholder="https://..." />
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeDesktopChannelModal">取消</button>
            <button class="modal-btn save" @click="confirmDesktopChannel">确定</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 内测名单弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="betaModalVisible" class="modal-backdrop beta-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>内测名单</h3>
            <button class="modal-close" @click="betaModalVisible = false">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <p class="field-hint">名单内的设备ID可收到「测试版」渠道的更新下发；正式版与官网下载对所有用户不变。点击设备可查看关联帐号与设备信息。</p>
            <div class="beta-add-row">
              <input v-model="betaDeviceIdDraft" type="text" placeholder="输入设备ID" />
              <input v-model="betaNoteDraft" type="text" placeholder="备注（可选）" class="beta-note-input" />
            </div>
            <button class="mobile-btn primary beta-add-btn" :disabled="betaSaving" @click="addBetaTester">{{ betaSaving ? '添加中...' : '+ 添加设备' }}</button>
            <div v-if="betaLoading" class="ver-empty">加载中...</div>
            <div v-else-if="betaList.length === 0" class="ver-desktop-empty"><p>暂无内测设备</p></div>
            <div v-else class="beta-list">
              <div v-for="t in betaList" :key="t.id" class="beta-item clickable" @click="openBetaDetail(t)">
                <div class="beta-item-main">
                  <template v-if="t.note">
                    <span class="beta-item-note" :title="t.note">{{ t.note }}</span>
                    <span class="beta-device-id beta-device-id-sub" :title="t.device_id">{{ t.device_id }}</span>
                  </template>
                  <span v-else class="beta-device-id" :title="t.device_id">{{ t.device_id }}</span>
                </div>
                <div class="beta-item-actions">
                  <button class="ver-icon-btn" title="查看详情" @click.stop="openBetaDetail(t)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="7"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
                  </button>
                  <button class="ver-icon-btn ver-icon-danger" title="移除" @click.stop="removeBetaTester(t)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="betaModalVisible = false">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 内测设备详情弹窗（关联帐号 / 厂商 / 型号 / 系统版本） -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="betaDetailVisible" class="modal-backdrop beta-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>内测设备详情</h3>
            <button class="modal-close" @click="closeBetaDetail">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body beta-detail-body">
            <div v-if="betaDetailLoading" class="ver-empty">加载中...</div>
            <template v-else-if="betaDetail">
              <div class="beta-detail-row full">
                <span class="beta-detail-label">设备ID</span>
                <code class="beta-detail-device-id">{{ betaDetail.tester?.device_id || '-' }}</code>
              </div>
              <div class="beta-detail-row full">
                <span class="beta-detail-label">备注</span>
                <div class="beta-note-edit">
                  <input v-model="betaNoteEditDraft" type="text" maxlength="255" placeholder="给这台设备起个好认的备注名" @keyup.enter="saveBetaNote" />
                  <button class="beta-note-save" :disabled="betaNoteSaving" @click="saveBetaNote">{{ betaNoteSaving ? '保存中' : '保存' }}</button>
                </div>
              </div>
              <div class="beta-detail-row">
                <span class="beta-detail-label">加入时间</span>
                <span class="beta-detail-value">{{ betaDetail.tester?.created_at || '-' }}</span>
              </div>

              <div class="beta-detail-section">设备信息</div>
              <template v-if="hasBetaDetailDevice">
                <div class="beta-detail-row">
                  <span class="beta-detail-label">厂商</span>
                  <span class="beta-detail-value">{{ betaDetail.device?.brand || '-' }}</span>
                </div>
                <div class="beta-detail-row">
                  <span class="beta-detail-label">型号</span>
                  <span class="beta-detail-value">{{ betaDetail.device?.model || '-' }}</span>
                </div>
                <div class="beta-detail-row">
                  <span class="beta-detail-label">系统版本</span>
                  <span class="beta-detail-value">{{ formatOsVersion(betaDetail.device?.os_version) || '-' }}</span>
                </div>
                <div v-if="betaDetail.device?.app_version" class="beta-detail-row">
                  <span class="beta-detail-label">应用版本</span>
                  <span class="beta-detail-value">{{ betaDetail.device.app_version }}</span>
                </div>
              </template>
              <p v-else class="beta-detail-empty">暂无该设备的上报数据（设备未反馈或未启动过）</p>

              <div class="beta-detail-section">关联帐号</div>
              <template v-if="betaDetail.accounts?.length">
                <div v-for="(a, i) in betaDetail.accounts" :key="i" class="beta-detail-row">
                  <span class="beta-detail-label">帐号{{ betaDetail.accounts.length > 1 ? i + 1 : '' }}</span>
                  <span class="beta-detail-value">
                    {{ a.nickname || '匿名用户' }}（{{ a.ciyuanxi_id }}）
                    <em v-if="a.source" class="beta-detail-src">{{ a.source }}</em>
                  </span>
                </div>
              </template>
              <p v-else class="beta-detail-empty">未找到该设备关联的帐号</p>
            </template>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeBetaDetail">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm, removeBackdropBlur } from '@/utils/mobileDialog'
import { fmtDateTime } from '@/utils/time'
import { formatOsVersion } from '@/utils/osVersion'

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

function platformLabelOf(item: any): string {
  return platformLabelKey(platformOf(item))
}

function formatFileSize(bytes: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function openUrl(url: string) {
  if (url) window.open(url, '_blank')
}

// ===== 版本配置 =====
const desktopList = ref<any[]>([])
const desktopLoading = ref(true)
const platformFilter = ref<PlatformKey>('desktop')
const currentPlatformLabel = computed(() => platformLabelKey(platformFilter.value))
const platformList = computed(() => desktopList.value.filter(v => platformOf(v) === platformFilter.value))
const enabledCount = computed(() => platformList.value.filter(v => v.enabled).length)

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
const desktopDraft = ref({ version: '', updateContent: '', downloadUrl: '' })
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
const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })

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
  return desktopDraftEnabled.value ? '启用更新时需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
})

/** 测试版组合版本号：主版本 + -beta- + beta号（如 1.2.0-beta-3） */
const composedVersion = computed(() => {
  if (desktopDraftChannel.value !== 'beta') return desktopDraft.value.version.trim()
  const main = desktopDraft.value.version.trim()
  const num = parseInt(desktopDraftBetaNum.value, 10) || 0
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
  desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
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
    try { fileData = await readFileAsBase64(desktopPackageFile.value) }
    catch { desktopSaving.value = false; showToast('安装包读取失败'); return }
  }
  const res = await adminApi('save_desktop_version', {
    platform: desktopDraftPlatform.value,
    channel: desktopDraftChannel.value,
    version,
    download_url: desktopDraft.value.downloadUrl?.trim() || '',
    update_content: desktopDraft.value.updateContent?.trim() || '',
    enabled: desktopDraftEnabled.value ? 1 : 0,
    file_name: desktopPackageFile.value?.name || '',
    file_data: fileData,
  })
  desktopSaving.value = false
  if (res.code === 200) {
    showToast('保存成功', 'success')
    desktopModalVisible.value = false
    loadDesktop()
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
  const ok = await mobileConfirm(`确认删除${platformLabelOf(item)} v${item.version} 的更新配置？`, { title: '删除配置', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_desktop_version', { platform: platformOf(item), version: item.version })
  if (res.code === 200) { showToast('删除成功', 'success'); loadDesktop() }
  else showToast(res.msg || '删除失败')
}

function openDesktopChannelModal() {
  desktopChannelMode.value = desktopDraft.value.downloadUrl && !desktopPackageFile.value ? 'link' : 'upload'
  desktopChannelLinkDraft.value = desktopDraft.value.downloadUrl || ''
  desktopPackageDraft.value = desktopPackageFile.value
    ? { fileName: desktopPackageFile.value.name, fileSize: desktopPackageFile.value.size, fileBase64: '' }
    : { fileName: '', fileSize: 0, fileBase64: '' }
  desktopChannelModalVisible.value = true
}

function closeDesktopChannelModal() { desktopChannelModalVisible.value = false }

function confirmDesktopChannel() {
  if (desktopChannelMode.value === 'link') {
    const url = desktopChannelLinkDraft.value.trim()
    if (!url) { showToast('请输入下载链接'); return }
    desktopDraft.value.downloadUrl = url
    desktopPackageFile.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
  } else {
    if (!desktopPackageDraft.value.fileName) { showToast('请选择安装包'); return }
    desktopPackageFile.value = desktopPackageDraft.value as any
    desktopDraft.value.downloadUrl = ''
    desktopChannelLinkDraft.value = ''
  }
  desktopChannelModalVisible.value = false
}

function onDesktopFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files || input.files.length === 0) {
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    return
  }
  const file = input.files[0]
  const ext = file.name.split('.').pop()?.toLowerCase() || ''
  const allowed = ['exe', 'msi', 'zip', '7z', 'rar', 'dmg', 'pkg', 'apk']
  if (!allowed.includes(ext)) {
    showToast('不支持该安装包格式')
    input.value = ''
    return
  }
  desktopPackageFile.value = file
  desktopPackageDraft.value = { fileName: file.name, fileSize: file.size, fileBase64: '' }
  desktopChannelLinkDraft.value = ''
}

function triggerDesktopFileInput() { desktopFileInputRef.value?.click() }

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

async function addBetaTester() {
  const deviceId = betaDeviceIdDraft.value.trim()
  if (!deviceId) { showToast('请输入设备ID'); return }
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
  const ok = await mobileConfirm(`确认将设备 ${t.device_id} 移出内测名单？`, { title: '移除内测设备', confirmText: '确认移除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_beta_tester', { device_id: t.device_id })
  if (res.code === 200) { showToast('已移除', 'success'); loadBeta() }
  else showToast(res.msg || '移除失败')
}

// 内测设备详情（关联帐号 / 厂商 / 型号 / 系统版本）+ 备注编辑
const betaDetailVisible = ref(false)
const betaDetailLoading = ref(false)
const betaDetail = ref<any>(null)
const betaNoteEditDraft = ref('')
const betaNoteSaving = ref(false)
const hasBetaDetailDevice = computed(() => {
  const d = betaDetail.value?.device || {}
  return !!(d.brand || d.model || d.os_version || d.architecture || d.machine_name)
})

async function openBetaDetail(t: any) {
  betaDetailVisible.value = true
  betaDetail.value = null
  betaDetailLoading.value = true
  const res = await adminApi<any>('get_beta_tester_detail', { device_id: t.device_id })
  betaDetailLoading.value = false
  if (res.code === 200 && res.data) {
    betaDetail.value = res.data
    betaNoteEditDraft.value = res.data.tester?.note || ''
  } else {
    showToast(res.msg || '加载详情失败')
    betaDetailVisible.value = false
  }
}

async function saveBetaNote() {
  const tester = betaDetail.value?.tester
  if (!tester) return
  const note = betaNoteEditDraft.value.trim()
  betaNoteSaving.value = true
  const res = await adminApi('update_beta_tester_note', { device_id: tester.device_id, note })
  betaNoteSaving.value = false
  if (res.code === 200) {
    tester.note = note
    showToast('已更新设备备注', 'success')
    loadBeta()
  } else {
    showToast(res.msg || '保存失败')
  }
}

function closeBetaDetail() {
  betaDetailVisible.value = false
  betaDetail.value = null
  betaNoteEditDraft.value = ''
}

onMounted(() => {
  loadDesktop()
})
</script>

<style scoped>
.ver-header { display: flex; flex-direction: column; gap: 12px; }
.ver-header-info { min-width: 0; }
.ver-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.ver-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.ver-header .mobile-btn { align-self: flex-start; padding: 10px 20px; }
.ver-header-actions { display: flex; gap: 8px; align-self: flex-start; }
.ver-header-actions .mobile-btn { align-self: auto; }

/* 内测名单按钮 */
.beta-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1.5px solid rgba(245, 158, 11, 0.55);
  background: rgba(245, 158, 11, 0.1);
  color: #d97706;
}
.beta-count {
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  background: #f59e0b;
  color: #fff;
  font-size: 10px;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* 平台切换 */
.platform-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 14px;
  background: var(--control-bg, var(--card-solid));
  border: 1px solid var(--border);
  border-radius: 12px;
}
.platform-tab {
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-light, var(--text-muted));
  padding: 7px 18px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.platform-tab.active {
  background: var(--card-solid);
  color: var(--accent);
}

/* 统计栏 */
.ver-stats { display: flex; gap: 10px; margin-bottom: 20px; }
.ver-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  padding: 10px 8px;
  border-radius: 12px;
  background: var(--card, var(--white));
  border: 1px solid var(--border);
}
.ver-stat-num { font-size: 18px; font-weight: 850; color: var(--text); }
.ver-stat-num.green { color: #10b981; }
.ver-stat-num.gray { color: #9ca3af; }
.ver-stat-label { font-size: 11px; color: var(--text-muted); }

/* 区块 */
.ver-section { margin-bottom: 20px; }
.ver-section-label { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; }
.ver-section-label h3 { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); }
.ver-section-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.dot-desktop { background: #3b82f6; }
.dot-mobile { background: #10b981; }
.dot-watch { background: #8b5cf6; }
.dot-app { background: #10b981; }

/* 空状态 */
.ver-empty {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
.ver-desktop-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 20px;
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  color: var(--text-muted);
  font-size: 13px;
}
.mobile-btn.outline {
  border: 1.5px dashed var(--accent);
  background: transparent;
  color: var(--accent);
  padding: 8px 16px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

/* 卡片列表 */
.ver-card-list { display: flex; flex-direction: column; gap: 12px; }

/* 卡片 */
.ver-card {
  display: flex;
  background: var(--card, var(--white));
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) backwards;
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.25s ease;
}
.ver-card:active { transform: scale(0.98); }
.ver-card.disabled { opacity: 0.55; }
@keyframes cardIn { from { opacity: 0; transform: translateY(12px); } to { opacity: 1; transform: translateY(0); } }

.ver-card-bar { width: 4px; flex-shrink: 0; }
.bar-desktop { background: #3b82f6; }
.bar-mobile { background: #10b981; }
.bar-watch { background: #8b5cf6; }
.bar-normal { background: #10b981; }
.bar-update { background: #3b82f6; }
.bar-force { background: #f59e0b; }
.bar-disabled { background: #9ca3af; }
.bar-crash { background: #ef4444; }
.bar-group { background: #8b5cf6; }

.ver-card-body { flex: 1; padding: 14px 16px; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
.ver-card-top { display: flex; justify-content: space-between; align-items: center; }

.ver-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
}
.badge-desktop { background: #eff6ff; color: #3b82f6; }
.badge-mobile { background: #ecfdf5; color: #10b981; }
.badge-watch { background: #f5f3ff; color: #8b5cf6; }
.badge-normal { background: #ecfdf5; color: #10b981; }
.badge-update { background: #eff6ff; color: #3b82f6; }
.badge-force { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.badge-disabled { background: #f3f4f6; color: #6b7280; }
.badge-crash { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.badge-group { background: #f5f3ff; color: #8b5cf6; }
.badge-beta { background: rgba(245, 158, 11, 0.14); color: #d97706; }
.ver-badges { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }

/* 渠道选择（正式版/测试版） */
.channel-stable.on { background: #10b981; border-color: #10b981; color: #fff; }
.channel-beta.on { background: #f59e0b; border-color: #f59e0b; color: #fff; }
.field-hint { font-size: 11px; color: var(--text-muted); line-height: 1.5; margin: 0; }

/* 测试版组合版本号 */
.ver-combo { display: flex; align-items: center; gap: 6px; }
.ver-combo input {
  flex: 1; min-width: 0;
  border: 1px solid var(--border); border-radius: 14px;
  padding: 11px 12px; font-size: 14px; font-family: inherit;
  outline: none; background: var(--control-bg); color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  box-sizing: border-box;
}
.ver-combo input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.ver-combo-sep {
  flex-shrink: 0; font-size: 12px; font-weight: 700; color: #d97706;
  background: rgba(245, 158, 11, 0.12); padding: 5px 8px; border-radius: 8px; user-select: none;
}
.ver-combo-num { max-width: 76px !important; }

/* 内测名单弹窗 */
.beta-backdrop { z-index: 11000; }
.beta-add-row { display: flex; gap: 8px; }
.beta-add-row input {
  flex: 1; min-width: 0;
  border: 1px solid var(--border); border-radius: 12px;
  padding: 11px 12px; font-size: 13px; font-family: inherit;
  outline: none; background: var(--control-bg); color: var(--text);
  transition: border-color 0.18s;
}
.beta-add-row input:focus { border-color: var(--accent); }
.beta-add-row .beta-note-input { flex: 0.7; }
.beta-add-btn { width: 100%; justify-content: center; }
.beta-list { display: flex; flex-direction: column; gap: 8px; max-height: 42vh; overflow-y: auto; }
.beta-item {
  display: flex; justify-content: space-between; align-items: center; gap: 10px;
  padding: 10px 12px; border: 1px solid var(--border); border-radius: 12px;
  background: var(--control-bg);
}
.beta-item-main { display: flex; align-items: center; gap: 8px; min-width: 0; flex: 1; }
.beta-device-id {
  font-size: 12px; font-weight: 600; color: var(--text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.beta-item-note {
  flex-shrink: 0; font-size: 11px; color: var(--text-muted);
  background: var(--border); padding: 2px 8px; border-radius: 999px;
  max-width: 110px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.beta-item-main .beta-item-note {
  flex-shrink: 0; font-weight: 600; color: var(--text); font-size: 12px;
  max-width: 150px; background: var(--track);
}
.beta-device-id-sub {
  font-weight: 400; font-size: 10px; color: var(--text-muted); max-width: 90px;
}
.beta-note-edit { display: flex; gap: 8px; }
.beta-note-edit input {
  flex: 1; min-width: 0; padding: 9px 12px; border-radius: 10px;
  border: 1px solid var(--border); background: var(--control-bg); color: var(--text);
  font-size: 13px; outline: none;
}
.beta-note-edit input:focus { border-color: var(--accent); }
.beta-note-save {
  padding: 0 16px; border: none; border-radius: 10px; background: #EC4141; color: #fff;
  font-size: 13px; font-weight: 700; cursor: pointer; flex-shrink: 0;
}
.beta-note-save:disabled { opacity: 0.55; }
.beta-item.clickable { cursor: pointer; }
.beta-item.clickable:active { background: var(--border); }
.beta-item-actions { display: flex; gap: 2px; flex-shrink: 0; }

/* 内测设备详情弹窗 */
.beta-detail-body { display: flex; flex-direction: column; gap: 10px; overflow-y: auto; }
.beta-detail-row { display: flex; gap: 12px; font-size: 13px; line-height: 1.6; }
.beta-detail-row.full { flex-direction: column; gap: 4px; }
.beta-detail-label { flex: 0 0 64px; color: var(--text-muted); }
.beta-detail-value { color: var(--text); min-width: 0; word-break: break-all; }
.beta-detail-device-id {
  font-size: 12px; color: var(--text); background: var(--border);
  padding: 3px 8px; border-radius: 6px; word-break: break-all;
}
.beta-detail-section {
  font-size: 12px; font-weight: 700; color: var(--text-muted);
  border-top: 1px solid var(--border); padding-top: 10px; margin-top: 4px;
}
.beta-detail-empty { font-size: 12px; color: var(--text-muted); }
.beta-detail-src { font-style: normal; font-size: 11px; color: var(--text-muted); margin-left: 6px; }

/* Toggle */
.ver-toggle { position: relative; display: inline-block; width: 38px; height: 22px; cursor: pointer; flex-shrink: 0; }
.ver-toggle input { opacity: 0; width: 0; height: 0; }
.ver-toggle-slider {
  position: absolute; inset: 0; background: #d1d5db; border-radius: 22px;
  transition: background 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.ver-toggle-slider::before {
  content: ''; position: absolute; width: 16px; height: 16px; left: 3px; top: 3px;
  background: var(--card-solid); border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.ver-toggle input:checked + .ver-toggle-slider { background: #10b981; }
.ver-toggle input:checked + .ver-toggle-slider::before { transform: translateX(16px); }

.ver-card-title { font-size: 14px; font-weight: 750; color: var(--text); line-height: 1.4; }
.ver-code {
  font-size: 11px; font-weight: 700; color: var(--accent);
  background: var(--accent-soft); padding: 1px 7px; border-radius: 999px; margin-left: 4px;
}
.ver-card-content {
  font-size: 12px; color: var(--text-light); line-height: 1.5; margin: 0;
  display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
}
.ver-card-link {
  display: flex; align-items: center; gap: 4px; font-size: 12px; color: #6366f1; cursor: pointer;
}
.ver-card-footer {
  display: flex; justify-content: space-between; align-items: center;
  margin-top: auto; padding-top: 8px; border-top: 1px solid #f5f5f5;
}
.ver-card-date { font-size: 11px; color: var(--text-muted); }
.ver-card-actions { display: flex; gap: 4px; }
.ver-icon-btn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border-radius: 8px; border: none;
  background: transparent; color: var(--text-muted); cursor: pointer;
  transition: all 0.18s;
}
.ver-icon-btn:active { background: #f5f5f5; color: var(--text); }
.ver-icon-danger:active { background: rgba(236, 65, 65, 0.12); color: #ef4444; }

/* 表单 */
.ver-form { display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.field input, .field textarea {
  border: 1px solid var(--border); border-radius: 14px;
  padding: 11px 12px; font-size: 14px; font-family: inherit;
  outline: none; background: var(--control-bg); color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%; box-sizing: border-box; resize: vertical;
}
.field input:focus, .field textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

.channel-card {
  width: 100%; display: flex; justify-content: space-between; align-items: center;
  gap: 12px; padding: 12px 14px; border: 1px solid var(--border);
  border-radius: 14px; background: var(--control-bg); color: var(--text);
  cursor: pointer; text-align: left; transition: border-color 0.18s;
}
.channel-card:active { border-color: var(--accent); }
.channel-card strong { display: block; font-size: 13px; margin-bottom: 3px; }
.channel-card p { margin: 0; font-size: 11px; color: var(--text-muted); word-break: break-all; }
.channel-card > span { flex-shrink: 0; font-size: 12px; color: var(--accent); font-weight: 700; }

.enable-row { display: flex; gap: 8px; }
.enable-btn {
  flex: 1; padding: 10px; border-radius: 12px; border: 1px solid var(--border);
  background: var(--control-bg); color: var(--text-muted);
  font-size: 13px; font-weight: 700; cursor: pointer; transition: all 0.18s;
}
.enable-btn.on { background: #EC4141; border-color: #EC4141; color: #fff; }

/* 分页 */
.pagination { display: flex; justify-content: center; gap: 6px; flex-wrap: wrap; margin-top: 16px; }
.page-btn {
  min-width: 34px; padding: 7px 11px; border: 1px solid var(--border);
  background: var(--card, var(--white)); color: var(--text-light);
  border-radius: 10px; font-size: 12px; font-weight: 700; cursor: pointer;
  transition: all 0.15s;
}
.page-btn.active { background: #EC4141; color: #fff; border-color: #EC4141; }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* 弹窗 */
.modal-backdrop {
  position: fixed; inset: 0; z-index: 10000;
  display: flex; align-items: center; justify-content: center;
  padding: 32px 24px;
  background: rgba(15, 23, 42, 0.38);
  backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
}
.modal-backdrop.channel-backdrop { z-index: 11000; }
.modal-dialog {
  width: 100%; max-width: 420px; max-height: 88vh;
  border-radius: 22px; background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden; display: flex; flex-direction: column;
}
.modal-head { display: flex; align-items: center; justify-content: space-between; padding: 18px 20px 0; }
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 850; color: var(--text); }
.modal-close { border: none; background: transparent; color: var(--text-muted); cursor: pointer; padding: 4px; border-radius: 8px; display: flex; }
.modal-close:active { background: var(--control-bg); color: var(--text); }
.modal-body { padding: 14px 20px; display: flex; flex-direction: column; gap: 14px; overflow-y: auto; }
.modal-foot { display: flex; gap: 10px; padding: 14px 20px 18px; border-top: 1px solid var(--border); }
.modal-btn { flex: 1; padding: 11px; border-radius: 12px; font-size: 14px; font-weight: 750; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: all 0.18s; }
.modal-btn.cancel { border: 1px solid var(--border); background: transparent; color: var(--text-muted); }
.modal-btn.cancel:active { background: var(--control-bg); }
.modal-btn.save { border: none; background: #EC4141; color: #fff; }
.modal-btn.save:disabled { opacity: 0.55; }

.channel-options { display: flex; flex-direction: column; gap: 8px; }
.channel-option {
  display: flex; flex-direction: column; gap: 5px;
  padding: 12px 14px; border: 1.5px solid var(--border); border-radius: 12px;
  background: var(--control-bg); color: var(--text);
  cursor: pointer; text-align: left; transition: all 0.18s;
}
.channel-option.active { border-color: var(--accent); background: var(--accent-soft); }
.channel-option strong { font-size: 13px; }
.channel-option span { font-size: 11px; color: var(--text-muted); line-height: 1.5; }

.package-dropzone {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 7px; min-height: 130px; padding: 16px;
  border: 1.5px dashed var(--border); border-radius: 14px;
  background: var(--control-bg); cursor: pointer; text-align: center;
  transition: all 0.18s;
}
.package-dropzone.selected { border-style: solid; border-color: var(--accent); }
.dropzone-icon {
  width: 38px; height: 38px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  background: var(--accent-soft); color: var(--accent); font-size: 18px; font-weight: 800;
}
.package-dropzone strong { font-size: 13px; color: var(--text); }
.package-dropzone span { font-size: 11px; color: var(--text-muted); }
.file-hidden { display: none; }
.file-info { font-size: 11px; color: var(--text-muted); }

.upload-progress { display: flex; align-items: center; gap: 8px; }
.progress-bar-track { flex: 1; height: 6px; background: var(--border); border-radius: 3px; overflow: hidden; }
.progress-bar-fill { height: 100%; background: var(--accent); transition: width 0.3s; }
.progress-text { font-size: 11px; color: var(--text-muted); min-width: 36px; text-align: right; }

/* 过渡动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.24s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn { from { opacity: 0; transform: scale(0.94); } to { opacity: 1; transform: scale(1); } }
@keyframes modalOut { from { opacity: 1; transform: scale(1); } to { opacity: 0; transform: scale(0.96); } }
</style>
