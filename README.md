# Sunnypot --- DER Security Awareness

The electric grid is adding distributed energy resources (DERs, like solar, batteries, EV chargers, etc.) at record pace.
This adoption begs the question: if critical infrastructure relies on thousands of consumer devices living on home WiFi networks, how does that change the security & threat landscape?

This project aims to shed light on cybersecurity threats facing DERs by providing a *honeypot*: an isolated service that mimics common DERs and can be exposed to the public internet to observe DER-specific traffic.

**Warning:** before running this project on any hardware or network that you care about, make sure you fully understand the steps needed to protect that environment (e.g. containerization, network isolation, ...).

## Supported devices

Sunnypot can currently emulate the following devices/protocols.

- SunSpec via Modbus TCP/IP
    - Single-phase solar (mimicking residential)

Future development may include...

- SunSpec via Modbus TCP/IP
    - Single-phase solar (mimicking residential)
    - Three-phase solar (mimicking C&I)
    - Storage
- IEEE 2030.5 via HTTPS REST
- Others on request (please file a GitHub issue with your request)

For all supported devices/protocols, Sunnypot emulates both the protocol and the physical behavior of the underlying device (e.g. solar output that varies over time).