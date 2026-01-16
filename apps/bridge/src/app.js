/**
 * SignalFlow Bridge - Main Application v2
 * Full-featured protocol mapping and bridging
 */

// State
const state = {
  devices: [],
  bridges: [],
  mappings: [],
  signals: [],
  signalRate: 0,
  paused: false,
  scanning: false,
  activeTab: 'bridges',
  learnMode: false,
  learnTarget: null, // 'source' or 'target'
  editingMapping: null,
  monitorFilter: '',
};

// Signal rate counter (at module level for hoisting)
let signalCount = 0;

// DOM Elements cache
const $ = (id) => document.getElementById(id);
const $$ = (sel) => document.querySelectorAll(sel);

// Icons (SVG strings)
const icons = {
  play: '<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>',
  pause: '<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>',
  scan: '<svg class="icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg>',
  delete: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>',
  arrow: '<svg class="bridge-arrow" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>',
  bridge: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 12h16M8 8l-4 4 4 4M16 8l4 4-4 4"/></svg>',
  mapping: '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="6" cy="12" r="3"/><circle cx="18" cy="12" r="3"/><line x1="9" y1="12" x2="15" y2="12"/></svg>',
};

// Protocol display names
const protocolNames = {
  osc: 'OSC',
  midi: 'MIDI',
  artnet: 'Art-Net',
  dmx: 'DMX',
  signalflow: 'SignalFlow',
};

// Default addresses for protocols
const defaultAddresses = {
  osc: '0.0.0.0:8000',
  midi: 'default',
  artnet: '0.0.0.0:6454',
  dmx: '/dev/ttyUSB0',
  signalflow: 'localhost:7330',
};

// Initialize application
async function init() {
  console.log('SignalFlow Bridge v2 initializing...');

  // Load saved data
  loadMappingsFromStorage();

  // Load data from backend
  await Promise.all([loadDevices(), loadBridges()]);

  // Set up UI
  setupTabs();
  setupModals();
  setupEventListeners();
  setupProtocolFieldSwitching();
  setupTransformParams();
  setupLearnMode();

  // Initial render
  renderDevices();
  renderBridges();
  renderMappings();
  renderSignalMonitor();
  updateStatus();

  // Start rate counter
  setInterval(updateSignalRate, 1000);

  console.log('SignalFlow Bridge initialized');
}

// ============================================
// Data Loading
// ============================================

async function loadDevices() {
  try {
    if (window.signalflow) {
      state.devices = await window.signalflow.getDevices();
    }
  } catch (e) {
    console.error('Failed to load devices:', e);
  }
}

async function loadBridges() {
  try {
    if (window.signalflow) {
      state.bridges = await window.signalflow.getBridges();
    }
  } catch (e) {
    console.error('Failed to load bridges:', e);
  }
}

function loadMappingsFromStorage() {
  try {
    const saved = localStorage.getItem('signalflow-mappings');
    if (saved) {
      state.mappings = JSON.parse(saved);
    }
  } catch (e) {
    console.error('Failed to load mappings:', e);
  }
}

function saveMappingsToStorage() {
  try {
    localStorage.setItem('signalflow-mappings', JSON.stringify(state.mappings));
  } catch (e) {
    console.error('Failed to save mappings:', e);
  }
}

// ============================================
// Tab Management
// ============================================

function setupTabs() {
  const tabs = $$('.tab');
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const tabName = tab.dataset.tab;
      switchTab(tabName);
    });
  });
}

function switchTab(tabName) {
  state.activeTab = tabName;

  // Update tab buttons
  $$('.tab').forEach(tab => {
    tab.classList.toggle('active', tab.dataset.tab === tabName);
  });

  // Update panels
  $$('.tab-panel').forEach(panel => {
    panel.classList.toggle('active', panel.id === `panel-${tabName}`);
  });
}

// ============================================
// Modal Management
// ============================================

function setupModals() {
  // Close buttons
  $$('[data-close-modal]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const modal = e.target.closest('dialog');
      modal?.close();
      resetLearnMode();
    });
  });

  // Click outside to close
  $$('.modal').forEach(modal => {
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        modal.close();
        resetLearnMode();
      }
    });
  });
}

// ============================================
// Protocol Field Switching
// ============================================

