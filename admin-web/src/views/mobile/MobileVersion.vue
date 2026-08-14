<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="ver-header">
      <div class="ver-header-info">
        <h2 class="ver-title">版本管理</h2>
        <p class="ver-desc">配置桌面端在线更新版本号与下载渠道，并管理安卓 APP 的各版本安装包与更新状态。</p>
      </div>
      <button class="mobile-btn primary" @click="showAddModal">+ 新增版本</button>
    </div>

    <!-- 版本管理列表 -->
    <section class="mobile-card list-card">
      <div class="list-head">
        <div>
          <h3 class="list-title">APP 版本管理</h3>
          <span class="list-sub">共 {{ total }} 个版本</span>
        </div>
      </div>

      <div v-if="loading" class="mobile-empty">加载中...</div>
      <div v-else-if="loadError" class="mobile-empty">{{ loadError }}</div>
      <div v-else-if="versions.length === 0" class="mobile-empty">暂无版本数据</div>
      <div v-else class="mobile-list">
        <div v-for="(v, idx) in versions" :key="v.id" class="mobile-item" :style="{ animationDelay: `${idx * 40}ms` }">
          <div class="mobile-item-head">
            <div class="ver-left">
              <div class="ver-badge">{{ String(v.id).slice(-2) }}</div>
              <div>
                <div class="mobile-item-title">{{ v.app_name || '-' }} <span class="ver-code">{{ v.version_code || '-' }}</span></div>
                <div class="mobile-item-sub">{{ v.created_at }} · {{ formatSize(v.file_size) }}</div>
              </div>
            </div>
            <span class="mobile-badge" :class="statusClass(v.status)">{{ statusLabel(v.status) }}</span>
          </div>
          <div class="ver-content">{{ v.update_content || '无更新说明' }}</div>
          <div class="mobile-actions">
            <a v-if="v.download_url" class="mobile-btn" :href="v.download_url" target="_blank">下载</a>
            <span v-else class="mobile-btn disabled-btn">无安装包</span>
            <select class="mobile-select status-select" :value="v.status" @change="changeStatus(v, ($event.target as HTMLSelectElement).value)">
              <option v-for="s in statusOptions" :key="s.value" :value="s.value">{{ s.label }}</option>
            </select>
            <button class="mobile-btn danger" @click="deleteVersion(v)">删除</button>
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
    </section>

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

    <!-- 新增版本弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="addModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增版本</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <!-- 桌面端在线更新 -->
            <div class="modal-section">
              <div class="section-head">
                <div class="section-icon">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="4" width="20" height="13" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>
                </div>
                <div>
                  <h3 class="section-name">桌面端在线更新</h3>
                  <p class="section-desc">桌面端启动时自动比对版本号，低于此版本将弹窗提示更新。</p>
                </div>
              </div>

              <div class="ver-form">
                <label class="field">
                  <span class="required">版本号</span>
                  <input v-model="desktop.version" type="text" placeholder="如 1.2.0" />
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
                  <textarea v-model="desktop.updateContent" rows="3" placeholder="本次更新内容"></textarea>
                </label>
                <label class="field">
                  <span>启用状态</span>
                  <div class="enable-row">
                    <button type="button" class="enable-btn" :class="{ on: desktopEnabled === 1 }" @click="desktopEnabled = 1">启用</button>
                    <button type="button" class="enable-btn" :class="{ on: desktopEnabled === 0 }" @click="desktopEnabled = 0">禁用</button>
                  </div>
                </label>
                <button class="mobile-btn primary" :disabled="desktopSaving" @click="saveDesktop">{{ desktopSaving ? '保存中...' : '保存配置' }}</button>
                <span v-if="desktop.updated_at" class="last-saved">上次保存：{{ desktop.updated_at }}</span>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeAddModal">取消</button>
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

const statusOptions = [
  { value: 'normal', label: '正常' },
  { value: 'update', label: '更新' },
  { value: 'force_update', label: '强制更新' },
  { value: 'disabled', label: '禁用' },
  { value: 'crash', label: '闪退' },
  { value: 'group_update', label: '进群更新' },
]

const statusMap: Record<string, { label: string; cls: string }> = {
  normal: { label: '正常', cls: 'green' },
  update: { label: '更新', cls: '' },
  force_update: { label: '强制更新', cls: 'red' },
  disabled: { label: '禁用', cls: 'red' },
  crash: { label: '闪退', cls: 'red' },
  group_update: { label: '进群更新', cls: '' },
}

function statusLabel(status: string): string { return statusMap[status]?.label || '未知' }
function statusClass(status: string): string { return statusMap[status]?.cls || '' }
function formatSize(n: number) { return !n ? '-' : n < 1024 * 1024 ? `${(n / 1024).toFixed(1)}KB` : `${(n / 1024 / 1024).toFixed(1)}MB` }

