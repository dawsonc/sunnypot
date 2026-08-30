# 04 — Cumulative energy survives restart

**What to build:** Lifetime energy totals that an attacker polling over days cannot catch out. They
only ever increase, they agree with the instantaneous power being reported, and a process restart
doesn't reset or jump them.

**Blocked by:** 03.

**Status:** ready-for-agent

- [ ] Cumulative energy registers increase monotonically
- [ ] The total is consistent with instantaneous power integrated over the elapsed period
- [ ] The total derives from wall-clock time rather than process uptime
- [ ] A test constructs a fresh application with the same config and a later clock, and observes a plausibly continued total rather than a reset or a discontinuity
