---
title: "Tutorials"
description: "Step-by-step guides to learn CLASP by doing. Each tutorial is designed to be completed in a single session and teaches fundamental concepts through..."
section: tutorials
order: 0
---
# Tutorials

Step-by-step guides to learn CLASP by doing. Each tutorial is designed to be completed in a single session and teaches fundamental concepts through practical examples.

## Getting Started

### [First Connection](first-connection.md)
**Time:** 5-10 minutes
**Prerequisites:** None

Connect two applications and send your first messages. Learn the basics of routers, clients, and addresses.

**What you'll learn:**
- Starting a CLASP router
- Connecting clients in JavaScript and Python
- Setting and getting values
- Subscribing to changes

## Building Real Applications

### [Control Lights from Web](control-lights-from-web.md)
**Time:** 20-30 minutes
**Prerequisites:** First Connection tutorial, basic HTML/JavaScript

Build a web interface that controls DMX lights through CLASP. Learn how protocol bridges work.

**What you'll learn:**
- Setting up DMX/Art-Net bridges
- Building a browser-based control UI
- Mapping web controls to lighting parameters
- Real-time bidirectional updates

### [Sensor to Visualization](sensor-to-visualization.md)
**Time:** 20-30 minutes
**Prerequisites:** First Connection tutorial, Python basics

Create a pipeline from IoT sensors to a visualization application. Learn about MQTT integration and data flow.

**What you'll learn:**
- Setting up MQTT bridges
- Processing sensor data in Python
- Publishing to visualization software
- Using wildcards for multiple sensors

### [Cross-Language Chat](cross-language-chat.md)
**Time:** 15-20 minutes
**Prerequisites:** First Connection tutorial

Build a chat application with clients in JavaScript, Python, and Rust communicating through CLASP.

**What you'll learn:**
- Writing clients in multiple languages
- Event-based messaging
- Subscription patterns
- Real-time message routing

## Embedded Systems

### [Embedded Sensor Node](embedded-sensor-node.md)
**Time:** 30-45 minutes
**Prerequisites:** First Connection tutorial, ESP32 or RP2040 board

Build a wireless sensor node using CLASP on a microcontroller.

**What you'll learn:**
- Using clasp-embedded
- HTTP POST transport
- Minimal resource usage
- Connecting to CLASP routers

## Choosing Your Path

| If you want to... | Start with |
|-------------------|------------|
| Learn the basics | [First Connection](first-connection.md) |
| Control lighting/AV | [Control Lights from Web](control-lights-from-web.md) |
| Work with IoT sensors | [Sensor to Visualization](sensor-to-visualization.md) |
| Integrate multiple apps | [Cross-Language Chat](cross-language-chat.md) |
| Build embedded devices | [Embedded Sensor Node](embedded-sensor-node.md) |

## Next Steps

After completing tutorials, explore:
- [How-To Guides](../how-to/README.md) for specific tasks
- [Reference Documentation](../reference/README.md) for complete API details
- [Use Cases](../use-cases/README.md) for real-world applications
