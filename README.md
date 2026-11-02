# verify-seo-for-agents

The command is `verify-seo`.

[Español](README.es.md)

Checks whether a website is ready for AI agents, on your own machine: 23 checks, a level from 0 to 5, a percentage, and a guide to fix each failure. It can scan `localhost`, so you can test a site before publishing it.

One small binary for Windows, macOS and Linux, with nothing else to install. It works from the terminal, in the browser, as an HTTP API and as an MCP server for your coding agent.

It is an independent tool that follows the public method of [isitagentready.com](https://isitagentready.com), and it is not affiliated with Cloudflare. Its results were compared with that site on 26 websites, and matched in 597 of 598 checks.

## Install

**Claude Code.** Installing the plugin is all: it downloads the binary on first use.

```
/plugin marketplace add marcmayol/drac-plugins
/plugin install verify-seo-for-agents@drac
```

**Other agents, or just the command.** This installs the binary and sets up every agent it finds on your computer: Codex, Gemini CLI, Cursor, OpenCode, Qwen Code, Kiro, Copilot CLI, Windsurf, Antigravity, Kimi Code, Pi, Amp, Kilo Code and Grok.

```sh
# Linux and macOS
curl -fsSL https://raw.githubusercontent.com/marcmayol/verify-seo-for-agents/main/install.sh | sh
```

```powershell
# Windows
irm https://raw.githubusercontent.com/marcmayol/verify-seo-for-agents/main/install.ps1 | iex
```

An agent you install later: `verify-seo setup <agent>`. Each file touched keeps a `.before-verify-seo` copy.

## Use

In your agent, ask for it in your own words, or run `/verify-seo https://example.com`. It scans the site and, if you are in its source code, offers to fix what fails.

From a terminal:

```
verify-seo scan example.com                 # summary
verify-seo scan localhost:3000              # your site before publishing it
verify-seo scan example.com --json          # full report
verify-seo scan example.com --agent         # Markdown with what to fix
verify-seo scan example.com --profile content
verify-seo scan example.com --min-level 3   # exits with 2 below that level, for CI
verify-seo guide mcpServerCard              # how to pass one check
verify-seo docs                             # all the documentation
verify-seo serve                            # web UI at http://127.0.0.1:8787
```

Profiles: `all`, `content` for a blog or documentation, and `apiApp` for an API or web app.

## What it checks

| Category | Checks |
|---|---|
| Discoverability | robots.txt, sitemap, Link headers, DNS-AID |
| Content | Markdown negotiation |
| Bot access | AI bot rules, Content Signals, Web Bot Auth |
| Discovery | API catalog, OAuth discovery, OAuth Protected Resource, Auth.md, MCP Server Card, A2A Agent Card, Agent Skills, WebMCP, ARD, Personal Agent Protocol |
| Commerce | x402, MPP, UCP, ACP, AP2. They only count on sites detected as stores |

Not every check makes sense for every site. A blog needs no login, and a site that sells nothing needs no payment protocol.

Each check has a guide in `guides/` with what it looks for, a working example and the usual mistakes. The binary carries them, and generates from them the `SKILL.md` files, the skills index, `llms.txt` and `llms-full.txt`, which `serve` publishes at the usual addresses.

## What it needs

Nothing to scan, except for one check: WebMCP runs the page, so it needs Chrome, Edge, Brave or Chromium. Without one it still finds tools declared in the HTML, and otherwise reports `unableToCheck`. Use `VERIFY_SEO_BROWSER` to choose the browser.

DNS-AID and ARD ask Cloudflare's DNS-over-HTTPS resolver, and Google's if that fails. Nothing else leaves your machine except the requests to the site you scan.

## Security

`serve` listens on `127.0.0.1` and refuses requests from other sites, because it fetches whatever URL it is given. `--host 0.0.0.0` lets anyone who reaches it make your machine fetch URLs on your network, so use it only on a network you trust.

A scan loads the page in your browser's sandbox, with a throwaway profile that is deleted afterwards.

## Known differences from the original

- A site listed only in the x402 Bazaar directory is not detected: that lookup depends on a third party.
- The MCP check validates the card and does not probe the endpoint.
- The original's rules were worked out from its answers, since its published method is less precise than its behaviour. This tool follows the behaviour. Which homepages count as a store, and what Auth.md needs without OAuth metadata, are the least certain, and DNS-AID has only been tested against the DNS answer format, as no compared site publishes it.

## Development

```
cargo test --release
cargo build --release
python3 -I tools/probar_guias.py             # every guide's example has to pass its check
python3 -I tools/web_prueba.py 8799          # local site that implements every standard
python3 -I tools/calibrar.py                 # compare with saved answers from the original
```

`calibrar.py`, and the text comparison in `probar_guias.py`, use reference material kept in `spec/`, which is private and stays out of git. Guides are written from the standards and from what this code checks, never by copying that material.

## License

MIT
