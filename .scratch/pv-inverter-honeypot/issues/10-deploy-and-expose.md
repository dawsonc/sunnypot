# 10 — Deploy to the Pi and expose safely

**What to build:** Sunnypot running on the Pi, reachable from the internet, shipping captures to
storage, and unable to reach anything else. Read ADR 0001 before changing anything about the network
boundary — the residential deployment is a deliberate trade and its consequences are recorded there.

**Blocked by:** 05, 08, 09.

**Status:** ready-for-agent

- [ ] The binary is cross-compiled for the Pi target and deploys as a file copy with no runtime dependencies
- [ ] A service unit starts sunnypot at boot and restarts it after a crash or power cut
- [ ] A setup script in the repository provisions a fresh device
- [ ] Each exposed surface is reachable through an inbound port-forward
- [ ] Egress is default-deny with a single exception for the storage endpoint, and the device has no reachability into the rest of the network
- [ ] The deployed storage credential grants object-put only, with bucket versioning and object lock enabled
- [ ] A billing alert is configured on the storage account
- [ ] No deployment configuration or credential is committed
