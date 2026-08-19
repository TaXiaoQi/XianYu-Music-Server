<template>
  <div class="version-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">版本管理</h2>
          <p class="page-desc">
            管理桌面端在线更新配置。新增或编辑版本后，卡片实时刷新。启用的版本将对客户端生效。
          </p>
        </div>
        <button class="btn-add" @click="openDesktopModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增版本
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ hasConfig ? 1 : 0 }}</span>
            <span class="stat-label">全部</span>
          </div>
        </div>
        <div class="stat-chip stat-on">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ desktop.enabled ? 1 : 0 }}</span>
            <span class="stat-label">已启用</span>
          </div>
        </div>
        <div class="stat-chip stat-off">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ hasConfig && !desktop.enabled ? 1 : 0 }}</span>
            <span class="stat-label">已禁用</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 桌面端在线更新卡片 -->
    <Transition name="fade-up" appear>
      <div class="desktop-section">
        <div class="section-label">
          <span class="section-dot dot-desktop"></span>
          <h3>桌面端在线更新</h3>
          <span class="section-hint">桌面端启动时自动比对版本号，低于此版本将弹窗提示更新</span>
        </div>

        <div v-if="desktopLoading" class="state-box"><div class="spinner"></div><span>加载中...</span></div>

        <div v-else-if="!desktop.version && !desktop.downloadUrl" class="desktop-empty">
          <p>暂未配置桌面端更新版本</p>
          <button class="btn-add-small" @click="openDesktopModal">+ 新增配置</button>
        </div>

        <div v-else class="desktop-card" :class="{ disabled: !desktop.enabled }">
          <div class="type-bar bar-desktop"></div>
          <div class="card-body">
            <div class="card-top">
              <span class="type-badge badge-desktop">桌面端</span>
              <label class="toggle-switch" :title="desktop.enabled ? '点击禁用' : '点击启用'">
                <input type="checkbox" :checked="desktop.enabled" @change="toggleDesktop($event)" />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <h3 class="card-title">v{{ desktop.version || '-' }}</h3>
            <p class="card-content">{{ desktop.updateContent || '无更新说明' }}</p>
            <div v-if="desktop.downloadUrl" class="card-link">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
              <a :href="desktop.downloadUrl" target="_blank">{{ desktop.downloadUrl }}</a>
            </div>
            <div class="card-footer">
              <span class="card-date">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                {{ desktop.updated_at || '-' }}
              </span>
              <div class="card-actions">
                <button class="icon-btn" title="编辑" @click="openDesktopModal">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="icon-btn icon-btn-danger" title="删除" @click="deleteDesktop">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 桌面端配置弹窗 -->
    <Transition name="modal">
      <div v-if="desktopModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ desktop.version ? '编辑桌面端配置' : '新增桌面端配置' }}</h3>
            <button class="modal-close" @click="closeDesktopModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field">
              <label class="required">版本号</label>
              <input v-model="desktopDraft.version" type="text" placeholder="如 1.2.0" />
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

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

// ===== 桌面端配置 =====
const desktop = ref<any>({ version: '', downloadUrl: '', updateContent: '', enabled: false, updated_at: '' })
const desktopLoading = ref(true)
const hasConfig = computed(() => !!(desktop.value.version || desktop.value.downloadUrl))

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
const desktopDraft = ref<{ version: string; updateContent: string; downloadUrl: string }>({ version: '', updateContent: '', downloadUrl: '' })
const desktopDraftEnabled = ref(false)
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
  if (desktopDraft.value.downloadUrl || desktop.value.downloadUrl) {
    const url = desktopDraft.value.downloadUrl || desktop.value.downloadUrl
    return url.startsWith('/uploads/packages/') ? '服务器安装包' : '下载链接'
  }
  return '未选择下载渠道'
})

const desktopChannelDesc = computed(() => {
  if (desktopPackageFile.value?.name) return `已选择：${desktopPackageFile.value.name}（${formatFileSize(desktopPackageFile.value.size)}）`
  const url = desktopDraft.value.downloadUrl || desktop.value.downloadUrl
  if (url) return url
  return desktopDraftEnabled.value ? '启用更新时，需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
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
  if (!desktopDraft.value.version.trim()) {
    showToast('请填写版本号')
    return
  }
  const hasPackage = !!desktopPackageFile.value
  const hasUrl = !!(desktopDraft.value as any).downloadUrl?.trim() || !!desktop.value.downloadUrl?.trim()
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
    version: desktopDraft.value.version.trim(),
    download_url: desktopDraft.value.downloadUrl?.trim() || desktop.value.downloadUrl?.trim() || '',
    update_content: desktopDraft.value.updateContent.trim(),
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
  const ok = await webConfirm('确认删除桌面端更新配置？', { title: '删除配置', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('save_desktop_version', {
    version: '',
    download_url: '',
    update_content: '',
    enabled: 0,
    file_name: '',
    file_data: '',
  })
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

onMounted(() => {
  loadDesktop()
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
  background: var(--white);
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
.stat-total .stat-icon { background: #f0f0f0; color: #1a1a1a; }
.stat-on .stat-icon { background: #f0fdf4; color: #16a34a; }
.stat-off .stat-icon { background: #fef2f2; color: #dc2626; }
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

/* ===== 统计栏 ===== */
.stats-row {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}
.stat-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border-radius: 10px;
  background: var(--white);
  border: 1px solid var(--border);
  transition: all 0.2s;
}
.stat-chip:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
.stat-num { font-size: 18px; font-weight: 800; }
.stat-label { font-size: 12px; color: var(--text-muted); }
.stat-total .stat-num { color: var(--text); }
.stat-on .stat-num { color: #10b981; }
.stat-off .stat-num { color: #9ca3af; }
.stat-desktop .stat-num { color: #3b82f6; }

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
.dot-app { background: #10b981; }

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
  background: var(--white);
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
  background: var(--white);
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
.badge-desktop { background: #eff6ff; color: #3b82f6; }
.badge-normal { background: #ecfdf5; color: #10b981; }
.badge-update { background: #eff6ff; color: #3b82f6; }
.badge-force { background: #fffbeb; color: #f59e0b; }
.badge-disabled { background: #f3f4f6; color: #6b7280; }
.badge-crash { background: #fef2f2; color: #ef4444; }
.badge-group { background: #f5f3ff; color: #8b5cf6; }

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
  background: #d1d5db;
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
  background: var(--white);
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
  background: #f5f5f5;
  color: var(--text);
}
.icon-btn-danger:hover {
  background: #fef2f2;
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
  background: var(--white);
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
  background: var(--white);
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
.modal-close:hover { background: #f5f5f5; color: var(--text); }

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
  background: #fafafa;
  font-family: inherit;
  resize: vertical;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus,
.field textarea:focus {
  border-color: var(--accent);
  background: var(--white);
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
  background: var(--control-bg, #fafafa);
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
  background: #fafafa;
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
  background: #d1d5db;
  transition: background 0.2s;
}
.pick-enable.active { background: #ecfdf5; color: #10b981; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: #f9fafb; color: #6b7280; }
.pick-disable.active .pick-dot { background: #6b7280; }

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
  background: #fafafa;
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
  background: #fafafa;
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
  background: #e0e0e8;
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
  background: var(--white);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }
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
  .page-desc { max-width: 100%; }
  .card-grid { grid-template-columns: 1fr; }
  .stats-row { flex-wrap: wrap; }
  .channel-options { grid-template-columns: 1fr; }
}
</style>
