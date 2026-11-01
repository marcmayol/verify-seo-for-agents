# DNS for AI Discovery (DNS-AID)

DNS-AID lets an agent find your endpoints before it has made a single HTTP request.

**Passes when** a query for `_index`, `_a2a` or `_mcp` under `_agents.<your domain>` returns a ServiceMode `SVCB` or `HTTPS` record, and the resolver validated it with DNSSEC. Both the host and, for `www` hosts, the bare domain are tried.

```dns
_mcp._agents.example.com. 3600 IN SVCB 1 mcp.example.com. alpn="h2" port=443
```

To confirm the answer yourself:

```sh
curl -H "accept: application/dns-json" "https://dns.google/resolve?name=_mcp._agents.example.com&type=SVCB&do=1"
```

Look for `"AD": true` and an entry of type 64 in `Answer`.

## Easy mistakes

- Publishing the record in an unsigned zone. It resolves, yet it fails this check, because the point is that an agent can trust the answer.
- Using priority 0. That is AliasMode, which only points to another name. Discovery needs ServiceMode, so priority 1 or higher.
- Expecting a TXT record to be enough. Only SVCB and HTTPS records count.
