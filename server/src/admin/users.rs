use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{default_nickname, int_of, parse_body, str_of, validate_ciyuanxi_id, validate_nickname};

/// 获取用户列表（分页 + 关键词搜索）
pub async fn get_users(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let keyword = str_of(&data, "keyword").trim().to_string();
    let offset = (page - 1) * page_size;

    // 查询总数
    let total: i64 = if keyword.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM app_users")
            .fetch_one(pool)
            .await
            .unwrap_or(0)
    } else {
        let pat = format!("%{}%", keyword);
        sqlx::query_scalar("SELECT COUNT(*) FROM app_users WHERE nickname LIKE ? OR email LIKE ?")
            .bind(&pat)
            .bind(&pat)
            .fetch_one(pool)
            .await
            .unwrap_or(0)
    };

    // 查询列表
    let rows = if keyword.is_empty() {
        sqlx::query("SELECT * FROM app_users ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await
    } else {
        let pat = format!("%{}%", keyword);
        sqlx::query("SELECT * FROM app_users WHERE nickname LIKE ? OR email LIKE ? ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(&pat)
            .bind(&pat)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await
    };

    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
                "list": list,
            }))
        }
        Err(e) => err(500, &format!("查询失败: {}", e)),
    }
}

pub async fn toggle_user_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = int_of(&data, "status");
    let reason = str_of(&data, "reason").trim().to_string();
    if id <= 0 {
        return err(400, "参数错误");
    }
    if status == 0 && reason.is_empty() {
        return err(400, "封禁原因不能为空");
    }
    let ban_reason = if status == 0 { reason.as_str() } else { "" };
    let _ = sqlx::query("UPDATE app_users SET status = ?, ban_reason = ? WHERE id = ?")
        .bind(status)
        .bind(ban_reason)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "更新用户状态", &format!("用户ID:{}", id), &format!("状态改为:{} 原因:{}", if status != 0 { "正常" } else { "禁用" }, ban_reason)).await;
    ok("操作成功", Value::Null)
}

pub async fn batch_toggle_user_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status = int_of(&data, "status");
    let reason = str_of(&data, "reason").trim().to_string();
    if status == 0 && reason.is_empty() {
        return err(400, "封禁原因不能为空");
    }
    let ban_reason = if status == 0 { reason.as_str() } else { "" };
    let r = sqlx::query("UPDATE app_users SET status = ?, ban_reason = ?")
        .bind(status)
        .bind(ban_reason)
        .execute(pool)
        .await;
    match r {
        Ok(res) => {
            let count = res.rows_affected();
            log_operation(pool, ctx, "批量更新用户状态", "全部用户", &format!("状态改为:{} 原因:{} 影响:{}人", if status != 0 { "正常" } else { "禁用" }, ban_reason, count)).await;
            ok(&format!("成功更新{}个用户状态", count), Value::Null)
        }
        Err(e) => err(500, &format!("操作失败: {}", e)),
    }
}

pub async fn delete_user(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let user = sqlx::query("SELECT nickname FROM app_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let nickname: String = user.get("nickname");
    let _ = sqlx::query("DELETE FROM app_users WHERE id = ?").bind(id).execute(pool).await;
    log_operation(pool, ctx, "删除用户", &format!("用户ID:{}", id), &format!("昵称:{}", nickname)).await;
    ok("删除成功", Value::Null)
}

pub async fn delete_user_avatar(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "user_id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let user = sqlx::query("SELECT nickname FROM app_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let nickname: String = user.get("nickname");
    let _ = sqlx::query("UPDATE app_users SET avatar_url = '' WHERE id = ?").bind(id).execute(pool).await;
    let _ = sqlx::query("UPDATE user_feedback SET nickname = (SELECT nickname FROM app_users WHERE id = ?) WHERE ciyuanxi_id = (SELECT ciyuanxi_id FROM app_users WHERE id = ?)")
        .bind(id)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "删除用户头像", &format!("用户ID:{}", id), &format!("昵称:{}", nickname)).await;
    ok("头像已删除", Value::Null)
}

