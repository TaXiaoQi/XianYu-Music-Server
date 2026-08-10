use axum::response::Response;
use serde_json::json;
use sqlx::{MySqlPool, Row};

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};

pub const DEFAULT_USER_AGREEMENT_TITLE: &str = "弦予音乐用户协议";

pub const DEFAULT_USER_AGREEMENT_CONTENT: &str = r#"一、协议范围
本协议适用于弦予音乐客户端账号系统及相关云端同步、资料管理、统计上报、风控安全服务。用户注册、登录或继续使用账号功能，即表示已阅读并同意本协议。

二、账号注册与使用
用户应使用真实、有效的邮箱完成注册，并妥善保管账号、密码和邮箱验证码。因用户主动泄露、共享账号或使用非官方客户端造成的损失，由用户自行承担。

三、本地数据读取说明
为提供账号登录、设备安全识别、播放统计、同步和故障排查功能，账号系统可能读取或生成以下本地数据：本机设备标识、客户端版本、操作系统版本、设备型号、登录状态凭证、用户主动上传的头像、本地收藏、歌单、播放历史、听歌时长等音乐使用数据，以及软件运行错误日志。上述数据仅用于账号服务、安全风控、功能同步、异常定位和产品维护。

四、数据上报与安全
客户端启动、登录、注册、搜索、播放统计、错误反馈等行为可能向服务器上报必要信息，包括设备ID、IP地址、账号ID、客户端版本、操作系统版本、设备型号、行为时间和必要的请求参数。我们将尽合理努力保护数据安全，不会主动出售用户个人信息。

五、禁止行为
用户不得利用账号系统进行恶意攻击、批量注册、刷量、破解、逆向、绕过限制、上传违法违规内容、干扰服务器稳定性或侵犯他人权益。发现异常行为时，平台有权限制、封禁账号或设备。

六、封禁与申诉
若账号或设备因违反协议、安全风控或恶意行为被封禁，登录时将提示封禁状态及原因。用户如认为处理有误，可联系管理员并提供账号、设备ID及相关说明进行核查。

七、协议更新
平台可根据功能调整、安全要求或法律合规需要更新本协议。更新后继续使用账号功能，视为接受更新后的协议内容。"#;

pub async fn load_user_agreement(pool: &MySqlPool) -> (String, String) {
    let rows = sqlx::query(
        "SELECT setting_key, setting_value FROM server_settings WHERE setting_key IN ('user_agreement_title', 'user_agreement_content')",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut title = DEFAULT_USER_AGREEMENT_TITLE.to_string();
    let mut content = DEFAULT_USER_AGREEMENT_CONTENT.to_string();
    for row in rows {
        let key: String = row.try_get("setting_key").unwrap_or_default();
        let value: String = row.try_get::<Option<String>, _>("setting_value").unwrap_or_default().unwrap_or_default();
        if key == "user_agreement_title" && !value.trim().is_empty() {
            title = value;
        } else if key == "user_agreement_content" && !value.trim().is_empty() {
            content = value;
        }
    }
    (title, content)
}

pub async fn get(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let (title, content) = load_user_agreement(pool).await;
    ok("ok", json!({ "title": title, "content": content }))
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    if title.is_empty() {
        return err(400, "协议标题不能为空");
    }
    if content.len() < 20 {
        return err(400, "协议内容过短");
    }

    let save_title = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES ('user_agreement_title', ?, '客户端账号登录/注册用户协议标题') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(&title)
    .execute(pool)
    .await;
    if let Err(e) = save_title {
        return err(500, &format!("保存标题失败: {}", e));
    }

    let save_content = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES ('user_agreement_content', ?, '客户端账号登录/注册用户协议内容') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(&content)
    .execute(pool)
    .await;
    if let Err(e) = save_content {
        return err(500, &format!("保存内容失败: {}", e));
    }

    log_operation(pool, ctx, "保存用户协议", "user_agreement", &format!("标题:{}", title)).await;
    ok("保存成功", json!({ "title": title, "content": content }))
}