// ===== 桌面端配置 =====
const desktop = ref<any>({ version: '', downloadUrl: '', updateContent: '', updated_at: '' })
const desktopEnabled = ref(0)
const desktopSaving = ref(false)
const desktopChannelModalVisible = ref(false)
const desktopChannelMode = ref<'link' | 'upload'>('link')
const desktopChannelLinkDraft = ref('')
const desktopFileInputRef = ref<HTMLInputElement | null>(null)
const desktopPackageFile = ref<File | null>(null)
const desktopPackageFileDraft = ref<File | null>(null)
const desktopPackageDraft = ref({ fileName: '', fileSize: 0, fileBase64: '' })

const desktopChannelLabel = computed(() => {
  if (desktopPackageFile.value?.name) return '上传安装包'
  if (desktop.value.downloadUrl) return desktop.value.downloadUrl.startsWith('/uploads/packages/') ? '服务器安装包' : '下载链接'
  return '未选择下载渠道'
})

const desktopChannelDesc = computed(() => {
  if (desktopPackageFile.value?.name) return `已选择：${desktopPackageFile.value.name}（${formatFileSize(desktopPackageFile.value.size)}）`
  if (desktop.value.downloadUrl) return desktop.value.downloadUrl
  return desktopEnabled.value === 1 ? '启用更新时，需要选择下载链接或上传安装包' : '点击选择下载链接或上传安装包'
})

async function loadDesktop() {
  const res = await adminApi<any>('get_desktop_version')
  if (res.code === 200 && res.data) {
    desktop.value = res.data
    desktopEnabled.value = res.data.enabled ? 1 : 0
  }
}

async function saveDesktop() {
  if (!desktop.value.version?.trim()) { showToast('请填写版本号'); return }
  if (desktopEnabled.value === 1 && !desktop.value.downloadUrl?.trim() && !desktopPackageFile.value) {
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
    version: desktop.value.version.trim(),
    download_url: desktop.value.downloadUrl?.trim() || '',
    update_content: desktop.value.updateContent?.trim() || '',
    enabled: desktopEnabled.value,
    file_name: desktopPackageFile.value?.name || '',
    file_data: fileData,
  })
  desktopSaving.value = false
  if (res.code === 200) {
    showToast('保存成功', 'success')
    desktopPackageFile.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    loadDesktop()
  } else {
    showToast(res.msg || '保存失败')
  }
}

function openDesktopChannelModal() {
  desktopChannelMode.value = desktop.value.downloadUrl && !desktopPackageFile.value ? 'link' : 'upload'
  desktopChannelLinkDraft.value = desktop.value.downloadUrl || ''
  desktopPackageDraft.value = desktopPackageFile.value
    ? { fileName: desktopPackageFile.value.name, fileSize: desktopPackageFile.value.size, fileBase64: '' }
    : { fileName: '', fileSize: 0, fileBase64: '' }
  desktopPackageFileDraft.value = desktopPackageFile.value
  desktopChannelModalVisible.value = true
}

function closeDesktopChannelModal() { desktopChannelModalVisible.value = false }

