/**
 * 移动端统一弹窗工具
 * 替代浏览器原生 confirm() / prompt()
 * 样式：毛玻璃 + 圆角 + 阴影 + 居中
 */

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
  document.body.appendChild(overlay)
  overlay.appendChild(dialog)
  requestAnimationFrame(() => {
    overlay.classList.add('show')
    dialog.classList.add('show')
  })
}

function closeDialog(overlay: HTMLDivElement) {
  const dialog = overlay.querySelector('.mobile-dialog')
  overlay.classList.remove('show')
  dialog?.classList.remove('show')
  setTimeout(() => overlay.remove(), 260)
}

/**
 * 确认弹窗（替代 confirm）
 * @returns Promise<boolean>
 */
export function mobileConfirm(message: string, title = '操作确认'): Promise<boolean> {
  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()

    dialog.innerHTML = `
      <div class="mobile-dialog-title">${title}</div>
      <div class="mobile-dialog-body">${message}</div>
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn cancel">取消</button>
        <button class="mobile-dialog-btn confirm">确认</button>
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
export function mobilePrompt(message: string, defaultValue = ''): Promise<string | null> {
  return new Promise((resolve) => {
    const overlay = createOverlay()
    const dialog = createDialog()

    dialog.innerHTML = `
      <div class="mobile-dialog-title">输入</div>
      <div class="mobile-dialog-body">${message}</div>
      <input class="mobile-dialog-input" type="text" value="${defaultValue.replace(/"/g, '&quot;')}" />
      <div class="mobile-dialog-actions">
        <button class="mobile-dialog-btn cancel">取消</button>
        <button class="mobile-dialog-btn confirm">确定</button>
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
