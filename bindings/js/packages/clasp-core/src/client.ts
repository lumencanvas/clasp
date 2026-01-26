/**
 * CLASP Client Library for JavaScript/TypeScript
 *
 * Provides a high-level client for the CLASP (Creative Low-Latency Application Streaming Protocol).
 * Works in both Node.js and browser environments.
 *
 * @example Basic Usage
 * ```typescript
 * import { Clasp } from '@clasp-to/core';
 *
 * const client = new Clasp('ws://localhost:7330');
 * await client.connect();
 *
 * // Subscribe to changes
 * client.on('/lumen/layer/*\/opacity', (value, address) => {
 *   console.log(`${address} = ${value}`);
 * });
 *
 * // Set a value
 * client.set('/lumen/layer/0/opacity', 0.75);
 * ```
 *
 * @example Builder Pattern
 * ```typescript
 * const client = Clasp.builder('ws://localhost:7330')
 *   .withName('my-app')
 *   .withFeatures(['param', 'event', 'stream'])
 *   .build();
 *
 * await client.connect();
 * ```
 *
 * @module
 */

import { encode, decode, encodeMessage } from './codec';
import {
  Message,
  Value,
  ConnectOptions,
  SubscriptionCallback,
  Unsubscribe,
  QoS,
  PROTOCOL_VERSION,
  WS_SUBPROTOCOL,
  SetMessage,
  SubscribeMessage,
  HelloMessage,
  WelcomeMessage,
  SnapshotMessage,
  PublishMessage,
  ParamValue,
  AckMessage,
  ErrorMessage,
  AnnounceMessage,
  SyncMessage,
  ResultMessage,
  BundleMessage,
  SignalDefinition,
} from './types';
import { ClaspBuilder } from './builder';

/**
 * Pattern matching for subscriptions
 */
function matchPattern(pattern: string, address: string): boolean {
  const regex = pattern
    .replace(/\*\*/g, '§§')
    .replace(/\*/g, '[^/]+')
    .replace(/§§/g, '.*');
  return new RegExp(`^${regex}$`).test(address);
}

/**
 * CLASP client for real-time communication with CLASP routers.
 *
 * The Clasp class provides methods for:
 * - Connecting to CLASP servers via WebSocket
 * - Subscribing to address patterns with wildcard support
 * - Setting persistent parameter values
 * - Emitting one-shot events
 * - Streaming high-rate data
 * - Sending atomic bundles of messages
 *
 * @example
 * ```typescript
 * const clasp = new Clasp('ws://localhost:7330');
 * await clasp.connect();
 *
 * // Subscribe with wildcards
 * clasp.on('/sensor/*', (value, address) => {
 *   console.log(`${address}: ${value}`);
 * });
 *
 * // Set a parameter
 * clasp.set('/control/volume', 0.8);
 *
 * // Emit an event
 * clasp.emit('/cue/trigger', { cue: 'intro' });
 *
 * // Stream data
 * clasp.stream('/sensor/temperature', 23.5);
 * ```
 */
export class Clasp {
  private url: string;
  private options: ConnectOptions;
  private ws: WebSocket | null = null;
  private sessionId: string | null = null;
  private _connected = false;
  private params = new Map<string, Value>();
  private subscriptions = new Map<number, { pattern: string; callback: SubscriptionCallback }>();
  private nextSubId = 1;
  private serverTimeOffset = 0;
  private pendingGets = new Map<string, (value: Value) => void>();
  private signals = new Map<string, SignalDefinition>();
  private lastError: ErrorMessage | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private intentionallyClosed = false;

  // Event callbacks
  private onConnectCallbacks: (() => void)[] = [];
  private onDisconnectCallbacks: ((reason?: string) => void)[] = [];
  private onErrorCallbacks: ((error: Error) => void)[] = [];
  private onReconnectCallbacks: ((attempt: number) => void)[] = [];

