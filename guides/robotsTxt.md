# robots.txt

Agents read this file first to learn what they may fetch, and most of the checks about bots depend on it.

**Passes when** `/robots.txt` answers 200, is not HTML, and has at least one `User-agent` line.

```text file=/robots.txt
User-agent: *
Allow: /
Disallow: /admin/

Sitemap: https://example.com/sitemap.xml
```

## Easy mistakes

- A single-page app that answers every path with its HTML shell. That is a 200, but it is not a robots.txt.
- A file with only `Sitemap:` lines. Without a `User-agent` group it says nothing about crawling.
- Serving it only on `www`. It has to live on the same host as the page being scanned.
