const CircuitState = {
  CLOSED: 'CLOSED',
  OPEN: 'OPEN',
  HALF_OPEN: 'HALF_OPEN',
};

class CircuitBreaker {
  constructor(options = {}) {
    this.failureThreshold = options.failureThreshold || 3;
    this.resetTimeout = options.resetTimeout || 30000;
    this.maxRetries = options.maxRetries || 10;
    this.halfOpenMaxAttempts = options.halfOpenMaxAttempts || 1;

    this.state = CircuitState.CLOSED;
    this.failures = 0;
    this.retries = 0;
    this.lastFailure = null;
    this.halfOpenAttempts = 0;
  }

  shouldRetry() {
    if (this.retries >= this.maxRetries) return false;

    switch (this.state) {
      case CircuitState.CLOSED:
        return true;
      case CircuitState.OPEN:
        if (this.lastFailure && Date.now() - this.lastFailure >= this.resetTimeout) {
          this.state = CircuitState.HALF_OPEN;
          this.halfOpenAttempts = 0;
          return true;
        }
        return false;
      case CircuitState.HALF_OPEN:
        return this.halfOpenAttempts < this.halfOpenMaxAttempts;
      default:
        return false;
    }
  }

  recordSuccess() {
    this.failures = 0;
    this.retries = 0;
    this.state = CircuitState.CLOSED;
    this.halfOpenAttempts = 0;
  }

  recordFailure() {
    this.failures++;
    this.retries++;
    this.lastFailure = Date.now();

    if (this.state === CircuitState.HALF_OPEN) {
      this.halfOpenAttempts++;
      if (this.halfOpenAttempts >= this.halfOpenMaxAttempts) {
        this.state = CircuitState.OPEN;
      }
    } else if (this.failures >= this.failureThreshold) {
      this.state = CircuitState.OPEN;
    }
  }

  getState() { return this.state; }
  getRetryCount() { return this.retries; }

  reset() {
    this.state = CircuitState.CLOSED;
    this.failures = 0;
    this.retries = 0;
    this.lastFailure = null;
    this.halfOpenAttempts = 0;
  }
}

function calculateBackoffDelay(attempt, baseDelay = 1000, maxDelay = 30000, jitterFactor = 0.2) {
  const exponentialDelay = baseDelay * Math.pow(2, attempt);
  const cappedDelay = Math.min(exponentialDelay, maxDelay);
  const jitter = cappedDelay * jitterFactor * (Math.random() * 2 - 1);
  return Math.round(cappedDelay + jitter);
}

module.exports = { CircuitBreaker, CircuitState, calculateBackoffDelay };
