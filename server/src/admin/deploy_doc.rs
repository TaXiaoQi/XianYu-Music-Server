use axum::response::Response;
use serde_json::json;
use sqlx::{MySqlPool, Row};

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};

pub const DEFAULT_DEPLOY_DOC_TITLE: &str = "弦予音乐服务端部署文档";

pub const DEFAULT_DEPLOY_DOC_CONTENT: &str = r#"## 一、环境要求
- Rust 1.85+（含 cargo）
- Node.js 18+（仅构建管理后台时需要）
- MySQL 5.7+ / 8.0（可选：不配置时自动降级为本地缓存模式）
- Linux / Windows / macOS 均可部署

## 二、获取源码
```
git clone https://github.com/TaXiaoQi/XianYu-Music-Server.git
cd XianYu-Music-Server
```

## 三、服务端配置
首次启动前在 server 目录创建 config.json：
```
{
  "host": "0.0.0.0",
  "port": 8080,
  "database_url": "mysql://user:pass@127.0.0.1:3306/xianyu_music"
}
```
数据库连接失败时自动降级为本地缓存模式（数据存储于 data/local_state.json），无需 MySQL 也可运行，响应结构与数据库模式逐字段一致。

## 四、编译与启动
```
cd server
cargo build --release
./target/release/server
```
也可使用仓库根目录的 start.bat（Windows）/ start.sh（Linux/macOS）一键启动。

## 五、管理后台
- 构建：cd admin-web && npm install && npm run build，产物 dist/ 由服务端直接托管
- 访问 http://127.0.0.1:8080/admin，首次使用请及时修改默认管理员密码

## 六、反向代理（Nginx 示例）
```
location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
location /ws {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```
WebSocket（/ws 路径）必须包含 Upgrade 与 Connection 升级头，否则协同等实时功能无法连接。

## 七、常见问题
- 端口占用：修改 config.json 的 port 后重启服务
- 忘记管理员密码：本地缓存模式删除 data 目录下的管理员状态后重启；数据库模式请在管理后台用其他超管重置
- 更多问题请到 GitHub Issues 反馈：https://github.com/TaXiaoQi/XianYu-Music-Server/issues"#;

pub async fn load_deploy_doc(pool: &MySqlPool) -> (String, String) {
    let rows = sqlx::query(
        "SELECT setting_key, setting_value FROM server_settings WHERE setting_key IN ('deploy_doc_title', 'deploy_doc_content')",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut title = DEFAULT_DEPLOY_DOC_TITLE.to_string();
    let mut content = DEFAULT_DEPLOY_DOC_CONTENT.to_string();
    for row in rows {
        let key: String = row.try_get("setting_key").unwrap_or_default();
        let value: String = row.try_get::<Option<String>, _>("setting_value").unwrap_or_default().unwrap_or_default();
        if key == "deploy_doc_title" && !value.trim().is_empty() {
            title = value;
        } else if key == "deploy_doc_content" && !value.trim().is_empty() {
            content = value;
        }
    }
    (title, content)
}

pub async fn get(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let (title, content) = load_deploy_doc(pool).await;
    ok("ok", json!({ "title": title, "content": content }))
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    if title.is_empty() {
        return err(400, "文档标题不能为空");
    }
    if content.len() < 20 {
        return err(400, "文档内容过短");
    }

    let save_title = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES ('deploy_doc_title', ?, '官网部署文档标题') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(&title)
    .execute(pool)
    .await;
    if let Err(e) = save_title {
        { tracing::error!("保存部署文档标题失败: {e}"); return err(500, "保存标题失败"); }
    }

    let save_content = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES ('deploy_doc_content', ?, '官网部署文档内容') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(&content)
    .execute(pool)
    .await;
    if let Err(e) = save_content {
        { tracing::error!("保存部署文档内容失败: {e}"); return err(500, "保存内容失败"); }
    }

    log_operation(pool, ctx, "保存部署文档", "deploy_doc", &format!("标题:{}", title)).await;
    ok("保存成功", json!({ "title": title, "content": content }))
}
