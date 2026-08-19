<template>
  <div class="db-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">数据库管理</h2>
          <p class="page-desc">
            查看数据库表结构与状态，修复缺失的表，备份、查看、恢复与删除数据库备份文件。所有操作均会记录到后台日志。
          </p>
        </div>
      </div>
    </Transition>

    <!-- 操作按钮 -->
    <Transition name="fade-up" appear>
      <div class="action-bar">
        <button class="btn btn-primary" :disabled="repairing" @click="doRepair">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
          </svg>
          {{ repairing ? '修复中...' : '修复数据库' }}
        </button>
        <button class="btn btn-success" :disabled="backingUp" @click="doBackup">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" /><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
          </svg>
          {{ backingUp ? '备份中...' : '数据库备份' }}
        </button>
        <button class="btn btn-auto-backup" :class="{ active: autoEnabled }" @click="openAutoBackupModal">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
          </svg>
          自动备份
          <span v-if="autoEnabled" class="auto-dot"></span>
        </button>
        <button class="btn" :disabled="importing" @click="triggerImport">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          {{ importing ? '导入中...' : '导入数据库' }}
        </button>
        <input ref="importFileInput" type="file" accept=".sql,.txt" class="hidden-input" @change="onImportFile" />
        <button class="btn-refresh" @click="reloadAll" :disabled="loadingTables || loadingBackups || repairing || backingUp">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loadingTables || loadingBackups }">
            <polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" /><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" /></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ tables.length }}</span><span class="stat-label">总表数</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-exist">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ existingCount }}</span><span class="stat-label">已存在</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-miss">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07" /></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ missingCount }}</span><span class="stat-label">缺失</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-backup">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 8v13H3V8" /><path d="M1 3h22v5H1z" /><path d="M10 12h4" /></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ backups.length }}</span><span class="stat-label">备份文件</span></div>
        </div>
      </div>
    </Transition>

    <!-- 数据库表状态 -->
    <Transition name="fade-up" appear>
      <div class="section-card">
        <div class="section-head">
          <h3 class="section-title">数据库表状态</h3>
          <span class="section-sub">{{ existingCount }}/{{ tables.length }} 存在</span>
        </div>

        <div v-if="loadingTables" class="state-box">
          <div class="spinner"></div>
          <span>加载中...</span>
        </div>
        <div v-else-if="tables.length === 0" class="state-box">暂无表数据</div>
        <div v-else class="table-scroll">
          <table class="data-table">
            <thead>
              <tr>
                <th>表名</th>
                <th>状态</th>
                <th>行数</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(t, idx) in pagedTables" :key="t.name" class="row-anim" :style="{ animationDelay: `${idx * 40}ms` }">
                <td>
                  <span
                    class="table-name"
                    :class="{ clickable: t.exists }"
                    :title="t.exists ? '点击查看表内容' : ''"
                    @click="t.exists && openTableViewer(t.name)"
                  >{{ t.name }}</span>
                </td>
                <td>
                  <span class="badge" :class="t.exists ? 'badge-success' : 'badge-error'">
                    {{ t.exists ? '存在' : '缺失' }}
                  </span>
                </td>
                <td>{{ t.exists ? t.row_count : '-' }}</td>
                <td>
                  <button v-if="t.exists" class="btn btn-sm" @click="openTableViewer(t.name)">
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" /><circle cx="12" cy="12" r="3" /></svg>
                    查看内容
                  </button>
                  <span v-else class="muted">-</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 表分页 -->
        <div v-if="!loadingTables && tables.length > 0" class="pagination">
          <button :disabled="tablePage <= 1" @click="goTableListPage(tablePage - 1)">上一页</button>
          <button
            v-for="p in tablePageNumbers"
            :key="p"
            :class="{ active: p === tablePage }"
            @click="goTableListPage(p)"
          >{{ p }}</button>
          <button :disabled="tablePage >= tableTotalPages" @click="goTableListPage(tablePage + 1)">下一页</button>
          <span>共 {{ tables.length }} 条</span>
        </div>
      </div>
    </Transition>

    <!-- 备份列表 -->
    <Transition name="fade-up" appear>
      <div class="section-card">
        <div class="section-head">
          <h3 class="section-title">备份列表</h3>
          <span class="section-sub">共 {{ backups.length }} 个备份</span>
        </div>

        <div v-if="loadingBackups" class="state-box">
          <div class="spinner"></div>
          <span>加载中...</span>
        </div>
        <div v-else-if="backups.length === 0" class="state-box">暂无备份文件，点击上方「数据库备份」创建</div>
        <div v-else class="table-scroll">
          <table class="data-table">
            <thead>
              <tr>
                <th>文件名</th>
                <th>大小</th>
                <th>创建时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(b, idx) in pagedBackups" :key="b.name" class="row-anim" :style="{ animationDelay: `${idx * 40}ms` }">
                <td>
                  <span class="backup-name" :title="b.name">{{ b.name }}</span>
                </td>
                <td>{{ b.size }}</td>
                <td class="nowrap-time">{{ fmtDateTime(b.created_at) || '-' }}</td>
                <td>
                  <div class="row-actions">
                    <button class="btn btn-sm" :disabled="isBusy(b.name, 'view')" @click="openBackupViewer(b)">查看</button>
                    <button class="btn btn-sm" :disabled="isBusy(b.name, 'dl')" @click="downloadBackup(b)">下载</button>
                    <button class="btn btn-sm btn-warning" :disabled="isBusy(b.name, 'rs')" @click="restoreBackup(b)">恢复</button>
                    <button class="btn btn-sm btn-danger" :disabled="isBusy(b.name, 'del')" @click="deleteBackup(b)">删除</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 备份分页 -->
        <div v-if="!loadingBackups && backups.length > 0" class="pagination">
          <button :disabled="backupPage <= 1" @click="goBackupPage(backupPage - 1)">上一页</button>
          <button
            v-for="p in backupPageNumbers"
            :key="p"
            :class="{ active: p === backupPage }"
            @click="goBackupPage(p)"
          >{{ p }}</button>
          <button :disabled="backupPage >= backupTotalPages" @click="goBackupPage(backupPage + 1)">下一页</button>
          <span>共 {{ backups.length }} 条</span>
        </div>
      </div>
    </Transition>

    <!-- 表内容查看弹窗 -->
    <Transition name="modal">
      <div v-if="showTableModal" class="modal-backdrop" @click.self="closeTableModal">
        <div class="modal-dialog modal-lg">
          <div class="modal-head">
            <h3>表内容 - {{ tableData?.table || '' }}</h3>
            <button class="modal-close" @click="closeTableModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="tableLoading" class="state-box">
              <div class="spinner"></div>
              <span>加载中...</span>
            </div>
            <template v-else-if="tableData">
              <div class="table-meta">
                共 {{ tableData.total }} 行 · 第 {{ tableData.page }}/{{ viewerTotalPages }} 页 · 每页 {{ tableData.pageSize }} 行
              </div>
              <div v-if="tableData.rows.length" class="table-scroll">
                <table class="data-table compact">
                  <thead>
                    <tr>
                      <th v-for="c in tableData.columns" :key="c">{{ c }}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, i) in tableData.rows" :key="i">
                      <td v-for="c in tableData.columns" :key="c" :title="formatCell(row[c])">{{ formatCell(row[c]) }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else class="state-box">该表暂无数据</div>
            </template>
            <div v-else class="state-box">该表不存在或无法读取，请点击「修复数据库」后刷新重试，或点击空白处关闭。</div>
          </div>
          <div class="modal-foot">
            <div class="pagination" v-if="tableData && tableData.total > 0">
              <button :disabled="tableLoading || (tableData?.page ?? 1) <= 1" @click="goViewerPage((tableData?.page ?? 1) - 1)">上一页</button>
              <button
                v-for="p in viewerPageNumbers"
                :key="p"
                :class="{ active: p === (tableData?.page ?? 1) }"
                :disabled="tableLoading"
                @click="goViewerPage(p)"
              >{{ p }}</button>
              <button :disabled="tableLoading || (tableData?.page ?? 1) >= viewerTotalPages" @click="goViewerPage((tableData?.page ?? 1) + 1)">下一页</button>
              <span>共 {{ tableData?.total ?? 0 }} 条</span>
            </div>
            <button class="btn-cancel" @click="closeTableModal">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 备份内容查看弹窗 -->
    <Transition name="modal">
      <div v-if="showBackupModal" class="modal-backdrop" @click.self="closeBackupModal">
        <div class="modal-dialog modal-lg">
          <div class="modal-head">
            <h3 class="backup-modal-title" :title="backupViewName">备份内容 - {{ backupViewName }}</h3>
            <button class="modal-close" @click="closeBackupModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="backupLoading" class="state-box">
              <div class="spinner"></div>
              <span>加载中...</span>
            </div>
            <pre v-else class="sql-pre">{{ backupContent }}</pre>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeBackupModal">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 自动备份设置弹窗 -->
    <Transition name="modal">
      <div v-if="showAutoBackupModal" class="modal-backdrop" @click.self="closeAutoBackupModal">
        <div class="modal-dialog modal-sm">
          <div class="modal-head">
            <h3>自动备份设置</h3>
            <button class="modal-close" @click="closeAutoBackupModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="autoLoading" class="state-box">
              <div class="spinner"></div>
              <span>加载中...</span>
            </div>
            <template v-else>
              <!-- 开关 -->
              <div class="form-row">
                <label class="form-label">启用自动备份</label>
                <label class="toggle">
                  <input type="checkbox" v-model="autoForm.enabled" />
                  <span class="toggle-slider"></span>
                </label>
              </div>

              <!-- 备份间隔 -->
              <div class="form-row">
                <label class="form-label">备份间隔</label>
                <select class="form-select" v-model.number="autoForm.interval_minutes">
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

              <!-- 最大保留次数 -->
              <div class="form-row">
                <label class="form-label">最大保留份数</label>
                <div class="form-input-group">
                  <input type="number" class="form-input" v-model.number="autoForm.max_count" min="1" max="1000" />
                  <span class="form-hint">超出自动清理最旧备份</span>
                </div>
              </div>

              <!-- 备份模式 -->
              <div class="form-row">
                <label class="form-label">备份模式</label>
                <div class="radio-group">
                  <label class="radio">
                    <input type="radio" v-model="autoForm.mode" value="full" />
                    <span class="radio-dot"></span>
                    <span>全量备份</span>
                  </label>
                  <label class="radio">
                    <input type="radio" v-model="autoForm.mode" value="incremental" />
                    <span class="radio-dot"></span>
                    <span>增量备份</span>
                  </label>
                </div>
              </div>

              <!-- 说明 -->
              <div class="form-desc">
                <p><strong>全量备份</strong>：每次备份所有表的完整结构和数据，文件较大。</p>
                <p><strong>增量备份</strong>：仅备份行数发生变化的表（基于行数快照对比），文件较小、速度更快。</p>
                <p class="form-desc-muted">下次自动备份将在设置保存后按间隔时间生效。</p>
              </div>
            </template>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAutoBackupModal">取消</button>
            <button class="btn btn-primary" :disabled="autoSaving" @click="saveAutoBackupConfig">
              {{ autoSaving ? '保存中...' : '保存设置' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast, getToken } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { fmtDateTime } from '@/utils/time'

// ===== 类型定义 =====
interface TableInfo {
  name: string
  exists: boolean
  row_count: number
}

interface BackupInfo {
  name: string
  size: string
  size_bytes: number
  created_at: string
}

interface RepairResult {
  created_tables: string[]
  errors: Array<{ table: string; msg: string }>
  summary: {
    created_tables_count: number
    added_columns_count: number
    added_indexes_count: number
    dropped_tables_count: number
  }
}

interface BackupResult {
  filename: string
  filepath?: string
  size: string
  tables: number
}

interface TableData {
  table: string
  columns: string[]
  rows: Record<string, any>[]
  total: number
  page: number
  pageSize: number
}

// ===== 列表数据 =====
const tables = ref<TableInfo[]>([])
const backups = ref<BackupInfo[]>([])
const loadingTables = ref(true)
const loadingBackups = ref(true)
const repairing = ref(false)
const backingUp = ref(false)

const existingCount = computed(() => tables.value.filter(t => t.exists).length)
const missingCount = computed(() => tables.value.filter(t => !t.exists).length)

// ===== 分页（每页 20 条） =====
const PAGE_SIZE = 20
const tablePage = ref(1)
const backupPage = ref(1)

const pagedTables = computed(() => {
  const start = (tablePage.value - 1) * PAGE_SIZE
  return tables.value.slice(start, start + PAGE_SIZE)
})
const tableTotalPages = computed(() => Math.max(1, Math.ceil(tables.value.length / PAGE_SIZE)))
const tablePageNumbers = computed(() => calcPageNumbers(tablePage.value, tableTotalPages.value))

const pagedBackups = computed(() => {
  const start = (backupPage.value - 1) * PAGE_SIZE
  return backups.value.slice(start, start + PAGE_SIZE)
})
const backupTotalPages = computed(() => Math.max(1, Math.ceil(backups.value.length / PAGE_SIZE)))
const backupPageNumbers = computed(() => calcPageNumbers(backupPage.value, backupTotalPages.value))

function calcPageNumbers(cur: number, total: number): number[] {
  const max = 7
  const pages: number[] = []
  if (total <= max) {
    for (let i = 1; i <= total; i++) pages.push(i)
  } else {
    let start = Math.max(1, cur - 3)
    let end = Math.min(total, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
}

function goTableListPage(p: number) {
  if (p < 1 || p > tableTotalPages.value || p === tablePage.value) return
  tablePage.value = p
}

function goBackupPage(p: number) {
  if (p < 1 || p > backupTotalPages.value || p === backupPage.value) return
  backupPage.value = p
}

// ===== 加载数据 =====
async function loadTables() {
  loadingTables.value = true
  const res = await adminApi<{ tables: TableInfo[] }>('list_tables')
  if (res.code === 200 && res.data) {
    tables.value = res.data.tables || []
  } else {
    tables.value = []
    showToast(res.msg || '表列表加载失败')
  }
  if (tablePage.value > tableTotalPages.value) tablePage.value = tableTotalPages.value
  loadingTables.value = false
}

async function loadBackups() {
  loadingBackups.value = true
  const res = await adminApi<{ backups: BackupInfo[]; total: number }>('list_backups')
  if (res.code === 200 && res.data) {
    backups.value = res.data.backups || []
  } else {
    backups.value = []
    showToast(res.msg || '备份列表加载失败')
  }
  if (backupPage.value > backupTotalPages.value) backupPage.value = backupTotalPages.value
  loadingBackups.value = false
}

async function reloadAll() {
  await Promise.all([loadTables(), loadBackups()])
}

// ===== 修复数据库 =====
async function doRepair() {
  if (repairing.value) return
  repairing.value = true
  const res = await adminApi<RepairResult>('repair_database')
  repairing.value = false
  if (res.code === 200 && res.data) {
    const d = res.data
    const created = d.created_tables?.length ?? 0
    const errors = d.errors?.length ?? 0
    if (errors > 0) {
      showToast(`修复完成：创建 ${created} 张表，${errors} 项错误`, 'error')
    } else {
      showToast(`修复完成：已创建/校验 ${created} 张表`, 'success')
    }
    loadTables()
  } else {
    showToast(res.msg || '修复失败')
  }
}

// ===== 数据库备份 =====
async function doBackup() {
  if (backingUp.value) return
  backingUp.value = true
  const res = await adminApi<BackupResult>('backup_db')
  backingUp.value = false
  if (res.code === 200 && res.data) {
    const d = res.data
    showToast(`备份成功：${d.filename}（${d.size}，${d.tables} 张表）`, 'success')
    loadBackups()
  } else {
    showToast(res.msg || '备份失败')
  }
}

// ===== 自动备份设置 =====
interface AutoBackupConfig {
  enabled: boolean
  interval_minutes: number
  max_count: number
  mode: string
  last_run?: string
}

const showAutoBackupModal = ref(false)
const autoLoading = ref(false)
const autoSaving = ref(false)
const autoEnabled = ref(false)
const autoForm = ref<AutoBackupConfig>({ enabled: false, interval_minutes: 1440, max_count: 20, mode: 'full' })

async function openAutoBackupModal() {
  showAutoBackupModal.value = true
  autoLoading.value = true
  const res = await adminApi<AutoBackupConfig>('get_auto_backup_config')
  autoLoading.value = false
  if (res.code === 200 && res.data) {
    autoForm.value = {
      enabled: !!res.data.enabled,
      interval_minutes: Number(res.data.interval_minutes) || 1440,
      max_count: Number(res.data.max_count) || 20,
      mode: res.data.mode === 'incremental' ? 'incremental' : 'full',
      last_run: res.data.last_run,
    }
    autoEnabled.value = autoForm.value.enabled
  } else {
    showToast(res.msg || '自动备份配置加载失败')
  }
}

function closeAutoBackupModal() {
  if (autoSaving.value) return
  showAutoBackupModal.value = false
}

async function saveAutoBackupConfig() {
  if (autoSaving.value) return
  const interval = Number(autoForm.value.interval_minutes)
  const maxCount = Number(autoForm.value.max_count)
  if (!interval || interval < 1 || interval > 43200) {
    showToast('备份间隔需在 1 ~ 43200 分钟之间')
    return
  }
  if (!maxCount || maxCount < 1 || maxCount > 1000) {
    showToast('备份最大次数需在 1 ~ 1000 之间')
    return
  }
  autoSaving.value = true
  const res = await adminApi<AutoBackupConfig>('save_auto_backup_config', {
    enabled: autoForm.value.enabled ? 1 : 0,
    interval_minutes: interval,
    max_count: maxCount,
    mode: autoForm.value.mode,
  })
  autoSaving.value = false
  if (res.code === 200) {
    autoEnabled.value = !!autoForm.value.enabled
    showToast('自动备份设置已保存', 'success')
    closeAutoBackupModal()
  } else {
    showToast(res.msg || '保存失败')
  }
}

// ===== 导入数据库 =====
const importing = ref(false)
const importFileInput = ref<HTMLInputElement | null>(null)

function triggerImport() {
  importFileInput.value?.click()
}

async function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = '' // 允许再次选择同一文件
  if (!file) return
  if (!/\.sql$/i.test(file.name)) {
    showToast('请选择 .sql 备份文件')
    return
  }
  const ok = await webConfirm(`确定导入数据库文件 "${file.name}" 吗？\n\n此操作将执行文件中的 SQL 语句，可能覆盖当前表数据，且不可恢复，请谨慎操作！`, { title: '导入数据库', confirmText: '确认导入' })
  if (!ok) return
  importing.value = true
  try {
    const text = await file.text()
    const res = await adminApi<{ filename: string; ok: number; errors: number }>('import_db', { content: text })
    if (res.code === 200) {
      const d = res.data
      if (d && d.errors > 0) {
        showToast(`导入完成：${d.ok} 条成功，${d.errors} 条失败`, 'error')
      } else {
        showToast(`导入成功：${d?.ok ?? 0} 条语句已执行`, 'success')
      }
      loadTables()
      loadBackups()
    } else {
      showToast(res.msg || '导入失败')
    }
  } catch {
    showToast('导入失败：文件读取错误')
  } finally {
    importing.value = false
  }
}

// ===== 表内容查看 =====
const showTableModal = ref(false)
const tableLoading = ref(false)
const tableData = ref<TableData | null>(null)

const viewerTotalPages = computed(() => {
  if (!tableData.value) return 1
  return Math.max(1, Math.ceil(tableData.value.total / tableData.value.pageSize))
})

const viewerPageNumbers = computed(() => {
  const max = 7
  const pages: number[] = []
  const total = viewerTotalPages.value
  const cur = tableData.value?.page ?? 1
  if (total <= max) {
    for (let i = 1; i <= total; i++) pages.push(i)
  } else {
    let start = Math.max(1, cur - 3)
    let end = Math.min(total, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

async function openTableViewer(name: string) {
  showTableModal.value = true
  tableData.value = null
  await fetchTable(name, 1)
}

async function fetchTable(name: string, page: number) {
  tableLoading.value = true
  const res = await adminApi<TableData>('view_table', { table_name: name, page })
  tableLoading.value = false
  if (res.code === 200 && res.data) {
    tableData.value = res.data
  } else {
    showToast(res.msg || '表内容加载失败')
    tableData.value = null
  }
}

function goViewerPage(p: number) {
  if (!tableData.value) return
  if (p < 1 || p > viewerTotalPages.value || p === tableData.value.page) return
  fetchTable(tableData.value.table, p)
}

function closeTableModal() {
  if (tableLoading.value) return
  showTableModal.value = false
  tableData.value = null
}

// ===== 备份内容查看 =====
const showBackupModal = ref(false)
const backupLoading = ref(false)
const backupContent = ref('')
const backupViewName = ref('')

async function openBackupViewer(item: BackupInfo) {
  showBackupModal.value = true
  backupViewName.value = item.name
  backupContent.value = ''
  backupLoading.value = true
  const res = await adminApi<{ content: string }>('view_backup', { filename: item.name })
  backupLoading.value = false
  if (res.code === 200 && res.data) {
    backupContent.value = res.data.content || ''
  } else {
    showToast(res.msg || '备份内容加载失败')
  }
}

function closeBackupModal() {
  if (backupLoading.value) return
  showBackupModal.value = false
  backupContent.value = ''
  backupViewName.value = ''
}

// ===== 备份下载 =====
async function downloadBackup(item: BackupInfo) {
  const token = getToken()
  try {
    const res = await fetch('/admin/api?action=download_backup', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify({ filename: item.name }),
    })
    if (!res.ok) {
      showToast('下载失败：服务异常')
      return
    }
    const blob = await res.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = item.name
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    showToast('下载已开始', 'success')
  } catch {
    showToast('下载失败：网络错误')
  }
}

// ===== 备份恢复 / 删除 =====
const actionLoading = ref<Record<string, boolean>>({})

function isBusy(name: string, op: string): boolean {
  return !!actionLoading.value[`${name}::${op}`]
}

function setBusy(name: string, op: string, val: boolean) {
  actionLoading.value = { ...actionLoading.value, [`${name}::${op}`]: val }
}

async function restoreBackup(item: BackupInfo) {
  const ok = await webConfirm(`确定从备份 "${item.name}" 恢复数据库吗？\n\n此操作将覆盖当前所有表数据，且不可恢复，请谨慎操作！`, { title: '恢复数据库', confirmText: '确认恢复' })
  if (!ok) return
  setBusy(item.name, 'rs', true)
  const res = await adminApi('restore_backup', { filename: item.name })
  setBusy(item.name, 'rs', false)
  if (res.code === 200) {
    showToast('恢复成功', 'success')
    loadTables()
  } else {
    showToast(res.msg || '恢复失败')
  }
}

async function deleteBackup(item: BackupInfo) {
  const ok = await webConfirm(`确定删除备份文件 "${item.name}" 吗？此操作不可恢复。`, { title: '删除备份', confirmText: '确认删除' })
  if (!ok) return
  setBusy(item.name, 'del', true)
  const res = await adminApi('delete_backup', { filename: item.name })
  setBusy(item.name, 'del', false)
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadBackups()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 工具函数 =====
function formatCell(val: any): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'boolean') return val ? '1' : '0'
  if (typeof val === 'object') {
    try {
      return JSON.stringify(val)
    } catch {
      return String(val)
    }
  }
  return String(val)
}

// ===== 初始化 =====
onMounted(() => {
  loadTables()
  loadBackups()
})
</script>

<style scoped>
.db-page {
  max-width: 1200px;
  margin: 0 auto;
}

/* ===== 页面头部 ===== */
.page-header {
  margin-bottom: 18px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0 0 6px 0;
  color: var(--text);
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 520px;
}

/* ===== 操作按钮行 ===== */
.action-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 18px;
  flex-wrap: wrap;
}
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: none;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  background: var(--card-solid);
  color: var(--text);
  border: 1px solid var(--border);
}
.btn-primary {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.btn-success {
  background: #16a34a;
  color: #fff;
  border-color: #16a34a;
}
.btn-auto-backup {
  background: #7c3aed;
  color: #fff;
  border-color: #7c3aed;
  position: relative;
}
.btn-auto-backup.active {
  background: #6d28d9;
  border-color: #6d28d9;
}
.auto-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
  animation: pulse 1.6s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}
.btn-warning {
  background: var(--card-solid);
  color: #d97706;
  border-color: var(--border);
}
.btn-warning:hover {
  background: rgba(245, 158, 11, 0.14);
}
.btn-danger {
  background: var(--card-solid);
  color: #dc2626;
  border-color: var(--border);
}
.btn-danger:hover {
  background: rgba(236, 65, 65, 0.12);
}
.btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
}
.btn:active { transform: scale(0.96); }
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}
.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
}
.btn-refresh {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-refresh:hover { border-color: var(--accent); transform: translateY(-1px); }
.btn-refresh:active { transform: scale(0.96); }
.btn-refresh:disabled { opacity: 0.5; cursor: not-allowed; }
.spinning { animation: spin 0.8s linear infinite; }
.hidden-input { display: none; }

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 18px;
}
.stat-chip {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: var(--track); color: var(--text-light); }
.stat-icon-exist { background: rgba(34, 197, 94, 0.14); color: #16a34a; }
.stat-icon-miss { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.stat-icon-backup { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

/* ===== 区块卡片 ===== */
.section-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  margin-bottom: 18px;
}
.section-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}
.section-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
  margin: 0;
}
.section-sub {
  font-size: 12px;
  color: var(--text-muted);
}

