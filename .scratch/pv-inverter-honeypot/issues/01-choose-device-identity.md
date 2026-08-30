# 01 — Choose and document the device identity

**What to build:** A decision, not code. Sunnypot impersonates one specific real PV inverter product,
chosen because a lot of them are actually exposed — that is what makes DER-hunting scanners find it.
Everything downstream reads the identity from this ticket.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Candidate PV inverter products surveyed on Shodan and Censys, with the exposed population count recorded for each
- [x] One product chosen, with the choice justified by exposed population rather than by preference
- [x] Identity recorded: vendor string, model string, serial and firmware version formats, HTTP `Server` header, login page appearance
- [x] The SunSpec Models that product exposes are listed, including which inverter Model applies
- [x] The decision is written down where the implementation tickets can read it
- [x] No copyrighted vendor firmware assets are copied into the repository — appearance is described, to be rebuilt

## Answer

**Fronius Primo 5.0-1 208-240 with an integrated Fronius Datamanager 2.0.** Recorded in
`docs/adr/0002-impersonate-fronius-primo.md`, with a glossary entry for _Device identity_ in
`CONTEXT.md`. Tickets 02, 03, 05, 06, 07 and 11 read their identity values from that ADR.

Headlines:

- **Survey.** Forescout's May 9 2025 Shodan census: ~35,000 exposed solar devices, 42 vendors. Top
  five products are SMA Sunny WebBox ~10,953 (33%), **Fronius ~4,000**, Solar-Log ~3,000, Contec
  SolarView Compact ~2,000–3,000, Sungrow WiNet/Logger1000 ~2,000. Fronius is the only inverter in
  that list — every other entry is a datalogger or monitor. Sunny WebBox is bigger but its count is
  reported *after* honeypot filtering, i.e. the fingerprint is already honeypot-saturated.
- **Identity.** `Mn = Fronius`, `Md = Primo 5.0-1 208-240`, `Opt = 3.28.1-3` (Datamanager firmware,
  format `3.MINOR.PATCH-BUILD`), `Vr = 1.19.10-0` (inverter firmware), `SN` = 8 numeric digits,
  `DA = 1`. HTTP `Server: webserver` on port 80 (lighttpd 1.4.33 with `server.tag` overridden), login
  users `admin` and `service`, unauthenticated Solar API under `/solar_api/v1/`.
- **SunSpec chain.** `SunS` at 40001, then Common (1, L=65) → **Inverter 101, single phase** →
  Nameplate (120) → Basic Settings (121) → Measurements_Status (122) → Immediate Controls (123) →
  Multiple MPPT (160, N=2) → `0xFFFF`. Integer + scale factor variant, not float. Model 124 storage
  deliberately absent. Exact addresses and control registers are in the ADR.
- **Assets.** Appearance is described in prose; no vendor logo artwork, CSS, JS, images, or extracted
  firmware in the repo.

## Comments

**Survey provenance.** No first-party Shodan or Censys query was run — this environment has no
credentials for either, and both APIs reject anonymous requests. The counts above are third-party
measured figures with sources and dates. The ADR carries the exact Shodan and Censys query strings to
run; record the counts and dates against them, especially the `country:US` variants, since those
measure how plausible this identity is on our own vantage point.

**Three fields are unverified and are verification actions for ticket 11**, before the collection clock
starts: `Md` (medium confidence — the register map's own example is `IG+150V`, no captured Primo `Md`
was found), `Vr` (low confidence — no primary source for the inverter firmware version format), and
inverter Model 101 vs 102 (the US 208-240 Primo lands across two lines of a split-phase service, so 102
is the plausible alternative). If a captured banner disagrees, change the ADR and let it flow
downstream rather than patching values at the surfaces.

**Firmware version choice is deliberate.** `3.28.1-3` is real, several releases behind the current
`3.34.1-5`, and past `3.14.1` — the fix for CVE-2019-19228/19229. Advertising a pre-3.14.1 version
would be better bait but would mean claiming a vulnerability sunnypot does not have, which measures our
bait rather than the population's behaviour.