pub async fn add_user(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut username = str_of(&data, "nickname").trim().to_string();
    if username.is_empty() {
        username = str_of(&data, "username").trim().to_string();
    }
    let password = str_of(&data, "password").to_string();
    let email = str_of(&data, "email").trim().to_string();
    let master_quota = int_of(&data, "master_quota");
    let master_quota = if master_quota == 0 { 200 } else { master_quota };
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    // 兼容前端将弦予号放在 username 字段的提交方式
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "username").trim().to_string();
    }
    // 弦予号必填 + 微信号规则校验
    if let Err(msg) = validate_ciyuanxi_id(&ciyuanxi_id) {
        return err(400, msg);
    }
    // 昵称可选，留空默认"弦予+号"
    if username.is_empty() {
        username = default_nickname(&ciyuanxi_id);
    }
    if username.len() < 2 || username.len() > 32 {
        return err(400, "昵称需 2-32 个字符");
    }
    if let Err(msg) = validate_nickname(&username, 2, 32) {
        return err(400, msg);
    }
    if password.len() < 6 {
        return err(400, "密码至少 6 位");
    }
    if !email.is_empty() && !crate::admin::is_valid_email(&email) {
        return err(400, "邮箱格式不正确");
    }
    // 弦予号唯一性校验
    let id_dup = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    let id_pretty = sqlx::query("SELECT id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if id_dup || id_pretty {
        return err(409, "该弦予号已被占用");
    }
    // 用户名唯一性：与客户端注册保持一致，同时检查管理员表和普通用户表
    let admin_exists = sqlx::query("SELECT id FROM admin_users WHERE username = ? LIMIT 1")
        .bind(&username)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    let exists = sqlx::query("SELECT id FROM app_users WHERE nickname = ? LIMIT 1")
        .bind(&username)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if admin_exists || exists {
        return err(409, "昵称已存在");
    }
    if !email.is_empty() {
        let exists = sqlx::query("SELECT id FROM app_users WHERE email = ? LIMIT 1")
            .bind(&email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some();
        if exists {
            return err(409, "邮箱已被使用");
        }
    }
    let hashed = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return err(500, "加密失败"),
    };
    let _ = sqlx::query("INSERT INTO app_users (nickname, password, email, email_verified, status, ciyuanxi_id, master_quota) VALUES (?,?,?,1,1,?,?)")
        .bind(&username)
        .bind(hashed)
        .bind(&email)
        .bind(&ciyuanxi_id)
        .bind(master_quota)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "添加用户", &format!("昵称:{}", username), &format!("弦予号:{} 额度:{}", ciyuanxi_id, master_quota)).await;
    ok("添加成功", json!({ "ciyuanxi_id": ciyuanxi_id }))
}

pub async fn set_user_master_quota(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let quota = int_of(&data, "quota");
    if id <= 0 {
        return err(400, "参数错误");
    }
    if quota < 0 {
        return err(400, "额度不能为负数");
    }
    let user = sqlx::query("SELECT nickname, ciyuanxi_id FROM app_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let nickname: String = user.get("nickname");
    let _ = sqlx::query("UPDATE app_users SET master_quota = ? WHERE id = ?")
        .bind(quota)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "设置母带额度", &format!("用户ID:{} 昵称:{}", id, nickname), &format!("额度设为:{}", quota)).await;
    ok("设置成功", Value::Null)
}

pub async fn batch_set_master_quota(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let quota = int_of(&data, "quota");
    if quota < 0 {
        return err(400, "请输入有效的额度值");
    }
    let _ = sqlx::query("UPDATE app_users SET master_quota = ?").bind(quota).execute(pool).await;
    log_operation(pool, ctx, "批量设置母带额度", "全部用户", &format!("额度设为:{}", quota)).await;
    ok("已设置", Value::Null)
}

