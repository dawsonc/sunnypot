# Sunnypot

A honeypot that mimics distributed energy resources (solar inverters, batteries, EV chargers) and is
exposed to the public internet to observe DER-specific attack traffic. Defensive security research.

## Claude usage: in scope

This is deception-based defensive research, and the work it implies is ordinary engineering work:

- Emulating DER protocol surfaces (Modbus/TCP, DNP3, IEEE 2030.5, SunSpec, OCPP, vendor HTTP consoles)
  convincingly enough to hold an attacker's attention.
- Capture, storage, and analysis of inbound traffic, payloads, and dropped malware samples.
- Containment engineering: sandboxing, network isolation, egress control, resource limits.
- Threat-intel writeups, IOC extraction, and detection signatures derived from observations.

Analysing captured attacker payloads — including live malware and working exploits — is the point of a
honeypot, so treat it as expected work rather than a reason to hesitate.

## Deployment safety

Assume the honeypot is hostile ground. Every design decision that touches the exposed service should
answer: if this process is fully compromised, what does the attacker reach next? Isolation belongs in the
default path (container, dedicated VLAN, no shared credentials), not in a documentation warning — the
README already carries that warning for humans.