function setupProtocolFieldSwitching() {
  // Source protocol in mapping modal
  $('mapping-source-protocol')?.addEventListener('change', (e) => {
    updateProtocolFields('source', e.target.value);
  });

  // Target protocol in mapping modal
  $('mapping-target-protocol')?.addEventListener('change', (e) => {
    updateProtocolFields('target', e.target.value);
  });

  // Source protocol in bridge modal
  $('bridge-source')?.addEventListener('change', (e) => {
    updateBridgeAddressPlaceholder('source', e.target.value);
  });

  // Target protocol in bridge modal
  $('bridge-target')?.addEventListener('change', (e) => {
    updateBridgeAddressPlaceholder('target', e.target.value);
  });
}

function updateProtocolFields(side, protocol) {
  // Hide all protocol-specific fields for this side
  const oscFields = $(`${side}-osc-fields`);
  const midiFields = $(`${side}-midi-fields`);
  const dmxFields = $(`${side}-dmx-fields`);

  oscFields?.classList.add('hidden');
  midiFields?.classList.add('hidden');
  dmxFields?.classList.add('hidden');

  // Show appropriate fields
  switch (protocol) {
    case 'osc':
      oscFields?.classList.remove('hidden');
      break;
    case 'midi':
      midiFields?.classList.remove('hidden');
      break;
    case 'artnet':
    case 'dmx':
      dmxFields?.classList.remove('hidden');
      break;
  }
}

function updateBridgeAddressPlaceholder(side, protocol) {
  const input = $(`bridge-${side}-addr`);
  if (input) {
    input.placeholder = defaultAddresses[protocol] || '';
  }
}

// ============================================
// Transform Parameters
// ============================================

function setupTransformParams() {
  $('mapping-transform')?.addEventListener('change', (e) => {
    const transform = e.target.value;

    // Hide all transform params
    $('scale-params')?.classList.add('hidden');
    $('threshold-params')?.classList.add('hidden');

    // Show appropriate params
    switch (transform) {
      case 'scale':
        $('scale-params')?.classList.remove('hidden');
        break;
      case 'threshold':
        $('threshold-params')?.classList.remove('hidden');
        break;
    }
  });
}

// ============================================
// Learn Mode
// ============================================

function setupLearnMode() {
  // Global learn button
  $('learn-btn')?.addEventListener('click', () => {
    toggleLearnMode('source');
  });

  // Source learn button in modal
  $('learn-source-btn')?.addEventListener('click', () => {
    toggleLearnMode('source');
  });
}

function toggleLearnMode(target) {
  if (state.learnMode && state.learnTarget === target) {
    // Turn off
    resetLearnMode();
  } else {
    // Turn on
    state.learnMode = true;
    state.learnTarget = target;

    // Visual feedback
    $('learn-btn')?.classList.add('learn-active');
    $('learn-source-btn')?.classList.add('learn-active');
  }
}

function resetLearnMode() {
  state.learnMode = false;
  state.learnTarget = null;
  $('learn-btn')?.classList.remove('learn-active');
  $('learn-source-btn')?.classList.remove('learn-active');
}

function handleLearnedSignal(signal) {
  if (!state.learnMode) return false;

  const modal = $('mapping-modal');
  if (!modal?.open) {
    // Open the modal and populate
    openMappingModal();
  }

  // Determine protocol from signal
  const protocol = detectProtocol(signal);

  if (state.learnTarget === 'source') {
    const protocolSelect = $('mapping-source-protocol');
    if (protocolSelect) {
      protocolSelect.value = protocol;
      updateProtocolFields('source', protocol);
    }

    // Fill in address
    if (protocol === 'osc') {
      const addressInput = document.querySelector('[name="sourceAddress"]');
      if (addressInput) addressInput.value = signal.address;
    } else if (protocol === 'midi') {
      // Parse MIDI info from signal
      const channelInput = document.querySelector('[name="sourceMidiChannel"]');
      const numberInput = document.querySelector('[name="sourceMidiNumber"]');
      if (channelInput && signal.channel) channelInput.value = signal.channel;
      if (numberInput && signal.note !== undefined) numberInput.value = signal.note;
      if (numberInput && signal.cc !== undefined) numberInput.value = signal.cc;
    }
  }

  resetLearnMode();
  return true;
}

