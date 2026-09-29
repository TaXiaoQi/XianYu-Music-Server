<template>
  <div class="agreement-page">
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">部署文档</h2>
        <p class="page-desc">配置官网「部署文档」页展示的内容，保存后官网实时生效。未保存自定义内容时，官网展示默认初版文档。</p>
      </div>
      <div class="actions">
        <button class="btn-secondary" :disabled="loading || saving" @click="resetDefault">恢复初版</button>
        <button class="btn-save" :disabled="loading || saving" @click="save">
          <span v-if="saving" class="spinner"></span>
          {{ saving ? '保存中...' : '保存文档' }}
        </button>
      </div>
      </div>
    </Transition>

    <Transition name="fade-up" appear>
      <div v-if="loading" class="state-box">
      <span class="loader"></span>
      加载中...
    </div>
      <div v-else class="config-card">
      <label class="field">
        <span>文档标题</span>
        <input v-model="form.title" type="text" placeholder="弦予音乐服务端部署文档" />
      </label>

      <label class="field content-field">
        <span>文档内容（支持轻量排版：## 小节标题、- 列表、``` 代码块）</span>
        <textarea v-model="form.content" placeholder="请输入部署文档内容"></textarea>
      </label>

      <div class="preview-card">
        <div class="preview-title">{{ form.title || '弦予音乐服务端部署文档' }}</div>
        <div class="preview-content">{{ form.content }}</div>
      </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

interface DeployDocConfig {
  title: string
  content: string
}

const defaultConfig: DeployDocConfig = {
  title: '弦予音乐服务端部署文档',
  content: `## 一、环境要求
- Rust 1.85+（含 cargo）
- Node.js 18+（仅构建管理后台时需要）
- MySQL 5.7+ / 8.0（可选：不配置时自动降级为本地缓存模式）
- Linux / Windows / macOS 均可部署

## 二、获取源码
\`\`\`
git clone https://github.com/TaXiaoQi/XianYu-Music-Server.git
cd XianYu-Music-Server
\`\`\`

## 三、服务端配置
首次启动前在 server 目录创建 config.json：
\`\`\`
{
  "host": "0.0.0.0",
  "port": 8080,
  "database_url": "mysql://user:pass@127.0.0.1:3306/xianyu_music"
}
\`\`\`
数据库连接失败时自动降级为本地缓存模式（数据存储于 data/local_state.json），无需 MySQL 也可运行，响应结构与数据库模式逐字段一致。

## 四、编译与启动
\`\`\`
cd server
cargo build --release
./target/release/server
\`\`\`
也可使用仓库根目录的 start.bat（Windows）/ start.sh（Linux/macOS）一键启动。

## 五、管理后台
- 构建：cd admin-web && npm install && npm run build，产物 dist/ 由服务端直接托管
- 访问 http://127.0.0.1:8080/admin，首次使用请及时修改默认管理员密码

## 六、反向代理（Nginx 示例）
\`\`\`
location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
location /ws {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
\`\`\`
WebSocket（/ws 路径）必须包含 Upgrade 与 Connection 升级头，否则协同等实时功能无法连接。

## 七、常见问题
- 端口占用：修改 config.json 的 port 后重启服务
- 忘记管理员密码：本地缓存模式删除 data 目录下的管理员状态后重启；数据库模式请在管理后台用其他超管重置
- 更多问题请到 GitHub Issues 反馈：https://github.com/TaXiaoQi/XianYu-Music-Server/issues`,
}

const loading = ref(true)
const saving = ref(false)
const form = ref<DeployDocConfig>({ ...defaultConfig })

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<DeployDocConfig>>('get_deploy_doc_admin')
  if (res.code === 200 && res.data) {
    form.value = {
      title: String(res.data.title || defaultConfig.title),
      content: String(res.data.content || defaultConfig.content),
    }
  } else {
    showToast(res.msg || '加载部署文档失败')
  }
  loading.value = false
}

async function resetDefault() {
  const ok = await webConfirm('确定恢复为初版部署文档吗？恢复后需要点击保存才会生效。', { title: '恢复初版文档', confirmText: '确认恢复' })
  if (!ok) return
  form.value = { ...defaultConfig }
}

async function save() {
  const title = form.value.title.trim()
  const content = form.value.content.trim()
  if (!title) {
    showToast('文档标题不能为空')
    return
  }
  if (content.length < 20) {
    showToast('文档内容过短')
    return
  }
  saving.value = true
  const res = await adminApi<DeployDocConfig>('save_deploy_doc', { title, content })
  saving.value = false
  if (res.code === 200) {
    showToast('保存成功', 'success')
    form.value = { title, content }
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.agreement-page {
  max-width: 980px;
  margin: 0 auto;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 20px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  margin: 0 0 6px;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.6;
}
.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.btn-save,
.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: none;
  border-radius: 10px;
  padding: 10px 18px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-save {
  background: var(--accent);
  color: var(--white);
}
.btn-secondary {
  background: var(--control-bg);
  color: var(--text-light);
  border: 1px solid var(--border);
}
.btn-save:hover:not(:disabled),
.btn-secondary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.12);
}
.btn-save:disabled,
.btn-secondary:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
.spinner,
.loader {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.35);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.loader {
  width: 24px;
  height: 24px;
  border-color: var(--border);
  border-top-color: var(--accent);
}
.state-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 220px;
  color: var(--text-muted);
}
.config-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin-bottom: 18px;
}
.field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field input,
.field textarea {
  border: 1.5px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  outline: none;
  background: #fafafa;
  font-size: 14px;
  color: var(--text);
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus,
.field textarea:focus {
  border-color: var(--accent);
  background: var(--card-solid);
}
.content-field textarea {
  min-height: 420px;
  resize: vertical;
  line-height: 1.7;
  font-family: ui-monospace, Consolas, monospace;
}
.preview-card {
  margin-top: 18px;
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px;
  background: var(--control-bg);
}
.preview-title {
  font-weight: 800;
  font-size: 16px;
  margin-bottom: 12px;
}
.preview-content {
  white-space: pre-wrap;
  color: var(--text-light);
  line-height: 1.7;
  font-size: 13px;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (max-width: 720px) {
  .page-header {
    flex-direction: column;
  }
  .actions {
    width: 100%;
  }
  .btn-save,
  .btn-secondary {
    flex: 1;
    justify-content: center;
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
