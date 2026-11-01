# Markdown negotiation

Markdown costs an agent far fewer tokens than the same page in HTML, and it drops the navigation and scripts around the content.

**Passes when** a request for the page with `Accept: text/markdown` answers with a 2xx status, `Content-Type: text/markdown` and a body that is not an HTML document. Browsers keep receiving HTML.

```http file=/ accept=text/markdown
HTTP/1.1 200 OK
Content-Type: text/markdown; charset=utf-8
Vary: Accept

# Home

What this site offers, as plain Markdown.
```

## Easy mistakes

- Forgetting `Vary: Accept`. Without it, a cache can hand the Markdown version to the next browser, or the reverse.
- Changing only the header. Labelling your HTML as Markdown does not make it Markdown, and this check looks at the body.
- Answering 406. A strict server refuses the header outright, so an agent gets nothing at all.
- Offering it only at `/index.md`. That is useful too, but a separate URL is not negotiation.
