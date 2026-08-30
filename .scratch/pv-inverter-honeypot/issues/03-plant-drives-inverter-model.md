# 03 — The plant drives live inverter registers

**What to build:** Reading the inverter Model returns generation the plant computed from solar
position at the current time — dark at night, peaking near local solar noon, varying across the year.
The device stops looking like a fixture and starts looking installed.

**Blocked by:** 02.

**Status:** ready-for-agent

- [ ] The plant is constructed from site config — latitude, longitude, timezone, array capacity, inverter rating
- [ ] The plant exposes a narrow interface: sample it at an instant, and apply a control write to it
- [ ] The inverter Model projects plant state through the register map with correct SunSpec scale factors
- [ ] Generation is zero at night and peaks near local solar noon for the configured latitude
- [ ] Generation varies across simulated dates in a way consistent with the season
- [ ] Cloud variation comes from a seeded RNG and is reproducible under a fixed seed
- [ ] All of the above is asserted by reading registers at controlled clock times, never by testing the plant directly
- [ ] Decoded register values are physically plausible magnitudes for the advertised array capacity
