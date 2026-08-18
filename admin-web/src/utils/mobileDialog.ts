/**
 * 移动端统一弹窗工具
 * 替代浏览器原生 confirm() / prompt()
 * 样式：毛玻璃 + 圆角 + 阴影 + 居中
 */

/**
 * Vue Transition @before-leave 钩子：在淡出动画开始前立即移除 backdrop-filter，
 * 让浏览器先卸载模糊层，避免遮罩在 opacity 过渡期间残留。
 */
export function removeBackdropBlur(el: Element) {
  const htmlEl = el as HTMLElement
  htmlEl.style.backdropFilter = 'none'
  htmlEl.style.setProperty('-webkit-backdrop-filter', 'none')
}

export interface MobileConfirmOptions {
  title?: string
  confirmText?: string
  cancelText?: string
  /** 危险操作时使用红色确认按钮 */
  danger?: boolean
}

export interface MobilePromptOptions {
  title?: string
  confirmText?: string
  cancelText?: string
  placeholder?: string
}

function createOverlay(): HTMLDivElement {
  const overlay = document.createElement('div')
  overlay.className = 'mobile-dialog-overlay'
  return overlay
}

function createDialog(): HTMLDivElement {
  const dialog = document.createElement('div')
  dialog.className = 'mobile-dialog'
  return dialog
}

function animateIn(overlay: HTMLDivElement, dialog: HTMLDivElement) {
  // 初始不可见状态
  overlay.style.opacity = '0'
  dialog.style.opacity = '0'
  dialog.style.transform = 'scale(0.94)'

  document.body.appendChild(overlay)
  overlay.appendChild(dialog)

  // 强制 reflow 确保初始状态已渲染
  void overlay.offsetHeight

  // 设置过渡并触发动画到最终状态
  overlay.style.transition = 'opacity 0.24s cubic-bezier(0.16, 1, 0.3, 1)'
  dialog.style.transition = 'transform 0.24s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.24s'
  overlay.style.opacity = '1'
  dialog.style.opacity = '1'
  dialog.style.transform = 'scale(1)'
}

function closeDialog(overlay: HTMLDivElement) {
  const dialog = overlay.querySelector('.mobile-dialog') as HTMLDivElement | null
  // 立即移除 backdrop-filter，避免模糊效果在 opacity 透明后残留
  overlay.style.setProperty('backdrop-filter', 'none')
  overlay.style.setProperty('-webkit-backdrop-filter', 'none')
  // 等待一帧让浏览器先卸载模糊层，再开始淡出，避免遮罩残留
  requestAnimationFrame(() => {
    overlay.style.opacity = '0'
    if (dialog) {
      dialog.style.opacity = '0'
      dialog.style.transform = 'scale(0.96)'
    }
    setTimeout(() => overlay.remove(), 240)
  })
}

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/\n/g, '<br/>')
}

/**
 * 确认弹窗（替代 confirm）
 * @returns Promise<boolean>
 */
export function mobileConfirm(message: string, options: MobileConfirmOptions = {}): Promise<boolean> {
  const {
    title = '操作确认',
    confirmText = '确认',
    cancelText = '取消',
    danger = false,
  } = options

  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()
    const confirmClass = danger ? 'danger' : ''

    dialog.innerHTML = `
      <div class="mobile-dialog-title">${escapeHtml(title)}</div>
      <div class="mobile-dialog-body">${escapeHtml(message)}</div>
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn cancel" type="button">${escapeHtml(cancelText)}</button>
        <button class="mobile-dialog-btn confirm ${confirmClass}" type="button">${escapeHtml(confirmText)}</button>
      </div>
    `

    const cancelBtn = dialog.querySelector('.cancel') as HTMLButtonElement
    const confirmBtn = dialog.querySelector('.confirm') as HTMLButtonElement

    const done = (result: boolean) => {
      closeDialog(overlay)
      resolve(result)
    }

    cancelBtn.onclick = () => done(false)
    confirmBtn.onclick = () => done(true)
    overlay.onclick = (e) => {
      if (e.target === overlay) done(false)
    }

    animateIn(overlay, dialog)
    setTimeout(() => confirmBtn.focus(), 100)
  })
}

