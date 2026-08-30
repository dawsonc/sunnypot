# 11 — Verify the external fingerprint

**What to build:** Proof that the emulation works, from outside. Shodan or Censys must classify the
host as a solar device — if they don't, no attacker searching for one will find it, and every later
observation is worthless. This is the project's go/no-go milestone and it gates collection.

**Blocked by:** 10.

**Status:** ready-for-agent

- [ ] Both surfaces are confirmed reachable from outside the network
- [ ] Shodan or Censys classifies the host as a solar or DER device, and the evidence is recorded
- [ ] The Modbus and HTTP fingerprints agree with each other in the external view
- [ ] If the host is not classified as a solar device, the gap is recorded and ticket 01 reopens before any collection begins
- [ ] Only once classification passes: the address is submitted for on-demand scanning and the 7-day and 30-day collection clocks start
