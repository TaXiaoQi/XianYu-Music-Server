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
          <p class="bpm-hint">一次发布同一版本号到多个平台：批量添加各平台安装包，自动识别平台 / 系统 / 架构，可逐条修正后统一保存。</p>

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
            <label>安装包（可多选，自动识别平台 / 系统 / 架构）</label>
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
                <button type="button" class="bpm-row-del" title="移除" @click="removeRow(row)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
              </div>
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

          <p v-if="saving" class="bpm-progress"><span class="bpm-spinner"></span>正在发布 {{ doneCount + 1 }}/{{ rows.length }}，大文件上传需要一些时间...</p>
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
import { ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import {
  PACKAGE_ALLOWED_EXT,
  PLATFORM_OPTIONS,
  SYSTEM_OPTIONS,
  ARCH_OPTIONS,
  defaultSystemOf,
  defaultArchOf,
  detectPackageMeta,
  type PlatformKey,
} from '@/utils/packageDetect'

interface Row {
  id: number
  file: File
  platform: PlatformKey | ''
  system: string
  arch: string
  error: string
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
    const key = meta ? `${meta.platform}-${meta.system}-${meta.arch}` : `raw-${file.name}`
    if (rows.value.some(r => (r.platform ? `${r.platform}-${r.system}-${r.arch}` : `raw-${r.file.name}`) === key)) {
      dup++
      continue
    }
    rows.value.push({
      id: rowSeq++,
      file,
      platform: meta?.platform || '',
      system: meta?.system || '',
      arch: meta?.arch || '',
      error: '',
    })
    added++
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
}

function onRowSystem(row: Row) {
  if (row.platform === 'watch' && (row.system === 'ohos' || row.system === 'watchos')) {
    row.arch = 'arm64'
  }
}

function removeRow(row: Row) {
  rows.value = rows.value.filter(r => r.id !== row.id)
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
    const key = `${r.platform}-${r.system}-${r.arch}`
    if (seen.has(key)) {
      showToast('存在重复的平台 / 系统 / 架构组合，请检查列表')
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
      const fileData = await readFileAsBase64(row.file)
      const res = await adminApi('save_desktop_version', {
        platform: row.platform,
        system: row.system,
        arch: row.arch,
        channel: channel.value,
        version: ver,
        download_url: '',
        update_content: updateContent.value.trim(),
        enabled: enabled.value ? 1 : 0,
        store_url: '',
        file_name: row.file.name,
        file_data: fileData,
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
  if (!saving.value) emit('close')
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