/**
 * 输入弹窗（替代 prompt）
 * @returns Promise<string | null> null 表示用户取消
 */
export function mobilePrompt(message: string, defaultValue = '', options: MobilePromptOptions = {}): Promise<string | null> {
  const {
    title = '输入',
    confirmText = '确定',
    cancelText = '取消',
    placeholder = '',
  } = options

  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()

    dialog.innerHTML = `
      <div class="mobile-dialog-title">${escapeHtml(title)}</div>
      <div class="mobile-dialog-body">${escapeHtml(message)}</div>
      <input class="mobile-dialog-input" type="text" value="${escapeHtml(defaultValue)}" placeholder="${escapeHtml(placeholder)}" />
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn cancel" type="button">${escapeHtml(cancelText)}</button>
        <button class="mobile-dialog-btn confirm" type="button">${escapeHtml(confirmText)}</button>
      </div>
    `

    const input = dialog.querySelector('.mobile-dialog-input') as HTMLInputElement
    const cancelBtn = dialog.querySelector('.cancel') as HTMLButtonElement
    const confirmBtn = dialog.querySelector('.confirm') as HTMLButtonElement

    const done = (result: string | null) => {
      closeDialog(overlay)
      resolve(result)
    }

    cancelBtn.onclick = () => done(null)
    confirmBtn.onclick = () => done(input.value)
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') done(input.value)
    })
    overlay.onclick = (e) => {
      if (e.target === overlay) done(null)
    }

    animateIn(overlay, dialog)
    setTimeout(() => {
      input.focus()
      input.select()
    }, 100)
  })
}

/**
 * 信息弹窗（仅一个确认按钮，用于展示通知/提示）
 * @returns Promise<void>
 */
export function mobileInfo(message: string, options: { title?: string; confirmText?: string } = {}): Promise<void> {
  const { title = '提示', confirmText = '知道了' } = options

  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()

    dialog.innerHTML = `
      <div class="mobile-dialog-title">${escapeHtml(title)}</div>
      <div class="mobile-dialog-body">${escapeHtml(message)}</div>
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn confirm" type="button">${escapeHtml(confirmText)}</button>
      </div>
    `

    const confirmBtn = dialog.querySelector('.confirm') as HTMLButtonElement
    const done = () => {
      closeDialog(overlay)
      resolve()
    }

    confirmBtn.onclick = done
    overlay.onclick = (e) => {
      if (e.target === overlay) done()
    }

    animateIn(overlay, dialog)
    setTimeout(() => confirmBtn.focus(), 100)
  })
}

export interface MobileAction {
  key: string
  label: string
  /** 危险操作（红色文字） */
  danger?: boolean
  /** 成功/启用类操作（绿色文字） */
  success?: boolean
  /** 是否显示（默认 true） */
  show?: boolean
}

/**
 * 操作菜单弹窗（替代下拉菜单）
 * 居中展示一组操作项，与其他移动端弹窗样式统一。
 * @returns Promise<string | null> 返回所选操作的 key，取消返回 null
 */
export function mobileActionMenu(title: string, actions: MobileAction[]): Promise<string | null> {
  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()

    const items = actions
      .filter((a) => a.show !== false)
      .map((a) => {
        const cls = a.danger ? 'action danger' : a.success ? 'action success' : 'action'
        return `<button class="${cls}" type="button" data-key="${escapeHtml(a.key)}">${escapeHtml(a.label)}</button>`
      })
      .join('')

    dialog.innerHTML = `
      <div class="mobile-dialog-title">${escapeHtml(title)}</div>
      <div class="mobile-dialog-actions-list">${items}</div>
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn cancel" type="button">取消</button>
      </div>
    `

    const cancelBtn = dialog.querySelector('.cancel') as HTMLButtonElement
    const actionBtns = dialog.querySelectorAll('.mobile-dialog-actions-list .action')

    const done = (result: string | null) => {
      closeDialog(overlay)
      resolve(result)
    }

    cancelBtn.onclick = () => done(null)
    actionBtns.forEach((btn) => {
      btn.addEventListener('click', () => done((btn as HTMLButtonElement).dataset.key || null))
    })
    overlay.onclick = (e) => {
      if (e.target === overlay) done(null)
    }

    animateIn(overlay, dialog)
  })
}