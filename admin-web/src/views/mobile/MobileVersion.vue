<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="ver-header">
      <div class="ver-header-info">
        <h2 class="ver-title">版本管理</h2>
        <p class="ver-desc">管理桌面端在线更新配置与安卓 APP 版本。新增或编辑版本后，卡片列表实时刷新。</p>
      </div>
      <button class="mobile-btn primary" @click="openAddModal">+ 新增版本</button>
    </div>

    <!-- 统计栏 -->
    <div class="ver-stats">
      <div class="ver-stat">
        <span class="ver-stat-num">{{ total }}</span>
        <span class="ver-stat-label">全部</span>
      </div>
      <div class="ver-stat">
        <span class="ver-stat-num green">{{ enabledCount }}</span>
        <span class="ver-stat-label">已启用</span>
      </div>
      <div class="ver-stat">
        <span class="ver-stat-num gray">{{ disabledCount }}</span>
        <span class="ver-stat-label">已禁用</span>
      </div>
    </div>

    <!-- 桌面端在线更新 -->
    <div class="ver-section">
      <div class="ver-section-label">
        <span class="ver-section-dot dot-desktop"></span>
        <h3>桌面端在线更新</h3>
      </div>

      <div v-if="desktopLoading" class="ver-empty">加载中...</div>

      <div v-else-if="!desktop.version && !desktop.downloadUrl" class="ver-desktop-empty">
        <p>暂未配置桌面端更新版本</p>
        <button class="mobile-btn outline" @click="openDesktopModal">+ 新增配置</button>
      </div>

      <div v-else class="ver-card" :class="{ disabled: !desktop.enabled }">
        <div class="ver-card-bar bar-desktop"></div>
        <div class="ver-card-body">
          <div class="ver-card-top">
            <span class="ver-badge badge-desktop">桌面端</span>
            <label class="ver-toggle" :title="desktop.enabled ? '点击禁用' : '点击启用'">
              <input type="checkbox" :checked="desktop.enabled" @change="toggleDesktop($event)" />
              <span class="ver-toggle-slider"></span>
            </label>
          </div>
          <div class="ver-card-title">v{{ desktop.version || '-' }}</div>
          <div class="ver-card-content">{{ desktop.updateContent || '无更新说明' }}</div>
          <div v-if="desktop.downloadUrl" class="ver-card-link" @click="openUrl(desktop.downloadUrl)">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
            <span>下载安装包</span>
          </div>
          <div class="ver-card-footer">
            <span class="ver-card-date">{{ fmtDateTime(desktop.updated_at) || '-' }}</span>
            <div class="ver-card-actions">
              <button class="ver-icon-btn" title="编辑" @click="openDesktopModal">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
              </button>
              <button class="ver-icon-btn ver-icon-danger" title="删除" @click="deleteDesktop">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- APP 版本卡片列表 -->
    <div class="ver-section">
      <div class="ver-section-label">
        <span class="ver-section-dot dot-app"></span>
        <h3>APP 版本管理</h3>
      </div>

      <div v-if="loading" class="ver-empty">加载中...</div>
      <div v-else-if="loadError" class="ver-empty">{{ loadError }}</div>
      <div v-else-if="versions.length === 0" class="ver-empty">
        暂无 APP 版本，点击「新增版本」上传
      </div>
      <div v-else class="ver-card-list">
        <div
          v-for="(v, idx) in versions"
          :key="v.id"
          class="ver-card"
          :class="[{ disabled: v.status === 'disabled' }]"
          :style="{ animationDelay: `${idx * 40}ms` }"
        >
          <div class="ver-card-bar" :class="statusBarClass(v.status)"></div>
          <div class="ver-card-body">
            <div class="ver-card-top">
              <span class="ver-badge" :class="statusBadgeClass(v.status)">{{ statusLabel(v.status) }}</span>
              <label class="ver-toggle" :title="v.status === 'disabled' ? '点击启用' : '点击禁用'">
                <input type="checkbox" :checked="v.status !== 'disabled'" @change="toggleVersion(v, ($event.target as HTMLInputElement).checked)" />
                <span class="ver-toggle-slider"></span>
              </label>
            </div>
            <div class="ver-card-title">{{ v.app_name || '-' }} <span class="ver-code">{{ v.version_code || '-' }}</span></div>
            <div class="ver-card-content">{{ v.update_content || '无更新说明' }}</div>
            <div v-if="v.download_url" class="ver-card-link" @click="openUrl(v.download_url)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
              <span>下载安装包</span>
            </div>
            <div class="ver-card-footer">
              <span class="ver-card-date">{{ fmtDateTime(v.created_at) || '-' }}</span>
              <div class="ver-card-actions">
                <button class="ver-icon-btn" title="编辑" @click="openEditModal(v)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="ver-icon-btn ver-icon-danger" title="删除" @click="deleteVersion(v)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && total > 0" class="pagination">
        <button class="page-btn" :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          class="page-btn"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button class="page-btn" :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
      </div>
    </div>

    <!-- 桌面端配置弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="desktopModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ desktop.version ? '编辑桌面端配置' : '新增桌面端配置' }}</h3>
            <button class="modal-close" @click="closeDesktopModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="ver-form">
              <label class="field">
                <span class="required">版本号</span>
                <input v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
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
                <textarea v-model="desktopDraft.updateContent" rows="9" placeholder="本次更新内容"></textarea>
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

    <!-- 新增/编辑 APP 版本弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="addModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ editingId ? '编辑版本' : '新增 APP 版本' }}</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="ver-form">
              <label class="field">
                <span class="required">软件名称</span>
                <input v-model="addForm.appName" type="text" placeholder="如 弦予·音乐" />
              </label>
              <label class="field">
                <span class="required">版本号</span>
                <input v-model="addForm.versionCode" type="text" placeholder="如 1.2.0" />
              </label>
              <label class="field">
                <span>更新内容</span>
                <textarea v-model="addForm.updateContent" rows="9" placeholder="本次更新内容"></textarea>
              </label>
              <div v-if="!editingId" class="field">
                <span class="required">安装包</span>
                <div class="package-dropzone" :class="{ selected: !!addForm.fileName }" @click="triggerApkFileInput">
                  <input ref="apkFileInputRef" type="file" accept=".apk" class="file-hidden" @change="onFileChange" />
                  <div class="dropzone-icon">⬆</div>
                  <strong>{{ addForm.fileName ? '已选择安装包' : '点击选择 APK' }}</strong>
                  <span>仅支持 APK 格式</span>
                </div>
                <div v-if="addForm.fileName" class="file-info">已选择：{{ addForm.fileName }}（{{ formatFileSize(addForm.fileSize) }}）</div>
              </div>
              <div v-if="uploading" class="upload-progress">
                <div class="progress-bar-track"><div class="progress-bar-fill" :style="{ width: uploadProgress + '%' }"></div></div>
                <span class="progress-text">{{ uploadProgress }}%</span>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeAddModal">取消</button>
            <button class="modal-btn save" :disabled="uploading" @click="saveVersion">{{ uploading ? '保存中...' : (editingId ? '保存修改' : '上传安装包') }}</button>
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

