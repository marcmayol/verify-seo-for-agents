//! Blocking HTTP with a per-scan cache, so several checks can share a fetch.

use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ureq::{Agent, ResponseExt};

const MAX_BODY: u64 = 3 * 1024 * 1024;
pub const USER_AGENT: &str = concat!("Mozilla/5.0 (compatible; verify-seo/", env!("CARGO_PKG_VERSION"), ")");

#[derive(Default)]
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub final_url: String,
    pub body: Vec<u8>,
    pub error: Option<String>,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    pub fn headers_all(&self, name: &str) -> Vec<&str> {
        self.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str()).collect()
    }

    pub fn content_type(&self) -> String {
        self.header("content-type").unwrap_or("").to_ascii_lowercase()
    }

    pub fn is_html(&self) -> bool {
        self.content_type().contains("html")
    }

    pub fn ok(&self) -> bool {
        self.status == 200
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).trim_start_matches('\u{feff}').to_string()
    }

    /// The JSON document in a 200 response, if that is what it holds.
    pub fn json(&self) -> Option<Value> {
        if !self.ok() {
            return None;
        }
        serde_json::from_slice(self.text().as_bytes()).ok()
    }
}

pub struct Http {
    agent: Agent,
    cache: Mutex<HashMap<String, Arc<Resp>>>,
}

impl Http {
    pub fn new() -> Self {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(12)))
            .http_status_as_error(false)
            .max_redirects(5)
            .user_agent(USER_AGENT)
            .build()
            .into();
        Http { agent, cache: Mutex::new(HashMap::new()) }
    }

    pub fn get(&self, url: &str, accept: Option<&str>) -> Arc<Resp> {
        let key = format!("{}\n{}", accept.unwrap_or(""), url);
        if let Some(hit) = self.cache.lock().unwrap().get(&key) {
            return hit.clone();
        }
        // A scan sends a burst of requests, which some sites answer with 429.
        let mut resp = self.fetch(url, accept);
        for _ in 0..2 {
            if resp.status != 429 {
                break;
            }
            let wait = resp.header("retry-after").and_then(|v| v.trim().parse::<f64>().ok()).unwrap_or(1.5);
            std::thread::sleep(Duration::from_secs_f64(wait.clamp(0.5, 3.0)));
            resp = self.fetch(url, accept);
        }
        let resp = Arc::new(resp);
        self.cache.lock().unwrap().insert(key, resp.clone());
        resp
    }

    fn fetch(&self, url: &str, accept: Option<&str>) -> Resp {
        let mut req = self.agent.get(url);
        if let Some(a) = accept {
            req = req.header("Accept", a);
        }
        match req.call() {
            Err(e) => Resp { final_url: url.to_string(), error: Some(e.to_string()), ..Default::default() },
            Ok(mut res) => {
                let status = res.status().as_u16();
                let final_url = res.get_uri().to_string();
                let headers = res
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()))
                    .collect();
                let mut body = Vec::new();
                let _ = res.body_mut().as_reader().take(MAX_BODY).read_to_end(&mut body);
                Resp { status, headers, final_url, body, error: None }
            }
        }
    }
}
