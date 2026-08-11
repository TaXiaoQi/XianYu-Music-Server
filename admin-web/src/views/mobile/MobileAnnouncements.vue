<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">{{ editingId ? '编辑公告' : '新增公告' }}</h3>
      <input v-model="form.title" class="mobile-input" placeholder="公告标题" />
      <textarea v-model="form.content" class="mobile-textarea" placeholder="公告内容"></textarea>
      <input v-model="form.sort_order" class="mobile-input" placeholder="排序值" type="number" />
      <select v-model="form.enabled" class="mobile-select">
        <option :value="1">显示</option>
        <option :value="0">隐藏</option>
      </select>
      <div class="mobile-actions">
        <button class="mobile-btn primary" :disabled="saving" @click="save">{{ editingId ? '保存修改' : '发布公告' }}</button>
        <button v-if="editingId" class="mobile-btn" @click="resetForm">取消编辑</button>
      </div>
    </section>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="a in list" :key="a.id" class="mobile-item">
        <div class="mobile-item-head"><div class="mobile-item-title">{{ a.title }}</div><span class="mobile-badge" :class="a.enabled ? 'green' : 'red'">{{ a.enabled ? '显示' : '隐藏' }}</span></div>
        <div class="mobile-item-sub">{{ a.content }}</div>
        <div class="mobile-actions">
          <button class="mobile-btn" @click="edit(a)">编辑</button>
          <button class="mobile-btn" @click="toggle(a)">{{ a.enabled ? '隐藏' : '显示' }}</button>
          <button class="mobile-btn danger" @click="remove(a)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const loading = ref(false), saving = ref(false)
const list = ref<any[]>([])
const editingId = ref(0)
const form = ref({ title: '', content: '', sort_order: 0, enabled: 1 })
async function loadList() { loading.value = true; const res = await adminApi<any[]>('list_announcements'); list.value = res.code === 200 && res.data ? res.data : []; loading.value = false }
function resetForm() {
  editingId.value = 0
  form.value = { title: '', content: '', sort_order: 0, enabled: 1 }
}
function edit(a: any) {
  editingId.value = a.id
  form.value = {
    title: a.title || '',
    content: a.content || '',
    sort_order: Number(a.sort_order || 0),
    enabled: a.enabled ? 1 : 0,
  }
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
async function save() {
  if (!form.value.title.trim() || !form.value.content.trim()) return showToast('请填写标题和内容')
  saving.value = true
  const payload = { id: editingId.value, ...form.value }
  const res = await adminApi(editingId.value ? 'update_announcement' : 'add_announcement', payload)
  saving.value = false
  if (res.code === 200) { showToast(editingId.value ? '已保存' : '已发布', 'success'); resetForm(); loadList() } else showToast(res.msg || '保存失败')
}
async function toggle(a: any) { const enabled = a.enabled ? 0 : 1; const res = await adminApi('toggle_announcement', { id: a.id, enabled }); if (res.code === 200) { a.enabled = enabled; showToast('已更新', 'success') } else showToast(res.msg || '操作失败') }
async function remove(a: any) { if (!(await mobileConfirm('确认删除公告？'))) return; const res = await adminApi('delete_announcement', { id: a.id }); if (res.code === 200) { showToast('已删除', 'success'); loadList() } else showToast(res.msg || '删除失败') }
onMounted(loadList)
</script>
