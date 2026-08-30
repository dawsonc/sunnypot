# Spec: PV inverter honeypot (v1)

Status: ready-for-agent

## Problem Statement

The grid is absorbing distributed energy resources at record pace, and a growing share of them are
consumer-owned devices reachable from the public internet. Nobody working on this problem has a good
answer to a basic question: what actually knocks on a DER's door? Published DER security work is
dominated by vulnerability research against specific products — what an attacker *could* do given a
device. There is far less on what unsolicited traffic a grid-edge device genuinely attracts once it is
visible, how quickly it is discovered, and whether any of that traffic is DER-aware at all rather than
the same commodity scanning every internet-facing host receives.

Answering it requires a device that is exposed, believable, and instrumented — and running a real
inverter on the public internet is not an option.

## Solution

Sunnypot: a low-interaction honeypot impersonating a grid-tied PV inverter, exposed on a residential
connection, recording everything it is sent.

It presents the two surfaces a real exposed inverter presents — a Modbus/TCP endpoint serving a
SunSpec register map, and an HTTP admin interface — and backs them with a plant that behaves like a
real installation: generation tracking the sun across the day and the seasons, and responding when an
attacker writes a control point. Every connection, request, credential, and uploaded file is captured
to append-only remote storage the honeypot itself cannot read back or tamper with.

The deliverable is an open-source repository and a blog post describing what a grid-edge device sees.

## User Stories

### Being found

1. As a researcher, I want the honeypot to serve a SunSpec-conformant register map, so that scanners
   fingerprinting for DERs classify it as a solar device rather than an unidentified Modbus endpoint.
2. As a researcher, I want the device identity chosen from a survey of what is actually exposed on
   Shodan and Censys, so that the fingerprint matches a population attackers already hunt for.
3. As a researcher, I want to verify from outside that Shodan or Censys tags the honeypot as a solar
   device, so that I know the emulation works before spending a month collecting data.
4. As an internet scanner, I want the SunSpec identifier at the documented base register, so that
   standard discovery logic finds the model chain without special-casing.
5. As an internet scanner, I want the HTTP surface to return a plausible vendor login page, so that
   web-based device fingerprinting classifies the host consistently with its Modbus surface.
6. As a researcher, I want the Modbus and HTTP surfaces to agree about what device this is, so that a
   scanner correlating both does not see a contradiction.

### Being believable over time

7. As an attacker, I want generation to be zero at night and peak near local solar noon, so that the
   device does not obviously contradict its own geolocation.
8. As an attacker, I want output to vary with the season, so that a device observed across weeks does
   not look like a static fixture.
9. As an attacker, I want generation to fluctuate irregularly under cloud, so that the curve does not
   read as a synthetic function.
10. As an attacker, I want cumulative energy totals to increase monotonically and consistently with
    observed instantaneous power, so that repeated polling does not expose an inconsistency.
11. As an attacker, I want register values to carry correct SunSpec scale factors, so that a decoded
    reading is a physically plausible magnitude for the array size advertised.
12. As a researcher, I want the plant's latitude to match the region my IP geolocates to, so that
    solar noon lands at the right wall-clock time for an observer who checks.
13. As an attacker, I want the device to respond to a curtailment write by actually reducing reported
    output, so that a control attempt appears to have worked.
14. As an attacker, I want a disconnect command to stop generation, so that the most consequential
    control point behaves as the protocol says it should.
15. As an attacker, I want unsupported function codes to return proper Modbus exception responses, so
    that error handling does not distinguish this device from real hardware.

### Capturing what happens

16. As a researcher, I want every TCP connection recorded with source address, timing, and duration,
    so that I can distinguish sustained interaction from a single-packet sweep.
17. As a researcher, I want every Modbus request recorded with its function code and register range,
    so that I can tell SunSpec-aware polling from blind register scraping.
18. As a researcher, I want every control write recorded with the value written, so that I can
    describe what attackers actually try to change.
19. As a researcher, I want every HTTP request recorded with method, path, headers, and body, so that
    I can identify exploit attempts against the web surface.
20. As a researcher, I want every credential pair attempted recorded, so that I can report which
    default credentials are in active use against DERs.
21. As a researcher, I want successful logins distinguishable from rejected ones, so that I can
    separate spraying from post-authentication behaviour.
22. As a researcher, I want uploaded firmware files captured whole and never unpacked, so that I hold
    the sample without ever giving it a chance to run.
23. As a researcher, I want request bodies captured inline up to a size cap with oversize ones flagged
    as truncated, so that I never silently lose evidence.
