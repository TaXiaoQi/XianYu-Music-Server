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

    <section class="mobile-card mobile-form audit-policy-card">
      <div class="policy-head">
        <h3 class="mobile-card-title">外部审核策略</h3>
        <button class="pill-switch" :class="{ on: auditConfig.enabled }" @click="auditConfig.enabled = !auditConfig.enabled">
          <span class="pill-knob"></span>
          <span class="pill-text">{{ auditConfig.enabled ? '已启用' : '未启用' }}</span>
        </button>
      </div>
      <p class="mobile-muted">开启后，昵称、头像、壁纸会先走外部机审；机审无法判断或服务失败时再进入人工审核队列。</p>
      <label class="mobile-field">
        <span>服务类型</span>
        <select v-model="auditConfig.provider" class="mobile-select">
          <option value="generic">通用 HTTP</option>
          <option value="aliyun">阿里云内容安全</option>
          <option value="tencent">腾讯云内容安全</option>
        </select>
      </label>
      <label class="mobile-field">
        <span>审核接口地址</span>
        <input v-model.trim="auditConfig.endpoint" class="mobile-input" placeholder="https://example.com/audit" />
      </label>
      <label class="mobile-field">
        <span>接口密钥</span>
        <input v-model.trim="auditConfig.api_key" class="mobile-input" type="password" placeholder="留空则保留原密钥" />
      </label>
      <label class="mobile-field">
        <span>超时时间 (ms)</span>
        <input v-model.number="auditConfig.timeout_ms" class="mobile-input" type="number" min="1000" max="30000" />
      </label>
      <div class="opt-grid">
        <label class="check-row"><input v-model="auditConfig.nickname_enabled" type="checkbox" /> 改名机审</label>
        <label class="check-row"><input v-model="auditConfig.avatar_enabled" type="checkbox" /> 头像机审</label>
        <label class="check-row"><input v-model="auditConfig.wallpaper_enabled" type="checkbox" /> 壁纸机审</label>
        <label class="check-row"><input v-model="auditConfig.fail_to_manual" type="checkbox" /> 失败转人工</label>
      </div>
      <input v-model.trim="auditTestText" class="mobile-input" placeholder="测试文本，例如：测试昵称" />
      <div class="mobile-actions">
        <button class="mobile-btn primary" :disabled="auditSaving" @click="saveAuditConfig">{{ auditSaving ? '保存中...' : '保存策略' }}</button>
        <button class="mobile-btn" :disabled="auditTesting" @click="testAuditConfig">{{ auditTesting ? '测试中...' : '测试连接' }}</button>
      </div>
      <p class="mobile-muted policy-tip">通用 HTTP 服务需返回 <code>{"decision":"pass|reject|review","reason":"原因"}</code>。阿里云可先通过网关或函数计算适配为该格式。</p>
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

// ===== 外部审核策略 =====
const auditSaving = ref(false)
const auditTesting = ref(false)
const auditTestText = ref('测试昵称')
const auditConfig = ref({
  enabled: false,
  provider: 'generic',
  endpoint: '',
  api_key: '',
  nickname_enabled: true,
  avatar_enabled: true,
  wallpaper_enabled: true,
  timeout_ms: 5000,
  fail_to_manual: true,
})

async function loadAuditConfig() {
  const res = await adminApi<any>('get_audit_external_config')
  if (res.code === 200 && res.data) {
    Object.assign(auditConfig.value, res.data, { api_key: '' })
  }
}

async function saveAuditConfig() {
  auditSaving.value = true
  const res = await adminApi<any>('save_audit_external_config', auditConfig.value as any)
  auditSaving.value = false
  if (res.code === 200) {
    auditConfig.value.api_key = ''
    showToast('外部审核策略已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function testAuditConfig() {
  auditTesting.value = true
  const res = await adminApi<any>('test_audit_external_config', { text: auditTestText.value || '测试昵称' })
  auditTesting.value = false
  if (res.code === 200 && res.data) {
    const label = res.data.decision === 'pass' ? '通过' : res.data.decision === 'reject' ? '拒绝' : '转人工'
    showToast(`测试结果：${label}${res.data.reason ? '，' + res.data.reason : ''}`, 'success')
  } else {
    showToast(res.msg || '测试失败')
  }
}

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

onMounted(() => {
  loadConfig()
  loadAuditConfig()
})
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

/* ===== 外部审核策略 ===== */
.audit-policy-card { margin-top: 4px; }
.policy-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 6px;
}
.policy-head .mobile-card-title { margin: 0; }
.pill-switch {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  cursor: pointer;
  transition: all 0.2s;
}
.pill-switch .pill-knob {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
  transition: all 0.2s;
}
.pill-switch .pill-text { font-size: 12px; font-weight: 800; color: var(--text-light); transition: color 0.2s; }
.pill-switch.on { background: var(--accent-soft); border-color: var(--accent); }
.pill-switch.on .pill-knob { background: #16a34a; }
.pill-switch.on .pill-text { color: var(--accent); }
.opt-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 12px;
  margin: 4px 0 12px;
}
.check-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}
.check-row input { accent-color: #EC4141; }
.policy-tip code { color: var(--accent); font-size: 11px; }
.mobile-actions {
  display: flex;
  gap: 10px;
  margin-top: 4px;
}
</style>
