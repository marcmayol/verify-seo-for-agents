# Content Signals

`robots.txt` rules say who may fetch a page. Content Signals say what may be done with it afterwards.

**Passes when** `robots.txt` has a `Content-Signal` line with at least one of these preferences:

| Signal | It covers |
|---|---|
| `search` | Building a search index and showing links to you |
| `ai-input` | Using the page live as a source for an AI answer |
| `ai-train` | Training or tuning models |

```text file=/robots.txt
User-agent: *
Content-Signal: search=yes, ai-input=yes, ai-train=no
Allow: /
```

## Easy mistakes

- Placing the line outside a `User-agent` group, so that it applies to nobody in particular.
- Reading it as enforcement. A signal states a preference, and it is up to each bot to respect it.
