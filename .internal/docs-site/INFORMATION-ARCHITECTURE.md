# CLASP Docs Site — Information Architecture

## Sidebar Navigation Structure

```
GETTING STARTED
  Overview                          -> index (docs/index.md)
  Install CLI                       -> how-to/installation/cli
  Install JavaScript                -> how-to/installation/javascript-library
  Install Python                    -> how-to/installation/python-library
  Install Rust                      -> how-to/installation/rust-library
  Your First Connection             -> tutorials/first-connection

TUTORIALS
  Control Lights from Web           -> tutorials/control-lights-from-web
  Sensor to Visualization           -> tutorials/sensor-to-visualization
  Cross-Language Chat               -> tutorials/cross-language-chat
  Embedded Sensor Node              -> tutorials/embedded-sensor-node

GUIDES
  Connections/
    Start Router                    -> how-to/connections/start-router
    Connect Client                  -> how-to/connections/connect-client
    Add OSC                         -> how-to/connections/add-osc
    Add MIDI                        -> how-to/connections/add-midi
    Add DMX                         -> how-to/connections/add-dmx
    Add Art-Net                     -> how-to/connections/add-artnet
    Add MQTT                        -> how-to/connections/add-mqtt
    Add HTTP                        -> how-to/connections/add-http
    Add WebSocket                   -> how-to/connections/add-websocket
  State Management/
    Get Set Values                  -> how-to/state/get-set-values
    Subscribe to Changes            -> how-to/state/subscribe-to-changes
    Handle Conflicts                -> how-to/state/handle-conflicts
    Late Joiner Sync                -> how-to/state/late-joiner-sync
    Use Locks                       -> how-to/state/use-locks
  Timing/
    Clock Sync                      -> how-to/timing/clock-sync
    Bundle Atomic                   -> how-to/timing/bundle-atomic
    Scheduled Bundles               -> how-to/timing/scheduled-bundles
  Discovery/
    mDNS Discovery                  -> how-to/discovery/mdns-discovery
    UDP Broadcast                   -> how-to/discovery/udp-broadcast
    Manual Connection               -> how-to/discovery/manual-connection
  Security/
    Capability Tokens               -> how-to/security/capability-tokens
    Enable TLS                      -> how-to/security/enable-tls
    Pairing                         -> how-to/security/pairing
  Advanced/
    Custom Bridge                   -> how-to/advanced/custom-bridge
    Embed Router                    -> how-to/advanced/embed-router
    P2P WebRTC                      -> how-to/advanced/p2p-webrtc
    Performance Tuning              -> how-to/advanced/performance-tuning
  Troubleshooting                   -> how-to/troubleshooting

CONCEPTS
  Overview                          -> explanation (README)
  Why CLASP?                        -> explanation/why-clasp
  Architecture                      -> explanation/architecture
  Signals, Not Messages             -> explanation/signals-not-messages
  Router vs Client                  -> explanation/router-vs-client
  State Management                  -> explanation/state-management
  Timing Model                      -> explanation/timing-model
  Security Model                    -> explanation/security-model
  Transport Agnosticism             -> explanation/transport-agnosticism
  Bridge Architecture               -> explanation/bridge-architecture
  Distributed Architecture          -> explanation/distributed-architecture
  Federation                        -> explanation/federation-*
  Capability Delegation             -> explanation/capability-delegation
  Conflict Resolution               -> explanation/conflict-resolution
  Token Validation Flow             -> explanation/token-validation-flow

REFERENCE
  Protocol/
    Overview                        -> reference/protocol/overview
    Messages                        -> reference/protocol/messages
    Addressing                      -> reference/protocol/addressing
    Signal Types                    -> reference/protocol/signal-types
    Data Types                      -> reference/protocol/data-types
    Frame Format                    -> reference/protocol/frame-format
    QoS                             -> reference/protocol/qos
  API - Rust/
    clasp-core                      -> reference/api/rust/clasp-core
    clasp-router                    -> reference/api/rust/clasp-router
    clasp-client                    -> reference/api/rust/clasp-client
    clasp-bridge                    -> reference/api/rust/clasp-bridge
    clasp-transport                 -> reference/api/rust/clasp-transport
    clasp-discovery                 -> reference/api/rust/clasp-discovery
    clasp-embedded                  -> reference/api/rust/clasp-embedded
  API - JavaScript/
    clasp-core                      -> reference/api/javascript/clasp-core
    Browser                         -> reference/api/javascript/browser
    Node.js                         -> reference/api/javascript/nodejs
  API - Python/
    clasp-to                        -> reference/api/python/clasp-to
  CLI/
    clasp-server                    -> reference/cli/clasp-server
    clasp-osc                       -> reference/cli/clasp-osc
    clasp-midi                      -> reference/cli/clasp-midi
    clasp-mqtt                      -> reference/cli/clasp-mqtt
    clasp-http                      -> reference/cli/clasp-http
  Bridges/
    OSC                             -> reference/bridges/osc
    MIDI                            -> reference/bridges/midi
    Art-Net                         -> reference/bridges/artnet
    DMX                             -> reference/bridges/dmx
    sACN                            -> reference/bridges/sacn
    MQTT                            -> reference/bridges/mqtt
    HTTP                            -> reference/bridges/http
  Transports/
    WebSocket                       -> reference/transports/websocket
    UDP                             -> reference/transports/udp
    QUIC                            -> reference/transports/quic
    WebRTC                          -> reference/transports/webrtc
    BLE                             -> reference/transports/ble
    Serial                          -> reference/transports/serial
  Configuration/
    Router Config                   -> reference/configuration/router-config
    Bridge Config                   -> reference/configuration/bridge-config
    Feature Flags                   -> reference/configuration/feature-flags

USE CASES
  Live Performance                  -> use-cases/live-performance
  Installation Art                  -> use-cases/installation-art
  Home Automation                   -> use-cases/home-automation
  Software Integration              -> use-cases/software-integration
  Embedded Systems                  -> use-cases/embedded-systems
  Cloud Deployment                  -> use-cases/cloud-deployment

INTEGRATIONS
  TouchOSC                          -> integrations/touchosc
  Resolume                          -> integrations/resolume
  QLab                              -> integrations/qlab
  Ableton                           -> integrations/ableton
  TouchDesigner                     -> integrations/touchdesigner
  MadMapper                         -> integrations/madmapper
  Home Assistant                    -> integrations/home-assistant

APPENDIX
  Glossary                          -> appendix/glossary
  FAQ                               -> appendix/faq
  Changelog                         -> appendix/changelog
  Migration from OSC                -> appendix/migration/from-osc
  Migration from MQTT               -> appendix/migration/from-mqtt
```

## URL Routing

Pattern: `docs.clasp.to/{path}` maps to `docs/{path}.md`

Examples:
- `docs.clasp.to/` -> Landing page (DocsHome.vue)
- `docs.clasp.to/tutorials/first-connection` -> `docs/tutorials/first-connection.md`
- `docs.clasp.to/reference/protocol/overview` -> `docs/reference/protocol/overview.md`
- `docs.clasp.to/explanation/why-clasp` -> `docs/explanation/why-clasp.md`

## Diataxis Quadrant Mapping

| Section | Diataxis | Purpose |
|---------|----------|---------|
| Getting Started | Tutorial | Onboarding, first steps |
| Tutorials | Tutorial | Learning-oriented walkthroughs |
| Guides | How-To | Task-oriented procedures |
| Concepts | Explanation | Understanding-oriented discussion |
| Reference | Reference | Information-oriented lookup |
| Use Cases | How-To/Explanation | Persona-specific guidance |
| Integrations | How-To | Third-party software setup |
| Appendix | Reference | Glossary, FAQ, changelog |
