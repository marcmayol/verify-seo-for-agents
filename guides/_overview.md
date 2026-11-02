# verify-seo-for-agents

verify-seo-for-agents checks whether a website is ready for AI agents. It runs on your own machine, and it can scan `localhost`.

## Run a scan

Terminal:

```sh
verify-seo scan https://example.com          # summary
verify-seo scan https://example.com --json   # full report
verify-seo scan https://example.com --agent  # Markdown with what to fix
verify-seo scan https://example.com --checks robotsTxt,sitemap
verify-seo guide robotsTxt                   # how to fix one check
```

Local server, started with `verify-seo serve`:

```
POST /api/scan
{"url": "https://example.com", "profile": "content", "format": "json"}
```

`url` is the only required field. `profile` is `all`, `content` or `apiApp`, `enabledChecks` is a list of checks that replaces the profile, and `format` is `json` or `agent`.

MCP, over stdio with `verify-seo mcp` or at `POST /mcp` on the server. The tools are `scan_site` and `get_fix_guide`.

## The report

| Field | Meaning |
|---|---|
| `url`, `targetUrl` | The origin scanned and the final page after redirects |
| `level`, `levelName` | From 0 to 5 |
| `score.overall` | Percentage of the applicable checks that pass |
| `score.categories` | The same, per category |
| `checks.<category>.<check>` | `status`, `message`, `details`, `evidence`, and `fix` and `guide` when it did not pass |
| `nextLevel` | The next level and what it takes |
| `isCommerce`, `commerceSignals` | Whether the homepage looks like a store, and why |

A status is `pass`, `fail`, `neutral` (does not apply, not counted) or `unableToCheck` (could not be determined, counted as not passed).

## Levels

| Level | Name | Needs |
|---|---|---|
| 0 | Not Ready | |
| 1 | Basic Web Presence | Two of: robots.txt, sitemap, Link headers |
| 2 | Bot-Aware | AI bot rules and Content Signals |
| 3 | Agent-Readable | Markdown negotiation |
| 4 | Agent-Integrated | One of: MCP Server Card, A2A Agent Card, Agent Skills, API catalog |
| 5 | Agent-Native | Two of: Web Bot Auth; MCP Server Card, Agent Skills and API catalog together; any of OAuth discovery, OAuth Protected Resource, Auth.md |

Each level needs all the ones before it.

The percentage leaves out commerce, neutral results and the Personal Agent Protocol. The A2A Agent Card and AP2 count only when requested by name.

Commerce checks only count on sites detected as stores, which takes three or more signals in the first 100,000 characters of the homepage: a store platform, payment brands, product schema, links such as `/cart`, or several prices.

If the homepage answers with an HTTP error, the report has a `siteError` with the status and no checks, level or score.
