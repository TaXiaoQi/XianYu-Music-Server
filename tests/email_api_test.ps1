<#
.SYNOPSIS
    邮箱注册登录模块 API 集成测试脚本
.DESCRIPTION
    对运行中的 Rust 后端执行邮箱模块的完整流程测试：
    发送验证码 → 注册 → 登录 → 获取用户信息 → 重置密码 → 重新登录
.PARAMETER BaseUrl
    后端 API 地址，默认 http://127.0.0.1:8081
.PARAMETER TestEmail
    测试用邮箱地址
.EXAMPLE
    .\tests\email_api_test.ps1
    .\tests\email_api_test.ps1 -BaseUrl http://127.0.0.1:8081 -TestEmail test@example.com
#>

param(
    [string]$BaseUrl = "http://127.0.0.1:8081",
    [string]$TestEmail = "test_$(Get-Random)@example.com"
)

$ErrorActionPreference = "Stop"
$PassCount = 0
$FailCount = 0

function Write-Result($name, $success, $detail = "") {
    $icon = if ($success) { "[PASS]" } else { "[FAIL]" }
    $color = if ($success) { "Green" } else { "Red" }
    Write-Host "$icon $name" -ForegroundColor $color
    if ($detail) { Write-Host "     $detail" -ForegroundColor Gray }
    if ($success) { $script:PassCount++ } else { $script:FailCount++ }
}

function Invoke-Api($action, $body) {
    $url = "$BaseUrl/api?action=$action"
    $json = $body | ConvertTo-Json -Compress
    $resp = Invoke-RestMethod -Uri $url -Method Post -ContentType "application/json" -Body $json
    return $resp
}

Write-Host "`n===== 邮箱模块 API 集成测试 =====" -ForegroundColor Cyan
Write-Host "目标: $BaseUrl"
Write-Host "测试邮箱: $TestEmail`n"

# ---- 1. 发送验证码 ----
Write-Host "`n--- 1. 发送验证码 ---" -ForegroundColor Yellow
try {
    $resp = Invoke-Api "email_send_code" @{ email = $TestEmail }
    $ok = $resp.code -eq 200
    Write-Result "发送验证码" $ok $resp.msg
    if (-not $ok) {
        Write-Host "`n发送验证码失败，后续测试无法继续。请确认：" -ForegroundColor Red
        Write-Host "  1. 后端服务已启动"
        Write-Host "  2. MySQL 数据库已连接"
        Write-Host "  3. 邮箱 API 配置正确"
        exit 1
    }
} catch {
    Write-Result "发送验证码" $false $_.Exception.Message
    exit 1
}

# ---- 2. 非法邮箱格式 ----
Write-Host "`n--- 2. 参数校验 ---" -ForegroundColor Yellow
$resp = Invoke-Api "email_send_code" @{ email = "notanemail" }
Write-Result "非法邮箱拒绝" ($resp.code -eq 400) $resp.msg

# ---- 3. 注册（无验证码 - 应失败） ----
Write-Host "`n--- 3. 注册流程 ---" -ForegroundColor Yellow
$resp = Invoke-Api "email_register" @{
    email = $TestEmail
    code = "000000"
    password = "Test123456"
    password2 = "Test123456"
    nickname = "TestUser"
}
Write-Result "错误验证码注册拒绝" ($resp.code -eq 400) $resp.msg

# ---- 4. 注册（密码不一致） ----
$resp = Invoke-Api "email_register" @{
    email = $TestEmail
    code = "000000"
    password = "Test123456"
    password2 = "Different"
    nickname = ""
}
Write-Result "密码不一致注册拒绝" ($resp.code -eq 400) $resp.msg

# ---- 5. 登录（用户不存在） ----
Write-Host "`n--- 4. 登录流程 ---" -ForegroundColor Yellow
$resp = Invoke-Api "email_login" @{
    email = "nonexistent_$(Get-Random)@nowhere.com"
    password = "SomePassword123"
}
Write-Result "未注册用户登录拒绝" ($resp.code -eq 400) $resp.msg

# ---- 6. 获取用户信息（无 token） ----
Write-Host "`n--- 5. 用户信息 ---" -ForegroundColor Yellow
$resp = Invoke-Api "email_get_profile" @{ token = "" }
Write-Result "空 token 获取信息拒绝" ($resp.code -eq 401) $resp.msg

$resp = Invoke-Api "email_get_profile" @{ token = "invalid.jwt.token" }
Write-Result "无效 token 获取信息拒绝" ($resp.code -eq 401) $resp.msg

# ---- 7. 重置密码（用户不存在） ----
Write-Host "`n--- 6. 重置密码 ---" -ForegroundColor Yellow
$resp = Invoke-Api "email_reset_password" @{
    email = "nonexistent_$(Get-Random)@nowhere.com"
    code = "000000"
    password = "NewPass123"
    password2 = "NewPass123"
}
Write-Result "未注册邮箱重置密码拒绝" ($resp.code -eq 400) $resp.msg

# ---- 汇总 ----
Write-Host "`n===== 测试汇总 =====" -ForegroundColor Cyan
Write-Host "通过: $PassCount" -ForegroundColor Green
Write-Host "失败: $FailCount" -ForegroundColor $(if ($FailCount -gt 0) { "Red" } else { "Gray" })
Write-Host ""

if ($FailCount -gt 0) {
    Write-Host "存在失败用例，请检查后端逻辑。" -ForegroundColor Red
    exit 1
} else {
    Write-Host "全部测试通过！" -ForegroundColor Green
    exit 0
}
