# Scan a site for agent readiness

Use this to find out whether a website works for AI agents, and what to change so that it does. It also works on `localhost`, so a site can be tested before it is published.

## How to run it

With the MCP server, call `scan_site` with `url`. Its answer is Markdown listing what to fix. Pass `format: "json"` for the full report, and `profile` to narrow the checks.

From a terminal:

```sh
verify-seo scan https://example.com --agent
```

## How to work through the result

1. Read the level. A site reaches a level only after meeting all the earlier ones, so work upward and ignore higher levels for now.
2. Take the checks in `nextLevel.requirements`, since those are what moves the level.
3. For each one, read its guide: `verify-seo guide <check>`, or the `get_fix_guide` MCP tool.
4. Scan again and confirm that the status changed.

Some checks do not apply to every site. Do not add OAuth metadata to a site with no login, or payment protocols to a site that sells nothing: choose the `content` profile for a blog or documentation.
