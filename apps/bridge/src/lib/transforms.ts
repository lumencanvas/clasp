import type { TransformConfig } from './types'

export function applyTransform(value: number, config: TransformConfig): number {
  switch (config.type) {
    case 'direct':
      return value
    case 'scale': {
      const range = (config.scaleInMax ?? 1) - (config.scaleInMin ?? 0)
      if (range === 0) return config.scaleOutMin ?? 0
      const normalized = (value - (config.scaleInMin ?? 0)) / range
      return (config.scaleOutMin ?? 0) + normalized * ((config.scaleOutMax ?? 127) - (config.scaleOutMin ?? 0))
    }
    case 'invert':
      return 1 - value
    case 'clamp':
      return Math.min(config.clampMax ?? 1, Math.max(config.clampMin ?? 0, value))
    case 'round':
      return Math.round(value)
    case 'toggle':
      return value > 0.5 ? 1 : 0
    case 'gate':
      return value > 0 ? 1 : 0
    case 'threshold':
      return value >= (config.threshold ?? 0.5) ? 1 : 0
    case 'trigger':
      return value !== 0 ? 1 : 0
    case 'expression':
      try {
        return evaluateExpression(config.expression ?? 'value', value)
      } catch {
        return value
      }
    case 'javascript':
      try {
        const fn = new Function('input', `${config.javascriptCode}\nreturn transform(input);`)
        return fn(value) as number
      } catch {
        return value
      }
    case 'deadzone': {
      const min = config.deadzoneMin ?? 0.4
      const max = config.deadzoneMax ?? 0.6
      if (value >= min && value <= max) return 0
      return value
    }
    case 'smooth': {
      const factor = config.smoothFactor ?? 0.3
      const prev = smoothStates.get(config) ?? value
      const result = prev * (1 - factor) + value * factor
      smoothStates.set(config, result)
      return result
    }
    case 'quantize': {
      const steps = config.quantizeSteps ?? 8
      return Math.round(value * steps) / steps
    }
    case 'curve':
      return applyCurve(value, config.curveType ?? 'linear')
    case 'modulo':
      return value % (config.moduloDivisor ?? 1)
    case 'negate':
      return -value
    case 'power':
      return Math.pow(value, config.powerExponent ?? 2)
    case 'wasm':
      // WASM transforms are async and handled by WasmTransformHost.
      // In synchronous preview context, pass through.
      return value
    default:
      return value
  }
}

// Per-config EMA state for smooth transform (keyed by config object identity)
const smoothStates = new WeakMap<object, number>()

function applyCurve(t: number, type: string): number {
  const v = Math.max(0, Math.min(1, t))
  switch (type) {
    case 'ease-in': return v * v
    case 'ease-out': return v * (2 - v)
    case 'ease-in-out': return v < 0.5 ? 2 * v * v : -1 + (4 - 2 * v) * v
    case 'exponential': return v === 0 ? 0 : Math.pow(2, 10 * (v - 1))
    case 'logarithmic': return Math.log(1 + v * (Math.E - 1))
    default: return v
  }
}

export function evaluateExpression(expr: string, value: number): number {
  const safeExpr = expr
    .replace(/\bvalue\b/g, String(value))
    .replace(/\bsin\b/g, 'Math.sin')
    .replace(/\bcos\b/g, 'Math.cos')
    .replace(/\btan\b/g, 'Math.tan')
    .replace(/\babs\b/g, 'Math.abs')
    .replace(/\bmin\b/g, 'Math.min')
    .replace(/\bmax\b/g, 'Math.max')
    .replace(/\bpow\b/g, 'Math.pow')
    .replace(/\bsqrt\b/g, 'Math.sqrt')
    .replace(/\bfloor\b/g, 'Math.floor')
    .replace(/\bceil\b/g, 'Math.ceil')
    .replace(/\bround\b/g, 'Math.round')
    .replace(/\bPI\b/g, 'Math.PI')

  if (!/^[0-9a-zA-Z+\-*/%().\s,]+$/.test(safeExpr)) {
    throw new Error('Invalid expression')
  }

  return Function(`"use strict"; return (${safeExpr})`)() as number
}

export function previewTransform(config: TransformConfig, testValue = 0.5): { output: number; error?: string } {
  try {
    const output = applyTransform(testValue, config)
    return { output }
  } catch (e: any) {
    return { output: testValue, error: e.message }
  }
}

export function formatTransformBadge(config: TransformConfig): string {
  switch (config.type) {
    case 'direct': return '->'
    case 'scale': return `${config.scaleInMin ?? 0}-${config.scaleInMax ?? 1} -> ${config.scaleOutMin ?? 0}-${config.scaleOutMax ?? 127}`
    case 'invert': return 'INV'
    case 'toggle': return 'TOG'
    case 'threshold': return `>=${config.threshold ?? 0.5}`
    case 'clamp': return `[${config.clampMin ?? 0}..${config.clampMax ?? 1}]`
    case 'round': return 'ROUND'
    case 'gate': return 'GATE'
    case 'trigger': return 'TRIG'
    case 'deadzone': return `DZ[${config.deadzoneMin ?? 0.4}-${config.deadzoneMax ?? 0.6}]`
    case 'smooth': return `EMA(${config.smoothFactor ?? 0.3})`
    case 'quantize': return `Q${config.quantizeSteps ?? 8}`
    case 'curve': return config.curveType ?? 'linear'
    case 'modulo': return `%${config.moduloDivisor ?? 1}`
    case 'negate': return 'NEG'
    case 'power': return `x^${config.powerExponent ?? 2}`
    case 'expression': return 'f(x)'
    case 'javascript': return 'JS'
    case 'wasm': return config.wasmModuleName || 'WASM'
    default: return '->'
  }
}
