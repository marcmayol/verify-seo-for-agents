# Changelog

## 0.1.2

- New optional checks and guides for `llms.txt` and `llms-full.txt`. They run only when asked for by name, and never change the level or the percentage.
- WebMCP: a declared form with fields now reports that it has a schema, and a tool seen both ways counts once.
- `scan_site` takes `enabledChecks`.

## 0.1.1

Renamed from agent-ready. The project is now verify-seo-for-agents, and the command, the MCP server and the slash command are now `verify-seo`, `verify-seo` and `/verify-seo`. Nothing else changes. If you installed 0.1.0, uninstall it first and remove its `agent-ready` block from your agents' configuration.

## 0.1.0

First version.

- 23 checks across discoverability, content, bot access, discovery and commerce, with a level from 0 to 5 and a percentage.
- A guide for each check, with a working example.
- Command line, local web UI, HTTP API and MCP server. It can scan localhost.
- Plugin for Claude Code, and `agent-ready setup` for 14 other coding agents.
