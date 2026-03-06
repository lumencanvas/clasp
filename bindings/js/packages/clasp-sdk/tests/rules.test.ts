import { describe, it, expect } from 'vitest'
import { buildRuleJSON } from '../src/rules'

describe('buildRuleJSON', () => {
  describe('Trigger types', () => {
    it('on_change from "when" without thresholds', () => {
      const rule = buildRuleJSON('mirror', {
        when: '/input/**',
        then: { set: ['/output/status', 'changed'] },
      })
      expect(rule.trigger).toEqual({ type: 'on_change', pattern: '/input/**' })
    })

    it('on_threshold with "above"', () => {
      const rule = buildRuleJSON('hot', {
        when: '/sensors/temp',
        above: 30,
        then: { set: ['/alarm', true] },
      })
      expect(rule.trigger).toEqual({
        type: 'on_threshold',
        address: '/sensors/temp',
        above: 30,
      })
    })

    it('on_threshold with "below"', () => {
      const rule = buildRuleJSON('cold', {
        when: '/sensors/temp',
        below: 10,
        then: { set: ['/heater', true] },
      })
      expect(rule.trigger).toEqual({
        type: 'on_threshold',
        address: '/sensors/temp',
        below: 10,
      })
    })

    it('on_threshold with both "above" and "below"', () => {
      const rule = buildRuleJSON('range', {
        when: '/sensors/temp',
        above: 30,
        below: 10,
        then: { set: ['/alert', true] },
      })
      expect(rule.trigger).toEqual({
        type: 'on_threshold',
        address: '/sensors/temp',
        above: 30,
        below: 10,
      })
    })

    it('on_event from "onEvent"', () => {
      const rule = buildRuleJSON('handler', {
        onEvent: '/alerts/**',
        then: { set: ['/count', 1] },
      })
      expect(rule.trigger).toEqual({ type: 'on_event', pattern: '/alerts/**' })
    })

    it('on_interval from "every"', () => {
      const rule = buildRuleJSON('heartbeat', {
        every: '30s',
        then: { emit: ['/system/heartbeat', { alive: true }] },
      })
      expect(rule.trigger).toEqual({ type: 'on_interval', seconds: 30 })
    })

    it('on_session_join trigger', () => {
      const rule = buildRuleJSON('welcome', {
        onSessionJoin: '/room/**',
        then: { emit: ['/room/welcome', { msg: 'Hello' }] },
      })
      expect(rule.trigger).toEqual({ type: 'on_session_join', pattern: '/room/**' })
    })

    it('on_session_leave trigger', () => {
      const rule = buildRuleJSON('goodbye', {
        onSessionLeave: '/room/**',
        then: { emit: ['/room/goodbye', { msg: 'Bye' }] },
      })
      expect(rule.trigger).toEqual({ type: 'on_session_leave', pattern: '/room/**' })
    })

    it('throws when no trigger specified', () => {
      expect(() => buildRuleJSON('bad', {
        then: { set: ['/x', 1] },
      })).toThrow('has no trigger')
    })
  })

  describe('Conditions', () => {
    it('simple equality: { "/path": value }', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        if: { '/system/enabled': true },
        then: { set: ['/y', 1] },
      })
      expect(rule.conditions).toEqual([
        { address: '/system/enabled', op: 'eq', value: true },
      ])
    })

    it('operator: gt, lt, gte, lte, ne, eq', () => {
      const rule = buildRuleJSON('ops', {
        when: '/x',
        if: {
          '/a': { gt: 10 },
          '/b': { lt: 5 },
          '/c': { gte: 100 },
          '/d': { lte: 0 },
          '/e': { ne: 'off' },
          '/f': { eq: 42 },
        },
        then: { set: ['/y', 1] },
      })
      const conds = rule.conditions as Array<{ address: string; op: string; value: unknown }>
      expect(conds).toContainEqual({ address: '/a', op: 'gt', value: 10 })
      expect(conds).toContainEqual({ address: '/b', op: 'lt', value: 5 })
      expect(conds).toContainEqual({ address: '/c', op: 'gte', value: 100 })
      expect(conds).toContainEqual({ address: '/d', op: 'lte', value: 0 })
      expect(conds).toContainEqual({ address: '/e', op: 'ne', value: 'off' })
      expect(conds).toContainEqual({ address: '/f', op: 'eq', value: 42 })
    })

    it('mixed equality and operator conditions', () => {
      const rule = buildRuleJSON('mixed', {
        when: '/x',
        if: {
          '/sensors/temp': { gt: 20 },
          '/system/mode': 'active',
        },
        then: { set: ['/output', true] },
      })
      const conds = rule.conditions as any[]
      expect(conds).toEqual([
        { address: '/sensors/temp', op: 'gt', value: 20 },
        { address: '/system/mode', op: 'eq', value: 'active' },
      ])
    })

    it('empty "if" produces no conditions', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        if: {},
        then: { set: ['/y', 1] },
      })
      expect(rule.conditions).toEqual([])
    })
  })

  describe('Actions', () => {
    it('single set action', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { set: ['/y', 42] },
      })
      expect(rule.actions).toEqual([
        { type: 'set', address: '/y', value: 42 },
      ])
    })

    it('single emit action', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { emit: ['/event', { data: 1 }] },
      })
      expect(rule.actions).toEqual([
        { type: 'publish', address: '/event', value: { data: 1 } },
      ])
    })

    it('array of mixed actions', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: [
          { set: ['/a', 1] },
          { emit: ['/b', 2] },
        ],
      })
      const actions = rule.actions as any[]
      expect(actions).toHaveLength(2)
      expect(actions[0].type).toBe('set')
      expect(actions[1].type).toBe('publish')
    })

    it('setFrom with transform', () => {
      const rule = buildRuleJSON('convert', {
        when: '/sensors/temp',
        then: {
          setFrom: ['/display/temp-f'],
          transform: { type: 'scale', factor: 1.8, offset: 32 },
        },
      })
      expect(rule.actions).toEqual([{
        type: 'set_from_trigger',
        address: '/display/temp-f',
        transform: { type: 'scale', factor: 1.8, offset: 32 },
      }])
    })

    it('setFrom without transform', () => {
      const rule = buildRuleJSON('copy', {
        when: '/input',
        then: { setFrom: ['/output'] },
      })
      expect(rule.actions).toEqual([{
        type: 'set_from_trigger',
        address: '/output',
      }])
    })

    it('delay action', () => {
      const rule = buildRuleJSON('delayed', {
        when: '/trigger',
        then: [
          { set: ['/warning', true] },
          { delay: 1000 },
          { set: ['/alarm', true] },
        ],
      })
      const actions = rule.actions as any[]
      expect(actions[1]).toEqual({ type: 'delay', milliseconds: 1000 })
    })

    it('all transform types', () => {
      const transforms = [
        { type: 'identity' as const },
        { type: 'scale' as const, factor: 2, offset: 0 },
        { type: 'clamp' as const, min: 0, max: 100 },
        { type: 'threshold' as const, value: 50, above: true, below: false },
        { type: 'invert' as const },
        { type: 'map' as const, table: { on: 1, off: 0 } },
        { type: 'round' as const, precision: 2 },
        { type: 'abs' as const },
      ]
      for (const transform of transforms) {
        const rule = buildRuleJSON(`t-${transform.type}`, {
          when: '/x',
          then: { setFrom: ['/y'], transform },
        })
        const actions = rule.actions as any[]
        expect(actions[0].transform.type).toBe(transform.type)
      }
    })

    it('throws on empty action', () => {
      expect(() => buildRuleJSON('bad', {
        when: '/x',
        then: {} as any,
      })).toThrow('has an action with no set, emit, setFrom, or delay')
    })
  })

  describe('Metadata', () => {
    it('name defaults to id', () => {
      const rule = buildRuleJSON('test-id', {
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.name).toBe('test-id')
    })

    it('explicit name overrides', () => {
      const rule = buildRuleJSON('test', {
        name: 'My Test Rule',
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.name).toBe('My Test Rule')
    })

    it('enabled defaults to true', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.enabled).toBe(true)
    })

    it('enabled: false', () => {
      const rule = buildRuleJSON('test', {
        enabled: false,
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.enabled).toBe(false)
    })

    it('cooldown parsed from duration string', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { set: ['/y', 1] },
        cooldown: '60s',
      })
      expect(rule.cooldown).toBe(60)
    })

    it('minute cooldown', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { set: ['/y', 1] },
        cooldown: '5m',
      })
      expect(rule.cooldown).toBe(300)
    })

    it('no cooldown when not specified', () => {
      const rule = buildRuleJSON('test', {
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.cooldown).toBeUndefined()
    })
  })
})
