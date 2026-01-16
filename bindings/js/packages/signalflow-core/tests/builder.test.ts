import { describe, it, expect } from 'vitest';
import { SignalFlowBuilder } from '../src/builder';

describe('SignalFlowBuilder', () => {
  it('should create builder with URL', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330');
    expect(builder).toBeDefined();
  });

  it('should set name', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330')
      .withName('Test Client');
    expect(builder).toBeDefined();
  });

  it('should set features', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330')
      .withFeatures(['param', 'event', 'stream']);
    expect(builder).toBeDefined();
  });

  it('should set token', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330')
      .withToken('secret-token');
    expect(builder).toBeDefined();
  });

  it('should set reconnect options', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330')
      .withReconnect(true, 5000);
    expect(builder).toBeDefined();
  });

  it('should chain all options', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330')
      .withName('Full Test')
      .withFeatures(['param', 'event'])
      .withToken('token123')
      .withReconnect(true, 3000);

    expect(builder).toBeDefined();
  });

  it('should have connect method', () => {
    const builder = new SignalFlowBuilder('ws://localhost:7330');
    expect(typeof builder.connect).toBe('function');
  });
});
