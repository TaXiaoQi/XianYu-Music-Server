<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">用户协议</h3>
      <p class="mobile-muted">配置客户端账号登录、注册页弹窗展示的用户协议。</p>
      <div v-if="loading" class="mobile-empty">加载中...</div>
      <template v-else>
        <label class="mobile-field">
          <span>协议标题</span>
          <input v-model="form.title" class="mobile-input" placeholder="弦予音乐用户协议" />
        </label>
        <label class="mobile-field">
          <span>协议内容</span>
          <textarea v-model="form.content" class="mobile-textarea agreement-textarea" placeholder="请输入用户协议内容"></textarea>
        </label>
        <div class="mobile-actions">
          <button class="mobile-btn" :disabled="saving" @click="resetDefault">恢复初版</button>
          <button class="mobile-btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存协议' }}</button>
        </div>
      </template>
    </section>

    <section class="mobile-card">
      <h3 class="mobile-card-title">协议预览</h3>
      <div class="agreement-preview-title">{{ form.title || '弦予音乐用户协议' }}</div>
      <div class="agreement-preview-content">{{ form.content }}</div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm } from '@/utils/mobileDialog'
import './MobilePage.css'

interface UserAgreementConfig {
  title: string
  content: string
}

const defaultConfig: UserAgreementConfig = {
  title: '弦予音乐用户协议',
  content: `一、协议范围
本协议适用于弦予音乐客户端账号系统及相关云端同步、资料管理、统计上报、风控安全服务。用户注册、登录或继续使用账号功能，即表示已阅读并同意本协议。

二、账号注册与使用
用户应使用真实、有效的邮箱完成注册，并妥善保管账号、密码和邮箱验证码。因用户主动泄露、共享账号或使用非官方客户端造成的损失，由用户自行承担。

三、本地数据读取说明
为提供账号登录、设备安全识别、播放统计、同步和故障排查功能，账号系统可能读取或生成以下本地数据：本机设备标识、客户端版本、操作系统版本、设备型号、登录状态凭证、用户主动上传的头像、本地收藏、歌单、播放历史、听歌时长等音乐使用数据，以及软件运行错误日志。上述数据仅用于账号服务、安全风控、功能同步、异常定位和产品维护。

四、数据上报与安全
客户端启动、登录、注册、搜索、播放统计、错误反馈等行为可能向服务器上报必要信息，包括设备ID、IP地址、账号ID、客户端版本、操作系统版本、设备型号、行为时间和必要的请求参数。我们将尽合理努力保护数据安全，不会主动出售用户个人信息。

五、禁止行为
用户不得利用账号系统进行恶意攻击、批量注册、刷量、破解、逆向、绕过限制、上传违法违规内容、干扰服务器稳定性或侵犯他人权益。发现异常行为时，平台有权限制、封禁账号或设备。

六、封禁与申诉
若账号或设备因违反协议、安全风控或恶意行为被封禁，登录时将提示封禁状态及原因。用户如认为处理有误，可联系管理员并提供账号、设备ID及相关说明进行核查。

七、协议更新
平台可根据功能调整、安全要求或法律合规需要更新本协议。更新后继续使用账号功能，视为接受更新后的协议内容。`,
}

const loading = ref(true)
const saving = ref(false)
const form = ref<UserAgreementConfig>({ ...defaultConfig })

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<UserAgreementConfig>>('get_user_agreement_admin')
  if (res.code === 200 && res.data) {
    form.value = {
      title: String(res.data.title || defaultConfig.title),
      content: String(res.data.content || defaultConfig.content),
    }
  } else {
    showToast(res.msg || '加载协议失败')
  }
  loading.value = false
}

async function resetDefault() {
  if (!(await mobileConfirm('确定恢复为初版用户协议吗？恢复后需要点击保存才会生效。'))) return
  form.value = { ...defaultConfig }
}

async function save() {
  const title = form.value.title.trim()
  const content = form.value.content.trim()
  if (!title) return showToast('协议标题不能为空')
  if (content.length < 20) return showToast('协议内容过短')
  saving.value = true
  const res = await adminApi<UserAgreementConfig>('save_user_agreement', { title, content })
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
