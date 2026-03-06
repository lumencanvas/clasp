const WebSocket = require('ws');
const {
  runningServers, claspMonitors, bridgeRouterConnections,
  circuitBreakers, getMainWindow,
} = require('./state');
const { MSG, encodeClaspFrame, decodeClaspFrame } = require('./clasp-protocol');
const { CircuitBreaker, calculateBackoffDelay } = require('./circuit-breaker');
const { ErrorType } = require('./error-classifier');
const { persistLog } = require('./logging');

function findClaspRouter(routerId = null) {
  if (routerId) {
    const server = runningServers.get(routerId);
    if (server && server.config?.type === 'clasp' && server.status === 'running' && server.port) {
      return {
        id: routerId,
        address: server.config.address || `localhost:${server.port}`,
        port: server.port,
        token: server.config.token,
      };
    }
  }

  for (const [id, server] of runningServers) {
    if (server.config?.type === 'clasp' && server.status === 'running' && server.port) {
      return {
        id,
        address: server.config.address || `localhost:${server.port}`,
        port: server.port,
        token: server.config.token,
      };
    }
  }
  return null;
}

async function connectBridgeToRouter(bridgeId, routerId = null) {
  const bridge = runningServers.get(bridgeId);
  const requestedRouterId = routerId || bridge?.config?.routerId;
  const router = findClaspRouter(requestedRouterId);
  const mainWindow = getMainWindow();

  if (!router) {
    if (mainWindow) {
      mainWindow.webContents.send('bridge-router-status', {
        bridgeId,
        connected: false,
        error: 'No CLASP router available',
      });
    }
    return false;
  }

  if (bridgeRouterConnections.has(bridgeId)) {
    try {
      const conn = bridgeRouterConnections.get(bridgeId);
      if (conn.ws) conn.ws.close();
    } catch (e) { /* ignore */ }
  }

  const wsUrl = router.address.startsWith('ws://')
    ? router.address
    : `ws://${router.address}`;

  return new Promise((resolve) => {
    const ws = new WebSocket(wsUrl);
    ws.binaryType = 'nodebuffer';
    let connected = false;
    let welcomed = false;

    const connection = {
      ws,
      routerId: router.id,
      routerAddress: router.address,
      token: router.token,
      welcomed: false,
    };

    ws.on('open', () => {
      connected = true;
      bridgeRouterConnections.set(bridgeId, connection);

      const helloMsg = {
        type: MSG.HELLO,
        version: 3,
        name: `Bridge ${bridgeId}`,
        features: ['param', 'event', 'stream'],
      };
      if (router.token) helloMsg.token = router.token;
      ws.send(encodeClaspFrame(helloMsg));
    });

    ws.on('message', (data) => {
      try {
        const buffer = Buffer.from(data);
        const msg = decodeClaspFrame(buffer);

        if (msg.type === MSG.WELCOME) {
          welcomed = true;
          connection.welcomed = true;
          const mainWindow = getMainWindow();
          if (mainWindow) {
            mainWindow.webContents.send('bridge-router-status', {
              bridgeId,
              connected: true,
              routerId: router.id,
              routerAddress: router.address,
            });
          }
          resolve(true);
          return;
        }

        if (msg.type === MSG.ERROR) {
          const errorCode = msg.code || 0;
          const errorMessage = msg.message || 'Unknown error';
          if (errorCode >= 300 && errorCode < 400) {
            ws.close();
            bridgeRouterConnections.delete(bridgeId);
            const mainWindow = getMainWindow();
            if (mainWindow) {
              mainWindow.webContents.send('bridge-router-status', {
                bridgeId,
                connected: false,
                error: `Authentication failed: ${errorMessage}`,
              });
            }
            resolve(false);
            return;
          }
        }

        if (msg.type === MSG.PING) {
          ws.send(encodeClaspFrame({ type: MSG.PONG }));
        }
      } catch (e) { /* decode error */ }
    });

    ws.on('error', (err) => {
      if (!connected) {
        bridgeRouterConnections.delete(bridgeId);
        const mainWindow = getMainWindow();
        if (mainWindow) {
          mainWindow.webContents.send('bridge-router-status', {
            bridgeId,
            connected: false,
            error: err.message,
          });
        }
        resolve(false);
      }
    });

    ws.on('close', () => {
      const existingConn = bridgeRouterConnections.get(bridgeId);
      if (existingConn && existingConn.routerId === router.id) {
        bridgeRouterConnections.delete(bridgeId);
        const mainWindow = getMainWindow();
        if (mainWindow) {
          mainWindow.webContents.send('bridge-router-status', {
            bridgeId,
            connected: false,
            error: 'Connection closed',
          });
        }

        let circuitBreaker = circuitBreakers.get(bridgeId);
        if (!circuitBreaker) {
          circuitBreaker = new CircuitBreaker({
            failureThreshold: 3,
            resetTimeout: 30000,
            maxRetries: 10,
          });
          circuitBreakers.set(bridgeId, circuitBreaker);
        }

        circuitBreaker.recordFailure();

        persistLog('warn', 'Bridge disconnected from router', {
          bridgeId,
          routerId: router.id,
          errorType: ErrorType.NETWORK,
          circuitState: circuitBreaker.getState(),
          retryCount: circuitBreaker.getRetryCount(),
        });

        if (circuitBreaker.shouldRetry()) {
          const delay = calculateBackoffDelay(
            circuitBreaker.getRetryCount(), 1000, 30000, 0.2
          );

          persistLog('info', 'Scheduling reconnection attempt', {
            bridgeId, delay, attempt: circuitBreaker.getRetryCount(),
          });

          setTimeout(() => {
            const server = runningServers.get(router.id);
            const br = runningServers.get(bridgeId);
            if (server && server.status === 'running' && br) {
              connectBridgeToRouter(bridgeId, br.config?.routerId).then(success => {
                if (success) {
                  circuitBreaker.recordSuccess();
                  persistLog('info', 'Reconnection successful', { bridgeId });
                }
              });
            }
          }, delay);
        } else {
          persistLog('error', 'Circuit breaker open - stopping reconnection attempts', {
            bridgeId,
            circuitState: circuitBreaker.getState(),
            totalRetries: circuitBreaker.getRetryCount(),
          });

          const mainWindow = getMainWindow();
          if (mainWindow) {
            mainWindow.webContents.send('bridge-router-status', {
              bridgeId,
              connected: false,
              error: `Reconnection failed after ${circuitBreaker.getRetryCount()} attempts. Circuit breaker open.`,
              circuitState: circuitBreaker.getState(),
            });
          }
        }
      }
    });

    setTimeout(() => {
      if (!connected || !welcomed) {
        if (ws) ws.terminate();
        bridgeRouterConnections.delete(bridgeId);
        const mainWindow = getMainWindow();
        if (mainWindow) {
          mainWindow.webContents.send('bridge-router-status', {
            bridgeId,
            connected: false,
            error: 'Connection timeout',
          });
        }
        resolve(false);
      }
    }, 5000);
  });
}

