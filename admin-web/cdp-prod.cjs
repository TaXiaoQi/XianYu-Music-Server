// CDP 驱动脚本：登录生产后台，点击查看内容，检查弹窗真实状态
const { spawn } = require('child_process')
const fs = require('fs')

const EDGE = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
const PORT = 9224
const APP = 'https://back.xymusic.cc'

function sleep(ms) { return new Promise(r => setTimeout(r, ms)) }

async function main() {
  const edge = spawn(EDGE, [
    '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
    `--remote-debugging-port=${PORT}`,
    '--user-data-dir=C:\\Users\\小奇\\AppData\\Local\\Temp\\edge-cdp-profile-prod',
    'about:blank',
  ], { stdio: 'ignore' })

  let version
  for (let i = 0; i < 30; i++) {
    try { const r = await fetch(`http://127.0.0.1:${PORT}/json/version`); version = await r.json(); break }
    catch { await sleep(500) }
  }
  if (!version) { console.log('FAIL: edge debug port not ready'); edge.kill(); return }

  const r = await fetch(`http://127.0.0.1:${PORT}/json/new?${encodeURIComponent('about:blank')}`, { method: 'PUT' })
  const page = await r.json()
  const ws = new WebSocket(page.webSocketDebuggerUrl)
  let msgId = 0
  const pending = new Map()
  const events = []
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data)
    if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id) }
    else if (msg.method) events.push(msg)
  }
  function send(method, params = {}) {
    return new Promise((resolve) => { const id = ++msgId; pending.set(id, resolve); ws.send(JSON.stringify({ id, method, params })) })
  }
  await new Promise(r => ws.onopen = r)
  await send('Page.enable'); await send('Runtime.enable'); await send('DOM.enable')
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false })
  await send('Page.navigate', { url: APP + '/login' })
  await sleep(4000)

  // 尝试登录
  const loginRes = await send('Runtime.evaluate', {
    expression: `(() => {
      const u = document.querySelector('input[type="text"], input[name="username"], input[placeholder*="账号"], input[placeholder*="用户名"]');
      const p = document.querySelector('input[type="password"]');
      if (!u || !p) return JSON.stringify({ ok: false, reason: 'inputs not found', html: document.body.innerText.slice(0, 200) });
      const setVal = (el, v) => { const s = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; s.call(el, v); el.dispatchEvent(new Event('input', { bubbles: true })); };
      setVal(u, 'admin'); setVal(p, 'adminadmin');
      const btn = Array.from(document.querySelectorAll('button')).find(b => /登\\s*录/.test(b.innerText));
      if (!btn) return JSON.stringify({ ok: false, reason: 'login btn not found' });
      btn.click();
      return JSON.stringify({ ok: true });
    })()`,
    returnByValue: true,
  })
  console.log('LOGIN:', loginRes.result.result.value)
  await sleep(4000)

  // 检查是否登录成功
  const after = await send('Runtime.evaluate', {
    expression: `JSON.stringify({ url: location.href, hasToken: !!localStorage.getItem('admin_token'), body: document.body.innerText.slice(0, 100) })`,
    returnByValue: true,
  })
  console.log('AFTER_LOGIN:', after.result.result.value)

  // 导航到数据库页
  await send('Page.navigate', { url: APP + '/database' })
  await sleep(4000)
  const dbState = await send('Runtime.evaluate', {
    expression: `JSON.stringify({ url: location.href, body: document.body.innerText.slice(0, 300) })`,
    returnByValue: true,
  })
  console.log('DB_PAGE:', dbState.result.result.value)

  // 点击查看内容
  const clickRes = await send('Runtime.evaluate', {
    expression: `(() => {
      const btns = Array.from(document.querySelectorAll('button')).filter(b => b.innerText.includes('查看内容'));
      if (!btns.length) return JSON.stringify({ clicked: false, count: 0 });
      btns[0].click();
      return JSON.stringify({ clicked: true, count: btns.length });
    })()`,
    returnByValue: true,
  })
  console.log('CLICK:', clickRes.result.result.value)
  await sleep(2500)

  // 检查弹窗状态
  const modalState = await send('Runtime.evaluate', {
    expression: `(() => {
      const backdrop = document.querySelector('.modal-backdrop');
      const dialog = document.querySelector('.modal-dialog');
      if (!backdrop) return JSON.stringify({ backdrop: false, body: document.body.innerText.slice(0, 200) });
      const bs = getComputedStyle(backdrop);
      const di = dialog ? (() => {
        const ds = getComputedStyle(dialog);
        const rect = dialog.getBoundingClientRect();
        return { display: ds.display, opacity: ds.opacity, visibility: ds.visibility, bg: ds.backgroundColor, rect: { x: rect.x, y: rect.y, w: rect.width, h: rect.height }, innerText: dialog.innerText.slice(0, 120) };
      })() : null;
      return JSON.stringify({ backdrop: true, backdropBg: bs.backgroundColor, dialog: di });
    })()`,
    returnByValue: true,
  })
  console.log('MODAL_STATE:', modalState.result.result.value)

  // 控制台错误
  const errs = events.filter(e => e.method === 'Runtime.exceptionThrown').map(e => {
    const d = e.params.exceptionDetails
    return (d.exception && d.exception.description || d.text || '').slice(0, 300)
  })
  console.log('JS_ERRORS:', JSON.stringify(errs))

  const shot = await send('Page.captureScreenshot', { format: 'png' })
  fs.writeFileSync('C:\\Users\\小奇\\AppData\\Local\\Temp\\prod-db-modal.png', Buffer.from(shot.result.data, 'base64'))
  console.log('SCREENSHOT saved')
  ws.close(); edge.kill()
}
main().catch(e => { console.log('ERROR:', e.message); process.exit(1) })