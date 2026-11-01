# Agent Payments Protocol

AP2 lets an agent show it has a user's authority to pay. It is declared inside an A2A Agent Card, so you need that card first. This check is opt-in.

**Passes when** `capabilities.extensions` in `/.well-known/agent-card.json` has an entry whose `uri` mentions AP2, with `params.roles`.

```json file=/.well-known/agent-card.json
{
  "name": "Example Store Agent",
  "version": "1.0.0",
  "supportedInterfaces": [
    { "url": "https://example.com/a2a", "protocolBinding": "JSONRPC", "protocolVersion": "1.0" }
  ],
  "capabilities": {
    "extensions": [
      { "uri": "https://github.com/google-agentic-commerce/ap2/tree/v0.1", "params": { "roles": ["merchant"] } }
    ]
  }
}
```

## Easy mistakes

- Declaring every role just in case. Roles such as `merchant` or `shopper` tell other agents what they can ask of you.
