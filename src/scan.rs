//! Runs the checks, works out the level and the score, and builds the report.

use crate::checks::{self as ck, Ctx, Out, Status};
use crate::docs;
use crate::http::Http;
use crate::meta::{self, CATEGORIES, CHECKS, INFORMATIONAL, OPT_IN};
use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const LEVEL_NAMES: [&str; 6] = ["Not Ready", "Basic Web Presence", "Bot-Aware", "Agent-Readable", "Agent-Integrated", "Agent-Native"];
const INTEGRATION: [&str; 4] = ["mcpServerCard", "a2aAgentCard", "agentSkills", "apiCatalog"];
const AUTH: [&str; 3] = ["oauthDiscovery", "oauthProtectedResource", "authMd"];
const BASICS: [&str; 3] = ["robotsTxt", "sitemap", "linkHeaders"];

type CheckFn = fn(&Ctx) -> Out;

fn runner(key: &str) -> CheckFn {
    match key {
        "robotsTxt" => ck::robots_txt,
        "sitemap" => ck::sitemap,
        "linkHeaders" => ck::link_headers,
        "dnsAid" => ck::dns_aid,
        "markdownNegotiation" => ck::markdown_negotiation,
        "robotsTxtAiRules" => ck::ai_rules,
        "contentSignals" => ck::content_signals,
        "webBotAuth" => ck::web_bot_auth,
        "apiCatalog" => ck::api_catalog,
        "oauthDiscovery" => ck::oauth_discovery,
        "oauthProtectedResource" => ck::oauth_protected_resource,
        "authMd" => ck::auth_md,
        "mcpServerCard" => ck::mcp_server_card,
        "a2aAgentCard" => ck::a2a_agent_card,
        "agentSkills" => ck::agent_skills,
        "webMcp" => ck::web_mcp,
        "ard" => ck::ard,
        "personalAgentProtocol" => ck::personal_agent_protocol,
        "x402" => ck::x402,
        "mpp" => ck::mpp,
        "ucp" => ck::ucp,
        "acp" => ck::acp,
        "ap2" => ck::ap2,
        other => unreachable!("unknown check {other}"),
    }
}

/// Splits a URL into origin and path + query. Only http and https get through.
fn split_url(url: &str) -> Option<(String, String, String)> {
    let re = Regex::new(r"(?i)^(https?)://([^/?#]+)([^#]*)").unwrap();
    let m = re.captures(url)?;
    let authority = m[2].to_string();
    if authority.contains('@') {
        return None;
    }
    let path = if m[3].is_empty() { "/".to_string() } else { m[3].to_string() };
    Some((format!("{}://{}", m[1].to_ascii_lowercase(), authority), authority, path))
}

pub fn normalize(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Missing url".into());
    }
    let url = if input.contains("://") { input.to_string() } else { format!("https://{input}") };
    if split_url(&url).is_none() {
        return Err(format!("Not a valid http or https URL: {input}"));
    }
    Ok(url)
}

/// Resolves a profile name or a list of check keys to the checks to run.
pub fn select(profile: Option<&str>, enabled: Option<Vec<String>>) -> Result<(Vec<&'static str>, HashSet<String>), String> {
    if let Some(list) = enabled {
        let unknown: Vec<&String> = list.iter().filter(|k| !CHECKS.iter().any(|m| m.key == k.as_str())).collect();
        if !unknown.is_empty() {
            return Err(format!("Unknown checks: {unknown:?}"));
        }
        if list.is_empty() {
            return Err("Select at least one check".into());
        }
        let keys = CHECKS.iter().map(|m| m.key).filter(|k| list.iter().any(|l| l == k)).collect();
        let asked = list.into_iter().collect();
        return Ok((keys, asked));
    }
    let keys: Vec<&'static str> = match profile.unwrap_or("all") {
        "all" => CHECKS.iter().map(|m| m.key).collect(),
        "content" => meta::PROFILE_CONTENT.to_vec(),
        "apiApp" => meta::PROFILE_API.to_vec(),
        other => return Err(format!("Unknown profile: {other} (use all, content or apiApp)")),
    };
    Ok((keys, HashSet::new()))
}

fn iso_now() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // Civil-from-days (Howard Hinnant).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z", rem / 3600, rem % 3600 / 60, rem % 60, d.subsec_millis())
}

pub fn level_of(pass: &dyn Fn(&str) -> bool) -> usize {
    let count = |keys: &[&str]| keys.iter().filter(|k| pass(k)).count();
    if count(&BASICS) < 2 {
        return 0;
    }
    if !(pass("robotsTxtAiRules") && pass("contentSignals")) {
        return 1;
    }
    if !pass("markdownNegotiation") {
        return 2;
    }
    if count(&INTEGRATION) == 0 {
        return 3;
    }
    // The A2A Agent Card is opt-in, so it can open level 4 but is not demanded here.
    let full_integration = INTEGRATION.iter().filter(|k| !OPT_IN.contains(k)).all(|k| pass(k));
    let groups = [pass("webBotAuth"), full_integration, count(&AUTH) > 0];
    if groups.iter().filter(|g| **g).count() < 2 {
        return 4;
    }
    5
}

