mod browser;
mod checks;
mod docs;
mod http;
mod mcp;
mod meta;
mod scan;
mod server;
mod setup;

use serde_json::Value;

const USAGE: &str = "verify-seo: check whether a website is ready for AI agents

Usage:
  verify-seo scan <url> [--json | --agent] [--profile all|content|apiApp] [--checks a,b] [--min-level N]
  verify-seo guide [check]
  verify-seo docs
  verify-seo serve [--port 8787] [--host 127.0.0.1]
  verify-seo mcp
  verify-seo setup <agent> [--dry-run]
  verify-seo setup --all [--dry-run]
  verify-seo --version

scan    Prints a summary, the full report (--json) or what to fix (--agent).
        With --min-level, exits with 2 when the site is below that level.
guide   How to pass one check, or the list of checks.
docs    The full documentation, as served at /llms-full.txt.
serve   Web UI, POST /api/scan, POST /mcp, /llms.txt and the skills index.
mcp     MCP server over stdio.
setup   Gets a coding agent ready: MCP server, instructions and /verify-seo. --all does it in
        every agent found on this computer. Agents: codex, gemini, cursor, opencode, qwen, kiro,
        copilot, windsurf, antigravity, kimi, pi, amp, kilo, grok. In Claude Code use the plugin:
        /plugin marketplace add marcmayol/drac-plugins, then /plugin install verify-seo-for-agents@drac

WebMCP needs a Chromium-based browser installed; set VERIFY_SEO_BROWSER to pick one.";

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn summary(report: &Value) -> String {
    if let Some(status) = report["siteError"]["httpStatus"].as_u64() {
        return format!(
            "{}\nThe homepage answered HTTP {status}, so the checks did not run.\nIf the site blocks automated requests, allow them or scan a copy on localhost.\n",
            report["targetUrl"].as_str().unwrap_or("")
        );
    }
    let mut out = format!(
        "{}\nLevel {}/5 - {} ({}%)\n",
        report["targetUrl"].as_str().unwrap_or(""),
        report["level"],
        report["levelName"].as_str().unwrap_or(""),
        report["score"]["overall"],
    );
    for (cat, name) in meta::CATEGORIES {
        let Some(group) = report["checks"][cat].as_object() else { continue };
        out.push_str(&format!("\n{name}\n"));
        for (key, check) in group {
            let mark = match check["status"].as_str().unwrap_or("") {
                "pass" => "[ok]  ",
                "fail" => "[fail]",
                "neutral" => "[ -- ]",
                _ => "[ ?  ]",
            };
            out.push_str(&format!("  {mark} {:<28} {}\n", key, check["message"].as_str().unwrap_or("")));
        }
    }
    out
}

fn cmd_scan(args: &[String]) -> Result<i32, String> {
    let values = ["--profile", "--min-level", "--checks"].map(|f| flag(args, f));
    let url = args.iter().find(|a| !a.starts_with("--") && !values.iter().any(|v| v.as_ref() == Some(*a))).ok_or("Missing url")?;
    let enabled = flag(args, "--checks").map(|c| c.split(',').map(|s| s.trim().to_string()).collect());
    let (keys, asked) = scan::select(flag(args, "--profile").as_deref(), enabled)?;
    let report = scan::scan(url, &keys, &asked)?;
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
    } else if args.iter().any(|a| a == "--agent") {
        print!("{}", scan::agent_markdown(&report));
    } else {
        print!("{}", summary(&report));
    }
    if let Some(min) = flag(args, "--min-level") {
        let min: u64 = min.parse().map_err(|_| "--min-level needs a number")?;
        if report["level"].as_u64().unwrap_or(0) < min {
            return Ok(2);
        }
    }
    Ok(0)
}

fn cmd_guide(args: &[String]) -> Result<i32, String> {
    let Some(name) = args.first() else {
        for m in &meta::CHECKS {
            println!("{:<26} {}", m.key, m.title);
        }
        return Ok(0);
    };
    let m = docs::find(name).ok_or_else(|| format!("Unknown check: {name}"))?;
    print!("{}", docs::guide(m));
    Ok(0)
}

fn cmd_setup(args: &[String]) -> Result<i32, String> {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let all = args.iter().any(|a| a == "--all");
    let agents: Vec<String> = if all {
        setup::presentes()?.into_iter().map(String::from).collect()
    } else {
        args.iter().filter(|a| !a.starts_with("--")).cloned().collect()
    };
    if !all && agents.is_empty() {
        return Err(format!("Which agent? {}, or --all", setup::AGENTES.join(", ")));
    }
    if all && agents.is_empty() {
        println!("No agent found on this computer. Name one: verify-seo setup <agent>");
    }
    let mut failed = 0;
    for agent in &agents {
        if agent == "claude" {
            println!("claude: use the plugin: /plugin marketplace add marcmayol/drac-plugins, then /plugin install verify-seo-for-agents@drac");
            continue;
        }
        match setup::plan(agent) {
            Ok(changes) if changes.is_empty() => println!("{agent}: already set up"),
            Ok(changes) => {
                println!("{agent}:");
                for c in &changes {
                    println!("  {} ({})", c.fichero.display(), c.que);
                }
                if dry_run {
                    println!("  dry run: nothing written");
                } else if let Err(e) = setup::aplicar(&changes) {
                    eprintln!("  {e}");
                    failed += 1;
                } else {
                    println!("  done: start a new session of the agent");
                }
            }
            Err(e) => {
                eprintln!("{agent}: {e}");
                failed += 1;
            }
        }
    }
    if failed > 0 { Err(format!("{failed} agent(s) failed")) } else { Ok(0) }
}

/// `verify-seo tool` lists the tools; `verify-seo tool <name>` runs one with JSON arguments from stdin.
fn cmd_tool(args: &[String]) -> Result<i32, String> {
    let Some(name) = args.first() else {
        println!("{}", mcp::list_tools());
        return Ok(0);
    };
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).map_err(|e| e.to_string())?;
    let arguments = if input.trim().is_empty() { Value::Null } else { serde_json::from_str(&input).map_err(|e| format!("Invalid JSON: {e}"))? };
    println!("{}", mcp::run_tool(name, arguments)?);
    Ok(0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    let result = match args.first().map(String::as_str) {
        Some("scan") => cmd_scan(rest),
        Some("setup") => cmd_setup(rest),
        Some("tool") => cmd_tool(rest),
        Some("--version" | "-V") => {
            println!("verify-seo {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Some("guide") => cmd_guide(rest),
        Some("docs") => {
            print!("{}", docs::llms_full());
            Ok(0)
        }
        Some("serve") => {
            let port = flag(rest, "--port").and_then(|p| p.parse().ok()).unwrap_or(8787);
            let host = flag(rest, "--host").unwrap_or_else(|| "127.0.0.1".into());
            server::run(&host, port).map(|_| 0)
        }
        Some("mcp") => {
            mcp::run_stdio();
            Ok(0)
        }
        _ => {
            println!("{USAGE}");
            Ok(0)
        }
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
