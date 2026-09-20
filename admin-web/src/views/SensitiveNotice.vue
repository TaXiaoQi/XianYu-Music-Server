<template>
  <div class="sensitive-page">
    <div class="sensitive-card">
      <div class="sensitive-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          <path d="M12 15v3"/>
        </svg>
      </div>
      <h2 class="sensitive-title">涉密资料</h2>
      <p class="sensitive-desc">权限不足，无法展示</p>
      <span class="sensitive-tag">访客账号仅可查看，如需操作请联系超级管理员</span>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.sensitive-page {
  min-height: calc(100vh - 140px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.sensitive-card {
  width: min(460px, 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 46px 34px;
  border-radius: 22px;
  border: 1px solid var(--border, #ebeef5);
  background: var(--card, #ffffff);
  box-shadow: var(--shadow-soft, 0 6px 24px rgba(15, 23, 42, 0.06));
  animation: sensitive-in 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}
.sensitive-icon {
  width: 76px;
  height: 76px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  color: var(--accent, #e04a4a);
  background: var(--accent-soft, rgba(224, 74, 74, 0.09));
  margin-bottom: 18px;
}
.sensitive-title {
  font-size: 22px;
  font-weight: 800;
  color: var(--text, #1f2d3d);
  margin: 0 0 8px;
  letter-spacing: 2px;
}
.sensitive-desc {
  font-size: 15px;
  color: var(--text-muted, #909399);
  margin: 0 0 18px;
}
.sensitive-tag {
  font-size: 12px;
  color: var(--accent, #e04a4a);
  background: var(--accent-soft, rgba(224, 74, 74, 0.09));
  padding: 6px 14px;
  border-radius: 999px;
}
@keyframes sensitive-in {
  from { opacity: 0; transform: translateY(10px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}
</style>