/* ===== 表格 ===== */
.table-scroll {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
.data-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--card-solid);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border);
}
.data-table th {
  padding: 12px 14px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  background: var(--control-bg);
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.data-table td {
  padding: 12px 14px;
  font-size: 13px;
  color: var(--text);
  border-bottom: 1px solid #f5f5f5;
}
.data-table tr:last-child td { border-bottom: none; }
.data-table tr:hover td { background: var(--table-row-hover); }
.data-table.compact th,
.data-table.compact td {
  padding: 8px 12px;
  font-size: 12px;
  white-space: nowrap;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.data-table.compact th {
  position: sticky;
  top: 0;
  z-index: 1;
}

.table-name {
  font-weight: 600;
  color: var(--text);
}
.table-name.clickable {
  color: #2563eb;
  cursor: pointer;
  transition: color 0.15s;
}
.table-name.clickable:hover { color: #1d4ed8; text-decoration: underline; }
.backup-name {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: var(--text);
}
.muted { color: var(--text-muted); }
.nowrap-time { white-space: nowrap; font-size: 12px; color: var(--text-light); }

/* 徽标 */
.badge {
  display: inline-flex;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
}
.badge-success { background: rgba(34, 197, 94, 0.14); color: #16a34a; }
.badge-error { background: rgba(236, 65, 65, 0.12); color: #dc2626; }

/* 行操作 */
.row-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* 表内容元信息 */
.table-meta {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

/* ===== 空状态 / 加载 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 20px;
  color: var(--text-muted);
  gap: 12px;
  font-size: 14px;
}
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* 行进入动画 */
.row-anim {
  animation: rowIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes rowIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}
.modal-dialog {
  background: var(--card-solid);
  border-radius: 16px;
  width: 100%;
  max-width: 800px;
  max-height: 80vh;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
}
.modal-lg { max-width: 960px; }
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.modal-head h3 {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.backup-modal-title { max-width: 80%; }
.modal-close {
  width: 32px;
  height: 32px;
  border: none;
  background: var(--track);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  transition: all 0.2s;
  flex-shrink: 0;
}
.modal-close:hover { background: var(--table-row-hover); color: var(--text); }
.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}
.modal-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
  flex-wrap: wrap;
}
.btn-cancel {
  padding: 9px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: var(--track); }

/* SQL 内容 */
.sql-pre {
  background: #1e1e2e;
  color: #e0e0e8;
  padding: 16px;
  border-radius: 10px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-x: auto;
  margin: 0;
  max-height: 60vh;
  overflow-y: auto;
}

/* 自动备份表单 */
.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid #f5f5f5;
}
.form-row:first-of-type { padding-top: 0; }
.form-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  flex-shrink: 0;
}
.form-select {
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  color: var(--text);
  font-size: 13px;
  min-width: 160px;
  cursor: pointer;
}
.form-input-group {
  display: flex;
  align-items: center;
  gap: 10px;
}
.form-input {
  width: 90px;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  color: var(--text);
  font-size: 13px;
}
.form-hint {
  font-size: 12px;
  color: var(--text-muted);
}
.radio-group {
  display: flex;
  gap: 16px;
}
.radio {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
}
.radio input { display: none; }
.radio-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--border);
  position: relative;
  transition: border-color 0.2s;
  flex-shrink: 0;
}
.radio input:checked + .radio-dot {
  border-color: var(--accent);
}
.radio input:checked + .radio-dot::after {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: var(--accent);
}
.form-desc {
  margin-top: 14px;
  padding: 12px 14px;
  background: var(--control-bg);
  border-radius: 10px;
  border: 1px solid var(--border);
}
.form-desc p {
  margin: 0 0 6px 0;
  font-size: 12px;
  color: var(--text-light);
  line-height: 1.6;
}
.form-desc p:last-child { margin-bottom: 0; }
.form-desc-muted { color: var(--text-muted) !important; }
/* 开关 */
.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
  cursor: pointer;
}
.toggle input { opacity: 0; width: 0; height: 0; }
.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--track);
  border-radius: 24px;
  transition: background 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-slider::before {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  left: 3px;
  top: 3px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}
.toggle input:checked + .toggle-slider { background: var(--accent); }
.toggle input:checked + .toggle-slider::before { transform: translateX(20px); }

/* 分页 */
.pagination {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  border-radius: 8px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.pagination button.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination span {
  font-size: 12px;
  color: var(--text-muted);
  margin-left: 8px;
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 12px; flex-direction: column; align-items: flex-start; gap: 6px; }
  .stat-num { font-size: 18px; }
  .section-card { padding: 14px; }
  .action-bar { gap: 8px; }
  .btn { padding: 10px 16px; font-size: 13px; }
  .modal-foot { justify-content: stretch; }
  .pagination span { margin-left: 0; width: 100%; text-align: center; order: 5; }
}
</style>