/// 查看用户同步的插件列表（读 data/sync/{id}/plugins.json）
pub async fn get_user_plugins(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "user_id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let user = sqlx::query("SELECT nickname, ciyuanxi_id FROM app_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let username: String = user.get("nickname");
    let ciyuanxi_id: String = user.get("ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        return ok("ok", json!({ "nickname": username, "plugins": [], "uploaded_at": Value::Null }));
    }
    let clean_id: String = ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let dir = std::path::Path::new("data").join("sync").join(&clean_id);
    let file = dir.join("plugins.json");
    let content = match std::fs::read_to_string(&file) {
        Ok(c) => c,
        Err(_) => return ok("ok", json!({ "nickname": username, "ciyuanxi_id": ciyuanxi_id, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null })),
    };
    let save_data: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return err(500, "数据解析失败"),
    };
    let mut plugins: Vec<Value> = Vec::new();
    let uploaded_at = save_data.get("uploaded_at").cloned().unwrap_or(Value::Null);
    if let Some(list) = save_data.get("plugins").and_then(|x| x.as_array()) {
        for p in list {
            let script_size = p.get("script").and_then(|s| s.as_str()).map(|s| s.len()).unwrap_or(0);
            plugins.push(json!({
                "name": p.get("name").and_then(|x| x.as_str()).unwrap_or("(未知)"),
                "format": p.get("format").and_then(|x| x.as_str()).unwrap_or("unknown"),
                "version": p.get("version").and_then(|x| x.as_str()).unwrap_or(""),
                "author": p.get("author").and_then(|x| x.as_str()).unwrap_or(""),
                "description": p.get("description").and_then(|x| x.as_str()).unwrap_or(""),
                "enabled": p.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false),
                "filePath": p.get("filePath").and_then(|x| x.as_str()).unwrap_or(""),
                "importedAt": p.get("importedAt").and_then(|x| x.as_i64()).unwrap_or(0),
                "scriptSize": script_size,
            }));
        }
    }
    ok("ok", json!({
        "nickname": username,
        "ciyuanxi_id": ciyuanxi_id,
        "uploaded_at": uploaded_at,
        "plugin_count": plugins.len(),
        "plugins": plugins,
    }))
}

