import { computed } from 'vue'
import type { Router, Connection } from '../lib/types'
import { useRouters } from './useRouters'
import { useConnections } from './useConnections'

function routerToCliArgs(r: Router): string[] {
  const args: string[] = ['clasp-relay']
  const addr = r.address || 'localhost:7330'
  args.push('--listen', addr)
  if (r.name) args.push('--name', `"${r.name}"`)
  if (r.announce !== false) args.push('--announce')
  if (r.maxSessions) args.push('--max-sessions', String(r.maxSessions))
  if (r.sessionTimeout) args.push('--session-timeout', String(r.sessionTimeout))
  if (r.authEnabled) {
    args.push('--auth')
    if (r.authPort) args.push('--auth-port', String(r.authPort))
    if (r.authDb) args.push('--auth-db', r.authDb)
    if (r.adminTokenPath) args.push('--admin-token-path', r.adminTokenPath)
    if (r.tokenTtl) args.push('--token-ttl', String(r.tokenTtl))
    if (r.corsOrigin) args.push('--cors-origin', r.corsOrigin)
  }
  if (r.quicEnabled) {
    args.push('--quic')
    if (r.quicPort) args.push('--quic-port', String(r.quicPort))
    if (r.certPath) args.push('--cert', r.certPath)
    if (r.keyPath) args.push('--key', r.keyPath)
  }
  if (r.mqttBridgeEnabled) {
    args.push('--mqtt')
    if (r.mqttBridgePort) args.push('--mqtt-port', String(r.mqttBridgePort))
    if (r.mqttBridgeNamespace) args.push('--mqtt-namespace', r.mqttBridgeNamespace)
  }
  if (r.oscBridgeEnabled) {
    args.push('--osc')
    if (r.oscBridgePort) args.push('--osc-port', String(r.oscBridgePort))
    if (r.oscBridgeNamespace) args.push('--osc-namespace', r.oscBridgeNamespace)
  }
  if (r.noTtl) {
    args.push('--no-ttl')
  } else {
    if (r.paramTtl) args.push('--param-ttl', String(r.paramTtl))
    if (r.signalTtl) args.push('--signal-ttl', String(r.signalTtl))
  }
  if (r.persistEnabled) {
    args.push('--persist')
    if (r.persistPath) args.push('--persist-path', r.persistPath)
    if (r.persistInterval) args.push('--persist-interval', String(r.persistInterval))
  }
  if (r.journalEnabled) {
    args.push('--journal')
    if (r.journalPath) args.push('--journal-path', r.journalPath)
    if (r.journalMemory) args.push('--journal-memory')
  }
  if (r.federationEnabled) {
    args.push('--federation')
    if (r.federationHub) args.push('--federation-hub', r.federationHub)
    if (r.federationId) args.push('--federation-id', r.federationId)
    if (r.federationToken) args.push('--federation-token', r.federationToken)
  }
  if (r.healthEnabled) {
    args.push('--health')
    if (r.healthPort) args.push('--health-port', String(r.healthPort))
  }
  if (r.metricsEnabled) {
    args.push('--metrics')
    if (r.metricsPort) args.push('--metrics-port', String(r.metricsPort))
  }
  if (r.drainTimeout) args.push('--drain-timeout', String(r.drainTimeout))
  if (r.rendezvousPort) args.push('--rendezvous-port', String(r.rendezvousPort))
  if (r.rendezvousTtl) args.push('--rendezvous-ttl', String(r.rendezvousTtl))
  if (r.rulesPath) args.push('--rules', r.rulesPath)
  return args
}

function connectionToBridgeCmd(c: Connection, routerAddr: string): string {
  const proto = c.protocol || c.type
  let sourceAddr = c.address || ''
  if (!sourceAddr && c.host && c.port) sourceAddr = `${c.host}:${c.port}`
  if (!sourceAddr && c.bind && c.port) sourceAddr = `${c.bind}:${c.port}`
  return `clasp bridge ${proto} ${sourceAddr} clasp ${routerAddr}`
}

export function useConfigExport() {
  const { routers } = useRouters()
  const { connections } = useConnections()

  const cliOutput = computed(() => {
    const lines: string[] = []
    for (const r of routers.value) {
      if (r.isRemote) continue
      lines.push(routerToCliArgs(r).join(' \\\n  '))
      lines.push('')
    }
    for (const c of connections.value) {
      const router = routers.value.find(r => r.id === c.routerId)
      const rAddr = router?.address || 'localhost:7330'
      lines.push(connectionToBridgeCmd(c, rAddr))
    }
    return lines.join('\n')
  })

  const dockerOutput = computed(() => {
    const services: Record<string, any> = {}
    for (const r of routers.value) {
      if (r.isRemote) continue
      const args = routerToCliArgs(r)
      const port = (r.address || '0.0.0.0:7330').split(':')[1] || '7330'
      const ports = [`${port}:${port}`]
      if (r.authEnabled && r.authPort) ports.push(`${r.authPort}:${r.authPort}`)
      if (r.quicEnabled && r.quicPort) ports.push(`${r.quicPort}:${r.quicPort}/udp`)
      if (r.healthEnabled && r.healthPort) ports.push(`${r.healthPort}:${r.healthPort}`)
      if (r.metricsEnabled && r.metricsPort) ports.push(`${r.metricsPort}:${r.metricsPort}`)
      services[r.name.toLowerCase().replace(/\s+/g, '-') || 'clasp-relay'] = {
        image: 'ghcr.io/clasp-to/relay:latest',
        command: args.slice(1).join(' '),
        ports,
        restart: 'unless-stopped',
      }
    }
    const compose = { version: '3.8', services }
    return `# docker-compose.yml\n${toYaml(compose)}`
  })

  const clientJsExample = computed(() => {
    const router = routers.value.find(r => !r.isRemote)
    if (!router) return '// No router configured'
    const wsAddr = router.address?.startsWith('ws') ? router.address : `ws://${router.address}`
    return `import { ClaspClient } from '@clasp-to/client'

const client = new ClaspClient('${wsAddr}')
${router.authEnabled ? `await client.authenticate('YOUR_TOKEN')` : ''}

// Subscribe to signals
client.on('signal', (address, value) => {
  console.log(address, value)
})

// Set a value
await client.set('/my/param', 0.5)`
  })

  const clientRustExample = computed(() => {
    const router = routers.value.find(r => !r.isRemote)
    if (!router) return '// No router configured'
    return `use clasp_client::ClaspClient;

#[tokio::main]
async fn main() {
    let client = ClaspClient::connect("${router.address || 'localhost:7330'}").await.unwrap();
    ${router.authEnabled ? 'client.authenticate("YOUR_TOKEN").await.unwrap();' : ''}
    client.set("/my/param", 0.5).await.unwrap();
}`
  })

  return {
    cliOutput,
    dockerOutput,
    clientJsExample,
    clientRustExample,
  }
}

function toYaml(obj: Record<string, unknown>, indent = 0): string {
  if (!obj || typeof obj !== 'object') return ''
  const pad = '  '.repeat(indent)
  let out = ''
  for (const [key, val] of Object.entries(obj)) {
    if (Array.isArray(val)) {
      out += `${pad}${key}:\n`
      for (const item of val) {
        out += `${pad}  - ${typeof item === 'object' ? JSON.stringify(item) : item}\n`
      }
    } else if (typeof val === 'object' && val !== null) {
      out += `${pad}${key}:\n${toYaml(val, indent + 1)}`
    } else {
      out += `${pad}${key}: ${val}\n`
    }
  }
  return out
}
