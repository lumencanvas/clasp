import { ref, readonly } from 'vue'
import type { Device } from '../lib/types'
import { useElectron } from './useElectron'

const devices = ref<Device[]>([])
const scanning = ref(false)

async function load() {
  const { invoke } = useElectron()
  const result = await invoke<Device[]>('getDevices')
  if (result) {
    for (const device of result) {
      const existing = devices.value.find(d => d.id === device.id || (d.host === device.host && d.port === device.port))
      if (!existing) devices.value.push(device)
      else Object.assign(existing, device)
    }
  }
}

async function scan() {
  if (scanning.value) return
  scanning.value = true
  const { invoke } = useElectron()
  try {
    await invoke('scanNetwork')
  } catch (err) {
    console.error('Scan failed:', err)
  } finally {
    scanning.value = false
    await load()
  }
}

function upsert(device: Device) {
  const idx = devices.value.findIndex(d => d.id === device.id)
  if (idx >= 0) devices.value[idx] = device
  else devices.value.push(device)
}

function remove(deviceId: string) {
  devices.value = devices.value.filter(d => d.id !== deviceId)
}

async function listMidiPorts() {
  const { invoke } = useElectron()
  return invoke<{ inputs: Array<{ id: string; name: string }>; outputs: Array<{ id: string; name: string }> }>('listMidiPorts')
}

async function listSerialPorts() {
  const { invoke } = useElectron()
  return invoke<Array<{ path: string; name: string }>>('listSerialPorts')
}

async function listNetworkInterfaces() {
  const { invoke } = useElectron()
  return invoke<Array<{ address: string; label: string }>>('listNetworkInterfaces')
}

async function testSerialPort(portPath: string) {
  const { invoke } = useElectron()
  return invoke<{ success: boolean; error?: string }>('testSerialPort', portPath)
}

async function testPortAvailable(host: string, port: number) {
  const { invoke } = useElectron()
  return invoke<{ success: boolean; error?: string }>('testPortAvailable', host, port)
}

export function useDevices() {
  return {
    devices: readonly(devices),
    scanning: readonly(scanning),
    load,
    scan,
    upsert,
    remove,
    listMidiPorts,
    listSerialPorts,
    listNetworkInterfaces,
    testSerialPort,
    testPortAvailable,
  }
}
