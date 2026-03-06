import type { Preset } from '../lib/types'
// Re-export from existing presets file (will be ported to TS later)
// For now, import from the JS file
import { presets, getPreset, searchPresets, categories } from '../presets/index.js'

export { presets, getPreset, searchPresets, categories }

export function usePresets() {
  return {
    presets: presets as Preset[],
    getPreset: getPreset as (id: string) => Preset | undefined,
    searchPresets: searchPresets as (query: string) => Preset[],
    categories,
  }
}
