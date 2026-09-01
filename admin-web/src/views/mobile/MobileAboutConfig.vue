<template>
  <div class="mobile-page">
    <!-- 页面头部 -->
    <section class="mobile-page-head">
      <div class="mobile-head-info">
        <div class="mobile-head-title">关于页配置</div>
        <div class="mobile-head-desc">按客户端平台分别配置关于页的官网、检查更新、项目地址等入口，保存后由对应平台客户端下发。</div>
      </div>
      <button class="mobile-btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存配置' }}</button>
    </section>

    <div class="platform-tabs">
      <button
        v-for="p in PLATFORMS"
        :key="p.key"
        class="platform-tab"
        :class="{ active: platform === p.key }"
        @click="switchPlatform(p.key)"
      >{{ p.label }}</button>
    </div>

    <div v-if="loading" class="mobile-empty">加载中...</div>

    <template v-else>
      <!-- 官网入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">官网入口</h3>
        <div class="about-field">
          <label>官网链接</label>
          <input v-model="form.officialSiteUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 加入群组入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">加入群组入口</h3>
        <div class="about-field">
          <label>群组链接</label>
          <input v-model="form.joinGroupUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 检查更新入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">检查更新入口</h3>
        <label class="switch-row">
          <input v-model="form.updateEnabled" type="checkbox" />
          <span>显示检查更新按钮</span>
        </label>
      </section>

      <!-- 项目地址 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">项目地址</h3>
        <div class="about-field">
          <label>项目链接</label>
          <input v-model="form.projectUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 参考项目 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">参考项目</h3>
        <div class="about-field">
          <label>参考项目链接</label>
          <input v-model="form.referenceProjectUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <p class="about-hint">只配置链接，按钮显示文字由客户端按语言本地化。链接留空后，{{ platform === 'mobile' ? '移动端' : '桌面端' }}会隐藏对应外链按钮；检查更新入口可通过开关隐藏。两个平台的配置独立存储、互不影响。</p>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

interface AboutConfig {
  officialSiteUrl: string
  updateEnabled: boolean
  projectUrl: string
  referenceProjectUrl: string
  joinGroupUrl: string
}

const desktopDefaults: AboutConfig = {
  officialSiteUrl: 'https://xianyumusic.cn',
  updateEnabled: true,
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop',
  referenceProjectUrl: 'https://github.com/Billy636/XianYuMusic',
  joinGroupUrl: 'https://qm.qq.com/q/kvteWSD8yY',
}

const mobileDefaults: AboutConfig = {
  ...desktopDefaults,
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Mobile',
  referenceProjectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop',
}

const loading = ref(true)
const saving = ref(false)
const platform = ref<'desktop' | 'mobile'>('desktop')
const defaultConfig = computed(() => (platform.value === 'mobile' ? mobileDefaults : desktopDefaults))
const form = ref<AboutConfig>({ ...desktopDefaults })

const PLATFORMS = [
  { key: 'desktop' as const, label: '桌面端' },
  { key: 'mobile' as const, label: '移动端' },
]

function switchPlatform(key: 'desktop' | 'mobile') {
  if (platform.value === key || loading.value || saving.value) return
  platform.value = key
  loadConfig()
}

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<AboutConfig>>('get_about_config_admin', { platform: platform.value })
  if (res.code === 200 && res.data) {
    form.value = { ...defaultConfig.value, ...res.data }
  } else {
    showToast(res.msg || '加载配置失败')
  }
  loading.value = false
}

async function save() {
  saving.value = true
  const res = await adminApi('save_about_config', {
    platform: platform.value,
    ...form.value,
    updateEnabled: form.value.updateEnabled ? 1 : 0,
  })
  saving.value = false
  if (res.code === 200) {
    showToast('保存成功', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.mobile-page-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}
.mobile-head-title {
  font-size: 18px;
  font-weight: 850;
  color: var(--text);
}
.mobile-head-desc {
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.mobile-page-head .mobile-btn {
  flex: 0 0 auto;
  white-space: nowrap;
}

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

.about-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.about-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.about-field label {
  font-size: 12px;
  font-weight: 800;
  color: var(--text-light);
}
.about-field .at {
  margin-top: 0;
}
.switch-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 0;
}
.switch-row input {
  width: 18px;
  height: 18px;
  accent-color: #EC4141;
}
.switch-row span {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}
.about-hint {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.6;
  padding: 0 4px;
}
</style>