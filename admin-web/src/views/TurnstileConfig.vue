<template>
  <div class="turnstile-config-page">
    <!-- 页面头部动效 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">人机验证设置</h2>
        <p class="page-desc">
          配置注册和找回密码验证码接口的人机验证，当前支持 Cloudflare Turnstile 与 hCaptcha。
          在对应平台创建站点后获取 Site Key 和 Secret Key，填入下方并启用即可。
        </p>
      </div>
      <button class="btn-save" :disabled="saving" @click="save">
        <span v-if="saving" class="spinner"></span>
        {{ saving ? '保存中...' : '保存配置' }}
      </button>
    </div>
    </Transition>

    <Transition name="fade-up" appear>
      <div v-if="loading" class="state-box"><span class="loader"></span> 加载中...</div>
      <div v-else class="config-card">
        <div class="section-title">基本设置</div>
        <div class="toggle-row">
          <div>
            <div class="toggle-label">启用人机验证</div>
            <div class="toggle-desc">开启后，注册和找回密码页面发送验证码前需完成所选服务商的人机验证</div>
          </div>
          <label class="switch">
            <input type="checkbox" v-model="form.enabled" />
            <span class="slider"></span>
          </label>
        </div>

        <div class="field-grid" style="margin-top: 20px">
          <div class="field">
            <span>验证服务商 <em class="required">*</em></span>
            <select v-model="form.provider" :disabled="saving">
              <option value="turnstile">Cloudflare Turnstile</option>
              <option value="hcaptcha">hCaptcha</option>
            </select>
            <small class="hint">切换后需填写对应平台的 Site Key 和 Secret Key</small>
          </div>
          <div class="field">
            <span>Site Key <em class="required">*</em></span>
            <input
              v-model="form.site_key"
              type="text"
              :placeholder="siteKeyPlaceholder"
              :disabled="saving"
            />
            <small class="hint">前端展示用，在所选验证服务商控制台获取</small>
          </div>
          <div class="field">
            <span>Secret Key</span>
            <input
              v-model="form.secret"
              type="password"
              :placeholder="secretPlaceholder"
              :disabled="saving"
            />
            <small class="hint">
              后端校验用{{ hasSecret ? '（已设置，留空保留原值）' : '（留空则回退环境变量 CAPTCHA_SECRET / 服务商专用 Secret）' }}
            </small>
          </div>
        </div>
      </div>
      </Transition>

      <Transition name="fade-up" appear>
      <div class="config-card">
        <div class="section-title">使用说明</div>
        <ul class="tips-list">
          <li>Turnstile：在 Cloudflare 控制台创建 Turnstile widget，建议选择 "Managed" 模式。</li>
          <li>hCaptcha：在 hCaptcha 控制台创建 site，获取对应的 Site Key 和 Secret Key。</li>
          <li>勾选"启用人机验证"并保存后，注册页和找回密码页会自动加载验证组件。</li>
          <li>关闭开关后，前端不会显示验证组件，后端也不会校验 token。</li>
          <li>Secret Key 为空时会依次回退环境变量 <code>CAPTCHA_SECRET</code>、<code>TURNSTILE_SECRET</code> 或 <code>HCAPTCHA_SECRET</code>。</li>
          <li>配置保存后立即生效，无需重启服务端。</li>
        </ul>
      </div>
      </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

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
  const res = await adminApi('get_captcha_config')
  loading.value = false

  if (res.code === 200 && res.data) {
    form.value.enabled = res.data.enabled
    form.value.provider = res.data.provider || 'turnstile'
    form.value.site_key = res.data.site_key || ''
    form.value.secret = res.data.secret || ''
    hasSecret.value = res.data.has_secret
  } else {
    showToast(res.msg || '加载失败')
  }
}

async function save() {
  if (!form.value.site_key.trim() && form.value.enabled) {
    showToast('启用时必须填写 Site Key')
    return
  }
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
.turnstile-config-page {
  max-width: 760px;
  margin: 0 auto;
}
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
  margin: 0 0 6px;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.6;
  max-width: 560px;
}
.page-desc a {
  color: var(--accent);
  text-decoration: none;
}
.page-desc a:hover {
  text-decoration: underline;
}
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: var(--white);
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-save:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.14);
}
.btn-save:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
.spinner,
.loader {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.35);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.loader {
  width: 24px;
  height: 24px;
  border-color: var(--border);
  border-top-color: var(--accent);
}
.state-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 220px;
  color: var(--text-muted);
}
.config-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04);
  margin-bottom: 20px;
}
.section-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 16px;
}
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.toggle-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}
.toggle-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
  line-height: 1.5;
}
.switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
  flex-shrink: 0;
}
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #ccc;
  border-radius: 26px;
  transition: 0.3s;
}
.slider::before {
  content: '';
  position: absolute;
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background: white;
  border-radius: 50%;
  transition: 0.3s;
}
.switch input:checked + .slider {
  background: var(--accent);
}
.switch input:checked + .slider::before {
  transform: translateX(22px);
}
.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field input,
.field select {
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--text);
  outline: none;
  transition: all 0.2s;
  background: #fafafa;
}
.field input:focus,
.field select:focus {
  border-color: var(--accent);
  background: var(--white);
  box-shadow: 0 0 0 3px rgba(var(--accent-rgb, 99, 102, 241), 0.08);
}
.field input:disabled,
.field select:disabled {
  opacity: 0.6;
}
.required {
  color: #e74c3c;
  font-style: normal;
}
.hint {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
}
.tips-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  color: var(--text-muted);
  line-height: 2;
}
.tips-list code {
  background: rgba(0, 0, 0, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text);
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.config-card { animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes cardIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
</style>
