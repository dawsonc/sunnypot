# 01 — Choose and document the device identity

**What to build:** A decision, not code. Sunnypot impersonates one specific real PV inverter product,
chosen because a lot of them are actually exposed — that is what makes DER-hunting scanners find it.
Everything downstream reads the identity from this ticket.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Candidate PV inverter products surveyed on Shodan and Censys, with the exposed population count recorded for each
- [ ] One product chosen, with the choice justified by exposed population rather than by preference
- [ ] Identity recorded: vendor string, model string, serial and firmware version formats, HTTP `Server` header, login page appearance
- [ ] The SunSpec Models that product exposes are listed, including which inverter Model applies
- [ ] The decision is written down where the implementation tickets can read it
- [ ] No copyrighted vendor firmware assets are copied into the repository — appearance is described, to be rebuilt
