<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">配置文件</h3>
      <p class="mobile-muted">管理服务端 config.json。数据库连接、监听地址和密钥类配置保存后需要重启服务端才会完全生效。</p>
      <div class="mobile-actions">
        <button class="mobile-btn" :disabled="loading || saving" @click="load">刷新</button>
        <button class="mobile-btn" :disabled="loading || saving || migrating" @click="migrateCache">{{ migrating ? '迁移中...' : '迁移本地缓存' }}</button>
        <button class="mobile-btn primary" :disabled="loading || saving" @click="save">{{ saving ? '保存中...' : '保存配置' }}</button>
      </div>
    </section>

    <section class="mobile-card warning-card">
      <strong>安全提示</strong>
      <span>密码和密钥不会明文回显，留空表示保留原值。</span>
    </section>

    <transition name="expand">
      <section v-if="migrateResult" class="mobile-card">
        <h3 class="mobile-card-title">最近一次迁移结果</h3>
        <div class="mobile-grid">
          <div class="mobile-stat"><span>用户成功</span><strong>{{ migrateResult.users.ok }}</strong></div>
          <div class="mobile-stat"><span>设置成功</span><strong>{{ migrateResult.settings.ok }}</strong></div>
          <div class="mobile-stat"><span>反馈成功</span><strong>{{ migrateResult.feedback.ok }}</strong></div>
          <div class="mobile-stat"><span>需重启</span><strong>{{ migrateResult.need_restart ? '是' : '否' }}</strong></div>
        </div>
      </section>
    </transition>

    <div v-if="loading" class="mobile-empty">加载中...</div>

    <template v-else>
      <section class="mobile-card mobile-form">
        <h3 class="mobile-card-title">数据库连接</h3>
        <label class="mobile-field"><span>数据库主机</span><input v-model.trim="form.db_host" class="mobile-input" placeholder="127.0.0.1" /></label>
        <label class="mobile-field"><span>数据库端口</span><input v-model.number="form.db_port" class="mobile-input" type="number" min="1" max="65535" placeholder="3306" /></label>
        <label class="mobile-field"><span>数据库名</span><input v-model.trim="form.db_name" class="mobile-input" placeholder="chexian" /></label>
        <label class="mobile-field"><span>数据库用户名</span><input v-model.trim="form.db_user" class="mobile-input" placeholder="chexian" /></label>
        <label class="mobile-field"><span>数据库密码</span><input v-model="form.db_pass" class="mobile-input" type="password" :placeholder="flags.has_db_pass ? '********（留空保留原值）' : '请输入数据库密码'" /></label>
        <label class="mobile-field"><span>字符集</span><input v-model.trim="form.db_charset" class="mobile-input" placeholder="utf8mb4" /></label>
      </section>

      <section class="mobile-card mobile-form">
        <h3 class="mobile-card-title">服务基础配置</h3>
        <label class="mobile-field"><span>监听地址</span><input v-model.trim="form.listen_addr" class="mobile-input" placeholder="0.0.0.0:8081" /></label>
        <label class="mobile-field"><span>后台静态目录</span><input v-model.trim="form.static_dir" class="mobile-input" placeholder="../admin-web/dist" /></label>
        <label class="mobile-field"><span>签名时间容差（秒）</span><input v-model.number="form.api_timestamp_tolerance" class="mobile-input" type="number" min="1" max="86400" /></label>
        <label class="mobile-check-row">
          <span>本地调试免数据库</span>
          <input v-model="form.local_debug_no_db" type="checkbox" />
        </label>
      </section>

      <section class="mobile-card mobile-form">
        <h3 class="mobile-card-title">管理员与密钥</h3>
        <label class="mobile-field"><span>初始管理员用户名</span><input v-model.trim="form.admin_username" class="mobile-input" placeholder="admin" /></label>
        <label class="mobile-field"><span>初始管理员密码</span><input v-model="form.admin_password" class="mobile-input" type="password" :placeholder="flags.has_admin_password ? '********（留空保留原值）' : '请输入初始管理员密码'" /></label>
        <label class="mobile-field"><span>API 签名密钥</span><input v-model="form.api_secret" class="mobile-input" type="password" :placeholder="flags.has_api_secret ? '********（留空保留原值）' : '请输入 API 密钥'" /></label>
        <label class="mobile-field"><span>JWT 密钥</span><input v-model="form.jwt_secret" class="mobile-input" type="password" :placeholder="flags.has_jwt_secret ? '********（留空保留原值）' : '请输入 JWT 密钥'" /></label>
        <p class="mobile-muted">修改 API/JWT 密钥后，客户端签名和后台登录态可能需要同步更新或重新登录。</p>
      </section>

      <section class="mobile-card mobile-form">
        <h3 class="mobile-card-title">邮件默认配置</h3>
        <label class="mobile-field"><span>邮箱 API 主地址</span><input v-model.trim="form.email_api_primary" class="mobile-input" placeholder="https://mail-api.example.com/send" /></label>
        <label class="mobile-field"><span>邮箱 API 备用地址</span><input v-model.trim="form.email_api_backup" class="mobile-input" placeholder="https://backup-mail-api.example.com/send" /></label>
        <label class="mobile-field"><span>发件邮箱</span><input v-model.trim="form.email_sender" class="mobile-input" placeholder="no-reply@example.com" /></label>
        <label class="mobile-field"><span>发件邮箱密码</span><input v-model="form.email_password" class="mobile-input" type="password" :placeholder="flags.has_email_password ? '********（留空保留原值）' : '留空表示未配置'" /></label>
      </section>

      <section class="mobile-card mobile-form">
        <h3 class="mobile-card-title">人机验证密钥</h3>
        <label class="mobile-field"><span>通用验证 Secret</span><input v-model="form.captcha_secret" class="mobile-input" type="password" :placeholder="flags.has_captcha_secret ? '********（留空保留原值）' : '留空表示未配置'" /></label>
        <label class="mobile-field"><span>Turnstile Secret</span><input v-model="form.turnstile_secret" class="mobile-input" type="password" :placeholder="flags.has_turnstile_secret ? '********（留空保留原值）' : '留空表示未配置'" /></label>
        <label class="mobile-field"><span>hCaptcha Secret</span><input v-model="form.hcaptcha_secret" class="mobile-input" type="password" :placeholder="flags.has_hcaptcha_secret ? '********（留空保留原值）' : '留空表示未配置'" /></label>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'

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
    if (res.code === 200 && res.data) assignConfig(res.data)
    else showToast(res.msg || '配置加载失败')
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
  if (!Number.isFinite(Number(form.db_port)) || Number(form.db_port) < 1 || Number(form.db_port) > 65535) return '数据库端口必须在 1-65535 之间'
  if (!Number.isFinite(Number(form.api_timestamp_tolerance)) || Number(form.api_timestamp_tolerance) < 1) return '签名时间容差必须大于 0'
  return ''
}

async function save() {
  const msg = validate()
  if (msg) return showToast(msg)
  if (!(await mobileConfirm('确定保存服务端配置文件吗？数据库连接、监听地址和密钥类配置需要重启服务端后才会完全生效。'))) return
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
  if (!(await mobileConfirm('迁移前请先保存正确的数据库连接配置。确定要把本地缓存数据迁移到数据库吗？'))) return
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
.warning-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: rgba(236, 65, 65, 0.06);
}
.warning-card strong {
  color: #EC4141;
  font-size: 13px;
}
.warning-card span {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}
.mobile-check-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border-radius: 14px;
  background: var(--control-bg);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 800;
}
.mobile-check-row input {
  width: 22px;
  height: 22px;
  accent-color: #EC4141;
}
</style>
