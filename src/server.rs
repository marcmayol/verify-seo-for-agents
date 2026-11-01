//! The local web UI, POST /api/scan and POST /mcp.

use crate::{docs, mcp, scan};
use serde_json::{Value, json};
use std::io::Read;
use tiny_http::{Header, Method, Request, Response, Server};

const UI: &str = include_str!("ui.html");
const MAX_REQUEST: u64 = 64 * 1024;

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn respond(req: Request, status: u16, kind: &str, body: String) {
    let _ = req.respond(Response::from_string(body).with_status_code(status).with_header(header("Content-Type", kind)));
}

fn respond_json(req: Request, status: u16, body: Value) {
    respond(req, status, "application/json", body.to_string());
}

fn header_value(req: &Request, name: &str) -> Option<String> {
    req.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.as_str().to_string())
}

/// This server fetches any URL it is given, including ones on the local
/// network, so a web page in the user's browser must not be able to drive it:
/// the Host has to be the one we listen on (DNS rebinding), and a browser's
/// Origin has to be our own (cross-site POST).
fn allowed(req: &Request, public: bool) -> bool {
    let host = header_value(req, "host").unwrap_or_default();
    let name = host.rsplit_once(':').map_or(host.as_str(), |(h, _)| h);
    if !public && !matches!(name, "127.0.0.1" | "localhost" | "[::1]") {
        return false;
    }
    match header_value(req, "origin") {
        None => true,
        Some(origin) => origin.split_once("://").is_some_and(|(_, rest)| rest == host),
    }
}

fn read_json(req: &mut Request) -> Result<Value, String> {
    let mut body = Vec::new();
    req.as_reader().take(MAX_REQUEST).read_to_end(&mut body).map_err(|e| e.to_string())?;
    serde_json::from_slice(&body).map_err(|e| format!("Invalid JSON: {e}"))
}

fn api_scan(mut req: Request) {
    let body = match read_json(&mut req) {
        Ok(b) => b,
        Err(e) => return respond_json(req, 400, json!({"error": e})),
    };
    let url = body["url"].as_str().unwrap_or("").to_string();
    let enabled = body["enabledChecks"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect());
    let selected = scan::select(body["profile"].as_str(), enabled);
    let (keys, asked) = match selected {
        Ok(s) => s,
        Err(e) => return respond_json(req, 400, json!({"error": e})),
    };
    match scan::scan(&url, &keys, &asked) {
        Err(e) => respond_json(req, 400, json!({"error": e})),
        Ok(report) if body["format"] == "agent" => respond(req, 200, "text/markdown; charset=utf-8", scan::agent_markdown(&report)),
        Ok(report) => respond_json(req, 200, report),
    }
}

fn api_mcp(mut req: Request) {
    let body = match read_json(&mut req) {
        Ok(b) => b,
        Err(e) => return respond_json(req, 400, json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": e}})),
    };
    match mcp::handle(&body) {
        Some(reply) => respond_json(req, 200, reply),
        None => {
            let _ = req.respond(Response::empty(202));
        }
    }
}

pub fn run(host: &str, port: u16) -> Result<(), String> {
    let server = Server::http((host, port)).map_err(|e| format!("Could not listen on {host}:{port}: {e}"))?;
    let public = !matches!(host, "127.0.0.1" | "localhost" | "::1");
    let port = server.server_addr().to_ip().map_or(port, |a| a.port());
    println!("agent-ready listening on http://{host}:{port}");
    if public {
        println!("Warning: anyone who can reach this address can make this machine fetch URLs on its network.");
    }
    for req in server.incoming_requests() {
        std::thread::spawn(move || {
            if !allowed(&req, public) {
                return respond_json(req, 403, json!({"error": "Forbidden"}));
            }
            let path = req.url().split('?').next().unwrap_or("").to_string();
            match (req.method(), path.as_str()) {
                (Method::Get, "/") => respond(req, 200, "text/html; charset=utf-8", UI.to_string()),
                (Method::Get, "/healthz") => respond(req, 200, "text/plain", "ok".into()),
                (Method::Get, "/llms.txt") => respond(req, 200, "text/plain; charset=utf-8", docs::llms()),
                (Method::Get, "/llms-full.txt") => respond(req, 200, "text/plain; charset=utf-8", docs::llms_full()),
                (Method::Get, "/.well-known/agent-skills/index.json") => respond_json(req, 200, docs::skills_index()),
                (Method::Get, p) if p.starts_with("/.well-known/agent-skills/") && p.ends_with("/SKILL.md") => {
                    let name = p.trim_start_matches("/.well-known/agent-skills/").trim_end_matches("/SKILL.md");
                    match docs::skill_md(name) {
                        Some(body) => respond(req, 200, "text/markdown; charset=utf-8", body),
                        None => respond(req, 404, "text/plain", "Not found".into()),
                    }
                }
                (Method::Post, "/api/scan") => api_scan(req),
                (Method::Post, "/mcp") => api_mcp(req),
                (Method::Get, "/mcp") => respond(req, 405, "text/plain", "Use POST".into()),
                _ => respond(req, 404, "text/plain", "Not found".into()),
            }
        });
    }
    Ok(())
}
