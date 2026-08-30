# Impersonate a Fronius Primo (Datamanager 2.0), not an SMA Sunny WebBox

Sunnypot presents itself as a **Fronius Primo 5.0-1 208-240** grid-tied PV inverter with an integrated
**Fronius Datamanager 2.0** datalogger, serving SunSpec over Modbus/TCP on 502 and the Datamanager web
interface on 80. Every downstream surface — the register map, the identity strings, the login page —
reads its values from this document.

The choice is made on exposed population, not preference: Fronius is the most numerous *inverter*
(as opposed to datalogger) in the measured population of internet-exposed solar equipment, and it is
the largest exposed product that natively presents both surfaces sunnypot intends to expose.

## The survey

No first-party Shodan or Censys query was run — this repo has no API credentials for either. The
counts below are from Forescout's May 9 2025 census of internet-exposed solar equipment, which used
Shodan and is the most recent published measurement: **~35,000 devices from 42 vendors** with exposed
management interfaces, 76% in Europe, 17% in Asia, 8% rest of world (Germany 20%, Greece 20%, Japan 9%,
Portugal 9%, Italy 6%).

| Product | Exposed (May 2025) | Device class | Presents Modbus/TCP SunSpec | Presents local HTTP admin | Verdict |
| --- | --- | --- | --- | --- | --- |
| SMA Sunny WebBox | ~10,953 (33%) | Datalogger, discontinued | No (Speedwire/RPC) | Yes | Rejected — not an inverter; count is *after* Forescout filtered honeypots, i.e. the fingerprint is already honeypot-saturated |
| **Fronius inverters (Primo/Symo + Datamanager)** | **~4,000** | **Inverter** | **Yes, native** | **Yes** | **Chosen** |
| Solare Datensysteme Solar-Log | ~3,000 | Datalogger | Primarily a Modbus master polling inverters | Yes | Rejected — not an inverter |
| Contec SolarView Compact | ~2,000–3,000 (8%) | Monitor | No | Yes | Rejected — not an inverter; Japan-concentrated, implausible on a US residential IP |
| Sungrow WiNet / Logger1000 | ~2,000 | Dongle / logger | Partial | Yes | Rejected — not an inverter |

Candidates with no published exposure count, rejected on fit rather than population:

- **SolarEdge SE-series** — Modbus/TCP is off by default and there is no rich local web admin (SetApp
  is a phone app over a local AP). The HTTP half of the fingerprint would have nothing to serve.
- **Enphase IQ Gateway / Envoy** — large US population and a genuinely exposed web UI, but it is a
  *gateway* fronting microinverters. Impersonating it means impersonating a system, not the single
  inverter identity in `CONTEXT.md`, and SunSpec Modbus is an add-on rather than the primary surface.
- **Huawei SUN2000** — very large global fleet and native Modbus/TCP, but effectively absent from the
  US residential market. Implausible behind a US consumer ISP (see ADR 0001).
- **Growatt** — inbound exposure is dataloggers and ShineWiFi dongles; the inverters themselves are
  cloud-attached outbound-only.

Fronius wins because it is the only product in the measured top five that is an inverter, and because
Modbus/TCP SunSpec and the web interface are both first-class documented features of the same box.
Fronius exposure is Europe/Australia-weighted rather than US-weighted, which is a real weakness of this
choice — see Consequences.

### Queries to run before exposure

These were not run here. Run them, record the count and the date, and append the numbers to this ADR.
The `country:US` variants matter more than the global ones: they measure how plausible this identity is
on our actual vantage point.

```
# Shodan
http.html:"solar_api"                       # Fronius Solar API, strongest Fronius web tell
http.html:"fronius"                         # broader, catches rebadged pages
"Server: webserver" port:80 http.html:"fronius"
port:502 "SunS"                             # SunSpec-answering Modbus endpoints
http.html:"solar_api" country:US            # US population, our vantage point
html:"Sunny WebBox"                         # baseline for the honeypot-saturated fingerprint
tag:ics port:502 country:US

# Censys
services.http.response.html_title: "Fronius"
services.http.response.body: "solar_api"
services.service_name: MODBUS and location.country: "United States"
```

## The identity

Field-by-field, with the confidence in each. Anything below `high` is a verification action for
ticket 11 before the collection clock starts.

### Device

| Property | Value | Confidence | Source |
| --- | --- | --- | --- |
| Vendor | `Fronius` | high | Fronius Datamanager register map fixes `Mn` to `Fronius` |
| Product | Fronius Primo 5.0-1 208-240 | high | Fronius US Primo line is `x.x-1 208-240`, single phase, 3.8–15.0 kW, residential |
| AC rating | 5.0 kW | high | product line |
| MPPT inputs | 2 | high | Primo has dual MPPT; matches SunSpec Model 160 with `N = 2` |
| Datalogger | Fronius Datamanager 2.0, integrated | high | shipped in every SnapINverter |

