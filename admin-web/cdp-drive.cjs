// CDP 驱动脚本：加载真实应用，点击查看内容，检查弹窗状态
const { spawn } = require('child_process')
const fs = require('fs')

const EDGE = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe'
const PORT = 9222
const APP = 'http://localhost:3000'

function sleep(ms) { return new Promise(r => setTimeout(r, ms)) }

async function main() {
  const edge = spawn(EDGE, [
    '--headless=new',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    `--remote-debugging-port=${PORT}`,
    '--user-data-dir=C:\\Users\\小奇\\AppData\\Local\\Temp\\edge-cdp-profile',
    'about:blank',
  ], { stdio: 'ignore' })

  let version
  for (let i = 0; i < 30; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/json/version`)
      version = await r.json()
      break
    } catch { await sleep(500) }
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
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg)
      pending.delete(msg.id)
    } else if (msg.method) {
      events.push(msg)
    }
  }

  function send(method, params = {}) {
    return new Promise((resolve) => {
      const id = ++msgId
      pending.set(id, resolve)
      ws.send(JSON.stringify({ id, method, params }))
    })
  }

  await new Promise(r => ws.onopen = r)

  await send('Page.enable')
  await send('Runtime.enable')
  await send('DOM.enable')
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 900, deviceScaleFactor: 1, mobile: false })

  await send('Page.navigate', { url: APP + '/login' })
  await sleep(2500)

  await send('Runtime.evaluate', {
    expression: `
      localStorage.setItem('admin_token', 'mock-token-123');
      localStorage.setItem('admin_user', JSON.stringify({ id: 1, username: 'admin', role: 'super_admin', avatar_url: '' }));
      'ok';
    `,
  })

  await send('Page.navigate', { url: APP + '/database' })
  await sleep(3000)

  let evalRes = await send('Runtime.evaluate', {
    expression: `(() => {
      const rows = document.querySelectorAll('.data-table tbody tr');
      const btns = document.querySelectorAll('.btn-sm');
      return JSON.stringify({
        url: location.href,
        tableRows: rows.length,
        viewBtns: btns.length,
        btnTexts: Array.from(btns).slice(0, 5).map(b => b.textContent.trim()),
      });
    })()`,
    returnByValue: true,
  })
  console.log('STATE1:', evalRes.result.result.value)

  evalRes = await send('Runtime.evaluate', {
    expression: `(() => {
      const btns = Array.from(document.querySelectorAll('.btn-sm'));
      const viewBtn = btns.find(b => b.textContent.includes('查看内容'));
      if (viewBtn) { viewBtn.click(); return 'clicked'; }
      return 'not found';
    })()`,
    returnByValue: true,
  })
  console.log('CLICK:', evalRes.result.result.value)

  await sleep(1200)

  evalRes = await send('Runtime.evaluate', {
    expression: `(() => {
      const backdrop = document.querySelector('.modal-backdrop');
      const dialog = document.querySelector('.modal-dialog');
      if (!backdrop) return JSON.stringify({ backdrop: false });
      const bs = getComputedStyle(backdrop);
      const dialogInfo = dialog ? (() => {
        const ds = getComputedStyle(dialog);
        const rect = dialog.getBoundingClientRect();
        return {
          display: ds.display,
          opacity: ds.opacity,
          visibility: ds.visibility,
          transform: ds.transform,
          animation: ds.animation,
          rect: { x: rect.x, y: rect.y, w: rect.width, h: rect.height },
          bg: ds.backgroundColor,
          innerText: dialog.innerText.slice(0, 100),
        };
      })() : null;
      return JSON.stringify({
        backdrop: true,
        backdropDisplay: bs.display,
        backdropOpacity: bs.opacity,
        backdropZ: bs.zIndex,
        dialog: dialogInfo,
      });
    })()`,
    returnByValue: true,
  })
  console.log('STATE2:', evalRes.result.result.value)

  const shot = await send('Page.captureScreenshot', { format: 'png' })
  fs.writeFileSync('C:\\Users\\小奇\\AppData\\Local\\Temp\\real-app-modal.png', Buffer.from(shot.result.data, 'base64'))
  console.log('SCREENSHOT saved')

  const errors = events.filter(e => e.method === 'Runtime.exceptionThrown' || (e.method === 'Runtime.consoleAPICalled' && e.params.type === 'error'))
  console.log('CONSOLE_ERRORS:', JSON.stringify(errors.map(e => e.params).slice(0, 5)))

  ws.close()
  edge.kill()
}

main().catch(e => { console.log('ERROR:', e.message); process.exit(1) })
