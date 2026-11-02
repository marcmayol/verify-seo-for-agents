//! Loads a page in the Chromium browser already on the machine and records the
//! WebMCP tools it registers. Everything talks to the browser's own debug port.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tungstenite::{Message, WebSocket};

const PAGE_DEADLINE: Duration = Duration::from_secs(20);
const SETTLE: Duration = Duration::from_secs(6);
const POLL: Duration = Duration::from_millis(500);

const SHIM: &str = r#"(() => {
  const tools = [];
  window.__verifySeoTools = tools;
  const add = (t) => { if (t && t.name) tools.push({ name: String(t.name), description: t.description ? String(t.description) : "", hasSchema: !!t.inputSchema }); };
  const mc = {
    registerTool(t) { add(t); return Promise.resolve(); },
    unregisterTool() {},
    provideContext(c) { ((c && c.tools) || []).forEach(add); },
    clearContext() {},
  };
  for (const proto of [Document.prototype, Navigator.prototype]) {
    // Not configurable: a page with a WebMCP origin trial token makes the browser
    // install the native API after this runs, which would replace this one and
    // hide every tool registered through it.
    try { Object.defineProperty(proto, "modelContext", { get: () => mc, configurable: false }); } catch {}
  }
})()"#;

const READ_TOOLS: &str = r#"JSON.stringify({
  imperative: window.__verifySeoTools || [],
  declarative: [...document.querySelectorAll("form[toolname]")].map(f => ({ name: f.getAttribute("toolname"), description: f.getAttribute("tooldescription") || "", hasSchema: false })),
})"#;

pub struct Found {
    pub imperative: Vec<Value>,
    pub declarative: Vec<Value>,
    pub loaded: bool,
}

pub enum Outcome {
    NoBrowser,
    Failed(String),
    Done(Found),
}

fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("VERIFY_SEO_BROWSER") {
        return vec![PathBuf::from(p)];
    }
    if cfg!(windows) {
        for var in ["ProgramFiles", "ProgramFiles(x86)", "LocalAppData"] {
            if let Ok(base) = std::env::var(var) {
                let base = PathBuf::from(base);
                out.push(base.join(r"Google\Chrome\Application\chrome.exe"));
                out.push(base.join(r"Microsoft\Edge\Application\msedge.exe"));
                out.push(base.join(r"Chromium\Application\chrome.exe"));
                out.push(base.join(r"BraveSoftware\Brave-Browser\Application\brave.exe"));
            }
        }
    } else if cfg!(target_os = "macos") {
        for app in [
            "Google Chrome.app/Contents/MacOS/Google Chrome",
            "Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "Chromium.app/Contents/MacOS/Chromium",
            "Brave Browser.app/Contents/MacOS/Brave Browser",
        ] {
            out.push(PathBuf::from("/Applications").join(app));
        }
    } else if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for name in ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "microsoft-edge", "brave-browser"] {
                out.push(dir.join(name));
            }
        }
    }
    out
}

/// Closes the browser and removes its throwaway profile, whatever happens.
struct Session {
    child: Child,
    profile: PathBuf,
    browser_ws: String,
}

impl Drop for Session {
    fn drop(&mut self) {
        // A polite close first: on Windows killing the parent leaves children
        // holding the profile folder open.
        if let Ok(mut ws) = connect(&self.browser_ws) {
            let _ = ws.send(Message::text(json!({"id": 1, "method": "Browser.close"}).to_string()));
        }
        let end = Instant::now() + Duration::from_secs(2);
        while Instant::now() < end {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        remove_profile(&self.profile);
    }
}

/// The browser's helper processes outlive it by a moment and keep files open,
/// which on Windows makes the first attempts to remove the folder fail.
fn remove_profile(profile: &PathBuf) {
    let end = Instant::now() + Duration::from_secs(5);
    while std::fs::remove_dir_all(profile).is_err() && profile.exists() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn connect(url: &str) -> Result<WebSocket<TcpStream>, String> {
    let authority = url.strip_prefix("ws://").and_then(|r| r.split('/').next()).ok_or("bad debugger url")?;
    let stream = TcpStream::connect(authority).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_millis(250))).map_err(|e| e.to_string())?;
    let (ws, _) = tungstenite::client(url, stream).map_err(|e| e.to_string())?;
    Ok(ws)
}

