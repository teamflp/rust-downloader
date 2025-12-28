use std::collections::HashMap;
use std::time::{Duration, Instant};
use rust_media_downloader_shared::VideoInfo;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct VideoInfoCache {
    cache: Arc<RwLock<HashMap<String, (VideoInfo, Instant)>>>,
    ttl: Duration,
}

impl VideoInfoCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, url: &str) -> Option<VideoInfo> {
        let cache = self.cache.read().await;
        cache.get(url).and_then(|(info, time)| {
            if time.elapsed() < self.ttl {
                Some(info.clone())
            } else {
                None
            }
        })
    }

    pub async fn set(&self, url: String, info: VideoInfo) {
        let mut cache = self.cache.write().await;
        cache.insert(url, (info, Instant::now()));
    }

    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, (_, time)| time.elapsed() < self.ttl);
    }
}