  constructor(url: string, options: ConnectOptions = {}) {
    this.url = url;
    this.options = {
      name: 'CLASP JS Client',
      features: ['param', 'event', 'stream'],
      reconnect: true,
      reconnectInterval: 5000,
      ...options,
    };
  }

  /**
   * Create a builder
   */
  static builder(url: string): ClaspBuilder {
    return new ClaspBuilder(url);
  }

  /**
   * Connect to server.
   *
   * @param timeout - Connection timeout in milliseconds (default: 10000 or options.connectionTimeout)
   * @throws Error if connection times out or fails
   */
  async connect(timeout?: number): Promise<void> {
    const effectiveTimeout = timeout ?? this.options.connectionTimeout ?? 10000;
    return new Promise((resolve, reject) => {
      let connectionTimeout: ReturnType<typeof setTimeout> | null = null;
      let resolved = false;

      const cleanup = () => {
        if (connectionTimeout) {
          clearTimeout(connectionTimeout);
          connectionTimeout = null;
        }
      };

      // Set up connection timeout
      connectionTimeout = setTimeout(() => {
        if (!resolved) {
          resolved = true;
          cleanup();
          this.ws?.close();
          this.ws = null;
          const error = new Error(`Connection timeout after ${effectiveTimeout}ms`);
          this.onErrorCallbacks.forEach((cb) => cb(error));
          reject(error);
        }
      }, effectiveTimeout);

      try {
        this.ws = new WebSocket(this.url, WS_SUBPROTOCOL);
        this.ws.binaryType = 'arraybuffer';

        this.ws.onopen = () => {
          this.sendHello();
        };

        this.ws.onmessage = (event) => {
          const data = new Uint8Array(event.data as ArrayBuffer);
          try {
            const message = decode(data);
            this.handleMessage(message);

            if (message.type === 'WELCOME' && !resolved) {
              resolved = true;
              cleanup();
              this._connected = true;
              resolve();
              this.onConnectCallbacks.forEach((cb) => cb());
            }
          } catch (e) {
            console.warn('Decode error:', e);
          }
        };

        this.ws.onerror = (event) => {
          if (!resolved) {
            resolved = true;
            cleanup();
            const error = new Error('WebSocket error');
            this.onErrorCallbacks.forEach((cb) => cb(error));
            reject(error);
          }
        };

        this.ws.onclose = (event) => {
          this._connected = false;
          this.onDisconnectCallbacks.forEach((cb) => cb(event.reason));

          // Reject if not yet resolved (e.g., closed before WELCOME)
          if (!resolved) {
            resolved = true;
            cleanup();
            reject(new Error(event.reason || 'Connection closed before WELCOME'));
          }

          // Reconnect if enabled and not intentionally closed
          if (this.options.reconnect && !this.intentionallyClosed) {
            this.scheduleReconnect();
          }
        };
      } catch (e) {
        resolved = true;
        cleanup();
        reject(e);
      }
    });
  }

  /**
   * Check if connected
   */
  get connected(): boolean {
    return this._connected;
  }

  /**
   * Get session ID
   */
  get session(): string | null {
    return this.sessionId;
  }

  /**
   * Get current server time (microseconds)
   */
  time(): number {
    return Date.now() * 1000 + this.serverTimeOffset;
  }

