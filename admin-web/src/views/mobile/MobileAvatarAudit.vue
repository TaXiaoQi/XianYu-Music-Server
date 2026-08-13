<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="audit-header">
      <div class="audit-header-info">
        <h2 class="audit-title">
          头像/改名审核
          <span v-if="totalPending > 0" class="pending-badge">{{ totalPending }} 项待审核</span>
        </h2>
        <p class="audit-desc">审核用户上传的头像和改名申请，通过后立即生效。</p>
      </div>
      <button class="refresh-btn" :disabled="loading" @click="loadAll()">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spin: loading }">
          <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
      </button>
    </div>

    <!-- 统计卡片 -->
    <div class="stat-grid">
      <div class="stat-chip pending">
        <span class="stat-num">{{ avatarStats.pending }}</span>
        <span class="stat-label">待审核</span>
      </div>
      <div class="stat-chip approved">
        <span class="stat-num">{{ avatarStats.approved }}</span>
        <span class="stat-label">已通过</span>
      </div>
      <div class="stat-chip rejected">
        <span class="stat-num">{{ avatarStats.rejected }}</span>
        <span class="stat-label">已拒绝</span>
      </div>
      <div class="stat-chip nickname">
        <span class="stat-num">{{ nicknameList.length }}</span>
        <span class="stat-label">待审核改名</span>
      </div>
    </div>

    <!-- 外部审核策略 -->
    <section class="mobile-card policy-card">
      <div class="policy-head">
        <h3>外部审核策略</h3>
        <button class="pill-switch" :class="{ on: auditConfig.enabled }" @click="auditConfig.enabled = !auditConfig.enabled">
          <span class="pill-knob"></span>
          <span class="pill-text">{{ auditConfig.enabled ? '已启用' : '未启用' }}</span>
        </button>
      </div>
      <p class="policy-tip">开启后，昵称、头像、壁纸会先走外部机审；机审无法判断或服务失败时再进入人工审核队列。</p>
      <select v-model="auditConfig.provider" class="mobile-select">
        <option value="generic">通用 HTTP</option>
        <option value="aliyun">阿里云内容安全</option>
        <option value="tencent">腾讯云内容安全</option>
      </select>
      <input v-model.trim="auditConfig.endpoint" class="mobile-input" placeholder="审核接口地址" />
      <input v-model.trim="auditConfig.api_key" class="mobile-input" type="password" placeholder="接口密钥（留空保留原值）" />
      <input v-model.number="auditConfig.timeout_ms" class="mobile-input" type="number" placeholder="超时时间 ms" />
      <div class="opt-grid">
        <label class="check-row"><input v-model="auditConfig.nickname_enabled" type="checkbox" /> 改名机审</label>
        <label class="check-row"><input v-model="auditConfig.avatar_enabled" type="checkbox" /> 头像机审</label>
        <label class="check-row"><input v-model="auditConfig.wallpaper_enabled" type="checkbox" /> 壁纸机审</label>
        <label class="check-row"><input v-model="auditConfig.fail_to_manual" type="checkbox" /> 失败转人工</label>
      </div>
      <input v-model.trim="auditTestText" class="mobile-input" placeholder="测试文本，例如：测试昵称" />
      <div class="mobile-actions">
        <button class="mobile-btn primary" :disabled="auditSaving" @click="saveAuditConfig">保存策略</button>
        <button class="mobile-btn" :disabled="auditTesting" @click="testAuditConfig">测试连接</button>
      </div>
    </section>

    <!-- tabs -->
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: tab === 'avatar' }" @click="tab = 'avatar'">头像 {{ avatarList.length }}</button>
      <button class="mobile-btn" :class="{ primary: tab === 'nickname' }" @click="tab = 'nickname'">改名 {{ nicknameList.length }}</button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 空状态 -->
    <div v-else-if="currentList.length === 0" class="mobile-empty">
      {{ tab === 'avatar' ? '暂无待审核头像' : '暂无待审核改名申请' }}
    </div>

    <!-- 改名审核 -->
    <div v-else-if="tab === 'nickname'" class="mobile-list">
      <div v-for="item in nicknameList" :key="item.id" class="mobile-item">
        <div class="nick-title">
          <span class="nick-id">{{ item.ciyuanxi_id || '未知' }}</span>
          <span class="nick-time">{{ item.created_at || '-' }}</span>
        </div>
        <div class="nick-change">
          <span class="nick-old">{{ item.old_name || '未知' }}</span>
          <span class="nick-arrow">→</span>
          <span class="nick-new">{{ item.new_name }}</span>
        </div>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="approveNickname(item)">通过</button>
          <button class="mobile-btn danger" @click="rejectNickname(item)">拒绝</button>
        </div>
      </div>
    </div>

    <!-- 头像审核 -->
    <div v-else class="mobile-list avatar-list">
      <div v-for="item in avatarList" :key="item.id" class="mobile-item avatar-item">
        <div class="avatar-compare">
          <div class="av-box">
            <img v-if="item.avatar_data" :src="item.avatar_data" :alt="item.username" class="av-img" @error="onImgError" />
            <div v-else class="av-fallback">
              <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <span class="av-label label-new">新头像</span>
          </div>
          <span class="av-vs">VS</span>
          <div class="av-box">
            <img v-if="item.current_avatar" :src="item.current_avatar" :alt="'当前头像'" class="av-img av-img-old" @error="onImgError" />
            <div v-else class="av-fallback av-fallback-old">
              <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <span class="av-label label-old">当前</span>
          </div>
        </div>
        <div class="av-info">
          <div class="mobile-item-title">{{ item.username || '未知用户' }}</div>
          <div class="mobile-item-sub">弦予号：{{ item.ciyuanxi_id || '-' }} · {{ item.created_at || '-' }}</div>
        </div>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="approveAvatar(item)">通过</button>
          <button class="mobile-btn danger" @click="rejectAvatar(item)">拒绝</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm } from '@/utils/mobileDialog'
