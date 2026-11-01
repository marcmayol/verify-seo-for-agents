// agent-ready as a Claude Code mod. Installing the plugin is the whole setup: on the first session this
// module downloads the binary that matches the plugin's version (checking its SHA-256) into
// ~/.agent-ready/bin. It serves the tools by running `agent-ready tool <name>`, which answers exactly as
// the MCP server does. No shell is involved anywhere, so it works the same on Windows, macOS and Linux.
import type { Register } from 'claude-code'

const REPO = 'marcmayol/agent-ready'

// What `/agent-ready` asks the agent to do. The other agents' command has the same text (src/setup.rs).
const SCAN = 'Scan the site the user wrote after this command with the `scan_site` tool. If they gave none, use the site of the current project, running it locally if that is how it works. Report the level, the score and what is missing for the next level. If the project is the site\'s source code, offer to fix those checks: read `get_fix_guide` for each, apply it, and scan again to confirm. Skip the checks that make no sense for this site, such as a payment protocol on a site that sells nothing, and say why.'

// A scan runs a browser for one of its checks, so it can take a while.
const TOOL_TIMEOUT_MS = 120_000

let listo: Promise<string | undefined> | undefined

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    const bin = await ready($)
    if (bin !== undefined) await registerTools($, bin)
    await $.command.register({ name: 'agent-ready', description: 'Check whether a website is ready for AI agents, and fix what fails', argumentHint: '[url]' })
    return next(e)
  })

  // A command.run hook can't submit a prompt itself (it would wait on its own turn), so a timer does it
  // right after; the prompt runs as a turn of its own once the session is idle.
  on('command.run', { command: 'agent-ready' }, ($, e) => {
    const lo = e.args.trim()
    const text = lo === '' ? SCAN : `${SCAN}\n\nWhat the user wrote after the command: ${lo}`
    $.clock.after(0, () => {
      void $.prompt.submit({ text }).catch(() => {})
    })
    return { text: 'agent-ready: scanning.' }
  })

  on('tool.call', { tool: 'mcp__agent-ready__scan_site' }, ($, e) => runTool($, 'scan_site', e))
  on('tool.call', { tool: 'mcp__agent-ready__get_fix_guide' }, ($, e) => runTool($, 'get_fix_guide', e))
}

// Whichever hook comes first starts the preparation; the others wait for the same one.
function ready($: any): Promise<string | undefined> {
  if (listo === undefined) listo = prepare($)
  return listo
}

// The binary to use, installing it when it's missing or of another version.
async function prepare($: any): Promise<string | undefined> {
  try {
    const manifiesto = JSON.parse(await $.fs.read($.plugin.root + '/.claude-plugin/plugin.json'))
    const version: string = manifiesto.version
    const windows = (await $.env.get('OS')) === 'Windows_NT'
    const home = (windows ? await $.env.get('USERPROFILE') : await $.env.get('HOME')) ?? ''
    const carpeta = home + '/.agent-ready/bin'
    const propio = carpeta + (windows ? '/agent-ready.exe' : '/agent-ready')

    const elegido = (await $.env.get('AGENT_READY_BIN')) ?? undefined
    if (elegido !== undefined) return elegido
    for (const c of [propio, 'agent-ready']) {
      if ((await versionOf($, c)) === version) return c
    }

    $.ui.toast(`agent-ready: installing ${version}…`)
    await install($, version, windows, carpeta)
    if ((await versionOf($, propio)) !== version) throw new Error(`the installed binary isn't ${version}`)
    $.ui.toast(`agent-ready: ${version} installed`)
    return propio
  } catch (err) {
    $.ui.toast(`agent-ready couldn't start: ${String((err as Error)?.message ?? err).slice(0, 200)}`, { timeoutMs: 15_000 })
    return undefined
  }
}

async function versionOf($: any, bin: string): Promise<string | undefined> {
  try {
    const r = await $.process.run([bin, '--version'])
    return r.exitCode === 0 ? r.stdout.trim().split(/\s+/)[1] : undefined
  } catch {
    return undefined
  }
}

