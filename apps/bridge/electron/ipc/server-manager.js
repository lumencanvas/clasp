const { spawn } = require('child_process');
const { app } = require('electron');
const path = require('path');
const fs = require('fs');
const {
  runningServers, claspMonitors, bridgeRouterConnections,
  MAX_LOG_LINES, getMainWindow, isBridgeReady,
} = require('./state');
const { sendToBridge } = require('./bridge-service');
const { connectBridgeToRouter, createClaspMonitor } = require('./router-connection');
const { getBinaryPath } = require('./paths');

function createServerStats() {
  return {
    startTime: Date.now(),
    messagesIn: 0,
    messagesOut: 0,
    bytesIn: 0,
    bytesOut: 0,
    errors: 0,
    connections: 0,
    lastActivity: null,
    lastError: null,
  };
}

async function startClaspServer(config) {
  const routerPath = getBinaryPath('clasp-router');
  const [host, port] = (config.address || 'localhost:7330').split(':');

  const args = [
    '--listen', `${host === 'localhost' ? '0.0.0.0' : host}:${port}`,
    '--name', config.name || 'CLASP Bridge Server',
  ];

  if (config.announce !== false) {
    args.push('--announce');
  }

  // Core
  if (config.maxSessions) args.push('--max-sessions', String(config.maxSessions));
  if (config.sessionTimeout) args.push('--session-timeout', String(config.sessionTimeout));

  // Auth
  let tokenFilePath = null;
  if (config.authEnabled && config.tokenFileContent) {
    const tokensDir = path.join(app.getPath('userData'), 'tokens');
    if (!fs.existsSync(tokensDir)) {
      fs.mkdirSync(tokensDir, { recursive: true });
    }
    tokenFilePath = path.join(tokensDir, `tokens-${config.id}.txt`);
    fs.writeFileSync(tokenFilePath, config.tokenFileContent, 'utf8');
    args.push('--auth-mode', 'authenticated');
    args.push('--token-file', tokenFilePath);
  } else if (config.token) {
    args.push('--auth-mode', 'authenticated');
    args.push('--token', config.token);
  }
  if (config.authEnabled) {
    if (config.authPort) args.push('--auth-port', String(config.authPort));
    if (config.authDb) args.push('--auth-db', config.authDb);
    if (config.adminTokenPath) args.push('--admin-token-path', config.adminTokenPath);
    if (config.tokenTtl) args.push('--token-ttl', String(config.tokenTtl));
    if (config.corsOrigin) args.push('--cors-origin', config.corsOrigin);
  }

  // Transports
  if (config.quicEnabled) {
    args.push('--quic');
    if (config.quicPort) args.push('--quic-port', String(config.quicPort));
    if (config.certPath) args.push('--cert', config.certPath);
    if (config.keyPath) args.push('--key', config.keyPath);
  }
  if (config.mqttBridgeEnabled) {
    args.push('--mqtt');
    if (config.mqttBridgePort) args.push('--mqtt-port', String(config.mqttBridgePort));
    if (config.mqttBridgeNamespace) args.push('--mqtt-namespace', config.mqttBridgeNamespace);
  }
  if (config.oscBridgeEnabled) {
    args.push('--osc');
    if (config.oscBridgePort) args.push('--osc-port', String(config.oscBridgePort));
    if (config.oscBridgeNamespace) args.push('--osc-namespace', config.oscBridgeNamespace);
  }

  // TTL
  if (config.noTtl) {
    args.push('--no-ttl');
  } else {
    if (config.paramTtl) args.push('--param-ttl', String(config.paramTtl));
    if (config.signalTtl) args.push('--signal-ttl', String(config.signalTtl));
  }

  // Persistence
  if (config.persistEnabled) {
    args.push('--persist');
    if (config.persistPath) args.push('--persist-path', config.persistPath);
    if (config.persistInterval) args.push('--persist-interval', String(config.persistInterval));
  }
  if (config.journalEnabled) {
    args.push('--journal');
    if (config.journalPath) args.push('--journal-path', config.journalPath);
    if (config.journalMemory) args.push('--journal-memory');
  }

  // Federation
  if (config.federationEnabled) {
    args.push('--federation');
    if (config.federationHub) args.push('--federation-hub', config.federationHub);
    if (config.federationId) args.push('--federation-id', config.federationId);
    if (config.federationToken) args.push('--federation-token', config.federationToken);
  }

  // Operations
  if (config.healthEnabled) {
    args.push('--health');
    if (config.healthPort) args.push('--health-port', String(config.healthPort));
  }
  if (config.metricsEnabled) {
    args.push('--metrics');
    if (config.metricsPort) args.push('--metrics-port', String(config.metricsPort));
  }
  if (config.drainTimeout) args.push('--drain-timeout', String(config.drainTimeout));
  if (config.rendezvousPort) args.push('--rendezvous-port', String(config.rendezvousPort));
  if (config.rendezvousTtl) args.push('--rendezvous-ttl', String(config.rendezvousTtl));

  // Rules
  if (config.rulesPath) args.push('--rules', config.rulesPath);

  return new Promise((resolve, reject) => {
    try {
      const proc = spawn(routerPath, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
      });

      const serverState = {
        process: proc,
        config,
        status: 'starting',
        logs: [],
        port: parseInt(port),
        stats: createServerStats(),
        tokenFilePath,
      };

      const mainWindow = getMainWindow();

      const addLog = (message, type = 'info') => {
        serverState.logs.push({ timestamp: Date.now(), message, type });
        if (serverState.logs.length > MAX_LOG_LINES) {
          serverState.logs.shift();
        }
        const mw = getMainWindow();
        mw?.webContents.send('server-log', {
          serverId: config.id,
          log: { timestamp: Date.now(), message, type },
        });
      };

      proc.stdout.on('data', (data) => {
        const lines = data.toString().trim().split('\n');
        for (const line of lines) {
          addLog(line, 'stdout');
          if (line.includes('Listening on') || line.includes('Router ready') || line.includes('accepting connections')) {
            serverState.status = 'running';
            getMainWindow()?.webContents.send('server-status', {
              id: config.id,
              status: 'running',
            });
          }
        }
      });

      proc.stderr.on('data', (data) => {
        const lines = data.toString().trim().split('\n');
        for (const line of lines) {
          addLog(line, 'stderr');
          if (line.includes('Listening on') || line.includes('Router ready') || line.includes('accepting connections')) {
            serverState.status = 'running';
            getMainWindow()?.webContents.send('server-status', {
              id: config.id,
              status: 'running',
            });
          }
        }
      });

      proc.on('close', (code) => {
        addLog(`Process exited with code ${code}`, code === 0 ? 'info' : 'error');
        serverState.status = code === 0 ? 'stopped' : 'error';
        getMainWindow()?.webContents.send('server-status', {
          id: config.id,
          status: serverState.status,
          exitCode: code,
        });

        // Close all bridge connections using this router
        for (const [bridgeId, conn] of bridgeRouterConnections) {
          if (conn.routerId === config.id) {
            try { conn.ws.close(); } catch (e) { /* ignore */ }
            bridgeRouterConnections.delete(bridgeId);
            getMainWindow()?.webContents.send('bridge-router-status', {
              bridgeId,
              connected: false,
              error: 'Router stopped',
            });
          }
        }

        runningServers.delete(config.id);
      });

      proc.on('error', (err) => {
        addLog(`Process error: ${err.message}`, 'error');
        serverState.status = 'error';
        serverState.error = err.message;
        getMainWindow()?.webContents.send('server-status', {
          id: config.id,
          status: 'error',
          error: err.message,
        });
        reject(new Error(err.message));
      });

      runningServers.set(config.id, serverState);

      setTimeout(async () => {
        if (serverState.status === 'starting' && proc.exitCode === null) {
          serverState.status = 'running';
          getMainWindow()?.webContents.send('server-status', {
            id: config.id,
            status: 'running',
          });
        }

        try {
          await createClaspMonitor(config.id, `ws://127.0.0.1:${port}`, config.token);
        } catch (err) {
          // Non-critical
        }

        // Connect any bridges waiting for a router
        for (const [bridgeId, server] of runningServers) {
          if (server.config?.target_addr === 'internal' && !bridgeRouterConnections.has(bridgeId)) {
            connectBridgeToRouter(bridgeId, server.config?.routerId).catch(err => {
              console.error(`Failed to connect bridge ${bridgeId} to new router:`, err);
            });
          }
        }

        resolve({ id: config.id, status: serverState.status });
      }, 500);

    } catch (err) {
      reject(err);
    }
  });
}

