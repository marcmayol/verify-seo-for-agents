"""Web local que implementa todos los estándares, para probar los casos positivos.

Uso: python -I tools/web_prueba.py [puerto]
"""
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8799
H = f"http://localhost:{PORT}"

ROBOTS = f"""User-agent: *
Content-Signal: search=yes, ai-input=yes, ai-train=no
Allow: /

User-agent: GPTBot
Disallow: /

Sitemap: {H}/sitemap.xml
"""

HTML = """<!DOCTYPE html><html><head><title>Prueba</title></head><body>
<a href="/cart/">Cart</a>
<script>document.modelContext.registerTool({name:"buscar",description:"Busca",inputSchema:{type:"object"},execute(){}});</script>
<form toolname="contacto" tooldescription="Contactar"><input name="q"></form>
</body></html>"""

AUTH_MD = "# auth.md\n\nYou are an agent. Register with POST /agent/register and receive an api_key credential.\n"

JSONS = {
    "/.well-known/http-message-signatures-directory": {"keys": [{"kty": "OKP", "crv": "Ed25519", "x": "abc"}]},
    "/.well-known/mcp/server-card.json": {"serverInfo": {"name": "prueba", "version": "1"}, "transport": {"type": "streamable-http"}},
    "/.well-known/agent-card.json": {
        "name": "prueba", "version": "1", "supportedInterfaces": [{"url": H, "protocolBinding": "JSONRPC"}],
        "capabilities": {"extensions": [{"uri": "https://ap2-protocol.org/ap2", "params": {"roles": ["merchant"]}}]},
    },
    "/.well-known/agent-skills/index.json": {"$schema": "x", "skills": [{"name": "a", "url": "/a"}]},
    "/.well-known/api-catalog": {"linkset": [{"anchor": H + "/api", "service-desc": [{"href": H + "/openapi.json"}]}]},
    "/.well-known/oauth-authorization-server": {
        "issuer": H, "authorization_endpoint": H + "/a", "token_endpoint": H + "/t",
        "agent_auth": {"register_uri": H + "/agent/register"}, "poppy_domains": ["localhost"],
    },
    "/.well-known/oauth-protected-resource": {"resource": H, "authorization_servers": [H]},
    "/.well-known/ai-catalog.json": {
        "specVersion": "1.0",
        "entries": [{"identifier": "urn:air:localhost:s:a", "displayName": "A", "type": "application/json", "url": H + "/a"}],
    },
    "/.well-known/poppy.json": {
        "protocol_version": "0.1", "organization": {"name": "P", "domain": "localhost"},
        "apis": [{"type": "rest", "url": H, "description": "d"}], "auth": {"issuer": H},
    },
    "/openapi.json": {"openapi": "3.1.0", "paths": {"/pago": {"post": {"x-payment-info": {"price": 1}}}}},
    "/.well-known/ucp": {"ucp": {"version": "2026-08-25", "services": {}}},
    "/.well-known/acp.json": {
        "protocol": {"name": "acp", "version": "1"}, "api_base_url": H, "transports": ["rest"], "capabilities": {"services": ["checkout"]},
    },
}


class Web(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def enviar(self, code, tipo, cuerpo, extra=None):
        b = cuerpo.encode()
        self.send_response(code)
        self.send_header("Content-Type", tipo)
        self.send_header("Content-Length", str(len(b)))
        for k, v in (extra or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(b)

    def do_GET(self):
        p = self.path.split("?")[0]
        if p == "/":
            if "text/markdown" in self.headers.get("Accept", ""):
                return self.enviar(200, "text/markdown; charset=utf-8", "# Prueba\n")
            link = '</.well-known/api-catalog>; rel="api-catalog", </docs>; rel="service-doc"'
            return self.enviar(200, "text/html; charset=utf-8", HTML, {"Link": link})
        if p == "/robots.txt":
            return self.enviar(200, "text/plain", ROBOTS)
        if p == "/sitemap.xml":
            return self.enviar(200, "application/xml", '<?xml version="1.0"?><urlset></urlset>')
        if p == "/auth.md":
            return self.enviar(200, "text/markdown", AUTH_MD)
        if p == "/api":
            return self.enviar(402, "application/json", json.dumps({"x402Version": 1, "accepts": []}))
        if p in JSONS:
            tipo = "application/linkset+json" if p.endswith("api-catalog") else "application/json"
            return self.enviar(200, tipo, json.dumps(JSONS[p]))
        return self.enviar(404, "text/plain", "no")


ThreadingHTTPServer(("127.0.0.1", PORT), Web).serve_forever()
