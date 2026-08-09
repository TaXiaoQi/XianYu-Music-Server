use axum::response::Response;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::admin;
use crate::config::Config;
use crate::handlers::helpers::{parse_body, random_hex, random_int, str_of};
use crate::response::ReqCtx;

const DEBUG_DIR: &str = "data/debug";

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

fn today_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn debug_file() -> PathBuf {
    PathBuf::from(DEBUG_DIR).join("state.json")
}

fn default_user() -> Value {
    json!({
        "id": 1,
        "username": "debug-user",
        "password": "123456",
        "email": "debug@example.local",
        "nickname": "本地调试用户",
        "avatar_url": "",
        "ciyuanxi_id": "1000",
        "status": 1,
        "master_quota": 0,
        "role": "member",
        "created_at": now_string()
    })
}

fn default_settings() -> Value {
    json!({
        "stream_cache_enabled": 1,
        "startup_play_enabled": 0,
        "bluetooth_lyric_enabled": 0,
        "download_lyric_enabled": 1,
        "download_cover_enabled": 1,
        "download_artist_enabled": 0,
        "search_board_enabled": 1,
        "page_animation_enabled": 1,
        "default_quality": "standard"
    })
}

fn default_state() -> Value {
    json!({
        "users": [default_user()],
        "captchas": [],
        "email_codes": [],
        "tv_codes": [],
        "settings": {},
        "plugins": {},
        "settings_sync": {},
        "file_sync": {},
        "feedback": [],
        "listen_stats": [],
        "quota_usage": {}
    })
}

fn load_state() -> Value {
    let path = debug_file();
    let mut state = std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<Value>(&c).ok())
        .unwrap_or_else(default_state);
    if state.get("users").and_then(|v| v.as_array()).map(|a| a.is_empty()).unwrap_or(true) {
        state["users"] = json!([default_user()]);
    }
    state
}

fn save_state(state: &Value) -> bool {
    let path = debug_file();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, serde_json::to_string_pretty(state).unwrap_or_default()).is_ok()
}

fn get_array(state: &Value, key: &str) -> Vec<Value> {
    state.get(key).and_then(|v| v.as_array()).cloned().unwrap_or_default()
}

fn set_array(state: &mut Value, key: &str, items: Vec<Value>) {
    state[key] = json!(items);
}

fn extract_id(data: &Value) -> String {
    for key in ["ciyuanxi_id", "user_id", "id", "uid"] {
        let v = str_of(data, key);
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
        if let Some(n) = data.get(key).and_then(|x| x.as_i64()) {
            return n.to_string();
        }
    }
    String::new()
}

fn user_matches(user: &Value, ident: &str) -> bool {
    let ident = ident.trim();
    if ident.is_empty() {
        return false;
    }
    user.get("username").and_then(|v| v.as_str()).unwrap_or("") == ident
        || user.get("email").and_then(|v| v.as_str()).unwrap_or("") == ident
        || user.get("ciyuanxi_id").and_then(|v| v.as_str()).unwrap_or("") == ident
        || user.get("id").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default() == ident
}

fn user_payload(user: &Value, token: &str) -> Value {
    json!({
        "user_id": user.get("id").and_then(|v| v.as_i64()).unwrap_or(1),
        "username": user.get("username").and_then(|v| v.as_str()).unwrap_or("debug-user"),
        "email": user.get("email").and_then(|v| v.as_str()).unwrap_or("debug@example.local"),
        "token": token,
        "role": user.get("role").and_then(|v| v.as_str()).unwrap_or("member"),
        "avatar_url": user.get("avatar_url").and_then(|v| v.as_str()).unwrap_or(""),
        "ciyuanxi_id": user.get("ciyuanxi_id").and_then(|v| v.as_str()).unwrap_or("1000"),
        "master_quota": user.get("master_quota").and_then(|v| v.as_i64()).unwrap_or(0),
        "status": if user.get("status").and_then(|v| v.as_i64()).unwrap_or(1) == 1 { "enabled" } else { "disabled" },
        "debug": true
    })
}

fn put_map_value(state: &mut Value, map_key: &str, item_key: &str, value: Value) {
    if !state.get(map_key).map(|v| v.is_object()).unwrap_or(false) {
        state[map_key] = json!({});
    }
    if let Some(obj) = state.get_mut(map_key).and_then(|v| v.as_object_mut()) {
        obj.insert(item_key.to_string(), value);
    }
}

