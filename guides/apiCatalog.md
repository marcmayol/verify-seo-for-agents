# API catalog

One well-known address lists all your APIs, so an agent does not have to hunt through your documentation for them.

**Passes when** `/.well-known/api-catalog` answers with JSON holding a non-empty `linkset` array. The right content type, `application/linkset+json`, is reported in the details but does not decide the result.

```json file=/.well-known/api-catalog
{
  "linkset": [
    {
      "anchor": "https://example.com/api/v1",
      "service-desc": [{ "href": "https://example.com/openapi.json", "type": "application/json" }],
      "service-doc": [{ "href": "https://example.com/docs/api", "type": "text/html" }],
      "status": [{ "href": "https://example.com/api/v1/health" }]
    }
  ]
}
```

## Easy mistakes

- Linking to a description that does not load. Agents follow `service-desc`, so test that URL too.
- Serving it only to clients that ask for `application/linkset+json`. Many send `application/json`.
