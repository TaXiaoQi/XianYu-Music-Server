<template>
  <div class="mobile-page">
    <div class="mobile-card">
      <h3 class="mobile-card-title">数据库操作</h3>
      <div class="mobile-actions"><button class="mobile-btn primary" @click="backup">立即备份</button><button class="mobile-btn" @click="repair">修复数据库</button></div>
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
onMounted(load)
</script>
<style scoped>
.backup-row{display:flex;justify-content:space-between;align-items:center;gap:8px;padding:9px 0;border-top:1px solid var(--border);font-size:12px;word-break:break-all}
.backup-name{min-width:0;overflow-wrap:anywhere}
.row-actions{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}
</style>