fn get_map_value(state: &Value, map_key: &str, item_key: &str) -> Option<Value> {
    state
        .get(map_key)
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(item_key))
        .cloned()
}

fn about_config() -> Value {
    json!({
        "officialSiteUrl": "https://xy.zh2026.cn/ciyuanxi/",
        "officialSiteText": "前往官网",
        "updateEnabled": true,
        "updateText": "检查更新",
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "projectText": "开源地址",
        "referenceProjectUrl": "https://github.com/Billy636/XianYuMusic",
        "referenceProjectText": "参考项目"
    })
}

fn require_captcha(data: &Value, ctx: &ReqCtx, state: &mut Value, purpose: &str) -> Option<Response> {
    let captcha_id = str_of(data, "captcha_id").trim().to_string();
    let captcha_answer = str_of(data, "captcha_answer").trim().to_string();
    if captcha_id.is_empty() || captcha_answer.is_empty() {
        return Some(ctx.err(400, "请完成人机验证"));
    }

    let mut captchas = get_array(state, "captchas");
    let mut matched = false;
    for item in captchas.iter_mut() {
        let same_id = item.get("captcha_id").and_then(|v| v.as_str()).unwrap_or("") == captcha_id;
        let same_purpose = item.get("purpose").and_then(|v| v.as_str()).unwrap_or("auth") == purpose;
        let same_ip = item.get("ip").and_then(|v| v.as_str()).unwrap_or("") == ctx.client_ip;
        let used = item.get("used").and_then(|v| v.as_bool()).unwrap_or(false);
        let valid_time = item.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0) > now_ts();
        if same_id && same_purpose && same_ip && !used && valid_time {
            matched = true;
            if item.get("answer").and_then(|v| v.as_str()).unwrap_or("") != captcha_answer.trim() {
                return Some(ctx.err(400, "人机验证错误，请刷新后重试"));
            }
            item["used"] = json!(true);
            break;
        }
    }
    if !matched {
        return Some(ctx.err(400, "人机验证已过期，请刷新后重试"));
    }
    set_array(state, "captchas", captchas);
    None
}

fn consume_email_code(state: &mut Value, email: &str, code: &str, typ: &str) -> bool {
    let mut codes = get_array(state, "email_codes");
    let mut ok = false;
    for item in codes.iter_mut().rev() {
        let same_email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").eq_ignore_ascii_case(email);
        let same_code = item.get("code").and_then(|v| v.as_str()).unwrap_or("") == code;
        let same_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("") == typ;
        let used = item.get("used").and_then(|v| v.as_bool()).unwrap_or(false);
        let valid_time = item.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0) > now_ts();
        if same_email && same_code && same_type && !used && valid_time {
            item["used"] = json!(true);
            ok = true;
            break;
        }
    }
    set_array(state, "email_codes", codes);
    ok
}

