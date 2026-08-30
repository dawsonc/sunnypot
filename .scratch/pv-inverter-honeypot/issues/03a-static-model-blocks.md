# 03a — The static Models that hold the chain's addresses

**What to build:** The SunSpec chain gains Nameplate (120), Basic Settings (121) and
Measurements_Status (122) between the inverter Model and Immediate Controls, plus Multiple MPPT
(160) after it. These carry constants and status, not new plant behaviour — but without them every
model after 101 sits 106 registers too low, and a Fronius-aware client reading published addresses
finds nothing there.

**Blocked by:** 03 — Model 101 has to exist before the blocks that follow it can be placed.

**Status:** ready-for-agent

- [ ] Models 120, 121 and 122 are served between Model 101 and Model 123, with the lengths ADR 0002 fixes: 26, 30 and 44
- [ ] Model 160 is served after Model 123 with `N = 2`, the two MPPT strings the Primo has
- [ ] A chain walk from the SunSpec identifier visits every model at the address ADR 0002's table gives — the whole table, model by model, not just the end marker
- [ ] Model 123's header lands at 40228, so ticket 05's control registers are at their published addresses
- [ ] `WMax` in Model 121 is the inverter rating from config, since Model 123's curtailment percentage is a percentage of it
- [ ] Nameplate and settings values derive from site config and are physically consistent with the array capacity and inverter rating the device advertises
- [ ] Fields SunSpec marks unimplemented use the documented not-implemented encoding for their type, not zero
- [ ] Scale factors are correct, so a decoded nameplate rating is a plausible magnitude
- [ ] The plant does not grow to fill these blocks — they are constants and status, and no new plant behaviour is added here
- [ ] All of it is asserted by reading registers over Modbus/TCP, never by inspecting the register map directly

## Comments

**Why this is a ticket at all.** ADR 0002 argued for it explicitly — "This is wider than the spec, and
deliberately so… 120, 121 and 122 are served as correctly-shaped blocks carrying nameplate constants
and status" — but that widening never reached a ticket when the spec was decomposed into 01–11. The
implementation tickets divide the chain as 02 → Common, 03 → Model 101, 05 → Model 123, and nothing
owned the middle. Found by the ticket 02 spec review.

**The arithmetic, for whoever picks this up.** Each block is a 2-register header plus its body, so an
address is the sum of everything before it. The three blocks contribute 2+26 + 2+30 + 2+44 = **106
registers**, and that is the entire reason Immediate Controls sits at 40228. Omit them and 123's
header lands at 40122, its body at 40124, and `Conn` — body offset 2 — at 40126 instead of 40232.
Every control register in ticket 05 moves by 106 and a client writing the published address gets an
illegal-data-address exception.

**Lengths are as load-bearing as presence.** A Nameplate block served with the wrong length shifts
everything downstream exactly as badly as omitting it. That is what the whole-table assertion in the
third checkbox is for: any future block that changes size should fail loudly at the address it
displaces, rather than quietly moving 123.

**Model 160 is not load-bearing.** It sits *after* 123, so its absence shifts nothing ticket 05
touches — it only moves the end marker. It is in this ticket because it is cheap and real, not
because anything depends on it. Drop it if it fights.

**Why this is invisible to ticket 02's tests.** A generic SunSpec client that walks the chain finds
Model 123 wherever it is and works fine, so the traversal test stays green either way. The breakage
is specific to clients using hardcoded Fronius addresses from the published register map — which is
exactly the population sunnypot is trying to look real to, and the contradiction ticket 11 exists to
catch.

**Numbering.** `03a` rather than a renumber: 04–11 are taken and already cross-referenced from other
tickets, the ADR, and commit messages. It bends `docs/agents/issue-tracker.md`'s "numbered from 01"
slightly, in exchange for not invalidating every existing reference.
