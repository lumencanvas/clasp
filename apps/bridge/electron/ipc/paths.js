const { app } = require('electron');
const path = require('path');

function getBinaryPath(name) {
  const devPath = path.join(__dirname, '..', '..', '..', '..', 'target', 'release', name);
  const prodPath = path.join(process.resourcesPath || '', 'bin', name);
  const isDev = !app.isPackaged;
  return isDev ? devPath : prodPath;
}

module.exports = { getBinaryPath };
