<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">部署文档</h3>
      <p class="mobile-muted">配置官网「部署文档」页展示的内容，保存后官网实时生效。</p>
      <div v-if="loading" class="mobile-empty">加载中...</div>
      <template v-else>
        <label class="mobile-field">
          <span>文档标题</span>
          <input v-model="form.title" class="mobile-input" placeholder="弦予音乐服务端部署文档" />
        </label>
        <label class="mobile-field">
          <span>文档内容（支持轻量排版：## 小节标题、- 列表、``` 代码块）</span>
          <textarea v-model="form.content" class="mobile-textarea agreement-textarea" placeholder="请输入部署文档内容"></textarea>
        </label>
        <div class="mobile-actions">
          <button class="mobile-btn" :disabled="saving" @click="resetDefault">恢复初版</button>
          <button class="mobile-btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存文档' }}</button>
        </div>
      </template>
    </section>

    <section class="mobile-card">
      <h3 class="mobile-card-title">文档预览</h3>
      <div class="agreement-preview-title">{{ form.title || '弦予音乐服务端部署文档' }}</div>
      <div class="agreement-preview-content">{{ form.content }}</div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm } from '@/utils/mobileDialog'
import './MobilePage.css'

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
  if (!(await mobileConfirm('确定恢复为初版部署文档吗？恢复后需要点击保存才会生效。'))) return
  form.value = { ...defaultConfig }
}

async function save() {
  const title = form.value.title.trim()
  const content = form.value.content.trim()
  if (!title) return showToast('文档标题不能为空')
  if (content.length < 20) return showToast('文档内容过短')
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
.agreement-textarea {
  min-height: 300px;
  line-height: 1.7;
  font-family: ui-monospace, Consolas, monospace;
}
.agreement-preview-title {
  margin-bottom: 10px;
  color: var(--text);
  font-size: 15px;
  font-weight: 850;
}
.agreement-preview-content {
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-light);
  font-size: 12px;
  line-height: 1.7;
}
</style>