interface AppVersion {
  id: number
  app_name: string
  version_code: string
  download_url: string
  update_content: string
  status: string
  file_size: number
  created_at: string
  [key: string]: any
}

const statusMap: Record<string, { label: string; bar: string; badge: string }> = {
  normal: { label: '正常', bar: 'bar-normal', badge: 'badge-normal' },
  update: { label: '更新', bar: 'bar-update', badge: 'badge-update' },
  force_update: { label: '强制更新', bar: 'bar-force', badge: 'badge-force' },
  disabled: { label: '禁用', bar: 'bar-disabled', badge: 'badge-disabled' },
  crash: { label: '闪退', bar: 'bar-crash', badge: 'badge-crash' },
  group_update: { label: '进群更新', bar: 'bar-group', badge: 'badge-group' },
}

function statusLabel(s: string) { return statusMap[s]?.label || '未知' }
function statusBarClass(s: string) { return statusMap[s]?.bar || 'bar-normal' }
function statusBadgeClass(s: string) { return statusMap[s]?.badge || 'badge-normal' }
function formatSize(n: number) { return !n ? '-' : n < 1024 * 1024 ? `${(n / 1024).toFixed(1)}KB` : `${(n / 1024 / 1024).toFixed(1)}MB` }
function formatFileSize(bytes: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function openUrl(url: string) {
  if (url) window.open(url, '_blank')
}

// ===== 桌面端配置 =====
const desktop = ref<any>({ version: '', downloadUrl: '', updateContent: '', enabled: false, updated_at: '' })
const desktopLoading = ref(true)

async function loadDesktop() {
  desktopLoading.value = true
  const res = await adminApi<any>('get_desktop_version')
  if (res.code === 200 && res.data) {
    desktop.value = res.data
  }
  desktopLoading.value = false
}

// 桌面端弹窗
const desktopModalVisible = ref(false)
const desktopDraft = ref({ version: '', updateContent: '', downloadUrl: '' })
const desktopDraftEnabled = ref(false)
const desktopSaving = ref(false)
const desktopChannelModalVisible = ref(false)
const desktopChannelMode = ref<'link' | 'upload'>('link')
const desktopChannelLinkDraft = ref('')
const desktopFileInputRef = ref<HTMLInputElement | null>(null)
const desktopPackageFile = ref<File | null>(null)
const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })

