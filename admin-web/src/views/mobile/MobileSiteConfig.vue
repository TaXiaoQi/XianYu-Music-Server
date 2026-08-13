<template>
  <div class="mobile-page">
    <div v-if="loading" class="mobile-card mobile-empty">加载中...</div>

    <template v-else>
      <!-- 当前 Logo 预览 -->
      <div class="mobile-card">
        <h3 class="mobile-card-title">当前 Logo</h3>
        <div class="logo-preview-row">
          <div class="logo-preview-box">
            <img :src="previewUrl || siteLogoUrl" alt="站点 Logo" class="logo-preview-img" />
          </div>
          <div class="logo-preview-info">
            <span class="mobile-badge">浏览器标签页</span>
            <div class="favicon-chip">
              <img :src="previewUrl || siteLogoUrl" alt="favicon" class="favicon-img" />
            </div>
            <p class="mobile-muted lock-hint">建议使用 512×512 以内、透明底的 PNG 图片，正方形效果最佳。</p>
          </div>
        </div>
      </div>

      <!-- 上传新 Logo -->
      <div class="mobile-card mobile-form">
        <h3 class="mobile-card-title">上传新 Logo</h3>
        <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp,image/gif" style="display:none" @change="onFileChange" />
        <button class="mobile-btn" @click="fileInput?.click()">选择图片</button>
        <p class="mobile-muted file-name">{{ fileName || '未选择图片' }}</p>

        <div v-if="previewUrl" class="new-preview">
          <img :src="previewUrl" alt="新 Logo 预览" class="new-preview-img" />
          <span>新 Logo 预览</span>
        </div>

        <div class="mobile-actions">
          <button class="mobile-btn primary" :disabled="saving || !hasNewImage" @click="upload">
            {{ saving ? '上传中...' : '上传并保存' }}
          </button>
          <button v-if="hasNewImage" class="mobile-btn" :disabled="saving" @click="reset">取消</button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { loadSiteLogo, setSiteLogo, siteLogoUrl } from '@/utils/siteLogo'
import './MobilePage.css'

const loading = ref(true)
const saving = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const fileName = ref('')
const selectedImage = ref('')

const previewUrl = computed(() => selectedImage.value)
const hasNewImage = computed(() => !!selectedImage.value)

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  if (!/^image\/(png|jpeg|webp|gif)$/.test(file.type)) {
    showToast('只支持 PNG/JPG/WEBP/GIF 图片')
    return
  }
  fileName.value = file.name
  const reader = new FileReader()
  reader.onload = () => {
    selectedImage.value = String(reader.result || '')
  }
  reader.readAsDataURL(file)
}

function reset() {
  fileName.value = ''
  selectedImage.value = ''
  if (fileInput.value) fileInput.value.value = ''
}

async function upload() {
  if (!hasNewImage.value) return
  saving.value = true
  const res = await adminApi<{ logo_url: string }>('upload_site_logo', { image: selectedImage.value })
  saving.value = false
  if (res.code === 200 && res.data?.logo_url) {
    setSiteLogo(res.data.logo_url)
    showToast('站点 Logo 已更新', 'success')
    reset()
  } else {
    showToast(res.msg || '上传失败')
  }
}

onMounted(async () => {
  await loadSiteLogo()
  loading.value = false
})
</script>

<style scoped>
.logo-preview-row {
  display: flex;
  align-items: center;
  gap: 16px;
}
.logo-preview-box {
  width: 96px;
  height: 96px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--control-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.logo-preview-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.logo-preview-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.favicon-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--control-bg);
}
.favicon-img {
  width: 22px;
  height: 22px;
  object-fit: contain;
}
.lock-hint {
  line-height: 1.5;
}
.file-name {
  word-break: break-all;
}
.new-preview {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px dashed var(--border);
  border-radius: 14px;
  background: var(--control-bg);
}
.new-preview-img {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  object-fit: contain;
  background: var(--card);
  flex-shrink: 0;
}
.new-preview span {
  font-size: 13px;
  font-weight: 800;
  color: var(--text-light);
}
</style>