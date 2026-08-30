# 05 — Model 123 controls: curtailment and disconnect

**What to build:** An attacker who writes a control point sees the device obey. Curtailment reduces
reported output on the next read; disconnect stops generation entirely. This is the most consequential
thing an attacker can do to a PV inverter over Modbus, and capturing the attempt is the point of the
whole project.

**Blocked by:** 03.

**Status:** ready-for-agent

- [ ] FC6 and FC16 writes are accepted against Model 123 control points
- [ ] The curtailment percentage and its enable flag are applied to the plant, and subsequent reads show correspondingly reduced output
- [ ] The connect control halts generation when disconnected and resumes it when reconnected
- [ ] Curtailment persists across subsequent polls rather than decaying back
- [ ] Writes to read-only registers return the correct exception response
- [ ] Every control write is captured with the register written and the value written