function createBridgeServer(config, logMessage, port = null) {
  const serverState = {
    process: null,
    config: { ...config, target_addr: 'internal' },
    status: 'running',
    logs: [{ timestamp: Date.now(), message: logMessage, type: 'info' }],
    port,
    stats: createServerStats(),
  };

  runningServers.set(config.id, serverState);

  connectBridgeToRouter(config.id, config.routerId).catch(err => {
    console.error(`Failed to connect bridge ${config.id} to router:`, err);
  });

  return { id: config.id, status: 'running' };
}

async function startOscServer(config) {
  const addr = `${config.bind || '0.0.0.0'}:${config.port || 9000}`;
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'osc',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
  });

  return createBridgeServer(config, `OSC listening on ${addr}`, config.port || 9000);
}

async function startMqttServer(config) {
  const addr = `${config.host || 'localhost'}:${config.port || 1883}`;
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'mqtt',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
    config: { topics: config.topics || ['#'] },
  });

  return createBridgeServer(config, `MQTT connecting to ${addr}`, config.port || 1883);
}

async function startWebSocketServer(config) {
  const addr = config.address || '0.0.0.0:8080';
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'websocket',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
    config: { mode: config.mode || 'server' },
  });

  return createBridgeServer(
    config,
    `WebSocket ${config.mode || 'server'} on ${addr}`,
    parseInt(addr.split(':')[1]) || 8080
  );
}

