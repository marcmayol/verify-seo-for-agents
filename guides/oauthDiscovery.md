# OAuth/OIDC discovery

This metadata lets an agent find out how to log in without any instructions written for humans. If nothing on your site needs a login, you can ignore this check.

**Passes when** `/.well-known/openid-configuration` or `/.well-known/oauth-authorization-server` answers with JSON that has an `issuer` and an authorization or token endpoint.

```json file=/.well-known/oauth-authorization-server
{
  "issuer": "https://example.com",
  "authorization_endpoint": "https://example.com/oauth/authorize",
  "token_endpoint": "https://example.com/oauth/token",
  "grant_types_supported": ["authorization_code", "refresh_token"],
  "code_challenge_methods_supported": ["S256"]
}
```

## Easy mistakes

- An `issuer` that differs from the address serving the document. Strict clients reject it, even though this check does not compare them.
- Leaving out `code_challenge_methods_supported`. Agents are public clients, and they need PKCE.
