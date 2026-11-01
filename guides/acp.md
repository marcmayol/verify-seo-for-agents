# Agentic Commerce Protocol

A discovery document that says where your ACP checkout API lives and what it supports.

**Passes when** `/.well-known/acp.json` has `protocol.name` set to `acp`, `protocol.version`, an http(s) `api_base_url`, and non-empty `transports` and `capabilities.services` arrays.

```json file=/.well-known/acp.json
{
  "protocol": { "name": "acp", "version": "2026-01-30" },
  "api_base_url": "https://example.com/acp",
  "transports": ["rest"],
  "capabilities": { "services": ["checkout"] }
}
```

## Easy mistakes

- A relative `api_base_url`. An agent reads this document without any page around it, so the address has to be absolute.
