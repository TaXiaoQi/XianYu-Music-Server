<template>
  <div class="mobile-page">
    <!-- 管理员账户 -->
    <div class="mobile-card">
      <h3 class="mobile-card-title">管理员账户</h3>
      <div v-if="loading" class="mobile-empty">加载中...</div>
      <div v-else class="mobile-list">
        <div v-for="a in list" :key="a.id" class="mobile-item">
          <div class="mobile-item-head">
            <div class="mobile-admin-left">
              <img v-if="a.avatar_url" :src="a.avatar_url" alt="" class="mobile-avatar" />
              <span v-else class="mobile-avatar mobile-avatar-letter">{{ initialOf(a.username) }}</span>
              <div>
                <div class="mobile-item-title">{{ a.username }}<span v-if="a.id === currentId" class="mobile-self-tag">你</span></div>
                <div class="mobile-item-sub">{{ a.role === 'super_admin' ? '超级管理员' : '管理员' }}</div>
                <div v-if="a.email" class="mobile-item-email">{{ a.email }}</div>
              </div>
            </div>
            <span class="mobile-badge" :class="a.status == 1 ? 'green' : 'red'">{{ a.status == 1 ? '正常' : '禁用' }}</span>
          </div>
          <div class="mobile-actions">
            <button v-if="canUploadAvatar(a)" class="mobile-btn" @click="pickAvatar(a)">改头像</button>
            <button v-if="isSuper || a.id === currentId" class="mobile-btn" @click="openLogin(a)">修改登录</button>
            <button v-if="isSuper && a.id !== currentId" class="mobile-btn" @click="toggle(a)">{{ a.status == 1 ? '禁用' : '启用' }}</button>
            <button v-if="isSuper && a.id !== currentId" class="mobile-btn danger" @click="remove(a)">删除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增管理员（仅超管） -->
    <div v-if="isSuper" class="mobile-card mobile-form">
      <h3 class="mobile-card-title">新增管理员</h3>
      <input v-model="form.username" class="mobile-input" placeholder="用户名" />
      <input v-model="form.password" class="mobile-input" type="password" placeholder="密码" />
      <input v-model="form.email" class="mobile-input" type="email" placeholder="邮箱（可选，用于接收通知）" />
      <select v-model="form.role" class="mobile-select"><option value="admin">管理员</option><option value="super_admin">超级管理员</option></select>
      <button class="mobile-btn primary" @click="add">新增</button>
    </div>

    <input ref="fileInput" type="file" accept="image/jpeg,image/png,image/webp,image/gif" style="display:none" @change="onFileChange" />

    <!-- 修改登录弹窗 -->
    <div v-if="loginModalVisible" class="mobile-dialog-overlay" :class="{ show: true }" @click.self="closeLogin">
      <div class="mobile-dialog show">
        <div class="mobile-dialog-title">修改登录 - {{ loginTarget?.username || '' }}</div>
        <div class="mobile-dialog-body">
          <div class="mobile-login-field">
            <label>新用户名</label>
            <input v-model="loginForm.new_username" class="mobile-dialog-input login-input" type="text" placeholder="请输入新用户名" />
          </div>
          <div class="mobile-login-field">
            <label>邮箱（留空则保持原邮箱）</label>
            <input v-model="loginForm.new_email" class="mobile-dialog-input login-input" type="email" placeholder="请输入接收通知邮箱" />
          </div>
          <div v-if="needOldPassword" class="mobile-login-field">
            <label>当前密码</label>
            <input v-model="loginForm.old_password" class="mobile-dialog-input login-input" type="password" placeholder="请输入当前密码" />
          </div>
          <div class="mobile-login-field">
            <label>新密码（留空则不修改）</label>
            <input v-model="loginForm.new_password" class="mobile-dialog-input login-input" type="password" placeholder="至少 6 个字符" />
          </div>
          <div class="mobile-login-field">
            <label>确认新密码（留空则不修改）</label>
            <input v-model="loginForm.confirm_password" class="mobile-dialog-input login-input" type="password" placeholder="再次输入新密码" />
          </div>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeLogin">取消</button>
          <button class="mobile-dialog-btn confirm" :disabled="loginSaving" @click="submitLogin">{{ loginSaving ? '保存中...' : '保存' }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast, getAdminUser } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const loading = ref(false), list = ref<any[]>([])
const form = ref({ username: '', password: '', email: '', role: 'admin' })
const adminUser = getAdminUser()
const isSuper = adminUser?.role === 'super_admin'
const currentId = adminUser?.id ?? 0
const fileInput = ref<HTMLInputElement | null>(null)
let avatarTarget: any = null

// 修改登录弹窗
const loginModalVisible = ref(false)
const loginSaving = ref(false)
const loginTarget = ref<any>(null)
const loginForm = ref({ new_username: '', new_email: '', old_password: '', new_password: '', confirm_password: '' })
const needOldPassword = computed(() => !!loginTarget.value && loginTarget.value.id === currentId)