function detectProtocol(signal) {
  if (signal.address?.startsWith('/')) return 'osc';
  if (signal.channel !== undefined) return 'midi';
  if (signal.universe !== undefined) return 'dmx';
  return 'osc'; // default
}

// ============================================
// Event Listeners
// ============================================

function setupEventListeners() {
  // Scan button
  $('scan-btn')?.addEventListener('click', handleScan);

  // Add server button
  $('add-server-btn')?.addEventListener('click', () => {
    $('server-modal')?.showModal();
  });

  // Server form
  $('server-form')?.addEventListener('submit', handleAddServer);

  // Add bridge button
  $('add-bridge-btn')?.addEventListener('click', () => {
    $('bridge-modal')?.showModal();
  });

  // Bridge form
  $('bridge-form')?.addEventListener('submit', handleCreateBridge);

  // Add mapping button
  $('add-mapping-btn')?.addEventListener('click', () => {
    state.editingMapping = null;
    openMappingModal();
  });

  // Mapping form
  $('mapping-form')?.addEventListener('submit', handleCreateMapping);

  // Monitor controls
  $('pause-btn')?.addEventListener('click', togglePause);
  $('clear-btn')?.addEventListener('click', clearSignals);
  $('monitor-filter')?.addEventListener('input', (e) => {
    state.monitorFilter = e.target.value.toLowerCase();
    renderSignalMonitor();
  });

  // IPC events
  if (window.signalflow) {
    window.signalflow.onDeviceFound?.((device) => {
      upsertDevice(device);
      renderDevices();
      updateStatus();
    });

    window.signalflow.onDeviceUpdated?.((device) => {
      upsertDevice(device);
      renderDevices();
      updateStatus();
    });

    window.signalflow.onDeviceLost?.((deviceId) => {
      state.devices = state.devices.filter(d => d.id !== deviceId);
      renderDevices();
      updateStatus();
    });

    window.signalflow.onSignal?.((signal) => {
      // Check learn mode first
      if (handleLearnedSignal(signal)) return;

      // Otherwise add to monitor
      if (!state.paused) {
        addSignal(signal);
        applyMappings(signal);
      }
    });

    window.signalflow.onScanStarted?.(() => {
      state.scanning = true;
      updateScanButton();
    });

    window.signalflow.onScanComplete?.(() => {
      state.scanning = false;
      updateScanButton();
      loadDevices().then(renderDevices);
    });
  }
}

function upsertDevice(device) {
  const idx = state.devices.findIndex(d => d.id === device.id);
  if (idx >= 0) {
    state.devices[idx] = device;
  } else {
    state.devices.push(device);
  }
}

// ============================================
// Event Handlers
// ============================================

async function handleScan() {
  if (state.scanning) return;

  state.scanning = true;
  updateScanButton();

  try {
    if (window.signalflow) {
      await window.signalflow.scanNetwork();
    } else {
      await new Promise(r => setTimeout(r, 1500));
    }
  } finally {
    state.scanning = false;
    updateScanButton();
    await loadDevices();
    renderDevices();
  }
}

function updateScanButton() {
  const btn = $('scan-btn');
  if (!btn) return;

  if (state.scanning) {
    btn.innerHTML = `<svg class="icon spinning" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg> SCANNING`;
    btn.disabled = true;
  } else {
    btn.innerHTML = `${icons.scan} SCAN`;
    btn.disabled = false;
  }
}

async function handleAddServer(e) {
  e.preventDefault();
  const form = e.target;
  const addr = new FormData(form).get('address');

  if (!addr) return;

  try {
    if (window.signalflow) {
      await window.signalflow.addServer(addr);
    } else {
      state.devices.push({
        id: Date.now().toString(),
        name: `Server @ ${addr}`,
        address: addr,
        protocol: 'signalflow',
        status: 'connected'
      });
    }
    renderDevices();
    updateStatus();
    $('server-modal')?.close();
    form.reset();
  } catch (err) {
    console.error('Failed to add server:', err);
  }
}

async function handleCreateBridge(e) {
  e.preventDefault();
  const form = e.target;
  const data = new FormData(form);

  const config = {
    source: data.get('source'),
    sourceAddr: data.get('sourceAddr') || defaultAddresses[data.get('source')],
    target: data.get('target'),
    targetAddr: data.get('targetAddr') || defaultAddresses[data.get('target')],
  };

  try {
    let bridge;
    if (window.signalflow) {
      bridge = await window.signalflow.createBridge(config);
    } else {
      bridge = { id: Date.now().toString(), ...config, active: true };
    }
    state.bridges.push(bridge);
    renderBridges();
    $('bridge-modal')?.close();
    form.reset();
  } catch (err) {
    console.error('Failed to create bridge:', err);
  }
}

