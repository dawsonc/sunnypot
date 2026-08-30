# Deploy on a residential connection, not a cloud VPS

Sunnypot runs on a Pi on an isolated home VLAN, port-forwarded to a residential ISP connection,
rather than on a cloud VPS. Attackers reach DERs by scanning for known device fingerprints, and real
PV inverters live behind consumer ISPs — a SunSpec endpoint answering from a datacentre ASN is a
tell that ASN-aware scanners can filter on. Credibility is the entire experiment, so we buy it with
blast radius rather than the other way round.

## Consequences

- The honeypot sits on the same WAN address as the household. Egress is default-deny precisely
  because a compromised Pi would otherwise be attributable to the ISP subscriber account.
- Attack traffic generates abuse reports against that account, and many residential contracts
  prohibit running servers. This is a live risk, accepted knowingly.
- Findings are specific to a residential vantage point and do not generalise to utility-scale or
  datacentre-hosted DER infrastructure. Say so in the writeup.
- A cloud-hosted comparison instance would strengthen any claim that observed traffic is
  DER-targeted rather than background noise. Deliberately out of scope for v1.
