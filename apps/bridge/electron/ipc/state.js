// Shared mutable state for all IPC modules
// Single source of truth for runtime state across the electron main process

let mainWindow = null;

const runningServers = new Map(); // id -> { process, config, status, logs, stats, port, tokenFilePath }
const claspMonitors = new Map(); // serverId -> WebSocket
const bridgeRouterConnections = new Map(); // bridgeId -> { ws, routerId, routerAddress, token, welcomed }
const circuitBreakers = new Map(); // bridgeId -> CircuitBreaker

const MAX_LOG_LINES = 500;

let bridgeService = null;
let bridgeReady = false;

const state = {
  devices: [],
  bridges: [],
};

function getMainWindow() {
  return mainWindow;
}

function setMainWindow(win) {
  mainWindow = win;
}

function getBridgeService() {
  return bridgeService;
}

function setBridgeService(proc) {
  bridgeService = proc;
}

function isBridgeReady() {
  return bridgeReady;
}

function setBridgeReady(ready) {
  bridgeReady = ready;
}

module.exports = {
  runningServers,
  claspMonitors,
  bridgeRouterConnections,
  circuitBreakers,
  MAX_LOG_LINES,
  state,
  getMainWindow,
  setMainWindow,
  getBridgeService,
  setBridgeService,
  isBridgeReady,
  setBridgeReady,
};