24. As a researcher, I want connection and request records joined by a shared identifier, so that I
    can reconstruct a full session from a flat event stream.
25. As a researcher, I want a stable event schema, so that a month of captures can be analysed as one
    corpus rather than several incompatible ones.

### Surviving and staying safe

26. As the operator, I want the honeypot to never execute or interpret anything it is sent, so that
    hostile input cannot become code on my network.
27. As the operator, I want captures shipped off the device continuously, so that a compromise cannot
    quietly edit the record of its own arrival.
28. As the operator, I want the honeypot's storage credential to permit writes and nothing else, so
    that a stolen key cannot read, alter, or delete the corpus.
29. As the operator, I want captures buffered locally through a network outage and delivered when it
    clears, so that connectivity trouble does not cost me data.
30. As the operator, I want the device to reach exactly one external endpoint and nothing else, so
    that a compromised honeypot cannot scan, pivot, or join a botnet.
31. As the operator, I want the honeypot to need no reachability into my own network, so that its
    isolation does not depend on host-level rules alone.
32. As the operator, I want a billing alert on the storage account, so that credential abuse shows up
    as a notification rather than an invoice.
33. As the operator, I want the device to recover on its own after a power cut or crash, so that
    collection continues without me noticing it stopped.
34. As the operator, I want the plant's state to be a function of wall-clock time rather than uptime,
    so that a restart does not produce an implausible discontinuity in generation.

### Building and publishing

35. As a developer, I want the whole thing to be one static binary with no runtime dependencies, so
    that deployment is a file copy and a service unit.
36. As a developer, I want site parameters, identity strings, credentials, and the storage endpoint in
    a single config file, so that a second instance is a config change rather than a rebuild.
37. As a developer, I want that config gitignored with a committed example, so that publishing the
    repository cannot leak my deployment.
38. As a developer, I want tests that drive the real server over real TCP, so that what I verify is
    what a scanner would see.
39. As a developer, I want time and randomness controllable in tests, so that plant behaviour is
    asserted deterministically.
40. As a blog reader, I want the accepted credential set and impersonated identity disclosed, so that
    I can judge which attacker populations the findings represent.
41. As a blog reader, I want the limits of a low-interaction honeypot on a single residential vantage
    point stated plainly, so that I do not over-read the conclusions.

## Implementation Decisions

**Language and target.** Rust, deployed to a Raspberry Pi Zero 2 W. Build for
`aarch64-unknown-linux-gnu`, or a musl target for a fully static binary. Tokio for the async runtime,
axum for the HTTP surface. Modbus framing is hand-written — available crates are client-oriented, and
this is a server.

**Module shape.** Four modules: the plant, the two surfaces, and capture. The plant owns all device
state; surfaces read from it and never hold their own copy. Capture is a sink both surfaces write to.

**The plant is concrete, not a trait.** A single `PvPlant` type constructed from config, with a
deliberately narrow interface: sample the plant at an instant to get a `PlantState`, and apply a
control write to it. One implementation means a trait would be indirection buying nothing; the
narrowness of that interface, not the presence of a trait, is what makes extraction cheap when a
second plant arrives. Note for whoever hits that point: adding storage is not a plant swap — it also
adds a Model to the register map and changes the device identity, so the abstraction that eventually
wants extracting is the whole profile, not the plant alone.

**Plant behaviour.** Clear-sky generation derived from solar position for the configured latitude,
longitude, and date, scaled by array capacity, reduced by a seeded noise term standing in for cloud,
then multiplied by the current curtailment factor. Cumulative energy accumulates from generation and
must stay monotonic and consistent with instantaneous power across restarts, so it is derived from
wall-clock time rather than process uptime.

**SunSpec register map, typed and in code.** Model 1 (common), the inverter model appropriate to the
impersonated product, and Model 123 (immediate controls). Big-endian, holding registers, SunSpec
identifier at the documented base address, correct scale factors throughout. The map is Rust types —
explicitly not config-described data, which is Conpot's design and is incompatible with a live plant.

**Modbus surface.** Function codes 3, 6, and 16. Reads project the current plant state through the
register map. Writes to Model 123 control points — the curtailment percentage and its enable flag, and
the connect control — are applied to the plant and are visible in subsequent reads. Unsupported
function codes and out-of-range register addresses return the correct Modbus exception responses.
Unauthenticated, as the protocol specifies.

**HTTP surface.** A vendor-plausible login page accepting a small configured set of real-world default
credentials, rejecting everything else. Behind it: a status dashboard rendering plant state, a settings
page whose writes reach the plant, and a firmware upload page that accepts a file, stores it as a
capture artifact, and never unpacks, inspects, or executes it. Pages are rebuilt to look plausible,
not copied from vendor firmware.

