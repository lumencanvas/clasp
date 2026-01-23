<script setup>
const capabilities = [
  {
    title: 'DISCOVERY',
    desc: 'mDNS auto-discovery with UDP broadcast fallback for local networks.'
  },
  {
    title: 'STATE',
    desc: 'Authoritative params with revisioning, conflict strategies (LWW, max, min, lock, merge).'
  },
  {
    title: 'TIME',
    desc: 'NTP-style clock sync, jitter buffers, scheduled bundle execution.'
  },
  {
    title: 'BRIDGES',
    desc: 'Bidirectional: MIDI, OSC, DMX-512, Art-Net, MQTT, WebSocket, HTTP REST, Socket.IO.'
  },
  {
    title: 'P2P',
    desc: 'WebRTC DataChannels with ICE/NAT traversal, dual channels (reliable + unreliable), automatic fallback to server relay.'
  },
  {
    title: 'EMBEDDED',
    desc: 'no_std client + MiniRouter server: <4KB RAM, subscription routing, compatible with any transport (UDP, HTTP, BLE).'
  }
]
</script>

<template>
  <section class="section">
    <h2>CAPABILITIES</h2>
    <div class="grid">
      <div class="card" v-for="cap in capabilities" :key="cap.title">
        <h3>{{ cap.title }}</h3>
        <p>{{ cap.desc }}</p>
      </div>
    </div>
  </section>
</template>
