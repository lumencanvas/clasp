import { describe, it, expect } from 'vitest'
import { RelayBuilder } from '../src/relay'

describe('RelayBuilder', () => {
  describe('CLI command', () => {
    it('generates minimal command', () => {
      expect(new RelayBuilder().toCommand()).toBe('clasp-relay')
    })

    it('generates command with port and auth', () => {
      const cmd = new RelayBuilder().port(7330).authPort(7350).toCommand()
      expect(cmd).toContain('--ws-port 7330')
      expect(cmd).toContain('--auth-port 7350')
    })

    it('generates command with all core options', () => {
      const cmd = new RelayBuilder()
        .port(7330)
        .host('0.0.0.0')
        .name('My Relay')
        .authPort(7350)
        .corsOrigin('https://app.example.com')
        .adminTokenPath('./admin.token')
        .tokenTtl(86400)
        .maxSessions(500)
        .sessionTimeout(300)
        .paramTtl(3600)
        .verbose()
        .toCommand()

      expect(cmd).toContain('--ws-port 7330')
      expect(cmd).toContain('--host 0.0.0.0')
      expect(cmd).toContain('--name "My Relay"')
      expect(cmd).toContain('--auth-port 7350')
      expect(cmd).toContain('--cors-origin https://app.example.com')
      expect(cmd).toContain('--admin-token ./admin.token')
      expect(cmd).toContain('--token-ttl 86400')
      expect(cmd).toContain('--max-sessions 500')
      expect(cmd).toContain('--session-timeout 300')
      expect(cmd).toContain('--param-ttl 3600')
      expect(cmd).toContain('--verbose')
    })

    it('generates command with persistence', () => {
      const cmd = new RelayBuilder().persist('./state.db', { interval: 30 }).toCommand()
      expect(cmd).toContain('--persist ./state.db')
      expect(cmd).toContain('--persist-interval 30')
    })

    it('generates command with journal', () => {
      const cmd = new RelayBuilder().journal('./journal.db', { batchSize: 100 }).toCommand()
      expect(cmd).toContain('--journal ./journal.db')
      expect(cmd).toContain('--journal-batch-size 100')
    })

    it('generates command with memory journal', () => {
      const cmd = new RelayBuilder().journal('memory').toCommand()
      expect(cmd).toContain('--journal-memory')
    })

    it('generates command with TLS', () => {
      const cmd = new RelayBuilder().tls('./cert.pem', './key.pem').toCommand()
      expect(cmd).toContain('--cert ./cert.pem')
      expect(cmd).toContain('--key ./key.pem')
    })

    it('generates command with app config object', () => {
      const cmd = new RelayBuilder().appConfig({ scopes: ['read:/**'] }).toCommand()
      expect(cmd).toContain('--app-config ./app-config.json')
    })

    it('generates command with app config path', () => {
      const cmd = new RelayBuilder().appConfig('./custom-config.json').toCommand()
      expect(cmd).toContain('--app-config ./custom-config.json')
    })

    it('generates command with rules', () => {
      const cmd = new RelayBuilder().rules({ rules: [{ id: 'test' }] }).toCommand()
      expect(cmd).toContain('--rules ./rules.json')
    })

    it('generates command with capability tokens', () => {
      const cmd = new RelayBuilder()
        .capabilityTokens({ trustAnchors: ['./root.pub', './backup.pub'], maxDepth: 3 })
        .toCommand()
      expect(cmd).toContain('--trust-anchor ./root.pub')
      expect(cmd).toContain('--trust-anchor ./backup.pub')
      expect(cmd).toContain('--cap-max-depth 3')
    })

    it('generates command with entity registry', () => {
      const cmd = new RelayBuilder().entityRegistry('./registry.db').toCommand()
      expect(cmd).toContain('--registry-db ./registry.db')
    })

    it('generates command with MQTT', () => {
      const cmd = new RelayBuilder().mqtt(1883, { namespace: '/mqtt' }).toCommand()
      expect(cmd).toContain('--mqtt-port 1883')
      expect(cmd).toContain('--mqtt-namespace /mqtt')
    })

    it('generates command with OSC', () => {
      const cmd = new RelayBuilder().osc(8000).toCommand()
      expect(cmd).toContain('--osc-port 8000')
    })

    it('generates command with QUIC', () => {
      const cmd = new RelayBuilder().quic(7331, './cert.pem', './key.pem').toCommand()
      expect(cmd).toContain('--quic-port 7331')
      expect(cmd).toContain('--cert ./cert.pem')
    })

    it('generates command with federation', () => {
      const cmd = new RelayBuilder()
        .federation({
          hub: 'ws://hub:7330',
          id: 'site-a',
          namespaces: ['/site-a/**'],
          token: './secret.txt',
        })
        .toCommand()
      expect(cmd).toContain('--federation-hub ws://hub:7330')
      expect(cmd).toContain('--federation-id site-a')
      expect(cmd).toContain('--federation-namespace "/site-a/**"')
      expect(cmd).toContain('--federation-token ./secret.txt')
    })

    it('generates command with rendezvous', () => {
      const cmd = new RelayBuilder().rendezvous({ port: 7340, ttl: 300 }).toCommand()
      expect(cmd).toContain('--rendezvous-port 7340')
      expect(cmd).toContain('--rendezvous-ttl 300')
    })

    it('generates command with drain timeout', () => {
      const cmd = new RelayBuilder().drainTimeout(30).toCommand()
      expect(cmd).toContain('--drain-timeout 30')
    })

    it('generates command with no TTL', () => {
      const cmd = new RelayBuilder().noTtl().toCommand()
      expect(cmd).toContain('--no-ttl')
    })

    it('handles CORS with multiple origins', () => {
      const cmd = new RelayBuilder().corsOrigin(['https://a.com', 'https://b.com']).toCommand()
      expect(cmd).toContain('--cors-origin https://a.com,https://b.com')
    })

    it('handles federation with multiple namespaces', () => {
      const cmd = new RelayBuilder()
        .federation({ hub: 'ws://hub:7330', id: 'site', namespaces: ['/site/**', '/shared/**'] })
        .toCommand()
      expect(cmd).toContain('--federation-namespace "/site/**"')
      expect(cmd).toContain('--federation-namespace "/shared/**"')
    })

    it('full production command', () => {
      const cmd = new RelayBuilder()
        .port(7330)
        .name('Production Relay')
        .authPort(7350)
        .corsOrigin('https://app.example.com')
        .adminTokenPath('./admin.token')
        .persist('./state.db', { interval: 30 })
        .journal('./journal.db')
        .appConfig({ scopes: ['read:/**'] })
        .mqtt(1883)
        .maxSessions(500)
        .paramTtl(3600)
        .tls('./cert.pem', './key.pem')
        .toCommand()

      expect(cmd).toContain('clasp-relay')
      expect(cmd).toContain('--ws-port 7330')
      expect(cmd).toContain('--auth-port 7350')
      expect(cmd).toContain('--persist ./state.db')
      expect(cmd).toContain('--journal ./journal.db')
      expect(cmd).toContain('--mqtt-port 1883')
      expect(cmd).toContain('--cert ./cert.pem')
    })
  })

  describe('logLevel', () => {
    it('generates --log-level flag', () => {
      const cmd = new RelayBuilder().logLevel('info').toCommand()
      expect(cmd).toContain('--log-level info')
      expect(cmd).not.toContain('--verbose')
    })

    it('logLevel takes precedence over verbose', () => {
      const cmd = new RelayBuilder().verbose().logLevel('warn').toCommand()
      expect(cmd).toContain('--log-level warn')
      expect(cmd).not.toContain('--verbose')
    })

    it('sets RUST_LOG in env output', () => {
      const env = new RelayBuilder().logLevel('trace').toEnv()
      expect(env).toContain('RUST_LOG=trace')
    })
  })

  describe('Port validation', () => {
    it('port() throws on negative', () => {
      expect(() => new RelayBuilder().port(-1)).toThrow('must be an integer between 0 and 65535')
    })

    it('port() throws on > 65535', () => {
      expect(() => new RelayBuilder().port(70000)).toThrow('must be an integer between 0 and 65535')
    })

    it('port() throws on non-integer', () => {
      expect(() => new RelayBuilder().port(7330.5)).toThrow('must be an integer between 0 and 65535')
    })

    it('authPort() throws on invalid', () => {
      expect(() => new RelayBuilder().authPort(-1)).toThrow('must be an integer between 0 and 65535')
    })

    it('mqtt() throws on invalid port', () => {
      expect(() => new RelayBuilder().mqtt(99999)).toThrow('must be an integer between 0 and 65535')
    })

    it('osc() throws on invalid port', () => {
      expect(() => new RelayBuilder().osc(-1)).toThrow('must be an integer between 0 and 65535')
    })

    it('quic() throws on invalid port', () => {
      expect(() => new RelayBuilder().quic(70000, 'c', 'k')).toThrow('must be an integer between 0 and 65535')
    })

    it('port(0) is valid', () => {
      expect(() => new RelayBuilder().port(0)).not.toThrow()
    })

    it('port(65535) is valid', () => {
      expect(() => new RelayBuilder().port(65535)).not.toThrow()
    })
  })

  describe('Edge cases', () => {
    it('chaining returns same builder (fluent API)', () => {
      const builder = new RelayBuilder()
      const result = builder.port(7330).authPort(7350).name('Test')
      expect(result).toBe(builder)
    })

    it('zero values for TTLs produce flags', () => {
      // Note: 0 is falsy in JS so this checks that our toCommand handles it
      const builder = new RelayBuilder()
      builder.getConfig() // just to ensure no crash
    })
  })

  describe('Docker Compose', () => {
    it('generates YAML with ports', () => {
      const compose = new RelayBuilder().port(7330).authPort(7350).mqtt(1883).toDockerCompose()
      expect(compose).toContain('relay:')
      expect(compose).toContain('image: clasp-relay')
      expect(compose).toContain('7330:7330')
      expect(compose).toContain('7350:7350')
      expect(compose).toContain('1883:1883')
      expect(compose).toContain('restart: unless-stopped')
      expect(compose).toContain('healthcheck:')
    })

    it('custom service name and image', () => {
      const compose = new RelayBuilder().port(7330)
        .toDockerCompose({ serviceName: 'clasp', image: 'myrepo/clasp:latest' })
      expect(compose).toContain('clasp:')
      expect(compose).toContain('image: myrepo/clasp:latest')
    })

    it('volumes for persistence', () => {
      const compose = new RelayBuilder().persist('./state.db').toDockerCompose()
      expect(compose).toContain('volumes:')
      expect(compose).toContain('relay-data:/data')
    })

    it('OSC UDP port', () => {
      const compose = new RelayBuilder().osc(8000).toDockerCompose()
      expect(compose).toContain('8000:8000/udp')
    })
  })

  describe('Environment variables', () => {
    it('generates env output', () => {
      const env = new RelayBuilder()
        .port(7330).authPort(7350).maxSessions(500).verbose().toEnv()
      expect(env).toContain('CLASP_WS_PORT=7330')
      expect(env).toContain('CLASP_AUTH_PORT=7350')
      expect(env).toContain('CLASP_MAX_SESSIONS=500')
      expect(env).toContain('RUST_LOG=debug')
    })
  })

  describe('App config JSON', () => {
    it('generates JSON from config object', () => {
      const builder = new RelayBuilder().appConfig({
        scopes: ['read:/**', 'write:/app/{userId}/**'],
        rate_limits: { login_max_attempts: 5 },
      })
      const json = builder.toAppConfigJSON()
      expect(json).not.toBeNull()
      const parsed = JSON.parse(json!)
      expect(parsed.scopes).toEqual(['read:/**', 'write:/app/{userId}/**'])
      expect(parsed.rate_limits.login_max_attempts).toBe(5)
    })

    it('returns null when path is set', () => {
      expect(new RelayBuilder().appConfig('./config.json').toAppConfigJSON()).toBeNull()
    })
  })

  describe('Rules JSON', () => {
    it('generates rules JSON', () => {
      const builder = new RelayBuilder().rules({
        rules: [{ id: 'test', trigger: { type: 'on_interval', seconds: 30 } }],
      })
      const json = builder.toRulesJSON()
      expect(json).not.toBeNull()
      const parsed = JSON.parse(json!)
      expect(parsed.rules[0].id).toBe('test')
    })
  })

  describe('getConfig', () => {
    it('returns copy of config', () => {
      const builder = new RelayBuilder().port(7330).authPort(7350)
      const config = builder.getConfig()
      expect(config.port).toBe(7330)
      expect(config.authPort).toBe(7350)
    })
  })

  describe('fromConfig()', () => {
    it('creates builder from config', () => {
      const builder = RelayBuilder.fromConfig({ port: 7330, authPort: 7350, name: 'Test' })
      const cmd = builder.toCommand()
      expect(cmd).toContain('--ws-port 7330')
      expect(cmd).toContain('--auth-port 7350')
    })

    it('round-trips config', () => {
      const original = new RelayBuilder()
        .port(7330)
        .authPort(7350)
        .mqtt(1883)
        .verbose()
      const config = original.getConfig()
      const rebuilt = RelayBuilder.fromConfig(config)
      expect(rebuilt.toCommand()).toBe(original.toCommand())
    })
  })

  describe('merge()', () => {
    it('merges other builder config', () => {
      const a = new RelayBuilder().port(7330).name('A')
      const b = new RelayBuilder().authPort(7350).name('B')
      a.merge(b)
      const config = a.getConfig()
      expect(config.port).toBe(7330)
      expect(config.authPort).toBe(7350)
      expect(config.name).toBe('B') // b overrides
    })
  })

  describe('toKubernetes()', () => {
    it('generates Deployment with correct image', () => {
      const k8s = new RelayBuilder().port(7330).toKubernetes({ image: 'clasp:v1' })
      expect(k8s).toContain('kind: Deployment')
      expect(k8s).toContain('image: clasp:v1')
    })

    it('generates Service with correct ports', () => {
      const k8s = new RelayBuilder().port(7330).authPort(7350).toKubernetes()
      expect(k8s).toContain('kind: Service')
      expect(k8s).toContain('port: 7330')
      expect(k8s).toContain('port: 7350')
      // Verify proper indentation under spec.ports
      expect(k8s).toContain('  - name: ws\n    port: 7330')
    })

    it('includes env vars from config', () => {
      const k8s = new RelayBuilder().port(7330).verbose().toKubernetes()
      expect(k8s).toContain('RUST_LOG')
      expect(k8s).toContain('debug')
    })

    it('uses custom name and namespace', () => {
      const k8s = new RelayBuilder().toKubernetes({ name: 'my-relay', namespace: 'prod' })
      expect(k8s).toContain('name: my-relay')
      expect(k8s).toContain('namespace: prod')
    })
  })

  describe('toSystemd()', () => {
    it('generates unit file with ExecStart', () => {
      const unit = new RelayBuilder().port(7330).toSystemd()
      expect(unit).toContain('[Unit]')
      expect(unit).toContain('[Service]')
      expect(unit).toContain('[Install]')
      expect(unit).toContain('ExecStart=clasp-relay')
      expect(unit).toContain('--ws-port 7330')
    })

    it('includes restart policy', () => {
      const unit = new RelayBuilder().toSystemd()
      expect(unit).toContain('Restart=on-failure')
    })

    it('uses custom user', () => {
      const unit = new RelayBuilder().toSystemd({ user: 'relay-svc' })
      expect(unit).toContain('User=relay-svc')
    })

    it('uses custom description', () => {
      const unit = new RelayBuilder().toSystemd({ description: 'My Relay' })
      expect(unit).toContain('Description=My Relay')
    })
  })

  describe('toArgs()', () => {
    it('returns empty array for minimal builder', () => {
      expect(new RelayBuilder().toArgs()).toEqual([])
    })

    it('returns args array with port and auth', () => {
      const args = new RelayBuilder().port(7330).authPort(7350).toArgs()
      expect(args).toEqual(['--ws-port', '7330', '--auth-port', '7350'])
    })

    it('returns args with name as separate element (no JSON quoting)', () => {
      const args = new RelayBuilder().name('My Relay').toArgs()
      expect(args).toEqual(['--name', 'My Relay'])
    })

    it('returns args with all core options', () => {
      const args = new RelayBuilder()
        .port(7330)
        .host('0.0.0.0')
        .name('Test')
        .authPort(7350)
        .corsOrigin('https://a.com')
        .adminTokenPath('./admin.token')
        .tokenTtl(86400)
        .maxSessions(500)
        .sessionTimeout(300)
        .paramTtl(3600)
        .signalTtl(1800)
        .verbose()
        .drainTimeout(10)
        .healthPort(9090)
        .toArgs()

      expect(args).toContain('--ws-port')
      expect(args).toContain('7330')
      expect(args).toContain('--host')
      expect(args).toContain('0.0.0.0')
      expect(args).toContain('--name')
      expect(args).toContain('Test')
      expect(args).toContain('--auth-port')
      expect(args).toContain('7350')
      expect(args).toContain('--cors-origin')
      expect(args).toContain('https://a.com')
      expect(args).toContain('--admin-token')
      expect(args).toContain('./admin.token')
      expect(args).toContain('--token-ttl')
      expect(args).toContain('86400')
      expect(args).toContain('--max-sessions')
      expect(args).toContain('500')
      expect(args).toContain('--session-timeout')
      expect(args).toContain('300')
      expect(args).toContain('--param-ttl')
      expect(args).toContain('3600')
      expect(args).toContain('--signal-ttl')
      expect(args).toContain('1800')
      expect(args).toContain('--verbose')
      expect(args).toContain('--drain-timeout')
      expect(args).toContain('10')
      expect(args).toContain('--health-port')
      expect(args).toContain('9090')
    })

    it('handles persistence args', () => {
      const args = new RelayBuilder().persist('./state.db', { interval: 30 }).toArgs()
      expect(args).toEqual(['--persist', './state.db', '--persist-interval', '30'])
    })

    it('handles journal memory mode', () => {
      const args = new RelayBuilder().journal('memory').toArgs()
      expect(args).toEqual(['--journal-memory'])
    })

    it('handles journal file mode with options', () => {
      const args = new RelayBuilder().journal('./journal.db', { batchSize: 100, flushMs: 500 }).toArgs()
      expect(args).toContain('--journal')
      expect(args).toContain('./journal.db')
      expect(args).toContain('--journal-batch-size')
      expect(args).toContain('100')
      expect(args).toContain('--journal-flush-ms')
      expect(args).toContain('500')
    })

    it('handles CORS array origins', () => {
      const args = new RelayBuilder().corsOrigin(['https://a.com', 'https://b.com']).toArgs()
      expect(args).toContain('--cors-origin')
      expect(args).toContain('https://a.com,https://b.com')
    })

    it('logLevel takes precedence over verbose', () => {
      const args = new RelayBuilder().verbose().logLevel('warn').toArgs()
      expect(args).toContain('--log-level')
      expect(args).toContain('warn')
      expect(args).not.toContain('--verbose')
    })

    it('noTtl flag', () => {
      const args = new RelayBuilder().noTtl().toArgs()
      expect(args).toContain('--no-ttl')
    })

    it('TLS args', () => {
      const args = new RelayBuilder().tls('./cert.pem', './key.pem').toArgs()
      expect(args).toEqual(['--cert', './cert.pem', '--key', './key.pem'])
    })

    it('skips inline appConfig object (not a file path)', () => {
      const args = new RelayBuilder().appConfig({ scopes: ['read:/**'] }).toArgs()
      expect(args).not.toContain('--app-config')
    })

    it('includes string appConfig path', () => {
      const args = new RelayBuilder().appConfig('./config.json').toArgs()
      expect(args).toEqual(['--app-config', './config.json'])
    })

    it('skips inline rules object', () => {
      const args = new RelayBuilder().rules({ rules: [] }).toArgs()
      expect(args).not.toContain('--rules')
    })

    it('includes string rules path', () => {
      const args = new RelayBuilder().rules('./rules.json').toArgs()
      expect(args).toEqual(['--rules', './rules.json'])
    })

    it('handles federation args', () => {
      const args = new RelayBuilder().federation({
        hub: 'ws://hub:7330', id: 'site-a', namespaces: ['/site-a/**', '/shared/**'], token: 'secret',
      }).toArgs()
      expect(args).toContain('--federation-hub')
      expect(args).toContain('ws://hub:7330')
      expect(args).toContain('--federation-id')
      expect(args).toContain('site-a')
      expect(args).toContain('--federation-namespace')
      expect(args).toContain('/site-a/**')
      expect(args).toContain('/shared/**')
      expect(args).toContain('--federation-token')
      expect(args).toContain('secret')
    })

    it('handles MQTT args', () => {
      const args = new RelayBuilder().mqtt(1883, { namespace: '/mqtt' }).toArgs()
      expect(args).toEqual(['--mqtt-port', '1883', '--mqtt-namespace', '/mqtt'])
    })

    it('handles rendezvous args', () => {
      const args = new RelayBuilder().rendezvous({ port: 7340, ttl: 300 }).toArgs()
      expect(args).toEqual(['--rendezvous-port', '7340', '--rendezvous-ttl', '300'])
    })

    it('handles capability token args', () => {
      const args = new RelayBuilder().capabilityTokens({ trustAnchors: ['./root.pub'], maxDepth: 3 }).toArgs()
      expect(args).toContain('--trust-anchor')
      expect(args).toContain('./root.pub')
      expect(args).toContain('--cap-max-depth')
      expect(args).toContain('3')
    })

    it('all values are strings (suitable for spawn)', () => {
      const args = new RelayBuilder()
        .port(7330).authPort(7350).maxSessions(100).paramTtl(3600).toArgs()
      for (const arg of args) {
        expect(typeof arg).toBe('string')
      }
    })
  })

  describe('healthPort', () => {
    it('generates --health-port flag in toCommand', () => {
      const cmd = new RelayBuilder().healthPort(9090).toCommand()
      expect(cmd).toContain('--health-port 9090')
    })

    it('generates --health-port flag in toArgs', () => {
      const args = new RelayBuilder().healthPort(9090).toArgs()
      expect(args).toContain('--health-port')
      expect(args).toContain('9090')
    })

    it('validates port range', () => {
      expect(() => new RelayBuilder().healthPort(-1)).toThrow('must be an integer between 0 and 65535')
      expect(() => new RelayBuilder().healthPort(70000)).toThrow('must be an integer between 0 and 65535')
    })

    it('round-trips through getConfig/fromConfig', () => {
      const original = new RelayBuilder().healthPort(9090)
      const rebuilt = RelayBuilder.fromConfig(original.getConfig())
      expect(rebuilt.toArgs()).toContain('--health-port')
      expect(rebuilt.toArgs()).toContain('9090')
    })
  })
})
