const { app } = require('electron');
const path = require('path');
const fs = require('fs');

let logFilePath = null;

function initPersistentLogging() {
  try {
    const logsDir = app.getPath('logs');
    logFilePath = path.join(logsDir, 'clasp-bridge.log');

    if (!fs.existsSync(logsDir)) {
      fs.mkdirSync(logsDir, { recursive: true });
    }

    persistLog('info', 'CLASP Bridge starting', { version: app.getVersion() });
  } catch (e) {
    console.error('Failed to initialize persistent logging:', e);
  }
}

function persistLog(level, message, data = null) {
  if (!logFilePath) return;

  try {
    const entry = JSON.stringify({
      timestamp: new Date().toISOString(),
      level,
      message,
      data,
    });
    fs.appendFileSync(logFilePath, entry + '\n');
  } catch (e) {
    // Silently fail
  }
}

module.exports = { initPersistentLogging, persistLog };