function openMappingModal() {
  const modal = $('mapping-modal');
  if (!modal) return;

  // Reset form
  $('mapping-form')?.reset();

  // Reset field visibility to defaults
  updateProtocolFields('source', 'osc');
  updateProtocolFields('target', 'midi');

  // Reset transform params
  $('scale-params')?.classList.add('hidden');
  $('threshold-params')?.classList.add('hidden');

  modal.showModal();
}

function handleCreateMapping(e) {
  e.preventDefault();
  const form = e.target;
  const data = new FormData(form);

  const mapping = {
    id: state.editingMapping || Date.now().toString(),
    enabled: true,
    source: {
      protocol: data.get('sourceProtocol'),
      address: data.get('sourceAddress') || null,
      midiType: data.get('sourceMidiType') || null,
      midiChannel: parseInt(data.get('sourceMidiChannel')) || null,
      midiNumber: data.get('sourceMidiNumber') ? parseInt(data.get('sourceMidiNumber')) : null,
      dmxUniverse: parseInt(data.get('sourceDmxUniverse')) || null,
      dmxChannel: parseInt(data.get('sourceDmxChannel')) || null,
    },
    target: {
      protocol: data.get('targetProtocol'),
      address: data.get('targetAddress') || null,
      midiType: data.get('targetMidiType') || null,
      midiChannel: parseInt(data.get('targetMidiChannel')) || null,
      midiNumber: parseInt(data.get('targetMidiNumber')) || null,
      dmxUniverse: parseInt(data.get('targetDmxUniverse')) || null,
      dmxChannel: parseInt(data.get('targetDmxChannel')) || null,
    },
    transform: {
      type: data.get('transform'),
      scaleInMin: parseFloat(data.get('scaleInMin')) || 0,
      scaleInMax: parseFloat(data.get('scaleInMax')) || 1,
      scaleOutMin: parseFloat(data.get('scaleOutMin')) || 0,
      scaleOutMax: parseFloat(data.get('scaleOutMax')) || 127,
      threshold: parseFloat(data.get('threshold')) || 0.5,
    },
  };

  // Add or update
  if (state.editingMapping) {
    const idx = state.mappings.findIndex(m => m.id === state.editingMapping);
    if (idx >= 0) state.mappings[idx] = mapping;
  } else {
    state.mappings.push(mapping);
  }

  saveMappingsToStorage();
  renderMappings();
  updateMappingCount();
  $('mapping-modal')?.close();
  state.editingMapping = null;
}

function deleteMapping(id) {
  state.mappings = state.mappings.filter(m => m.id !== id);
  saveMappingsToStorage();
  renderMappings();
  updateMappingCount();
}

async function deleteBridge(id) {
  try {
    if (window.signalflow) {
      await window.signalflow.deleteBridge(id);
    }
    state.bridges = state.bridges.filter(b => b.id !== id);
    renderBridges();
  } catch (err) {
    console.error('Failed to delete bridge:', err);
  }
}

function togglePause() {
  state.paused = !state.paused;
  const btn = $('pause-btn');
  if (btn) {
    btn.innerHTML = state.paused ? icons.play : icons.pause;
    btn.title = state.paused ? 'Resume' : 'Pause';
  }
}

function clearSignals() {
  state.signals = [];
  renderSignalMonitor();
}

// ============================================
// Signal Processing & Mapping
// ============================================

function addSignal(signal) {
  signalCount++;

  state.signals.unshift({
    ...signal,
    timestamp: Date.now(),
  });

  if (state.signals.length > 200) {
    state.signals = state.signals.slice(0, 200);
  }

  renderSignalMonitor();
}

function applyMappings(signal) {
  for (const mapping of state.mappings) {
    if (!mapping.enabled) continue;
    if (!matchesSource(signal, mapping.source)) continue;

    // Get value from signal
    let value = extractValue(signal, mapping.source);

    // Apply transform
    value = applyTransform(value, mapping.transform);

    // Send to target (would need IPC support)
    // For now just log it
    console.log(`Mapping ${mapping.id}: ${value} -> ${mapping.target.protocol}`);
  }
}

