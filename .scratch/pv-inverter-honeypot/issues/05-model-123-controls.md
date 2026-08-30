# 05 — Model 123 controls: curtailment and disconnect

**What to build:** An attacker who writes a control point sees the device obey. Curtailment reduces
reported output on the next read; disconnect stops generation entirely. This is the most consequential
thing an attacker can do to a PV inverter over Modbus, and capturing the attempt is the point of the
whole project.

**Blocked by:** 03a — the control addresses below are only correct once the Models between 101 and 123 exist.

**Status:** ready-for-agent

- [ ] FC6 and FC16 writes are accepted against Model 123 control points
- [ ] The curtailment percentage and its enable flag are applied to the plant, and subsequent reads show correspondingly reduced output
- [ ] The connect control halts generation when disconnected and resumes it when reconnected
- [ ] Curtailment persists across subsequent polls rather than decaying back
- [ ] Writes to read-only registers return the correct exception response
- [ ] Every control write is captured with the register written and the value written

## Comments

**The Models this ticket's addresses depend on are ticket 03a.** ADR 0002 puts Immediate Controls at
40228 *because* Nameplate (120), Basic Settings (121) and Measurements_Status (122) sit between Model
101 and Model 123: "Serving only three models puts Immediate Controls at 40122 instead of 40228 —
every Fronius-aware client and every published Fronius register map would then read the wrong
registers." The addresses here (40232 `Conn`, 40233 `WMaxLimPct`, 40237 `WMaxLim_Ena`, ...) are wrong
by 106 registers until those blocks exist. `WMaxLimPct` is also a percentage of Model 121's `WMax`,
which 03a supplies.

Nothing owned those blocks when this ticket was written; ticket 03a now does. Raised by the ticket 02
spec review.
