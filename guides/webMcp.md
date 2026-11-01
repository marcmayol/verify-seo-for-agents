# WebMCP

WebMCP lets an agent working inside a browser call your site's actions directly, instead of guessing at buttons.

**Passes when** the loaded page registers a tool with `document.modelContext.registerTool()` (the older `navigator.modelContext` also counts), or has a form with a `toolname` attribute. This check runs the page in a browser. Without a Chromium browser installed it can only see forms, and otherwise it reports `unableToCheck`.

```html file=/
<!DOCTYPE html>
<title>Shop</title>
<form action="/search" toolname="search_products" tooldescription="Search the catalog by keyword">
  <input name="q" toolparamdescription="Words to look for">
</form>
<script>
  const context = document.modelContext || navigator.modelContext;
  context?.registerTool({
    name: "get_cart",
    description: "Return the items in the current cart",
    inputSchema: { type: "object", properties: {} },
    execute: async () => ({ content: [{ type: "text", text: "[]" }] }),
  });
</script>
```

## Easy mistakes

- Registering tools after a click. Only what appears within a few seconds of the page loading gets detected.
- Calling the API with no check. Most browsers do not have it yet, and an unguarded call breaks the rest of your script.
