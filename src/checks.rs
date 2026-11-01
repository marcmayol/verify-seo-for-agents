//! The checks themselves. Each one takes the scan context and returns its verdict.

use crate::browser;
use crate::http::{Http, Resp};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct Ctx {
    pub http: Http,
    pub origin: String,
    pub host: String,
    pub target_url: String,
    pub target_path: String,
    pub home: Arc<Resp>,
    pub robots: Arc<Resp>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Pass,
    Fail,
    Neutral,
    Unable,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Neutral => "neutral",
            Status::Unable => "unableToCheck",
        }
    }
}

pub struct Out {
    pub status: Status,
    pub message: String,
    pub details: Value,
    pub evidence: Vec<Value>,
}

/// What a check saw, in the order it saw it.
#[derive(Default)]
struct Trace(Vec<Value>);

impl Trace {
    fn fetch(&mut self, url: &str, r: &Resp) {
        let mut e = json!({"action": "fetch", "url": url, "status": r.status});
        if !r.content_type().is_empty() {
            e["contentType"] = json!(r.content_type());
        }
        if let Some(err) = &r.error {
            e["error"] = json!(err);
        }
        self.0.push(e);
    }

    fn note(&mut self, text: impl Into<String>) {
        self.0.push(json!({"action": "note", "summary": text.into()}));
    }

    fn pass(self, msg: impl Into<String>, details: Value) -> Out {
        Out { status: Status::Pass, message: msg.into(), details, evidence: self.0 }
    }

    fn fail(self, msg: impl Into<String>) -> Out {
        self.fail_with(msg, Value::Null)
    }

    fn fail_with(self, msg: impl Into<String>, details: Value) -> Out {
        Out { status: Status::Fail, message: msg.into(), details, evidence: self.0 }
    }

    fn neutral(self, msg: impl Into<String>) -> Out {
        Out { status: Status::Neutral, message: msg.into(), details: Value::Null, evidence: self.0 }
    }

    fn unable(self, msg: impl Into<String>) -> Out {
        Out { status: Status::Unable, message: msg.into(), details: Value::Null, evidence: self.0 }
    }
}

impl Ctx {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.origin, path)
    }

    fn get(&self, t: &mut Trace, url: &str, accept: Option<&str>) -> Arc<Resp> {
        let r = self.http.get(url, accept);
        t.fetch(url, &r);
        r
    }

    fn get_path(&self, t: &mut Trace, path: &str, accept: Option<&str>) -> Arc<Resp> {
        let url = self.url(path);
        self.get(t, &url, accept)
    }

    fn robots_available(&self) -> bool {
        self.robots.ok() && !self.robots.is_html() && !parse_robots(&self.robots.text()).groups.is_empty()
    }

    /// Host, plus the bare domain when the host starts with www.
    fn domains(&self) -> Vec<String> {
        let host = self.host.split(':').next().unwrap_or("").to_string();
        let mut out = vec![host.clone()];
        if let Some(bare) = host.strip_prefix("www.") {
            out.push(bare.to_string());
        }
        out
    }

    /// Answers of a DNS-over-HTTPS query, with whether DNSSEC validated them.
    /// A second resolver covers the first one being down or blocked.
    fn doh(&self, t: &mut Trace, name: &str, kind: &str) -> Option<(Vec<Value>, bool)> {
        for base in ["https://cloudflare-dns.com/dns-query", "https://dns.google/resolve"] {
            let url = format!("{base}?name={name}&type={kind}&do=1");
            let r = self.get(t, &url, Some("application/dns-json"));
            if let Some(v) = r.json() {
                let answers = v["Answer"].as_array().cloned().unwrap_or_default();
                return Some((answers, v["AD"].as_bool().unwrap_or(false)));
            }
        }
        None
    }
}

// ---------- robots.txt ----------

pub struct RobotsGroup {
    pub agents: Vec<String>,
    pub rules: Vec<(String, String)>,
}

#[derive(Default)]
pub struct Robots {
    pub groups: Vec<RobotsGroup>,
    pub sitemaps: Vec<String>,
    pub agentmaps: Vec<String>,
    pub signals: Vec<Value>,
}

pub fn parse_robots(text: &str) -> Robots {
    let mut out = Robots::default();
    let mut in_agents = false;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once(':') else { continue };
        let (key, value) = (key.trim().to_ascii_lowercase(), value.trim());
        match key.as_str() {
            "user-agent" => {
                if !in_agents {
                    out.groups.push(RobotsGroup { agents: vec![], rules: vec![] });
                }
                in_agents = true;
                out.groups.last_mut().unwrap().agents.push(value.to_ascii_lowercase());
            }
            "allow" | "disallow" => {
                in_agents = false;
                if let Some(g) = out.groups.last_mut() {
                    g.rules.push((key, value.to_string()));
                }
            }
            "sitemap" => out.sitemaps.push(value.to_string()),
            "agentmap" => out.agentmaps.push(value.to_string()),
            "content-signal" => {
                in_agents = false;
                let pick = |name: &str| {
                    value.split(',').find_map(|p| {
                        let (k, v) = p.split_once('=')?;
                        (k.trim().eq_ignore_ascii_case(name)).then(|| v.trim().to_string())
                    })
                };
                let (train, search, input) = (pick("ai-train"), pick("search"), pick("ai-input"));
                if train.is_some() || search.is_some() || input.is_some() {
                    let agent = out.groups.last().and_then(|g| g.agents.first()).cloned();
                    out.signals.push(json!({"userAgent": agent, "aiTrain": train, "search": search, "aiInput": input}));
                }
            }
            _ => in_agents = false,
        }
    }
    out
}

