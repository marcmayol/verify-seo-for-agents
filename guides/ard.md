# Agentic Resource Discovery

A single manifest listing what agents can use on your site, whether MCP servers, A2A agents or skills.

**Passes when** a manifest is found through any of these, has a non-empty `specVersion`, and has at least one valid entry:

- `/.well-known/ai-catalog.json`
- an `Agentmap:` line in `robots.txt`
- `<link rel="ai-catalog" href="...">` in the page
- a TXT record at `_catalog._agents.<your domain>` holding `url=...`

A valid entry has `identifier`, `displayName`, `type` and exactly one of `url` or `data`.

```json file=/.well-known/ai-catalog.json
{
  "specVersion": "1.0",
  "host": { "displayName": "Example", "identifier": "did:web:example.com" },
  "entries": [
    {
      "identifier": "urn:air:example.com:mcp:orders",
      "displayName": "Orders MCP server",
      "type": "application/mcp-server-card+json",
      "url": "https://example.com/.well-known/mcp/server-card.json"
    }
  ]
}
```

## Easy mistakes

- Giving an entry both `url` and `data`, or neither. Each must have exactly one.
- Reading `specVersion` as the ARD version. It refers to the catalog data model, and only has to be present.
