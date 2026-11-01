# A2A Agent Card

This applies if you run an agent that other agents can talk to. It is opt-in, and it only enters the percentage when you ask for this check by name, though it does count toward the level.

**Passes when** `/.well-known/agent-card.json` has `name`, `version` and a non-empty `supportedInterfaces`.

```json file=/.well-known/agent-card.json
{
  "name": "Example Support Agent",
  "description": "Answers questions about orders on example.com.",
  "version": "1.0.0",
  "supportedInterfaces": [
    { "url": "https://example.com/a2a", "protocolBinding": "JSONRPC", "protocolVersion": "1.0" }
  ],
  "capabilities": { "streaming": false },
  "skills": [
    { "id": "order-status", "name": "Order status", "description": "Looks up where an order is.", "tags": ["orders"] }
  ]
}
```

## Easy mistakes

- Copying an older card that has a top-level `url` and no `supportedInterfaces`. That layout is not accepted.