const AI_BOTS: [&str; 20] = [
    "gptbot", "chatgpt-user", "oai-searchbot", "google-extended", "ccbot", "anthropic-ai", "claude-web", "bytespider",
    "perplexitybot", "cohere-ai", "applebot-extended", "amazonbot", "meta-externalagent", "facebookbot", "omgilibot",
    "diffbot", "claudebot", "claude-searchbot", "claude-user", "perplexity-user",
];

/// RFC 9309 precedence: the longest matching pattern wins, and allow wins ties.
fn access_for(rules: &[&(String, String)], path: &str) -> &'static str {
    let mut best: Option<(usize, bool)> = None;
    for (directive, pattern) in rules.iter().map(|r| (&r.0, &r.1)) {
        if pattern.is_empty() {
            continue;
        }
        let prefix = pattern.split('*').next().unwrap_or("").trim_end_matches('$');
        if !path.starts_with(prefix) {
            continue;
        }
        let allow = directive == "allow";
        let better = match best {
            None => true,
            Some((len, was_allow)) => pattern.len() > len || (pattern.len() == len && allow && !was_allow),
        };
        if better {
            best = Some((pattern.len(), allow));
        }
    }
    if matches!(best, Some((_, false))) { "blocked" } else { "allowed" }
}

pub fn robots_txt(c: &Ctx) -> Out {
    let mut t = Trace::default();
    t.fetch(&c.url("/robots.txt"), &c.robots);
    if !c.robots.ok() || c.robots.is_html() {
        return t.fail("robots.txt not found");
    }
    if parse_robots(&c.robots.text()).groups.is_empty() {
        return t.fail("robots.txt has no User-agent directive");
    }
    t.pass("robots.txt exists with valid format", Value::Null)
}

fn is_sitemap(url: &str, r: &Resp) -> bool {
    if !r.ok() {
        return false;
    }
    if url.ends_with(".gz") || r.body.starts_with(&[0x1f, 0x8b]) {
        return true;
    }
    let text = r.text();
    text.contains("<urlset") || text.contains("<sitemapindex") || text.contains(":urlset") || text.contains(":sitemapindex")
}

pub fn sitemap(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let declared = if c.robots.ok() { parse_robots(&c.robots.text()).sitemaps } else { vec![] };
    t.note(format!("{} Sitemap directive(s) in robots.txt", declared.len()));
    for raw in &declared {
        let url = if raw.starts_with("http") { raw.clone() } else { c.url(raw) };
        let r = c.get(&mut t, &url, None);
        if is_sitemap(&url, &r) {
            return t.pass("Sitemap found", json!({"url": url, "fromRobotsTxt": true}));
        }
    }
    for path in ["/sitemap.xml", "/sitemap-index.xml", "/sitemap_index.xml", "/sitemap.xml.gz", "/wp-sitemap.xml"] {
        let url = c.url(path);
        let r = c.get(&mut t, &url, None);
        if is_sitemap(&url, &r) {
            return t.pass("Sitemap found", json!({"url": url, "fromRobotsTxt": false}));
        }
    }
    t.fail("Sitemap not found")
}

// ---------- headers ----------

