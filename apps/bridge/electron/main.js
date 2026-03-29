const { app, BrowserWindow } = require('electron');
const path = require('path');
const { setMainWindow, claspMonitors } = require('./ipc/state');
const { initPersistentLogging } = require('./ipc/logging');
const { startBridgeService, stopBridgeService } = require('./ipc/bridge-service');
const { stopAllServers } = require('./ipc/server-manager');
const { registerFileHandlers } = require('./ipc/files');
const { registerHardwareHandlers } = require('./ipc/hardware');
const { registerDiagnosticsHandlers, startStatsBroadcast, stopStatsBroadcast } = require('./ipc/diagnostics');
const { registerAcpHandlers } = require('./ipc/acp');

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1152,
    height: 810,
    minWidth: 900,
    minHeight: 600,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
    titleBarStyle: 'hiddenInset',
    backgroundColor: '#f5f5f4',
    show: false,
  });

  setMainWindow(mainWindow);

  const isDev = !app.isPackaged;

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173').catch(() => {
      mainWindow.loadFile(path.join(__dirname, '../dist/index.html')).catch(() => {
        console.error('Failed to load app - neither Vite dev server nor dist/index.html available');
      });
    });
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
    startStatsBroadcast();
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
    setMainWindow(null);
    stopStatsBroadcast();
  });
}

// Register all IPC handlers
registerFileHandlers();
registerHardwareHandlers();
registerDiagnosticsHandlers();
registerAcpHandlers();

app.whenReady().then(() => {
  console.log('App ready, initializing...');
  initPersistentLogging();
  console.log('Starting bridge service...');
  startBridgeService();
  console.log('Creating window...');
  createWindow();
});

app.on('window-all-closed', async () => {
  await stopAllServers();
  stopBridgeService();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

app.on('will-quit', () => {
  stopStatsBroadcast();

  for (const [id, ws] of claspMonitors) {
    try { ws.close(); } catch (e) { /* ignore */ }
  }
  claspMonitors.clear();

  stopBridgeService();
});

app.on('before-quit', async (event) => {
  event.preventDefault();
  try {
    await Promise.race([
      stopAllServers(),
      new Promise(resolve => setTimeout(resolve, 2000)),
    ]);
  } catch (e) { /* ignore cleanup errors */ }
  app.exit(0);
});
