# Changelog

## 0.1.1

Renamed from agent-ready. The project is now verify-seo-for-agents, and the command, the MCP server and the slash command are now `verify-seo`, `verify-seo` and `/verify-seo`. Nothing else changes. If you installed 0.1.0, uninstall it first and remove its `agent-ready` block from your agents' configuration.

## 0.1.0

First version.

- 23 checks across discoverability, content, bot access, discovery and commerce, with a level from 0 to 5 and a percentage.
- A guide for each check, with a working example.
- Command line, local web UI, HTTP API and MCP server. It can scan localhost.
- Plugin for Claude Code, and `agent-ready setup` for 14 other coding agents.
