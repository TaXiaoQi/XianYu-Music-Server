<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">新增管理员</h3>
      <input v-model="form.username" class="mobile-input" placeholder="用户名" />
      <input v-model="form.password" class="mobile-input" type="password" placeholder="密码" />
      <select v-model="form.role" class="mobile-select"><option value="admin">管理员</option><option value="super_admin">超级管理员</option></select>
      <button class="mobile-btn primary" @click="add">新增</button>
    </div>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else class="mobile-list">
      <div v-for="a in list" :key="a.id" class="mobile-item">
        <div class="mobile-item-head"><div><div class="mobile-item-title">{{ a.username }}</div><div class="mobile-item-sub">{{ a.role }} · {{ a.email || '未绑定邮箱' }}</div></div><span class="mobile-badge" :class="a.status == 1 ? 'green' : 'red'">{{ a.status == 1 ? '正常' : '禁用' }}</span></div>
        <div class="mobile-actions"><button class="mobile-btn" @click="toggle(a)">{{ a.status == 1 ? '禁用' : '启用' }}</button><button class="mobile-btn danger" @click="remove(a)">删除</button></div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const loading = ref(false), list = ref<any[]>([])
const form = ref({ username: '', password: '', role: 'admin' })
async function load() { loading.value = true; const res = await adminApi<any>('list_admins'); list.value = res.code === 200 && res.data ? (res.data.list || []) : []; loading.value = false }
async function add() { const res = await adminApi('add_admin', form.value); if (res.code === 200) { showToast('已新增', 'success'); form.value = { username: '', password: '', role: 'admin' }; load() } else showToast(res.msg || '新增失败') }
async function toggle(a: any) { const res = await adminApi('toggle_admin_status', { id: a.id }); if (res.code === 200) { a.status = a.status == 1 ? 0 : 1; showToast('已更新', 'success') } else showToast(res.msg || '操作失败') }
async function remove(a: any) { if (!confirm('确认删除管理员？')) return; const res = await adminApi('delete_admin', { id: a.id }); if (res.code === 200) { showToast('已删除', 'success'); load() } else showToast(res.msg || '删除失败') }
onMounted(load)
</script>