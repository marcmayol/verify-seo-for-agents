# verify-seo-for-agents

El comando es `verify-seo`.

[English](README.md)

Comprueba en tu propio ordenador si una web está preparada para agentes de IA: 23 comprobaciones, un nivel de 0 a 5, un porcentaje y una guía para arreglar cada fallo. También escanea `localhost`, así que puedes probar una web antes de publicarla.

Es un binario pequeño para Windows, macOS y Linux, sin nada más que instalar. Funciona en la terminal, en el navegador, como API HTTP y como servidor MCP para tu agente.

## Instalar

**Claude Code.** Basta con instalar el plugin, que descarga el binario la primera vez que se usa.

```
/plugin marketplace add marcmayol/drac-plugins
/plugin install verify-seo-for-agents@drac
```

**Otros agentes, o solo el comando.** Esto instala el binario y prepara todos los agentes que encuentre en tu ordenador: Codex, Gemini CLI, Cursor, OpenCode, Qwen Code, Kiro, Copilot CLI, Windsurf, Antigravity, Kimi Code, Pi, Amp, Kilo Code y Grok.

```sh
# Linux y macOS
curl -fsSL https://raw.githubusercontent.com/marcmayol/verify-seo-for-agents/main/install.sh | sh
```

```powershell
# Windows
irm https://raw.githubusercontent.com/marcmayol/verify-seo-for-agents/main/install.ps1 | iex
```

Para un agente que instales más tarde: `verify-seo setup <agente>`. Cada fichero que toca guarda una copia `.before-verify-seo`.

## Usar

En tu agente, pídelo con tus palabras o escribe `/verify-seo https://example.com`. Escanea la web y, si estás en su código, se ofrece a arreglar lo que falla.

En la terminal:

```
verify-seo scan example.com                 # resumen
verify-seo scan localhost:3000              # tu web antes de publicarla
verify-seo scan example.com --json          # informe completo
verify-seo scan example.com --agent         # Markdown con lo que hay que arreglar
verify-seo scan example.com --profile content
verify-seo scan example.com --min-level 3   # sale con 2 por debajo de ese nivel, para CI
verify-seo guide mcpServerCard              # cómo aprobar una comprobación
verify-seo docs                             # toda la documentación
verify-seo serve                            # interfaz web en http://127.0.0.1:8787
```

Perfiles: `all`, `content` para un blog o documentación, y `apiApp` para una API o aplicación web.

## Qué comprueba

| Categoría | Comprobaciones |
|---|---|
| Descubrimiento básico | robots.txt, sitemap, cabeceras Link, DNS-AID |
| Contenido | Negociación de Markdown. Opcionales, si se piden: `llms.txt` y `llms-full.txt` |
| Acceso de bots | Reglas para bots de IA, Content Signals, Web Bot Auth |
| Descubrimiento de servicios | Catálogo de API, OAuth discovery, OAuth Protected Resource, Auth.md, MCP Server Card, A2A Agent Card, Agent Skills, WebMCP, ARD, Personal Agent Protocol |
| Comercio | x402, MPP, UCP, ACP, AP2. Solo cuentan en webs detectadas como tiendas |

No todas tienen sentido en todas las webs. Un blog no necesita login, y una web que no vende nada no necesita un protocolo de pago.

Cada comprobación tiene una guía en `guides/` (en inglés) con lo que busca, un ejemplo que funciona y los errores típicos. El binario las lleva dentro y genera a partir de ellas los `SKILL.md`, el índice de skills, `llms.txt` y `llms-full.txt`, que `serve` publica en las direcciones habituales.

## Qué necesita

Nada para escanear, salvo en una comprobación: WebMCP ejecuta la página, así que necesita Chrome, Edge, Brave o Chromium. Sin ninguno sigue encontrando las herramientas declaradas en el HTML, y si no las hay responde `unableToCheck`. Con `VERIFY_SEO_BROWSER` eliges el navegador.

DNS-AID y ARD preguntan al resolutor DNS-over-HTTPS de Cloudflare, y al de Google si ese falla. Aparte de eso, de tu ordenador solo salen las peticiones a la web que escaneas.

## Seguridad

`serve` escucha en `127.0.0.1` y rechaza peticiones de otras webs, porque descarga cualquier URL que le pidan. Con `--host 0.0.0.0`, cualquiera que llegue a él puede hacer que tu ordenador pida URLs de tu red, así que úsalo solo en una red de confianza.

El escaneo carga la página en el sandbox de tu navegador, con un perfil temporal que se borra al terminar.

## Límites

- x402 se detecta por las respuestas de la propia web, así que una API de pago en rutas distintas de `/api` o `/api/v1` no se encuentra.
- La comprobación de MCP valida la tarjeta y no prueba el endpoint.
- `llms.txt` y `llms-full.txt` van apagadas por defecto: `verify-seo scan example.com --checks llmsTxt,llmsFullTxt`.
- Qué portadas cuentan como tienda y qué necesita Auth.md sin metadatos OAuth son heurísticas. DNS-AID solo se ha probado contra el formato de respuesta DNS.

## Desarrollo

```
cargo test --release
cargo build --release
python3 -I tools/probar_guias.py             # el ejemplo de cada guía tiene que aprobar su comprobación
python3 -I tools/web_prueba.py 8799          # web local que implementa todos los estándares
```

## Licencia

MIT
