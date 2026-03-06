import { ref, readonly } from 'vue'
import type { Token } from '../lib/types'
import { loadFromStorage, saveToStorage } from './useStorage'

const tokens = ref<Token[]>([])

function load() {
  tokens.value = loadFromStorage<Token[]>('tokens', [])
}

function save() {
  saveToStorage('tokens', tokens.value)
}

function generateCpskToken(): string {
  const chars = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
  const array = new Uint8Array(32)
  crypto.getRandomValues(array)
  let random = ''
  for (let i = 0; i < 32; i++) random += chars[array[i] % chars.length]
  return `cpsk_${random}`
}

function create(name: string, pattern: string, scopes: string[]): Token {
  const token: Token = {
    id: Date.now().toString(),
    name,
    token: generateCpskToken(),
    scopes,
    created: new Date().toISOString(),
  }
  tokens.value.push(token)
  save()
  return token
}

function remove(id: string) {
  tokens.value = tokens.value.filter(t => t.id !== id)
  save()
}

function getTokenFileContent(): string {
  return tokens.value.map(t => `${t.token} ${t.scopes.join(',')}`).join('\n')
}

export function useTokens() {
  return {
    tokens: readonly(tokens),
    load,
    save,
    create,
    remove,
    getTokenFileContent,
  }
}
