//! MCP: the same JSON-RPC handler serves stdio and Streamable HTTP.

use crate::{docs, scan};
use serde_json::{Value, json};
use std::io::{BufRead, Write};

const VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

fn tool_schema() -> Value {
    json!({
        "name": "scan_site",
        "description": "Scan a website for AI agent readiness. Returns a level from 0 to 5, a score, the result of 23 checks and how to fix what fails. Can scan localhost.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "The site to scan."},
                "format": {"type": "string", "enum": ["agent", "json"], "description": "agent: Markdown with what to fix (default). json: the full report."},
                "profile": {"type": "string", "enum": ["all", "content", "apiApp"], "description": "Which checks to run. Default: all."},
            },
            "required": ["url"],
        },
    })
}

fn guide_schema() -> Value {
    json!({
        "name": "get_fix_guide",
        "description": "How to make a site pass one check of scan_site: what it looks for, a working example and the usual mistakes.",
        "inputSchema": {
            "type": "object",
            "properties": {"check": {"type": "string", "description": "The check, for example robotsTxt or mcpServerCard."}},
            "required": ["check"],
        },
    })
}

fn call_tool(params: &Value) -> Value {
    if params["name"] == "get_fix_guide" {
        let name = params["arguments"]["check"].as_str().unwrap_or("");
        return match docs::find(name) {
            Some(m) => json!({"content": [{"type": "text", "text": docs::guide(m)}]}),
            None => json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown check: {name}")}]}),
        };
    }
    if params["name"] != "scan_site" {
        return json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown tool: {}", params["name"])}]});
    }
    let args = &params["arguments"];
    let url = args["url"].as_str().unwrap_or("");
    let result = scan::select(args["profile"].as_str(), None).and_then(|(keys, asked)| scan::scan(url, &keys, &asked));
    match result {
        Err(e) => json!({"isError": true, "content": [{"type": "text", "text": e}]}),
        Ok(report) => {
            let text = if args["format"] == "json" { report.to_string() } else { scan::agent_markdown(&report) };
            json!({"content": [{"type": "text", "text": text}]})
        }
    }
}

/// `verify-seo tool`: the tools as JSON, for the Claude Code module to register.
pub fn list_tools() -> Value {
    json!({"tools": [tool_schema(), guide_schema()]})
}

/// `verify-seo tool <name>`: runs one with its arguments, answering as the MCP server does.
pub fn run_tool(name: &str, arguments: Value) -> Result<String, String> {
    let result = call_tool(&json!({"name": name, "arguments": arguments}));
    let text = result["content"][0]["text"].as_str().unwrap_or("").to_string();
    if result["isError"] == true { Err(text) } else { Ok(text) }
}

/// The answer to a JSON-RPC message; notifications have none.
pub fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let result = match msg["method"].as_str().unwrap_or("") {
        "initialize" => {
            let asked = msg["params"]["protocolVersion"].as_str().unwrap_or("");
            let version = VERSIONS.iter().find(|v| **v == asked).copied().unwrap_or(VERSIONS[0]);
            json!({
                "protocolVersion": version,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "verify-seo", "version": env!("CARGO_PKG_VERSION")},
            })
        }
        "ping" => json!({}),
        "tools/list" => list_tools(),
        "tools/call" => call_tool(&msg["params"]),
        other => {
            return Some(json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("Method not found: {other}")}}));
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

pub fn run_stdio() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(&msg),
            Err(e) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": e.to_string()}})),
        };
        if let Some(r) = reply {
            let _ = writeln!(stdout, "{r}");
            let _ = stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notifications_get_no_answer() {
        assert!(handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none());
    }

    #[test]
    fn initialize_echoes_a_known_version_and_falls_back() {
        let r = handle(&json!({"id": 1, "method": "initialize", "params": {"protocolVersion": "2025-03-26"}})).unwrap();
        assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
        let r = handle(&json!({"id": 1, "method": "initialize", "params": {"protocolVersion": "1999"}})).unwrap();
        assert_eq!(r["result"]["protocolVersion"], VERSIONS[0]);
    }

    #[test]
    fn tools_run_directly_as_they_do_over_mcp() {
        assert!(run_tool("get_fix_guide", json!({"check": "acp"})).unwrap().contains("acp.json"));
        assert!(run_tool("get_fix_guide", json!({"check": "nope"})).is_err());
        assert!(run_tool("nope", json!({})).is_err());
        assert_eq!(list_tools()["tools"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn unknown_method_is_an_error() {
        let r = handle(&json!({"id": 2, "method": "nope"})).unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }
}