/// A process killed mid-scan cannot clean up after itself, so the next run does.
/// No scan lasts an hour, so anything older is not in use.
fn sweep_old_profiles() {
    let Ok(dir) = std::fs::read_dir(std::env::temp_dir()) else { return };
    for entry in dir.flatten() {
        let stale = entry.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age > Duration::from_secs(3600));
        if stale && entry.file_name().to_string_lossy().starts_with("verify-seo-") {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

fn start(exe: &PathBuf) -> Result<Session, String> {
    sweep_old_profiles();
    let profile =std::env::temp_dir().join(format!("verify-seo-{}-{:?}", std::process::id(), std::thread::current().id()).replace(['(', ')'], ""));
    let mut child = Command::new(exe)
        .args(["--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check", "--disable-extensions", "--remote-debugging-port=0"])
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg("about:blank")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stderr = child.stderr.take().ok_or("no stderr")?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Some(i) = line.find("ws://") {
                let _ = tx.send(line[i..].trim().to_string());
            }
        }
    });
    match rx.recv_timeout(Duration::from_secs(15)) {
        Ok(browser_ws) => Ok(Session { child, profile, browser_ws }),
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            remove_profile(&profile);
            Err("the browser did not open its debug port".into())
        }
    }
}

fn send(ws: &mut WebSocket<TcpStream>, id: u64, method: &str, params: Value) -> Result<(), String> {
    ws.send(Message::text(json!({"id": id, "method": method, "params": params}).to_string())).map_err(|e| e.to_string())
}

/// One CDP message, or None when nothing arrived before the short read timeout.
fn recv(ws: &mut WebSocket<TcpStream>) -> Result<Option<Value>, String> {
    match ws.read() {
        Ok(Message::Text(t)) => Ok(serde_json::from_str(t.as_str()).ok()),
        Ok(_) => Ok(None),
        Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn detect(url: &str) -> Outcome {
    let Some(exe) = candidates().into_iter().find(|p| p.is_file()) else {
        return Outcome::NoBrowser;
    };
    match run(&exe, url) {
        Ok(found) => Outcome::Done(found),
        Err(e) => Outcome::Failed(e),
    }
}

fn run(exe: &PathBuf, url: &str) -> Result<Found, String> {
    let session = start(exe)?;
    let authority = session.browser_ws.strip_prefix("ws://").and_then(|r| r.split('/').next()).ok_or("bad debugger url")?;

    let agent: ureq::Agent = ureq::Agent::config_builder().http_status_as_error(false).build().into();
    let mut res = agent.put(&format!("http://{authority}/json/new?about:blank")).send_empty().map_err(|e| e.to_string())?;
    let target: Value = serde_json::from_reader(res.body_mut().as_reader()).map_err(|e| e.to_string())?;
    let page_ws = target["webSocketDebuggerUrl"].as_str().ok_or("no page target")?;

    let mut ws = connect(page_ws)?;
    send(&mut ws, 1, "Page.enable", json!({}))?;
    send(&mut ws, 2, "Page.addScriptToEvaluateOnNewDocument", json!({"source": SHIM}))?;
    // The shim has to be in place before navigating, or the page's first
    // inline scripts run without it.
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        if Instant::now() > ready {
            return Err("the browser did not accept the detection script".into());
        }
        if recv(&mut ws)?.is_some_and(|m| m["id"] == 2) {
            break;
        }
    }
    send(&mut ws, 3, "Page.navigate", json!({"url": url}))?;

    let deadline = Instant::now() + PAGE_DEADLINE;
    let mut loaded = false;
    while !loaded && Instant::now() < deadline {
        if let Some(msg) = recv(&mut ws)? {
            loaded = msg["method"] == "Page.loadEventFired";
        }
    }
    // Apps often register their tools well after load, once the framework has
    // hydrated, so keep looking for a while and stop as soon as there is one.
    let settle = Instant::now() + SETTLE;
    let mut id = 4;
    loop {
        let found = read_tools(&mut ws, id, loaded)?;
        if !found.imperative.is_empty() || !found.declarative.is_empty() || Instant::now() >= settle {
            return Ok(found);
        }
        id += 1;
        let pause = Instant::now() + POLL;
        while Instant::now() < pause {
            recv(&mut ws)?;
        }
    }
}

fn read_tools(ws: &mut WebSocket<TcpStream>, id: u64, loaded: bool) -> Result<Found, String> {
    send(ws, id, "Runtime.evaluate", json!({"expression": READ_TOOLS, "returnByValue": true}))?;
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        if let Some(msg) = recv(ws)?
            && msg["id"] == id
        {
            let raw = msg["result"]["result"]["value"].as_str().unwrap_or("{}");
            let v: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
            let list = |k: &str| v[k].as_array().cloned().unwrap_or_default();
            return Ok(Found { imperative: list("imperative"), declarative: list("declarative"), loaded });
        }
    }
    Err("the browser did not answer".into())
}