// Downloads the release package for this machine, checks it against SHA256SUMS and unpacks it.
async function install($: any, version: string, windows: boolean, carpeta: string): Promise<void> {
  let destino: string
  if (windows) {
    destino = 'x86_64-pc-windows-msvc'
  } else {
    const u = await $.process.run(['uname', '-sm'])
    const [so, maquina] = u.stdout.trim().split(/\s+/)
    const tabla: Record<string, string> = {
      'Linux x86_64': 'x86_64-unknown-linux-musl',
      'Linux aarch64': 'aarch64-unknown-linux-musl',
      'Linux arm64': 'aarch64-unknown-linux-musl',
      'Darwin arm64': 'aarch64-apple-darwin',
      'Darwin x86_64': 'x86_64-apple-darwin',
    }
    destino = tabla[`${so} ${maquina}`]
    if (destino === undefined) throw new Error(`no prebuilt binary for ${so} ${maquina}; build it with: cargo install --git https://github.com/${REPO}`)
  }
  const paquete = `agent-ready-${destino}.${windows ? 'zip' : 'tar.gz'}`
  const base = `https://github.com/${REPO}/releases/download/v${version}`

  const sumas = await $.http.fetch(`${base}/SHA256SUMS`)
  if (!sumas.ok) throw new Error(`can't download SHA256SUMS (HTTP ${sumas.status})`)
  const esperada = sumas.text
    .split('\n')
    .map((l: string) => l.trim().split(/\s+/))
    .find((p: string[]) => p[1] === paquete)?.[0]
  if (esperada === undefined) throw new Error(`${paquete} isn't in SHA256SUMS`)

  if (windows) await $.process.run(['cmd', '/c', 'mkdir', carpeta.replaceAll('/', '\\')])
  else await $.process.run(['mkdir', '-p', carpeta])
  const archivo = `${carpeta}/${paquete}`
  const bajada = await $.process.run(['curl', '-fsSL', '--retry', '2', '-o', archivo, `${base}/${paquete}`], { timeoutMs: 300_000 })
  if (bajada.exitCode !== 0) throw new Error(`can't download ${paquete}: ${bajada.stderr.trim()}`)

  const real = await sha256($, archivo, windows)
  if (real !== esperada.toLowerCase()) {
    await remove($, archivo, windows)
    throw new Error(`the SHA-256 of ${paquete} doesn't match; nothing was installed`)
  }

  // Windows ships bsdtar, which unpacks zip too; Git's own tar earlier on the PATH doesn't.
  const tar = windows ? ((await $.env.get('SystemRoot')) ?? 'C:\\Windows') + '\\System32\\tar.exe' : 'tar'
  const abrir = await $.process.run([tar, '-xf', archivo, '-C', carpeta])
  await remove($, archivo, windows)
  if (abrir.exitCode !== 0) throw new Error(`can't unpack ${paquete}: ${abrir.stderr.trim()}`)
}

async function sha256($: any, archivo: string, windows: boolean): Promise<string> {
  if (windows) {
    const r = await $.process.run(['certutil', '-hashfile', archivo.replaceAll('/', '\\'), 'SHA256'])
    // The second line holds the hash (older Windows put spaces between the bytes).
    return (r.stdout.split(/\r?\n/)[1] ?? '').replace(/\s+/g, '').toLowerCase()
  }
  for (const orden of [['sha256sum', archivo], ['shasum', '-a', '256', archivo]]) {
    try {
      const r = await $.process.run(orden)
      if (r.exitCode === 0) return r.stdout.trim().split(/\s+/)[0].toLowerCase()
    } catch {
      // Not on this system: try the next one.
    }
  }
  throw new Error('neither sha256sum nor shasum is available to check the download')
}

async function remove($: any, archivo: string, windows: boolean): Promise<void> {
  try {
    if (windows) await $.process.run(['cmd', '/c', 'del', '/q', archivo.replaceAll('/', '\\')])
    else await $.process.run(['rm', '-f', archivo])
  } catch {
    // A leftover package is harmless.
  }
}

// Registers the tools as `agent-ready tool` lists them, so names, descriptions and schemas have one source.
async function registerTools($: any, bin: string): Promise<void> {
  const r = await $.process.run([bin, 'tool'])
  for (const t of JSON.parse(r.stdout).tools) {
    await $.tool.register({ name: t.name, description: t.description, inputSchema: t.inputSchema })
  }
}

async function runTool($: any, nombre: string, e: any) {
  const bin = await ready($)
  if (bin === undefined) return { deny: 'agent-ready is not ready: its binary could not be installed. See the toast at session start.' }
  const { tool, tool_use_id, agentId, ...args } = e
  const r = await $.process.run([bin, 'tool', nombre], { stdin: JSON.stringify(args), timeoutMs: TOOL_TIMEOUT_MS })
  const texto = (r.stdout.trim() || r.stderr.trim() || '(no output)') as string
  return r.exitCode === 0 ? { result: texto } : { deny: texto }
}
