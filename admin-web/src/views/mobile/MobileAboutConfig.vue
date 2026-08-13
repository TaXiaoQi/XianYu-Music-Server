<template>
  <div class="mobile-page">
    <!-- 页面头部 -->
    <section class="mobile-page-head">
      <div class="mobile-head-info">
        <div class="mobile-head-title">关于页配置</div>
        <div class="mobile-head-desc">配置桌面端关于页的官网、检查更新、项目地址等入口，保存后由客户端从后台下发。</div>
      </div>
      <button class="mobile-btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存配置' }}</button>
    </section>

    <div v-if="loading" class="mobile-empty">加载中...</div>

    <template v-else>
      <!-- 官网入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">官网入口</h3>
        <div class="about-field">
          <label>按钮文字</label>
          <input v-model="form.officialSiteText" class="mobile-input" type="text" placeholder="前往官网" />
        </div>
        <div class="about-field">
          <label>官网链接</label>
          <input v-model="form.officialSiteUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 加入群组入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">加入群组入口</h3>
        <div class="about-field">
          <label>按钮文字</label>
          <input v-model="form.joinGroupText" class="mobile-input" type="text" placeholder="加入群组" />
        </div>
        <div class="about-field">
          <label>群组链接</label>
          <input v-model="form.joinGroupUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 检查更新入口 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">检查更新入口</h3>
        <div class="about-field">
          <label>按钮文字</label>
          <input v-model="form.updateText" class="mobile-input" type="text" placeholder="检查更新" />
        </div>
        <label class="switch-row">
          <input v-model="form.updateEnabled" type="checkbox" />
          <span>显示检查更新按钮</span>
        </label>
      </section>

      <!-- 项目地址 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">项目地址</h3>
        <div class="about-field">
          <label>按钮文字</label>
          <input v-model="form.projectText" class="mobile-input" type="text" placeholder="开源地址" />
        </div>
        <div class="about-field">
          <label>项目链接</label>
          <input v-model="form.projectUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <!-- 参考项目 -->
      <section class="mobile-card about-section">
        <h3 class="mobile-card-title">参考项目</h3>
        <div class="about-field">
          <label>按钮文字</label>
          <input v-model="form.referenceProjectText" class="mobile-input" type="text" placeholder="参考项目" />
        </div>
        <div class="about-field">
          <label>参考项目链接</label>
          <input v-model="form.referenceProjectUrl" class="mobile-input" type="text" placeholder="https://..." />
        </div>
      </section>

      <p class="about-hint">链接留空后，桌面端会隐藏对应外链按钮；检查更新入口可通过开关隐藏。</p>
    </template>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

interface AboutConfig {
  officialSiteUrl: string
  officialSiteText: string
  updateEnabled: boolean
  updateText: string
  projectUrl: string
  projectText: string
  referenceProjectUrl: string
  referenceProjectText: string
  joinGroupUrl: string
  joinGroupText: string
}

const defaultConfig: AboutConfig = {
  officialSiteUrl: 'https://xymusic.cc',
  officialSiteText: '前往官网',
  updateEnabled: true,
  updateText: '检查更新',
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop',
  projectText: '开源地址',
  referenceProjectUrl: 'https://github.com/Billy636/XianYuMusic',
  referenceProjectText: '参考项目',
  joinGroupUrl: 'https://qm.qq.com/q/kvteWSD8yY',
  joinGroupText: '加入群组',
}

const loading = ref(true)
const saving = ref(false)
const form = ref<AboutConfig>({ ...defaultConfig })

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<AboutConfig>>('get_about_config_admin')
  if (res.code === 200 && res.data) {
    form.value = { ...defaultConfig, ...res.data }
  } else {
    showToast(res.msg || '加载配置失败')
  }
  loading.value = false
}

async function save() {
  saving.value = true
  const res = await adminApi('save_about_config', {
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