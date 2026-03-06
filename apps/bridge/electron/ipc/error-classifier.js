const ErrorType = {
  TIMEOUT: 'TIMEOUT',
  NETWORK: 'NETWORK',
  AUTH: 'AUTH',
  PROTOCOL: 'PROTOCOL',
  UNKNOWN: 'UNKNOWN',
};

function classifyError(error) {
  if (!error) return ErrorType.UNKNOWN;

  const code = error.code || '';
  const message = (error.message || '').toLowerCase();

  if (code === 'ETIMEDOUT' || code === 'ESOCKETTIMEDOUT' ||
      message.includes('timeout') || message.includes('timed out')) {
    return ErrorType.TIMEOUT;
  }

  if (code === 'ECONNREFUSED' || code === 'ENOTFOUND' || code === 'ENETUNREACH' ||
      code === 'ECONNRESET' || code === 'EPIPE' || message.includes('network')) {
    return ErrorType.NETWORK;
  }

  if (message.includes('401') || message.includes('403') ||
      message.includes('unauthorized') || message.includes('forbidden') ||
      message.includes('authentication')) {
    return ErrorType.AUTH;
  }

  if (message.includes('protocol') || message.includes('handshake') ||
      message.includes('version')) {
    return ErrorType.PROTOCOL;
  }

  return ErrorType.UNKNOWN;
}

module.exports = { ErrorType, classifyError };
