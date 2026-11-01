# Machine Payment Protocol

MPP marks which operations of your API cost money, inside the OpenAPI document that you may already publish.

**Passes when** `/openapi.json` has `x-payment-info` on at least one operation.

```json file=/openapi.json
{
  "openapi": "3.1.0",
  "info": { "title": "Example API", "version": "1.0.0" },
  "paths": {
    "/reports": {
      "post": {
        "summary": "Generate a report",
        "x-payment-info": { "intent": "charge", "method": "stripe", "amount": "500", "currency": "usd" },
        "responses": { "200": { "description": "The report" } }
      }
    }
  }
}
```

## Easy mistakes

- Placing `x-payment-info` at the path level. It belongs on the operation, next to `summary`.
- Publishing the document only as YAML or at `/docs/openapi.json`. This check reads `/openapi.json`.