/// 后台修改指定账号昵称：填新昵称 + 原因，写库并下发客户端回执通知。
pub async fn change_user_nickname(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let new_nickname = str_of(&data, "new_nickname").trim().to_string();
    let reason = str_of(&data, "reason").trim().to_string();
    if id <= 0 {
        return err(400, "参数错误");
    }
    if new_nickname.len() < 2 || new_nickname.len() > 32 {
        return err(400, "昵称需 2-32 个字符");
    }
    if let Err(msg) = validate_nickname(&new_nickname, 2, 32) {
        return err(400, msg);
    }
    if reason.is_empty() {
        return err(400, "修改原因不能为空");
    }
    if reason.chars().count() > 255 {
        return err(400, "修改原因不能超过 255 字");
    }
    let user = sqlx::query("SELECT nickname, ciyuanxi_id FROM app_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let old_nickname: String = user.get("nickname");
    let ciyuanxi_id: String = user.get("ciyuanxi_id");
    if old_nickname == new_nickname {
        return err(400, "新昵称与当前昵称相同");
    }
    // 昵称唯一性：与注册/新增保持一致，同时检查管理员表和普通用户表
    let admin_exists = sqlx::query("SELECT id FROM admin_users WHERE username = ? LIMIT 1")
        .bind(&new_nickname)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    let exists = sqlx::query("SELECT id FROM app_users WHERE nickname = ? AND id <> ? LIMIT 1")
        .bind(&new_nickname)
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if admin_exists || exists {
        return err(409, "昵称已存在");
    }
    // 更新昵称
    let upd = sqlx::query("UPDATE app_users SET nickname = ? WHERE id = ?")
        .bind(&new_nickname)
        .bind(id)
        .execute(pool)
        .await;
    if let Err(e) = upd {
        return err(500, &format!("修改失败: {}", e));
    }
    // 同步 user_feedback 表中的昵称（保持头像/昵称一致性）
    if !ciyuanxi_id.is_empty() {
        let _ = sqlx::query("UPDATE user_feedback SET nickname = ? WHERE ciyuanxi_id = ? AND nickname = ?")
            .bind(&new_nickname)
            .bind(&ciyuanxi_id)
            .bind(&old_nickname)
            .execute(pool)
            .await;
    }
    // 写入客户端回执通知
    let _ = sqlx::query(
        "INSERT INTO nickname_change_notices (ciyuanxi_id, old_nickname, new_nickname, reason, changed_by) VALUES (?,?,?,?,?)",
    )
    .bind(&ciyuanxi_id)
    .bind(&old_nickname)
    .bind(&new_nickname)
    .bind(&reason)
    .bind(&ctx.username)
    .execute(pool)
    .await;
    log_operation(
        pool, ctx, "修改用户昵称",
        &format!("用户ID:{} 昵称:{}", id, old_nickname),
        &format!("昵称改为:{} 原因:{}", new_nickname, reason),
    ).await;
    ok("昵称已修改", json!({ "old_nickname": old_nickname, "new_nickname": new_nickname }))
}

/// 一键替换 user_id -> ciyuanxi_id
pub async fn replace_user_id_to_ciyuanxi(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let skip = ["app_users", "admin_users", "admin_operation_log", "admin_login_log"];
    let tables: Vec<String> = match sqlx::query("SHOW TABLES").fetch_all(pool).await {
        Ok(rows) => rows
            .iter()
            .filter_map(|r| {
                crate::admin::row_to_value(r)
                    .as_object()
                    .map(|m| m.values().next().and_then(|v| v.as_str().map(|s| s.to_string())))
                    .flatten()
            })
            .collect(),
        Err(e) => return err(500, &format!("服务器错误: {}", e)),
    };
    let mut report: Vec<Value> = Vec::new();
    for table in &tables {
        if skip.contains(&table.as_str()) {
            continue;
        }
        // 获取所有列
        let cols_rows = match sqlx::query(&format!("SHOW COLUMNS FROM `{}`", table)).fetch_all(pool).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut target_cols: Vec<String> = Vec::new();
        let mut pk = String::default();
        let mut all_cols: Vec<(String, String)> = Vec::new(); // (name, key) key=PRI
        for r in &cols_rows {
            let field: String = r.get("Field");
            let key: String = r.get("Key");
            all_cols.push((field.clone(), key.clone()));
            if key == "PRI" && pk.is_empty() {
                pk = field.clone();
            }
            if field == "user_id" || field == "owner_user_id" || field == "added_by_user_id" || field.ends_with("_user_id") {
                target_cols.push(field);
            }
        }
        if pk.is_empty() && !all_cols.is_empty() {
            pk = all_cols[0].0.clone();
        }
        for col_name in &target_cols {
            let select_sql = format!(
                "SELECT `{}`, `{}` FROM `{}` WHERE `{}` IS NOT NULL AND `{}` != '' AND `{}` != '0'",
                pk, col_name, table, col_name, col_name, col_name
            );
            let rows = match sqlx::query(&select_sql).fetch_all(pool).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut count = 0;
            for row in rows {
                let old_val: String = match row.try_get(col_name.as_str()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if !old_val.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                let cid = match lookup_ciyuanxi(pool, &old_val).await {
                    Some(c) => c,
                    None => continue,
                };
                if cid.is_empty() || cid == old_val {
                    continue;
                }
                let pk_val: i64 = match row.try_get(pk.as_str()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let upd = format!("UPDATE `{}` SET `{}` = ? WHERE `{}` = ?", table, col_name, pk);
                match sqlx::query(&upd).bind(&cid).bind(pk_val).execute(pool).await {
                    Ok(_) => count += 1,
                    Err(_) => {
                        let alter = format!("ALTER TABLE `{}` MODIFY `{}` VARCHAR(64) NOT NULL DEFAULT ''", table, col_name);
                        let _ = sqlx::query(&alter).execute(pool).await;
                        let _ = sqlx::query(&upd).bind(&cid).bind(pk_val).execute(pool).await;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                report.push(json!({ "table": table, "column": col_name, "replaced": count }));
            }
        }
    }
    log_operation(pool, ctx, "一键替换 user_id 为 ciyuanxi_id", "replace_user_id_to_ciyuanxi", &format!("{:?}", report.len())).await;
    ok("替换完成", json!({ "report": report, "total_columns": report.len() }))
}

/// 获取封禁设备列表
pub async fn list_banned_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let offset = (page - 1) * page_size;
    let keyword = str_of(&data, "keyword").trim().to_string();

    // 查询封禁设备列表：关联 app_open_log 取每台设备最新一条记录，获取硬件型号/系统/版本/所属账号，再关联 app_users 取昵称
    let base_sql = "
        SELECT 
            b.*,
            a.device_model,
            a.os_version,
            a.app_version,
            a.ciyuanxi_id,
            u.nickname
        FROM banned_devices b
        LEFT JOIN app_open_log a ON a.id = (
            SELECT o.id FROM app_open_log o
            WHERE o.device_id = b.device_id
            ORDER BY o.created_at DESC, o.id DESC LIMIT 1
        )
        LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
    ";

    let (total, rows) = if keyword.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM banned_devices")
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} ORDER BY b.created_at DESC LIMIT ? OFFSET ?", base_sql))
            .bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    } else {
        let pat = format!("%{}%", keyword);
        let where_clause = "WHERE b.device_id LIKE ? OR b.reason LIKE ? OR a.ciyuanxi_id LIKE ? OR u.nickname LIKE ?";
        let count_sql = format!(
            "SELECT COUNT(*) FROM banned_devices b
             LEFT JOIN app_open_log a ON a.id = (
                 SELECT o.id FROM app_open_log o
                 WHERE o.device_id = b.device_id
                 ORDER BY o.created_at DESC, o.id DESC LIMIT 1
             )
             LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
             {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat)
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} {} ORDER BY b.created_at DESC LIMIT ? OFFSET ?", base_sql, where_clause))
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    };

    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({ "total": total, "page": page, "page_size": page_size, "total_pages": total_pages, "list": list }))
        }
        Err(e) => err(500, &format!("查询失败: {}", e)),
    }
}

