// Webhook 通知：配置读取/保存与测试推送
import { reactive, ref } from 'vue'
import { showToast } from '@/api/client'
import { getWebhookConfig, saveWebhookConfig as saveWebhookConfigApi, testWebhook as testWebhookApi } from '@/api/commtool'
import type { WebhookConfig, WebhookTestResult } from '@/api/commtool'

export function useCommWebhook() {
  const webhookForm = reactive<WebhookConfig>({
    enabled: false,
    url: '',
    method: 'POST',
    headers: '',
    body_template: '',
    modules: {
      wh_wallpaper: true,
      wh_avatar: true,
      wh_nickname: true,
      wh_feedback: true,
    },
  })
  const webhookSaving = ref(false)
  const webhookTesting = ref(false)
  const webhookTestResult = ref<WebhookTestResult | null>(null)

  async function loadWebhookConfig() {
    const res = await getWebhookConfig()
    if (res.code === 200 && res.data) {
      webhookForm.enabled = res.data.enabled
      webhookForm.url = res.data.url || ''
      webhookForm.method = res.data.method || 'POST'
      webhookForm.headers = res.data.headers || ''
      webhookForm.body_template = res.data.body_template || ''
      if (res.data.modules) {
        webhookForm.modules = { ...webhookForm.modules, ...res.data.modules }
      }
    }
  }

  async function saveWebhookConfig() {
    if (webhookForm.enabled && !webhookForm.url.trim()) {
      showToast('启用 Webhook 时请输入回调地址')
      return
    }
    webhookSaving.value = true
    const res = await saveWebhookConfigApi({
      enabled: webhookForm.enabled,
      url: webhookForm.url.trim(),
      method: webhookForm.method,
      headers: webhookForm.headers,
      body_template: webhookForm.body_template,
      modules: webhookForm.modules,
    })
    webhookSaving.value = false
    if (res.code === 200) {
      showToast(res.msg || '已保存', 'success')
    } else {
      showToast(res.msg || '保存失败')
    }
  }

  async function testWebhook() {
    if (!webhookForm.url.trim()) {
      showToast('请输入回调地址')
      return
    }
    webhookTesting.value = true
    const res = await testWebhookApi({
      url: webhookForm.url.trim(),
      method: webhookForm.method,
      headers: webhookForm.headers,
      body_template: webhookForm.body_template,
    })
    webhookTesting.value = false
    if (res.code === 200 && res.data) {
      webhookTestResult.value = res.data
      showToast('测试完成', 'success')
    } else {
      webhookTestResult.value = null
      showToast(res.msg || '测试失败')
    }
  }

  function toggleWebhookModule(key: string) {
    webhookForm.modules['wh_' + key] = !webhookForm.modules['wh_' + key]
  }

  return { webhookForm, webhookSaving, webhookTesting, webhookTestResult, loadWebhookConfig, saveWebhookConfig, testWebhook, toggleWebhookModule }
}
