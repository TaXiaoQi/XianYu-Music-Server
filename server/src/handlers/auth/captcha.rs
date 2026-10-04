use super::*;

pub async fn get_captcha(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let purpose = str_of(&data, "purpose").trim().to_string();
    let purpose = if purpose.is_empty() { "auth".to_string() } else { purpose };
    let ip = ctx.client_ip.clone();

    let recent_count: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM human_captcha_challenges WHERE ip = ? AND created_at > DATE_SUB(NOW(), INTERVAL 60 SECOND)",
    )
    .bind(&ip)
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);
    if recent_count >= 20 {
        return ctx.err(429, "请求过于频繁，请稍后再试");
    }

    let _ = sqlx::query("DELETE FROM human_captcha_challenges WHERE expires_at <= NOW() OR created_at < DATE_SUB(NOW(), INTERVAL 1 DAY)")
        .execute(pool)
        .await;

    let ops = ['+', '-', '×', '÷'];
    let op = ops[crate::handlers::helpers::random_int(0, 3) as usize];
    let (left, right, answer) = match op {
        '+' => {
            let a = crate::handlers::helpers::random_int(1, 9);
            let b = crate::handlers::helpers::random_int(1, 9);
            (a, b, a + b)
        }
        '-' => {
            let a = crate::handlers::helpers::random_int(2, 9);
            let b = crate::handlers::helpers::random_int(1, a - 1);
            (a, b, a - b)
        }
        '×' => {
            let a = crate::handlers::helpers::random_int(1, 9);
            let b = crate::handlers::helpers::random_int(1, 9);
            (a, b, a * b)
        }
        '÷' => {
            let a = crate::handlers::helpers::random_int(1, 9);
            let b = crate::handlers::helpers::random_int(1, 9);
            (a * b, b, a)
        }
        _ => unreachable!(),
    };
    let captcha_id = crate::handlers::helpers::random_hex(16);
    let answer = answer.to_string();
    let question = format!("{} {} {} = ?", left, op, right);

    let result = sqlx::query(
        "INSERT INTO human_captcha_challenges (captcha_id, purpose, answer, ip, expires_at) VALUES (?,?,?,?,DATE_ADD(NOW(), INTERVAL ? MINUTE))",
    )
    .bind(&captcha_id)
    .bind(&purpose)
    .bind(&answer)
    .bind(&ip)
    .bind(CAPTCHA_TTL_MINUTES)
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok(
            "ok",
            json!({
                "captcha_id": captcha_id,
                "question": question,
                "expire_seconds": CAPTCHA_TTL_MINUTES * 60,
            }),
        ),
        Err(e) => { tracing::error!("验证码生成失败: {e}"); ctx.err(500, "验证码生成失败") },
    }
}

pub async fn verify_captcha(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let captcha_id = str_of(&data, "captcha_id").trim().to_string();
    let captcha_answer = str_of(&data, "captcha_answer").trim().to_string();
    let purpose = str_of(&data, "purpose").trim().to_string();
    let purpose = if purpose.is_empty() { "auth".to_string() } else { purpose };

    if captcha_id.is_empty() || captcha_answer.is_empty() {
        return ctx.err(400, "请完成人机验证");
    }

    let row = sqlx::query(
        "SELECT answer, ip FROM human_captcha_challenges WHERE captcha_id = ? AND purpose = ? AND used = 0 AND expires_at > NOW() LIMIT 1",
    )
    .bind(&captcha_id)
    .bind(&purpose)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return ctx.err(400, "人机验证已过期，请刷新后重试");
    };

    let expected: String = row.get("answer");
    let ip: String = row.get("ip");
    // 一次性消费：无论对错本题即废（对齐 require_captcha），防反复枚举答案；
    // 通过时保留 used=0 供业务请求二次消费后作废
    let id: i64 = row.get("id");
    let _ = sqlx::query("UPDATE human_captcha_challenges SET used = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;
    if ip != ctx.client_ip || expected.trim() != captcha_answer {
        return ctx.err(400, "人机验证错误，请重新输入");
    }

    ctx.ok("验证通过", json!({ "verified": true }))
}

pub(crate) async fn require_captcha(data: &serde_json::Value, ctx: &ReqCtx, pool: &MySqlPool, purpose: &str) -> Option<Response> {
    let captcha_config = crate::handlers::email_auth::load_captcha_config(pool, &ctx.config).await;
    let use_provider_captcha = captcha_config.enabled
        && !captcha_config.site_key.trim().is_empty()
        && !captcha_config.secret.trim().is_empty();
    if use_provider_captcha {
        let captcha_token = {
            let token = str_of(data, "captcha_token");
            if token.trim().is_empty() {
                str_of(data, "turnstile_token")
            } else {
                token
            }
        };
        match crate::handlers::email_auth::verify_captcha_token(&captcha_config, &captcha_token, &ctx.client_ip).await {
            Ok(true) => return None,
            Ok(false) => return Some(ctx.err(400, "请先完成人机验证")),
            Err(e) => {
                eprintln!("[auth] 人机验证校验失败: {}", e);
                return Some(ctx.err(500, "人机验证服务暂不可用，请稍后重试"));
            }
        }
    }

    let captcha_id = str_of(data, "captcha_id").trim().to_string();
    let captcha_answer = str_of(data, "captcha_answer").trim().to_string();
    if captcha_id.is_empty() || captcha_answer.is_empty() {
        return Some(ctx.err(400, "请完成人机验证"));
    }

    let row = sqlx::query(
        "SELECT id, answer, ip FROM human_captcha_challenges WHERE captcha_id = ? AND purpose = ? AND used = 0 AND expires_at > NOW() LIMIT 1",
    )
    .bind(&captcha_id)
    .bind(purpose)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return Some(ctx.err(400, "人机验证已过期，请刷新后重试"));
    };

    let id: i64 = row.get("id");
    let expected: String = row.get("answer");
    let ip: String = row.get("ip");
    let _ = sqlx::query("UPDATE human_captcha_challenges SET used = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;

    if ip != ctx.client_ip || expected.trim() != captcha_answer {
        return Some(ctx.err(400, "人机验证错误，请刷新后重试"));
    }
    None
}
