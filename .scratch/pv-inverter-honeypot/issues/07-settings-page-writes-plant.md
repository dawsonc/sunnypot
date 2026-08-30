# 07 — Settings page writes reach the plant

**What to build:** Changing a setting in the web interface changes what the Modbus surface reports.
An attacker probing both surfaces finds one device, not two that disagree.

**Blocked by:** 05, 06.

**Status:** ready-for-agent

- [ ] The settings page exposes controls corresponding to plant-affecting values, including curtailment
- [ ] A change made through the web interface is visible on the Modbus surface on the next read
- [ ] A change made over Modbus is reflected on the settings page
- [ ] Every settings change is captured with the value submitted