function openLogin(a: any) {
  loginTarget.value = a
  loginForm.value = { new_username: a.username, new_email: a.email || '', old_password: '', new_password: '', confirm_password: '' }
  loginModalVisible.value = true
}
function closeLogin() {
  if (loginSaving.value) return
  loginModalVisible.value = false
  loginTarget.value = null
}
async function submitLogin() {
  const target = loginTarget.value
  if (!target) return
  const { new_username, new_email, old_password, new_password, confirm_password } = loginForm.value
  const uname = new_username.trim()
  const email = new_email.trim()
  const isSelf = target.id === currentId
  if (!uname) { showToast('请输入新用户名'); return }
  if (email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) { showToast('邮箱格式不正确'); return }
  if (isSelf && !old_password) { showToast('请输入当前密码'); return }
  if (new_password && new_password.length < 6) { showToast('新密码至少需要 6 个字符'); return }
  if (new_password && new_password !== confirm_password) { showToast('两次输入的新密码不一致'); return }
  loginSaving.value = true
  const payload: Record<string, any> = { new_username: uname, admin_id: target.id }
  if (email) payload.new_email = email
  if (isSelf) payload.old_password = old_password
  if (new_password) { payload.new_password = new_password; payload.confirm_password = confirm_password }
  const res = await adminApi('change_login', payload)
  loginSaving.value = false
  if (res.code === 200) {
    showToast(res.msg || '修改成功', 'success')
    loginModalVisible.value = false
    loginTarget.value = null
    load()
  } else {
    showToast(res.msg || '修改失败')
  }
}

function initialOf(name: string): string {
  return (name || 'A').charAt(0).toUpperCase()
}
// 权限：超管可上传任意管理员头像，普通管理员只能上传自己的
function canUploadAvatar(a: any): boolean {
  if (isSuper) return true
  return a.id === currentId
}
function pickAvatar(a: any) {
  avatarTarget = a
  if (fileInput.value) fileInput.value.value = ''
  fileInput.value?.click()
}
async function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) {
    avatarTarget = null
    return
  }
  if (!/^image\/(jpeg|png|webp|gif)$/i.test(file.type)) {
    showToast('只支持 JPG / PNG / WEBP / GIF 图片')
    avatarTarget = null
    return
  }
  if (file.size > 4 * 1024 * 1024) {
    showToast('图片不能超过 4MB')
    avatarTarget = null
    return
  }
  const reader = new FileReader()
  reader.onload = async () => {
    const target = avatarTarget
    avatarTarget = null
    if (!target) return
    const res = await adminApi('upload_admin_avatar', { admin_id: target.id, image: reader.result as string })
    if (res.code === 200) {
      target.avatar_url = res.data?.avatar_url || ''
      showToast('头像已更新', 'success')
    } else {
      showToast(res.msg || '上传失败')
    }
  }
  reader.readAsDataURL(file)
}
async function load() { loading.value = true; const res = await adminApi<any>('list_admins'); list.value = res.code === 200 && res.data ? (res.data.list || []) : []; loading.value = false }
async function add() {
  if (!form.value.username.trim() || !form.value.password.trim()) { showToast('请输入用户名和密码'); return }
  const email = form.value.email.trim()
  if (email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) { showToast('邮箱格式不正确'); return }
  const res = await adminApi('add_admin', { ...form.value, email }); if (res.code === 200) { showToast('已新增', 'success'); form.value = { username: '', password: '', email: '', role: 'admin' }; load() } else showToast(res.msg || '新增失败')
}
async function toggle(a: any) { const res = await adminApi('toggle_admin_status', { id: a.id }); if (res.code === 200) { a.status = a.status == 1 ? 0 : 1; showToast('已更新', 'success') } else showToast(res.msg || '操作失败') }
async function remove(a: any) { if (!(await mobileConfirm('确认删除管理员？'))) return; const res = await adminApi('delete_admin', { id: a.id }); if (res.code === 200) { showToast('已删除', 'success'); load() } else showToast(res.msg || '删除失败') }
onMounted(load)
</script>
<style scoped>
.mobile-admin-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.mobile-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  flex-shrink: 0;
  border: 1px solid var(--border);
}
.mobile-avatar-letter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 17px;
  font-weight: 800;
}
.mobile-item-email {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  word-break: break-all;
}
.mobile-self-tag {
  display: inline-block;
  margin-left: 6px;
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--accent);
  color: #fff;
  vertical-align: middle;
}
.mobile-login-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}
.mobile-login-field:last-child { margin-bottom: 0; }
.mobile-login-field label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}
.mobile-login-field .mobile-dialog-input {
  width: 100%;
  box-sizing: border-box;
}
</style>