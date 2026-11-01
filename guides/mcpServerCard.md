# MCP Server Card

The card describes your MCP server before any agent connects to it: its name, its address and what it can do.

**Passes when** valid JSON sits at `/.well-known/mcp/server-card.json`, `/.well-known/mcp/server-cards.json` or `/.well-known/mcp.json`, with a `serverInfo.name`. A flat `name` also works when it comes with `version`, `remotes` or `packages`.

```json file=/.well-known/mcp/server-card.json
{
  "serverInfo": { "name": "example-mcp", "version": "1.0.0" },
  "description": "Search and manage orders on example.com.",
  "transport": { "type": "streamable-http", "endpoint": "https://example.com/mcp" },
  "capabilities": { "tools": {} }
}
```

## Easy mistakes

- Publishing a `name` with an endpoint and little else, in any field names. Any JSON file can have a name, so without a version this is not accepted as a card.
- Saying a server needs no login when it does. Agents plan around what the card says.
