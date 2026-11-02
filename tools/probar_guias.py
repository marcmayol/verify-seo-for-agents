"""Comprueba las guías: cada ejemplo tiene que aprobar su propio check.

Cada bloque de código con file=/ruta se sirve en una web local que solo contiene los
ficheros de esa guía, y se escanea ese único check.

Uso: python -I tools/probar_guias.py [ruta del binario]
"""
import glob
import json
import os
import re
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NOMBRE = "verify-seo.exe" if os.name == "nt" else "verify-seo"
BIN = sys.argv[1] if len(sys.argv) > 1 else os.path.join(BASE, "target", "release", NOMBRE)
PORT = 8798
sys.stdout.reconfigure(encoding="utf-8")

SIN_PRUEBA_LOCAL = {"dnsAid"}  # Necesita DNS real con DNSSEC.
BLOQUE = re.compile(r"^```(\w+) file=(\S+)(?: accept=(\S+))?\n(.*?)^```", re.M | re.S)
TIPOS = {"text": "text/plain", "xml": "application/xml", "html": "text/html; charset=utf-8", "markdown": "text/markdown", "json": "application/json"}
sitio = {}


def localizar(texto):
    return texto.replace("https://example.com", f"http://localhost:{PORT}").replace("example.com", "localhost")


def leer_http(crudo):
    cabecera, _, cuerpo = crudo.partition("\n\n")
    lineas = cabecera.splitlines()
    estado = int(lineas[0].split()[1])
    cabeceras = [tuple(x.strip() for x in l.split(":", 1)) for l in lineas[1:]]
    return estado, cabeceras, cuerpo


class Web(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def do_GET(self):
        ruta = self.path.split("?")[0]
        acepta = self.headers.get("Accept", "")
        opciones = sitio.get(ruta, [])
        elegido = next((o for o in opciones if o[0] and o[0] in acepta), None) or next((o for o in opciones if not o[0]), None)
        if elegido is None and ruta == "/":
            elegido = (None, "html", "<!DOCTYPE html><title>Prueba</title>")
        if elegido is None:
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        _, lengua, cuerpo = elegido
        if lengua == "http":
            estado, cabeceras, cuerpo = leer_http(cuerpo)
        else:
            estado, cabeceras = 200, [("Content-Type", TIPOS.get(lengua, "application/json"))]
        datos = cuerpo.encode()
        self.send_response(estado)
        for k, v in cabeceras:
            self.send_header(k, v)
        self.send_header("Content-Length", str(len(datos)))
        self.end_headers()
        self.wfile.write(datos)


servidor = ThreadingHTTPServer(("127.0.0.1", PORT), Web)
threading.Thread(target=servidor.serve_forever, daemon=True).start()

mal = 0
guias = {os.path.basename(f)[:-3]: open(f, encoding="utf-8").read() for f in glob.glob(os.path.join(BASE, "guides", "*.md"))}
if len(guias) != 27:
    sys.exit(f"Esperaba 27 guías y hay {len(guias)}")
print("--- Ejemplos")
for clave, texto in sorted(guias.items()):
    if clave.startswith("_") or clave in SIN_PRUEBA_LOCAL:
        continue
    sitio.clear()
    for lengua, ruta, acepta, cuerpo in BLOQUE.findall(texto):
        sitio.setdefault(ruta, []).append((acepta, lengua, localizar(cuerpo)))
    if not sitio:
        print(f"MAL  {clave}: sin ejemplo que probar")
        mal += 1
        continue
    p = subprocess.run([BIN, "scan", f"http://localhost:{PORT}", "--json", "--checks", clave], capture_output=True, encoding="utf-8", timeout=90)
    if p.returncode:
        print(f"MAL  {clave}: {p.stderr.strip()}")
        mal += 1
        continue
    informe = json.loads(p.stdout)
    r = next(c[clave] for c in informe["checks"].values() if clave in c)
    if r["status"] != "pass":
        mal += 1
    print(f"{'ok ' if r['status'] == 'pass' else 'MAL'}  {clave}: {r['message']}")


servidor.shutdown()
print(f"\n{'TODO BIEN' if not mal else str(mal) + ' GUÍAS CON ERRORES'}")
sys.exit(1 if mal else 0)
