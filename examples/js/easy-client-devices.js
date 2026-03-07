/**
 * Easy Client SDK -- device management example
 *
 * Demonstrates device registration, child devices, provisioning,
 * and credential bundles using @clasp-to/sdk.
 *
 * Requires a CLASP relay with auth enabled:
 *   clasp-relay --port 7330 --auth-port 7350
 *
 * Usage:
 *   npm install @clasp-to/sdk
 *   node easy-client-devices.js
 */

import clasp from '@clasp-to/sdk'

const CLASP_URL = process.env.CLASP_URL || 'ws://localhost:7330'

const c = await clasp(CLASP_URL, { name: 'Device Manager' })
console.log('Connected')

// --- Register a device ------------------------------------------------------

const device = await c.register({
  name: 'Living Room Hub',
  scopes: ['write:/lights/living-room/**', 'read:/**'],
})
console.log('Registered device:', device.name, device.id)

// --- Create a child device --------------------------------------------------

const dimmer = await device.createChild({
  name: 'Dimmer Switch',
  scopes: ['write:/lights/living-room/dimmer'],
})
console.log('Child device:', dimmer.name, dimmer.id)

// Connect as the child device and publish
const dimmerClient = await dimmer.connect()
await dimmerClient.set('/lights/living-room/dimmer', 0.5)
console.log('Dimmer set to 0.5')

// --- Provision credentials for firmware -------------------------------------

const creds = await device.provision({
  name: 'Kitchen Sensor',
  scopes: ['write:/sensors/kitchen/**'],
  expires: '30d',
})

// Output in different formats for different deployment methods
console.log('\nCredential bundle (JSON):')
console.log(creds.toJSON())

console.log('\nCredential bundle (env vars):')
console.log(creds.toEnv())

// Connect immediately with the credentials
const sensorClient = await creds.connect()
await sensorClient.set('/sensors/kitchen/temperature', 22.5)
console.log('\nSensor connected and published')

// --- Bulk provision ---------------------------------------------------------

const batch = await device.provisionBatch([
  { name: 'Sensor 1', scopes: ['write:/sensors/1/**'] },
  { name: 'Sensor 2', scopes: ['write:/sensors/2/**'] },
  { name: 'Sensor 3', scopes: ['write:/sensors/3/**'] },
])
console.log('\nProvisioned', batch.length, 'devices')
for (const b of batch) {
  console.log(' -', b.name, b.token.slice(0, 20) + '...')
}

// --- List and revoke --------------------------------------------------------

const children = await device.children()
console.log('\nChildren:', children.map(ch => ch.name))

// Revoke the dimmer
await device.revoke(dimmer.id)
console.log('Revoked dimmer:', dimmer.id)

// --- Cleanup ----------------------------------------------------------------

dimmerClient.close()
sensorClient.close()
c.close()
console.log('Done')