/// 获取所有设备列表（分页 + 关键词搜索，从 app_open_log 取每台设备最新一条记录）
pub async fn list_all_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let offset = (page - 1) * page_size;
    let keyword = str_of(&data, "keyword").trim().to_string();

    // 关联 app_open_log 取每台设备最新一条记录，再关联 app_users 取昵称，关联 banned_devices 判断是否被封禁。
    // 关联账号优先取 app_users.last_device_id（登录时写入，最可靠），回退到 app_open_log.ciyuanxi_id。
    let base_sql = "
        SELECT
            a.device_id,
            a.device_model,
            a.os_version,
            a.app_version,
            a.ciyuanxi_id,
            a.ip,
            a.created_at,
            COALESCE(
                (SELECT u2.nickname FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1),
                u.nickname
            ) AS nickname,
            COALESCE(
                (SELECT u2.ciyuanxi_id FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1),
                a.ciyuanxi_id
            ) AS ciyuanxi_id,
            b.id AS ban_id,
            b.reason AS ban_reason,
            (SELECT COUNT(*) FROM (
                SELECT ciyuanxi_id FROM app_open_log WHERE device_id = a.device_id AND ciyuanxi_id != ''
                UNION
                SELECT ciyuanxi_id FROM app_users WHERE last_device_id = a.device_id AND ciyuanxi_id != ''
            ) t) AS account_count,
            (SELECT COUNT(*) FROM app_users WHERE last_device_id = a.device_id) AS current_account_count
        FROM app_open_log a
        INNER JOIN (
            SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id
        ) latest ON a.id = latest.max_id
        LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
        LEFT JOIN banned_devices b ON b.device_id = a.device_id
    ";

    let (total, rows) = if keyword.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT device_id) FROM app_open_log")
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} ORDER BY a.created_at DESC LIMIT ? OFFSET ?", base_sql))
            .bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    } else {
        let pat = format!("%{}%", keyword);
        let where_clause = "WHERE a.device_id LIKE ? OR a.device_model LIKE ? OR a.ciyuanxi_id LIKE ? OR u.nickname LIKE ? OR (SELECT u2.nickname FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1) LIKE ?";
        let count_sql = format!(
            "SELECT COUNT(*) FROM (
                SELECT 1 FROM app_open_log a
                INNER JOIN (
                    SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id
                ) latest ON a.id = latest.max_id
                LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
                {}
            ) t", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat)
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} {} ORDER BY a.created_at DESC LIMIT ? OFFSET ?", base_sql, where_clause))
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    };

    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({ "total": total, "page": page, "page_size": page_size, "total_pages": total_pages, "list": list }))
        }
        Err(e) => err(500, &format!("查询失败: {}", e)),
    }
}

/// 封禁设备
pub async fn ban_device(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let reason = str_of(&data, "reason").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }
    if reason.is_empty() {
        return err(400, "封禁原因不能为空");
    }
    let result = sqlx::query("INSERT IGNORE INTO banned_devices (device_id, reason, banned_by) VALUES (?, ?, ?)")
        .bind(&device_id)
        .bind(&reason)
        .bind(&ctx.username)
        .execute(pool)
        .await;
    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return ok("该设备已在封禁列表中", Value::Null);
            }
            log_operation(pool, ctx, "封禁设备", &format!("设备ID:{}", device_id), &format!("原因:{} 操作人:{}", reason, ctx.username)).await;
            ok("已封禁", Value::Null)
        }
        Err(e) => err(500, &format!("操作失败: {}", e)),
    }
}

/// 解封设备
pub async fn unban_device(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let id = int_of(&data, "id");
    if device_id.is_empty() && id <= 0 {
        return err(400, "需要提供设备ID或记录ID");
    }
    let result = if id > 0 {
        sqlx::query("DELETE FROM banned_devices WHERE id = ?").bind(id).execute(pool).await
    } else {
        sqlx::query("DELETE FROM banned_devices WHERE device_id = ?").bind(&device_id).execute(pool).await
    };
    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return ok("该设备不在封禁列表中", Value::Null);
            }
            log_operation(pool, ctx, "解封设备", &format!("设备ID:{} ID:{}", device_id, id), &format!("操作人:{}", ctx.username)).await;
            ok("已解封", Value::Null)
        }
        Err(e) => err(500, &format!("操作失败: {}", e)),
    }
}