Site parameters (array capacity, latitude, longitude, timezone) are *not* fixed here — they are config,
per the spec. Constraint only: the array should be sized for a DC:AC ratio around 1.2–1.3 against the
5.0 kW inverter (≈6.0–6.5 kWp), and the latitude must match where the deployment IP geolocates.

### SunSpec Common block strings

| Register | Field | Type | Value | Confidence |
| --- | --- | --- | --- | --- |
| 40005–40020 | `Mn` Manufacturer | String32 | `Fronius` | high |
| 40021–40036 | `Md` Device model | String32 | `Primo 5.0-1 208-240` | medium — the map's own example is `IG+150V`, i.e. the product name without the vendor prefix, but no captured Primo `Md` was found |
| 40037–40044 | `Opt` Options | String16 | `3.28.1-3` — Datamanager firmware version | high (format), medium (value) |
| 40045–40052 | `Vr` SW version | String16 | `1.19.10-0` — inverter firmware | **low** — format unconfirmed |
| 40053–40068 | `SN` Serial number | String32 | 8 numeric digits, e.g. `30514231` | medium |
| 40069 | `DA` Modbus device address | uint16 | `1` | high |

Notes on the version fields. Fronius versions the Datamanager as `3.MINOR.PATCH-BUILD` and pairs it
with a Hybridmanager `HM 1.x.y` — SEC Consult's advisory quotes a real device as
`SWVersion 3.10.3-1 (HM 1.9.2)`. The published changelog's release sequence runs
`3.12.2-2, 3.12.5-1, 3.13.3-2, 3.14.1-10, 3.15.3-1, 3.16.7-1, 3.17.3-1, 3.18.6-1, 3.18.7-1, 3.19.10-1,
3.20.6-1, 3.21.4-1, 3.23.6-1, 3.24.2-1, 3.25.1-3, 3.25.2-1, 3.26.1-3, 3.27.1-3, 3.28.1-3, 3.29.1-3,
3.30.1-3, 3.31.1-5, 3.31.1-7, 3.32.1-2, 3.34.1-5`, newest last. `3.28.1-3` is chosen deliberately: real,
several releases behind current, which is what an installed-and-occasionally-updated residential unit
looks like. It is also comfortably past `3.14.1`, the fix for CVE-2019-19228/19229 — we are not
advertising a device with a known unauthenticated path traversal, because sunnypot must not appear to
*have* the vulnerability an attacker then exploits for real.

`Vr` is the one field where no primary source was found. A wrong value here is low-risk — scanners key
on `Mn` and `Md`, not on inverter firmware — but confirm it from a captured banner before exposure.

The serial is arbitrary within a plausible format and is set per deployment in config. It must not be
copied from a real unit: a real serial would misattribute sunnypot's traffic to a real owner's device.

### SunSpec model chain

Reproduce the Fronius Datamanager chain exactly, including its quirks. Base register 40001, big-endian
holding registers, unit ID 1, function codes 3/6/16.

| Address | Model | ID | Length | v1 |
| --- | --- | --- | --- | --- |
| 40001–40002 | SunSpec identifier `SunS` (`0x53756E53`) | — | 2 | Required |
| 40003 | Common | 1 | 65 | Required |
| 40070 | **Inverter, single phase** | **101** | 50 | Required |
| 40122 | Nameplate | 120 | 26 | Required as structure — see below |
| 40150 | Basic Settings | 121 | 30 | Required as structure — see below |
| 40182 | Measurements_Status | 122 | 44 | Required as structure — see below |
| 40228 | Immediate Controls | 123 | 24 | Required |
| 40254 | Multiple MPPT | 160 | 48 | Optional, `N = 2` |
| 40304 | End marker: ID `0xFFFF`, length `0` at 40305 | — | 2 | Required |
| — | Basic Storage Controls | 124 | — | **Never** — Symo Hybrid only, and storage is out of scope |

**This is wider than the spec, and deliberately so.** The spec's implementation decisions name three
models: Common, the inverter model, and Immediate Controls. But SunSpec is a linked chain walked by
length, and the Fronius addresses are what they are *because* 120, 121 and 122 sit between 101 and 123.
Serving only three models puts Immediate Controls at 40122 instead of 40228 — every Fronius-aware
client and every published Fronius register map would then read the wrong registers, which is precisely
the contradiction ticket 11 exists to catch. So 120, 121 and 122 are served as correctly-shaped blocks
carrying nameplate constants and status, not as new dynamic behaviour: the plant does not grow to fill
them. Model 160 is the one genuinely optional addition, and it is cheap — two MPPT strings the Primo
really has.

