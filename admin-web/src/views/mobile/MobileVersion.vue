<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">桌面端更新</h3>
      <input v-model="desktop.version" class="mobile-input" placeholder="桌面端版本号，如 1.2.0" />
      <select v-model="desktopEnabled" class="mobile-select">
        <option :value="0">禁用更新</option>
        <option :value="1">启用更新</option>
      </select>
      <input v-model="desktop.downloadUrl" class="mobile-input" placeholder="下载链接，上传安装包后自动生成" />
      <textarea v-model="desktop.updateContent" class="mobile-textarea" placeholder="桌面端更新内容"></textarea>
      <label class="mobile-upload">
        <input type="file" accept=".exe,.msi,.zip,.7z,.rar,.dmg,.pkg,.apk" @change="onDesktopFileChange" />
        <span>{{ desktopFile ? desktopFile.name : '选择桌面端安装包上传到服务器' }}</span>
      </label>
      <button class="mobile-btn primary" :disabled="desktopSaving" @click="saveDesktop">
        {{ desktopSaving ? '保存中...' : '保存桌面端配置' }}
      </button>
    </section>
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">新增 APP 版本</h3>
      <input v-model="form.appName" class="mobile-input" placeholder="软件名称" />
      <input v-model="form.versionCode" class="mobile-input" placeholder="版本号" />
      <textarea v-model="form.updateContent" class="mobile-textarea" placeholder="更新内容"></textarea>
      <label class="mobile-upload">
        <input type="file" accept=".apk" @change="onAppFileChange" />
        <span>{{ appFile ? appFile.name : '选择 APK 安装包上传到服务器' }}</span>
      </label>
      <button class="mobile-btn primary" :disabled="appUploading" @click="addVersion">
        {{ appUploading ? '上传中...' : '上传 APP 版本' }}
      </button>
    </section>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="v in list" :key="v.id" class="mobile-item">
        <div class="mobile-item-head">
          <div>
            <div class="mobile-item-title">{{ v.app_name || '弦予音乐' }} {{ v.version_code }}</div>
            <div class="mobile-item-sub">{{ v.update_content || '无更新说明' }}</div>
          </div>
          <span class="mobile-badge">{{ v.status }}</span>
        </div>
        <div class="mobile-item-sub">{{ v.created_at }} · {{ formatSize(v.file_size) }}</div>
        <div class="mobile-actions">
          <a v-if="v.download_url" class="mobile-btn" :href="v.download_url" target="_blank">下载</a>
          <select class="mobile-select status-select" :value="v.status" @change="changeStatus(v, ($event.target as HTMLSelectElement).value)">
            <option value="normal">正常</option>
            <option value="update">更新</option>
            <option value="force_update">强制更新</option>
            <option value="disabled">禁用</option>
            <option value="crash">闪退</option>
            <option value="group_update">进群更新</option>
          </select>
          <button class="mobile-btn danger" @click="deleteVersion(v)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const loading = ref(false)
const desktopSaving = ref(false)
const appUploading = ref(false)
const desktop = ref<any>({})
const desktopEnabled = ref(0)
const desktopFile = ref<File | null>(null)
const appFile = ref<File | null>(null)
const list = ref<any[]>([])
const form = ref({ appName: '', versionCode: '', updateContent: '' })
function formatSize(n: number) { return !n ? '-' : n < 1024 * 1024 ? `${(n / 1024).toFixed(1)}KB` : `${(n / 1024 / 1024).toFixed(1)}MB` }
function onDesktopFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  desktopFile.value = input.files?.[0] || null
}
function onAppFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  appFile.value = input.files?.[0] || null
}
function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
    reader.onerror = () => reject(new Error('文件读取失败'))
    reader.readAsDataURL(file)
  })
}
async function loadAll() {
  loading.value = true
  const [d, v] = await Promise.all([adminApi<any>('get_desktop_version'), adminApi<any>('list_versions', { page: 1, page_size: 30 })])
  if (d.code === 200 && d.data) {
    desktop.value = d.data
    desktopEnabled.value = d.data.enabled ? 1 : 0
  }
  list.value = v.code === 200 && v.data ? (v.data.list || []) : []
  loading.value = false
}
async function saveDesktop() {
  if (!desktop.value.version?.trim()) return showToast('请填写桌面端版本号')
  if (desktopEnabled.value === 1 && !desktop.value.downloadUrl?.trim() && !desktopFile.value) return showToast('启用更新时请填写下载链接或上传安装包')
  desktopSaving.value = true
  let fileData = ''
  try {
    if (desktopFile.value) fileData = await readFileAsBase64(desktopFile.value)
  } catch {
    desktopSaving.value = false
    return showToast('安装包读取失败')
  }
  const res = await adminApi('save_desktop_version', {
    version: desktop.value.version.trim(),
    download_url: desktop.value.downloadUrl?.trim() || '',
    update_content: desktop.value.updateContent?.trim() || '',
    enabled: desktopEnabled.value,
    file_name: desktopFile.value?.name || '',
    file_data: fileData,
  })
  desktopSaving.value = false
  if (res.code === 200) {
    showToast('桌面端配置已保存', 'success')
    desktopFile.value = null
    loadAll()
  } else showToast(res.msg || '保存失败')
}
async function addVersion() {
  if (!form.value.appName.trim() || !form.value.versionCode.trim()) return showToast('请填写软件名称和版本号')
  if (!appFile.value) return showToast('请选择 APK 安装包')
  appUploading.value = true
  let fileData = ''
  try {
    fileData = await readFileAsBase64(appFile.value)
  } catch {
    appUploading.value = false
    return showToast('安装包读取失败')
  }
  const res = await adminApi('add_version', {
    app_name: form.value.appName.trim(),
    version_code: form.value.versionCode.trim(),
    update_content: form.value.updateContent.trim(),
    file_data: fileData,
  })
  appUploading.value = false
  if (res.code === 200) {
    showToast('APP 版本已上传', 'success')
    form.value = { appName: '', versionCode: '', updateContent: '' }
    appFile.value = null
    loadAll()
  } else showToast(res.msg || '上传失败')
}
async function changeStatus(v: any, status: string) {
  const res = await adminApi('change_version_status', { id: v.id, status })
  if (res.code === 200) { v.status = status; showToast('状态已更新', 'success') } else showToast(res.msg || '更新失败')
}
async function deleteVersion(v: any) {
  if (!(await mobileConfirm(`确认删除版本 ${v.version_code}？`))) return
  const res = await adminApi('delete_version', { id: v.id })
  if (res.code === 200) { showToast('已删除', 'success'); loadAll() } else showToast(res.msg || '删除失败')
}
onMounted(loadAll)
</script>
<style scoped>
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
.mobile-upload input {
  display: none;
}
.status-select {
  width: auto;
  min-width: 112px;
  padding: 8px 10px;
}
</style>