/// 查询用户关联的设备ID列表
pub async fn get_user_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let user_id = int_of(&data, "user_id");
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if user_id <= 0 && ciyuanxi_id.is_empty() {
        return err(400, "需要提供用户ID或弦予号");
    }

    // 从 app_users 获取最后登录设备
    let user_row = if user_id > 0 {
        sqlx::query("SELECT ciyuanxi_id, nickname, last_device_id FROM app_users WHERE id = ? LIMIT 1")
            .bind(user_id).fetch_optional(pool).await
    } else {
        sqlx::query("SELECT ciyuanxi_id, nickname, last_device_id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id).fetch_optional(pool).await
    };

    let user_row = match user_row {
        Ok(Some(r)) => r,
        _ => return err(404, "用户不存在"),
    };

    let username: String = user_row.get("nickname");
    let user_ciyuanxi_id: String = user_row.get("ciyuanxi_id");
    let last_device_id: String = user_row.get("last_device_id");

    // 查询该设备的登录记录
    let login_logs = if !last_device_id.is_empty() {
        sqlx::query("SELECT device_id, ip, created_at FROM admin_app_login_log WHERE device_id = ? ORDER BY created_at DESC LIMIT 20")
            .bind(&last_device_id).fetch_all(pool).await
    } else {
        Ok(vec![])
    };

    // 查询该设备的启动记录
    let open_logs = if !last_device_id.is_empty() {
        sqlx::query("SELECT device_id, ip, app_version, created_at FROM app_open_log WHERE device_id = ? ORDER BY created_at DESC LIMIT 20")
            .bind(&last_device_id).fetch_all(pool).await
    } else {
        Ok(vec![])
    };

    // 检查设备是否被封禁
    let is_banned = if !last_device_id.is_empty() {
        sqlx::query("SELECT id FROM banned_devices WHERE device_id = ? LIMIT 1")
            .bind(&last_device_id).fetch_optional(pool).await
            .ok().flatten().is_some()
    } else {
        false
    };

    let login_list: Vec<Value> = login_logs.unwrap_or_default().iter().map(|r| json!({
        "device_id": r.try_get::<String, _>("device_id").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).collect();

    let open_list: Vec<Value> = open_logs.unwrap_or_default().iter().map(|r| json!({
        "device_id": r.try_get::<String, _>("device_id").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).collect();

    ok("ok", json!({
        "nickname": username,
        "ciyuanxi_id": user_ciyuanxi_id,
        "last_device_id": last_device_id,
        "is_banned": is_banned,
        "login_logs": login_list,
        "open_logs": open_list,
    }))
}

/// 获取设备详情：关联账号列表、当前关联账号、封禁状态、听歌统计
pub async fn get_device_detail(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    // 设备最新一条启动记录
    let latest = sqlx::query(
        "SELECT device_id, device_model, os_version, app_version, ip, ciyuanxi_id, created_at
         FROM app_open_log WHERE device_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    // 封禁状态
    let ban_row = sqlx::query("SELECT id, reason, banned_by, created_at FROM banned_devices WHERE device_id = ? LIMIT 1")
        .bind(&device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let is_banned = ban_row.is_some();
    let ban_info = ban_row.map(|r| json!({
        "id": r.try_get::<i64, _>("id").unwrap_or_default(),
        "reason": r.try_get::<String, _>("reason").unwrap_or_default(),
        "banned_by": r.try_get::<String, _>("banned_by").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).unwrap_or(Value::Null);

    // 关联账号：从 app_open_log 取所有出现过的 ciyuanxi_id，并合并 app_users.last_device_id 关联的账号
    let account_rows = sqlx::query(
        "SELECT DISTINCT ciyuanxi_id FROM (
            SELECT a.ciyuanxi_id FROM app_open_log a WHERE a.device_id = ? AND a.ciyuanxi_id != ''
            UNION
            SELECT u.ciyuanxi_id FROM app_users u WHERE u.last_device_id = ? AND u.ciyuanxi_id != ''
        ) t",
    )
    .bind(&device_id)
    .bind(&device_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 当前关联账号：app_users.last_device_id = device_id
    let current_account = sqlx::query(
        "SELECT id, ciyuanxi_id, nickname, listen_duration, unique_songs_count, avatar_url
         FROM app_users WHERE last_device_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let current_ciyuanxi_id: String = current_account
        .as_ref()
        .and_then(|r| r.try_get::<String, _>("ciyuanxi_id").ok())
        .unwrap_or_default();

    let mut accounts: Vec<Value> = Vec::new();
    for row in &account_rows {
        let cid: String = row.try_get("ciyuanxi_id").unwrap_or_default();
        if cid.is_empty() {
            continue;
        }
        // 查每个账号的详情
        let user_info = sqlx::query(
            "SELECT id, nickname, listen_duration, unique_songs_count, avatar_url, last_device_id
             FROM app_users WHERE ciyuanxi_id = ? LIMIT 1",
        )
        .bind(&cid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
        if let Some(u) = user_info {
            let is_current = u.try_get::<String, _>("last_device_id").unwrap_or_default() == device_id;
            accounts.push(json!({
                "ciyuanxi_id": cid,
                "nickname": u.try_get::<String, _>("nickname").unwrap_or_default(),
                "listen_duration": u.try_get::<u32, _>("listen_duration").unwrap_or_default(),
                "unique_songs_count": u.try_get::<u32, _>("unique_songs_count").unwrap_or_default(),
                "avatar_url": u.try_get::<Option<String>, _>("avatar_url").unwrap_or_default(),
                "is_current": is_current,
            }));
        }
    }

    // 如果 app_open_log 里有 ciyuanxi_id 但 app_users 表里没有的，也加入（未注册的）
    // 同时确保当前关联账号也在列表中
    let has_current = accounts.iter().any(|a| a["ciyuanxi_id"].as_str() == Some(&current_ciyuanxi_id));
    if !has_current && !current_ciyuanxi_id.is_empty() {
        if let Some(u) = &current_account {
            accounts.insert(0, json!({
                "ciyuanxi_id": current_ciyuanxi_id,
                "nickname": u.try_get::<String, _>("nickname").unwrap_or_default(),
                "listen_duration": u.try_get::<u32, _>("listen_duration").unwrap_or_default(),
                "unique_songs_count": u.try_get::<u32, _>("unique_songs_count").unwrap_or_default(),
                "avatar_url": u.try_get::<Option<String>, _>("avatar_url").unwrap_or_default(),
                "is_current": true,
            }));
        }
    }

    let device_info = latest.map(|r| json!({
        "device_model": r.try_get::<String, _>("device_model").unwrap_or_default(),
        "os_version": r.try_get::<String, _>("os_version").unwrap_or_default(),
        "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).unwrap_or(Value::Null);

    ok("ok", json!({
        "device_id": device_id,
        "device_info": device_info,
        "is_banned": is_banned,
        "ban_info": ban_info,
        "associated_accounts": accounts,
        "account_count": accounts.len(),
        "current_ciyuanxi_id": current_ciyuanxi_id,
    }))
}

/// 重置设备的听歌统计：对该设备上所有关联账号执行重置
pub async fn reset_device_listen_stats(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    // 收集所有关联账号的 ciyuanxi_id
    let rows = sqlx::query(
        "SELECT DISTINCT ciyuanxi_id FROM app_open_log WHERE device_id = ? AND ciyuanxi_id != ''",
    )
    .bind(&device_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 也加上 app_users.last_device_id 关联的账号
    let current = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE last_device_id = ?")
        .bind(&device_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let mut ciyuanxi_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &rows {
        let cid: String = r.try_get("ciyuanxi_id").unwrap_or_default();
        if !cid.is_empty() {
            ciyuanxi_ids.insert(cid);
        }
    }
    for r in &current {
        let cid: String = r.try_get("ciyuanxi_id").unwrap_or_default();
        if !cid.is_empty() {
            ciyuanxi_ids.insert(cid);
        }
    }

    if ciyuanxi_ids.is_empty() {
        return ok("该设备未关联任何账号，无需重置", Value::Null);
    }

    let mut reset_count = 0u32;
    for cid in &ciyuanxi_ids {
        let _ = sqlx::query(
            "UPDATE app_users SET listen_duration = 0, unique_songs_count = 0,
             listen_stats_reset_at = NOW(), listen_duration_offset = 0, unique_songs_offset = 0
             WHERE ciyuanxi_id = ?",
        )
        .bind(cid)
        .execute(pool)
        .await;
        let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
            .bind(cid)
            .execute(pool)
            .await;
        reset_count += 1;
    }

    log_operation(
        pool, ctx, "重置设备听歌统计",
        &format!("device_id={} 关联{}个账号", device_id, reset_count), "",
    ).await;

    ok(&format!("已重置 {} 个关联账号的听歌统计", reset_count), Value::Null)
}

/// 删除设备记录：从 app_open_log 和 banned_devices 中删除
pub async fn delete_device_record(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let _ = sqlx::query("DELETE FROM app_open_log WHERE device_id = ?")
        .bind(&device_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM banned_devices WHERE device_id = ?")
        .bind(&device_id)
        .execute(pool)
        .await;

    log_operation(pool, ctx, "删除设备记录", &format!("device_id={}", device_id), "").await;

    ok("设备记录已删除", Value::Null)
}

/// 批量删除设备记录
pub async fn batch_delete_devices(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_ids: Vec<String> = data.get("device_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    if device_ids.is_empty() {
        return err(400, "请选择要删除的设备");
    }

    let mut total_deleted = 0u64;
    for did in &device_ids {
        let r1 = sqlx::query("DELETE FROM app_open_log WHERE device_id = ?")
            .bind(did)
            .execute(pool)
            .await;
        let r2 = sqlx::query("DELETE FROM banned_devices WHERE device_id = ?")
            .bind(did)
            .execute(pool)
            .await;
        if r1.is_ok() || r2.is_ok() {
            total_deleted += 1;
        }
    }

    log_operation(
        pool, ctx, "批量删除设备记录",
        &format!("删除{}台设备", total_deleted), "",
    ).await;

    ok(&format!("已删除 {} 台设备的记录", total_deleted), Value::Null)
}

/// 获取设备关联账号的插件信息（取当前关联账号）
pub async fn get_device_plugins(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    // 找当前关联账号
    let row = sqlx::query("SELECT ciyuanxi_id, nickname FROM app_users WHERE last_device_id = ? ORDER BY id DESC LIMIT 1")
        .bind(&device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return ok("ok", json!({ "device_id": device_id, "nickname": null, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null, "message": "该设备未关联账号" }));
    };

    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let nickname: String = row.get("nickname");

    if ciyuanxi_id.is_empty() {
        return ok("ok", json!({ "device_id": device_id, "nickname": nickname, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null }));
    }

    let clean_id: String = ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let dir = std::path::Path::new("data").join("sync").join(&clean_id);
    let file = dir.join("plugins.json");
    let content = match std::fs::read_to_string(&file) {
        Ok(c) => c,
        Err(_) => return ok("ok", json!({ "device_id": device_id, "nickname": nickname, "ciyuanxi_id": ciyuanxi_id, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null })),
    };
    let save_data: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return err(500, "数据解析失败"),
    };
    let mut plugins: Vec<Value> = Vec::new();
    let uploaded_at = save_data.get("uploaded_at").cloned().unwrap_or(Value::Null);
    if let Some(list) = save_data.get("plugins").and_then(|x| x.as_array()) {
        for p in list {
            let script_size = p.get("script").and_then(|s| s.as_str()).map(|s| s.len()).unwrap_or(0);
            plugins.push(json!({
                "name": p.get("name").and_then(|x| x.as_str()).unwrap_or("(未知)"),
                "format": p.get("format").and_then(|x| x.as_str()).unwrap_or("unknown"),
                "version": p.get("version").and_then(|x| x.as_str()).unwrap_or(""),
                "author": p.get("author").and_then(|x| x.as_str()).unwrap_or(""),
                "description": p.get("description").and_then(|x| x.as_str()).unwrap_or(""),
                "enabled": p.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false),
                "filePath": p.get("filePath").and_then(|x| x.as_str()).unwrap_or(""),
                "importedAt": p.get("importedAt").and_then(|x| x.as_i64()).unwrap_or(0),
                "scriptSize": script_size,
            }));
        }
    }
    let plugin_count = plugins.len();
    ok("ok", json!({
        "device_id": device_id,
        "nickname": nickname,
        "ciyuanxi_id": ciyuanxi_id,
        "plugins": plugins,
        "plugin_count": plugin_count,
        "uploaded_at": uploaded_at,
    }))
}

async fn lookup_ciyuanxi(pool: &MySqlPool, user_id: &str) -> Option<String> {
    let row = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE id = ? OR ciyuanxi_id = ? LIMIT 1")
        .bind(user_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;
    row.try_get("ciyuanxi_id").ok()
}