/// Splits a Link header into (href, rel values, type).
pub fn parse_link_header(value: &str) -> Vec<(String, Vec<String>, Option<String>)> {
    let mut parts = Vec::new();
    let (mut cur, mut in_uri, mut in_quote) = (String::new(), false, false);
    for ch in value.chars() {
        match ch {
            '<' if !in_quote => in_uri = true,
            '>' if !in_quote => in_uri = false,
            '"' if !in_uri => in_quote = !in_quote,
            ',' if !in_uri && !in_quote => {
                parts.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    parts.push(cur);

    let mut out = Vec::new();
    for part in parts {
        let mut pieces = part.split(';');
        let Some(uri) = pieces.next().map(str::trim) else { continue };
        let Some(href) = uri.strip_prefix('<').and_then(|u| u.strip_suffix('>')) else { continue };
        let (mut rels, mut kind) = (Vec::new(), None);
        for attr in pieces {
            let Some((k, v)) = attr.split_once('=') else { continue };
            let v = v.trim().trim_matches('"');
            match k.trim().to_ascii_lowercase().as_str() {
                "rel" => rels = v.split_whitespace().map(|s| s.to_ascii_lowercase()).collect(),
                "type" => kind = Some(v.to_string()),
                _ => {}
            }
        }
        out.push((href.to_string(), rels, kind));
    }
    out
}

pub fn link_headers(c: &Ctx) -> Out {
    const USEFUL: [&str; 4] = ["service-desc", "service-doc", "describedby", "api-catalog"];
    let mut t = Trace::default();
    t.fetch(&c.target_url, &c.home);
    let values = c.home.headers_all("link");
    if values.is_empty() {
        return t.fail("No Link headers found on target page");
    }
    let links: Vec<_> = values.iter().flat_map(|v| parse_link_header(v)).collect();
    let mut found = Vec::new();
    for (href, rels, kind) in &links {
        for rel in rels {
            // An alternate only helps an agent when it is machine-readable,
            // which rules out the usual per-language alternates.
            let useful = USEFUL.contains(&rel.as_str())
                || (rel == "alternate" && kind.as_deref().is_some_and(|k| k.contains("markdown") || k.contains("json")));
            if useful {
                found.push(json!({"rel": rel, "href": href, "type": kind}));
            }
        }
    }
    if found.is_empty() {
        return t.fail(format!("Found {} Link header(s), none with an agent-useful relation", links.len()));
    }
    let names: Vec<&str> = found.iter().filter_map(|f| f["rel"].as_str()).collect();
    t.pass(format!("Found agent-useful Link relations: {}", names.join(", ")), json!({"relationsFound": found, "totalLinks": links.len()}))
}

/// SVCB priority: 0 means AliasMode. Resolvers answer either in presentation
/// form ("1 . alpn=h2") or in RFC 3597 hex ("\# 12 00 01 ...").
fn svcb_priority(data: &str) -> Option<u32> {
    let data = data.trim();
    if let Some(rest) = data.strip_prefix("\\#") {
        let mut tokens = rest.split_whitespace().skip(1);
        let joined: String = tokens.by_ref().collect();
        return u32::from_str_radix(joined.get(0..4)?, 16).ok();
    }
    data.split_whitespace().next()?.parse().ok()
}

pub fn dns_aid(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let (mut service, mut unsigned, mut answered) = (Vec::new(), 0, false);
    for domain in c.domains() {
        for label in ["_index", "_a2a", "_mcp"] {
            let name = format!("{label}._agents.{domain}");
            for (kind, code) in [("SVCB", 64), ("HTTPS", 65)] {
                let Some((answers, ad)) = c.doh(&mut t, &name, kind) else { continue };
                answered = true;
                for a in answers.iter().filter(|a| a["type"] == code) {
                    let data = a["data"].as_str().unwrap_or("");
                    if svcb_priority(data).is_some_and(|p| p > 0) {
                        if ad {
                            service.push(json!({"name": name, "type": kind, "data": data}));
                        } else {
                            unsigned += 1;
                        }
                    }
                }
            }
        }
    }
    if !answered {
        return t.unable("No DNS-over-HTTPS resolver could be reached");
    }
    if !service.is_empty() {
        return t.pass(format!("Found {} DNSSEC-validated DNS-AID record(s)", service.len()), json!({"records": service, "dnssecValidated": true}));
    }
    if unsigned > 0 {
        return t.fail(format!("Found {unsigned} DNS-AID record(s), but DNSSEC did not validate them"));
    }
    t.fail("DNS-AID records not found")
}

fn looks_like_html(body: &str) -> bool {
    let start: String = body.trim_start().chars().take(15).collect::<String>().to_ascii_lowercase();
    start.starts_with("<!doctype") || start.starts_with("<html")
}

pub fn markdown_negotiation(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get(&mut t, &c.target_url, Some("text/markdown"));
    if r.error.is_some() {
        return t.unable("The markdown request could not complete");
    }
    if !(200..300).contains(&r.status) {
        return t.fail(format!("Site returned status {} for a markdown request", r.status));
    }
    let ct = r.content_type();
    if ct.contains("text/markdown") {
        if looks_like_html(&r.text()) {
            return t.fail("Site advertises text/markdown but returned an HTML body");
        }
        return t.pass("Site supports Markdown for Agents", json!({"contentType": ct}));
    }
    t.fail("Site does not support Markdown for Agents")
}

pub fn ai_rules(c: &Ctx) -> Out {
    let mut t = Trace::default();
    t.fetch(&c.url("/robots.txt"), &c.robots);
    if !c.robots_available() {
        return t.fail("Cannot check AI rules without robots.txt");
    }
    let robots = parse_robots(&c.robots.text());
    let rules_of = |agent: &str| -> Vec<&(String, String)> {
        robots.groups.iter().filter(|g| g.agents.iter().any(|a| a == agent)).flat_map(|g| g.rules.iter()).collect()
    };
    // A `User-agent: *` with nothing under it does not state any rule.
    let wildcard = robots.groups.iter().any(|g| !g.rules.is_empty() && g.agents.iter().any(|a| a == "*"));
    let explicit: Vec<&str> = AI_BOTS.iter().copied().filter(|b| robots.groups.iter().any(|g| g.agents.iter().any(|a| a == b))).collect();
    let policies: Vec<Value> = AI_BOTS
        .iter()
        .map(|bot| {
            let own = rules_of(bot);
            let is_explicit = robots.groups.iter().any(|g| g.agents.iter().any(|a| a == bot));
            let rules = if is_explicit { own } else { rules_of("*") };
            json!({
                "userAgent": bot,
                "ruleSource": if is_explicit { "explicit" } else if wildcard { "wildcard" } else { "none" },
                "effectiveAccess": access_for(&rules, &c.target_path),
            })
        })
        .collect();
    let details = json!({"targetPath": c.target_path, "botPolicies": policies});
    if !explicit.is_empty() {
        return t.pass(format!("Found rules for AI bots: {}", explicit.join(", ")), details);
    }
    if wildcard {
        return t.pass("No AI-specific bot rules; wildcard rules apply to all crawlers including AI bots", details);
    }
    t.fail("robots.txt has no rules that apply to AI bots")
}

pub fn content_signals(c: &Ctx) -> Out {
    let mut t = Trace::default();
    t.fetch(&c.url("/robots.txt"), &c.robots);
    if !c.robots_available() {
        return t.fail("Cannot check Content Signals without robots.txt");
    }
    let signals = parse_robots(&c.robots.text()).signals;
    if signals.is_empty() {
        return t.fail("No Content Signals found in robots.txt");
    }
    t.pass("Content Signals found in robots.txt", json!({"signalCount": signals.len(), "signals": signals}))
}

pub fn web_bot_auth(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/http-message-signatures-directory", None);
    if r.json().is_some_and(|v| v["keys"].as_array().is_some_and(|k| !k.is_empty())) {
        return t.pass("Web Bot Auth directory found", Value::Null);
    }
    t.neutral("Web Bot Auth directory not found (informational only)")
}

// ---------- discovery ----------

/// A flat name alone is not a card: plenty of JSON has a name. It has to come
/// with a version, or with the remotes or packages of the registry format.
fn card_name(v: &Value) -> Option<String> {
    if let Some(name) = v["serverInfo"]["name"].as_str().filter(|s| !s.is_empty()) {
        return Some(name.to_string());
    }
    let looks_like_card = ["version", "remotes", "packages"].iter().any(|k| v.get(*k).is_some());
    v["name"].as_str().filter(|s| !s.is_empty() && looks_like_card).map(String::from)
}

pub fn mcp_server_card(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let mut invalid = None;
    for path in ["/.well-known/mcp/server-card.json", "/.well-known/mcp/server-cards.json", "/.well-known/mcp.json"] {
        let r = c.get_path(&mut t, path, None);
        let Some(v) = r.json() else { continue };
        // server-cards.json holds several cards, as a list or under a key.
        let cards: Vec<&Value> = if let Some(a) = v.as_array() {
            a.iter().collect()
        } else {
            let listed = ["cards", "serverCards", "servers"].iter().find_map(|k| v[*k].as_array());
            match listed {
                Some(a) => a.iter().collect(),
                None => vec![&v],
            }
        };
        if let Some((card, name)) = cards.iter().find_map(|x| card_name(x).map(|n| (*x, n))) {
            let registry = card.get("remotes").is_some() || card.get("packages").is_some();
            return t.pass(
                format!("MCP Server Card found at {path}"),
                json!({"path": path, "format": if registry { "registry" } else { "sep" }, "name": name}),
            );
        }
        invalid.get_or_insert(format!("{path} returned JSON but has no serverInfo.name or name"));
    }
    t.fail(invalid.unwrap_or_else(|| "MCP Server Card not found".into()))
}

pub fn a2a_agent_card(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/agent-card.json", None);
    let Some(v) = r.json() else { return t.fail("A2A Agent Card not found") };
    let interfaces = v["supportedInterfaces"].as_array().is_some_and(|a| !a.is_empty());
    if v["name"].is_string() && v["version"].is_string() && interfaces {
        return t.pass("A2A Agent Card found", json!({"name": v["name"]}));
    }
    t.fail("A2A Agent Card is missing name, version or supportedInterfaces")
}

pub fn agent_skills(c: &Ctx) -> Out {
    let mut t = Trace::default();
    for path in ["/.well-known/agent-skills/index.json", "/.well-known/skills/index.json"] {
        let r = c.get_path(&mut t, path, None);
        let Some(v) = r.json() else { continue };
        let Some(skills) = v["skills"].as_array() else {
            return t.fail(format!("{path} has no skills array"));
        };
        if skills.is_empty() {
            return t.fail(format!("{path} lists no skills"));
        }
        return t.pass("Agent Skills index exists with valid JSON", json!({"path": path, "skillCount": skills.len(), "hasSchema": v.get("$schema").is_some()}));
    }
    t.fail("Agent Skills index not found")
}

pub fn api_catalog(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/api-catalog", Some("application/linkset+json, application/json"));
    let Some(v) = r.json() else { return t.fail("API Catalog not found") };
    let Some(set) = v["linkset"].as_array().filter(|a| !a.is_empty()) else {
        return t.fail("API Catalog has no entries in its linkset array");
    };
    let right_type = r.content_type().contains("application/linkset+json");
    t.pass(format!("API Catalog found ({} in linkset)", set.len()), json!({"apiCount": set.len(), "correctContentType": right_type}))
}

fn valid_oauth_metadata(v: &Value) -> bool {
    v["issuer"].is_string() && (v["authorization_endpoint"].is_string() || v["token_endpoint"].is_string())
}

pub fn oauth_discovery(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let paths = [
        ("/.well-known/openid-configuration", "OpenID Connect discovery metadata found"),
        ("/.well-known/oauth-authorization-server", "OAuth 2.0 (RFC 8414) discovery metadata found"),
    ];
    let mut hit = None;
    for (path, msg) in paths {
        let r = c.get_path(&mut t, path, None);
        if let Some(v) = r.json().filter(valid_oauth_metadata) {
            hit.get_or_insert((path, msg, v));
        }
    }
    match hit {
        Some((path, msg, v)) => t.pass(msg, json!({"source": path, "issuer": v["issuer"]})),
        None => t.fail("No OAuth/OIDC discovery metadata found"),
    }
}

/// Only `resource` is required by RFC 9728; `authorization_servers` is optional.
fn valid_prm(v: &Value) -> bool {
    v["resource"].as_str().is_some_and(|r| !r.is_empty())
}

pub fn oauth_protected_resource(c: &Ctx) -> Out {
    let mut t = Trace::default();
    t.fetch(&c.target_url, &c.home);
    let mut urls = Vec::new();
    // RFC 9728 lets a 401 point straight to the metadata.
    let re = Regex::new(r#"resource_metadata="([^"]+)""#).unwrap();
    if let Some(m) = c.home.header("www-authenticate").and_then(|h| re.captures(h)) {
        urls.push((m[1].to_string(), "www-authenticate"));
    }
    if c.target_path != "/" {
        urls.push((c.url(&format!("/.well-known/oauth-protected-resource{}", c.target_path)), "well-known"));
    }
    urls.push((c.url("/.well-known/oauth-protected-resource"), "well-known"));
    for (url, method) in urls {
        let r = c.get(&mut t, &url, None);
        if let Some(v) = r.json().filter(valid_prm) {
            return t.pass(
                format!("OAuth Protected Resource Metadata found ({method})"),
                json!({"discoveryMethod": method, "resource": v["resource"], "authorizationServers": v["authorization_servers"]}),
            );
        }
    }
    t.fail("No OAuth Protected Resource Metadata found")
}

pub fn auth_md(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/auth.md", Some("text/markdown, text/plain, */*"));
    if !r.ok() {
        return t.fail("auth.md not found");
    }
    if r.is_html() {
        return t.fail("auth.md returned HTML instead of Markdown");
    }
    let body = r.text();
    let heading = Regex::new(r"(?mi)^#[ \t]+.*auth\.md").unwrap();
    if !heading.is_match(&body) {
        return t.fail("auth.md exists but is missing the expected Auth.md heading");
    }

    let lower = body.to_ascii_lowercase();
    let methods: Vec<&str> = [("anonymous", "anonymous"), ("id-jag", "id-jag"), ("verified_email", "verified_email")]
        .iter()
        .filter(|(needle, _)| lower.contains(needle))
        .map(|(_, name)| *name)
        .collect();

    // With OAuth metadata, the agent_auth block is the source of truth.
    let prm = c.get_path(&mut t, "/.well-known/oauth-protected-resource", Some("application/json")).json();
    let mut servers: Vec<String> = prm
        .as_ref()
        .and_then(|p| p["authorization_servers"].as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default();
    if servers.is_empty() {
        servers.push(c.origin.clone());
    }
    for server in servers {
        let url = format!("{}/.well-known/oauth-authorization-server", server.trim_end_matches('/'));
        let meta = c.get(&mut t, &url, Some("application/json")).json();
        if meta.is_some_and(|m| m["agent_auth"]["register_uri"].is_string()) {
            return t.pass("Auth.md support detected (agent_auth metadata)", json!({"source": "agent_auth", "methods": methods}));
        }
    }

    // Without it, auth.md has to explain registration by itself.
    let endpoint = Regex::new(r"(?i)\b(?:POST|PUT)\b[\s`]+((?:https?://|/)[^\s`)]+)").unwrap();
    let mut endpoints: Vec<&str> = endpoint.captures_iter(&body).map(|m| m.get(1).unwrap().as_str()).collect();
    endpoints.sort_unstable();
    endpoints.dedup();
    let credentials = ["api_key", "api key", "access_token", "bearer", "credential"].iter().any(|m| lower.contains(m));
    if !endpoints.is_empty() && credentials {
        return t.pass("Auth.md support detected (self-contained)", json!({"source": "auth.md", "registrationEndpoints": endpoints, "methods": methods}));
    }
    t.fail("auth.md does not describe a registration endpoint and its credentials")
}

fn ard_invalid_entry(e: &Value) -> Option<&'static str> {
    let filled = |k: &str| e[k].as_str().is_some_and(|s| !s.is_empty());
    if !filled("identifier") {
        return Some("identifier");
    }
    if !filled("displayName") {
        return Some("displayName");
    }
    if !filled("type") {
        return Some("type");
    }
    if e.get("url").is_some() == e.get("data").is_some() {
        return Some("exactly one of url or data");
    }
    None
}

pub fn ard(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let mut urls: Vec<(String, &str)> = Vec::new();
    if c.robots.ok() {
        urls.extend(parse_robots(&c.robots.text()).agentmaps.into_iter().map(|u| (u, "robots")));
    }
    let tag = Regex::new(r"(?i)<link\b[^>]*>").unwrap();
    let rel = Regex::new(r#"(?i)\brel=["']?ai-catalog\b"#).unwrap();
    let href = Regex::new(r#"(?i)\bhref=["']([^"']+)["']"#).unwrap();
    let html = c.home.text();
    for m in tag.find_iter(&html).filter(|m| rel.is_match(m.as_str())) {
        if let Some(h) = href.captures(m.as_str()) {
            let u = h[1].to_string();
            urls.push((if u.starts_with("http") { u } else { c.url(&u) }, "html"));
        }
    }
    urls.push((c.url("/.well-known/ai-catalog.json"), "well-known"));
    for domain in c.domains() {
        if let Some((answers, _)) = c.doh(&mut t, &format!("_catalog._agents.{domain}"), "TXT") {
            for a in answers {
                let data = a["data"].as_str().unwrap_or("").trim_matches('"').to_string();
                if let Some(u) = data.split_whitespace().find_map(|p| p.strip_prefix("url=")) {
                    urls.push((u.to_string(), "dns"));
                }
            }
        }
    }

    let mut problem = None;
    for (url, mechanism) in urls {
        let r = c.get(&mut t, &url, Some("application/json"));
        let Some(v) = r.json() else { continue };
        if !v["specVersion"].as_str().is_some_and(|s| !s.is_empty()) {
            problem.get_or_insert("ARD manifest has no specVersion".to_string());
            continue;
        }
        let entries = v["entries"].as_array().cloned().unwrap_or_default();
        let valid = entries.iter().filter(|e| ard_invalid_entry(e).is_none()).count();
        if valid > 0 {
            return t.pass(format!("ARD capability manifest found with {valid} agentic resources"), json!({"url": url, "mechanism": mechanism, "validEntries": valid, "entries": entries.len()}));
        }
        let why = entries.first().and_then(ard_invalid_entry).map(|m| format!(" (entry 0 is missing {m})")).unwrap_or_default();
        problem.get_or_insert(format!("ARD capability manifest has no valid entries{why}"));
    }
    t.fail(problem.unwrap_or_else(|| "ARD capability manifest not found".into()))
}

pub fn personal_agent_protocol(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/poppy.json", Some("application/json"));
    let Some(v) = r.json() else { return t.fail("Personal Agent Protocol manifest not found") };
    let bare = |s: &str| s.trim_start_matches("www.").to_ascii_lowercase();
    let host = c.host.split(':').next().unwrap_or("");
    if !v["organization"]["domain"].as_str().is_some_and(|d| bare(d) == bare(host)) {
        return t.fail("poppy.json organization.domain does not match this host");
    }
    let has_web = v.get("web").is_some();
    let session = v["agent"].is_object() || v["apis"].is_array() || v["web"]["browser_session_endpoint"].is_string();
    if !has_web && !session {
        return t.fail("poppy.json declares no interfaces");
    }
    if session {
        let Some(issuer) = v["auth"]["issuer"].as_str() else {
            return t.fail("poppy.json declares session interfaces without auth.issuer");
        };
        let url = format!("{}/.well-known/oauth-authorization-server", issuer.trim_end_matches('/'));
        let meta = c.get(&mut t, &url, Some("application/json")).json();
        let matches = meta.is_some_and(|m| {
            m["issuer"] == issuer
                && m["poppy_domains"].as_array().is_some_and(|d| d.iter().any(|x| x.as_str().is_some_and(|s| bare(s) == bare(host))))
        });
        if !matches {
            return t.fail("The auth.issuer metadata does not match or lacks poppy_domains");
        }
    }
    t.pass("Personal Agent Protocol manifest found", json!({"manifestDetected": true}))
}

pub fn web_mcp(c: &Ctx) -> Out {
    let mut t = Trace::default();
    match browser::detect(&c.target_url) {
        browser::Outcome::NoBrowser => {
            let form = Regex::new(r"(?i)<form\b[^>]*\btoolname\s*=").unwrap();
            if form.is_match(&c.home.text()) {
                return t.pass("Found declarative WebMCP tools in the HTML", json!({"detectionMethod": "declarative_html"}));
            }
            t.unable("No Chromium-based browser found; WebMCP needs one to run the page")
        }
        browser::Outcome::Failed(e) => {
            t.note(e);
            t.unable("The browser could not check the page")
        }
        browser::Outcome::Done(found) => {
            let method = match (found.imperative.is_empty(), found.declarative.is_empty()) {
                (false, false) => "both",
                (false, true) => "imperative_api",
                (true, false) => "declarative_html",
                (true, true) => {
                    if !found.loaded {
                        return t.unable("Browser session timed out");
                    }
                    return t.fail("No WebMCP tools detected on page load");
                }
            };
            let tools: Vec<Value> = found.imperative.into_iter().chain(found.declarative).collect();
            let names: Vec<&str> = tools.iter().filter_map(|x| x["name"].as_str()).collect();
            t.pass(
                format!("Found {} WebMCP tool(s): {}", tools.len(), names.join(", ")),
                json!({"toolCount": tools.len(), "tools": tools, "detectionMethod": method}),
            )
        }
    }
}

// ---------- commerce ----------

/// How much of the homepage counts. Further down, a page mentioning a brand is
/// usually talking about it (a comparison, a footer), not selling with it.
const COMMERCE_WINDOW: usize = 100_000;
const COMMERCE_MIN_SIGNALS: usize = 3;
const MANY_PRICES: usize = 3;

/// Signals of a store in the homepage. It takes several to call it one, since a
/// lone signal is common elsewhere: a blog can mention Apple Pay, and a company
/// can be named BigCommerce without selling anything on its homepage.
pub fn commerce_signals(html: &str) -> (bool, Vec<String>) {
    let h: String = html.chars().take(COMMERCE_WINDOW).collect::<String>().to_ascii_lowercase();
    let mut out: Vec<String> = Vec::new();
    for p in ["shopify", "woocommerce", "magento", "bigcommerce", "prestashop"] {
        if h.contains(p) {
            out.push(format!("platform:{p}"));
        }
    }
    for (name, needle) in [("stripe", "stripe"), ("paypal", "paypal"), ("apple-pay", "apple-pay"), ("google-pay", "google-pay"), ("klarna", "klarna")] {
        if h.contains(needle) {
            out.push(format!("payment:{name}"));
        }
    }
    if h.contains("shopify-") {
        out.push("meta:shopify".into());
    }
    for (needle, name) in [("product", "Product"), ("offer", "Offer"), ("aggregateoffer", "AggregateOffer")] {
        let re = Regex::new(&format!(r#""@type"\s*:\s*"{needle}""#)).unwrap();
        if re.is_match(&h) {
            out.push(format!("schema:{name}"));
        }
    }
    for (name, pattern) in [("/product", "products?"), ("/shop", "shop"), ("/store", "store"), ("/buy", "buy"), ("/cart", "cart"), ("/checkout", "checkout")] {
        let re = Regex::new(&format!(r#"href=["'][^"']*/{pattern}[/"'?]"#)).unwrap();
        if re.is_match(&h) {
            out.push(format!("url:{name}"));
        }
    }
    let price = Regex::new(r"[$€£]\s?\d[\d,.]*").unwrap();
    let prices: std::collections::HashSet<&str> = price.find_iter(&h).map(|m| m.as_str()).collect();
    if prices.len() >= MANY_PRICES {
        out.push("prices:multiple".into());
    }
    (out.len() >= COMMERCE_MIN_SIGNALS, out)
}

pub fn x402(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let mut urls = vec![c.target_url.clone()];
    urls.push(c.url("/api"));
    urls.push(c.url("/api/v1"));
    for url in urls {
        let r = c.get(&mut t, &url, None);
        if r.status != 402 {
            continue;
        }
        let header = r.headers.iter().any(|(k, v)| {
            let k = k.to_ascii_lowercase();
            k.starts_with("x-payment") || k.starts_with("x402") || (k == "www-authenticate" && v.to_ascii_lowercase().contains("x402"))
        });
        let body = serde_json::from_slice::<Value>(&r.body).ok();
        let declared = body.is_some_and(|b| b.get("x402Version").is_some() || b["accepts"].is_array());
        if header || declared {
            return t.pass("x402 payment protocol detected", json!({"url": url}));
        }
    }
    t.fail("x402 payment protocol not detected")
}

pub fn mpp(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/openapi.json", None);
    let Some(v) = r.json() else { return t.fail("MPP payment discovery not detected") };
    let paid = v["paths"].as_object().is_some_and(|paths| {
        paths.values().filter_map(Value::as_object).any(|ops| ops.values().any(|op| op.get("x-payment-info").is_some()))
    });
    if paid {
        return t.pass("MPP payment discovery found", Value::Null);
    }
    t.fail("MPP payment discovery not detected")
}

pub fn ucp(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/ucp", None);
    let Some(v) = r.json() else { return t.fail("UCP profile not found") };
    let version = !v["ucp"]["version"].is_null() || !v["protocol_version"].is_null();
    if version {
        return t.pass("UCP profile exists", json!({"hasVersion": true, "hasServices": !v["ucp"]["services"].is_null() || !v["services"].is_null()}));
    }
    t.fail("UCP profile has no version")
}

pub fn acp(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/acp.json", None);
    let Some(v) = r.json() else { return t.fail("ACP discovery document not found") };
    let filled = |x: &Value| x.as_array().is_some_and(|a| !a.is_empty());
    let complete = v["protocol"]["name"] == "acp"
        && v["protocol"]["version"].is_string()
        && v["api_base_url"].as_str().is_some_and(|u| u.starts_with("http://") || u.starts_with("https://"))
        && filled(&v["transports"])
        && filled(&v["capabilities"]["services"]);
    if complete {
        return t.pass("ACP discovery document found", Value::Null);
    }
    t.fail("ACP discovery document is missing required fields")
}

pub fn ap2(c: &Ctx) -> Out {
    let mut t = Trace::default();
    let r = c.get_path(&mut t, "/.well-known/agent-card.json", None);
    let Some(v) = r.json() else { return t.fail("AP2 not detected (no A2A Agent Card)") };
    let found = v["capabilities"]["extensions"].as_array().is_some_and(|list| {
        list.iter().any(|e| e["uri"].as_str().is_some_and(|u| u.to_ascii_lowercase().contains("ap2")) && !e["params"]["roles"].is_null())
    });
    if found {
        return t.pass("AP2 extension found in the A2A Agent Card", Value::Null);
    }
    t.fail("AP2 not detected in the A2A Agent Card")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_keeps_groups_signals_and_maps() {
        let r = parse_robots("# c\nUser-agent: GPTBot\nUser-agent: *\nContent-Signal: search=yes, ai-train=no\nDisallow: /x\n\nSitemap: https://a/s.xml\nAgentmap: https://a/c.json\n");
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].agents, ["gptbot", "*"]);
        assert_eq!(r.sitemaps, ["https://a/s.xml"]);
        assert_eq!(r.agentmaps, ["https://a/c.json"]);
        assert_eq!(r.signals[0]["aiTrain"], "no");
    }

    #[test]
    fn robots_without_user_agent_has_no_groups() {
        assert!(parse_robots("Sitemap: https://a/s.xml\n").groups.is_empty());
    }

    #[test]
    fn longest_robots_rule_wins_and_allow_wins_ties() {
        let d = ("disallow".to_string(), "/".to_string());
        let a = ("allow".to_string(), "/pub".to_string());
        assert_eq!(access_for(&[&d, &a], "/pub/x"), "allowed");
        assert_eq!(access_for(&[&d, &a], "/other"), "blocked");
        assert_eq!(access_for(&[], "/"), "allowed");
    }

    #[test]
    fn link_header_survives_commas_in_uris_and_quotes() {
        let l = parse_link_header(r#"</a,b>; rel="service-desc describedby"; type="x/y", </c>; title="p, q"; rel=api-catalog"#);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].0, "/a,b");
        assert_eq!(l[0].1, ["service-desc", "describedby"]);
        assert_eq!(l[1].1, ["api-catalog"]);
    }

    #[test]
    fn svcb_priority_reads_both_forms() {
        assert_eq!(svcb_priority("1 . alpn=h2"), Some(1));
        assert_eq!(svcb_priority("0 target.example."), Some(0));
        assert_eq!(svcb_priority("\\# 12 00 01 00 00"), Some(1));
        assert_eq!(svcb_priority("\\# 4 00 00 00 00"), Some(0));
    }

    #[test]
    fn html_labelled_as_markdown_is_spotted() {
        assert!(looks_like_html("
 <!DOCTYPE html><title>x</title>"));
        assert!(looks_like_html("<HTML>"));
        assert!(!looks_like_html("# Title

<div>inline html is fine in markdown</div>"));
    }

    #[test]
    fn it_takes_three_signals_to_be_a_store() {
        assert!(!commerce_signals("we take apple-pay").0);
        assert!(!commerce_signals(r#"<script src="https://cdn.bigcommerce.com/x.js">"#).0);
        assert!(commerce_signals(r#"paypal klarna <a href="/cart/">"#).0);
    }

    #[test]
    fn what_a_long_page_says_far_down_does_not_count() {
        let far = format!("{}paypal klarna stripe", " ".repeat(COMMERCE_WINDOW));
        assert!(commerce_signals(&far).1.is_empty());
    }

    #[test]
    fn many_different_prices_are_a_signal() {
        assert!(commerce_signals("$1 $2 $3").1.contains(&"prices:multiple".to_string()));
        assert!(!commerce_signals("$1 $1 $1 $1").1.contains(&"prices:multiple".to_string()));
    }

    #[test]
    fn a_flat_name_needs_something_else_to_be_a_card() {
        assert_eq!(card_name(&json!({"serverInfo": {"name": "a"}})).as_deref(), Some("a"));
        assert_eq!(card_name(&json!({"name": "a", "remotes": []})).as_deref(), Some("a"));
        assert_eq!(card_name(&json!({"name": "a", "version": "1"})).as_deref(), Some("a"));
        assert_eq!(card_name(&json!({"name": "a", "description": "d", "endpoint": "e"})), None);
        assert_eq!(card_name(&json!({"name": "a", "transport": "http", "mcp_endpoint": "e"})), None);
    }

    #[test]
    fn ard_entry_needs_exactly_one_of_url_or_data() {
        let base = json!({"identifier": "i", "displayName": "d", "type": "t"});
        assert!(ard_invalid_entry(&base).is_some());
        let mut with_url = base.clone();
        with_url["url"] = json!("u");
        assert!(ard_invalid_entry(&with_url).is_none());
        with_url["data"] = json!({});
        assert!(ard_invalid_entry(&with_url).is_some());
    }
}