import './MobilePage.css'

const tab = ref<'avatar' | 'nickname'>('avatar')
const loading = ref(true)
const avatarList = ref<any[]>([])
const nicknameList = ref<any[]>([])
const avatarStats = reactive({ pending: 0, approved: 0, rejected: 0 })
const auditSaving = ref(false)
const auditTesting = ref(false)
const auditTestText = ref('测试昵称')
const auditConfig = reactive<any>({
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

const currentList = computed(() => (tab.value === 'avatar' ? avatarList.value : nicknameList.value))
const totalPending = computed(() => avatarStats.pending + nicknameList.value.length)

async function loadAvatarConfig() {
  const res = await adminApi<any>('get_audit_external_config')
  if (res.code === 200 && res.data) Object.assign(auditConfig, res.data, { api_key: '' })
}
async function saveAuditConfig() {
  auditSaving.value = true
  const res = await adminApi<any>('save_audit_external_config', auditConfig as any)
  auditSaving.value = false
  if (res.code === 200) { auditConfig.api_key = ''; showToast('外部审核策略已保存', 'success'); }
  else showToast(res.msg || '保存失败')
}
async function testAuditConfig() {
  auditTesting.value = true
  const res = await adminApi<any>('test_audit_external_config', { text: auditTestText.value || '测试昵称' })
  auditTesting.value = false
  if (res.code === 200 && res.data) {
    const label = res.data.decision === 'pass' ? '通过' : res.data.decision === 'reject' ? '拒绝' : '转人工'
    showToast(`测试结果：${label}${res.data.reason ? '，' + res.data.reason : ''}`, 'success')
  } else showToast(res.msg || '测试失败')
}

async function loadAvatars() {
  const res = await adminApi<any>('list_avatar_pending')
  if (res.code === 200 && res.data) {
    avatarList.value = res.data.list || []
    const s = res.data.stats
    avatarStats.pending = s?.pending ?? 0
    avatarStats.approved = s?.approved ?? 0
    avatarStats.rejected = s?.rejected ?? 0
  } else avatarList.value = []
}
async function loadNicknames() {
  const res = await adminApi<any>('list_nickname_pending')
  nicknameList.value = res.code === 200 && Array.isArray(res.data) ? res.data : []
}
async function loadAll(silent = false) {
  if (!silent) loading.value = true
  await Promise.all([loadAvatars(), loadNicknames(), loadAvatarConfig()])
  if (!silent) loading.value = false
}

async function approveAvatar(i: any) {
  const ok = await mobileConfirm('确认通过该头像审核？通过后将更新为用户的新头像。', { title: '通过审核', confirmText: '确认通过' })
  if (!ok) return
  const res = await adminApi('approve_avatar', { id: i.id })
  if (res.code === 200) {
    showToast('审核通过', 'success')
    avatarList.value = avatarList.value.filter(a => a.id !== i.id)
    avatarStats.pending = Math.max(0, avatarStats.pending - 1)
    avatarStats.approved += 1
  } else showToast(res.msg || '操作失败')
}
async function rejectAvatar(i: any) {
  const ok = await mobileConfirm('确认拒绝该头像审核？', { title: '拒绝审核', confirmText: '确认拒绝', danger: true })
  if (!ok) return
  const res = await adminApi('reject_avatar', { id: i.id })
  if (res.code === 200) {
    showToast('已拒绝', 'success')
    avatarList.value = avatarList.value.filter(a => a.id !== i.id)
    avatarStats.pending = Math.max(0, avatarStats.pending - 1)
    avatarStats.rejected += 1
  } else showToast(res.msg || '操作失败')
}
async function approveNickname(i: any) {
  const ok = await mobileConfirm('确认通过该改名申请？通过后用户名将更新。', { title: '通过改名', confirmText: '确认通过' })
  if (!ok) return
  const res = await adminApi('approve_nickname', { id: i.id })
  if (res.code === 200) { showToast('审核通过', 'success'); nicknameList.value = nicknameList.value.filter(n => n.id !== i.id); }
  else showToast(res.msg || '操作失败')
}
async function rejectNickname(i: any) {
  const ok = await mobileConfirm('确认拒绝该改名申请？', { title: '拒绝改名', confirmText: '确认拒绝', danger: true })
  if (!ok) return
  const res = await adminApi('reject_nickname', { id: i.id })
  if (res.code === 200) { showToast('已拒绝', 'success'); nicknameList.value = nicknameList.value.filter(n => n.id !== i.id); }
  else showToast(res.msg || '操作失败')
}

function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => loadAll(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
onMounted(() => { loadAll(); startPolling() })
onUnmounted(() => stopPolling())
</script>
<style scoped>
.audit-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.audit-header-info { min-width: 0; }
.audit-title {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 18px;
  font-weight: 850;
  margin: 0 0 4px;
  color: var(--text);
}
.pending-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 20px;
  background: #fffbeb;
  color: #f59e0b;
  animation: badgePulse 2s ease-in-out infinite;
}
@keyframes badgePulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.3); }
  50% { box-shadow: 0 0 0 5px rgba(245, 158, 11, 0); }
}
.audit-desc {
  font-size: 12px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
}
.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  flex-shrink: 0;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
}
.refresh-btn:active { transform: scale(0.92); }
.refresh-btn:disabled { opacity: 0.5; }
.spin { animation: spin 0.8s linear infinite; }
@keyframes spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.stat-chip {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 12px 10px;
  border-radius: 14px;
  background: var(--card);
  border: 1px solid var(--border);
}
.stat-num { font-size: 20px; font-weight: 850; line-height: 1.1; }
.stat-label { font-size: 11px; color: var(--text-muted); }
.stat-chip.pending .stat-num { color: #f59e0b; }
.stat-chip.approved .stat-num { color: #16a34a; }
.stat-chip.rejected .stat-num { color: #dc2626; }
.stat-chip.nickname .stat-num { color: #2563eb; }

.policy-card { padding: 16px; }
.policy-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 6px;
}
.policy-head h3 { margin: 0; font-size: 15px; }
.policy-tip { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0 0 12px; }
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

.nick-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
}
.nick-id { font-size: 13px; font-weight: 700; color: var(--text); }
.nick-time { font-size: 11px; color: var(--text-muted); }
.nick-change {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--control-bg);
  border: 1px solid var(--border);
  margin-bottom: 10px;
}
.nick-old { flex: 1; min-width: 0; font-size: 13px; font-weight: 700; color: var(--text-muted); text-decoration: line-through; word-break: break-word; }
.nick-arrow { flex: 0 0 auto; color: #EC4141; font-weight: 900; }
.nick-new { flex: 1; min-width: 0; font-size: 13px; font-weight: 800; color: var(--text); word-break: break-word; }

.avatar-list { display: flex; flex-direction: column; gap: 12px; }
.avatar-item { align-items: center; }
.avatar-compare {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 12px;
}
.av-box { display: flex; flex-direction: column; align-items: center; gap: 6px; }
.av-img {
  width: 72px; height: 72px;
  border-radius: 50%;
  object-fit: cover;
  border: 3px solid var(--border);
  background: var(--control-bg);
}
.av-img-old { width: 52px; height: 52px; opacity: 0.7; border-width: 2px; }
.av-fallback {
  width: 72px; height: 72px;
  border-radius: 50%;
  background: var(--control-bg);
  display: flex; align-items: center; justify-content: center;
  color: var(--text-muted);
  border: 3px solid var(--border);
}
.av-fallback-old { width: 52px; height: 52px; border-width: 2px; }
.av-vs { font-size: 11px; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
.av-label { font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 8px; }
.label-new { background: #eff6ff; color: #2563eb; }
.label-old { background: var(--control-bg); color: var(--text-muted); }
.av-info { text-align: center; margin-bottom: 12px; }
</style>