pub fn handle_api(action: &str, body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    match action {
        "debug_ping" => ctx.json(200, "本地调试服务端连接正常", Some(json!({
            "mode": "local_debug_no_db",
            "time": now_string(),
        }))),
        "check" => ctx.json(200, "本地调试模式：未连接数据库", Some(json!({
            "connection": false,
            "debug": true,
            "tables": {}
        }))),
        "install" => ctx.json(200, "本地调试模式：跳过数据库安装", Some(json!([]))),
        "get_captcha" => {
            let mut state = load_state();
            let purpose = {
                let p = str_of(&data, "purpose");
                if p.trim().is_empty() { "auth".to_string() } else { p.trim().to_string() }
            };
            let left = random_int(2, 9);
            let right = random_int(1, 9);
            let captcha_id = random_hex(16);
            let mut captchas = get_array(&state, "captchas")
                .into_iter()
                .filter(|v| v.get("expires_at").and_then(|x| x.as_i64()).unwrap_or(0) > now_ts())
                .collect::<Vec<_>>();
            captchas.push(json!({
                "captcha_id": captcha_id,
                "purpose": purpose,
                "answer": (left + right).to_string(),
                "ip": ctx.client_ip,
                "used": false,
                "created_at": now_ts(),
                "expires_at": now_ts() + 300
            }));
            set_array(&mut state, "captchas", captchas);
            let _ = save_state(&state);
            ctx.ok("ok", json!({
                "captcha_id": captcha_id,
                "question": format!("{} + {} = ?", left, right),
                "expire_seconds": 300,
                "debug": true
            }))
        }
        "verify_captcha" => {
            let state = load_state();
            let purpose = {
                let p = str_of(&data, "purpose");
                if p.trim().is_empty() { "auth".to_string() } else { p.trim().to_string() }
            };
            let captcha_id = str_of(&data, "captcha_id");
            let answer = str_of(&data, "captcha_answer");
            let ok = get_array(&state, "captchas").iter().any(|item| {
                item.get("captcha_id").and_then(|v| v.as_str()).unwrap_or("") == captcha_id
                    && item.get("purpose").and_then(|v| v.as_str()).unwrap_or("auth") == purpose
                    && item.get("ip").and_then(|v| v.as_str()).unwrap_or("") == ctx.client_ip
                    && item.get("answer").and_then(|v| v.as_str()).unwrap_or("") == answer.trim()
                    && item.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0) > now_ts()
            });
            if ok {
                ctx.ok("验证通过", json!({ "verified": true, "debug": true }))
            } else {
                ctx.err(400, "人机验证错误，请重新输入")
            }
        }
        "get_source_status" => ctx.json(200, "ok", Some(json!({
            "source_name": "kg",
            "is_enabled": true,
            "debug": true,
            "sources": {
                "kg": {"source_name": "酷狗音乐", "is_enabled": true},
                "tx": {"source_name": "QQ音乐", "is_enabled": true},
                "kw": {"source_name": "酷我音乐", "is_enabled": true},
                "mg": {"source_name": "咪咕音乐", "is_enabled": true},
                "wy": {"source_name": "网易音乐", "is_enabled": true}
            }
        }))),
        "get_latest_version" => ctx.json(200, "ok", Some(json!({
            "id": 0,
            "app_name": "弦予音乐",
            "version": "0.0.0-debug",
            "content": "本地调试模式：未连接数据库，版本接口返回 mock 数据。",
            "download_url": "",
            "file_size": 0,
            "status": "normal"
        }))),
        "get_version_status" => ctx.json(200, "ok", Some(json!({ "status": "normal", "debug": true }))),
        "get_announcement" => ctx.json(200, "ok", Some(json!({
            "id": "debug",
            "title": "本地调试模式",
            "content": "服务端当前未连接数据库，但客户端已经可以连接到本地服务端。",
            "type": "info",
            "date": today_string(),
            "actionUrl": "",
            "actionText": "",
            "updatedAt": now_string()
        }))),
        "get_about_config" => ctx.json(200, "ok", Some(about_config())),
        "get_server_load" => ctx.json(200, "ok", Some(json!({
            "cpu": 0,
            "memory": 0,
            "user_count": get_array(&load_state(), "users").len(),
            "debug": true
        }))),
        "send_verify_code" | "email_send_code" => {
            let mut state = load_state();
            let email = str_of(&data, "email").trim().to_string();
            let typ = {
                let t = str_of(&data, "type");
                if t.trim().is_empty() { "register".to_string() } else { t.trim().to_string() }
            };
            if email.is_empty() || !email.contains('@') {
                return ctx.err(400, "邮箱格式不正确");
            }
            if action == "send_verify_code" {
                if let Some(resp) = require_captcha(&data, &ctx, &mut state, "auth") {
                    return resp;
                }
            }
            let code = format!("{:06}", random_int(100000, 999999));
            let mut codes = get_array(&state, "email_codes");
            codes.push(json!({
                "email": email,
                "code": code,
                "type": typ,
                "ip": ctx.client_ip,
                "used": false,
                "created_at": now_ts(),
                "expires_at": now_ts() + 600
            }));
            set_array(&mut state, "email_codes", codes);
            let _ = save_state(&state);
            ctx.ok("验证码已发送，请查收邮件", json!({ "debug_code": code, "expire_seconds": 600, "debug": true }))
        }
        "register" | "email_register" => {
            let mut state = load_state();
            let username = {
                let u = str_of(&data, "username");
                if u.trim().is_empty() { str_of(&data, "email").split('@').next().unwrap_or("debug-user").to_string() } else { u.trim().to_string() }
            };
            let password = str_of(&data, "password");
            let email = str_of(&data, "email").trim().to_string();
            let verify_code = str_of(&data, "verify_code");
            if username.chars().count() < 2 || username.chars().count() > 32 {
                return ctx.err(400, "用户名长度需2-32个字符");
            }
            if password.len() < 6 {
                return ctx.err(400, "密码长度至少6位");
            }
            if email.is_empty() || !email.contains('@') {
                return ctx.err(400, "邮箱格式不正确");
            }
            if action == "register" {
                if let Some(resp) = require_captcha(&data, &ctx, &mut state, "auth") {
                    return resp;
                }
            }
            if !consume_email_code(&mut state, &email, &verify_code, "register") {
                return ctx.err(400, "验证码无效或已过期");
            }
            let mut users = get_array(&state, "users");
            if users.iter().any(|u| u.get("username").and_then(|v| v.as_str()).unwrap_or("") == username) {
                return ctx.err(400, "用户名已存在");
            }
            if users.iter().any(|u| u.get("email").and_then(|v| v.as_str()).unwrap_or("").eq_ignore_ascii_case(&email)) {
                return ctx.err(400, "该邮箱已注册");
            }
            let id = users.iter().filter_map(|u| u.get("id").and_then(|v| v.as_i64())).max().unwrap_or(0) + 1;
            let ciyuanxi_id = (1000 + id).to_string();
            let token = random_hex(32);
            let user = json!({
                "id": id,
                "username": username,
                "password": password,
                "email": email,
                "nickname": "",
                "avatar_url": "",
                "ciyuanxi_id": ciyuanxi_id,
                "status": 1,
                "master_quota": 0,
                "role": "member",
                "created_at": now_string()
            });
            users.push(user.clone());
            set_array(&mut state, "users", users);
            let _ = save_state(&state);
            ctx.json(200, "注册成功", Some(user_payload(&user, &token)))
        }
        "user_login" | "email_login" => {
            let mut state = load_state();
            let ident = {
                let u = str_of(&data, "username");
                if u.trim().is_empty() { str_of(&data, "email") } else { u }
            };
            let password = str_of(&data, "password");
            if ident.trim().is_empty() || password.is_empty() {
                return ctx.err(400, "用户名和密码不能为空");
            }
            if action == "user_login" {
                if let Some(resp) = require_captcha(&data, &ctx, &mut state, "auth") {
                    return resp;
                }
            }
            let users = get_array(&state, "users");
            let Some(user) = users.iter().find(|u| user_matches(u, &ident)).cloned() else {
                return ctx.err(401, "用户名或密码错误");
            };
            if user.get("password").and_then(|v| v.as_str()).unwrap_or("") != password {
                return ctx.err(401, "用户名或密码错误");
            }
            if user.get("status").and_then(|v| v.as_i64()).unwrap_or(1) == 0 {
                return ctx.err(403, "账号已被禁用，请联系管理员");
            }
            ctx.json(200, "登录成功", Some(user_payload(&user, &random_hex(32))))
        }
        "login_by_code" => {
            let mut state = load_state();
            let email = str_of(&data, "email");
            let verify_code = str_of(&data, "verify_code");
            if let Some(resp) = require_captcha(&data, &ctx, &mut state, "auth") {
                return resp;
            }
            if !consume_email_code(&mut state, &email, &verify_code, "login") {
                return ctx.err(400, "验证码无效或已过期");
            }
            let users = get_array(&state, "users");
            let Some(user) = users.iter().find(|u| u.get("email").and_then(|v| v.as_str()).unwrap_or("").eq_ignore_ascii_case(&email)).cloned() else {
                return ctx.err(401, "该邮箱未注册");
            };
            let _ = save_state(&state);
            ctx.json(200, "登录成功", Some(user_payload(&user, &random_hex(32))))
        }
        "reset_password" | "email_reset_password" => {
            let mut state = load_state();
            let email = str_of(&data, "email");
            let verify_code = str_of(&data, "verify_code");
            let new_password = {
                let p = str_of(&data, "new_password");
                if p.is_empty() { str_of(&data, "password") } else { p }
            };
            if action == "reset_password" {
                if let Some(resp) = require_captcha(&data, &ctx, &mut state, "auth") {
                    return resp;
                }
            }
            if new_password.len() < 6 {
                return ctx.err(400, "新密码长度至少6位");
            }
            if !consume_email_code(&mut state, &email, &verify_code, "reset_password") {
                return ctx.err(400, "验证码无效或已过期");
            }
            let mut users = get_array(&state, "users");
            let mut found = false;
            for user in users.iter_mut() {
                if user.get("email").and_then(|v| v.as_str()).unwrap_or("").eq_ignore_ascii_case(&email) {
                    user["password"] = json!(new_password);
                    found = true;
                    break;
                }
            }
            if !found {
                return ctx.err(400, "该邮箱未注册");
            }
            set_array(&mut state, "users", users);
            let _ = save_state(&state);
            ctx.ok("密码已重置", json!({ "debug": true }))
        }
        "delete_account" => {
            let mut state = load_state();
            let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
            let email = str_of(&data, "email");
            let verify_code = str_of(&data, "verify_code");
            if !consume_email_code(&mut state, &email, &verify_code, "delete_account") {
                return ctx.err(400, "验证码无效或已过期");
            }
            let users = get_array(&state, "users")
                .into_iter()
                .filter(|u| u.get("ciyuanxi_id").and_then(|v| v.as_str()).unwrap_or("") != ciyuanxi_id)
                .collect::<Vec<_>>();
            set_array(&mut state, "users", users);
            let _ = save_state(&state);
            ctx.ok("账号已注销", json!({ "debug": true }))
        }
        "get_user_info" | "email_get_profile" => {
            let state = load_state();
            let ident = {
                let i = extract_id(&data);
                if i.is_empty() { str_of(&data, "email") } else { i }
            };
            let users = get_array(&state, "users");
            let user = users.iter().find(|u| user_matches(u, &ident)).cloned().unwrap_or_else(default_user);
            ctx.json(200, "ok", Some(user_payload(&user, &random_hex(32))))
        }
        "check_username" => {
            let state = load_state();
            let username = str_of(&data, "username");
            let exists = get_array(&state, "users").iter().any(|u| u.get("username").and_then(|v| v.as_str()).unwrap_or("") == username);
            ctx.ok("ok", json!({ "available": !exists, "exists": exists, "debug": true }))
        }
        "change_password" => {
            let mut state = load_state();
            let ident = extract_id(&data);
            let old_password = str_of(&data, "old_password");
            let new_password = str_of(&data, "new_password");
            let mut users = get_array(&state, "users");
            let mut ok = false;
            for user in users.iter_mut() {
                if user_matches(user, &ident) && user.get("password").and_then(|v| v.as_str()).unwrap_or("") == old_password {
                    user["password"] = json!(new_password);
                    ok = true;
                    break;
                }
            }
            if !ok {
                return ctx.err(400, "原密码错误");
            }
            set_array(&mut state, "users", users);
            let _ = save_state(&state);
            ctx.ok("密码已修改", json!({ "debug": true }))
        }
        "update_profile" => {
            let mut state = load_state();
            let ident = extract_id(&data);
            let mut users = get_array(&state, "users");
            let mut payload = Value::Null;
            for user in users.iter_mut() {
                if user_matches(user, &ident) {
                    for key in ["username", "nickname", "avatar_url", "email"] {
                        if let Some(v) = data.get(key).cloned() {
                            user[key] = v;
                        }
                    }
                    payload = user_payload(user, &random_hex(32));
                    break;
                }
            }
            if payload.is_null() {
                return ctx.err(404, "用户不存在");
            }
            set_array(&mut state, "users", users);
            let _ = save_state(&state);
            ctx.ok("资料已更新", payload)
        }
        "get_user_settings" => {
            let state = load_state();
            let ciyuanxi_id = extract_id(&data);
            let settings = get_map_value(&state, "settings", &ciyuanxi_id).unwrap_or_else(default_settings);
            ctx.ok("ok", settings)
        }
        "update_user_settings" => {
            let mut state = load_state();
            let ciyuanxi_id = extract_id(&data);
            if ciyuanxi_id.is_empty() {
                return ctx.err(400, "弦予号不能为空");
            }
            let mut settings = get_map_value(&state, "settings", &ciyuanxi_id).unwrap_or_else(default_settings);
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    if k != "ciyuanxi_id" && k != "user_id" && k != "id" {
                        settings[k] = v.clone();
                    }
                }
            }
            put_map_value(&mut state, "settings", &ciyuanxi_id, settings.clone());
            let _ = save_state(&state);
            ctx.ok("设置已保存", settings)
        }
        "get_avatar_status" | "get_nickname_status" => ctx.ok("ok", json!({ "status": "approved", "debug": true })),
        "error" | "report_user_behavior" | "open" | "search" | "input_stats" | "report_listen_stats" => {
            let mut state = load_state();
            let mut rows = get_array(&state, "listen_stats");
            rows.push(json!({ "action": action, "data": data, "created_at": now_string() }));
            set_array(&mut state, "listen_stats", rows);
            let _ = save_state(&state);
            ctx.json(200, "本地调试模式：上报已保存到临时存储", Some(json!({ "id": now_ts(), "debug": true })))
        }
        "deduct_master_quota" => ctx.ok("扣减成功", json!({ "remaining": 999, "debug": true })),
        "get_master_quota_usage" => ctx.ok("ok", json!({ "used": 0, "limit": 999, "remaining": 999, "debug": true })),
        "submit_feedback" => {
            let mut state = load_state();
            let mut rows = get_array(&state, "feedback");
            rows.push(json!({ "id": rows.len() + 1, "data": data, "created_at": now_string(), "status": "pending" }));
            set_array(&mut state, "feedback", rows);
            let _ = save_state(&state);
            ctx.ok("反馈已提交", json!({ "debug": true }))
        }
        "check_ciyuanxi_id" => {
            let state = load_state();
            let id = extract_id(&data);
            let exists = get_array(&state, "users").iter().any(|u| user_matches(u, &id));
            ctx.ok("ok", json!({ "exists": exists, "available": !exists, "debug": true }))
        }
        "list_wallpapers" => ctx.ok("ok", json!({
            "total": 2,
            "list": [
                { "id": 1, "title": "本地调试壁纸 1", "url": "", "status": "approved" },
                { "id": 2, "title": "本地调试壁纸 2", "url": "", "status": "approved" }
            ],
            "debug": true
        })),
        "my_wallpapers" => ctx.ok("ok", json!({ "total": 0, "list": [], "debug": true })),
        "upload_wallpaper" => ctx.ok("上传成功", json!({ "id": now_ts(), "status": "pending", "debug": true })),
        "delete_playlist" => ctx.ok("删除成功", json!({ "debug": true })),
        "file_sync_upload_start" => ctx.ok("ok", json!({ "chunk_dir_ready": true, "debug": true })),
        "file_sync_upload_chunk" => ctx.ok("ok", json!({
            "chunk_index": data.get("chunk_index").cloned().unwrap_or(json!(0)),
            "total_chunks": data.get("total_chunks").cloned().unwrap_or(json!(1)),
            "debug": true
        })),
        "file_sync_upload_finish" => {
            let mut state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let playlists = data.get("playlists").cloned().unwrap_or_else(|| json!([]));
            let song_total = playlists.as_array().map(|arr| arr.iter().map(|pl| pl.get("songs").and_then(|s| s.as_array()).map(|s| s.len()).unwrap_or(0)).sum::<usize>()).unwrap_or(0);
            let save = json!({
                "version": 4,
                "uploaded_at": now_string(),
                "timestamp": now_ts(),
                "stats": { "playlist_count": playlists.as_array().map(|a| a.len()).unwrap_or(0), "song_total": song_total },
                "playlists": playlists
            });
            put_map_value(&mut state, "file_sync", &ciyuanxi_id, save);
            let _ = save_state(&state);
            ctx.ok("同步成功", json!({ "playlist_count": playlists.as_array().map(|a| a.len()).unwrap_or(0), "song_total": song_total, "debug": true }))
        }
        "file_sync_download" => {
            let state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let v = get_map_value(&state, "file_sync", &ciyuanxi_id).unwrap_or_else(|| json!({ "playlists": [] }));
            ctx.ok("获取成功", v)
        }
        "plugin_sync_upload_one" => {
            let mut state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let plugin = data.get("plugin").cloned().unwrap_or(Value::Null);
            let mut save = get_map_value(&state, "plugins", &ciyuanxi_id).unwrap_or_else(|| json!({ "version": 1, "plugins": [] }));
            let mut plugins = save.get("plugins").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let pid = plugin.get("id").cloned().unwrap_or(Value::Null);
            plugins.retain(|p| p.get("id").cloned().unwrap_or(Value::Null) != pid);
            plugins.push(plugin);
            save["plugins"] = json!(plugins);
            save["stats"] = json!({ "plugin_count": save["plugins"].as_array().map(|a| a.len()).unwrap_or(0) });
            save["uploaded_at"] = json!(now_string());
            save["timestamp"] = json!(now_ts());
            put_map_value(&mut state, "plugins", &ciyuanxi_id, save.clone());
            let _ = save_state(&state);
            ctx.ok("上传成功", json!({ "plugin_count": save["plugins"].as_array().map(|a| a.len()).unwrap_or(0), "debug": true }))
        }
        "plugin_sync_download" => {
            let state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let v = get_map_value(&state, "plugins", &ciyuanxi_id).unwrap_or_else(|| json!({ "plugins": [] }));
            ctx.ok("获取成功", v)
        }
        "settings_sync_upload" => {
            let mut state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let save = json!({ "version": 1, "uploaded_at": now_string(), "timestamp": now_ts(), "settings": data.get("settings").cloned().unwrap_or(Value::Null) });
            put_map_value(&mut state, "settings_sync", &ciyuanxi_id, save.clone());
            let _ = save_state(&state);
            ctx.ok("上传成功", json!({ "uploaded_at": save["uploaded_at"], "debug": true }))
        }
        "settings_sync_download" => {
            let state = load_state();
            let ciyuanxi_id = str_of(&data, "user_id");
            let v = get_map_value(&state, "settings_sync", &ciyuanxi_id).unwrap_or_else(|| json!({ "settings": null }));
            ctx.ok("获取成功", v)
        }
        "generate_tv_login_code" => {
            let mut state = load_state();
            let device_id = str_of(&data, "device_id");
            if device_id.is_empty() {
                return ctx.err(400, "设备标识不能为空");
            }
            let code = random_hex(16);
            let mut codes = get_array(&state, "tv_codes");
            codes.push(json!({ "code": code, "device_id": device_id, "status": "pending", "expires_at": now_ts() + 300 }));
            set_array(&mut state, "tv_codes", codes);
            let _ = save_state(&state);
            ctx.ok("ok", json!({ "code": code, "expire_seconds": 300, "debug": true }))
        }
        "poll_tv_login_status" => {
            let state = load_state();
            let code = str_of(&data, "code");
            let item = get_array(&state, "tv_codes").into_iter().find(|v| v.get("code").and_then(|x| x.as_str()).unwrap_or("") == code);
            ctx.ok("ok", item.unwrap_or_else(|| json!({ "status": "pending", "debug": true })))
        }
        "scan_tv_login" => {
            let mut state = load_state();
            let code = str_of(&data, "code");
            let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
            let mut codes = get_array(&state, "tv_codes");
            for item in codes.iter_mut() {
                if item.get("code").and_then(|v| v.as_str()).unwrap_or("") == code {
                    item["status"] = json!("scanned");
                    item["ciyuanxi_id"] = json!(ciyuanxi_id);
                }
            }
            set_array(&mut state, "tv_codes", codes);
            let _ = save_state(&state);
            ctx.ok("扫码成功，请在手机端确认登录", json!({ "ciyuanxi_id": ciyuanxi_id, "debug": true }))
        }
        "confirm_tv_login" => {
            let mut state = load_state();
            let code = str_of(&data, "code");
            let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
            let mut codes = get_array(&state, "tv_codes");
            for item in codes.iter_mut() {
                if item.get("code").and_then(|v| v.as_str()).unwrap_or("") == code {
                    item["status"] = json!("logged_in");
                    item["ciyuanxi_id"] = json!(ciyuanxi_id);
                    item["token"] = json!(random_hex(32));
                }
            }
            set_array(&mut state, "tv_codes", codes);
            let _ = save_state(&state);
            ctx.ok("登录成功", json!({ "ciyuanxi_id": ciyuanxi_id, "debug": true }))
        }
        _ => ctx.json(200, "本地调试模式：接口已连通", Some(json!({
            "action": action,
            "debug": true
        }))),
    }
}

