<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">外部审核配置</h3>
      <select v-model="config.enabled" class="mobile-select">
        <option :value="false">关闭外部审核</option>
        <option :value="true">启用外部审核</option>
      </select>
      <input v-model="config.provider" class="mobile-input" placeholder="服务商标识" />
      <input v-model="config.endpoint" class="mobile-input" placeholder="接口地址" />
      <input v-model="config.api_key" class="mobile-input" placeholder="API Key（留空不改）" type="password" />
      <select v-model="config.nickname_enabled" class="mobile-select"><option :value="true">启用昵称审核</option><option :value="false">关闭昵称审核</option></select>
      <select v-model="config.avatar_enabled" class="mobile-select"><option :value="true">启用头像审核</option><option :value="false">关闭头像审核</option></select>
      <select v-model="config.wallpaper_enabled" class="mobile-select"><option :value="true">启用壁纸审核</option><option :value="false">关闭壁纸审核</option></select>
      <input v-model="config.timeout_ms" class="mobile-input" type="number" placeholder="超时时间 ms" />
      <select v-model="config.fail_to_manual" class="mobile-select"><option :value="true">失败转人工</option><option :value="false">失败不转人工</option></select>
      <div class="mobile-actions">
        <button class="mobile-btn primary" @click="saveConfig">保存配置</button>
        <button class="mobile-btn" @click="testConfig">测试配置</button>
      </div>
      <transition name="expand">
        <pre v-if="testText" class="mobile-code">{{ testText }}</pre>
      </transition>
    </section>
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: tab === 'avatar' }" @click="tab = 'avatar'">头像 {{ avatars.length }}</button>
      <button class="mobile-btn" :class="{ primary: tab === 'nickname' }" @click="tab = 'nickname'">改名 {{ nicknames.length }}</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="currentList.length === 0" class="mobile-empty">
      {{ tab === 'avatar' ? '暂无待审核头像' : '暂无待审核改名申请' }}
    </div>
    <div v-else class="mobile-list">
      <div v-for="item in currentList" :key="item.id" class="mobile-item">
        <img v-if="tab === 'avatar' && (item.avatar_data || item.avatar_url || item.new_avatar)" :src="item.avatar_data || item.avatar_url || item.new_avatar" class="avatar-img" />
        <template v-if="tab === 'nickname'">
          <div class="mobile-item-title">弦予号：{{ item.ciyuanxi_id || item.user_id || '-' }}</div>
          <div class="nickname-change">
            <span class="nickname-old">{{ item.old_name || '未知' }}</span>
            <span class="nickname-arrow">→</span>
            <span class="nickname-new">{{ item.new_name || item.new_nickname || item.nickname || '-' }}</span>
          </div>
        </template>
        <template v-else>
          <div class="mobile-item-title">{{ item.username || item.ciyuanxi_id || item.user_id || '-' }}</div>
        </template>
        <div class="mobile-item-sub">{{ item.created_at || '-' }}</div>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="approve(item)">通过</button>
          <button class="mobile-btn danger" @click="reject(item)">拒绝</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const tab = ref<'avatar' | 'nickname'>('avatar')
const loading = ref(false), avatars = ref<any[]>([]), nicknames = ref<any[]>([])
const config = ref<any>({ enabled: false, provider: 'generic', endpoint: '', api_key: '', nickname_enabled: true, avatar_enabled: true, wallpaper_enabled: true, timeout_ms: 5000, fail_to_manual: true })
const testText = ref('')
const currentList = computed(() => tab.value === 'avatar' ? avatars.value : nicknames.value)
async function loadConfig() {
  const res = await adminApi<any>('get_audit_external_config')
  if (res.code === 200 && res.data) config.value = { ...config.value, ...res.data, api_key: '' }
}
async function saveConfig() {
  const res = await adminApi('save_audit_external_config', config.value)
  if (res.code === 200) showToast('配置已保存', 'success'); else showToast(res.msg || '保存失败')
}
async function testConfig() {
  const res = await adminApi<any>('test_audit_external_config', { scene: tab.value, text: '测试内容' })
  testText.value = res.code === 200 ? JSON.stringify(res.data || {}, null, 2) : (res.msg || '测试失败')
}
async function load(silent = false) { if (!silent) loading.value = true; const [a,n] = await Promise.all([adminApi<any>('list_avatar_pending'), adminApi<any[]>('list_nickname_pending')]); avatars.value = a.code === 200 && a.data ? (a.data.list || []) : []; nicknames.value = n.code === 200 && n.data ? n.data : []; if (!silent) loading.value = false }
async function approve(i: any) { const res = await adminApi(tab.value === 'avatar' ? 'approve_avatar' : 'approve_nickname', { id: i.id }); if (res.code === 200) { showToast('已通过', 'success'); load() } else showToast(res.msg || '操作失败') }
async function reject(i: any) { const res = await adminApi(tab.value === 'avatar' ? 'reject_avatar' : 'reject_nickname', { id: i.id }); if (res.code === 200) { showToast('已拒绝', 'success'); load() } else showToast(res.msg || '操作失败') }
let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => load(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
onMounted(() => { loadConfig(); load(); startPolling() })
onUnmounted(() => stopPolling())
</script>
<style scoped>
.avatar-img{width:92px;height:92px;border-radius:50%;object-fit:cover;margin-bottom:10px;background:var(--control-bg)}
.nickname-change {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--control-bg);
  border: 1px solid var(--border);
}
.nickname-old,
.nickname-new {
  min-width: 0;
  flex: 1;
  font-size: 13px;
  font-weight: 800;
  word-break: break-word;
}
.nickname-old {
  color: var(--text-muted);
  text-decoration: line-through;
}
.nickname-arrow {
  flex: 0 0 auto;
  color: #EC4141;
  font-weight: 900;
}
.nickname-new {
  color: var(--text);
}
</style>
