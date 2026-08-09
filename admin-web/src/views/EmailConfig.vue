<template>
  <div class="email-config-page">
    <div class="page-header">
      <div>
        <h2 class="page-title">邮箱机设置</h2>
        <p class="page-desc">
          配置服务端内置邮箱机，支持内置投递、外部 HTTP API 和标准 SMTP 三种方案。留空字段使用环境变量默认值，保存后立即生效。
        </p>
      </div>
      <button class="btn-save" :disabled="saving" @click="save">
        <span v-if="saving" class="spinner"></span>
        {{ saving ? '保存中...' : '保存配置' }}
      </button>
    </div>

    <div v-if="loading" class="state-box">
      <span class="loader"></span>
      加载中...
    </div>

    <template v-else>
      <!-- 发送方式选择 -->
      <div class="config-card">
        <div class="section-title">发送方式</div>
        <div class="provider-toggle">
          <label class="provider-option" :class="{ active: form.email_provider === 'builtin' }">
            <input v-model="form.email_provider" type="radio" value="builtin" />
            <span class="provider-label">内置邮箱机</span>
            <span class="provider-desc">服务端统一记录、调度和回退，优先走 SMTP 出口，失败后可回退外部 API</span>
          </label>
          <label class="provider-option" :class="{ active: form.email_provider === 'http_api' }">
            <input v-model="form.email_provider" type="radio" value="http_api" />
            <span class="provider-label">外部 HTTP API</span>
            <span class="provider-desc">通过第三方邮箱 API 服务发送，作为内置邮箱机的备选通道</span>
          </label>
          <label class="provider-option" :class="{ active: form.email_provider === 'smtp' }">
            <input v-model="form.email_provider" type="radio" value="smtp" />
            <span class="provider-label">标准 SMTP</span>
            <span class="provider-desc">直接通过 SMTP 协议发送（QQ邮箱、Gmail、企业邮等）</span>
          </label>
        </div>
      </div>

      <!-- 通用配置 -->
      <div class="config-card">
        <div class="section-title">通用配置</div>
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
        <p class="hint">
          外部 API 模式下作为 API 认证凭证；SMTP/内置邮箱机模式下，若 SMTP 密码留空则使用此密码。
        </p>
      </div>

      <!-- HTTP API 配置 -->
      <div v-if="form.email_provider === 'builtin' || form.email_provider === 'http_api'" class="config-card">
        <div class="section-title">{{ form.email_provider === 'builtin' ? '外部 API 备选' : 'HTTP API 地址' }}</div>
        <div class="field-grid">
          <label class="field">
            <span :class="{ required: form.email_provider === 'http_api' }">主地址</span>
            <input v-model="form.email_api_primary" type="text" placeholder="https://mail-api.example.com/send" />
          </label>
          <label class="field">
            <span>备用地址</span>
            <input v-model="form.email_api_backup" type="text" placeholder="https://backup-mail-api.example.com/send" />
          </label>
        </div>
        <p class="hint">
          {{ form.email_provider === 'builtin' ? '内置邮箱机在 SMTP 出口失败时，可回退到这里配置的外部邮箱机 API。' : '主地址请求失败时自动回退到备用地址。' }}
        </p>
      </div>

      <!-- SMTP 配置 -->
      <div v-if="form.email_provider === 'builtin' || form.email_provider === 'smtp'" class="config-card">
        <div class="section-title">{{ form.email_provider === 'builtin' ? '内置邮箱机 SMTP 出口' : 'SMTP 服务器' }}</div>
        <div class="field-grid">
          <label class="field">
            <span class="required">SMTP 服务器地址</span>
            <input v-model="form.smtp_host" type="text" placeholder="smtp.qq.com" />
          </label>
          <label class="field">
            <span class="required">SMTP 端口</span>
            <input v-model="form.smtp_port" type="number" placeholder="465" />
          </label>
        </div>

        <div class="section-title">SMTP 认证</div>
        <div class="field-grid">
          <label class="field">
            <span class="required">SMTP 用户名</span>
            <input v-model="form.smtp_username" type="text" placeholder="通常与发件邮箱相同" />
          </label>
          <label class="field">
            <span class="required">SMTP 密码 / 授权码</span>
            <input
              v-model="form.smtp_password"
              type="password"
              :placeholder="hasSmtpPassword ? '********（留空保留原值）' : '留空则使用通用密码'"
            />
          </label>
        </div>
        <p class="hint">
          端口 465 使用隐式 SSL/TLS 加密；端口 587 或 25 使用 STARTTLS。内置邮箱机优先使用这里的 SMTP 出口投递。
        </p>
      </div>

      <!-- SMTP 账号池 -->
      <div v-if="form.email_provider === 'builtin'" class="config-card">
        <div class="section-head">
          <div>
            <div class="section-title">SMTP 账号池</div>
            <p class="hint pool-hint">
              配置多个发件邮箱后，内置邮箱机会按发送次数轮流选择账号，降低单个邮箱触发风控的概率。账号池为空时继续使用上面的单 SMTP 出口。
            </p>
          </div>
          <button type="button" class="btn-add-account" @click="addSmtpAccount">添加邮箱</button>
        </div>

        <div v-if="form.smtp_accounts.length === 0" class="empty-pool">
          暂未配置账号池，当前仍使用单 SMTP 出口发送。
        </div>

        <div v-for="(account, index) in form.smtp_accounts" :key="index" class="smtp-account-card">
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
              <input v-model="account.sender" type="text" placeholder="no-reply@example.com" />
            </label>
            <label class="field">
              <span>备注</span>
              <input v-model="account.remark" type="text" placeholder="例如：QQ邮箱一号" />
            </label>
            <label class="field">
              <span class="required">SMTP 服务器地址</span>
              <input v-model="account.host" type="text" placeholder="smtp.qq.com" />
            </label>
            <label class="field">
              <span class="required">SMTP 端口</span>
              <input v-model="account.port" type="number" placeholder="465" />
            </label>
            <label class="field">
              <span class="required">SMTP 用户名</span>
              <input v-model="account.username" type="text" placeholder="通常与发件邮箱相同" />
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
        </div>
      </div>
    </template>

    <!-- 测试邮件 -->
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
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface EmailConfigForm {
  email_provider: string
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

const loading = ref(true)
const saving = ref(false)
const hasPassword = ref(false)
const hasSmtpPassword = ref(false)
const form = ref<EmailConfigForm>({
  email_provider: 'builtin',
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
    email_provider: string
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
      email_provider: res.data.email_provider || 'builtin',
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
  form.value.smtp_accounts.push({
    sender: '',
    host: form.value.smtp_host || '',
    port: form.value.smtp_port || '465',
    username: '',
    password: '',
    enabled: true,
    remark: '',
  })
}

function removeSmtpAccount(index: number) {
  form.value.smtp_accounts.splice(index, 1)
}

async function save() {
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
    showToast('邮箱机配置已保存', 'success')
    await loadConfig()
  } else {
    showToast(res.msg || '保存失败')
  }
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
.provider-toggle {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
.provider-option {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
  border: 2px solid var(--border);
  border-radius: 14px;
  cursor: pointer;
  transition: all 0.2s;
  background: #fafafa;
}
.provider-option:hover {
  border-color: var(--accent);
}
.provider-option.active {
  border-color: var(--accent);
  background: rgba(var(--accent-rgb, 99, 102, 241), 0.05);
}
.provider-option input {
  display: none;
}
.provider-label {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
}
.provider-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
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
  background: var(--white);
}
.hint {
  margin: 22px 0 0;
  font-size: 12px;
  color: var(--text-muted);
}
.pool-hint {
  margin-top: 0;
  line-height: 1.6;
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
  background: var(--white);
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
  .provider-toggle {
    grid-template-columns: 1fr;
  }
}
</style>
