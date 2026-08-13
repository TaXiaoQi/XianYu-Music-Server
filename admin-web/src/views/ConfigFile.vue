<template>
  <div class="config-file-page">
    <!-- 页面头部动效 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">配置文件管理</h2>
        <p class="page-desc">
          管理服务端 config.json 中的基础配置。数据库连接、监听地址和密钥类配置保存后需要重启服务端才会完全生效。
        </p>
      </div>
      <div class="header-actions">
        <button class="btn-secondary" :disabled="loading || saving" @click="load">刷新</button>
        <button class="btn-secondary" :disabled="loading || saving || migrating" @click="migrateCache">
          <span v-if="migrating" class="spinner"></span>
          {{ migrating ? '迁移中...' : '迁移本地缓存' }}
        </button>
        <button class="btn-primary" :disabled="loading || saving" @click="save">
          <span v-if="saving" class="spinner"></span>
          {{ saving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </div>
    </Transition>

    <!-- 主要内容动效 -->
    <Transition name="fade-up" appear>
      <div class="config-content">
        <div class="notice-card">
      <strong>安全提示</strong>
      <span>密码和密钥不会明文回显。对应输入框留空表示保留原值，只有重新填写时才会覆盖。</span>
    </div>

    <div v-if="migrateResult" class="migrate-card">
      <div class="section-title">最近一次迁移结果</div>
      <div class="migrate-grid">
        <div>
          <strong>{{ migrateResult.users.ok }}</strong>
          <span>用户成功</span>
          <small v-if="migrateResult.users.failed">失败 {{ migrateResult.users.failed }}</small>
        </div>
        <div>
          <strong>{{ migrateResult.settings.ok }}</strong>
          <span>设置成功</span>
          <small v-if="migrateResult.settings.failed">失败 {{ migrateResult.settings.failed }}</small>
        </div>
        <div>
          <strong>{{ migrateResult.feedback.ok }}</strong>
          <span>反馈成功</span>
          <small v-if="migrateResult.feedback.failed">失败 {{ migrateResult.feedback.failed }}</small>
        </div>
      </div>
      <p class="hint">迁移完成后请重启服务端，重启后会按当前 config.json 连接数据库。</p>
    </div>

    <div v-if="loading" class="state-box">
      <span class="loader"></span>
      加载中...
    </div>

    <template v-else>
      <section class="config-card">
        <div class="section-title">数据库连接</div>
        <div class="field-grid">
          <label class="field">
            <span class="required">数据库主机</span>
            <input v-model.trim="form.db_host" type="text" placeholder="127.0.0.1" />
          </label>
          <label class="field">
            <span class="required">数据库端口</span>
            <input v-model.number="form.db_port" type="number" min="1" max="65535" placeholder="3306" />
          </label>
          <label class="field">
            <span class="required">数据库名</span>
            <input v-model.trim="form.db_name" type="text" placeholder="chexian" />
          </label>
          <label class="field">
            <span class="required">数据库用户名</span>
            <input v-model.trim="form.db_user" type="text" placeholder="chexian" />
          </label>
          <label class="field">
            <span class="required">数据库密码</span>
            <input
              v-model="form.db_pass"
              type="password"
              :placeholder="flags.has_db_pass ? '********（留空保留原值）' : '请输入数据库密码'"
            />
          </label>
          <label class="field">
            <span class="required">字符集</span>
            <input v-model.trim="form.db_charset" type="text" placeholder="utf8mb4" />
          </label>
        </div>
      </section>

      <section class="config-card">
        <div class="section-title">服务基础配置</div>
        <div class="field-grid">
          <label class="field">
            <span class="required">监听地址</span>
            <input v-model.trim="form.listen_addr" type="text" placeholder="0.0.0.0:8081" />
          </label>
          <label class="field">
            <span class="required">后台静态目录</span>
            <input v-model.trim="form.static_dir" type="text" placeholder="../admin-web/dist" />
          </label>
          <label class="field">
            <span class="required">签名时间容差（秒）</span>
            <input v-model.number="form.api_timestamp_tolerance" type="number" min="1" max="86400" />
          </label>
          <label class="field checkbox-field">
            <span>本地调试免数据库</span>
            <label class="switch-line">
              <input v-model="form.local_debug_no_db" type="checkbox" />
              <span>{{ form.local_debug_no_db ? '已开启' : '已关闭' }}</span>
            </label>
          </label>
        </div>
      </section>

      <section class="config-card">
        <div class="section-title">管理员与密钥</div>
        <div class="field-grid">
          <label class="field">
            <span class="required">初始管理员用户名</span>
            <input v-model.trim="form.admin_username" type="text" placeholder="admin" />
          </label>
          <label class="field">
            <span class="required">初始管理员密码</span>
            <input
              v-model="form.admin_password"
              type="password"
              :placeholder="flags.has_admin_password ? '********（留空保留原值）' : '请输入初始管理员密码'"
            />
          </label>
          <label class="field">
            <span class="required">API 签名密钥</span>
            <input
              v-model="form.api_secret"
              type="password"
              :placeholder="flags.has_api_secret ? '********（留空保留原值）' : '请输入 API 密钥'"
            />
          </label>
          <label class="field">
            <span class="required">JWT 密钥</span>
            <input
              v-model="form.jwt_secret"
              type="password"
              :placeholder="flags.has_jwt_secret ? '********（留空保留原值）' : '请输入 JWT 密钥'"
            />
          </label>
        </div>
        <p class="hint">修改 API/JWT 密钥后，客户端签名和后台登录态可能需要同步更新或重新登录。</p>
      </section>

      <section class="config-card">
        <div class="section-title">邮件默认配置</div>
        <div class="field-grid">
          <label class="field">
            <span>邮箱 API 主地址</span>
            <input v-model.trim="form.email_api_primary" type="text" placeholder="https://mail-api.example.com/send" />
          </label>
          <label class="field">
            <span>邮箱 API 备用地址</span>
            <input v-model.trim="form.email_api_backup" type="text" placeholder="https://backup-mail-api.example.com/send" />
          </label>
          <label class="field">
            <span>发件邮箱</span>
            <input v-model.trim="form.email_sender" type="text" placeholder="no-reply@example.com" />
          </label>
          <label class="field">
            <span>发件邮箱密码</span>
            <input
              v-model="form.email_password"
              type="password"
              :placeholder="flags.has_email_password ? '********（留空保留原值）' : '留空表示未配置'"
            />
          </label>
        </div>
        <p class="hint">这里是 config.json 的默认值；邮箱机设置页面保存到数据库的配置仍会优先生效。</p>
      </section>

      <section class="config-card">
        <div class="section-title">人机验证密钥</div>
        <div class="field-grid">
          <label class="field">
            <span>通用验证 Secret</span>
            <input
              v-model="form.captcha_secret"
              type="password"
              :placeholder="flags.has_captcha_secret ? '********（留空保留原值）' : '留空表示未配置'"
            />
          </label>
          <label class="field">
            <span>Turnstile Secret</span>
            <input
              v-model="form.turnstile_secret"
              type="password"
              :placeholder="flags.has_turnstile_secret ? '********（留空保留原值）' : '留空表示未配置'"
            />
          </label>
          <label class="field">
            <span>hCaptcha Secret</span>
            <input
              v-model="form.hcaptcha_secret"
              type="password"
              :placeholder="flags.has_hcaptcha_secret ? '********（留空保留原值）' : '留空表示未配置'"
            />
          </label>
        </div>
        <p class="hint">后台“人机验证设置”页面保存到数据库的配置通常优先生效，这里用于维护配置文件默认值。</p>
      </section>
    </template>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

interface ServerConfigForm {
  db_host: string
  db_port: number
  db_name: string
  db_user: string
  db_pass: string
  db_charset: string
  api_secret: string
  api_timestamp_tolerance: number
  admin_username: string
  admin_password: string
  listen_addr: string
  jwt_secret: string
  email_api_primary: string
  email_api_backup: string
  email_sender: string
  email_password: string
  captcha_secret: string
  turnstile_secret: string
  hcaptcha_secret: string
  static_dir: string
  local_debug_no_db: boolean
}

interface SecretFlags {
  has_db_pass: boolean
  has_api_secret: boolean
  has_admin_password: boolean
  has_jwt_secret: boolean
  has_email_password: boolean
  has_captcha_secret: boolean
  has_turnstile_secret: boolean
  has_hcaptcha_secret: boolean
}

interface MigrateCount {
  ok: number
  failed: number
}

interface MigrateResult {
  users: MigrateCount
  settings: MigrateCount
  feedback: MigrateCount
  need_restart: boolean
}

const loading = ref(false)
const saving = ref(false)
const migrating = ref(false)
const migrateResult = ref<MigrateResult | null>(null)

const form = reactive<ServerConfigForm>({
  db_host: '',
  db_port: 3306,
  db_name: '',
  db_user: '',
  db_pass: '',
  db_charset: 'utf8mb4',
  api_secret: '',
  api_timestamp_tolerance: 300,
  admin_username: 'admin',
  admin_password: '',
  listen_addr: '0.0.0.0:8081',
  jwt_secret: '',
  email_api_primary: '',
  email_api_backup: '',
  email_sender: '',
  email_password: '',
  captcha_secret: '',
  turnstile_secret: '',
  hcaptcha_secret: '',
  static_dir: '../admin-web/dist',
  local_debug_no_db: false,
})

const flags = reactive<SecretFlags>({
  has_db_pass: false,
  has_api_secret: false,
  has_admin_password: false,
  has_jwt_secret: false,
  has_email_password: false,
  has_captcha_secret: false,
  has_turnstile_secret: false,
  has_hcaptcha_secret: false,
})

function assignConfig(data: Partial<ServerConfigForm & SecretFlags>) {
  Object.assign(form, {
    db_host: data.db_host ?? '',
    db_port: Number(data.db_port ?? 3306),
    db_name: data.db_name ?? '',
    db_user: data.db_user ?? '',
    db_pass: '',
    db_charset: data.db_charset ?? 'utf8mb4',
    api_secret: '',
    api_timestamp_tolerance: Number(data.api_timestamp_tolerance ?? 300),
    admin_username: data.admin_username ?? 'admin',
    admin_password: '',
    listen_addr: data.listen_addr ?? '0.0.0.0:8081',
    jwt_secret: '',
    email_api_primary: data.email_api_primary ?? '',
    email_api_backup: data.email_api_backup ?? '',
    email_sender: data.email_sender ?? '',
    email_password: '',
    captcha_secret: '',
    turnstile_secret: '',
    hcaptcha_secret: '',
    static_dir: data.static_dir ?? '../admin-web/dist',
    local_debug_no_db: Boolean(data.local_debug_no_db),
  })

  Object.assign(flags, {
    has_db_pass: Boolean(data.has_db_pass),
    has_api_secret: Boolean(data.has_api_secret),
    has_admin_password: Boolean(data.has_admin_password),
    has_jwt_secret: Boolean(data.has_jwt_secret),
    has_email_password: Boolean(data.has_email_password),
    has_captcha_secret: Boolean(data.has_captcha_secret),
    has_turnstile_secret: Boolean(data.has_turnstile_secret),
    has_hcaptcha_secret: Boolean(data.has_hcaptcha_secret),
  })
}

async function load() {
  loading.value = true
  try {
    const res = await adminApi<Partial<ServerConfigForm & SecretFlags>>('get_server_config_file')
    if (res.code === 200 && res.data) {
      assignConfig(res.data)
    } else {
      showToast(res.msg || '配置加载失败')
    }
  } finally {
    loading.value = false
  }
}

function validate(): string {
  if (!form.db_host.trim()) return '数据库主机不能为空'
  if (!form.db_name.trim()) return '数据库名不能为空'
  if (!form.db_user.trim()) return '数据库用户名不能为空'
  if (!form.db_charset.trim()) return '数据库字符集不能为空'
  if (!form.listen_addr.trim()) return '监听地址不能为空'
  if (!Number.isFinite(Number(form.db_port)) || Number(form.db_port) < 1 || Number(form.db_port) > 65535) {
    return '数据库端口必须在 1-65535 之间'
  }
  if (!Number.isFinite(Number(form.api_timestamp_tolerance)) || Number(form.api_timestamp_tolerance) < 1) {
    return '签名时间容差必须大于 0'
  }
  return ''
}

async function save() {
  const msg = validate()
  if (msg) {
    showToast(msg)
    return
  }
  const ok = await webConfirm('确定保存服务端配置文件吗？数据库连接、监听地址和密钥类配置需要重启服务端后才会完全生效。', { title: '保存配置', confirmText: '确定保存' })
  if (!ok) {
    return
  }
  saving.value = true
  try {
    const res = await adminApi<Partial<ServerConfigForm & SecretFlags>>('save_server_config_file', {
      ...form,
      db_port: Number(form.db_port),
      api_timestamp_tolerance: Number(form.api_timestamp_tolerance),
    })
    if (res.code === 200 && res.data) {
      assignConfig(res.data)
      showToast(res.msg || '保存成功，重启服务端后生效', 'success')
    } else {
      showToast(res.msg || '保存失败')
    }
  } finally {
    saving.value = false
  }
}

async function migrateCache() {
  const ok = await webConfirm('迁移前请先保存正确的数据库连接配置。确定要把本地缓存数据迁移到数据库吗？', { title: '迁移数据', confirmText: '确定迁移' })
  if (!ok) {
    return
  }
  migrating.value = true
  try {
    const res = await adminApi<MigrateResult>('migrate_local_cache_to_database')
    if (res.code === 200 && res.data) {
      migrateResult.value = res.data
      showToast(res.msg || '迁移完成，重启服务端后生效', 'success')
    } else {
      showToast(res.msg || '迁移失败')
    }
  } finally {
    migrating.value = false
  }
}

onMounted(load)
</script>

<style scoped>
.config-file-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}
.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 900;
  color: var(--text);
}
.page-desc {
  margin: 8px 0 0;
  color: var(--text-muted);
  font-size: 14px;
  line-height: 1.7;
}
.header-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}
.notice-card,
.migrate-card,
.config-card,
.state-box {
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
}
.notice-card {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 13px 16px;
  color: var(--text);
  background: rgba(236, 65, 65, 0.06);
}
.notice-card strong {
  color: #EC4141;
}
.notice-card span {
  color: var(--text-muted);
  font-size: 13px;
}
.migrate-card {
  padding: 16px;
}
.migrate-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}
.migrate-grid div {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: color-mix(in srgb, var(--card) 88%, #EC4141 4%);
}
.migrate-grid strong {
  font-size: 24px;
  color: #EC4141;
}
.migrate-grid span {
  font-size: 13px;
  font-weight: 800;
  color: var(--text);
}
.migrate-grid small {
  color: var(--text-muted);
}
.config-card {
  padding: 18px;
}
.section-title {
  margin-bottom: 14px;
  font-size: 16px;
  font-weight: 900;
  color: var(--text);
}
.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.field span {
  font-size: 13px;
  font-weight: 800;
  color: var(--text);
}
.required::after {
  content: ' *';
  color: #EC4141;
}
.field input {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 11px 12px;
  background: var(--input-bg, var(--card));
  color: var(--text);
  outline: none;
}
.field input:focus {
  border-color: #EC4141;
  box-shadow: 0 0 0 3px rgba(236, 65, 65, 0.10);
}
.checkbox-field {
  justify-content: flex-end;
}
.switch-line {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 42px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
}
.switch-line input {
  width: auto;
}
.hint {
  margin: 12px 0 0;
  color: var(--text-muted);
  font-size: 13px;
  line-height: 1.7;
}
.state-box {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  padding: 40px;
  color: var(--text-muted);
}
.btn-primary,
.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 0;
  border-radius: 12px;
  padding: 10px 16px;
  font-weight: 800;
  cursor: pointer;
}
.btn-primary {
  background: #EC4141;
  color: #fff;
}
.btn-secondary {
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--text);
}
.btn-primary:disabled,
.btn-secondary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.spinner,
.loader {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.45);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
.loader {
  border-color: rgba(236, 65, 65, 0.18);
  border-top-color: #EC4141;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
  }
  .header-actions {
    width: 100%;
  }
  .header-actions button {
    flex: 1;
    justify-content: center;
  }
  .field-grid {
    grid-template-columns: 1fr;
  }
  .migrate-grid {
    grid-template-columns: 1fr;
  }
  .notice-card {
    align-items: flex-start;
    flex-direction: column;
  }
}

/* ===== 过渡动画 ===== */
.config-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.config-card { animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes cardIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
</style>
