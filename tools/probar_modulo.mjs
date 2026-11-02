// Runs the real Claude Code module (hooks/verify-seo.ts) against the published Release, from an empty
// home folder: it has to download the binary, check it, register the tools and answer through them.
// Claude Code itself isn't involved, only the small part of its API that the module uses.
//   node tools/probar_modulo.mjs
import { spawnSync } from 'node:child_process'
import { existsSync, mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..')
const casa = mkdtempSync(join(tmpdir(), 'modulo-'))
process.env.HOME = casa
process.env.USERPROFILE = casa
delete process.env.VERIFY_SEO_BIN

const herramientas = []
const ordenes = []
const avisos = []

const $ = {
  plugin: { root: raiz },
  fs: { read: async f => readFileSync(f, 'utf8') },
  env: { get: async k => process.env[k] },
  ui: { toast: t => { avisos.push(t); console.log('toast:', t) } },
  tool: { register: async t => { herramientas.push(t) } },
  command: { register: async c => { ordenes.push(c) } },
  clock: { after: () => {} },
  prompt: { submit: async () => {} },
  http: {
    fetch: async url => {
      const r = await fetch(url)
      return { ok: r.ok, status: r.status, text: await r.text() }
    },
  },
  process: {
    run: async (argv, op = {}) => {
      const r = spawnSync(argv[0], argv.slice(1), { input: op.stdin, encoding: 'utf8', timeout: op.timeoutMs })
      if (r.error) throw r.error
      return { exitCode: r.status, stdout: r.stdout, stderr: r.stderr }
    },
  },
}

const ganchos = []
const { register } = await import(new URL('../hooks/verify-seo.ts', import.meta.url))
register((nombre, a, b) => ganchos.push({ nombre, filtro: b ? a : undefined, f: b ?? a }))

const gancho = (nombre, clave, valor) => ganchos.find(g => g.nombre === nombre && (clave === undefined || g.filtro?.[clave] === valor)).f
const comprobar = (bien, que) => {
  console.log(`${bien ? 'ok ' : 'MAL'}  ${que}`)
  if (!bien) process.exitCode = 1
}

await gancho('session.start')($, {}, async e => e)

const bin = join(casa, '.verify-seo', 'bin', process.platform === 'win32' ? 'verify-seo.exe' : 'verify-seo')
comprobar(existsSync(bin), `the binary was installed in ${bin}`)
comprobar(!avisos.some(t => t.includes("couldn't")), 'no install error')
comprobar(herramientas.map(t => t.name).join() === 'scan_site,get_fix_guide', 'both tools are registered')
comprobar(ordenes.some(c => c.name === 'verify-seo'), 'the /verify-seo command is registered')

const guia = await gancho('tool.call', 'tool', 'mcp__verify-seo-for-agents__get_fix_guide')($, { tool: 'x', tool_use_id: 'y', check: 'acp' })
comprobar(guia.result?.includes('acp.json'), 'get_fix_guide answers')

const escaneo = await gancho('tool.call', 'tool', 'mcp__verify-seo-for-agents__scan_site')($, { tool: 'x', tool_use_id: 'y', url: 'https://github.com', format: 'json' })
comprobar(JSON.parse(escaneo.result ?? '{}').checks?.discoverability?.robotsTxt?.status === 'pass', 'scan_site scans a real site')

const malo = await gancho('tool.call', 'tool', 'mcp__verify-seo-for-agents__get_fix_guide')($, { tool: 'x', tool_use_id: 'y', check: 'nope' })
comprobar(malo.deny !== undefined, 'a tool error comes back as an error')