  /**
   * Subscribe to an address pattern.
   *
   * Patterns support wildcards:
   * - `*` matches exactly one path segment: `/a/*\/c` matches `/a/b/c`
   * - `**` matches any number of segments: `/a/**` matches `/a/b/c/d`
   *
   * @param pattern - Address pattern to subscribe to (supports wildcards)
   * @param callback - Function called when matching values are received
   * @param options - Optional subscription options
   * @param options.maxRate - Maximum updates per second (rate limiting)
   * @param options.epsilon - Minimum change threshold for numeric values
   * @returns Unsubscribe function - call to remove the subscription
   *
   * @example
   * ```typescript
   * // Subscribe to all faders
   * const unsub = clasp.subscribe('/mixer/fader/*', (value, address) => {
   *   console.log(`${address} = ${value}`);
   * }, { maxRate: 30 });
   *
   * // Later: unsubscribe
   * unsub();
   * ```
   */
  subscribe(pattern: string, callback: SubscriptionCallback, options?: { maxRate?: number; epsilon?: number }): Unsubscribe {
    const id = this.nextSubId++;

    this.subscriptions.set(id, { pattern, callback });

    const msg: SubscribeMessage = {
      type: 'SUBSCRIBE',
      id,
      pattern,
      options: options ? { maxRate: options.maxRate, epsilon: options.epsilon } : undefined,
    };

    this.send(msg);

    return () => {
      this.subscriptions.delete(id);
      this.send({ type: 'UNSUBSCRIBE', id });
    };
  }

  /**
   * Shorthand for subscribe
   */
  on(pattern: string, callback: SubscriptionCallback, options?: { maxRate?: number; epsilon?: number }): Unsubscribe {
    return this.subscribe(pattern, callback, options);
  }

  /**
   * Set a persistent parameter value.
   *
   * The value is stored on the server and broadcast to all subscribers.
   * Uses QoS.Confirm for reliable delivery.
   *
   * @param address - The parameter address (e.g., '/control/volume')
   * @param value - The value to set (number, string, boolean, array, or object)
   *
   * @example
   * ```typescript
   * clasp.set('/mixer/channel/1/volume', 0.75);
   * clasp.set('/scene/name', 'Main Stage');
   * clasp.set('/config/enabled', true);
   * ```
   */
  set(address: string, value: Value): void {
    const msg: SetMessage = {
      type: 'SET',
      address,
      value,
    };
    this.send(msg, QoS.Confirm);
  }