function matchesSource(signal, source) {
  switch (source.protocol) {
    case 'osc':
      if (!signal.address) return false;
      if (source.address) {
        // Support wildcards
        const pattern = source.address.replace(/\*/g, '.*');
        return new RegExp(`^${pattern}$`).test(signal.address);
      }
      return true;

    case 'midi':
      if (source.midiChannel && signal.channel !== source.midiChannel) return false;
      if (source.midiNumber !== null && signal.note !== source.midiNumber && signal.cc !== source.midiNumber) return false;
      return true;

    case 'dmx':
    case 'artnet':
      if (source.dmxUniverse !== null && signal.universe !== source.dmxUniverse) return false;
      if (source.dmxChannel !== null && signal.channel !== source.dmxChannel) return false;
      return true;

    default:
      return false;
  }
}

function extractValue(signal, source) {
  if (typeof signal.value === 'number') return signal.value;
  if (signal.velocity !== undefined) return signal.velocity / 127;
  if (signal.value !== undefined) return signal.value;
  return 0;
}

function applyTransform(value, transform) {
  switch (transform.type) {
    case 'direct':
      return value;

    case 'scale':
      // Map from input range to output range
      const normalized = (value - transform.scaleInMin) / (transform.scaleInMax - transform.scaleInMin);
      return transform.scaleOutMin + normalized * (transform.scaleOutMax - transform.scaleOutMin);

    case 'invert':
      return 1 - value;

    case 'toggle':
      return value > 0.5 ? 1 : 0;

    case 'threshold':
      return value >= transform.threshold ? 1 : 0;

    default:
      return value;
  }
}

// ============================================
// Rendering
// ============================================

function renderDevices() {
  const list = $('device-list');
  if (!list) return;

  if (state.devices.length === 0) {
    list.innerHTML = `
      <div class="empty-state-small">
        <span class="empty-state-text">No devices found</span>
      </div>
    `;
    return;
  }

  list.innerHTML = state.devices.map(device => `
    <div class="device-item" data-id="${device.id}">
      <span class="status-dot ${device.status || 'available'}"></span>
      <span class="device-name">${device.name}</span>
    </div>
  `).join('');

  // Update badge
  const badge = $('device-badge');
  if (badge) badge.textContent = state.devices.length;
}

function renderBridges() {
  const list = $('bridge-list');
  if (!list) return;

  if (state.bridges.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">${icons.bridge}</div>
        <div class="empty-state-text">No bridges configured</div>
        <div class="empty-state-hint">Create a bridge to connect protocols</div>
      </div>
    `;
    return;
  }

  list.innerHTML = state.bridges.map(bridge => `
    <div class="bridge-card" data-id="${bridge.id}">
      <div class="bridge-endpoint">
        <span class="bridge-endpoint-label">${protocolNames[bridge.source] || bridge.source}</span>
        <span class="bridge-endpoint-value">${bridge.sourceAddr || '--'}</span>
      </div>
      ${icons.arrow}
      <div class="bridge-endpoint">
        <span class="bridge-endpoint-label">${protocolNames[bridge.target] || bridge.target}</span>
        <span class="bridge-endpoint-value">${bridge.targetAddr || '--'}</span>
      </div>
      <div class="bridge-actions">
        <button class="btn btn-sm btn-delete" onclick="deleteBridge('${bridge.id}')" title="Delete">
          ${icons.delete}
        </button>
      </div>
    </div>
  `).join('');
}

function renderMappings() {
  const list = $('mapping-list');
  if (!list) return;

  if (state.mappings.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">${icons.mapping}</div>
        <div class="empty-state-text">No mappings configured</div>
        <div class="empty-state-hint">Create mappings to route signals between protocols</div>
      </div>
    `;
    return;
  }

  list.innerHTML = state.mappings.map(mapping => `
    <div class="mapping-item" data-id="${mapping.id}">
      <div class="mapping-source">
        <span class="mapping-protocol">${protocolNames[mapping.source.protocol]}</span>
        <span class="mapping-address">${formatMappingEndpoint(mapping.source)}</span>
      </div>
      <span class="mapping-transform-badge">${formatTransform(mapping.transform)}</span>
      <div class="mapping-target">
        <span class="mapping-protocol">${protocolNames[mapping.target.protocol]}</span>
        <span class="mapping-address">${formatMappingEndpoint(mapping.target)}</span>
      </div>
      <div class="bridge-actions">
        <button class="btn btn-sm btn-delete" onclick="deleteMapping('${mapping.id}')" title="Delete">
          ${icons.delete}
        </button>
      </div>
    </div>
  `).join('');
}