async function startHttpServer(config) {
  const addr = config.bind || '0.0.0.0:3000';
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'http',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
    config: {
      base_path: config.basePath || '/api',
      cors: config.cors !== false,
    },
  });

  return createBridgeServer(
    config,
    `HTTP API on ${addr}${config.basePath || '/api'}`,
    parseInt(addr.split(':')[1]) || 3000
  );
}

async function startArtNetServer(config) {
  const addr = config.bind || '0.0.0.0:6454';
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'artnet',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
    config: {
      subnet: config.subnet || 0,
      universe: config.universe || 0,
    },
  });

  return createBridgeServer(
    config,
    `Art-Net on ${addr} (${config.subnet || 0}:${config.universe || 0})`,
    6454
  );
}

async function startSacnServer(config) {
  const universe = config.universe || 1;
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'sacn',
    source_addr: `${config.bind || '0.0.0.0'}:5568`,
    target: 'clasp',
    target_addr: 'internal',
    config: { universe },
  });

  return createBridgeServer(config, `sACN universe ${universe}`, 5568);
}

async function startDmxServer(config) {
  const serialPort = config.serialPort || '/dev/ttyUSB0';
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'dmx',
    source_addr: serialPort,
    target: 'clasp',
    target_addr: 'internal',
    config: { universe: config.universe || 0 },
  });

  return createBridgeServer(
    config,
    `DMX on ${serialPort} (U${config.universe || 0})`,
    null
  );
}

async function startMidiServer(config) {
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'midi',
    source_addr: config.device || 'default',
    target: 'clasp',
    target_addr: 'internal',
    config: { channel: config.channel || 0 },
  });

  return createBridgeServer(config, `MIDI ${config.device || 'default'}`, null);
}

async function startSocketIoServer(config) {
  const addr = config.address || '0.0.0.0:3001';
  if (!isBridgeReady()) throw new Error('Bridge service not ready');

  sendToBridge({
    type: 'create_bridge',
    id: config.id,
    source: 'socketio',
    source_addr: addr,
    target: 'clasp',
    target_addr: 'internal',
  });

  return createBridgeServer(
    config,
    `Socket.IO on ${addr}`,
    parseInt(addr.split(':')[1]) || 3001
  );
}

async function startServer(config) {
  const serverType = config.type || config.protocol || 'clasp';
  const serverId = config.id || Date.now().toString();
  config.id = serverId;

  const starters = {
    clasp: startClaspServer,
    osc: startOscServer,
    mqtt: startMqttServer,
    websocket: startWebSocketServer,
    http: startHttpServer,
    artnet: startArtNetServer,
    sacn: startSacnServer,
    dmx: startDmxServer,
    midi: startMidiServer,
    socketio: startSocketIoServer,
  };

  const starter = starters[serverType];
  if (!starter) throw new Error(`Unknown server type: ${serverType}`);

  const result = await starter(config);
  return { id: serverId, status: result.status || 'running' };
}

async function stopServer(id) {
  const server = runningServers.get(id);
  if (!server) return false;

  if (claspMonitors.has(id)) {
    try {
      claspMonitors.get(id).close();
      claspMonitors.delete(id);
    } catch (e) { /* ignore */ }
  }

  if (server.process) {
    server.process.kill('SIGTERM');
    await new Promise(resolve => setTimeout(resolve, 500));
    if (server.process && server.process.exitCode === null) {
      server.process.kill('SIGKILL');
    }
    if (server.tokenFilePath) {
      try { fs.unlinkSync(server.tokenFilePath); } catch (e) { /* ignore */ }
    }
  } else {
    if (isBridgeReady()) {
      sendToBridge({ type: 'delete_bridge', id });
    }
    if (bridgeRouterConnections.has(id)) {
      try {
        const conn = bridgeRouterConnections.get(id);
        if (conn.ws) conn.ws.close();
      } catch (e) { /* ignore */ }
      bridgeRouterConnections.delete(id);
    }
    runningServers.delete(id);
  }

  return true;
}

async function stopAllServers() {
  const ids = Array.from(runningServers.keys());
  for (const id of ids) {
    await stopServer(id);
  }
}

module.exports = {
  createServerStats,
  startServer,
  stopServer,
  stopAllServers,
};
