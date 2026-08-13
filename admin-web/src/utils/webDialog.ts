/**
 * 后台（PC 端）统一弹窗工具
 * 替代浏览器原生 confirm() / prompt()
 * 样式：毛玻璃 + 圆角 + 阴影 + 居中，适配后台整体设计（见 main.css 中的 .web-dialog-*）
 */

export interface WebConfirmOptions {
  title?: string
  confirmText?: string
  cancelText?: string
  /** 危险操作时使用红色确认按钮 */
  danger?: boolean
}

export interface WebPromptOptions {
  title?: string
  confirmText?: string
  cancelText?: string
  placeholder?: string
}

function createOverlay(): HTMLDivElement {
  const overlay = document.createElement('div')
  overlay.className = 'web-dialog-overlay'
  return overlay
}

function createDialog(): HTMLDivElement {
  const dialog = document.createElement('div')
  dialog.className = 'web-dialog'
  return dialog
}

function animateIn(overlay: HTMLDivElement, dialog: HTMLDivElement) {
  document.body.appendChild(overlay)
  overlay.appendChild(dialog)
  requestAnimationFrame(() => {
    overlay.classList.add('show')
    dialog.classList.add('show')
  })
}

function closeDialog(overlay: HTMLDivElement) {
  const dialog = overlay.querySelector('.web-dialog')
  overlay.classList.remove('show')
  dialog?.classList.remove('show')
  setTimeout(() => overlay.remove(), 240)
}

/** 转义 HTML，防止注入 */
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
export function webConfirm(message: string, options: WebConfirmOptions = {}): Promise<boolean> {
  const {
    title = '操作确认',
    confirmText = '确认',
    cancelText = '取消',
    danger = true,
  } = options

  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()
    const confirmClass = danger ? 'danger' : 'primary'

    dialog.innerHTML = `
      <div class="web-dialog-title">${escapeHtml(title)}</div>
      <div class="web-dialog-body">${escapeHtml(message)}</div>
      <div class="web-dialog-actions">
        <button class="web-dialog-btn cancel" type="button">${escapeHtml(cancelText)}</button>
        <button class="web-dialog-btn confirm ${confirmClass}" type="button">${escapeHtml(confirmText)}</button>
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
    document.addEventListener('keydown', onKey)
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        document.removeEventListener('keydown', onKey)
        done(false)
      } else if (e.key === 'Enter') {
        document.removeEventListener('keydown', onKey)
        done(true)
      }
    }

    animateIn(overlay, dialog)
  })
}

/**
 * 输入弹窗（替代 prompt）
 * @returns Promise<string | null> null 表示用户取消
 */
export function webPrompt(message: string, defaultValue = '', options: WebPromptOptions = {}): Promise<string | null> {
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
      <div class="web-dialog-title">${escapeHtml(title)}</div>
      <div class="web-dialog-body">${escapeHtml(message)}</div>
      <input class="web-dialog-input" type="text" value="${escapeHtml(defaultValue)}" placeholder="${escapeHtml(placeholder)}" />
      <div class="web-dialog-actions">
        <button class="web-dialog-btn cancel" type="button">${escapeHtml(cancelText)}</button>
        <button class="web-dialog-btn confirm primary" type="button">${escapeHtml(confirmText)}</button>
      </div>
    `

    const input = dialog.querySelector('.web-dialog-input') as HTMLInputElement
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
    document.addEventListener('keydown', onKey)
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        document.removeEventListener('keydown', onKey)
        done(null)
      }
    }

    animateIn(overlay, dialog)
    setTimeout(() => {
      input.focus()
      input.select()
    }, 100)
  })
}

export interface WebAction {
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
 * 居中展示一组操作项，与其他 webDialog 弹窗样式统一。
 * @returns Promise<string | null> 返回所选操作的 key，取消返回 null
 */
export function webActionMenu(title: string, actions: WebAction[]): Promise<string | null> {
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
      <div class="web-dialog-title">${escapeHtml(title)}</div>
      <div class="web-dialog-actions-list">${items}</div>
      <div class="web-dialog-actions">
        <button class="web-dialog-btn cancel" type="button">取消</button>
      </div>
    `

    const cancelBtn = dialog.querySelector('.cancel') as HTMLButtonElement
    const actionBtns = dialog.querySelectorAll('.web-dialog-actions-list .action')

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
    document.addEventListener('keydown', onKey)
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        document.removeEventListener('keydown', onKey)
        done(null)
      }
    }

    animateIn(overlay, dialog)
  })
}