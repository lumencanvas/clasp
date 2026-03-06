const { ipcMain } = require('electron');
const os = require('os');
const { execSync } = require('child_process');
const WebSocket = require('ws');
const { state, getMainWindow } = require('./state');
const { MSG, encodeClaspFrame, decodeClaspFrame } = require('./clasp-protocol');

async function probeServer(host, port) {
  return new Promise((resolve) => {
    const wsUrl = `ws://${host}:${port}`;
    let ws;
    let resolved = false;
    let welcomeTimeout;
    let connectionTimeout;

    const cleanup = () => {
      if (!resolved) {
        resolved = true;
        if (connectionTimeout) clearTimeout(connectionTimeout);
        if (welcomeTimeout) clearTimeout(welcomeTimeout);
        if (ws) {
          try {
            if (ws.readyState === 1 || ws.readyState === 0) ws.terminate();
          } catch (e) { /* ignore */ }
        }
      }
    };

    connectionTimeout = setTimeout(() => {
      if (!resolved) {
        cleanup();
        resolve(null);
      }
    }, 3000);

    try {
      ws = new WebSocket(wsUrl);
      ws.binaryType = 'nodebuffer';

      ws.on('open', () => {
        if (resolved) return;
        try {
          const hello = encodeClaspFrame({
            type: MSG.HELLO,
            version: 3,
            name: 'CLASP Scanner',
            features: ['param', 'event', 'stream'],
          });
          ws.send(hello);

          welcomeTimeout = setTimeout(() => {
            if (!resolved) {
              const device = {
                id: `discovered-${host}-${port}`,
                name: `CLASP Server (${host}:${port})`,
                host, port,
                address: wsUrl,
                protocol: 'clasp',
                status: 'available',
              };
              cleanup();
              try { ws.close(); } catch (e) { /* ignore */ }
              resolve(device);
            }
          }, 2000);
        } catch (e) {
          cleanup();
          resolve(null);
        }
      });

      ws.on('message', (data) => {
        if (resolved) return;
        try {
          const buffer = Buffer.from(data);
          const msg = decodeClaspFrame(buffer);
          if (msg.type === MSG.WELCOME) {
            if (!resolved) {
              resolved = true;
              if (connectionTimeout) clearTimeout(connectionTimeout);
              if (welcomeTimeout) clearTimeout(welcomeTimeout);
              const device = {
                id: `discovered-${host}-${port}`,
                name: `CLASP Server (${host}:${port})`,
                host, port,
                address: wsUrl,
                protocol: 'clasp',
                status: 'available',
              };
              try { ws.close(); } catch (e) { /* ignore */ }
              resolve(device);
            }
          }
        } catch (e) { /* decode error */ }
      });

      ws.on('error', (err) => {
        if (resolved) return;
        cleanup();
        resolve(null);
      });

      ws.on('close', () => { /* handled by timeout */ });
      ws.on('unexpected-response', () => {
        if (resolved) return;
        cleanup();
        resolve(null);
      });
    } catch (e) {
      if (resolved) return;
      cleanup();
      resolve(null);
    }
  });
}

