/**
 * Circuit Breaker Unit Tests
 *
 * Tests for the CircuitBreaker class functionality:
 * - Opens after failure threshold
 * - Transitions to half-open after timeout
 * - Closes after successful request in half-open
 * - Respects max retries
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { CircuitBreaker, CircuitState } from '../../electron/ipc/circuit-breaker.js';

describe('CircuitBreaker', () => {
  let breaker: InstanceType<typeof CircuitBreaker>;

  beforeEach(() => {
    breaker = new CircuitBreaker({
      failureThreshold: 3,
      resetTimeout: 1000, // 1 second for tests
      maxRetries: 10,
      halfOpenMaxAttempts: 1,
    });
  });

  describe('Initial State', () => {
    it('should start in CLOSED state', () => {
      expect(breaker.getState()).toBe(CircuitState.CLOSED);
    });

    it('should allow retries in CLOSED state', () => {
      expect(breaker.shouldRetry()).toBe(true);
    });

    it('should have zero retry count initially', () => {
      expect(breaker.getRetryCount()).toBe(0);
    });
  });

  describe('Failure Threshold', () => {
    it('should remain CLOSED under threshold', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.getState()).toBe(CircuitState.CLOSED);
      expect(breaker.getRetryCount()).toBe(2);
    });

    it('should open after reaching threshold', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.getState()).toBe(CircuitState.OPEN);
    });

    it('should not allow retries when OPEN', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.shouldRetry()).toBe(false);
    });
  });

  describe('Half-Open State', () => {
    it('should transition to HALF_OPEN after reset timeout', async () => {
      // Open the circuit
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.getState()).toBe(CircuitState.OPEN);

      // Wait for reset timeout
      await new Promise(resolve => setTimeout(resolve, 1100));

      // Should transition to HALF_OPEN when shouldRetry is called
      expect(breaker.shouldRetry()).toBe(true);
      expect(breaker.getState()).toBe(CircuitState.HALF_OPEN);
    });

    it('should close on success in HALF_OPEN', async () => {
      // Open the circuit
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();

      // Wait for reset timeout
      await new Promise(resolve => setTimeout(resolve, 1100));
      breaker.shouldRetry(); // Transitions to HALF_OPEN

      // Record success
      breaker.recordSuccess();
      expect(breaker.getState()).toBe(CircuitState.CLOSED);
    });

    it('should re-open on failure in HALF_OPEN', async () => {
      // Open the circuit
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();

      // Wait for reset timeout
      await new Promise(resolve => setTimeout(resolve, 1100));
      breaker.shouldRetry(); // Transitions to HALF_OPEN

      // Record failure
      breaker.recordFailure();
      expect(breaker.getState()).toBe(CircuitState.OPEN);
    });
  });

  describe('Max Retries', () => {
    it('should stop retrying after max retries', () => {
      const maxRetries = 10;

      for (let i = 0; i < maxRetries; i++) {
        breaker.recordFailure();
      }

      expect(breaker.getRetryCount()).toBe(maxRetries);
      expect(breaker.shouldRetry()).toBe(false);
    });

    it('should respect max retries even in CLOSED state', () => {
      const limitedBreaker = new CircuitBreaker({
        failureThreshold: 100, // High threshold
        maxRetries: 5,
      });

      for (let i = 0; i < 5; i++) {
        limitedBreaker.recordFailure();
      }

      expect(limitedBreaker.getState()).toBe(CircuitState.CLOSED);
      expect(limitedBreaker.shouldRetry()).toBe(false);
    });
  });

  describe('Success Recovery', () => {
    it('should reset failure count on success', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.getRetryCount()).toBe(2);

      breaker.recordSuccess();
      expect(breaker.getRetryCount()).toBe(0);
      expect(breaker.getState()).toBe(CircuitState.CLOSED);
    });

    it('should allow retries after success', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.getState()).toBe(CircuitState.OPEN);

      breaker.recordSuccess();
      expect(breaker.shouldRetry()).toBe(true);
    });
  });

  describe('Reset', () => {
    it('should fully reset the circuit breaker', () => {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordFailure();

      breaker.reset();

      expect(breaker.getState()).toBe(CircuitState.CLOSED);
      expect(breaker.getRetryCount()).toBe(0);
      expect(breaker.shouldRetry()).toBe(true);
    });
  });

  describe('Custom Configuration', () => {
    it('should respect custom failure threshold', () => {
      const customBreaker = new CircuitBreaker({ failureThreshold: 5 });

      for (let i = 0; i < 4; i++) {
        customBreaker.recordFailure();
      }
      expect(customBreaker.getState()).toBe(CircuitState.CLOSED);

      customBreaker.recordFailure();
      expect(customBreaker.getState()).toBe(CircuitState.OPEN);
    });

    it('should respect custom max retries', () => {
      const customBreaker = new CircuitBreaker({ maxRetries: 3 });

      customBreaker.recordFailure();
      customBreaker.recordFailure();
      customBreaker.recordFailure();

      expect(customBreaker.shouldRetry()).toBe(false);
    });
  });
});