Two Fronius-specific details worth reproducing, because a scanner that knows Fronius would notice them
missing:

- The Common block length is **65**, not the 66 of a textbook SunSpec map. Standard discovery walks the
  chain by length, so 65 is what keeps the chain traversable *and* Fronius-shaped.
- The Datamanager also serves non-SunSpec Fronius registers in the same map: `212` `F_Delete_Data`,
  `213` `F_Store_Data`, `214` `F_Active_State_Code`, `215` `F_Reset_All_Event_Flags`,
  `216` `F_ModelType` (1 = float, 2 = int+SF), `217` `F_Storage_Restrictions_View_Mode`, and site totals
  at `500` `F_Site_Power`, `502` `F_Site_Energy_Day`, `506` `F_Site_Energy_Year`,
  `510` `F_Site_Energy_Total`. Optional for v1; note it if ticket 11 finds the fingerprint thin.

**Inverter Model 101 is the one that applies.** The Primo is a single-phase inverter; Fronius maps
101 single phase / 102 split phase / 103 three phase. The US 208-240 variant lands across two lines of a
split-phase service, so 102 is the plausible alternative — if a captured Fronius banner shows 102,
change it here and let the change flow downstream.

The register map is the **integer + scale factor** variant (Models 101/120–123/160), which corresponds
to `F_ModelType = 2`. Fronius also ships a float variant (Models 111/112/113, 211–213). Integer is what
this ADR specifies; SunSpec scale factors are then load-bearing for plausibility, which is the point of
ticket 03's decode assertions. Immediate Controls scale factors from the map: `WMaxLimPct_SF = -2`,
`OutPFSet_SF = -3`, `VArPct_SF = 0`.

Control points ticket 05 must honour, all read/write via FC 6 and FC 16:

| Register | Field | Meaning |
| --- | --- | --- |
| 40232 | `Conn` | 0 = Disconnected, 1 = Connected |
| 40233 | `WMaxLimPct` | Power output limit, % of `WMax`, scale factor −2 |
| 40237 | `WMaxLim_Ena` | 0 = Disabled, 1 = Enabled |
| 40238 | `OutPFSet` | Power factor setpoint, scale factor −3 |
| 40242 | `OutPFSet_Ena` | 0 = Disabled, 1 = Enabled |
| 40244 | `VArMaxPct` | Reactive power, % of `VArMax` |
| 40250 | `VArPct_Ena` | 0 = Disabled, 1 = Enabled |

Note that Modbus/TCP is **off by default** on a real Datamanager — an exposed Fronius on 502 is one
whose owner turned it on for a home-automation integration and then port-forwarded it. That is a real
and common configuration, and it is the population sunnypot is joining.

### HTTP surface

| Property | Value | Confidence | Source |
| --- | --- | --- | --- |
| Port | 80, cleartext | high | the Datamanager serves HTTP; SEC Consult flagged unencrypted HTTP as a finding |
| `Server` header | `webserver` | high | SEC Consult captured `Server: webserver` — lighttpd 1.4.33 with `server.tag` overridden |
| Login usernames | `admin`, `service` | high | Datamanager has three password types: administrator (`admin`), service, and user |
| Solar API base | `/solar_api/v1/` | high | Fronius Solar API, **unauthenticated** on real devices |
| Notable paths | `/solar_api/v1/GetAPIVersion.cgi`, `/solar_api/v1/GetInverterRealtimeData.cgi`, `/admincgi-bin/service.fcgi` | high | Solar API docs; the `admincgi-bin` path is the CVE-2019-19229 traversal target |

Login page appearance, to be **rebuilt from this description** — see the constraint below:

- White page, Fronius red as the single accent colour on a top bar.
- The Fronius wordmark set as **text**, never the trademark logo artwork.
- A centred login dialog: a user selector offering `admin` and `service`, a password field, a submit
  button, and a "Forgot your password?" link.
- Language toggle (English / German) in the header.
- Footer line naming the device and the firmware version, consistent with `Opt` above:
  `Fronius Datamanager 2.0 · 3.28.1-3`.
- The dashboard behind it is the Datamanager "System Overview": current AC power, energy today, energy
  this year, energy total, and device status — the same quantities the Solar API returns and the same
  quantities the Modbus surface serves, because ticket 06 requires the two surfaces to agree.

A real Datamanager 2.0 has **no factory default password** — the admin password is set at commissioning.
Sunnypot deliberately accepts a small configured set of weak credentials anyway, because observing
post-authentication behaviour is the point and no attacker gets in otherwise. That is a knowing
deviation from the real product, and user story 40 already requires disclosing the accepted credential
set in the writeup.

