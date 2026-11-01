# Link headers

A `Link` response header tells an agent where the machine-readable resources are, without parsing any HTML.

**Passes when** the homepage sends a `Link` header with at least one of these relations: `service-desc`, `service-doc`, `describedby`, `api-catalog`, or an `alternate` whose type is Markdown or JSON.

```http file=/
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Link: </.well-known/api-catalog>; rel="api-catalog", </docs/api>; rel="service-doc"

<!DOCTYPE html><title>Home</title>
```

## Easy mistakes

- Expecting `preload` or `preconnect` to count. They help browsers, and an agent can do nothing with them.
- Adding `alternate` links for other languages only. Those are still HTML, so they do not help here.
- Setting the header in the application while a CDN strips it. Check the response from outside.