**Capture schema.** Newline-delimited JSON, one flat stream. Connection-lifecycle events carry source
address, transport details, and duration; request events carry protocol payload. A shared connection
identifier joins them. Bodies and uploads are base64-inlined up to roughly 64KB with a truncation flag
beyond that, so artifacts never need a second transport path.

**Capture sink.** A trait, with an in-memory implementation for tests and an S3-compatible uploader in
production. The uploader batches events, gzips them, and PUTs objects on a size or time trigger, with
local buffering across outages. Provider is late-bound — R2, S3, and B2 are interchangeable at this
interface. The deployed credential grants object-put only: no list, no get, no delete. Bucket
versioning and object lock are enabled so the corpus is append-only from the honeypot's perspective.

**Configuration.** One file supplying site parameters (latitude, longitude, timezone, array capacity,
inverter rating), identity strings, accepted credentials, and the storage endpoint. Gitignored, with a
committed example documenting shape but not values.

**Deployment.** Raspberry Pi OS Lite, a systemd unit that restarts on failure and starts at boot, and
a setup script in the repository. A single inbound port-forward per exposed surface. Egress is
default-deny at the VLAN with one exception: outbound 443 to the storage endpoint. The honeypot needs
no reachability into the rest of the network.

**Sequencing.** The device identity is chosen first, by surveying Shodan and Censys for the exposed
population of candidate products. Everything downstream — register map details, identity strings, the
look of the login page — depends on that choice.

## Testing Decisions

**What makes a good test here.** Only external behaviour: the bytes a client receives, and the events
that reach the capture sink. No test reaches into plant internals or asserts on how a value was
computed. If a change is invisible to both a scanner and the corpus, no test should fail.

**One seam.** The application is constructed with its config, a clock, and a capture sink, and bound
to an ephemeral port. Tests speak real Modbus/TCP and real HTTP over loopback and assert on responses
and on captured events. The clock is controllable and the RNG seeded, so plant behaviour is
deterministic without any plant-level test double.

**Deliberately no plant seam.** Solar behaviour is asserted by reading registers at controlled times
rather than by unit-testing the plant directly. This costs a decode step and buys coverage of the
scale-factor and encoding path, which is where realism-breaking bugs live: a register reading 42 when
it should read 4200 is invisible to a plant unit test and obvious to a scanner.

**Areas requiring coverage.** SunSpec discovery and model-chain traversal; scale factors producing
plausible magnitudes; generation at night, at solar noon, and across seasons; monotonic energy totals
across a simulated restart; curtailment write followed by a read showing reduced output; disconnect
halting generation; Modbus exception responses for unsupported function codes and bad addresses;
credential acceptance and rejection; capture of requests, credentials, and uploads; body truncation at
the cap; the join between connection and request events; and sink batching behaviour including
buffering across a simulated outage.

**Prior art.** None — this is a greenfield repository. These tests establish the pattern.

## Out of Scope

- Battery storage in any form: no SOC, no Model 124, no hybrid inverter identity. That is a different
  device identity with a different fingerprint, not an increment on this one.
- EV chargers, OCPP, and any CSMS emulation; IEEE 2030.5; DNP3.
- Medium- and high-interaction surfaces. Nothing an attacker sends is ever executed or interpreted,
  and no real firmware runs anywhere in this system.
- Unpacking, analysing, or detonating captured samples. v1 captures and stores them, nothing more.
- A cloud-hosted comparison instance. Recorded in ADR 0001 as a deliberate omission — its absence
  bounds what the findings can claim.
- Dashboards, Grafana, or any hosted log platform. The corpus is JSONL in object storage; `jq`
  answers the first month's questions.
- Config-driven register maps, a plant trait, or any device-profile abstraction.
- Publishing the capture corpus. Code is public; whether raw captures ever are is a separate decision
  taken after review.

## Further Notes

The first milestone is external and precedes any data collection: confirm that Shodan or Censys
classifies the honeypot as a solar device. If they do not, no attacker searching for one will find it
either, and everything downstream is wasted effort. Only once that passes should the IP be submitted
for on-demand scanning.

Collection timeline is a first look at 7 days and no conclusions before 30. DER-targeted traffic is
rare relative to commodity scanning, and a week's data would support claims about background noise
only.

ADR 0001 records the residential deployment and its consequences, including the accepted risk of abuse
reports against the ISP subscriber account. Read it before changing anything about the network
boundary.

Open and non-blocking: the storage provider, which the sink interface makes a deploy-time choice.
