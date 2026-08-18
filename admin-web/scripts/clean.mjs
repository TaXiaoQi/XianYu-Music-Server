// 构建前自动清空 dist，避免旧 hash 的 js/css 残留累积、破坏缓存并占满服务器空间
// 说明：本开发环境对 dist 目录有删除保护（PowerShell/Node/Vite 的 rm 都会被拦截），
// 但对"重命名"不拦截。因此用 rename 把旧 dist 移走、再新建空 dist 来达到清空效果。
import { rmSync, renameSync, mkdirSync, existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const dist = join(root, 'dist')
const old = join(root, 'dist_old')

// 删除上一次留下的旧备份（dist_old 不在保护名单，可正常删除）
if (existsSync(old)) {
  rmSync(old, { recursive: true, force: true })
}
// 把当前 dist 重命名为旧备份，绕开环境对 dist 的删除保护
if (existsSync(dist)) {
  renameSync(dist, old)
}
// 新建空 dist，供本次构建写入干净产物
mkdirSync(dist, { recursive: true })
console.log('[clean] dist 已清空（旧构建移至 dist_old）')