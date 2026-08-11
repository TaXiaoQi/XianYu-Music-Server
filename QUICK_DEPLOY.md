# 最简部署

适合只想尽快把服务跑起来的场景。当前部署包已包含后台前端、Rust 服务端、配置文件和启动脚本，上传后可以由服务端直接托管后台页面。

## Windows 构建 Linux 部署包

在项目根目录双击或运行：

```bat
build-linux.cmd
```

完成后会生成：

```text
deploy-linux/
├── server/
│   ├── server
│   ├── config.json
│   └── uploads/
├── admin-web/dist/
├── start-server-linux.sh
└── xianyu-music-server.service
```

把整个 `deploy-linux/` 上传到 Linux 服务器。

## Linux 服务器启动

进入上传后的目录：

```bash
chmod +x server/server
chmod +x start-server-linux.sh
./start-server-linux.sh
```

然后访问：

```text
http://服务器IP:8081/login
```

默认后台账号来自 `server/config.json`：

```text
admin / adminadmin
```

首次上线前建议修改：

- `admin_password`
- `api_secret`
- `jwt_secret`
- `db_pass`

## 可选：配置数据库

没有数据库也能先启动，服务端会进入本地缓存模式。

需要正式使用数据库时：

1. 登录后台。
2. 打开「配置文件管理」。
3. 填写数据库连接信息并保存。
4. 点击「迁移本地缓存」。
5. 重启服务端。

## 可选：绑定域名

如果只用 IP 和端口访问，不需要 Nginx。

需要域名、HTTPS 或隐藏 `8081` 端口时，再配置 Nginx 反向代理：

```nginx
location / {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

## 更新发布

重新运行：

```bat
build-linux.cmd
```

上传新的：

- `server/server`
- `admin-web/dist/`

不要覆盖生产环境已经修改过的 `server/config.json`。
