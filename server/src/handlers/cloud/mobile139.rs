// 占位：本盘代理实现待后续提交填充（返回 501，客户端将回退本地直连）。

use super::types::{CloudOut, CloudReq};

pub async fn dispatch(req: &CloudReq) -> Result<CloudOut, (i32, String)> {
    let _ = req;
    Err((501, "该盘暂未接入服务端代理".into()))
}
