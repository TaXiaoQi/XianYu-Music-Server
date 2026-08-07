/**
 * EmailForgot 组件测试
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import EmailForgot from '../EmailForgot.vue'
import * as emailApi from '@/api/email'

// mock vue-router
const mockPush = vi.fn()
const mockReplace = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({ push: mockPush, replace: mockReplace }),
}))

// mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, val: string) => { store[key] = val },
    removeItem: (key: string) => { delete store[key] },
    clear: () => { store = {} },
  }
})()
vi.stubGlobal('localStorage', localStorageMock)

// RouterLink stub
const RouterLinkStub = {
  template: '<a :href="to"><slot /></a>',
  props: ['to'],
}

function mountComponent() {
  return mount(EmailForgot, {
    global: {
      stubs: { RouterLink: RouterLinkStub },
    },
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.useFakeTimers()
  localStorageMock.clear()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('EmailForgot.vue', () => {
  it('应渲染标题和所有表单字段', () => {
    const wrapper = mountComponent()
    expect(wrapper.find('h1').text()).toBe('找回密码')
    const inputs = wrapper.findAll('input')
    // email, code, password, password2
    expect(inputs.length).toBe(4)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
    expect(wrapper.find('.code-btn').exists()).toBe(true)
  })

  it('应渲染返回登录链接指向 /email/login', () => {
    const wrapper = mountComponent()
    const links = wrapper.findAll('a')
    const loginLink = links.find(l => l.text().includes('返回登录'))
    expect(loginLink?.attributes('href')).toBe('/email/login')
  })

  it('空邮箱点击获取验证码不应调用 API', async () => {
    const sendSpy = vi.spyOn(emailApi, 'sendCode')

    const wrapper = mountComponent()
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    expect(sendSpy).not.toHaveBeenCalled()
  })

  it('非法邮箱点击获取验证码不应调用 API', async () => {
    const sendSpy = vi.spyOn(emailApi, 'sendCode')

    const wrapper = mountComponent()
    await wrapper.find('input[type="email"]').setValue('bad-email')
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    expect(sendSpy).not.toHaveBeenCalled()
  })

  it('合法邮箱获取验证码成功后应启动倒计时', async () => {
    vi.spyOn(emailApi, 'sendCode').mockResolvedValueOnce({
      code: 200,
      msg: '验证码已发送',
      data: null,
    })

    const wrapper = mountComponent()
    await wrapper.find('input[type="email"]').setValue('user@test.com')
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    const btn = wrapper.find('.code-btn')
    expect(btn.text()).toContain('60s')
    expect(btn.attributes('disabled')).toBeDefined()

    vi.advanceTimersByTime(30000)
    await flushPromises()
    expect(wrapper.find('.code-btn').text()).toContain('30s')
  })

  it('重置成功应跳转到登录页', async () => {
    vi.spyOn(emailApi, 'emailResetPassword').mockResolvedValueOnce({
      code: 200,
      msg: '密码已重置成功',
      data: null,
    })

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('user@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')   // code
    await inputs[2].setValue('NewPass123') // password
    await inputs[3].setValue('NewPass123') // password2

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    vi.advanceTimersByTime(1200)
    await flushPromises()

    expect(mockPush).toHaveBeenCalledWith('/email/login')
  })

  it('密码不一致应阻止提交', async () => {
    const resetSpy = vi.spyOn(emailApi, 'emailResetPassword')

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('user@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')
    await inputs[2].setValue('pass123')
    await inputs[3].setValue('different')

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    expect(resetSpy).not.toHaveBeenCalled()
    expect(mockPush).not.toHaveBeenCalled()
  })

  it('重置失败不应跳转', async () => {
    vi.spyOn(emailApi, 'emailResetPassword').mockResolvedValueOnce({
      code: 400,
      msg: '该邮箱尚未注册',
      data: null,
    })

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('user@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')
    await inputs[2].setValue('NewPass123')
    await inputs[3].setValue('NewPass123')

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()
    vi.advanceTimersByTime(2000)
    await flushPromises()

    expect(mockPush).not.toHaveBeenCalled()
  })

  it('loading 状态应禁用提交按钮', async () => {
    vi.spyOn(emailApi, 'emailResetPassword').mockReturnValue(new Promise(() => {}))

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('user@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')
    await inputs[2].setValue('pass123')
    await inputs[3].setValue('pass123')

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    const btn = wrapper.find('button[type="submit"]')
    expect(btn.attributes('disabled')).toBeDefined()
    expect(btn.text()).toContain('重置中')
  })
})
