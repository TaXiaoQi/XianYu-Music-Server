<template>
  <div class="version-wrap">
    <!-- 桌面端在线更新配置 -->
    <div class="card desktop-card">
      <h3 class="section-title">桌面端在线更新</h3>
      <p class="section-desc">
        桌面端启动时自动比对版本号，低于此版本将弹窗提示更新。仅对桌面端生效，不影响下方安卓 APP 版本管理。
      </p>
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
          <textarea v-model="desktop.updateContent" rows="3" placeholder="本次更新内容"></textarea>
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
        <span v-if="desktop.updated_at" class="last-saved">上次保存：{{ desktop.updated_at }}</span>
      </div>
    </div>

    <!-- 桌面端下载渠道弹窗 -->
    <Transition name="modal">
    <div v-if="desktopChannelModalVisible" class="modal-overlay" @click.self="closeDesktopChannelModal">
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

    <!-- APP 版本管理 -->
    <div class="card">
      <div class="card-header">
        <h3 class="section-title">版本管理</h3>
        <button class="btn btn-primary" @click="showAddModal">
          <span class="plus-icon">+</span> 新增版本
        </button>
      </div>

      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="versions.length === 0" class="empty">暂无版本数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>版本ID</th>
              <th>软件名称</th>
              <th>版本号</th>
              <th>状态</th>
              <th>更新内容</th>
              <th>上传时间</th>
              <th>安装包</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="v in versions" :key="v.id">
              <td>{{ v.id }}</td>
              <td>{{ v.app_name || '-' }}</td>
              <td>{{ v.version_code || '-' }}</td>
              <td>
                <span class="badge" :class="statusClass(v.status)">{{ statusLabel(v.status) }}</span>
              </td>
              <td class="ellipsis" :title="v.update_content">{{ v.update_content || '-' }}</td>
              <td class="nowrap-time">{{ v.created_at }}</td>
              <td>
                <a v-if="v.download_url" :href="v.download_url" target="_blank" class="download-link">下载</a>
                <span v-else>-</span>
              </td>
              <td class="nowrap">
                <select
                  :value="v.status"
                  class="status-select"
                  @change="changeStatus(v.id, ($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="s in statusOptions" :key="s.value" :value="s.value">{{ s.label }}</option>
                </select>
                <button class="btn btn-danger btn-sm" @click="deleteVersion(v.id)">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
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

    <!-- 新增版本弹窗 -->
    <Transition name="modal">
    <div v-if="addModalVisible" class="modal-overlay" @click.self="closeAddModal">
      <div class="modal">
        <div class="modal-header">
          <span class="modal-title">新增版本</span>
          <button class="modal-close" @click="closeAddModal">&times;</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label class="required">软件名称</label>
            <input v-model="addForm.appName" type="text" placeholder="弦予·音乐" />
          </div>
          <div class="form-group">
            <label class="required">版本号</label>
            <input v-model="addForm.versionCode" type="text" placeholder="1.0.0" />
          </div>
          <div class="form-group">
            <label>更新内容</label>
            <textarea v-model="addForm.updateContent" rows="3" placeholder="请输入更新内容"></textarea>
          </div>
          <div class="form-group">
            <label class="required">安装包</label>
            <input type="file" accept=".apk" @change="onFileChange" />
          </div>
          <div v-if="uploadProgress !== null" class="upload-progress">
            <div class="progress-bar-track">
              <div class="progress-bar-fill" :style="{ width: uploadProgress + '%' }"></div>
            </div>
            <span class="progress-text">{{ uploadProgress }}%</span>
          </div>
          <div v-if="addForm.fileName" class="file-info">
            已选择：{{ addForm.fileName }}（{{ formatFileSize(addForm.fileSize) }}）
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="closeAddModal">取消</button>
          <button class="btn btn-primary" :disabled="uploading" @click="doAddVersion">
            {{ uploading ? '上传中...' : '上传' }}
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
  if (!confirm('确认删除该版本？')) return
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
  const fileInput = document.querySelector('.modal input[type="file"]') as HTMLInputElement
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

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.plus-icon {
  font-size: 16px;
  font-weight: 700;
  margin-right: 2px;
}

/* 表格 */
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
.modal {
  background: var(--white);
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
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
</style>
