# 02 — Modbus responder serving Model 1 over the test seam

**What to build:** A Modbus/TCP client connects to sunnypot, reads the SunSpec identifier and the
Model 1 common block, and gets back the chosen device identity. The connection and every request it
carried appear as captured events. This is the tracer bullet: it establishes the seam, the config, the
framing, and the capture path in one narrow complete slice.

**Blocked by:** 01 — the Model 1 block serves the identity strings chosen there.

**Status:** ready-for-agent

- [ ] The application is constructed from config, a clock, and a capture sink, and binds an ephemeral port
- [ ] Tests drive real Modbus/TCP over loopback and assert on returned bytes and on captured events — no plant-level or internal assertions
- [ ] The SunSpec identifier is present at the documented base address and the Model 1 chain is traversable by standard discovery logic
- [ ] Model 1 serves vendor, model, serial, and firmware version from config
- [ ] FC3 reads return big-endian holding registers
- [ ] Unsupported function codes and out-of-range addresses return correct Modbus exception responses
- [ ] A connection-lifecycle event and per-request events are captured, joined by a shared connection identifier
- [ ] Config supplies identity and site parameters, is gitignored, and has a committed example documenting shape but not values
