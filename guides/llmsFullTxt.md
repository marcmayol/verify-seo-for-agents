# llms-full.txt

The content itself in one Markdown file, so an agent can read your documentation with a single request instead of following links. It suits documentation and APIs more than a shop or a blog.

This check is an extra. It only runs when you ask for it by name, and it never changes the level or the percentage.

**Passes when** `/llms-full.txt` answers 200, is not an HTML page, and has an H1 title.

```markdown file=/llms-full.txt
# Example API

> Everything you need to use the Example API.

## Authentication

Send your key in the `Authorization` header as a bearer token.

## Create an order

`POST /v1/orders` with a JSON body holding `sku` and `quantity`. It answers with the new order and its `id`.
```

## Easy mistakes

- Writing it by hand. Generate it on deploy from the same sources as your documentation, or it will fall behind.
- Letting it grow without limit. An agent has to fit it in its context, so leave out changelogs and repeated pages.
- Publishing it with no `llms.txt`. Agents usually look there first, so link to this file from it.
