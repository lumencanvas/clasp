const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('signalflow', {
  // Device discovery
  getDevices: () => ipcRenderer.invoke('get-devices'),
  scanNetwork: () => ipcRenderer.invoke('scan-network'),
  addServer: (address) => ipcRenderer.invoke('add-server', address),

  // Bridge management
  getBridges: () => ipcRenderer.invoke('get-bridges'),
  createBridge: (config) => ipcRenderer.invoke('create-bridge', config),
  deleteBridge: (id) => ipcRenderer.invoke('delete-bridge', id),

  // Events
  onDeviceFound: (callback) => {
    ipcRenderer.on('device-found', (event, device) => callback(device));
  },
  onDeviceUpdated: (callback) => {
    ipcRenderer.on('device-updated', (event, device) => callback(device));
  },
  onDeviceLost: (callback) => {
    ipcRenderer.on('device-lost', (event, deviceId) => callback(deviceId));
  },
  onSignal: (callback) => {
    ipcRenderer.on('signal', (event, signal) => callback(signal));
  },
  onScanStarted: (callback) => {
    ipcRenderer.on('scan-started', () => callback());
  },
  onScanComplete: (callback) => {
    ipcRenderer.on('scan-complete', () => callback());
  },
});
