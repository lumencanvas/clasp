const { ipcMain, dialog, app } = require('electron');
const path = require('path');
const fs = require('fs');
const { state, getMainWindow, isBridgeReady } = require('./state');
const { sendToBridge, setLearnMode } = require('./bridge-service');
const { startServer, stopServer } = require('./server-manager');

const configPath = path.join(app.getPath('userData'), 'clasp-config.json');

function isPathAllowed(filePath) {
  const resolvedPath = path.resolve(filePath);
  const allowedDirs = [
    app.getPath('userData'),
    app.getPath('documents'),
    app.getPath('downloads'),
    app.getPath('home'),
  ];
  return allowedDirs.some(dir => resolvedPath.startsWith(dir));
}

function registerFileHandlers() {
  ipcMain.handle('get-devices', async () => state.devices);

  ipcMain.handle('get-bridges', async () => state.bridges);

  ipcMain.handle('create-bridge', async (event, config) => {
    if (isBridgeReady()) {
      try {
        sendToBridge({
          type: 'create_bridge',
          id: config.id || null,
          source: config.source,
          source_addr: config.sourceAddr,
          target: config.target,
          target_addr: config.targetAddr,
        });

        const bridge = {
          id: config.id || Date.now().toString(),
          source: config.source,
          sourceAddr: config.sourceAddr,
          target: config.target,
          targetAddr: config.targetAddr,
          active: true,
        };
        state.bridges.push(bridge);
        return bridge;
      } catch (e) {
        console.error('Failed to create bridge:', e);
        throw e;
      }
    }

    const bridge = { id: Date.now().toString(), ...config, active: false };
    state.bridges.push(bridge);
    return bridge;
  });

  ipcMain.handle('delete-bridge', async (event, id) => {
    if (isBridgeReady()) {
      sendToBridge({ type: 'delete_bridge', id });
    }
    state.bridges = state.bridges.filter(b => b.id !== id);
    return true;
  });

  ipcMain.handle('add-server', async (event, address) => {
    const server = {
      id: Date.now().toString(),
      name: `Server @ ${address}`,
      address,
      protocol: 'clasp',
      status: 'available',
    };
    state.devices.push(server);
    getMainWindow()?.webContents.send('device-found', server);
    return server;
  });

  ipcMain.handle('start-server', async (event, config) => {
    try {
      return await startServer(config);
    } catch (err) {
      console.error('Failed to start server:', err);
      const serverId = config.id || Date.now().toString();
      getMainWindow()?.webContents.send('server-status', {
        id: serverId,
        status: 'error',
        error: err.message,
      });
      throw err;
    }
  });

  ipcMain.handle('stop-server', async (event, id) => {
    try {
      const stopped = await stopServer(id);
      const idx = state.devices.findIndex(d => d.id === id);
      if (idx !== -1) state.devices.splice(idx, 1);
      getMainWindow()?.webContents.send('server-status', {
        id,
        status: 'stopped',
      });
      return stopped;
    } catch (err) {
      console.error('Failed to stop server:', err);
      throw err;
    }
  });

  ipcMain.handle('get-server-logs', async (event, id) => {
    const { runningServers } = require('./state');
    const server = runningServers.get(id);
    return server ? server.logs : [];
  });

  ipcMain.handle('send-signal', async (event, { bridgeId, address, value }) => {
    if (isBridgeReady()) {
      sendToBridge({
        type: 'send_signal',
        bridge_id: bridgeId,
        address,
        value,
      });
      return true;
    }
    return false;
  });

  // Learn mode
  ipcMain.handle('start-learn-mode', async (event, target) => {
    setLearnMode(true, target);
    return true;
  });

  ipcMain.handle('stop-learn-mode', async () => {
    setLearnMode(false);
    return true;
  });

  // Configuration
  ipcMain.handle('get-app-version', () => app.getVersion());

  ipcMain.handle('is-first-run', () => {
    try {
      if (!fs.existsSync(configPath)) return true;
      const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      return !config.firstRunComplete;
    } catch (e) {
      return true;
    }
  });

  ipcMain.handle('set-first-run-complete', () => {
    try {
      let config = {};
      if (fs.existsSync(configPath)) {
        config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      }
      config.firstRunComplete = true;
      config.firstRunDate = new Date().toISOString();
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
      return true;
    } catch (e) {
      console.error('Failed to save first run state:', e);
      return false;
    }
  });

  // File dialogs
  ipcMain.handle('show-save-dialog', async (event, options) => {
    const mainWindow = getMainWindow();
    return dialog.showSaveDialog(mainWindow, {
      title: options.title || 'Save Configuration',
      defaultPath: options.defaultPath || 'clasp-config.json',
      filters: options.filters || [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
  });

  ipcMain.handle('show-open-dialog', async (event, options) => {
    const mainWindow = getMainWindow();
    return dialog.showOpenDialog(mainWindow, {
      title: options.title || 'Load Configuration',
      filters: options.filters || [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] },
      ],
      properties: ['openFile'],
    });
  });

  ipcMain.handle('write-file', async (event, { path: filePath, content }) => {
    try {
      if (!isPathAllowed(filePath)) {
        return { success: false, error: 'Access denied: path not in allowed directories' };
      }
      fs.writeFileSync(filePath, content, 'utf8');
      return { success: true };
    } catch (e) {
      return { success: false, error: e.message };
    }
  });

  ipcMain.handle('read-file', async (event, filePath) => {
    try {
      if (!isPathAllowed(filePath)) {
        return { success: false, error: 'Access denied: path not in allowed directories' };
      }
      const content = fs.readFileSync(filePath, 'utf8');
      return { success: true, content };
    } catch (e) {
      return { success: false, error: e.message };
    }
  });
}

module.exports = { registerFileHandlers };