function forwardSignalToRouter(bridgeId, address, value) {
  const connection = bridgeRouterConnections.get(bridgeId);
  if (!connection || !connection.welcomed) return false;
  if (connection.ws.readyState !== 1) return false;

  try {
    const frame = encodeClaspFrame({
      type: MSG.SET,
      address,
      value,
    });
    connection.ws.send(frame);

    const server = runningServers.get(bridgeId);
    if (server && server.stats) {
      server.stats.messagesOut++;
      server.stats.lastActivity = Date.now();
    }
    return true;
  } catch (e) {
    console.error(`Failed to forward signal to router for bridge ${bridgeId}:`, e);
    return false;
  }
}

async function createClaspMonitor(serverId, wsUrl, token = null) {
  if (claspMonitors.has(serverId)) {
    try { claspMonitors.get(serverId).close(); } catch (e) { /* ignore */ }
  }

  return new Promise((resolve, reject) => {
    const ws = new WebSocket(wsUrl);
    ws.binaryType = 'nodebuffer';
    let connected = false;
    let welcomed = false;

    ws.on('open', () => {
      connected = true;
      claspMonitors.set(serverId, ws);

      const helloMsg = {
        type: MSG.HELLO,
        version: 3,
        name: 'CLASP Bridge Monitor',
        features: ['param', 'event', 'stream'],
      };
      if (token) helloMsg.token = token;
      ws.send(encodeClaspFrame(helloMsg));
    });

    ws.on('message', (data) => {
      try {
        const buffer = Buffer.from(data);
        const msg = decodeClaspFrame(buffer);

        if (msg.type === MSG.WELCOME) {
          welcomed = true;
          ws.send(encodeClaspFrame({
            type: MSG.SUBSCRIBE,
            id: 1,
            pattern: '/**',
            types: ['param', 'event', 'stream'],
          }));
          resolve(ws);
          return;
        }

        if (msg.type === MSG.ERROR) {
          const errorCode = msg.code || 0;
          const errorMessage = msg.message || 'Unknown error';
          if (errorCode >= 300 && errorCode < 400) {
            ws.close();
            claspMonitors.delete(serverId);
            const mainWindow = getMainWindow();
            mainWindow?.webContents.send('server-status', {
              id: serverId,
              status: 'error',
              error: `Authentication failed: ${errorMessage}`,
            });
            reject(new Error(`Authentication failed: ${errorMessage}`));
            return;
          }
          console.error(`CLASP error from server ${serverId}: ${errorCode} - ${errorMessage}`);
          return;
        }

        if (msg.type === MSG.PING) {
          ws.send(encodeClaspFrame({ type: MSG.PONG }));
          return;
        }

        if (msg.type === MSG.SET || msg.type === MSG.PUBLISH) {
          const serverInfo = runningServers.get(serverId);
          if (serverInfo && serverInfo.stats) {
            serverInfo.stats.messagesIn++;
            serverInfo.stats.lastActivity = Date.now();
          }

          const signal = {
            bridgeId: serverId,
            address: msg.address || '/',
            value: msg.value !== undefined ? msg.value : msg.payload,
            protocol: 'clasp',
            serverName: serverInfo?.config?.name || 'CLASP Server',
            serverPort: serverInfo?.port,
          };

          const mainWindow = getMainWindow();
          mainWindow?.webContents.send('signal', signal);

          const bridgeSvc = require('./bridge-service');
          if (bridgeSvc.isLearnModeActive() && bridgeSvc.getLearnModeTarget()) {
            mainWindow?.webContents.send('learned-signal', {
              ...signal,
              target: bridgeSvc.getLearnModeTarget(),
            });
          }
        }

        if (msg.type === MSG.SNAPSHOT && msg.params) {
          const serverInfo = runningServers.get(serverId);
          const mainWindow = getMainWindow();
          for (const param of msg.params) {
            mainWindow?.webContents.send('signal', {
              bridgeId: serverId,
              address: param.address,
              value: param.value,
              protocol: 'clasp',
              serverName: serverInfo?.config?.name || 'CLASP Server',
              serverPort: serverInfo?.port,
            });
          }
        }
      } catch (e) { /* decode error */ }
    });

    ws.on('error', (err) => {
      if (!connected) reject(err);
    });

    ws.on('close', () => {
      claspMonitors.delete(serverId);
    });

    ws.on('unexpected-response', () => { /* not a WS server */ });

    setTimeout(() => {
      if (!connected) {
        ws.terminate();
        reject(new Error('Connection timeout'));
      }
    }, 5000);
  });
}

module.exports = {
  findClaspRouter,
  connectBridgeToRouter,
  forwardSignalToRouter,
  createClaspMonitor,
};
