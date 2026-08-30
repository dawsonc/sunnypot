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

## Comments

**Models 120, 121, 122 and 160 are unowned, and this ticket depends on them.** ADR 0002 puts
Immediate Controls at 40228 *because* Nameplate (120), Basic Settings (121) and Measurements_Status
(122) sit between Model 101 and Model 123 in the Fronius chain: "Serving only three models puts
Immediate Controls at 40122 instead of 40228 — every Fronius-aware client and every published
Fronius register map would then read the wrong registers." The control addresses this ticket must
honour (40232 `Conn`, 40233 `WMaxLimPct`, 40237 `WMaxLim_Ena`, ...) are only correct if those blocks
exist and are the right length.

Ticket 02 built the chain as far as Common and terminated it; ticket 03 owns Model 101. No ticket
owns 120/121/122, or the optional 160. Either this ticket grows to serve them as correctly-shaped
constant blocks, or a ticket between 03 and 05 does — but the addresses here are wrong until
something does. Raised by the ticket 02 spec review.
