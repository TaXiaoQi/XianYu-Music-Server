use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

/// 算法缓存有效期（秒）：同一用户当天的算法在有效期内直接复用
const ALGO_CACHE_TTL: Duration = Duration::from_secs(600);
/// 缓存容量上限，超出时整体清空（简单防膨胀）
const ALGO_CACHE_MAX_ENTRIES: usize = 1024;
/// 推荐结果数量（客户端按此截取展示）
const TARGET_COUNT: i64 = 30;
/// 画像聚合窗口（天）
const PROFILE_WINDOW_DAYS: i32 = 90;
/// 排除窗口（天）：近 N 天听过的歌曲不再推荐
const EXCLUDE_WINDOW_DAYS: i32 = 14;
/// 正反馈权重：收藏/添加到歌单记 1 次 ≈ 听 N 次（强偏好信号）
const LIKE_WEIGHT: i64 = 5;

struct AlgoCacheEntry {
    date: String,
    algo: Value,
    at: Instant,
}

static ALGO_CACHE: OnceLock<Mutex<HashMap<String, AlgoCacheEntry>>> = OnceLock::new();

fn cache_get(ciyuanxi_id: &str, date: &str) -> Option<Value> {
    let cache = ALGO_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().ok()?;
    let expired = |e: &AlgoCacheEntry| e.date != date || e.at.elapsed() > ALGO_CACHE_TTL;
    match guard.get(ciyuanxi_id) {
        Some(entry) if !expired(entry) => Some(entry.algo.clone()),
        Some(_) => {
            guard.remove(ciyuanxi_id);
            None
        }
        None => None,
    }
}

fn cache_put(ciyuanxi_id: String, date: String, algo: Value) {
    let cache = ALGO_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= ALGO_CACHE_MAX_ENTRIES {
            guard.clear();
        }
        guard.insert(ciyuanxi_id, AlgoCacheEntry { date, algo, at: Instant::now() });
    }
}

/// 负反馈上报后移除该用户当日算法缓存，使排除项变更即时生效
fn cache_remove(ciyuanxi_id: &str) {
    if let Some(cache) = ALGO_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.remove(ciyuanxi_id);
        }
    }
}

/// FNV-1a 哈希：用于生成确定性每日种子（同用户同一天结果一致，跨天变化）
fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// 从环形池中取 count 个元素（起点由每日种子决定，实现查询词每日轮换）
fn rotate_pick<T: Clone>(items: &[T], start: usize, count: usize) -> Vec<T> {
    let n = items.len();
    if n == 0 {
        return vec![];
    }
    (0..count.min(n)).map(|i| items[(start + i) % n].clone()).collect()
}

/// 清理歌名作为搜索词：去掉括号后缀（Live/翻自/伴奏等）与过短的词
fn clean_song_query(name: &str) -> String {
    let mut s = name.trim().to_string();
    for sep in ['（', '('] {
        if let Some(pos) = s.find(sep) {
            s.truncate(pos);
        }
    }
    let s = s.trim().to_string();
    if s.chars().count() < 2 {
        String::new()
    } else {
        s
    }
}

/// 清理歌手名作为搜索词：多歌手时取第一位
fn clean_artist_query(artist: &str) -> String {
    let s = artist.trim();
    for sep in ['/', '、', ',', '&'] {
        if let Some(pos) = s.find(sep) {
            let first = s[..pos].trim();
            if first.chars().count() >= 2 {
                return first.to_string();
            }
        }
    }
    if s.chars().count() < 2 {
        String::new()
    } else {
        s.to_string()
    }
}

/// 兜底发现关键词池：新用户或历史不足时以热门词发现新歌
const FRESH_KEYWORD_POOL: &[&str] = &[
    "华语流行", "热门歌曲", "经典老歌", "轻音乐", "伤感", "治愈", "粤语经典",
    "民谣精选", "电子舞曲", "抖音热歌", "影视金曲", "摇滚经典",
];

