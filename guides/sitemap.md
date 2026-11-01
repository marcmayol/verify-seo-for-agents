# Sitemap

A sitemap gives an agent the list of canonical pages, so it does not have to crawl to find them.

**Passes when** a sitemap named in `robots.txt` loads, or one exists at a usual path: `/sitemap.xml`, `/sitemap-index.xml`, `/sitemap_index.xml`, `/sitemap.xml.gz` or `/wp-sitemap.xml`. The document needs a `urlset` or `sitemapindex` root.

```xml file=/sitemap.xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/</loc></url>
  <url><loc>https://example.com/pricing</loc></url>
</urlset>
```

## Easy mistakes

- Using a custom filename without naming it in `robots.txt`. Add `Sitemap: https://example.com/my-map.xml`, since nothing can guess it.
- Letting it go stale. Generate it on deploy, so that removed pages leave with the release.