pub fn handle_admin_login(cfg: &Config) -> Response {
    let token = admin::sign_token(cfg, 1, "debug-admin", "super_admin");
    admin::ok("本地调试登录成功", json!({
        "token": token,
        "admin_id": 1,
        "username": "debug-admin",
        "role": "super_admin",
        "expires_in": 86400,
        "debug": true
    }))
}

pub fn handle_admin_api(action: &str) -> Response {
    match action {
        "dashboard_stats" => admin::ok("ok", json!({
            "total_users": 128,
            "today_users": 12,
            "yesterday_users": 9,
            "total_admins": 1,
            "total_source_calls": 4096,
            "today_source_calls": 386,
            "yesterday_source_calls": 342,
            "today_source_success": 360,
            "total_source_success": 3900,
            "total_errors": 3,
            "today_errors": 0,
            "yesterday_errors": 1,
            "total_shares": 26,
            "today_shares": 2,
            "yesterday_shares": 4,
            "total_logins": 88,
            "today_logins": 15,
            "debug": true
        })),
        "get_about_config_admin" => admin::ok("ok", about_config()),
        "save_about_config" => admin::ok("本地调试模式：配置已模拟保存", about_config()),
        "list_versions" => admin::ok("ok", json!({
            "total": 1,
            "total_pages": 1,
            "list": [{
                "id": 1,
                "app_name": "弦予音乐",
                "version_code": "0.0.0-debug",
                "download_url": "",
                "update_content": "本地调试版本数据",
                "status": "normal",
                "file_size": 0,
                "created_at": now_string()
            }]
        })),
        "list_announcements" => admin::ok("ok", json!([{
            "id": "debug",
            "title": "本地调试公告",
            "content": "后台当前运行在无数据库调试模式。",
            "type": "info",
            "date": today_string(),
            "actionUrl": "",
            "actionText": "",
            "enabled": true,
            "created_at": now_string(),
            "updated_at": now_string()
        }])),
        "list_wallpapers" => admin::ok("ok", json!([])),
        "get_source_config" => admin::ok("ok", json!({
            "kg": {"source_name": "酷狗音乐", "source_code": "kg", "is_enabled": true},
            "tx": {"source_name": "QQ音乐", "source_code": "tx", "is_enabled": true},
            "kw": {"source_name": "酷我音乐", "source_code": "kw", "is_enabled": true},
            "mg": {"source_name": "咪咕音乐", "source_code": "mg", "is_enabled": true},
            "wy": {"source_name": "网易音乐", "source_code": "wy", "is_enabled": true}
        })),
        "get_users" => admin::ok("ok", json!({
            "total": 1,
            "total_pages": 1,
            "list": [{
                "id": 1,
                "username": "debug-user",
                "nickname": "本地调试用户",
                "email": "debug@example.local",
                "ciyuanxi_id": "1000",
                "status": 1,
                "created_at": now_string()
            }]
        })),
        "list_admins" => admin::ok("ok", json!([{
            "id": 1,
            "username": "debug-admin",
            "role": "super_admin",
            "status": 1,
            "created_at": now_string()
        }])),
        "list_error_logs" | "list_app_login_log" | "list_operation_logs" | "list_admin_login_logs" | "list_feedback" => {
            admin::ok("ok", json!({ "total": 0, "total_pages": 0, "list": [] }))
        }
        "get_audit_external_config" => admin::ok("ok", json!({
            "enabled": false,
            "provider": "generic",
            "endpoint": "",
            "api_key": "",
            "nickname_enabled": true,
            "avatar_enabled": true,
            "wallpaper_enabled": true,
            "timeout_ms": 5000,
            "fail_to_manual": true
        })),
        "save_audit_external_config" => admin::ok("本地调试模式：审核配置已模拟保存", json!({ "debug": true })),
        "test_audit_external_config" => admin::ok("测试完成", json!({
            "decision": "manual",
            "reason": "本地调试模式未调用外部审核服务",
            "provider": "debug"
        })),
        "admin_logout" => admin::ok("已退出", Value::Null),
        _ => admin::ok("本地调试模式：操作已模拟完成", json!({
            "action": action,
            "debug": true
        })),
    }
}