function formatMappingEndpoint(endpoint) {
  switch (endpoint.protocol) {
    case 'osc':
      return endpoint.address || '/*';
    case 'midi':
      const type = endpoint.midiType || 'note';
      const ch = endpoint.midiChannel || '*';
      const num = endpoint.midiNumber !== null ? endpoint.midiNumber : '*';
      return `Ch${ch} ${type.toUpperCase()} ${num}`;
    case 'dmx':
    case 'artnet':
      const uni = endpoint.dmxUniverse !== null ? endpoint.dmxUniverse : '*';
      const chan = endpoint.dmxChannel !== null ? endpoint.dmxChannel : '*';
      return `U${uni} Ch${chan}`;
    default:
      return '--';
  }
}

function formatTransform(transform) {
  switch (transform.type) {
    case 'direct': return '→';
    case 'scale': return `${transform.scaleInMin}-${transform.scaleInMax} → ${transform.scaleOutMin}-${transform.scaleOutMax}`;
    case 'invert': return '↔ INV';
    case 'toggle': return '⊡ TOG';
    case 'threshold': return `≥${transform.threshold}`;
    default: return '→';
  }
}

function renderSignalMonitor() {
  const monitor = $('signal-monitor');
  if (!monitor) return;

  // Filter signals
  let signals = state.signals;
  if (state.monitorFilter) {
    signals = signals.filter(s =>
      (s.address && s.address.toLowerCase().includes(state.monitorFilter)) ||
      (s.bridgeId && s.bridgeId.toLowerCase().includes(state.monitorFilter))
    );
  }

  if (signals.length === 0) {
    monitor.innerHTML = `
      <div class="signal-empty">
        <span>${state.monitorFilter ? 'No matching signals' : 'Waiting for signals...'}</span>
      </div>
    `;
    return;
  }

  monitor.innerHTML = signals.slice(0, 100).map(s => {
    const val = typeof s.value === 'number' ? s.value : 0;
    const percent = Math.min(100, Math.max(0, Math.abs(val) * 100));
    const displayVal = formatSignalValue(s.value);

    return `
      <div class="signal-item">
        <span class="signal-address">${s.address || s.bridgeId || '--'}</span>
        <span class="signal-value">${displayVal}</span>
        <div class="signal-bar">
          <div class="signal-bar-fill" style="width: ${percent}%"></div>
        </div>
      </div>
    `;
  }).join('');
}

function formatSignalValue(value) {
  if (typeof value === 'number') {
    return value % 1 === 0 ? value.toString() : value.toFixed(3);
  }
  if (typeof value === 'boolean') {
    return value ? 'ON' : 'OFF';
  }
  if (Array.isArray(value)) {
    return `[${value.length}]`;
  }
  if (typeof value === 'object') {
    return '{...}';
  }
  return String(value);
}

function updateStatus() {
  const connected = state.devices.filter(d => d.status === 'connected').length;

  const deviceCount = $('device-count');
  if (deviceCount) deviceCount.textContent = connected;

  const indicator = $('status-indicator');
  if (indicator) {
    indicator.className = connected > 0 ? 'status-indicator connected' : 'status-indicator';
  }
}

function updateMappingCount() {
  const count = $('mapping-count');
  if (count) count.textContent = state.mappings.length;
}

// Signal rate tracking
function updateSignalRate() {
  state.signalRate = signalCount;
  signalCount = 0;

  const rateEl = $('signal-rate');
  if (rateEl) rateEl.textContent = state.signalRate;

  const rateStat = $('rate-stat');
  if (rateStat) rateStat.textContent = `${state.signalRate}/s`;
}

// ============================================
// Global Functions (for onclick handlers)
// ============================================

window.deleteBridge = deleteBridge;
window.deleteMapping = deleteMapping;

// ============================================
// Initialize
// ============================================

document.addEventListener('DOMContentLoaded', init);
