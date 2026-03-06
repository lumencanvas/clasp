const { ipcMain } = require('electron');
const WebSocket = require('ws');
const {
  runningServers, getMainWindow,
  isBridgeReady, getBridgeService,
} = require('./state');
const { sendToBridge } = require('./bridge-service');

function formatUptime(ms) {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ${hours % 24}h`;
  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
}

let statsInterval = null;

function startStatsBroadcast() {
  if (statsInterval) clearInterval(statsInterval);

  statsInterval = setInterval(() => {
    const mainWindow = getMainWindow();
    if (!mainWindow) return;

    const allStats = [];
    for (const [id, server] of runningServers) {
      const stats = server.stats || {};
      allStats.push({
        id,
        status: server.status,
        messagesIn: stats.messagesIn || 0,
        messagesOut: stats.messagesOut || 0,
        errors: stats.errors || 0,
        connections: stats.connections || 0,
        lastActivity: stats.lastActivity,
      });
    }

    mainWindow.webContents.send('server-stats-update', allStats);
  }, 1000);
}

function stopStatsBroadcast() {
  if (statsInterval) {
    clearInterval(statsInterval);
    statsInterval = null;
  }
}

function registerDiagnosticsHandlers() {
  ipcMain.handle('get-server-stats', async (event, id) => {
    const server = runningServers.get(id);
    if (!server) return null;

    const stats = server.stats || {};
    const uptime = stats.startTime ? Date.now() - stats.startTime : 0;

    return {
      id,
      status: server.status,
      uptime,
      uptimeFormatted: formatUptime(uptime),
      messagesIn: stats.messagesIn || 0,
      messagesOut: stats.messagesOut || 0,
      bytesIn: stats.bytesIn || 0,
      bytesOut: stats.bytesOut || 0,
      errors: stats.errors || 0,
      connections: stats.connections || 0,
      lastActivity: stats.lastActivity,
      lastError: stats.lastError,
      config: server.config,
      port: server.port,
    };
  });

  ipcMain.handle('get-all-server-stats', async () => {
    const allStats = [];
    for (const [id, server] of runningServers) {
      const stats = server.stats || {};
      const uptime = stats.startTime ? Date.now() - stats.startTime : 0;
      allStats.push({
        id,
        status: server.status,
        uptime,
        uptimeFormatted: formatUptime(uptime),
        messagesIn: stats.messagesIn || 0,
        messagesOut: stats.messagesOut || 0,
        errors: stats.errors || 0,
        connections: stats.connections || 0,
        lastActivity: stats.lastActivity,
        protocol: server.config?.protocol || server.config?.type,
        name: server.config?.name,
      });
    }
    return allStats;
  });

  ipcMain.handle('health-check', async (event, id) => {
    const server = runningServers.get(id);
    if (!server) return { healthy: false, error: 'Server not found' };

    const checks = {
      processRunning: false,
      portOpen: false,
      lastActivityRecent: false,
      noRecentErrors: true,
    };

    if (server.process) {
      checks.processRunning = server.process.exitCode === null;
    } else {
      checks.processRunning = server.status === 'running';
    }

    if (server.port && server.config?.type !== 'dmx') {
      try {
        const net = require('net');
        checks.portOpen = await new Promise((resolve) => {
          const socket = new net.Socket();
          socket.setTimeout(2000);
          socket.on('connect', () => { socket.destroy(); resolve(true); });
          socket.on('error', () => resolve(false));
          socket.on('timeout', () => { socket.destroy(); resolve(false); });
          socket.connect(server.port, '127.0.0.1');
        });
      } catch (e) {
        checks.portOpen = false;
      }
    } else {
      checks.portOpen = true;
    }

    const stats = server.stats || {};
    if (stats.lastActivity) {
      checks.lastActivityRecent = Date.now() - stats.lastActivity < 60000;
    }
    checks.noRecentErrors = !stats.lastError || (Date.now() - stats.lastError > 60000);

    return {
      healthy: checks.processRunning && checks.portOpen,
      checks,
      status: server.status,
      uptime: stats.startTime ? Date.now() - stats.startTime : 0,
    };
  });

  ipcMain.handle('get-bridge-status', async () => {
    const bridgeService = getBridgeService();
    return {
      ready: isBridgeReady(),
      running: bridgeService !== null,
      pid: bridgeService?.pid || null,
    };
  });

  ipcMain.handle('run-diagnostics', async () => {
    const bridgeService = getBridgeService();
    const diagnostics = {
      bridgeService: {
        running: bridgeService !== null && isBridgeReady(),
        pid: bridgeService?.pid,
      },
      servers: [],
      system: {
        platform: process.platform,
        nodeVersion: process.version,
        electronVersion: process.versions.electron,
        memoryUsage: process.memoryUsage(),
        uptime: process.uptime(),
      },
    };

    for (const [id, server] of runningServers) {
      const stats = server.stats || {};
      diagnostics.servers.push({
        id,
        name: server.config?.name,
        type: server.config?.type || server.config?.protocol,
        status: server.status,
        processRunning: server.process ? server.process.exitCode === null : true,
        port: server.port,
        uptime: stats.startTime ? Date.now() - stats.startTime : 0,
        messagesIn: stats.messagesIn || 0,
        messagesOut: stats.messagesOut || 0,
        errors: stats.errors || 0,
        lastActivity: stats.lastActivity,
        lastError: stats.lastError,
      });
    }

    return diagnostics;
  });

  ipcMain.handle('send-test-signal', async (event, { protocol, address, signalAddress, value }) => {
    if (!isBridgeReady()) return { success: false, error: 'Bridge service not ready' };

    try {
      sendToBridge({
        type: 'send_signal',
        protocol,
        target_addr: address,
        signal: { address: signalAddress, value },
      });

      for (const [id, server] of runningServers) {
        if (server.config?.address === address || server.config?.bind === address) {
          if (server.stats) {
            server.stats.messagesOut++;
            server.stats.lastActivity = Date.now();
          }
        }
      }

      return { success: true };
    } catch (e) {
      return { success: false, error: e.message };
    }
  });

  ipcMain.handle('send-test-signal-batch', async (event, { signals }) => {
    if (!isBridgeReady()) return { success: false, error: 'Bridge service not ready' };

    let sent = 0;
    for (const signal of signals) {
      try {
        sendToBridge({
          type: 'send_signal',
          protocol: signal.protocol,
          target_addr: signal.address,
          signal: { address: signal.signalAddress, value: signal.value },
        });
        sent++;
      } catch (e) {
        console.error('Failed to send test signal:', e);
      }
    }

    return { success: true, sent };
  });

  ipcMain.handle('test-connection', async (event, address) => {
    return new Promise((resolve) => {
      const wsUrl = address.startsWith('ws://') ? address : `ws://${address}`;
      let ws;
      let timeout;

      try {
        ws = new WebSocket(wsUrl);

        timeout = setTimeout(() => {
          if (ws) ws.terminate();
          resolve({ success: false, error: 'Connection timeout' });
        }, 5000);

        ws.on('open', () => {
          clearTimeout(timeout);
          ws.close();
          resolve({ success: true });
        });

        ws.on('error', (err) => {
          clearTimeout(timeout);
          resolve({ success: false, error: err.message });
        });
      } catch (e) {
        if (timeout) clearTimeout(timeout);
        resolve({ success: false, error: e.message });
      }
    });
  });
}

module.exports = { registerDiagnosticsHandlers, startStatsBroadcast, stopStatsBroadcast };