fn requirements_for(target: usize, pass: &dyn Fn(&str) -> bool) -> Vec<Value> {
    let keys: Vec<&str> = match target {
        1 => BASICS.to_vec(),
        2 => vec!["robotsTxtAiRules", "contentSignals"],
        3 => vec!["markdownNegotiation"],
        4 => INTEGRATION.to_vec(),
        _ => INTEGRATION.iter().chain(AUTH.iter()).copied().chain(["webBotAuth"]).collect(),
    };
    keys.into_iter()
        .filter(|k| !pass(k))
        .map(|k| {
            let m = meta::meta(k);
            json!({"check": k, "title": m.title, "fix": m.fix, "specUrls": m.specs})
        })
        .collect()
}

/// The percentage: passed over applicable, without commerce or informational checks.
fn score(results: &HashMap<&str, Out>, asked: &HashSet<String>) -> Value {
    let mut cats = Map::new();
    let (mut all_passed, mut all_total) = (0, 0);
    for (cat, name) in CATEGORIES {
        let (mut passed, mut total) = (0, 0);
        for m in CHECKS.iter().filter(|m| m.category == cat) {
            let Some(r) = results.get(m.key) else { continue };
            if OPT_IN.contains(&m.key) && !asked.contains(m.key) {
                continue;
            }
            if INFORMATIONAL.contains(&m.key) || r.status == Status::Neutral {
                continue;
            }
            total += 1;
            if r.status == Status::Pass {
                passed += 1;
            }
        }
        if cat != "commerce" {
            all_passed += passed;
            all_total += total;
        }
        let pct = if total > 0 { (passed as f64 / total as f64 * 100.0).round() as u32 } else { 0 };
        cats.insert(cat.into(), json!({"name": name, "score": pct, "passed": passed, "total": total}));
    }
    let overall = if all_total > 0 { (all_passed as f64 / all_total as f64 * 100.0).round() as u32 } else { 0 };
    json!({"overall": overall, "categories": cats})
}

