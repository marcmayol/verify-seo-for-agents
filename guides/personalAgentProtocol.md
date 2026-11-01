# Personal Agent Protocol

A draft for declaring how a person's own agent can act for them on your site. This check is informational: it never changes your level or percentage.

**Passes when** `/.well-known/poppy.json` has an `organization.domain` matching the host (`www.` aside) and declares an interface. If you declare `agent`, `apis` or `web.browser_session_endpoint`, you also need `auth.issuer`, and that issuer's OAuth metadata must have the same `issuer` and list your domain in `poppy_domains`.

The simplest valid form, for a plain website:

```json file=/.well-known/poppy.json
{
  "protocol_version": "0.1",
  "organization": { "name": "Example", "domain": "example.com" },
  "web": {}
}
```

## Easy mistakes

- Using your hosting provider's domain in `organization.domain`. It has to be your own, even after a redirect.
