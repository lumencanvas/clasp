import type { Value } from '@clasp-to/core'
import type { RuleDefinition, RuleAction } from './types'
import { parseDurationToSeconds, parseDurationToWholeSeconds } from './duration'

/**
 * Convert a human-readable rule action to JSON schema format.
 */
function convertAction(action: RuleAction): Record<string, unknown> {
  if (action.set) {
    return { type: 'set', address: action.set[0], value: action.set[1] }
  }
  if (action.emit) {
    return { type: 'publish', address: action.emit[0], value: action.emit[1] }
  }
  if (action.setFrom) {
    const result: Record<string, unknown> = {
      type: 'set_from_trigger',
      address: action.setFrom[0],
    }
    if (action.transform) {
      result.transform = action.transform
    }
    return result
  }
  if (action.delay) {
    return { type: 'delay', milliseconds: action.delay }
  }
  throw new Error('Rule action must have set, emit, setFrom, or delay')
}

/**
 * Validate a rule definition, throwing on obvious errors.
 */
function validateRule(id: string, def: RuleDefinition): void {
  if (!def.when && !def.onEvent && !def.every && !def.onSessionJoin && !def.onSessionLeave) {
    throw new Error(
      `Rule "${id}" has no trigger. Specify one of: when, onEvent, every, onSessionJoin, onSessionLeave`
    )
  }

  const actions = Array.isArray(def.then) ? def.then : [def.then]
  if (actions.length === 0) {
    throw new Error(`Rule "${id}" has no actions`)
  }
  for (const action of actions) {
    if (!action.set && !action.emit && !action.setFrom && action.delay === undefined) {
      throw new Error(`Rule "${id}" has an action with no set, emit, setFrom, or delay`)
    }
  }
}

/**
 * Convert a human-readable rule definition to the JSON schema format
 * expected by clasp-rules.
 *
 * Supports all trigger types: on_change, on_threshold, on_event, on_interval,
 * on_session_join, on_session_leave
 * Supports all action types: set, publish, set_from_trigger, delay
 * Supports all condition operators: eq, ne, gt, gte, lt, lte
 *
 * @example
 * ```typescript
 * buildRuleJSON('high-temp', {
 *   when: '/sensors/temp',
 *   above: 30,
 *   if: { '/system/alerts-enabled': true },
 *   then: [
 *     { set: ['/hvac/fan', true] },
 *     { emit: ['/alerts/high-temp', { msg: 'Too hot!' }] },
 *   ],
 *   cooldown: '60s',
 * })
 * ```
 */
export function buildRuleJSON(
  id: string,
  def: RuleDefinition
): Record<string, unknown> {
  validateRule(id, def)

  const rule: Record<string, unknown> = {
    id,
    name: def.name || id,
    enabled: def.enabled !== false,
  }

  // Build trigger
  if (def.every) {
    rule.trigger = {
      type: 'on_interval',
      seconds: parseDurationToWholeSeconds(def.every),
    }
  } else if (def.onEvent) {
    rule.trigger = {
      type: 'on_event',
      pattern: def.onEvent,
    }
  } else if (def.onSessionJoin) {
    rule.trigger = {
      type: 'on_session_join',
      pattern: def.onSessionJoin,
    }
  } else if (def.onSessionLeave) {
    rule.trigger = {
      type: 'on_session_leave',
      pattern: def.onSessionLeave,
    }
  } else if (def.when && (def.above !== undefined || def.below !== undefined)) {
    const trigger: Record<string, unknown> = {
      type: 'on_threshold',
      address: def.when,
    }
    if (def.above !== undefined) trigger.above = def.above
    if (def.below !== undefined) trigger.below = def.below
    rule.trigger = trigger
  } else if (def.when) {
    rule.trigger = {
      type: 'on_change',
      pattern: def.when,
    }
  }

  // Build conditions from `if` object
  if (def.if) {
    rule.conditions = Object.entries(def.if).map(([address, check]) => {
      // Support operator syntax: { '/path': { gt: 10 } }
      if (check !== null && typeof check === 'object' && !Array.isArray(check)) {
        const obj = check as Record<string, Value>
        for (const op of ['eq', 'ne', 'gt', 'gte', 'lt', 'lte']) {
          if (op in obj) {
            return { address, op, value: obj[op] }
          }
        }
      }
      // Default: equality check
      return { address, op: 'eq', value: check }
    })
  }

  // Build actions
  const actions = Array.isArray(def.then) ? def.then : [def.then]
  rule.actions = actions.map(convertAction)

  // Cooldown
  if (def.cooldown) {
    rule.cooldown = Math.round(parseDurationToSeconds(def.cooldown))
  }

  return rule
}