pub fn scan(input: &str, keys: &[&'static str], asked: &HashSet<String>) -> Result<Value, String> {
    let url = normalize(input)?;
    let http = Http::new();
    let home = http.get(&url, None);
    if let Some(e) = &home.error {
        return Err(format!("Could not reach {url}: {e}"));
    }
    let target_url = home.final_url.clone();
    let (origin, host, target_path) = split_url(&target_url).ok_or_else(|| format!("Bad final URL: {target_url}"))?;
    let (input_origin, _, _) = split_url(&url).unwrap_or_default_tuple();
    // A site that refuses the homepage (often a bot wall) would fail every check
    // for a reason that has nothing to do with them, so say that instead.
    if home.status >= 400 {
        let preview: String = home.text().chars().take(300).collect();
        let mut report = json!({
            "url": origin,
            "targetUrl": target_url,
            "scannedAt": iso_now(),
            "siteError": {"httpStatus": home.status, "bodyPreview": preview},
        });
        if input_origin != origin {
            report["redirectedFrom"] = json!(url);
        }
        return Ok(report);
    }
    let robots = http.get(&format!("{origin}/robots.txt"), None);
    let ctx = Ctx { http, origin: origin.clone(), host, target_url: target_url.clone(), target_path, home, robots };

    let (is_commerce, signals) = ck::commerce_signals(&ctx.home.text());

    let mut timed: Vec<(&str, Out, u128)> = std::thread::scope(|s| {
        let handles: Vec<_> = keys
            .iter()
            .map(|k| {
                let ctx = &ctx;
                let k = *k;
                s.spawn(move || {
                    let start = Instant::now();
                    let out = runner(k)(ctx);
                    (k, out, start.elapsed().as_millis())
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a check panicked")).collect()
    });

    // A store is scored on commerce; anywhere else those results only inform.
    for (k, out, _) in timed.iter_mut() {
        if meta::meta(k).category == "commerce" && !is_commerce && out.status != Status::Pass {
            out.status = Status::Neutral;
            out.message = format!("{} (not a commerce site)", out.message);
        }
    }

    let mut durations = HashMap::new();
    let mut results: HashMap<&str, Out> = HashMap::new();
    for (k, out, ms) in timed {
        durations.insert(k, ms);
        results.insert(k, out);
    }
    let pass = |k: &str| results.get(k).is_some_and(|r| r.status == Status::Pass);
    let level = level_of(&pass);
    let next = if level < 5 {
        let target = level + 1;
        json!({"target": target, "name": LEVEL_NAMES[target], "requirements": requirements_for(target, &pass)})
    } else {
        Value::Null
    };

    let mut checks = Map::new();
    for (cat, _) in CATEGORIES {
        let mut group = Map::new();
        for m in CHECKS.iter().filter(|m| m.category == cat) {
            let Some(r) = results.get(m.key) else { continue };
            let mut entry = json!({"status": r.status.as_str(), "message": r.message});
            if !r.details.is_null() {
                entry["details"] = r.details.clone();
            }
            if r.status != Status::Pass {
                entry["fix"] = json!(m.fix);
                entry["guide"] = json!(docs::skill_name(m.key));
            }
            entry["evidence"] = json!(r.evidence);
            entry["durationMs"] = json!(durations[m.key]);
            group.insert(m.key.into(), entry);
        }
        if !group.is_empty() {
            checks.insert(cat.into(), Value::Object(group));
        }
    }

    let mut report = json!({
        "url": origin,
        "targetUrl": target_url,
        "scannedAt": iso_now(),
        "level": level,
        "levelName": LEVEL_NAMES[level],
    });
    if input_origin != origin {
        report["redirectedFrom"] = json!(url);
    }
    report["score"] = score(&results, asked);
    report["checks"] = Value::Object(checks);
    report["nextLevel"] = next;
    report["isCommerce"] = json!(is_commerce);
    report["commerceSignals"] = json!(signals);
    Ok(report)
}

trait DefaultTuple {
    fn unwrap_or_default_tuple(self) -> (String, String, String);
}

impl DefaultTuple for Option<(String, String, String)> {
    fn unwrap_or_default_tuple(self) -> (String, String, String) {
        self.unwrap_or_else(|| (String::new(), String::new(), String::new()))
    }
}

/// The report as Markdown for a coding agent: only what is left to fix.
pub fn agent_markdown(report: &Value) -> String {
    if let Some(status) = report["siteError"]["httpStatus"].as_u64() {
        return format!(
            "# Site analysis: {}\n\nThe homepage answered HTTP {status}, so the checks did not run. If the site blocks automated requests, allow them or scan a copy on localhost.\n",
            report["url"].as_str().unwrap_or("")
        );
    }
    let mut out = format!(
        "# Site analysis: {}\n\nLevel: {}/5 ({})\nScore: {}%\n\n",
        report["url"].as_str().unwrap_or(""),
        report["level"],
        report["levelName"].as_str().unwrap_or(""),
        report["score"]["overall"]
    );
    let mut any = false;
    for (cat, _) in CATEGORIES {
        let Some(group) = report["checks"][cat].as_object() else { continue };
        for (key, check) in group {
            if check["status"] != "fail" || INFORMATIONAL.contains(&key.as_str()) {
                continue;
            }
            let m = meta::meta(key);
            any = true;
            out.push_str(&format!(
                "## {}\n{}\nFix: {}\nFull guide: `agent-ready guide {key}` or the get_fix_guide tool with check \"{key}\"\nSpec: {}\n\n",
                m.title,
                check["message"].as_str().unwrap_or(""),
                m.fix,
                m.specs.join(", "),
                key = m.key,
            ));
        }
    }
    if !any {
        out.push_str("Nothing left to fix.\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(passing: &[&str]) -> usize {
        level_of(&|k: &str| passing.contains(&k))
    }

    const L3: [&str; 5] = ["robotsTxt", "sitemap", "robotsTxtAiRules", "contentSignals", "markdownNegotiation"];

    #[test]
    fn levels_build_on_each_other() {
        assert_eq!(level(&[]), 0);
        assert_eq!(level(&["robotsTxt"]), 0);
        assert_eq!(level(&["robotsTxt", "linkHeaders"]), 1);
        assert_eq!(level(&["robotsTxt", "sitemap", "robotsTxtAiRules"]), 1);
        assert_eq!(level(&["robotsTxt", "sitemap", "robotsTxtAiRules", "contentSignals"]), 2);
        assert_eq!(level(&L3), 3);
    }

    #[test]
    fn a_later_level_does_not_skip_an_earlier_one() {
        assert_eq!(level(&["robotsTxt", "sitemap", "mcpServerCard", "markdownNegotiation"]), 1);
    }

    #[test]
    fn level_five_needs_two_of_three_groups() {
        let with = |extra: &[&'static str]| {
            let mut v = L3.to_vec();
            v.extend_from_slice(extra);
            level(&v)
        };
        assert_eq!(with(&["apiCatalog"]), 4);
        assert_eq!(with(&["apiCatalog", "webBotAuth"]), 4);
        assert_eq!(with(&["apiCatalog", "webBotAuth", "authMd"]), 5);
        assert_eq!(with(&["mcpServerCard", "a2aAgentCard", "agentSkills", "apiCatalog"]), 4);
        assert_eq!(with(&["mcpServerCard", "agentSkills", "apiCatalog", "oauthDiscovery"]), 5);
        assert_eq!(with(&["agentSkills", "apiCatalog", "oauthDiscovery"]), 4);
    }

    #[test]
    fn urls_are_normalized_and_checked() {
        assert_eq!(normalize(" example.com ").unwrap(), "https://example.com");
        assert!(normalize("ftp://example.com").is_err());
        assert!(normalize("https://user@example.com").is_err());
        assert!(normalize("").is_err());
    }

    #[test]
    fn unknown_profile_or_check_is_refused() {
        assert!(select(Some("nope"), None).is_err());
        assert!(select(None, Some(vec!["nope".into()])).is_err());
        assert!(select(None, Some(vec![])).is_err());
        assert_eq!(select(Some("content"), None).unwrap().0.len(), 7);
        assert_eq!(select(None, None).unwrap().0.len(), 23);
    }
}
