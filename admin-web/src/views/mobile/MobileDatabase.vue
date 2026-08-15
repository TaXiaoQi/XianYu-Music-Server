<template>
  <div class="mobile-page">
    <div class="mobile-card">
      <h3 class="mobile-card-title">数据库操作</h3>
      <div class="mobile-actions">
        <button class="mobile-btn primary" @click="backup">立即备份</button>
        <button class="mobile-btn" @click="repair">修复数据库</button>
        <button class="mobile-btn auto" :class="{ active: autoForm.enabled }" @click="openAutoBackup">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
          自动备份<span v-if="autoForm.enabled" class="auto-dot"></span>
        </button>
      </div>
    </div>
    <section class="mobile-card">
      <h3 class="mobile-card-title">数据表</h3>
      <div v-for="t in tables" :key="t.name || t.table_name" class="backup-row">
        <span class="backup-name">{{ t.name || t.table_name }} · {{ t.rows ?? t.count ?? '-' }} 行</span>
        <button class="mobile-btn" @click="viewTable(t)">查看</button>
      </div>
    </section>
    <transition name="expand">
      <section v-if="tableText" class="mobile-card">
        <h3 class="mobile-card-title">表内容预览</h3>
        <pre class="mobile-code">{{ tableText }}</pre>
      </section>
    </transition>
    <section class="mobile-card">
      <h3 class="mobile-card-title">备份文件</h3>
      <div v-for="b in backups" :key="b.name" class="backup-row">
        <span class="backup-name">{{ b.name }}</span>
        <div class="row-actions">
          <button class="mobile-btn" @click="viewBackup(b)">查看</button>
          <button class="mobile-btn" @click="restoreBackup(b)">恢复</button>
          <button class="mobile-btn danger" @click="removeBackup(b)">删</button>
        </div>
      </div>
    </section>
    <transition name="expand">
      <section v-if="backupText" class="mobile-card">
        <h3 class="mobile-card-title">备份内容预览</h3>
        <pre class="mobile-code">{{ backupText }}</pre>
      </section>
    </transition>

    <!-- 自动备份设置弹窗 -->
    <transition name="mobile-fade">
      <div v-if="autoPopup" class="mobile-dialog-overlay" @click.self="closeAutoBackup">
        <div class="mobile-dialog">
          <div class="mobile-dialog-title">自动备份设置</div>
          <div class="auto-body">
            <div class="auto-toggle">
              <span class="auto-label">启用自动备份</span>
              <label class="auto-switch">
                <input type="checkbox" v-model="autoForm.enabled" />
                <span class="auto-slider"></span>
              </label>
            </div>
            <div class="auto-field">
              <label>备份间隔</label>
              <select class="mobile-select" v-model.number="autoForm.interval_minutes">
                <option :value="30">每 30 分钟</option>
                <option :value="60">每 1 小时</option>
                <option :value="180">每 3 小时</option>
                <option :value="360">每 6 小时</option>
                <option :value="720">每 12 小时</option>
                <option :value="1440">每天</option>
                <option :value="2880">每 2 天</option>
                <option :value="4320">每 3 天</option>
                <option :value="10080">每周</option>
              </select>
            </div>
            <div class="auto-field">
              <label>最大保留份数（超出自动清理）</label>
              <input type="number" class="mobile-input" v-model.number="autoForm.max_count" min="1" max="1000" />
            </div>
            <div class="auto-field">
              <label>备份模式</label>
              <div class="auto-radio-group">
                <label class="auto-radio">
                  <input type="radio" v-model="autoForm.mode" value="full" />
                  <span class="auto-radio-dot"></span>
                  <span>全量备份</span>
                </label>
                <label class="auto-radio">
                  <input type="radio" v-model="autoForm.mode" value="incremental" />
                  <span class="auto-radio-dot"></span>
                  <span>增量备份</span>
                </label>
              </div>
            </div>
            <div class="auto-desc">
              <p><strong>全量备份</strong>：每次备份所有表的完整结构，文件较大。</p>
              <p><strong>增量备份</strong>：仅备份行数变化的表，文件更小、速度更快。</p>
              <p class="auto-desc-muted">下次自动备份将在保存后按间隔生效。</p>
            </div>
          </div>
          <div class="mobile-dialog-actions">
            <button class="mobile-dialog-btn cancel" type="button" :disabled="autoSaving" @click="closeAutoBackup">取消</button>
            <button class="mobile-dialog-btn confirm" type="button" :disabled="autoSaving" @click="saveAutoBackup">
              {{ autoSaving ? '保存中...' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const tables = ref<any[]>([]), backups = ref<any[]>([])
const tableText = ref('')
const backupText = ref('')
async function load() { const [t,b] = await Promise.all([adminApi<any>('list_tables'), adminApi<any>('list_backups')]); tables.value = t.code === 200 && t.data ? (t.data.tables || []) : []; backups.value = b.code === 200 && b.data ? (b.data.backups || []) : [] }
async function backup() { const res = await adminApi('backup_db'); res.code === 200 ? (showToast('备份完成', 'success'), load()) : showToast(res.msg || '备份失败') }
async function repair() { const res = await adminApi('repair_database'); res.code === 200 ? showToast('修复完成', 'success') : showToast(res.msg || '修复失败') }
async function viewTable(t: any) {
  const name = t.name || t.table_name
  const res = await adminApi<any>('view_table', { table_name: name, page: 1 })
  tableText.value = res.code === 200 ? JSON.stringify(res.data || {}, null, 2) : (res.msg || '查看失败')
}
async function viewBackup(b: any) {
  const res = await adminApi<any>('view_backup', { filename: b.name })
  backupText.value = res.code === 200 ? (res.data?.content || JSON.stringify(res.data || {}, null, 2)) : (res.msg || '查看失败')
}
async function restoreBackup(b: any) {
  if (!(await mobileConfirm(`确认恢复备份 ${b.name}？当前数据会被覆盖。`))) return
  const res = await adminApi('restore_backup', { filename: b.name })
  res.code === 200 ? showToast('恢复完成', 'success') : showToast(res.msg || '恢复失败')
}
async function removeBackup(b: any) { if (!(await mobileConfirm('确认删除备份？'))) return; const res = await adminApi('delete_backup', { filename: b.name }); res.code === 200 ? (showToast('已删除', 'success'), load()) : showToast(res.msg || '删除失败') }

// ===== 自动备份设置 =====
interface AutoBackupConfig {
  enabled: boolean
  interval_minutes: number
  max_count: number
  mode: string
  last_run?: string
}
const autoPopup = ref(false)
const autoSaving = ref(false)
const autoForm = ref<AutoBackupConfig>({ enabled: false, interval_minutes: 1440, max_count: 20, mode: 'full' })

async function openAutoBackup() {
  autoPopup.value = true
  const res = await adminApi<AutoBackupConfig>('get_auto_backup_config')
  if (res.code === 200 && res.data) {
    autoForm.value = {
      enabled: !!res.data.enabled,
      interval_minutes: Number(res.data.interval_minutes) || 1440,
      max_count: Number(res.data.max_count) || 20,
      mode: res.data.mode === 'incremental' ? 'incremental' : 'full',
      last_run: res.data.last_run,
    }
  } else {
    showToast(res.msg || '自动备份配置加载失败')
  }
}

function closeAutoBackup() {
  if (autoSaving.value) return
  autoPopup.value = false
}

async function saveAutoBackup() {
  if (autoSaving.value) return
  const interval = Number(autoForm.value.interval_minutes)
  const maxCount = Number(autoForm.value.max_count)
  if (!interval || interval < 1 || interval > 43200) { showToast('备份间隔需在 1 ~ 43200 分钟之间'); return }
  if (!maxCount || maxCount < 1 || maxCount > 1000) { showToast('备份最大次数需在 1 ~ 1000 之间'); return }
  autoSaving.value = true
  const res = await adminApi<AutoBackupConfig>('save_auto_backup_config', {
    enabled: autoForm.value.enabled ? 1 : 0,
    interval_minutes: interval,
    max_count: maxCount,
    mode: autoForm.value.mode,
  })
  autoSaving.value = false
  if (res.code === 200) {
    showToast('自动备份设置已保存', 'success')
    closeAutoBackup()
  } else {
    showToast(res.msg || '保存失败')
  }
}
onMounted(load)
</script>
<style scoped>
.backup-row{display:flex;justify-content:space-between;align-items:center;gap:8px;padding:9px 0;border-top:1px solid var(--border);font-size:12px;word-break:break-all}
.backup-name{min-width:0;overflow-wrap:anywhere}
.row-actions{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}
.mobile-btn.auto{display:inline-flex;align-items:center;gap:6px;border-color:#7c3aed;background:rgba(124,58,237,0.08);color:#7c3aed}
.mobile-btn.auto.active{border-color:#7c3aed;background:#7c3aed;color:#fff}
.auto-dot{width:7px;height:7px;border-radius:50%;background:currentColor;box-shadow:0 0 0 2px rgba(255,255,255,0.35);animation:pulse 1.6s ease-in-out infinite}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.35}}
/* 弹窗内容 */
.auto-body{padding:4px 20px 20px;display:flex;flex-direction:column;gap:14px;max-height:52vh;overflow-y:auto;-webkit-overflow-scrolling:touch}
.auto-toggle{display:flex;align-items:center;justify-content:space-between}
.auto-label{font-size:14px;font-weight:850;color:var(--text)}
.auto-switch{position:relative;display:inline-block;width:46px;height:26px;flex-shrink:0;cursor:pointer}
.auto-switch input{opacity:0;width:0;height:0}
.auto-slider{position:absolute;inset:0;background:#d1d5db;border-radius:26px;transition:background 0.25s cubic-bezier(0.16,1,0.3,1)}
.auto-slider::before{content:'';position:absolute;width:20px;height:20px;left:3px;top:3px;background:#fff;border-radius:50%;transition:transform 0.25s cubic-bezier(0.16,1,0.3,1);box-shadow:0 1px 3px rgba(0,0,0,0.2)}
.auto-switch input:checked + .auto-slider{background:var(--accent)}
.auto-switch input:checked + .auto-slider::before{transform:translateX(20px)}
.auto-field{display:flex;flex-direction:column;gap:7px}
.auto-field label{color:var(--text-light);font-size:12px;font-weight:800}
.auto-radio-group{display:flex;gap:18px}
.auto-radio{display:inline-flex;align-items:center;gap:8px;cursor:pointer;font-size:13px;color:var(--text)}
.auto-radio input{display:none}
.auto-radio-dot{width:17px;height:17px;border-radius:50%;border:2px solid var(--border);position:relative;transition:border-color 0.2s}
.auto-radio input:checked + .auto-radio-dot{border-color:var(--accent)}
.auto-radio input:checked + .auto-radio-dot::after{content:'';position:absolute;inset:3px;border-radius:50%;background:var(--accent)}
.auto-desc{padding:12px 14px;background:var(--control-bg);border-radius:14px;border:1px solid var(--border)}
.auto-desc p{margin:0 0 6px 0;font-size:12px;color:var(--text-light);line-height:1.6}
.auto-desc p:last-child{margin-bottom:0}
.auto-desc-muted{color:var(--text-muted)!important}
</style>
