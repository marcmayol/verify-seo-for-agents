# OAuth Protected Resource

This is the other half of OAuth discovery. It tells an agent which authorization server issues tokens for this API.

**Passes when** the metadata has a `resource`. An agent also needs `authorization_servers` to know where to get a token, so include it whenever you use OAuth. It is looked for in three places, in this order: the `resource_metadata` of a `WWW-Authenticate` header on the page, the well-known path with the page's path added, and `/.well-known/oauth-protected-resource`.

```json file=/.well-known/oauth-protected-resource
{
  "resource": "https://example.com",
  "authorization_servers": ["https://example.com"],
  "scopes_supported": ["read", "write"],
  "bearer_methods_supported": ["header"]
}
```

A protected route should also point to it when it refuses a request:

```http
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer resource_metadata="https://example.com/.well-known/oauth-protected-resource"
```

## Easy mistakes

- A `resource` value that does not match the API's address. RFC 9728 clients have to reject that mismatch.