function confirmDesktopChannel() {
  if (desktopChannelMode.value === 'link') {
    const url = desktopChannelLinkDraft.value.trim()
    if (!url) { showToast('请输入下载链接'); return }
    desktop.value.downloadUrl = url
    desktopPackageFile.value = null
    desktopPackageDraft.value = { fileName: '', fileSize: 0, fileBase64: '' }
    if (desktopFileInputRef.value) desktopFileInputRef.value.value = ''
  } else {
    if (!desktopPackageDraft.value.fileName) { showToast('请选择安装包'); return }
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
  const file = input.files[0]
  const ext = file.name.split('.').pop()?.toLowerCase() || ''
  const allowed = ['exe', 'msi', 'zip', '7z', 'rar', 'dmg', 'pkg', 'apk']
  if (!allowed.includes(ext)) {
    showToast('不支持该安装包格式')
    input.value = ''
    return
  }
  desktopPackageFileDraft.value = file
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

async function changeStatus(v: any, status: string) {
  const res = await adminApi('change_version_status', { id: v.id, status })
  if (res.code === 200) { v.status = status; showToast('状态已更新', 'success') }
  else showToast(res.msg || '更新失败')
}

async function deleteVersion(v: any) {
  const ok = await mobileConfirm(`确认删除版本 ${v.version_code || v.id}？`, { title: '删除版本', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_version', { id: v.id })
  if (res.code === 200) { showToast('删除成功', 'success'); loadList() }
  else showToast(res.msg || '删除失败')
}

// ===== 新增版本 =====
const addModalVisible = ref(false)
const uploading = ref(false)
const uploadProgress = ref(0)
const addForm = ref({ appName: '弦予·音乐', versionCode: '', updateContent: '', fileName: '', fileSize: 0, fileBase64: '' })

function showAddModal() {
  addForm.value = { appName: '弦予·音乐', versionCode: '', updateContent: '', fileName: '', fileSize: 0, fileBase64: '' }
  uploadProgress.value = 0
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
    reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
    reader.onerror = () => reject(new Error('文件读取失败'))
    reader.readAsDataURL(file)
  })
}

async function doAddVersion() {
  if (!addForm.value.appName.trim() || !addForm.value.versionCode.trim()) {
    showToast('请填写软件名称和版本号')
    return
  }
  const fileInput = document.querySelector('.modal-backdrop input[type="file"]') as HTMLInputElement
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

/* 桌面端卡片 */
.section-head { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 14px; }
.section-icon {
  width: 36px; height: 36px; border-radius: 10px; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  background: #eff6ff; color: #3b82f6;
}
.section-name { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); }
.section-desc { font-size: 11px; color: var(--text-muted); line-height: 1.5; margin: 2px 0 0; }

.ver-form { display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.field input, .field textarea {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
  resize: vertical;
}
.field input:focus, .field textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

.channel-card {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--control-bg);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.18s;
}
.channel-card:active { border-color: var(--accent); }
.channel-card strong { display: block; font-size: 13px; margin-bottom: 3px; }
.channel-card p { margin: 0; font-size: 11px; color: var(--text-muted); word-break: break-all; }
.channel-card > span { flex-shrink: 0; font-size: 12px; color: var(--accent); font-weight: 700; }

.enable-row { display: flex; gap: 8px; }
.enable-btn {
  flex: 1;
  padding: 10px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.18s;
}
.enable-btn.on { background: #EC4141; border-color: #EC4141; color: #fff; }
.last-saved { font-size: 11px; color: var(--text-muted); }

/* 版本列表 */
.list-card { display: flex; flex-direction: column; gap: 12px; }
.list-head { display: flex; align-items: center; justify-content: space-between; }
.list-title { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); }
.list-sub { font-size: 11px; color: var(--text-muted); }
.mobile-item { animation: itemIn 0.4s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)) backwards; }
@keyframes itemIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.ver-left { display: flex; align-items: center; gap: 10px; min-width: 0; }
.ver-badge {
  width: 38px; height: 38px; border-radius: 11px; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  background: var(--accent-soft); color: var(--accent);
  font-size: 12px; font-weight: 850;
}
.ver-code { font-size: 11px; font-weight: 700; color: var(--accent); background: var(--accent-soft); padding: 1px 7px; border-radius: 999px; margin-left: 4px; }
.ver-content { font-size: 12px; color: var(--text-light); line-height: 1.5; margin-top: 8px; word-break: break-word; }
.status-select { flex: 1; min-width: 0; width: auto; padding: 8px 10px; }
.disabled-btn { opacity: 0.6; }

/* 分页 */
.pagination { display: flex; justify-content: center; gap: 6px; flex-wrap: wrap; }
.page-btn {
  min-width: 34px;
  padding: 7px 11px;
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--text-light);
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s;
}
.page-btn.active { background: #EC4141; color: #fff; border-color: #EC4141; }
.page-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* 弹窗 */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  background: rgba(15, 23, 42, 0.38);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.modal-backdrop.channel-backdrop {
  z-index: 11000;
}
.modal-dialog {
  width: 100%;
  max-width: 420px;
  max-height: 88vh;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
/* 弹窗内分区 */
.modal-section { display: flex; flex-direction: column; gap: 12px; }
.modal-section .section-head { margin-bottom: 0; }
.modal-divider { height: 1px; background: var(--border); margin: 2px 0; }
.modal-section .ver-form { gap: 12px; }
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
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 12px 14px;
  border: 1.5px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: all 0.18s;
}
.channel-option.active { border-color: var(--accent); background: var(--accent-soft); }
.channel-option strong { font-size: 13px; }
.channel-option span { font-size: 11px; color: var(--text-muted); line-height: 1.5; }

.package-dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 130px;
  padding: 16px;
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  background: var(--control-bg);
  cursor: pointer;
  text-align: center;
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

.mobile-upload {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 86px;
  border: 1.5px dashed var(--border);
  border-radius: 16px;
  background: var(--control-bg);
  color: var(--text-muted);
  padding: 12px;
  text-align: center;
  font-size: 13px;
  font-weight: 800;
}
.mobile-upload input { display: none; }

.upload-progress { display: flex; align-items: center; gap: 8px; }
.progress-bar-track { flex: 1; height: 6px; background: var(--border); border-radius: 3px; overflow: hidden; }
.progress-bar-fill { height: 100%; background: var(--accent); transition: width 0.3s; }
.progress-text { font-size: 11px; color: var(--text-muted); min-width: 36px; text-align: right; }

/* 过渡动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn { from { opacity: 0; transform: scale(0.94); } to { opacity: 1; transform: scale(1); } }
@keyframes modalOut { from { opacity: 1; transform: scale(1); } to { opacity: 0; transform: scale(0.96); } }
</style>