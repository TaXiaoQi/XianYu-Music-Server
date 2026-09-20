use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

const ALGO_CACHE_TTL: Duration = Duration::from_secs(600);
const ALGO_CACHE_MAX_ENTRIES: usize = 1024;
const TARGET_COUNT: i64 = 30;
const PROFILE_WINDOW_DAYS: i32 = 90;
const EXCLUDE_WINDOW_DAYS: i32 = 14;
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

fn cache_remove(ciyuanxi_id: &str) {
    if let Some(cache) = ALGO_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.remove(ciyuanxi_id);
        }
    }
}

fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn rotate_pick<T: Clone>(items: &[T], start: usize, count: usize) -> Vec<T> {
    let n = items.len();
    if n == 0 {
        return vec![];
    }
    (0..count.min(n)).map(|i| items[(start + i) % n].clone()).collect()
}

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

const FRESH_KEYWORD_POOL: &[&str] = &[
    "华语流行", "热门歌曲", "经典老歌", "轻音乐", "伤感", "治愈", "粤语经典",
    "民谣精选", "电子舞曲", "抖音热歌", "影视金曲", "摇滚经典",
];

pub async fn get_daily_recommend(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        return ctx.err(401, "请先登录后使用每日推荐");
    }

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
    cache_remove(&ciyuanxi_id);
    ctx.ok("ok", json!({}))
}

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
    cache_remove(&ciyuanxi_id);
    ctx.ok("ok", json!({ "saved": saved }))
}

fn clip(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
