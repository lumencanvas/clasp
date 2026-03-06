const { spawn } = require('child_process');
const readline = require('readline');
const {
  getMainWindow, getBridgeService, setBridgeService,
  setBridgeReady, isBridgeReady, runningServers,
} = require('./state');
const { persistLog } = require('./logging');
const { getBinaryPath } = require('./paths');

let learnModeActive = false;
let learnModeTarget = null;

function sendToBridge(message) {
  const bridgeService = getBridgeService();
  if (bridgeService && bridgeService.stdin) {
    const json = JSON.stringify(message);
    bridgeService.stdin.write(json + '\n');
  }
}

function handleBridgeMessage(message) {
  const mainWindow = getMainWindow();

  switch (message.type) {
    case 'ready':
      console.log('[bridge-service] Bridge service is now ready!');
      setBridgeReady(true);
      if (mainWindow) {
        mainWindow.webContents.send('bridge-ready', true);
      }
      break;

    case 'signal': {
      let serverMeta = null;
      if (message.bridge_id) {
        const server = runningServers.get(message.bridge_id);
        if (server) {
          if (server.stats) {
            server.stats.messagesIn++;
            server.stats.lastActivity = Date.now();
          }
          serverMeta = {
            protocol: server.config?.type || 'unknown',
            serverName: server.config?.name || server.config?.claspName || server.config?.type?.toUpperCase() || 'Unknown',
            port: server.port || server.config?.port || null,
            address: server.config?.address || server.config?.claspAddress || null,
          };
        }
      }

      // Forward signal to CLASP router if bridge has 'internal' target
      if (message.bridge_id) {
        const server = runningServers.get(message.bridge_id);
        if (server && server.config?.target_addr === 'internal') {
          const { forwardSignalToRouter } = require('./router-connection');
          forwardSignalToRouter(message.bridge_id, message.address, message.value);
        }
      }

      if (mainWindow) {
        mainWindow.webContents.send('signal', {
          bridgeId: message.bridge_id,
          address: message.address,
          value: message.value,
          protocol: serverMeta?.protocol || message.protocol || 'unknown',
          serverName: serverMeta?.serverName || null,
          serverPort: serverMeta?.port || null,
          serverAddress: serverMeta?.address || null,
        });
      }
      break;
    }

    case 'bridge_event': {
      if (message.bridge_id) {
        const server = runningServers.get(message.bridge_id);
        if (server && server.stats) {
          if (message.event === 'connected') {
            server.stats.connections++;
          } else if (message.event === 'error') {
            server.stats.errors++;
            server.stats.lastError = Date.now();
          }
          server.stats.lastActivity = Date.now();
        }
      }

      if (mainWindow) {
        mainWindow.webContents.send('bridge-event', {
          bridgeId: message.bridge_id,
          event: message.event,
          data: message.data,
        });
      }
      break;
    }
  }
}

function startBridgeService() {
  const servicePath = getBinaryPath('clasp-service');
  const fs = require('fs');

  if (!fs.existsSync(servicePath)) {
    console.error(`Bridge service binary not found at: ${servicePath}`);
    console.error('Please build it with: cargo build --release -p clasp-service');
    return;
  }

  console.log(`Starting bridge service from: ${servicePath}`);

  try {
    const proc = spawn(servicePath, [], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    setBridgeService(proc);
    console.log(`Bridge service spawned with PID: ${proc.pid}`);

    const rl = readline.createInterface({
      input: proc.stdout,
      crlfDelay: Infinity,
    });

    rl.on('line', (line) => {
      console.log(`[bridge-service stdout] ${line}`);
      try {
        const message = JSON.parse(line);
        handleBridgeMessage(message);
      } catch (e) {
        // Non-JSON output
      }
    });

    proc.stderr.on('data', (data) => {
      console.log(`[bridge-service stderr] ${data.toString().trim()}`);
    });

    proc.on('close', (code) => {
      console.log(`Bridge service exited with code: ${code}`);
      setBridgeService(null);
      setBridgeReady(false);
      const mainWindow = getMainWindow();
      if (mainWindow) {
        mainWindow.webContents.send('bridge-ready', false);
      }
    });

    proc.on('error', (err) => {
      console.error('Bridge service error:', err);
      setBridgeService(null);
      setBridgeReady(false);
      const mainWindow = getMainWindow();
      if (mainWindow) {
        mainWindow.webContents.send('bridge-ready', false);
      }
    });

    setTimeout(() => {
      if (!isBridgeReady()) {
        console.error('Bridge service did not become ready within 5 seconds');
        const mainWindow = getMainWindow();
        if (mainWindow) {
          mainWindow.webContents.send('bridge-ready', false);
        }
      }
    }, 5000);

  } catch (err) {
    console.error('Failed to start bridge service:', err);
  }
}

function stopBridgeService() {
  const bridgeService = getBridgeService();
  if (bridgeService) {
    sendToBridge({ type: 'shutdown' });
    setTimeout(() => {
      const bs = getBridgeService();
      if (bs) {
        bs.kill();
        setBridgeService(null);
      }
    }, 1000);
  }
}

function setLearnMode(active, target = null) {
  learnModeActive = active;
  learnModeTarget = target;
}

function isLearnModeActive() {
  return learnModeActive;
}

function getLearnModeTarget() {
  return learnModeTarget;
}

module.exports = {
  sendToBridge,
  handleBridgeMessage,
  startBridgeService,
  stopBridgeService,
  setLearnMode,
  isLearnModeActive,
  getLearnModeTarget,
};
