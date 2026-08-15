// 模拟后端服务器：在 8081 端口提供后台 API
const http = require('http')

function ok(data) {
  return { code: 200, msg: '', data }
}
function err(code, msg) {
  return { code, msg, data: null }
}

const tables = [
  { name: 'app_users', exists: true, row_count: 12345 },
  { name: 'user_feedback', exists: true, row_count: 678 },
  { name: 'wallpapers', exists: true, row_count: 89 },
  { name: 'admin_logs', exists: true, row_count: 4321 },
  { name: 'app_versions', exists: true, row_count: 15 },
  { name: 'announcements', exists: true, row_count: 7 },
  { name: 'login_logs', exists: true, row_count: 9999 },
  { name: 'device_bans', exists: true, row_count: 3 },
  { name: 'pretty_ids', exists: true, row_count: 120 },
  { name: 'server_settings', exists: true, row_count: 55 },
  { name: 'missing_table', exists: false, row_count: 0 },
]

const backups = [
  { name: 'backup_20260815_100000.sql', size: '2.3 MB', size_bytes: 2411724, created_at: '2026-08-15 10:00:00' },
  { name: 'backup_20260814_100000.sql', size: '2.1 MB', size_bytes: 2202009, created_at: '2026-08-14 10:00:00' },
]

const server = http.createServer((req, res) => {
  res.setHeader('Content-Type', 'application/json; charset=utf-8')
  res.setHeader('Access-Control-Allow-Origin', '*')
  res.setHeader('Access-Control-Allow-Headers', '*')
  res.setHeader('Access-Control-Allow-Methods', '*')
  if (req.method === 'OPTIONS') {
    res.writeHead(204)
    res.end()
    return
  }

  let body = ''
  req.on('data', (c) => (body += c))
  req.on('end', () => {
    let parsed = {}
    try { parsed = body ? JSON.parse(body) : {} } catch { parsed = {} }

    const url = new URL(req.url, 'http://localhost')
    const action = url.searchParams.get('action') || ''

    let result
    switch (action) {
      case 'admin_login':
        result = ok({ token: 'mock-token-123', admin_id: 1, username: 'admin', role: 'super_admin', avatar_url: '', expires_in: 3600 })
        break
      case 'admin_logout':
        result = ok(null)
        break
      case 'list_tables':
        result = ok({ tables })
        break
      case 'list_backups':
        result = ok({ backups, total: backups.length })
        break
      case 'view_table': {
        const name = parsed.table_name || ''
        const page = parsed.page || 1
        const t = tables.find((x) => x.name === name)
        if (!t || !t.exists) {
          result = err(400, '表不存在')
        } else {
          const rows = []
          const count = Math.min(t.row_count, 100)
          for (let i = 0; i < count; i++) {
            rows.push({ id: (page - 1) * 100 + i + 1, name: `row_${i}`, status: i % 2 ? 'active' : 'inactive' })
          }
          result = ok({ table: name, columns: ['id', 'name', 'status'], rows, total: t.row_count, page, pageSize: 100 })
        }
        break
      }
      case 'view_backup':
        result = ok({ content: '-- 备份内容\nCREATE TABLE test (id INT);\nINSERT INTO test VALUES (1);' })
        break
      case 'repair_database':
        result = ok({ created_tables: [], errors: [], summary: { created_tables_count: 0, added_columns_count: 0, added_indexes_count: 0, dropped_tables_count: 0 } })
        break
      case 'backup_db':
        result = ok({ filename: 'backup_20260815_110000.sql', size: '1 KB', tables: 10 })
        break
      case 'import_db':
        result = ok({ filename: 'x.sql', ok: 0, errors: 0 })
        break
      default:
        result = err(404, '接口不存在')
    }

    res.writeHead(200)
    res.end(JSON.stringify(result))
  })
})

server.listen(8081, '127.0.0.1', () => {
  console.log('mock server listening on 8081')
})
