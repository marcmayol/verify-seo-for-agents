//! The guides, and the documents built from them: SKILL.md files, the skills
//! index, llms.txt and llms-full.txt. Generated, so they cannot drift apart.

use crate::meta::{CATEGORIES, CHECKS, Meta};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

macro_rules! guides {
    ($($key:literal),* $(,)?) => { [$(($key, include_str!(concat!("../guides/", $key, ".md")))),*] };
}

const GUIDES: [(&str, &str); 23] = guides![
    "robotsTxt", "sitemap", "linkHeaders", "dnsAid", "markdownNegotiation", "robotsTxtAiRules", "contentSignals",
    "webBotAuth", "apiCatalog", "oauthDiscovery", "oauthProtectedResource", "authMd", "mcpServerCard", "a2aAgentCard",
    "agentSkills", "webMcp", "ard", "personalAgentProtocol", "x402", "mpp", "ucp", "acp", "ap2",
];
const OVERVIEW: &str = include_str!("../guides/_overview.md");
const SCAN_SKILL: &str = include_str!("../guides/_scan-site.md");
const SCAN_SKILL_NAME: &str = "scan-site";
const SCAN_SKILL_DESCRIPTION: &str = "Scan a website for AI agent readiness and work through what to fix. Use when asked whether a site supports agents, or to make it support them.";

/// robotsTxtAiRules -> robots-txt-ai-rules: skill names are lowercase with hyphens.
pub fn skill_name(key: &str) -> String {
    let mut out = String::new();
    for ch in key.chars() {
        if ch.is_ascii_uppercase() && !out.is_empty() {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// Finds a check by its key or its skill name, in any case.
pub fn find(name: &str) -> Option<&'static Meta> {
    CHECKS.iter().find(|m| m.key.eq_ignore_ascii_case(name) || skill_name(m.key) == name.to_ascii_lowercase())
}

pub fn guide(m: &Meta) -> String {
    let body = GUIDES.iter().find(|(k, _)| *k == m.key).map_or("", |(_, b)| b).trim_end();
    let specs: String = m.specs.iter().map(|s| format!("- {s}\n")).collect();
    format!(
        "{body}\n\n## Check it\n\n```sh\nagent-ready scan https://your-site --json --checks {key}\n```\n\nLook at `checks.{cat}.{key}.status`.\n\n## Specification\n\n{specs}",
        key = m.key,
        cat = m.category,
    )
}

fn with_frontmatter(name: &str, description: &str, body: &str) -> String {
    let quoted = serde_json::to_string(description).unwrap_or_default();
    format!("---\nname: {name}\ndescription: {quoted}\n---\n\n{body}")
}

fn check_description(m: &Meta) -> String {
    format!("Make a site pass the {} check of agent readiness. {}", m.title, m.fix)
}

/// A SKILL.md by skill name, whether of a check or the general one.
pub fn skill_md(name: &str) -> Option<String> {
    if name == SCAN_SKILL_NAME {
        return Some(with_frontmatter(SCAN_SKILL_NAME, SCAN_SKILL_DESCRIPTION, SCAN_SKILL));
    }
    let m = CHECKS.iter().find(|m| skill_name(m.key) == name)?;
    Some(with_frontmatter(name, &check_description(m), &guide(m)))
}

fn skill_entry(name: &str, description: &str, content: &str) -> Value {
    let digest: String = Sha256::digest(content.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    json!({
        "name": name,
        "type": "skill-md",
        "description": description,
        "url": format!("/.well-known/agent-skills/{name}/SKILL.md"),
        "digest": format!("sha256:{digest}"),
    })
}

pub fn skills_index() -> Value {
    let mut skills = vec![skill_entry(SCAN_SKILL_NAME, SCAN_SKILL_DESCRIPTION, &skill_md(SCAN_SKILL_NAME).unwrap_or_default())];
    for m in &CHECKS {
        let name = skill_name(m.key);
        skills.push(skill_entry(&name, &check_description(m), &skill_md(&name).unwrap_or_default()));
    }
    json!({"$schema": "https://schemas.agentskills.io/discovery/0.2.0/schema.json", "skills": skills})
}

pub fn llms() -> String {
    let mut out = String::from("# agent-ready\n\n> Checks whether a website is ready for AI agents: 23 checks, a level from 0 to 5, and how to fix each failure.\n\n");
    out.push_str("Scan with `POST /api/scan` and `{\"url\": \"https://example.com\"}`, add `\"format\": \"agent\"` for Markdown with what to fix, or use the MCP server at `POST /mcp`.\n\n## Docs\n\n");
    out.push_str("- [Full documentation](/llms-full.txt): every check, in one file\n- [Skills index](/.well-known/agent-skills/index.json)\n\n## Checks\n\n");
    for m in &CHECKS {
        out.push_str(&format!("- [{}](/.well-known/agent-skills/{}/SKILL.md)\n", m.title, skill_name(m.key)));
    }
    out
}

pub fn llms_full() -> String {
    let mut out = format!("{}\n# The checks\n", OVERVIEW.trim_end());
    for (cat, name) in CATEGORIES {
        out.push_str(&format!("\n# Category: {name}\n"));
        for m in CHECKS.iter().filter(|m| m.category == cat) {
            out.push_str(&format!("\n{}\n", nest_headings(&guide(m))));
        }
    }
    out
}

/// Moves headings a level down, so the guides sit under their category. A `#`
/// inside a code block is an example, not a heading, and stays as it is.
fn nest_headings(text: &str) -> String {
    let mut fenced = false;
    let mut out = String::new();
    for line in text.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        if line.starts_with('#') && !fenced {
            out.push('#');
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_check_has_a_guide_with_its_title() {
        for m in &CHECKS {
            let g = guide(m);
            assert!(g.starts_with(&format!("# {}", m.title)), "{} starts with its title", m.key);
            assert!(g.contains("**Passes when**"), "{} says when it passes", m.key);
            assert!(g.contains(m.specs[0]), "{} links its spec", m.key);
        }
    }

    #[test]
    fn checks_are_found_by_key_or_skill_name() {
        assert_eq!(skill_name("robotsTxtAiRules"), "robots-txt-ai-rules");
        assert_eq!(skill_name("x402"), "x402");
        assert_eq!(find("robots-txt-ai-rules").unwrap().key, "robotsTxtAiRules");
        assert_eq!(find("ROBOTSTXT").unwrap().key, "robotsTxt");
        assert!(find("nope").is_none());
    }

    #[test]
    fn the_index_digests_what_is_served() {
        let index = skills_index();
        let skills = index["skills"].as_array().unwrap();
        assert_eq!(skills.len(), 24);
        for s in skills {
            let name = s["name"].as_str().unwrap();
            let body = skill_md(name).unwrap();
            assert!(body.starts_with(&format!("---\nname: {name}\n")));
            assert_eq!(s["digest"], skill_entry(name, "x", &body)["digest"]);
        }
    }

    #[test]
    fn full_docs_keep_comments_in_code_and_every_check() {
        let full = llms_full();
        for m in &CHECKS {
            assert!(full.contains(&format!("## {}", m.title)), "{}", m.key);
        }
        assert!(full.contains("# Category: Discoverability"));
        assert!(full.contains("\n# Example auth.md\n"), "a heading in a code block stays as it is");
        assert!(full.contains("\n### Easy mistakes\n"), "a real heading moves down");
    }
}
