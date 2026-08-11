<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">人机验证设置</h3>
      <p class="mobile-muted">配置注册和找回密码验证码接口的人机验证，支持 Cloudflare Turnstile 与 hCaptcha。</p>
      <div v-if="loading" class="mobile-empty">加载中...</div>
      <template v-else>
        <label class="mobile-switch-row">
          <span>
            <strong>启用人机验证</strong>
            <small>开启后，发送验证码前需完成人机验证。</small>
          </span>
          <input v-model="form.enabled" type="checkbox" :disabled="saving" />
        </label>
        <label class="mobile-field">
          <span>验证服务商</span>
          <select v-model="form.provider" class="mobile-select" :disabled="saving">
            <option value="turnstile">Cloudflare Turnstile</option>
            <option value="hcaptcha">hCaptcha</option>
          </select>
        </label>
        <label class="mobile-field">
          <span>Site Key</span>
          <input v-model="form.site_key" class="mobile-input" :placeholder="siteKeyPlaceholder" :disabled="saving" />
        </label>
        <label class="mobile-field">
          <span>Secret Key</span>
          <input v-model="form.secret" class="mobile-input" type="password" :placeholder="secretPlaceholder" :disabled="saving" />
          <small class="mobile-muted">{{ hasSecret ? '已设置 Secret，留空会保留原值。' : '留空时会回退环境变量中的 Secret。' }}</small>
        </label>
        <button class="mobile-btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存配置' }}</button>
      </template>
    </section>

    <section class="mobile-card">
      <h3 class="mobile-card-title">使用说明</h3>
      <div class="mobile-tip-list">
        <p>Turnstile：在 Cloudflare 控制台创建 Turnstile widget，建议选择 Managed 模式。</p>
        <p>hCaptcha：在 hCaptcha 控制台创建 site，获取对应 Site Key 和 Secret Key。</p>
        <p>配置保存后立即生效；关闭开关后，前端和后端都会停止校验 token。</p>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

const loading = ref(true)
const saving = ref(false)
const hasSecret = ref(false)
const form = ref({
  enabled: false,
  provider: 'turnstile',
  site_key: '',
  secret: '',
})

const siteKeyPlaceholder = computed(() => form.value.provider === 'hcaptcha' ? '10000000-ffff-ffff-ffff-000000000001' : '0x4AAAAAAB...')
const secretPlaceholder = computed(() => form.value.provider === 'hcaptcha' ? '0x0000000000000000000000000000000000000000' : '0x4AAAAAAB...')

async function loadConfig() {
  loading.value = true
  const res = await adminApi<any>('get_captcha_config')
  loading.value = false
  if (res.code === 200 && res.data) {
    form.value.enabled = Boolean(res.data.enabled)
    form.value.provider = res.data.provider || 'turnstile'
    form.value.site_key = res.data.site_key || ''
    form.value.secret = res.data.secret || ''
    hasSecret.value = Boolean(res.data.has_secret)
  } else {
    showToast(res.msg || '加载失败')
  }
}

async function save() {
  if (form.value.enabled && !form.value.site_key.trim()) return showToast('启用时必须填写 Site Key')
  saving.value = true
  const res = await adminApi('save_captcha_config', {
    enabled: form.value.enabled ? '1' : '0',
    provider: form.value.provider,
    site_key: form.value.site_key.trim(),
    secret: form.value.secret.trim(),
  })
  saving.value = false
  if (res.code === 200) {
    showToast(res.msg || '保存成功', 'success')
    if (form.value.secret && form.value.secret !== '********') {
      form.value.secret = '********'
      hasSecret.value = true
    }
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.mobile-switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border-radius: 16px;
  background: var(--control-bg);
}
.mobile-switch-row span {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}
.mobile-switch-row strong {
  font-size: 14px;
}
.mobile-switch-row small,
.mobile-tip-list {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}
.mobile-switch-row input {
  flex: 0 0 auto;
  width: 22px;
  height: 22px;
  accent-color: #EC4141;
}
.mobile-tip-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.mobile-tip-list p {
  margin: 0;
  word-break: break-word;
}
</style>