The Solar API is the strongest single web fingerprint Fronius has, and it is unauthenticated on real
devices, so a scanner can read it without credentials. Whether v1 serves it is ticket 06's call; if
ticket 11 finds Shodan will not classify the host, serving `GetAPIVersion.cgi` and
`GetInverterRealtimeData.cgi` from plant state is the first thing to try.

### What is not copied

No vendor firmware asset enters this repository: no logo artwork, no CSS, no JavaScript, no images, no
extracted firmware. The appearance above is a written description, and the pages are rebuilt from it.
The strings, register addresses, and version numbers recorded here are interoperability facts taken from
public documentation and a public security advisory, which is what makes the deception work; the visual
identity is described so that nobody is tempted to unpack a firmware image to get it.

## Consequences

- **The vantage point and the identity disagree geographically.** Fronius exposure is concentrated in
  Europe and Australia; ADR 0001 puts sunnypot on a US residential connection. A Fronius Primo
  208-240 is a real US product, so the pairing is coherent, but a scanner segmenting by country sees a
  less common combination than a European Fronius or a US Enphase would be. Say so in the writeup.
- **Findings are Fronius-shaped.** Traffic that targets Fronius specifically (the `admincgi-bin`
  traversal, Solar API scraping) will be visible; traffic targeting SMA, SolarEdge, or Enphase will
  not. Any claim about "DER-targeted traffic" is really a claim about traffic targeting *this* identity
  plus generic SunSpec/Modbus scanning.
- **Advertising 3.28.1-3 costs some attacker interest.** A device advertising a pre-3.14.1 version
  would attract exploitation attempts against a known CVE, which is tempting bait. We are not doing
  that: sunnypot would then be claiming to have a vulnerability it does not have, and the resulting
  traffic would measure our bait rather than the population's behaviour.
- **Deliberately avoiding the Sunny WebBox fingerprint costs population but buys credibility.** The
  largest exposed fingerprint is also the one that census work already filters as honeypot-contaminated,
  and Shodan's Honeyscore exists. Joining the second-largest population that no honeypot template ships
  by default is the better trade.
- **Three fields are unverified** (`Md`, `Vr`, and inverter Model 101 vs 102). Ticket 11 gates
  collection on external classification; confirm these against a real banner sample before the clock
  starts, and reopen this ADR rather than patching values downstream.
- **Sunnypot answers every Modbus unit id, not only unit 1.** The identity above advertises
  `DA = 1`, and the chain is documented as unit ID 1. The implementation (ticket 02) deliberately
  answers whatever unit id it is asked for and echoes it back, because refusing other unit ids turns
  a scanner's unit-id sweep into silence, and engagement is the point of the exercise. Recorded here
  rather than left in the code: this is the one place where the served device knowingly differs from
  the documented one. If ticket 11 finds it reads as non-Fronius, restrict it and amend this bullet.
- **If storage is ever added**, this identity does not stretch. A Fronius Symo Hybrid is a different
  product with a different fingerprint and Model 124 — a new ADR, not an edit to this one.

## Sources

- Forescout, *The Security Risks of Internet-Exposed Solar Power Systems*, May 9 2025 —
  https://www.forescout.com/blog/the-security-risks-of-internet-exposed-solar-power-systems/
- SecurityWeek, *35,000 Solar Power Systems Exposed to Internet* (per-product breakdown) —
  https://www.securityweek.com/35000-solar-power-systems-exposed-to-internet/
- Fronius Datamanager register map, integer inverter models 101/102/103 —
  https://www.forum-fronius.pl/wp-content/uploads/asgarosforum/2812/Inverter_register_map.pdf
- Fronius Datamanager 2.0 operating instructions —
  https://manuals.fronius.com/html/4204260191/en-US.html
- Fronius Datamanager firmware changelog —
  https://www.fronius.com/en/~/downloads/Solar%20Energy/Firmware/SE_FW_Changelog_Fronius_Datamanager_DE-EN.pdf
- SEC Consult, *Multiple vulnerabilities in Fronius Solar Inverter Series (CVE-2019-19229,
  CVE-2019-19228)* — `Server: webserver`, lighttpd 1.4.33, version scheme —
  https://sec-consult.com/vulnerability-lab/advisory/multiple-vulnerabilites-in-fronius-solar-inverter-series-cve-2019-19229-cve-2019-19228/
- Fronius Primo 5.0-1 208-240 product page —
  https://www.fronius.com/en-us/usa/solar-energy/installers-partners/technical-data/all-products/inverters/fronius-primo-ul/fronius-primo-5-0-1-208-240
- Maesschalck et al., *World Wide ICS Honeypots: A Study into the Deployment of Conpot Honeypots*
  (Shodan Honeyscore precision, Conpot fingerprinting) —
  https://www.acsac.org/2021/workshops/icss/2021-icss-maesschalck.pdf
