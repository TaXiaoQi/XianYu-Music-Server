<template>
  <Transition name="bpm-fade" appear>
    <div class="bpm-backdrop" @click.self="requestClose">
      <div class="bpm-dialog">
        <div class="bpm-head">
          <h3>批量发布</h3>
          <button class="bpm-close" @click="requestClose">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="bpm-form">
          <p class="bpm-hint">一次发布同一版本号到多个平台：批量添加各平台安装包，自动识别平台 / 系统 / 架构 / 格式，可逐条修正后统一保存。</p>

          <div class="bpm-field">
            <label class="bpm-required">更新渠道</label>
            <div class="bpm-picker">
              <button type="button" :class="{ active: channel === 'stable' }" @click="channel = 'stable'">正式版</button>
              <button type="button" :class="{ active: channel === 'beta' }" @click="channel = 'beta'">测试版</button>
            </div>
          </div>

          <div class="bpm-field">
            <label class="bpm-required">版本号</label>
            <div v-if="channel === 'stable'" class="bpm-version-row">
              <input v-model="version" type="text" placeholder="如 1.2.0" />
            </div>
            <div v-else class="bpm-version-row">
              <input v-model="version" type="text" placeholder="如 1.2.0" />
              <span class="bpm-beta-sep">-beta-</span>
              <input v-model="betaNum" type="text" inputmode="numeric" placeholder="1" class="bpm-num" />
            </div>
          </div>

          <div class="bpm-field">
            <label>安装包（可多选，自动识别平台 / 系统 / 架构 / 格式）</label>
            <div
              class="bpm-dropzone"
              :class="{ dragging }"
              @click="fileRef?.click()"
              @dragover.prevent="dragging = true"
              @dragleave.prevent="dragging = false"
              @drop.prevent="onDrop"
            >
              <input ref="fileRef" type="file" multiple :accept="acceptAttr" class="bpm-file-hidden" @change="onPick" />
              <div class="bpm-drop-icon">⬆</div>
              <strong>点击或拖拽安装包到此处（支持多选）</strong>
              <span>EXE / MSI / ZIP / 7Z / RAR / DMG / PKG / APK / HAP / IPA / DEB / APPIMAGE</span>
            </div>
          </div>

          <div v-if="rows.length" class="bpm-rows">
            <div v-for="row in rows" :key="row.id" class="bpm-row" :class="{ invalid: !row.platform }">
              <div class="bpm-row-file">
                <strong :title="row.file.name">{{ row.file.name }}</strong>
                <span>{{ formatFileSize(row.file.size) }}</span>
              </div>
              <div class="bpm-row-pickers">
                <select v-model="row.platform" @change="onRowPlatform(row)">
                  <option value="" disabled>平台</option>
                  <option v-for="p in PLATFORM_OPTIONS" :key="p.key" :value="p.key">{{ p.label }}</option>
                </select>
                <select v-model="row.system" :disabled="!row.platform" @change="onRowSystem(row)">
                  <option v-for="s in systemOptionsOf(row.platform)" :key="s.key" :value="s.key">{{ s.label }}</option>
                </select>
                <select v-model="row.arch" :disabled="!row.platform || row.platform === 'mobile'">
                  <option v-for="a in archOptionsOf(row.platform)" :key="a.key" :value="a.key">{{ a.label }}</option>
                </select>
                <select v-model="row.pkg" :disabled="!row.platform" title="安装包格式">
                  <option value="">格式</option>
                  <option v-for="f in pkgOptionsOf(row.platform, row.system)" :key="f.key" :value="f.key">{{ f.label }}</option>
                </select>
                <button type="button" class="bpm-row-del" title="移除" @click="removeRow(row)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
              </div>
              <!-- 后台上传状态：选中文件即开始上传，与新增版本弹窗一致 -->
              <div v-if="row.uploading" class="bpm-row-up">
                <div class="bpm-up-track"><div class="bpm-up-fill" :style="{ width: row.progress + '%' }"></div></div>
                <span class="bpm-up-pct">{{ row.progress }}%</span>
                <button type="button" class="bpm-up-cancel" @click="cancelRowUpload(row)">取消</button>
              </div>
              <p v-else-if="row.uploaded" class="bpm-row-up-ok">安装包已上传完成</p>
              <p v-else-if="row.uploadError" class="bpm-row-err">{{ row.uploadError }}（发布时将自动重试）</p>
              <p v-if="!row.platform" class="bpm-row-err">未能识别平台，请手动选择平台 / 系统 / 架构</p>
              <p v-else-if="row.error" class="bpm-row-err">{{ row.error }}</p>
            </div>
          </div>

          <div class="bpm-field">
            <label>更新内容（所有平台共用）</label>
            <textarea v-model="updateContent" rows="6" placeholder="本次更新内容"></textarea>
          </div>

          <div class="bpm-field">
            <label>启用状态</label>
            <div class="bpm-picker">
              <button type="button" class="bpm-on" :class="{ active: enabled }" @click="enabled = true">启用</button>
              <button type="button" class="bpm-off" :class="{ active: !enabled }" @click="enabled = false">禁用</button>
            </div>
          </div>

          <p v-if="saving" class="bpm-progress"><span class="bpm-spinner"></span>正在发布 {{ doneCount + 1 }}/{{ rows.length }}...</p>
          <p v-else-if="uploadingCount" class="bpm-progress"><span class="bpm-spinner"></span>正在上传安装包 {{ uploadingCount }}/{{ rows.length }}，发布时会自动等待上传完成</p>
          <p v-else-if="rows.length" class="bpm-progress bpm-progress-idle">共 {{ rows.length }} 个平台，将共用版本号 {{ composedVersion || '（未填写）' }}</p>
        </div>
        <div class="bpm-foot">
          <button class="bpm-cancel" :disabled="saving" @click="requestClose">取消</button>
          <button class="bpm-save" :disabled="saving || !rows.length" @click="saveAll">
            {{ saving ? '发布中...' : `发布${rows.length ? `（${rows.length} 个平台）` : ''}` }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { adminApi, showToast, type ApiResponse } from '@/api/client'
import { uploadPackage } from '@/api/version'
import {
  PACKAGE_ALLOWED_EXT,
  PLATFORM_OPTIONS,
  SYSTEM_OPTIONS,
  ARCH_OPTIONS,
  defaultSystemOf,
  defaultArchOf,
  detectPackageMeta,
  pkgOptionsOf,
  type PlatformKey,
} from '@/utils/packageDetect'

interface Row {
  id: number
  file: File
  platform: PlatformKey | ''
  system: string
  arch: string
  pkg: string
  error: string
  // 后台上传状态（选中文件即开始上传，与新增版本弹窗一致）
  uploading: boolean
  progress: number
  uploaded: boolean
  downloadUrl: string
  uploadError: string
}

const emit = defineEmits<{ (e: 'close'): void; (e: 'saved'): void }>()

const version = ref('')
const betaNum = ref('')
const channel = ref<'stable' | 'beta'>('stable')
const updateContent = ref('')
const enabled = ref(true)
const rows = ref<Row[]>([])
let rowSeq = 1
const dragging = ref(false)
const saving = ref(false)
const doneCount = ref(0)
const fileRef = ref<HTMLInputElement | null>(null)

// 上传句柄与任务（非响应式，按行 id 索引）
const uploadHandles = new Map<number, { abort: () => void }>()
const uploadTasks = new Map<number, Promise<ApiResponse>>()
const uploadingCount = computed(() => rows.value.filter(r => r.uploading).length)

const acceptAttr = PACKAGE_ALLOWED_EXT.map(e => `.${e}`).join(',')

const composedVersion = ref('')

function refreshComposedVersion() {
  const main = version.value.trim()
  if (channel.value !== 'beta') {
    composedVersion.value = main
    return
  }
  const num = parseInt(betaNum.value, 10) || 0
  composedVersion.value = main && num > 0 ? `${main}-beta-${num}` : ''
}

function addFiles(files: FileList | File[]) {
  let added = 0
  let unrecognized = 0
  let dup = 0
  for (const file of Array.from(files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    if (!PACKAGE_ALLOWED_EXT.includes(ext)) {
      showToast(`不支持的安装包格式：${file.name}`)
      continue
    }
    const meta = detectPackageMeta(file.name)
    if (!meta) unrecognized++
    const key = meta ? `${meta.platform}-${meta.system}-${meta.arch}-${meta.pkg}` : `raw-${file.name}`
    if (rows.value.some(r => (r.platform ? `${r.platform}-${r.system}-${r.arch}-${r.pkg}` : `raw-${r.file.name}`) === key)) {
      dup++
      continue
    }
    const row: Row = {
      id: rowSeq++,
      file,
      platform: meta?.platform || '',
      system: meta?.system || '',
      arch: meta?.arch || '',
      pkg: meta?.pkg || '',
      error: '',
      uploading: false,
      progress: 0,
      uploaded: false,
      downloadUrl: '',
      uploadError: '',
    }
    rows.value.push(row)
    added++
    // 选中即后台上传，与新增版本弹窗一致；发布时等待/重试
    startRowUpload(row)
  }
  if (added && unrecognized) showToast(`${unrecognized} 个安装包未能识别平台，请在列表中手动选择`)
  if (dup) showToast(`${dup} 个安装包与已添加条目重复，已跳过`)
}

function onPick(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files?.length) addFiles(input.files)
  input.value = ''
}

function onDrop(e: DragEvent) {
  dragging.value = false
  if (e.dataTransfer?.files?.length) addFiles(e.dataTransfer.files)
}

function onRowPlatform(row: Row) {
  if (!row.platform) return
  row.system = defaultSystemOf(row.platform)
  row.arch = defaultArchOf(row.platform)
  // 平台 / 系统变更后，原格式若不属于新维度则重置
  if (!pkgOptionsOf(row.platform, row.system).some(o => o.key === row.pkg)) row.pkg = ''
}

function onRowSystem(row: Row) {
  if (row.platform === 'watch' && (row.system === 'ohos' || row.system === 'watchos')) {
    row.arch = 'arm64'
  }
  if (row.platform && !pkgOptionsOf(row.platform, row.system).some(o => o.key === row.pkg)) row.pkg = ''
}

function removeRow(row: Row) {
  abortRowUpload(row)
  rows.value = rows.value.filter(r => r.id !== row.id)
}

// ==================== 行级后台上传 ====================

function startRowUpload(row: Row) {
  if (uploadTasks.has(row.id)) return
  row.uploading = true
  row.progress = 0
  row.uploaded = false
  row.downloadUrl = ''
  row.uploadError = ''
  const task = (async (): Promise<ApiResponse> => {
    let fileData = ''
    try {
      fileData = await readFileAsBase64(row.file)
    } catch {
      if (!rows.value.some(r => r.id === row.id)) return { code: 499, msg: '上传已取消', data: null }
      row.uploading = false
      row.uploadError = '安装包读取失败'
      return { code: 500, msg: '安装包读取失败', data: null }
    }
    if (!rows.value.some(r => r.id === row.id)) return { code: 499, msg: '上传已取消', data: null }
    const handle = uploadPackage({ file_name: row.file.name, file_data: fileData }, (p) => {
      row.progress = p
    })
    uploadHandles.set(row.id, handle)
    const res = await handle.promise
    uploadHandles.delete(row.id)
    if (!rows.value.some(r => r.id === row.id)) return res
    row.uploading = false
    if (res.code === 200 && res.data?.download_url) {
      row.progress = 100
      row.uploaded = true
      row.downloadUrl = res.data.download_url
    } else if (res.code !== 499) {
      row.uploadError = res.msg || '安装包上传失败'
    }
    return res
  })()
  uploadTasks.set(row.id, task)
  task.finally(() => {
    if (uploadTasks.get(row.id) === task) uploadTasks.delete(row.id)
  })
}

// 取消单行上传（移除该行时调用）
function abortRowUpload(row: Row) {
  uploadHandles.get(row.id)?.abort()
  uploadHandles.delete(row.id)
  uploadTasks.delete(row.id)
}

// 用户点击行内取消：中断上传并重置状态
async function cancelRowUpload(row: Row) {
  abortRowUpload(row)
  row.uploading = false
  row.progress = 0
  row.uploaded = false
  row.downloadUrl = ''
  row.uploadError = ''
  showToast(`已取消上传：${row.file.name}`)
}

// 发布前调用：已完成直接通过；上传中等待；失败/被取消则重启上传再等待
async function ensureRowUploaded(row: Row): Promise<boolean> {
  if (row.uploaded && row.downloadUrl) return true
  const running = uploadTasks.get(row.id)
  if (running) {
    await running
  } else {
    row.uploadError = ''
    startRowUpload(row)
    await uploadTasks.get(row.id)
  }
  return row.uploaded && !!row.downloadUrl
}

// 关闭弹窗时取消全部未完成上传
function abortAllUploads() {
  for (const handle of uploadHandles.values()) handle.abort()
  uploadHandles.clear()
  uploadTasks.clear()
}

function systemOptionsOf(platform: PlatformKey | '') {
  return platform ? SYSTEM_OPTIONS[platform] : []
}

function archOptionsOf(platform: PlatformKey | '') {
  return platform ? ARCH_OPTIONS[platform] : []
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

async function saveAll() {
  if (saving.value) return
  refreshComposedVersion()
  const ver = composedVersion.value
  if (!ver) {
    showToast(channel.value === 'beta' ? '请填写版本号和 beta 号（正整数）' : '请填写版本号')
    return
  }
  if (!rows.value.length) {
    showToast('请先添加至少一个安装包')
    return
  }
  const missing = rows.value.filter(r => !r.platform)
  if (missing.length) {
    showToast(`${missing.length} 个安装包未选择平台，请补全后发布`)
    return
  }
  const seen = new Set<string>()
  for (const r of rows.value) {
    const key = `${r.platform}-${r.system}-${r.arch}-${r.pkg}`
    if (seen.has(key)) {
      showToast('存在重复的平台 / 系统 / 架构 / 格式组合，请检查列表')
      return
    }
    seen.add(key)
  }
  saving.value = true
  doneCount.value = 0
  const remain: Row[] = []
  for (let i = 0; i < rows.value.length; i++) {
    const row = rows.value[i]
    try {
      // 等待/重试该行的后台上传，拿到 download_url 后再保存版本
      const uploaded = await ensureRowUploaded(row)
      if (!uploaded) {
        row.error = row.uploadError || '安装包上传未完成，请重试'
        remain.push(row)
        doneCount.value = i + 1
        continue
      }
      const res = await adminApi('save_desktop_version', {
        platform: row.platform,
        system: row.system,
        arch: row.arch,
        pkg: row.pkg,
        channel: channel.value,
        version: ver,
        download_url: row.downloadUrl,
        update_content: updateContent.value.trim(),
        enabled: enabled.value ? 1 : 0,
        store_url: '',
      })
      if (res.code === 200) {
        row.error = ''
      } else {
        row.error = res.msg || '保存失败'
        remain.push(row)
      }
    } catch {
      row.error = '网络错误，请重试'
      remain.push(row)
    }
    doneCount.value = i + 1
  }
  saving.value = false
  const okCount = rows.value.length - remain.length
  rows.value = remain
  if (!remain.length) {
    showToast(`批量发布完成：${okCount} 个平台全部成功`, 'success')
    emit('saved')
    emit('close')
  } else {
    showToast(`已成功 ${okCount} 个，失败 ${remain.length} 个（保留在列表中，可修正后重试）`)
  }
}

function requestClose() {
  if (saving.value) return
  abortAllUploads()
  emit('close')
}
</script>

<style scoped>
.bpm-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 20, 28, 0.55);
  backdrop-filter: blur(4px);
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.bpm-dialog {
  width: 560px;
  max-width: 100%;
  max-height: calc(100vh - 48px);
  display: flex;
  flex-direction: column;
  background: #fff;
  border-radius: 18px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.22);
  overflow: hidden;
}
.bpm-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 22px 0;
}
.bpm-head h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
  color: #1f2329;
}
.bpm-close {
  border: none;
  background: transparent;
  color: #8a919f;
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  display: flex;
}
.bpm-close:hover { background: #f2f3f5; color: #1f2329; }
.bpm-form {
  padding: 14px 22px 6px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.bpm-hint {
  margin: 0;
  font-size: 12.5px;
  color: #8a919f;
  line-height: 1.6;
  background: #f7f8fa;
  border-radius: 10px;
  padding: 9px 12px;
}
.bpm-field { display: flex; flex-direction: column; gap: 7px; }
.bpm-field > label { font-size: 13px; font-weight: 600; color: #3c4149; }
.bpm-required::after { content: ' *'; color: #e5484d; }
.bpm-picker { display: flex; gap: 8px; }
.bpm-picker button {
  border: 1px solid #e6e8eb;
  background: #fff;
  color: #5c6370;
  font-size: 13px;
  padding: 7px 16px;
  border-radius: 999px;
  cursor: pointer;
  transition: all 0.15s;
}
.bpm-picker button.active { border-color: #e5484d; color: #e5484d; background: #fdf1f2; font-weight: 600; }
.bpm-version-row { display: flex; align-items: center; gap: 8px; }
.bpm-version-row input {
  flex: 1;
  border: 1px solid #e6e8eb;
  border-radius: 10px;
  padding: 9px 12px;
  font-size: 13.5px;
  color: #1f2329;
  outline: none;
}
.bpm-version-row input:focus { border-color: #e5484d; }
.bpm-beta-sep { font-size: 13px; color: #8a919f; }
.bpm-num { max-width: 90px; }
.bpm-dropzone {
  border: 1.5px dashed #d5d9de;
  border-radius: 12px;
  padding: 18px 14px;
  text-align: center;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  transition: border-color 0.15s, background 0.15s;
}
.bpm-dropzone:hover, .bpm-dropzone.dragging { border-color: #e5484d; background: #fdf5f5; }
.bpm-drop-icon { font-size: 20px; color: #e5484d; }
.bpm-dropzone strong { font-size: 13.5px; color: #3c4149; }
.bpm-dropzone span { font-size: 12px; color: #a0a6b0; }
.bpm-file-hidden { display: none; }
.bpm-rows { display: flex; flex-direction: column; gap: 10px; }
.bpm-row {
  border: 1px solid #eceef1;
  border-radius: 12px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: #fbfbfc;
}
.bpm-row.invalid { border-color: #f3b9be; background: #fef7f7; }
.bpm-row-file { display: flex; align-items: baseline; gap: 8px; }
.bpm-row-file strong {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  color: #1f2329;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.bpm-row-file span { font-size: 12px; color: #a0a6b0; flex-shrink: 0; }
.bpm-row-pickers { display: flex; gap: 8px; align-items: center; }
.bpm-row-pickers select {
  flex: 1;
  min-width: 0;
  border: 1px solid #e6e8eb;
  border-radius: 9px;
  padding: 6px 8px;
  font-size: 12.5px;
  color: #1f2329;
  background: #fff;
  outline: none;
}
.bpm-row-pickers select:focus { border-color: #e5484d; }
.bpm-row-pickers select:disabled { background: #f4f5f7; color: #a0a6b0; }
.bpm-row-del {
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: #a0a6b0;
  cursor: pointer;
  padding: 6px;
  border-radius: 8px;
  display: flex;
}
.bpm-row-del:hover { background: #fdeced; color: #e5484d; }
.bpm-row-err { margin: 0; font-size: 12px; color: #e5484d; }
.bpm-row-up { display: flex; align-items: center; gap: 8px; }
.bpm-up-track {
  flex: 1;
  height: 6px;
  background: #f2f3f5;
  border-radius: 999px;
  overflow: hidden;
}
.bpm-up-fill {
  height: 100%;
  background: linear-gradient(90deg, #ff7a45, #e5484d);
  border-radius: 999px;
  transition: width 0.2s ease;
}
.bpm-up-pct {
  font-size: 12px;
  color: #5c6370;
  min-width: 34px;
  text-align: right;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}
.bpm-up-cancel {
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: #a0a6b0;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 6px;
}
.bpm-up-cancel:hover { background: #fdeced; color: #e5484d; }
.bpm-row-up-ok { margin: 0; font-size: 12px; color: #30a46c; }
.bpm-progress {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  color: #5c6370;
}
.bpm-progress-idle { color: #a0a6b0; }
.bpm-spinner {
  width: 13px;
  height: 13px;
  border: 2px solid #e5484d;
  border-top-color: transparent;
  border-radius: 50%;
  animation: bpm-spin 0.8s linear infinite;
  flex-shrink: 0;
}
@keyframes bpm-spin { to { transform: rotate(360deg); } }
.bpm-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 22px 18px;
  border-top: 1px solid #f2f3f5;
}
.bpm-cancel, .bpm-save {
  border-radius: 10px;
  font-size: 13.5px;
  font-weight: 600;
  padding: 9px 20px;
  cursor: pointer;
  border: none;
  transition: opacity 0.15s;
}
.bpm-cancel { background: #f2f3f5; color: #5c6370; }
.bpm-save { background: #e5484d; color: #fff; }
.bpm-save:disabled { opacity: 0.55; cursor: not-allowed; }
.bpm-cancel:disabled { opacity: 0.55; cursor: not-allowed; }
.bpm-fade-enter-active, .bpm-fade-leave-active { transition: opacity 0.18s ease; }
.bpm-fade-enter-active .bpm-dialog, .bpm-fade-leave-active .bpm-dialog { transition: transform 0.18s ease; }
.bpm-fade-enter-from, .bpm-fade-leave-to { opacity: 0; }
.bpm-fade-enter-from .bpm-dialog, .bpm-fade-leave-to .bpm-dialog { transform: translateY(12px) scale(0.98); }
@media (max-width: 640px) {
  .bpm-backdrop { padding: 12px; align-items: flex-end; }
  .bpm-dialog { max-height: calc(100vh - 24px); border-radius: 16px 16px 0 0; }
  .bpm-row-pickers { flex-wrap: wrap; }
  .bpm-row-pickers select { flex: 1 1 30%; }
}
</style>