function registerHardwareHandlers() {
  ipcMain.handle('scan-network', async () => {
    const mainWindow = getMainWindow();
    mainWindow?.webContents.send('scan-started');

    const portsToScan = [7330, 8080, 9000];
    const hosts = ['localhost', '127.0.0.1'];

    try {
      const interfaces = os.networkInterfaces();
      const localIPs = new Set(['localhost', '127.0.0.1']);

      for (const iface of Object.values(interfaces)) {
        for (const config of iface) {
          if (config.family === 'IPv4') {
            localIPs.add(config.address);
            if (!config.internal) {
              const parts = config.address.split('.');
              const subnet = `${parts[0]}.${parts[1]}.${parts[2]}`;
              for (let i = 1; i <= 10; i++) {
                hosts.push(`${subnet}.${i}`);
              }
            }
          }
        }
      }

      localIPs.forEach(ip => {
        if (!hosts.includes(ip)) hosts.push(ip);
      });
    } catch (e) { /* continue with defaults */ }

    const probePromises = [];
    for (const host of hosts) {
      for (const port of portsToScan) {
        probePromises.push(probeServer(host, port));
      }
    }

    const results = await Promise.allSettled(probePromises);
    const discoveredDevices = [];
    const seen = new Set();

    for (const result of results) {
      if (result.status === 'fulfilled' && result.value) {
        const server = result.value;
        const key = `${server.host}:${server.port}`;
        if (!seen.has(key)) {
          seen.add(key);
          discoveredDevices.push(server);
          mainWindow?.webContents.send('device-found', server);
        }
      }
    }

    for (const device of discoveredDevices) {
      const existing = state.devices.find(d => d.id === device.id);
      if (!existing) state.devices.push(device);
    }

    mainWindow?.webContents.send('scan-complete');
    return discoveredDevices;
  });

  ipcMain.handle('list-serial-ports', async () => {
    try {
      const { SerialPort } = require('serialport');
      const ports = await SerialPort.list();
      return ports.map((port) => ({
        path: port.path,
        manufacturer: port.manufacturer || 'Unknown',
        serialNumber: port.serialNumber,
        vendorId: port.vendorId,
        productId: port.productId,
        name: port.manufacturer
          ? `${port.manufacturer} (${port.path})`
          : port.path,
      }));
    } catch (e) {
      console.error('Failed to list serial ports:', e);
      return [];
    }
  });

  ipcMain.handle('list-midi-ports', async () => {
    const ports = { inputs: [], outputs: [] };

    try {
      if (process.platform === 'darwin') {
        try {
          const output = execSync(
            'system_profiler SPMIDIDataType -json 2>/dev/null || echo "{}"',
            { encoding: 'utf8', timeout: 5000 }
          );
          const data = JSON.parse(output);
          if (data.SPMIDIDataType) {
            for (const device of data.SPMIDIDataType) {
              if (device._name) {
                ports.inputs.push({ id: device._name, name: device._name, manufacturer: device.manufacturer || 'Unknown' });
                ports.outputs.push({ id: device._name, name: device._name, manufacturer: device.manufacturer || 'Unknown' });
              }
            }
          }
        } catch (e) {
          const commonPorts = ['IAC Driver Bus 1', 'Network Session 1'];
          for (const name of commonPorts) {
            ports.inputs.push({ id: name, name, manufacturer: 'System' });
            ports.outputs.push({ id: name, name, manufacturer: 'System' });
          }
        }
      } else if (process.platform === 'linux') {
        try {
          const output = execSync('aconnect -l 2>/dev/null || echo ""', {
            encoding: 'utf8', timeout: 5000,
          });
          for (const line of output.split('\n')) {
            const match = line.match(/client (\d+): '([^']+)'/);
            if (match) {
              const [, id, name] = match;
              if (name !== 'System' && name !== 'Midi Through') {
                ports.inputs.push({ id, name, manufacturer: 'ALSA' });
                ports.outputs.push({ id, name, manufacturer: 'ALSA' });
              }
            }
          }
        } catch (e) { /* MIDI not available */ }
      } else if (process.platform === 'win32') {
        ports.inputs.push({ id: 'default', name: 'Default MIDI Input', manufacturer: 'System' });
        ports.outputs.push({ id: 'default', name: 'Default MIDI Output', manufacturer: 'System' });
      }
    } catch (e) {
      console.error('Failed to enumerate MIDI ports:', e);
    }

    if (!ports.inputs.find(p => p.id === 'default')) {
      ports.inputs.unshift({ id: 'default', name: 'System Default', manufacturer: 'System' });
    }
    if (!ports.outputs.find(p => p.id === 'default')) {
      ports.outputs.unshift({ id: 'default', name: 'System Default', manufacturer: 'System' });
    }

    return ports;
  });

  ipcMain.handle('list-network-interfaces', async () => {
    const interfaces = [];
    try {
      const netInterfaces = os.networkInterfaces();
      for (const [name, addrs] of Object.entries(netInterfaces)) {
        for (const addr of addrs) {
          if (addr.family === 'IPv4') {
            interfaces.push({
              name,
              address: addr.address,
              internal: addr.internal,
              label: addr.internal
                ? `${addr.address} (${name} - loopback)`
                : `${addr.address} (${name})`,
            });
          }
        }
      }
    } catch (e) {
      console.error('Failed to list network interfaces:', e);
    }

    interfaces.unshift({
      name: 'all',
      address: '0.0.0.0',
      internal: false,
      label: '0.0.0.0 (All Interfaces)',
    });

    return interfaces;
  });

  ipcMain.handle('test-serial-port', async (event, portPath) => {
    return new Promise((resolve) => {
      try {
        const { SerialPort } = require('serialport');
        const port = new SerialPort({
          path: portPath,
          baudRate: 250000,
          autoOpen: false,
        });

        port.open((err) => {
          if (err) {
            resolve({ success: false, error: err.message });
          } else {
            port.close();
            resolve({ success: true });
          }
        });

        setTimeout(() => {
          try { port.close(); } catch (e) { /* ignore */ }
          resolve({ success: false, error: 'Connection timeout' });
        }, 3000);
      } catch (e) {
        resolve({ success: false, error: e.message });
      }
    });
  });

  ipcMain.handle('test-port-available', async (event, { host, port }) => {
    return new Promise((resolve) => {
      const dgram = require('dgram');
      const socket = dgram.createSocket('udp4');

      socket.on('error', (err) => {
        socket.close();
        resolve({ success: false, error: err.message });
      });

      socket.bind(port, host, () => {
        socket.close();
        resolve({ success: true });
      });

      setTimeout(() => {
        try { socket.close(); } catch (e) { /* ignore */ }
        resolve({ success: false, error: 'Timeout' });
      }, 2000);
    });
  });
}

module.exports = { registerHardwareHandlers };
