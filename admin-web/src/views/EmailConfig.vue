<template>
  <div class="email-config-page">
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">邮箱机设置</h2>
        <p class="page-desc">
          配置服务端邮箱机。通用配置、外部 API、SMTP 账号池各自可通过顶部开关启用，开哪个用哪个；多个开启时按顺序依次尝试，发送成功即轮换。SMTP 地址与端口会根据发件邮箱域名自动识别。
        </p>
      </div>
      <button class="btn-save" :disabled="saving" @click="save">
        <span v-if="saving" class="spinner"></span>
        {{ saving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </Transition>

    <div v-if="loading" class="state-box">
      <span class="loader"></span>
      加载中...
    </div>

    <template v-else>
      <!-- 通用配置 -->
      <Transition name="fade-up" appear>
      <div class="config-card">
        <div class="section-head">
          <div class="section-title">通用配置</div>
          <label class="channel-switch">
            <input v-model="form.email_channel_general" type="checkbox" />
            <span class="switch-track"><span class="switch-thumb"></span></span>
            <span class="switch-label">启用 SMTP 投递</span>
          </label>
        </div>
        <div class="field-grid">
          <label class="field">
            <span class="required">发件邮箱地址</span>
            <input v-model="form.email_sender" type="text" placeholder="admin@example.com" />
          </label>
          <label class="field">
            <span class="required">邮箱密码 / 授权码</span>
            <input
              v-model="form.email_password"
              type="password"
              :placeholder="hasPassword ? '********（留空保留原值）' : '请输入密码'"
            />
          </label>
        </div>
        <div class="endpoint-box">
          <span class="endpoint-label">SMTP 服务器（自动识别）</span>
          <code class="endpoint-value">{{ smtpEndpointText(form.email_sender) }}</code>
        </div>
        <p class="hint">
          启用后使用「发件邮箱 + 授权码」作为单一 SMTP 通道投递，SMTP 地址与端口按域名自动识别。
        </p>
      </div>
      </Transition>

      <!-- HTTP API 配置 -->
      <Transition name="fade-up" appear>
      <div class="config-card">
        <div class="section-head">
          <div class="section-title">外部 API</div>
          <label class="channel-switch">
            <input v-model="form.email_channel_api" type="checkbox" />
            <span class="switch-track"><span class="switch-thumb"></span></span>
            <span class="switch-label">启用外部 API 通道</span>
          </label>
        </div>
        <div class="field-grid">
          <label class="field">
            <span>主地址</span>
            <input v-model="form.email_api_primary" type="text" placeholder="https://mail-api.example.com/send" />
          </label>
          <label class="field">
            <span>备用地址</span>
            <input v-model="form.email_api_backup" type="text" placeholder="https://backup-mail-api.example.com/send" />
          </label>
        </div>
        <p class="hint">
          启用后作为一条投递通道参与尝试与轮换，主地址请求失败时自动回退备用地址。
        </p>
      </div>
      </Transition>

      <!-- SMTP 账号池 -->
      <Transition name="fade-up" appear>
      <div class="config-card">
        <div class="section-head">
          <div>
            <div class="section-title">SMTP 账号池</div>
            <p class="hint pool-hint">
              启用后每个可用账号可作为独立投递通道轮流发送，降低单个邮箱触发风控的概率。每个账号的 SMTP 地址与端口按发件邮箱域名自动识别，只需填写发件邮箱和授权码。
            </p>
          </div>
          <label class="channel-switch">
            <input v-model="form.email_channel_pool" type="checkbox" />
            <span class="switch-track"><span class="switch-thumb"></span></span>
            <span class="switch-label">启用账号池</span>
          </label>
        </div>
          <div class="pool-actions">
            <button type="button" class="btn-save-pool" :disabled="saving" @click="savePool">
              {{ saving ? '保存中...' : '保存邮箱池' }}
            </button>
            <button type="button" class="btn-add-account" @click="addSmtpAccount">添加邮箱</button>
          </div>

        <div v-if="form.smtp_accounts.length === 0" class="empty-pool">
          暂未配置账号池，开启后每条账号将作为独立投递通道参与轮换。
        </div>

        <div
          v-for="(account, index) in form.smtp_accounts"
          :key="index"
          class="smtp-account-card"
          :style="{ animationDelay: `${index * 40}ms` }"
        >
          <div class="account-head">
            <strong>发件账号 {{ index + 1 }}</strong>
            <label class="enabled-line">
              <input v-model="account.enabled" type="checkbox" />
              启用
            </label>
            <button type="button" class="btn-remove-account" @click="removeSmtpAccount(index)">删除</button>
          </div>

          <div class="field-grid">
            <label class="field">
              <span class="required">发件邮箱</span>
              <input v-model="account.sender" type="text" placeholder="no-reply@qq.com" @input="syncAccountEndpoint(account)" />
            </label>
            <label class="field">
              <span>备注</span>
              <input v-model="account.remark" type="text" placeholder="例如：QQ邮箱一号" />
            </label>
            <label class="field">
              <span>SMTP 用户名</span>
              <input v-model="account.username" type="text" placeholder="留空则使用发件邮箱" />
            </label>
            <label class="field">
              <span class="required">SMTP 密码 / 授权码</span>
              <input
                v-model="account.password"
                type="password"
                :placeholder="account.has_password ? '********（留空保留原值）' : '请输入授权码'"
              />
            </label>
          </div>
          <div class="endpoint-box endpoint-box--account">
            <span class="endpoint-label">SMTP 服务器（自动识别）</span>
            <code class="endpoint-value">{{ smtpEndpointText(account.sender) }}</code>
          </div>
        </div>
      </div>
      </Transition>
    </template>

    <!-- 测试邮件 -->
    <Transition name="fade-up" appear>
      <div v-if="!loading" class="config-card test-card">
        <div class="section-title">发送测试邮件</div>
      <p class="test-desc">使用当前已保存的邮箱机配置向指定邮箱发送一封测试邮件，验证配置是否正常。</p>
      <label class="test-label required">测试收件邮箱</label>
      <div class="test-row">
        <input
          v-model="testEmail"
          type="text"
          placeholder="输入测试收件邮箱地址"
          class="test-input"
          @keydown.enter="sendTest"
        />
        <button class="btn-test" :disabled="testing" @click="sendTest">
          <span v-if="testing" class="spinner"></span>
          {{ testing ? '发送中...' : '发送测试' }}
        </button>
      </div>
      <p v-if="testResult" :class="['test-result', testResultType]">{{ testResult }}</p>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface EmailConfigForm {
  email_channel_general: boolean
  email_channel_api: boolean
  email_channel_pool: boolean
  email_api_primary: string
  email_api_backup: string
  email_sender: string
  email_password: string
  smtp_host: string
  smtp_port: string
  smtp_username: string
  smtp_password: string
  smtp_accounts: SmtpAccountForm[]
}

interface SmtpAccountForm {
  sender: string
  host: string
  port: string | number
  username: string
  password: string
  has_password?: boolean
  enabled: boolean
  remark: string
}

// 常见邮箱服务商域名 → [SMTP 服务器地址, 端口]。未命中的域名回退为 smtp.<域名>:465
const SMTP_ENDPOINTS: Record<string, [string, number]> = {
  'qq.com': ['smtp.qq.com', 465],
  'vip.qq.com': ['smtp.qq.com', 465],
  'foxmail.com': ['smtp.qq.com', 465],
  'exmail.qq.com': ['smtp.exmail.qq.com', 465],
  '163.com': ['smtp.163.com', 465],
  '126.com': ['smtp.126.com', 465],
  'yeah.net': ['smtp.yeah.net', 465],
  '188.com': ['smtp.188.com', 465],
  'qiye.163.com': ['smtp.qiye.163.com', 465],
  '139.com': ['smtp.139.com', 465],
  '21cn.com': ['smtp.21cn.com', 465],
  'sohu.com': ['smtp.sohu.com', 465],
  'sina.com': ['smtp.sina.com', 465],
  'vip.sina.com': ['smtp.sina.com', 465],
  'aliyun.com': ['smtp.aliyun.com', 465],
  'aliyunmail.com': ['smtp.aliyun.com', 465],
  'qiye.aliyun.com': ['smtp.qiye.aliyun.com', 465],
  '189.cn': ['smtp.189.cn', 465],
  '263.net': ['smtp.263.net', 465],
  'x263.net': ['smtp.263.net', 465],
  'gmail.com': ['smtp.gmail.com', 465],
  'outlook.com': ['smtp.office365.com', 587],
  'hotmail.com': ['smtp.office365.com', 587],
  'hotmail.co.uk': ['smtp.office365.com', 587],
  'live.com': ['smtp.office365.com', 587],
  'msn.com': ['smtp.office365.com', 587],
  'yandex.com': ['smtp.yandex.ru', 465],
  'yandex.ru': ['smtp.yandex.ru', 465],
  'zoho.com': ['smtp.zoho.com', 465],
  'zohomail.com': ['smtp.zoho.com', 465],
}

function smtpEndpoint(sender: string): { host: string; port: number } {
  const domain = (sender.split('@')[1] || '').trim().toLowerCase()
  if (!domain) return { host: '', port: 465 }
  const hit = SMTP_ENDPOINTS[domain]
  if (hit) return { host: hit[0], port: hit[1] }
  return { host: `smtp.${domain}`, port: 465 }
}

function smtpEndpointText(sender: string): string {
  const { host, port } = smtpEndpoint(sender)
  return host ? `${host}:${port}` : '请先填写发件邮箱'
}

function syncAccountEndpoint(account: SmtpAccountForm) {
  const { host, port } = smtpEndpoint(account.sender)
  account.host = host
  account.port = port
}

const loading = ref(true)
const saving = ref(false)
const hasPassword = ref(false)
const hasSmtpPassword = ref(false)
const form = ref<EmailConfigForm>({
  email_channel_general: true,
  email_channel_api: false,
  email_channel_pool: true,
  email_api_primary: '',
  email_api_backup: '',
  email_sender: '',
  email_password: '',
  smtp_host: '',
  smtp_port: '465',
  smtp_username: '',
  smtp_password: '',
  smtp_accounts: [],
})

const testEmail = ref('')
const testing = ref(false)
const testResult = ref('')
const testResultType = ref<'success' | 'error'>('success')

async function loadConfig() {
  loading.value = true
  const res = await adminApi<{
    email_channel_general: boolean
    email_channel_api: boolean
    email_channel_pool: boolean
    email_api_primary: string
    email_api_backup: string
    email_sender: string
    email_password: string
    has_password: boolean
    smtp_host: string
    smtp_port: number
    smtp_username: string
    smtp_password: string
    has_smtp_password: boolean
    smtp_accounts?: SmtpAccountForm[]
  }>('get_email_config')

  if (res.code === 200 && res.data) {
    form.value = {
      email_channel_general: res.data.email_channel_general !== false,
      email_channel_api: res.data.email_channel_api === true,
      email_channel_pool: res.data.email_channel_pool !== false,
      email_api_primary: res.data.email_api_primary || '',
      email_api_backup: res.data.email_api_backup || '',
      email_sender: res.data.email_sender || '',
      email_password: res.data.has_password ? '********' : '',
      smtp_host: res.data.smtp_host || '',
      smtp_port: String(res.data.smtp_port || 465),
      smtp_username: res.data.smtp_username || '',
      smtp_password: res.data.has_smtp_password ? '********' : '',
      smtp_accounts: (res.data.smtp_accounts || []).map(account => ({
        sender: account.sender || '',
        host: account.host || '',
        port: String(account.port || 465),
        username: account.username || '',
        password: account.has_password ? '********' : '',
        has_password: account.has_password,
        enabled: account.enabled !== false,
        remark: account.remark || '',
      })),
    }
    hasPassword.value = res.data.has_password
    hasSmtpPassword.value = res.data.has_smtp_password
  } else {
    showToast(res.msg || '加载配置失败')
  }
  loading.value = false
}

function addSmtpAccount() {
  const { host, port } = smtpEndpoint(form.value.email_sender)
  form.value.smtp_accounts.push({
    sender: '',
    host,
    port,
    username: '',
    password: '',
    enabled: true,
    remark: '',
  })
}

function removeSmtpAccount(index: number) {
  form.value.smtp_accounts.splice(index, 1)
}

async function doSave(successMsg: string) {
  saving.value = true
  const payload = {
    ...form.value,
    smtp_accounts: form.value.smtp_accounts.map(account => ({
      ...account,
      port: Number(account.port || 465),
    })),
  }
  const res = await adminApi('update_email_config', payload)
  saving.value = false

  if (res.code === 200) {
    showToast(successMsg, 'success')
    await loadConfig()
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function save() {
  await doSave('邮箱机配置已保存')
}

async function savePool() {
  await doSave('邮箱池已保存')
}

async function sendTest() {
  if (!testEmail.value.trim()) {
    showToast('请输入测试收件邮箱')
    return
  }
  testing.value = true
  testResult.value = ''
  const res = await adminApi('test_email_config', { email: testEmail.value.trim() })
  testing.value = false

  if (res.code === 200) {
    testResult.value = res.msg || '测试邮件已发送'
    testResultType.value = 'success'
  } else {
    testResult.value = res.msg || '发送失败'
    testResultType.value = 'error'
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.email-config-page {
  max-width: 980px;
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
  max-width: 600px;
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
  background: var(--card-solid);
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
  margin: 22px 0 12px;
}
.section-title:first-child {
  margin-top: 0;
}
.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}
.channel-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  flex-shrink: 0;
  user-select: none;
}
.channel-switch input {
  display: none;
}
.switch-track {
  width: 40px;
  height: 22px;
  border-radius: 999px;
  background: var(--border);
  position: relative;
  transition: background 0.25s;
}
.switch-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--card-solid);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s;
}
.channel-switch input:checked + .switch-track {
  background: var(--accent);
}
.channel-switch input:checked + .switch-track .switch-thumb {
  transform: translateX(18px);
}
.switch-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-light);
}
.provider-toggle {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
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
.field input {
  border: 1.5px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  outline: none;
  background: #fafafa;
  font-size: 14px;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus {
  border-color: var(--accent);
  background: var(--card-solid);
}
.hint {
  margin: 22px 0 0;
  font-size: 12px;
  color: var(--text-muted);
}
.endpoint-box {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 14px;
  border: 1.5px dashed var(--border);
  border-radius: 10px;
  background: #fafafa;
}
.endpoint-box--account {
  margin-top: 12px;
}
.endpoint-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-light);
  white-space: nowrap;
}
.endpoint-value {
  font-size: 13px;
  font-weight: 700;
  color: var(--accent);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  overflow-wrap: anywhere;
}
.pool-hint {
  margin-top: 0;
  line-height: 1.6;
}
.pool-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.btn-save-pool {
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: var(--white);
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  box-shadow: 0 2px 8px rgba(220, 38, 38, 0.18);
}
.btn-save-pool:hover:not(:disabled) {
  filter: brightness(1.08);
}
.btn-save-pool:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-add-account,
.btn-remove-account {
  border: 1.5px solid var(--accent);
  border-radius: 10px;
  background: transparent;
  color: var(--accent);
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
}
.btn-add-account:hover,
.btn-remove-account:hover {
  background: var(--accent);
  color: var(--white);
}
.btn-remove-account {
  border-color: #dc2626;
  color: #dc2626;
}
.btn-remove-account:hover {
  background: #dc2626;
  color: var(--white);
}
.empty-pool {
  border: 1px dashed var(--border);
  border-radius: 12px;
  color: var(--text-muted);
  font-size: 13px;
  padding: 18px;
  background: #fafafa;
}
.smtp-account-card {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px;
  background: #fafafa;
  margin-top: 14px;
}
.account-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}
.account-head strong {
  color: var(--text);
  font-size: 14px;
  margin-right: auto;
}
.enabled-line {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-light);
  font-size: 13px;
  font-weight: 700;
}
.test-card {
  padding: 24px;
}
.test-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 14px;
}
.test-label {
  display: inline-block;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-light);
}
.test-row {
  display: flex;
  gap: 12px;
  align-items: center;
}
.test-input {
  flex: 1;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  outline: none;
  background: #fafafa;
  font-size: 14px;
  transition: border-color 0.2s, background 0.2s;
}
.test-input:focus {
  border-color: var(--accent);
  background: var(--card-solid);
}
.btn-test {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1.5px solid var(--accent);
  border-radius: 10px;
  background: transparent;
  color: var(--accent);
  padding: 10px 18px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-test:hover:not(:disabled) {
  background: var(--accent);
  color: var(--white);
}
.btn-test:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
.test-result {
  margin: 12px 0 0;
  font-size: 13px;
  font-weight: 600;
}
.test-result.success {
  color: #16a34a;
}
.test-result.error {
  color: #dc2626;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
@media (max-width: 760px) {
  .page-header,
  .section-head,
  .field-grid,
  .test-row {
    grid-template-columns: 1fr;
    flex-direction: column;
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* 卡片入场动画 */
.smtp-account-card { animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes cardIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