const desktopChannelLabel = computed(() => {
  if (desktopPackageFile.value?.name) return '上传安装包'
  const url = desktopDraft.value.downloadUrl || desktop.value.downloadUrl
  if (url) return url.startsWith('/uploads/packages/') ? '服务器安装包' : '下载链接'
  return '未选择下载渠道'
})

const desktopChannelDesc = computed(() => {
  if (desktopPackageFile.value?.name) return `已选择：${desktopPackageFile.value.name}（${formatFileSize(desktopPackageFile.value.size)}）`
  const url = desktopDraft.value.downloadUrl || desktop.value.downloadUrl
  if (url) return url
  return desktopDraftEnabled.value ? '启用更新时需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
})

function openDesktopModal() {
  desktopDraft.value = {
    version: desktop.value.version || '',
    updateContent: desktop.value.updateContent || '',
    downloadUrl: desktop.value.downloadUrl || '',
  }
  desktopDraftEnabled.value = desktop.value.enabled || false
  desktopPackageFile.value = null
  desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
  desktopModalVisible.value = true
}

function closeDesktopModal() {
  if (desktopSaving.value) return
  desktopModalVisible.value = false
}

async function saveDesktop() {
  if (!desktopDraft.value.version?.trim()) { showToast('请填写版本号'); return }
  const hasPackage = !!desktopPackageFile.value
  const hasUrl = !!desktopDraft.value.downloadUrl?.trim() || !!desktop.value.downloadUrl?.trim()
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
    version: desktopDraft.value.version.trim(),
    download_url: desktopDraft.value.downloadUrl?.trim() || desktop.value.downloadUrl?.trim() || '',
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

async function toggleDesktop(e: Event) {
  const enabled = (e.target as HTMLInputElement).checked
  const res = await adminApi('save_desktop_version', {
    version: desktop.value.version,
    download_url: desktop.value.downloadUrl,
    update_content: desktop.value.updateContent,
    enabled: enabled ? 1 : 0,
    file_name: '',
    file_data: '',
  })
  if (res.code === 200) {
    showToast(enabled ? '已启用' : '已禁用', 'success')
    loadDesktop()
  } else {
    showToast(res.msg || '操作失败')
    loadDesktop()
  }
}

async function deleteDesktop() {
  const ok = await mobileConfirm('确认删除桌面端更新配置？', { title: '删除配置', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('save_desktop_version', {
    version: '', download_url: '', update_content: '', enabled: 0, file_name: '', file_data: '',
  })
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

// ===== APP 版本列表 =====
const versions = ref<AppVersion[]>([])
const loading = ref(true)
const loadError = ref('')
const page = ref(1)
const pageSize = 15
const total = ref(0)
const totalPages = ref(0)

const enabledCount = computed(() => versions.value.filter(v => v.status !== 'disabled').length)
const disabledCount = computed(() => versions.value.filter(v => v.status === 'disabled').length)

const pageNumbers = computed(() => {
  const max = 5
  const pages: number[] = []
  if (totalPages.value <= max) {
    for (let i = 1; i <= totalPages.value; i++) pages.push(i)
  } else {
    let start = Math.max(1, page.value - 2)
    let end = Math.min(totalPages.value, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<{ total: number; total_pages: number; list: AppVersion[] }>('list_versions', { page: page.value, page_size: pageSize })
  if (res.code === 200 && res.data) {
    versions.value = res.data.list || []
    total.value = res.data.total
    totalPages.value = res.data.total_pages
  } else {
    loadError.value = res.msg || '加载失败'
    versions.value = []
  }
  loading.value = false
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadList()
}

async function toggleVersion(v: AppVersion, enabled: boolean) {
  const newStatus = enabled ? 'normal' : 'disabled'
  const res = await adminApi('change_version_status', { id: v.id, status: newStatus })
  if (res.code === 200) { v.status = newStatus; showToast(enabled ? '已启用' : '已禁用', 'success') }
  else showToast(res.msg || '操作失败')
}

async function deleteVersion(v: AppVersion) {
  const ok = await mobileConfirm(`确认删除版本 ${v.version_code || v.id}？`, { title: '删除版本', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_version', { id: v.id })
  if (res.code === 200) { showToast('删除成功', 'success'); loadList() }
  else showToast(res.msg || '删除失败')
}

// ===== 新增/编辑弹窗 =====
const addModalVisible = ref(false)
const editingId = ref(0)
const uploading = ref(false)
const uploadProgress = ref(0)
const apkFileInputRef = ref<HTMLInputElement | null>(null)
const addForm = ref({ appName: '弦予·音乐', versionCode: '', updateContent: '', fileName: '', fileSize: 0 })

function openAddModal() {
  editingId.value = 0
  addForm.value = { appName: '弦予·音乐', versionCode: '', updateContent: '', fileName: '', fileSize: 0 }
  uploadProgress.value = 0
  addModalVisible.value = true
}

function openEditModal(v: AppVersion) {
  editingId.value = v.id
  addForm.value = {
    appName: v.app_name || '',
    versionCode: v.version_code || '',
    updateContent: v.update_content || '',
    fileName: '',
    fileSize: 0,
  }
  uploadProgress.value = 0
  addModalVisible.value = true
}

function closeAddModal() {
  if (uploading.value) return
  addModalVisible.value = false
}

function triggerApkFileInput() { apkFileInputRef.value?.click() }

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files || input.files.length === 0) return
  const file = input.files[0]
  const ext = file.name.split('.').pop()?.toLowerCase()
  if (ext !== 'apk') {
    showToast('只允许上传 APK 文件')
    input.value = ''
    return
  }
  addForm.value.fileName = file.name
  addForm.value.fileSize = file.size
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
    reader.onerror = () => reject(new Error('文件读取失败'))
    reader.readAsDataURL(file)
  })
}

async function saveVersion() {
  if (!addForm.value.appName.trim() || !addForm.value.versionCode.trim()) {
    showToast('请填写软件名称和版本号')
    return
  }

  if (editingId.value) {
    uploading.value = true
    const res = await adminApi('update_version', {
      id: editingId.value,
      app_name: addForm.value.appName.trim(),
      version_code: addForm.value.versionCode.trim(),
      update_content: addForm.value.updateContent.trim(),
    })
    uploading.value = false
    if (res.code === 200) {
      showToast('修改成功', 'success')
      closeAddModal()
      loadList()
    } else {
      showToast(res.msg || '修改失败')
    }
    return
  }

  const fileInput = apkFileInputRef.value
  if (!fileInput || !fileInput.files || fileInput.files.length === 0) {
    showToast('请选择安装包')
    return
  }

  uploading.value = true
  uploadProgress.value = 10
  const file = fileInput.files[0]
  let base64 = ''
  try { base64 = await readFileAsBase64(file) }
  catch { uploading.value = false; uploadProgress.value = 0; showToast('文件读取失败'); return }
  uploadProgress.value = 50
  const res = await adminApi('add_version', {
    app_name: addForm.value.appName.trim(),
    version_code: addForm.value.versionCode.trim(),
    update_content: addForm.value.updateContent.trim(),
    file_data: base64,
  })
  uploadProgress.value = 100
  uploading.value = false
  if (res.code === 200) {
    showToast('上传成功', 'success')
    closeAddModal()
    loadList()
  } else {
    showToast(res.msg || '上传失败')
  }
}

onMounted(() => {
  loadDesktop()
  loadList()
})
</script>

<style scoped>
.ver-header { display: flex; flex-direction: column; gap: 12px; }
.ver-header-info { min-width: 0; }
.ver-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.ver-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.ver-header .mobile-btn { align-self: flex-start; padding: 10px 20px; }

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
.badge-normal { background: #ecfdf5; color: #10b981; }
.badge-update { background: #eff6ff; color: #3b82f6; }
.badge-force { background: #fffbeb; color: #f59e0b; }
.badge-disabled { background: #f3f4f6; color: #6b7280; }
.badge-crash { background: #fef2f2; color: #ef4444; }
.badge-group { background: #f5f3ff; color: #8b5cf6; }

/* Toggle */
.ver-toggle { position: relative; display: inline-block; width: 38px; height: 22px; cursor: pointer; flex-shrink: 0; }
.ver-toggle input { opacity: 0; width: 0; height: 0; }
.ver-toggle-slider {
  position: absolute; inset: 0; background: #d1d5db; border-radius: 22px;
  transition: background 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.ver-toggle-slider::before {
  content: ''; position: absolute; width: 16px; height: 16px; left: 3px; top: 3px;
  background: var(--white); border-radius: 50%;
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
.ver-icon-danger:active { background: #fef2f2; color: #ef4444; }

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
