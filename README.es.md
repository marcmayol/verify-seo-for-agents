# agent-ready

[English](README.md)

Comprueba en tu propio ordenador si una web está preparada para agentes de IA: 23 comprobaciones, un nivel de 0 a 5, un porcentaje y una guía para arreglar cada fallo. También escanea `localhost`, así que puedes probar una web antes de publicarla.

Es un binario pequeño para Windows, macOS y Linux, sin nada más que instalar. Funciona en la terminal, en el navegador, como API HTTP y como servidor MCP para tu agente.

Es una herramienta independiente que sigue el método público de [isitagentready.com](https://isitagentready.com) y no tiene relación con Cloudflare. Sus resultados se compararon con los de esa web en 26 sitios y coincidieron en 597 de 598 comprobaciones.

## Instalar

**Claude Code.** Basta con instalar el plugin, que descarga el binario la primera vez que se usa.

```
/plugin marketplace add marcmayol/drac-plugins
/plugin install agent-ready@drac
```

**Otros agentes, o solo el comando.** Esto instala el binario y prepara todos los agentes que encuentre en tu ordenador: Codex, Gemini CLI, Cursor, OpenCode, Qwen Code, Kiro, Copilot CLI, Windsurf, Antigravity, Kimi Code, Pi, Amp, Kilo Code y Grok.

```sh
# Linux y macOS
curl -fsSL https://raw.githubusercontent.com/marcmayol/agent-ready/main/install.sh | sh
```

```powershell
# Windows
irm https://raw.githubusercontent.com/marcmayol/agent-ready/main/install.ps1 | iex
```

Para un agente que instales más tarde: `agent-ready setup <agente>`. Cada fichero que toca guarda una copia `.before-agent-ready`.

## Usar

En tu agente, pídelo con tus palabras o escribe `/agent-ready https://example.com`. Escanea la web y, si estás en su código, se ofrece a arreglar lo que falla.

En la terminal:

```
agent-ready scan example.com                 # resumen
agent-ready scan localhost:3000              # tu web antes de publicarla
agent-ready scan example.com --json          # informe completo
agent-ready scan example.com --agent         # Markdown con lo que hay que arreglar
agent-ready scan example.com --profile content
agent-ready scan example.com --min-level 3   # sale con 2 por debajo de ese nivel, para CI
agent-ready guide mcpServerCard              # cómo aprobar una comprobación
agent-ready docs                             # toda la documentación
agent-ready serve                            # interfaz web en http://127.0.0.1:8787
```

Perfiles: `all`, `content` para un blog o documentación, y `apiApp` para una API o aplicación web.

## Qué comprueba

| Categoría | Comprobaciones |
|---|---|
| Descubrimiento básico | robots.txt, sitemap, cabeceras Link, DNS-AID |
| Contenido | Negociación de Markdown |
| Acceso de bots | Reglas para bots de IA, Content Signals, Web Bot Auth |
| Descubrimiento de servicios | Catálogo de API, OAuth discovery, OAuth Protected Resource, Auth.md, MCP Server Card, A2A Agent Card, Agent Skills, WebMCP, ARD, Personal Agent Protocol |
| Comercio | x402, MPP, UCP, ACP, AP2. Solo cuentan en webs detectadas como tiendas |

No todas tienen sentido en todas las webs. Un blog no necesita login, y una web que no vende nada no necesita un protocolo de pago.

Cada comprobación tiene una guía en `guides/` (en inglés) con lo que busca, un ejemplo que funciona y los errores típicos. El binario las lleva dentro y genera a partir de ellas los `SKILL.md`, el índice de skills, `llms.txt` y `llms-full.txt`, que `serve` publica en las direcciones habituales.

## Qué necesita

Nada para escanear, salvo en una comprobación: WebMCP ejecuta la página, así que necesita Chrome, Edge, Brave o Chromium. Sin ninguno sigue encontrando las herramientas declaradas en el HTML, y si no las hay responde `unableToCheck`. Con `AGENT_READY_BROWSER` eliges el navegador.

DNS-AID y ARD preguntan al resolutor DNS-over-HTTPS de Cloudflare, y al de Google si ese falla. Aparte de eso, de tu ordenador solo salen las peticiones a la web que escaneas.

## Seguridad

`serve` escucha en `127.0.0.1` y rechaza peticiones de otras webs, porque descarga cualquier URL que le pidan. Con `--host 0.0.0.0`, cualquiera que llegue a él puede hacer que tu ordenador pida URLs de tu red, así que úsalo solo en una red de confianza.

El escaneo carga la página en el sandbox de tu navegador, con un perfil temporal que se borra al terminar.

## Diferencias conocidas con el original

- Una web que solo aparece en el directorio x402 Bazaar no se detecta: esa consulta depende de un tercero.
- La comprobación de MCP valida la tarjeta y no prueba el endpoint.
- Las reglas del original se dedujeron de sus respuestas, porque su método publicado es menos preciso que su comportamiento, y esta herramienta sigue el comportamiento. Lo menos seguro es qué portadas cuentan como tienda y qué necesita Auth.md sin metadatos OAuth. DNS-AID solo se ha probado contra el formato de respuesta DNS, porque ninguna web comparada lo publica.

## Desarrollo

```
cargo test --release
cargo build --release
python3 -I tools/probar_guias.py             # el ejemplo de cada guía tiene que aprobar su comprobación
python3 -I tools/web_prueba.py 8799          # web local que implementa todos los estándares
python3 -I tools/calibrar.py                 # compara con respuestas guardadas del original
```

`calibrar.py` y la comparación de texto de `probar_guias.py` usan material de referencia en `spec/`, que es privado y queda fuera de git. Las guías se escriben a partir de los estándares y de lo que comprueba este código, nunca copiando ese material.

## Licencia

MIT
