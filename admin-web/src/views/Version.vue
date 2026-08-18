<template>
  <div class="version-wrap">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">版本管理</h2>
          <p class="page-desc">
            管理桌面端与移动端应用的版本发布。桌面端启动时自动比对版本号，低于配置版本将弹窗提示更新；APP 版本用于移动端下载更新。
          </p>
        </div>
        <button class="btn-add" @click="showAddModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增版本
        </button>
      </div>
    </Transition>

    <!-- 桌面端下载渠道弹窗 -->
    <Transition name="modal">
    <div v-if="desktopChannelModalVisible" class="modal-overlay channel-overlay">
      <div class="modal">
        <div class="modal-header">
          <span class="modal-title">选择下载渠道</span>
          <button class="modal-close" @click="closeDesktopChannelModal">&times;</button>
        </div>
        <div class="modal-body">
          <div class="channel-options">
            <button
              type="button"
              class="channel-option"
              :class="{ active: desktopChannelMode === 'upload' }"
              @click="desktopChannelMode = 'upload'"
            >
              <strong>上传安装包</strong>
              <span>安装包保存到本服务端，并自动生成下发链接。</span>
            </button>
            <button
              type="button"
              class="channel-option"
              :class="{ active: desktopChannelMode === 'link' }"
              @click="desktopChannelMode = 'link'"
            >
              <strong>填写下载链接</strong>
              <span>适合安装包已放在其它文件服务器或网盘直链。</span>
            </button>
          </div>

          <div v-if="desktopChannelMode === 'upload'" class="form-group">
            <label class="required">安装包</label>
            <div
              class="package-dropzone"
              :class="{ dragging: desktopPackageDragging, selected: !!desktopPackageDraft.fileName }"
              @click="triggerDesktopFileInput"
              @dragover.prevent="desktopPackageDragging = true"
              @dragleave.prevent="desktopPackageDragging = false"
              @drop.prevent="onDesktopPackageDrop"
            >
              <input
                ref="desktopFileInputRef"
                type="file"
                accept=".exe,.msi,.zip,.7z,.rar,.dmg,.pkg,.apk"
                class="file-hidden"
                @change="onDesktopFileChange"
              />
              <div class="dropzone-icon">⬆</div>
              <strong>{{ desktopPackageDraft.fileName ? '已选择安装包' : '点击或拖拽安装包到此处' }}</strong>
              <span>支持 EXE / MSI / ZIP / 7Z / RAR / DMG / PKG / APK</span>
            </div>
            <div v-if="desktopPackageDraft.fileName" class="file-info desktop-file-info">
              已选择：{{ desktopPackageDraft.fileName }}（{{ formatFileSize(desktopPackageDraft.fileSize) }}）
            </div>
            <div class="hint">保存配置后，服务端会自动上传并生成下载链接。</div>
          </div>

          <div v-else class="form-group">
            <label class="required">下载链接</label>
            <input v-model="desktopChannelLinkDraft" type="text" placeholder="https://..." />
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="closeDesktopChannelModal">取消</button>
          <button class="btn btn-primary" @click="confirmDesktopChannel">确定</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 桌面端在线更新配置卡片 -->
    <Transition name="fade-up" appear>
    <div class="card desktop-card">
      <div class="card-header">
        <h3 class="section-title">桌面端在线更新</h3>
        <span v-if="desktop.updated_at" class="last-saved">上次保存：{{ fmtDateTime(desktop.updated_at) }}</span>
      </div>
      <p class="section-desc">桌面端启动时自动比对版本号，低于此版本将弹窗提示更新。</p>
      <div class="form-grid">
        <div class="form-group">
          <label class="required">版本号</label>
          <input v-model="desktop.version" type="text" placeholder="如 1.2.0" />
        </div>
        <div class="form-group form-group-full">
          <label :class="{ required: desktopEnabled === 1 }">下载渠道</label>
          <button type="button" class="channel-card" @click="openDesktopChannelModal">
            <div>
              <strong>{{ desktopChannelLabel }}</strong>
              <p>{{ desktopChannelDesc }}</p>
            </div>
            <span>选择渠道</span>
          </button>
        </div>
        <div class="form-group form-group-full">
          <label>更新内容</label>
          <button type="button" class="content-editor" @click="openContentEdit('desktop-update')">
            <span>{{ desktop.updateContent ? desktop.updateContent : '点击填写更新内容' }}</span>
            <span class="expand-hint">展开编辑</span>
          </button>
        </div>
        <div class="form-group">
          <label>启用状态</label>
          <select v-model="desktopEnabled">
            <option :value="0">禁用</option>
            <option :value="1">启用</option>
          </select>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn btn-primary" :disabled="desktopSaving" @click="saveDesktop">
          {{ desktopSaving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </div>
    </Transition>

    <!-- 更新内容编辑弹窗 -->
    <Transition name="modal">
    <div v-if="contentEditVisible" class="modal-overlay">
      <div class="modal">
        <div class="modal-header">
          <span class="modal-title">编辑更新内容</span>
          <button class="modal-close" @click="closeContentEdit">&times;</button>
        </div>
        <div class="modal-body">
          <textarea v-model="contentDraft" class="content-edit-area" placeholder="请输入本次更新内容"></textarea>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="closeContentEdit">取消</button>
          <button class="btn btn-primary" @click="confirmContentEdit">确定</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- APP 版本管理 -->
    <Transition name="fade-up" appear>
    <div class="card">
      <div class="card-header">
        <h3 class="section-title">APP 版本管理</h3>
        <span class="section-desc">上传安卓安装包并配置版本信息，移动端可在此下载更新。</span>
      </div>

      <!-- 统计栏 -->
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <span class="stat-num">{{ versions.length }}</span>
          <span class="stat-label">全部</span>
        </div>
        <div class="stat-chip stat-on">
          <span class="stat-num">{{ enabledCount }}</span>
          <span class="stat-label">已启用</span>
        </div>
        <div class="stat-chip stat-off">
          <span class="stat-num">{{ disabledCount }}</span>
          <span class="stat-label">已禁用</span>
        </div>
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="state-box">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <!-- 加载失败 -->
      <div v-else-if="loadError" class="state-box state-error">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        <span>{{ loadError }}</span>
      </div>

      <!-- 空状态 -->
      <Transition name="fade-up" appear v-else-if="versions.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
          </div>
          <p class="empty-title">暂无版本数据</p>
          <p class="empty-sub">点击右上角「新增版本」上传第一个安装包</p>
        </div>
      </Transition>

      <!-- 版本卡片列表 -->
      <div v-else class="card-grid">
        <TransitionGroup name="card">
          <div
            v-for="(v, idx) in versions"
            :key="v.id"
            class="version-card"
            :class="[`status-${v.status}`, { disabled: v.status === 'disabled' }]"
            :style="{ animationDelay: `${idx * 60}ms` }"
          >
            <!-- 状态指示条 -->
            <div class="type-bar"></div>

            <!-- 卡片内容 -->
            <div class="card-body">
              <div class="card-top">
                <span class="type-badge" :class="statusClass(v.status)">
                  {{ statusLabel(v.status) }}
                </span>
                <div class="card-actions">
                  <a v-if="v.download_url" :href="v.download_url" target="_blank" rel="noopener" class="icon-btn" title="下载安装包">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                  </a>
                  <button class="icon-btn icon-btn-danger" title="删除" @click="deleteVersion(v.id)">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                  </button>
                </div>
              </div>

              <h3 class="card-title">{{ v.app_name }}</h3>
              <p class="card-version">版本号：{{ v.version_code }}</p>
              <p class="card-content">{{ v.update_content || '暂无更新内容' }}</p>

              <div class="card-footer">
                <span class="card-date">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                  {{ fmtDateTime(v.created_at) }}
                </span>
                <select class="status-select" :value="v.status" @change="changeStatus(v.id, ($event.target as HTMLSelectElement).value)">
                  <option v-for="s in statusOptions" :key="s.value" :value="s.value">{{ s.label }}</option>
                </select>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && total > 0" class="pagination">
        <button :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
      </div>
    </div>
    </Transition>

    <!-- 新增版本弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增版本</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field">
              <label class="required">软件名称</label>
              <input v-model="addForm.appName" type="text" placeholder="如 弦予·音乐" />
            </div>
            <div class="field">
              <label class="required">版本号</label>
              <input v-model="addForm.versionCode" type="text" placeholder="如 1.2.0" />
            </div>
            <div class="field">
              <label>更新内容</label>
              <button type="button" class="content-editor" @click="openContentEdit('add-update')">
                <span>{{ addForm.updateContent ? addForm.updateContent : '点击填写更新内容' }}</span>
                <span class="expand-hint">展开编辑</span>
              </button>
            </div>
            <div class="field">
              <label class="required">安装包</label>
              <div class="package-dropzone" @click="triggerApkInput">
                <input ref="apkFileInputRef" type="file" accept=".apk" class="file-hidden" @change="onFileChange" />
                <div class="dropzone-icon">⬆</div>
                <strong>{{ addForm.fileName ? '已选择安装包' : '点击选择 APK 安装包' }}</strong>
                <span v-if="addForm.fileName">已选择：{{ addForm.fileName }}（{{ formatFileSize(addForm.fileSize) }}）</span>
                <span v-else>仅支持 APK 文件</span>
              </div>
            </div>
            <div v-if="uploading" class="upload-progress">
              <div class="progress-bar-track"><div class="progress-bar-fill" :style="{ width: (uploadProgress || 0) + '%' }"></div></div>
              <span class="progress-text">{{ uploadProgress }}%</span>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAddModal">取消</button>
            <button class="btn-save" :disabled="uploading" @click="doAddVersion">
              <span v-if="uploading" class="btn-spinner"></span>
              {{ uploading ? '上传中...' : '上传安装包' }}
            </button>
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

interface DesktopVersion {
  version: string
  downloadUrl: string
  updateContent: string
  enabled: boolean
  updated_at: string
}

const statusOptions = [
  { value: 'normal', label: '正常' },
  { value: 'update', label: '更新' },
  { value: 'force_update', label: '强制更新' },
  { value: 'disabled', label: '禁用' },
  { value: 'crash', label: '闪退' },
  { value: 'group_update', label: '进群更新' },
]

const enabledCount = computed(() => versions.value.filter(v => v.status !== 'disabled').length)
const disabledCount = computed(() => versions.value.filter(v => v.status === 'disabled').length)

const statusMap: Record<string, { label: string; class: string }> = {
  normal: { label: '正常', class: 'badge-success' },
  update: { label: '更新', class: 'badge-info' },
  force_update: { label: '强制更新', class: 'badge-warning' },
  disabled: { label: '禁用', class: 'badge-error' },
  crash: { label: '闪退', class: 'badge-error' },
  group_update: { label: '进群更新', class: 'badge-warning' },
}

function statusLabel(status: string): string {
  return statusMap[status]?.label || '未知'
}
function statusClass(status: string): string {
  return statusMap[status]?.class || 'badge-error'
}

// ===== 桌面端配置 =====
const desktop = ref<DesktopVersion>({
  version: '',
  downloadUrl: '',
  updateContent: '',
  enabled: false,
  updated_at: '',
})
const desktopEnabled = ref(0)
const desktopSaving = ref(false)
const desktopChannelModalVisible = ref(false)
const desktopChannelMode = ref<'link' | 'upload'>('link')
const desktopChannelLinkDraft = ref('')
const desktopFileInputRef = ref<HTMLInputElement | null>(null)
const desktopPackageFile = ref<File | null>(null)
const desktopPackageFileDraft = ref<File | null>(null)
const desktopPackageDragging = ref(false)
const desktopPackage = ref({
  fileName: '',
  fileSize: 0,
  fileBase64: '',
})
const desktopPackageDraft = ref({
  fileName: '',
  fileSize: 0,
  fileBase64: '',
})

const desktopChannelLabel = computed(() => {
  if (desktopPackage.value.fileName) return '上传安装包'
  if (desktop.value.downloadUrl) {
    return desktop.value.downloadUrl.startsWith('/uploads/packages/') ? '服务器安装包' : '下载链接'
  }
  return '未选择下载渠道'
})

const desktopChannelDesc = computed(() => {
  if (desktopPackage.value.fileName) {
    return `已选择：${desktopPackage.value.fileName}（${formatFileSize(desktopPackage.value.fileSize)}）`
  }
  if (desktop.value.downloadUrl) return desktop.value.downloadUrl
  return desktopEnabled.value === 1 ? '启用更新时，需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
})

async function loadDesktop() {
  const res = await adminApi<DesktopVersion>('get_desktop_version')
  if (res.code === 200 && res.data) {
    desktop.value = res.data
    desktopEnabled.value = res.data.enabled ? 1 : 0
  }
}

async function saveDesktop() {
  if (!desktop.value.version.trim()) {
    showToast('请填写版本号')
    return
  }
  if (desktopEnabled.value === 1 && !desktop.value.downloadUrl.trim() && !desktopPackage.value.fileName) {
    showToast('启用更新时，请填写下载链接或选择安装包')
    return
  }
  desktopSaving.value = true
  let fileData = ''
  if (desktopPackage.value.fileName) {
    const file = desktopPackageFile.value
    if (!file) {
      desktopSaving.value = false
      showToast('请选择安装包')
      return
    }
    try {
      fileData = await readFileAsBase64(file)
    } catch {
      desktopSaving.value = false
      showToast('安装包读取失败')
      return
    }
  }
  const res = await adminApi('save_desktop_version', {
    version: desktop.value.version.trim(),
    download_url: desktop.value.downloadUrl.trim(),
    update_content: desktop.value.updateContent.trim(),
    enabled: desktopEnabled.value,
    file_name: desktopPackage.value.fileName,
    file_data: fileData,
  })
  desktopSaving.value = false
  if (res.code === 200) {
    showToast('保存成功', 'success')
    desktopPackage.value = { fileName: '', fileSize: 0, fileBase64: '' }
    desktopPackageFile.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    desktopPackageFileDraft.value = null
    loadDesktop()
  } else {
    showToast(res.msg || '保存失败')
  }
}

function openDesktopChannelModal() {
  desktopChannelMode.value = desktop.value.downloadUrl && !desktopPackage.value.fileName ? 'link' : 'upload'
  desktopChannelLinkDraft.value = desktop.value.downloadUrl
  desktopPackageDraft.value = { ...desktopPackage.value }
  desktopPackageFileDraft.value = desktopPackageFile.value
  desktopChannelModalVisible.value = true
}

function closeDesktopChannelModal() {
  desktopChannelModalVisible.value = false
}

function confirmDesktopChannel() {
  if (desktopChannelMode.value === 'link') {
    const url = desktopChannelLinkDraft.value.trim()
    if (!url) {
      showToast('请输入下载链接')
      return
    }
    desktop.value.downloadUrl = url
    desktopPackage.value = { fileName: '', fileSize: 0, fileBase64: '' }
    desktopPackageFile.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    desktopPackageFileDraft.value = null
    if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
  } else {
    if (!desktopPackageDraft.value.fileName) {
      showToast('请选择安装包')
      return
    }
    desktopPackage.value = { ...desktopPackageDraft.value }
    desktopPackageFile.value = desktopPackageFileDraft.value
    desktop.value.downloadUrl = ''
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
  desktopPackageDraft.value = {
    fileName: file.name,
    fileSize: file.size,
    fileBase64: '',
  }
  desktopChannelLinkDraft.value = ''
}

// ===== 更新内容展开编辑 =====
const contentEditVisible = ref(false)
const contentDraft = ref('')
const contentEditField = ref<'desktop-update' | 'add-update' | null>(null)

function openContentEdit(field: 'desktop-update' | 'add-update') {
  contentEditField.value = field
  contentDraft.value = field === 'desktop-update' ? desktop.value.updateContent : addForm.value.updateContent
  contentEditVisible.value = true
}

function closeContentEdit() { contentEditVisible.value = false }

function confirmContentEdit() {
  if (contentEditField.value === 'desktop-update') desktop.value.updateContent = contentDraft.value
  else if (contentEditField.value === 'add-update') addForm.value.updateContent = contentDraft.value
  contentEditVisible.value = false
}

// ===== APP 版本列表 =====
const versions = ref<AppVersion[]>([])
const loading = ref(true)
const loadError = ref('')
const page = ref(1)
const pageSize = 15
const total = ref(0)
const totalPages = ref(0)

const pageNumbers = computed(() => {
  const max = 7
  const pages: number[] = []
  if (totalPages.value <= max) {
    for (let i = 1; i <= totalPages.value; i++) pages.push(i)
  } else {
    let start = Math.max(1, page.value - 3)
    let end = Math.min(totalPages.value, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<{ total: number; total_pages: number; list: AppVersion[] }>('list_versions', {
    page: page.value,
    page_size: pageSize,
  })
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

async function changeStatus(id: number, status: string) {
  const res = await adminApi('change_version_status', { id, status })
  if (res.code === 200) {
    showToast('操作成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function deleteVersion(id: number) {
  const ok = await webConfirm('确认删除该版本？', { title: '删除版本', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_version', { id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 新增版本 =====
const addModalVisible = ref(false)
const uploading = ref(false)
const uploadProgress = ref<number | null>(null)
const apkFileInputRef = ref<HTMLInputElement | null>(null)
const addForm = ref({
  appName: '弦予·音乐',
  versionCode: '',
  updateContent: '',
  fileName: '',
  fileSize: 0,
  fileBase64: '',
})

function showAddModal() {
  addForm.value = {
    appName: '弦予·音乐',
    versionCode: '',
    updateContent: '',
    fileName: '',
    fileSize: 0,
    fileBase64: '',
  }
  uploadProgress.value = null
  addModalVisible.value = true
}

function closeAddModal() {
  if (uploading.value) return
  addModalVisible.value = false
}

function triggerApkInput() { apkFileInputRef.value?.click() }

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files || input.files.length === 0) {
    addForm.value.fileName = ''
    addForm.value.fileSize = 0
    return
  }
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

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      // 去掉 data:...;base64, 前缀
      const base64 = result.split(',')[1] || ''
      resolve(base64)
    }
    reader.onerror = () => reject(new Error('文件读取失败'))
    reader.readAsDataURL(file)
  })
}

async function doAddVersion() {
  if (!addForm.value.appName.trim() || !addForm.value.versionCode.trim()) {
    showToast('请填写软件名称和版本号')
    return
  }
  const fileInput = apkFileInputRef.value
  if (!fileInput || !fileInput.files || fileInput.files.length === 0) {
    showToast('请选择安装包')
    return
  }

  uploading.value = true
  uploadProgress.value = 0

  // 模拟读取进度
  uploadProgress.value = 10
  const file = fileInput.files[0]
  let base64 = ''
  try {
    base64 = await readFileAsBase64(file)
  } catch (e) {
    uploading.value = false
    uploadProgress.value = null
    showToast('文件读取失败')
    return
  }
  uploadProgress.value = 50

  const res = await adminApi('add_version', {
    app_name: addForm.value.appName.trim(),
    version_code: addForm.value.versionCode.trim(),
    update_content: addForm.value.updateContent.trim(),
    file_data: base64,
  })
  uploadProgress.value = 100

  setTimeout(() => {
    uploadProgress.value = null
  }, 500)

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
.version-wrap {
  max-width: 1320px;
  margin: 0 auto;
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
  max-width: 640px;
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

/* ===== 统计栏 ===== */
.stats-row {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
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

/* ===== 版本卡片网格 ===== */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}
.version-card {
  display: flex;
  background: var(--white);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.version-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
.version-card.disabled { opacity: 0.6; }
.version-card.disabled:hover { opacity: 0.85; }

@keyframes cardEnter {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 状态指示条 */
.type-bar {
  width: 4px;
  flex-shrink: 0;
}
.status-normal .type-bar { background: #10b981; }
.status-update .type-bar { background: #3b82f6; }
.status-force_update .type-bar { background: #f59e0b; }
.status-disabled .type-bar { background: #9ca3af; }
.status-crash .type-bar { background: #ef4444; }
.status-group_update .type-bar { background: #8b5cf6; }

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
.type-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
}
.badge-success { background: #ecfdf5; color: #10b981; }
.badge-info { background: #eff6ff; color: #3b82f6; }
.badge-warning { background: #fffbeb; color: #f59e0b; }
.badge-error { background: #fef2f2; color: #ef4444; }
.badge-purple { background: #f5f3ff; color: #8b5cf6; }

.card-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.card-version {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
  margin: 0;
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
.status-select {
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  font-size: 12px;
  background: var(--white);
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s;
}
.status-select:focus { border-color: var(--accent); }

/* ===== 弹窗（公告管理页同款） ===== */
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
.modal-dialog {
  background: var(--white);
  border-radius: 18px;
  width: 100%;
  max-width: 520px;
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
.field input {
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: #fafafa;
  font-family: inherit;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus {
  border-color: var(--accent);
  background: var(--white);
}
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
.modal-leave-to .modal-dialog { transform: scale(0.92) translateY(20px); }
.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog { transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1); }

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
}

/* 桌面端配置卡片 */
.desktop-card {
  margin-bottom: 16px;
}
.section-title {
  font-size: 16px;
  font-weight: 700;
  margin: 0 0 8px 0;
}
.section-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 16px 0;
  line-height: 1.6;
}
.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 24px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form-group-full {
  grid-column: 1 / -1;
}
.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.form-group input,
.form-group textarea,
.form-group select {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  background: var(--white);
  font-family: inherit;
  resize: vertical;
}
.form-group input:focus,
.form-group textarea:focus,
.form-group select:focus {
  border-color: var(--accent);
}
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
  box-shadow: var(--shadow-soft);
  transform: translateY(-1px);
}
.channel-card strong {
  display: block;
  font-size: 14px;
  margin-bottom: 4px;
}
.channel-card p {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
  word-break: break-all;
}
.channel-card > span {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--accent);
  font-weight: 700;
}
.form-actions {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}
.last-saved {
  font-size: 12px;
  color: var(--text-muted);
}
.content-editor {
  width: 100%;
  min-height: 72px;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--white);
  cursor: pointer;
  text-align: left;
  color: var(--text);
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
}
.content-editor:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-soft);
  transform: translateY(-1px);
}
.content-editor > span:first-child {
  font-size: 14px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  flex: 1;
}
.expand-hint {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
}
.content-edit-area {
  width: 100%;
  min-height: 260px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--white);
  color: var(--text);
  resize: vertical;
  line-height: 1.6;
  box-sizing: border-box;
}
.content-edit-area:focus { border-color: var(--accent); }

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.card-header .section-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
  text-align: right;
}
.plus-icon {
  display: none;
}

/* 表格（已迁移至卡片，保留原样式作兼容） */
.ellipsis {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.nowrap { white-space: nowrap; }
.nowrap-time { white-space: nowrap; font-size: 12px; }
.download-link {
  color: var(--purple);
  text-decoration: none;
  font-size: 12px;
}
.download-link:hover { text-decoration: underline; }
.status-select {
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border);
  font-size: 12px;
  background: var(--white);
  margin-right: 4px;
  outline: none;
  cursor: pointer;
}
.status-select:focus { border-color: var(--accent); }

/* 分页 */
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

/* 弹窗 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: center;
}
.modal-overlay.channel-overlay {
  z-index: 10010;
}
.modal {
  background: var(--white);
  border-radius: 12px;
  width: 90%;
  max-width: 620px;
  max-height: 85vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #eee;
}
.modal-title { font-weight: 700; font-size: 16px; }
.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
  line-height: 1;
}
.modal-close:hover { color: #333; }
.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.modal-body .form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.modal-body .form-group input,
.modal-body .form-group textarea {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  background: var(--white);
  font-family: inherit;
  resize: vertical;
}
/* 弹窗内分区 */
.modal-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
  margin: 0 0 4px 0;
}
.section-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
}
.modal-section-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
  margin: 0 0 12px 0;
}
.modal-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.modal-section .form-actions {
  margin-top: 10px;
}
.modal-body .form-group input:focus,
.modal-body .form-group textarea:focus {
  border-color: var(--accent);
}
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
.channel-option:hover {
  transform: translateY(-1px);
}
.channel-option strong {
  font-size: 14px;
}
.channel-option span {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
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
  box-shadow: var(--shadow-soft);
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
.package-dropzone strong {
  font-size: 14px;
  color: var(--text);
}
.package-dropzone span {
  font-size: 12px;
  color: var(--text-muted);
}
.file-hidden {
  display: none;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid #eee;
}

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
.file-info {
  font-size: 12px;
  color: var(--text-muted);
}
.hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

@media (max-width: 768px) {
  .form-grid { grid-template-columns: 1fr; }
  .channel-card {
    align-items: flex-start;
    flex-direction: column;
  }
  .channel-options {
    grid-template-columns: 1fr;
  }
}

/* 弹窗淡进淡出 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.92) translateY(20px);
}

/* ===== 页面进入动效 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
