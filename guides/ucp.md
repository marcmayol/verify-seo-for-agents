# Universal Commerce Protocol

A profile that says how an agent can buy from your store: which services you offer and which versions you speak. Shopify stores may already serve one.

**Passes when** `/.well-known/ucp` is JSON with a version, in either `ucp.version` or `protocol_version`. Services are reported in the details.

```json file=/.well-known/ucp
{
  "ucp": {
    "version": "2026-08-25",
    "services": {
      "dev.ucp.shopping": [
        { "version": "2026-08-25", "transport": "rest", "endpoint": "https://example.com/ucp/v1" }
      ]
    },
    "capabilities": {}
  }
}
```

Take the current version and the service names from the UCP specification, since it changes quickly.

## Easy mistakes

- Treating a pass as complete. This check confirms the profile exists, yet an agent still needs your services to work.
