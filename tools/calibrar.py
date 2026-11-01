"""Compara el escáner local con las respuestas guardadas del original (spec/ref).

Uso: python -I tools/calibrar.py [ruta del binario]
"""
import glob
import json
import os
import subprocess
import sys

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = sys.argv[1] if len(sys.argv) > 1 else os.path.join(BASE, "target", "release", "agent-ready.exe")
sys.stdout.reconfigure(encoding="utf-8")

total = iguales = 0
fallos = {}
for ruta in sorted(glob.glob(os.path.join(BASE, "spec", "ref", "*.json"))):
    ref = json.load(open(ruta, encoding="utf-8"))
    sitio = os.path.basename(ruta)[:-5].replace("_", ".")
    p = subprocess.run([BIN, "scan", "https://" + sitio, "--json"], capture_output=True, encoding="utf-8", timeout=180)
    if p.returncode != 0:
        print(f"{sitio}: ERROR {p.stderr.strip()}")
        continue
    mio = json.loads(p.stdout)
    if "siteError" in ref or "siteError" in mio:
        a, b = ref.get("siteError", {}).get("httpStatus"), mio.get("siteError", {}).get("httpStatus")
        print(f"{sitio:28} siteError {b} / {a}")
        if "checks" not in ref or "checks" not in mio:
            continue
    marca = "==" if mio["level"] == ref["level"] else "!="
    linea = f"{sitio:28} nivel {mio['level']} {marca} {ref['level']}  comercio {mio['isCommerce']}/{ref['isCommerce']}"
    difs = []
    for cat, checks in ref["checks"].items():
        for clave, r in checks.items():
            m = mio["checks"].get(cat, {}).get(clave)
            total += 1
            if m and m["status"] == r["status"]:
                iguales += 1
                continue
            difs.append(clave)
            fallos.setdefault(clave, []).append(
                f"{sitio}: mío={m and m['status']} [{m and m['message']}] | original={r['status']} [{r['message']}]"
            )
    if set(mio["commerceSignals"]) != set(ref["commerceSignals"]):
        difs.append("señales[" + ",".join(sorted(set(mio["commerceSignals"]) ^ set(ref["commerceSignals"]))) + "]")
    print(linea + ("  dif: " + ", ".join(difs) if difs else ""))

print(f"\n{iguales}/{total} checks iguales\n")
for clave, lista in sorted(fallos.items()):
    print(clave)
    for x in lista:
        print("  " + x[:300])