/// get_daily_recommend 每日推荐算法下发
///
/// 基于账号播放历史（play_history）聚合用户画像（常听歌手/歌曲），
/// 生成"推荐算法"描述（策略 DSL：类型 + 权重 + 查询词 + 排除项 + 每日种子），
/// 由客户端在本机调用已安装的音源插件执行搜索并整理出每日推荐歌曲板块。
/// 算法本体（策略构成、权重、查询词轮换）由服务器决策并可随时调整，无需发版。
pub async fn get_daily_recommend(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        return ctx.err(401, "请先登录后使用每日推荐");
    }

    // 北京时间当日日期与生成时间
    let today: Option<String> = sqlx::query_scalar(
        "SELECT DATE_FORMAT(NOW() + INTERVAL 8 HOUR, '%Y-%m-%d')",
    )
    .fetch_one(pool)
    .await
    .ok();
    let today = match today {
        Some(d) if !d.is_empty() => d,
        _ => return ctx.err(500, "服务器错误"),
    };

    if let Some(algo) = cache_get(&ciyuanxi_id, &today) {
        return ctx.ok("ok", algo);
    }

    // ── 画像聚合：近 90 天常听歌手 ──────────────────────────────
    let artist_rows = sqlx::query(
        "SELECT singer, COUNT(*) AS c FROM play_history \
         WHERE ciyuanxi_id = ? AND singer <> '' \
         AND played_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY \
         GROUP BY singer ORDER BY c DESC LIMIT 15",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;

    let top_artists: Vec<(String, i64)> = artist_rows
        .unwrap_or_default()
        .iter()
        .filter_map(|r| {
            let singer: String = r.try_get("singer").unwrap_or_default();
            let c: i64 = r.try_get("c").unwrap_or(0);
            if singer.is_empty() { None } else { Some((singer, c)) }
        })
        .collect();

    // ── 画像聚合：近 90 天常听歌曲 ──────────────────────────────
    let song_rows = sqlx::query(
        "SELECT song_name, singer, COUNT(*) AS c FROM play_history \
         WHERE ciyuanxi_id = ? AND song_name <> '' \
         AND played_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY \
         GROUP BY song_name, singer ORDER BY c DESC LIMIT 15",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;

    let top_songs: Vec<(String, String, i64)> = song_rows
        .unwrap_or_default()
        .iter()
        .filter_map(|r| {
            let name: String = r.try_get("song_name").unwrap_or_default();
            let singer: String = r.try_get("singer").unwrap_or_default();
            let c: i64 = r.try_get("c").unwrap_or(0);
            if name.is_empty() { None } else { Some((name, singer, c)) }
        })
        .collect();

    // ── 正反馈并入画像：收藏 / 添加到歌单 = 「喜欢这类歌」 ─────────
    // 与播放历史同键聚合，按 LIKE_WEIGHT 加权后重排取前 15。
    let like_artist_rows = sqlx::query(
        "SELECT singer, COUNT(*) AS c FROM daily_like \
         WHERE ciyuanxi_id = ? AND singer <> '' \
         AND created_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY \
         GROUP BY singer",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;
    let mut artist_scores: std::collections::HashMap<String, i64> = HashMap::new();
    for (s, c) in &top_artists {
        *artist_scores.entry(s.clone()).or_insert(0) += *c;
    }
    for r in like_artist_rows.unwrap_or_default() {
        let singer: String = r.try_get("singer").unwrap_or_default();
        let c: i64 = r.try_get::<i64, _>("c").unwrap_or(0);
        if !singer.is_empty() {
            *artist_scores.entry(singer).or_insert(0) += c * LIKE_WEIGHT;
        }
    }
    let mut top_artists: Vec<(String, i64)> = artist_scores.into_iter().collect();
    top_artists.sort_by(|a, b| b.1.cmp(&a.1));
    top_artists.truncate(15);

    let like_song_rows = sqlx::query(
        "SELECT song_name, singer, COUNT(*) AS c FROM daily_like \
         WHERE ciyuanxi_id = ? AND song_name <> '' \
         AND created_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY \
         GROUP BY song_name, singer",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;
    let mut song_scores: std::collections::HashMap<(String, String), i64> = HashMap::new();
    for (n, s, c) in &top_songs {
        *song_scores.entry((n.clone(), s.clone())).or_insert(0) += *c;
    }
    for r in like_song_rows.unwrap_or_default() {
        let name: String = r.try_get("song_name").unwrap_or_default();
        let singer: String = r.try_get("singer").unwrap_or_default();
        let c: i64 = r.try_get::<i64, _>("c").unwrap_or(0);
        if !name.is_empty() {
            *song_scores.entry((name, singer)).or_insert(0) += c * LIKE_WEIGHT;
        }
    }
    let mut top_songs: Vec<(String, String, i64)> = song_scores
        .into_iter()
        .map(|((n, s), c)| (n, s, c))
        .collect();
    top_songs.sort_by(|a, b| b.2.cmp(&a.2));
    top_songs.truncate(15);

    // ── 收听规模（决定策略权重） ─────────────────────────────────
    let overview = sqlx::query(
        "SELECT COUNT(*) AS total, COUNT(DISTINCT DATE(played_at)) AS days FROM play_history \
         WHERE ciyuanxi_id = ? \
         AND played_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_optional(pool)
    .await;
    let (total_plays, active_days) = match overview {
        Ok(Some(r)) => (
            r.try_get::<i64, _>("total").unwrap_or(0),
            r.try_get::<i64, _>("days").unwrap_or(0),
        ),
        _ => (0, 0),
    };

    // ── 排除项：近 14 天听过的歌曲 ──────────────────────────────
    let exclude_rows = sqlx::query(
        "SELECT song_name, singer FROM play_history \
         WHERE ciyuanxi_id = ? AND song_name <> '' \
         AND played_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY \
         ORDER BY played_at DESC LIMIT 300",
    )
    .bind(&ciyuanxi_id)
    .bind(EXCLUDE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;
    let exclusions: Vec<Value> = exclude_rows
        .unwrap_or_default()
        .iter()
        .map(|r| {
            json!({
                "title": r.try_get::<String, _>("song_name").unwrap_or_default(),
                "artist": r.try_get::<String, _>("singer").unwrap_or_default(),
            })
        })
        .collect();

    // ── 负反馈：用户「不喜欢」上报的歌曲并入排除项（近 90 天有效） ─
    let dislike_rows = sqlx::query(
        "SELECT song_name, singer FROM daily_dislike \
         WHERE ciyuanxi_id = ? AND song_name <> '' \
         AND created_at >= (NOW() + INTERVAL 8 HOUR) - INTERVAL ? DAY",
    )
    .bind(&ciyuanxi_id)
    .bind(PROFILE_WINDOW_DAYS)
    .fetch_all(pool)
    .await;
    let mut exclusions = exclusions;
    for r in dislike_rows.unwrap_or_default() {
        let name: String = r.try_get("song_name").unwrap_or_default();
        let singer: String = r.try_get("singer").unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        exclusions.push(json!({"title": name, "artist": singer}));
    }

    // ── 每日种子与查询词轮换 ─────────────────────────────────────
    let seed = (fnv1a(&format!("{}-{}", ciyuanxi_id, today)) % 900_000_000 + 100_000_000) as i64;
    let rotate = (seed % 97) as usize;

    let artist_pool: Vec<String> = top_artists
        .iter()
        .map(|(s, _)| clean_artist_query(s))
        .filter(|s| !s.is_empty())
        .collect();
    let song_pool: Vec<String> = top_songs
        .iter()
        .map(|(n, _, _)| clean_song_query(n))
        .filter(|s| !s.is_empty())
        .collect();

    let artist_queries = rotate_pick(&artist_pool, rotate, 4);
    let song_queries = rotate_pick(&song_pool, rotate.wrapping_add(3), 4);
    let fresh_pool: Vec<&str> = FRESH_KEYWORD_POOL.to_vec();
    let fresh_queries: Vec<String> = rotate_pick(&fresh_pool, rotate.wrapping_add(7), 3)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // ── 策略权重决策：历史越丰富，越倚重个人画像 ─────────────────
    let (mut artist_w, mut song_w, mut fresh_w): (f64, f64, f64) = if total_plays >= 40 {
        (0.45, 0.25, 0.30)
    } else if total_plays >= 10 {
        (0.35, 0.25, 0.40)
    } else {
        (0.20, 0.15, 0.65)
    };
    if artist_queries.is_empty() {
        fresh_w += artist_w;
        artist_w = 0.0;
    }
    if song_queries.is_empty() {
        fresh_w += song_w;
        song_w = 0.0;
    }

    let mut strategies: Vec<Value> = vec![];
    if artist_w > 0.0 {
        strategies.push(json!({
            "id": "artist_explore",
            "type": "artist_search",
            "weight": (artist_w * 1000.0).round() / 1000.0,
            "queries": artist_queries,
            "reason": "根据你常听的歌手",
        }));
    }
    if song_w > 0.0 {
        strategies.push(json!({
            "id": "song_recall",
            "type": "song_search",
            "weight": (song_w * 1000.0).round() / 1000.0,
            "queries": song_queries,
            "reason": "根据你常听的歌曲",
        }));
    }
    if fresh_w > 0.0 {
        strategies.push(json!({
            "id": "fresh_discover",
            "type": "keyword_search",
            "weight": (fresh_w * 1000.0).round() / 1000.0,
            "queries": fresh_queries,
            "reason": if total_plays >= 10 { "为你发现新歌" } else { "为你准备的热门歌单" },
        }));
    }

    let algo = json!({
        "version": 1,
        "date": today,
        "daily_seed": seed,
        "target_count": TARGET_COUNT,
        "profile": {
            "top_artists": top_artists.iter().map(|(s, c)| json!({"name": s, "play_count": c})).collect::<Vec<_>>(),
            "top_songs": top_songs.iter().map(|(n, s, c)| json!({"name": n, "singer": s, "play_count": c})).collect::<Vec<_>>(),
            "total_plays": total_plays,
            "active_days": active_days,
        },
        "strategies": strategies,
        "exclusions": {
            "match_mode": "title_artist",
            "songs": exclusions,
        },
        "shuffle": {
            "algorithm": "seeded",
            "seed": seed,
        },
    });

    cache_put(ciyuanxi_id.clone(), today, algo.clone());
    ctx.ok("ok", algo)
}

/// report_daily_dislike 每日推荐负反馈上报
///
/// 用户在播放页对日推歌曲点「不喜欢」时上报（歌名 + 歌手）：
/// 写入 daily_dislike 表（同用户同曲去重），并清除该用户当日算法缓存，
/// 下次生成日推时负反馈立即并入 exclusions 排除项，实现算法自调整。
pub async fn report_daily_dislike(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        return ctx.err(401, "请先登录后使用每日推荐");
    }
    let song_name = str_of(&data, "song_name");
    let singer = str_of(&data, "singer");
    if song_name.is_empty() {
        return ctx.err(400, "缺少歌曲信息");
    }
    let _ = sqlx::query(
        "INSERT IGNORE INTO daily_dislike (ciyuanxi_id, song_name, singer) VALUES (?, ?, ?)",
    )
    .bind(&ciyuanxi_id)
    .bind(&song_name)
    .bind(&singer)
    .execute(pool)
    .await;
    // 负反馈即时生效：清掉当日缓存，下次请求重新生成算法。
    cache_remove(&ciyuanxi_id);
    ctx.ok("ok", json!({}))
}

/// report_daily_like 日推正反馈上报
///
/// 收藏 / 添加到歌单代表「喜欢这类歌」：批量写入 daily_like
/// （同用户同曲去重，signal_type 记录信号来源），并清除该用户当日算法缓存，
/// 下次生成日推时正反馈按 LIKE_WEIGHT 加权并入画像（top_artists / top_songs）。
pub async fn report_daily_like(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        return ctx.err(401, "请先登录后使用每日推荐");
    }
    let songs = match data.get("songs").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return ctx.err(400, "缺少歌曲列表"),
    };
    if songs.is_empty() || songs.len() > 200 {
        return ctx.err(400, "歌曲列表为空或超限");
    }
    let signal_type = str_of(&data, "signal_type");
    let mut saved = 0i64;
    for s in &songs {
        let song_name = s.get("song_name").and_then(|v| v.as_str()).unwrap_or("");
        let singer = s.get("singer").and_then(|v| v.as_str()).unwrap_or("");
        let song_name = clip(song_name.trim(), 200);
        let singer = clip(singer.trim(), 200);
        if song_name.is_empty() {
            continue;
        }
        let res = sqlx::query(
            "INSERT IGNORE INTO daily_like (ciyuanxi_id, song_name, singer, signal_type) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(&ciyuanxi_id)
        .bind(&song_name)
        .bind(&singer)
        .bind(&signal_type)
        .execute(pool)
        .await;
        if res.is_ok() {
            saved += 1;
        }
    }
    // 正反馈即时生效：清掉当日缓存，下次请求重新生成算法。
    cache_remove(&ciyuanxi_id);
    ctx.ok("ok", json!({ "saved": saved }))
}

/// 按 UTF-8 字符边界安全截断到 max_chars 字符（数据库列宽保护）
fn clip(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
