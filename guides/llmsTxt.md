# llms.txt

A short Markdown map of your site, written for language models: what the site is and which pages matter. It saves an agent from crawling to find them.

This check is an extra. It only runs when you ask for it by name, and it never changes the level or the percentage.

**Passes when** `/llms.txt` answers 200, is not an HTML page, and has an H1 title.

```text file=/llms.txt
# Example

> Example sells handmade bicycle parts and ships them across Europe.

Orders placed before noon leave the same day.

## Docs

- [Shipping](https://example.com/shipping.md): prices and delivery times
- [Returns](https://example.com/returns.md): how to send a part back

## Optional

- [Our story](https://example.com/about.md)
```

The title is the only part this check requires. The summary in a quote, and the sections of links with a line on each, are what make the file useful.

## Easy mistakes

- Pasting your sitemap into it. A sitemap lists everything, and this file should pick what is worth reading.
- A single-page app answering `/llms.txt` with its HTML shell. That is a 200, but it is not this file.
- Linking to pages an agent cannot read. Where you can, link to a Markdown version.