  /**
   * Get current value (from cache or server)
   */
  async get(address: string): Promise<Value> {
    // Check cache first
    if (this.params.has(address)) {
      return this.params.get(address)!;
    }

    // Request from server
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingGets.delete(address);
        reject(new Error('Timeout'));
      }, 5000);

      this.pendingGets.set(address, (value) => {
        clearTimeout(timeout);
        resolve(value);
      });

      this.send({ type: 'GET', address });
    });
  }

  /**
   * Emit a one-shot event.
   *
   * Events are not persisted on the server. They are delivered once
   * to all current subscribers and then forgotten.
   *
   * @param address - The event address
   * @param payload - Optional event payload data
   *
   * @example
   * ```typescript
   * // Trigger a cue
   * clasp.emit('/show/cue/trigger', { cue: 'intro', fadeTime: 2.0 });
   *
   * // Simple notification
   * clasp.emit('/alert/fire');
   * ```
   */
  emit(address: string, payload?: Value): void {
    const msg: PublishMessage = {
      type: 'PUBLISH',
      address,
      signal: 'event',
      payload: payload ?? null,
      timestamp: this.time(),
    };
    this.send(msg, QoS.Confirm);
  }

  /**
   * Send a high-rate stream sample.
   *
   * Stream messages use QoS.Fire (best effort, no confirmation) for
   * minimal latency. Use this for sensor data, audio levels, etc.
   * where occasional packet loss is acceptable.
   *
   * @param address - The stream address
   * @param value - The sample value
   *
   * @example
   * ```typescript
   * // Stream sensor data at high rate
   * setInterval(() => {
   *   clasp.stream('/sensor/accelerometer', {
   *     x: getAccelX(),
   *     y: getAccelY(),
   *     z: getAccelZ()
   *   });
   * }, 16); // ~60 Hz
   * ```
   */
  stream(address: string, value: Value): void {
    const msg: PublishMessage = {
      type: 'PUBLISH',
      address,
      signal: 'stream',
      value,
      timestamp: this.time(),
    };
    this.send(msg, QoS.Fire);
  }

  /**
   * Send gesture input (touch/pen/motion).
   *
   * Gesture messages track multi-touch or pen input through phases:
   * - 'start': Finger/pen touches the surface
   * - 'move': Finger/pen moves while touching
   * - 'end': Finger/pen lifts off
   * - 'cancel': Gesture was interrupted
   *
   * Uses QoS.Fire for minimal latency (suitable for high-frequency move events).
   *
   * @param address - The gesture address (e.g., '/input/touch/0')
   * @param gestureId - Unique identifier for this gesture stream
   * @param phase - Phase of the gesture ('start', 'move', 'end', 'cancel')
   * @param payload - Optional gesture data (e.g., position, pressure)
   *
   * @example
   * ```typescript
   * // Touch start
   * clasp.gesture('/input/touch/0', 1, 'start', { x: 100, y: 200 });
   *
   * // Touch move (high frequency)
   * clasp.gesture('/input/touch/0', 1, 'move', { x: 105, y: 210, pressure: 0.8 });
   *
   * // Touch end
   * clasp.gesture('/input/touch/0', 1, 'end', { x: 110, y: 220 });
   * ```
   */
  gesture(
    address: string,
    gestureId: number,
    phase: 'start' | 'move' | 'end' | 'cancel',
    payload?: Value
  ): void {
    const msg: PublishMessage = {
      type: 'PUBLISH',
      address,
      signal: 'gesture',
      phase,
      id: gestureId,
      payload: payload ?? null,
      timestamp: this.time(),
    };
    this.send(msg, QoS.Fire);
  }

  /**
   * Send timeline automation data.
   *
   * Timeline messages define keyframe animations that the router can interpolate.
   * Each keyframe specifies a time offset (in microseconds) and target value.
   *
   * @param address - The parameter address to animate
   * @param keyframes - Array of keyframes with time and value
   * @param options - Optional timeline options
   * @param options.loop - Whether the timeline should loop (default: false)
   * @param options.startTime - Start time in microseconds (defaults to current time)
   *
   * @example
   * ```typescript
   * // Fade opacity from 0 to 1 over 1 second
   * clasp.timeline('/lumen/layer/0/opacity', [
   *   { time: 0, value: 0.0 },
   *   { time: 1000000, value: 1.0 },  // 1 second = 1,000,000 microseconds
   * ]);
   *
   * // Looping animation
   * clasp.timeline('/effect/pulse', [
   *   { time: 0, value: 0.0 },
   *   { time: 500000, value: 1.0 },
   *   { time: 1000000, value: 0.0 },
   * ], { loop: true });
   *
   * // Scheduled start
   * clasp.timeline('/cue/fade', [
   *   { time: 0, value: 1.0 },
   *   { time: 2000000, value: 0.0 },
   * ], { startTime: clasp.time() + 5000000 });  // Start 5 seconds from now
   * ```
   */
  timeline(
    address: string,
    keyframes: Array<{ time: number; value: Value; easing?: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out' | 'step' }>,
    options?: { loop?: boolean; startTime?: number }
  ): void {
    const msg: PublishMessage = {
      type: 'PUBLISH',
      address,
      signal: 'timeline',
      keyframes,
      loop: options?.loop ?? false,
      timestamp: options?.startTime ?? this.time(),
    };
    this.send(msg, QoS.Confirm);
  }

  /**
   * Send an atomic bundle of messages.
   *
   * All messages in a bundle are delivered together, ensuring consistency.
   * Optionally schedule the bundle for execution at a specific server time.
   *
   * @param messages - Array of messages to bundle
   * @param options - Optional bundle options
   * @param options.at - Server timestamp (microseconds) for scheduled execution
   *
   * @example
   * ```typescript
   * // Atomic update of multiple values
   * clasp.bundle([
   *   { set: ['/light/1', 1.0] },
   *   { set: ['/light/2', 0.0] },
   *   { emit: ['/cue/complete', { cue: 'crossfade' }] }
   * ]);
   *
   * // Scheduled bundle (execute 100ms from now)
   * clasp.bundle([
   *   { set: ['/effect/strobe', true] }
   * ], { at: clasp.time() + 100000 });
   * ```
   */
  bundle(messages: Array<{ set?: [string, Value]; emit?: [string, Value] }>, options?: { at?: number }): void {
    const formatted: Message[] = messages.map((m) => {
      if (m.set) {
        return { type: 'SET' as const, address: m.set[0], value: m.set[1] };
      }
      if (m.emit) {
        return { type: 'PUBLISH' as const, address: m.emit[0], signal: 'event' as const, payload: m.emit[1] };
      }
      throw new Error('Invalid bundle message');
    });

    this.send(
      { type: 'BUNDLE', timestamp: options?.at, messages: formatted },
      QoS.Commit
    );
  }

  /**
   * Get cached value
   */
  cached(address: string): Value | undefined {
    return this.params.get(address);
  }

  /**
   * Get all announced signals.
   * Returns signals received via ANNOUNCE or RESULT messages.
   */
  getSignals(): SignalDefinition[] {
    return Array.from(this.signals.values());
  }

  /**
   * Query signals matching a pattern.
   *
   * @param pattern - Address pattern to match (supports wildcards)
   * @returns Array of matching signal definitions
   */
  querySignals(pattern: string): SignalDefinition[] {
    return Array.from(this.signals.values()).filter((signal) =>
      matchPattern(pattern, signal.address)
    );
  }

  /**
   * Get the last error received from the server.
   * Returns null if no error has been received.
   */
  getLastError(): ErrorMessage | null {
    return this.lastError;
  }

  /**
   * Clear the last error.
   */
  clearError(): void {
    this.lastError = null;
  }

  /**
   * Register connect callback
   */
  onConnect(callback: () => void): void {
    this.onConnectCallbacks.push(callback);
  }

  /**
   * Register disconnect callback
   */
  onDisconnect(callback: (reason?: string) => void): void {
    this.onDisconnectCallbacks.push(callback);
  }

  /**
   * Register error callback
   */
  onError(callback: (error: Error) => void): void {
    this.onErrorCallbacks.push(callback);
  }

  /**
   * Register reconnect callback.
   * Called when a reconnection attempt is made.
   *
   * @param callback - Function called with reconnect attempt number
   */
  onReconnect(callback: (attempt: number) => void): void {
    this.onReconnectCallbacks.push(callback);
  }

  /**
   * Close connection.
   * Disables auto-reconnect and closes the WebSocket.
   */
  close(): void {
    this.intentionallyClosed = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
    this._connected = false;
  }

  // Private methods

  /**
   * Schedule a reconnection attempt with exponential backoff.
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      const error = new Error(`Max reconnect attempts (${this.maxReconnectAttempts}) reached`);
      this.onErrorCallbacks.forEach((cb) => cb(error));
      return;
    }

    // Exponential backoff: base interval * 2^attempts, max 30 seconds
    const baseInterval = this.options.reconnectInterval || 5000;
    const delay = Math.min(baseInterval * Math.pow(1.5, this.reconnectAttempts), 30000);

    this.reconnectTimer = setTimeout(async () => {
      this.reconnectAttempts++;
      this.onReconnectCallbacks.forEach((cb) => cb(this.reconnectAttempts));

      try {
        await this.connect();
        // Reconnect successful - reset attempts and resubscribe
        this.reconnectAttempts = 0;
        await this.resubscribeAll();
      } catch (e) {
        // connect() will trigger onclose which will schedule another reconnect
      }
    }, delay);
  }

  /**
   * Resubscribe to all previously registered patterns after reconnect.
   */
  private async resubscribeAll(): Promise<void> {
    // Send subscribe messages for all existing subscriptions
    for (const [id, sub] of this.subscriptions) {
      const msg: SubscribeMessage = {
        type: 'SUBSCRIBE',
        id,
        pattern: sub.pattern,
      };
      this.send(msg);
    }
  }

  private sendHello(): void {
    const hello: HelloMessage = {
      type: 'HELLO',
      version: PROTOCOL_VERSION,
      name: this.options.name!,
      features: this.options.features!,
      token: this.options.token,
    };
    this.send(hello);
  }

  private send(message: Message, qos: QoS = QoS.Fire): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const frame = encodeMessage(message, { qos });
      this.ws.send(frame);
    }
  }

  private handleMessage(message: Message): void {
    switch (message.type) {
      case 'WELCOME': {
        const welcome = message as WelcomeMessage;
        this.sessionId = welcome.session;
        this.serverTimeOffset = welcome.time - Date.now() * 1000;
        break;
      }

      case 'SET': {
        const set = message as SetMessage;
        this.params.set(set.address, set.value);
        this.notifySubscribers(set.address, set.value);
        break;
      }

      case 'SNAPSHOT': {
        const snapshot = message as SnapshotMessage;
        for (const param of snapshot.params) {
          this.params.set(param.address, param.value);

          // Resolve pending gets
          const resolver = this.pendingGets.get(param.address);
          if (resolver) {
            resolver(param.value);
            this.pendingGets.delete(param.address);
          }

          this.notifySubscribers(param.address, param.value, param);
        }
        break;
      }

      case 'PUBLISH': {
        const pub = message as PublishMessage;
        const value = pub.value ?? pub.payload ?? null;
        this.notifySubscribers(pub.address, value);
        break;
      }

      case 'PING':
        this.send({ type: 'PONG' });
        break;

      case 'PONG':
        // Response to our ping, nothing to do
        break;

      case 'ERROR': {
        const error = message as ErrorMessage;
        console.error(`CLASP error ${error.code}: ${error.message}`, error.address ? `(address: ${error.address})` : '');
        this.lastError = error;
        break;
      }

      case 'ACK': {
        const ack = message as AckMessage;
        // Acknowledgment received - could be extended to track pending requests
        // For now, just log at debug level
        if (typeof console.debug === 'function') {
          console.debug('CLASP ACK received:', ack.address, 'revision:', ack.revision);
        }
        break;
      }

      case 'ANNOUNCE': {
        const announce = message as AnnounceMessage;
        // Store announced signals
        for (const signal of announce.signals) {
          this.signals.set(signal.address, signal);
        }
        break;
      }

      case 'SYNC': {
        const sync = message as SyncMessage;
        // Process clock sync response
        if (sync.t2 !== undefined && sync.t3 !== undefined) {
          // Calculate round-trip time and update offset
          const t4 = Date.now() * 1000;
          const roundTrip = (t4 - sync.t1) - (sync.t3 - sync.t2);
          this.serverTimeOffset = sync.t2 - sync.t1 - roundTrip / 2;
        }
        break;
      }

      case 'RESULT': {
        const result = message as ResultMessage;
        // Store returned signals
        for (const signal of result.signals) {
          this.signals.set(signal.address, signal);
        }
        break;
      }

      case 'BUNDLE': {
        const bundle = message as BundleMessage;
        // Process all messages in the bundle
        for (const innerMsg of bundle.messages) {
          this.handleMessage(innerMsg);
        }
        break;
      }

      // Client-initiated messages not expected from server
      case 'HELLO':
      case 'SUBSCRIBE':
      case 'UNSUBSCRIBE':
      case 'GET':
      case 'QUERY':
        console.warn('Received unexpected client-type message:', message.type);
        break;
    }
  }

  private notifySubscribers(address: string, value: Value, meta?: ParamValue): void {
    for (const [, sub] of this.subscriptions) {
      if (matchPattern(sub.pattern, address)) {
        sub.callback(value, address, meta);
      }
    }
  }
}
