# 02 — Modbus responder serving Model 1 over the test seam

**What to build:** A Modbus/TCP client connects to sunnypot, reads the SunSpec identifier and the
Model 1 common block, and gets back the chosen device identity. The connection and every request it
carried appear as captured events. This is the tracer bullet: it establishes the seam, the config, the
framing, and the capture path in one narrow complete slice.

**Blocked by:** 01 — the Model 1 block serves the identity strings chosen there.

**Status:** resolved

- [x] The application is constructed from config, a clock, and a capture sink, and binds an ephemeral port
- [x] Tests drive real Modbus/TCP over loopback and assert on returned bytes and on captured events — no plant-level or internal assertions
- [x] The SunSpec identifier is present at the documented base address and the Model 1 chain is traversable by standard discovery logic
- [x] Model 1 serves vendor, model, serial, and firmware version from config
- [x] FC3 reads return big-endian holding registers
- [x] Unsupported function codes and out-of-range addresses return correct Modbus exception responses
- [x] A connection-lifecycle event and per-request events are captured, joined by a shared connection identifier
- [x] Config supplies identity and site parameters, is gitignored, and has a committed example documenting shape but not values

## Answer

Implemented on `feat/02-modbus-responder-model-1`. A Modbus/TCP client connects, reads the SunSpec
identifier and the Common block, and gets the configured identity back; the session appears in the
corpus as an open event, one event per request, and a close event, all sharing a `connection_id`.

**The seam.** `App::bind(config, clock, sink)` binds every surface and returns once the ports are
bound, so a test that asks for port 0 can read back what it got (`src/app.rs`). Nothing else is
substitutable: the clock (`src/clock.rs`) and the capture sink (`src/capture.rs`) are the only two
doubles, exactly as the spec allows. 20 tests; 15 of them drive real Modbus/TCP over loopback and
assert only on returned bytes or on captured events. The other five are in `tests/config.rs` and
exercise `Config::from_toml_str`'s error contract directly, because a config that fails to load
cannot be observed at the seam at all. The one config test that *can* run through the seam does:
`the_committed_example_serves_a_coherent_device` starts an app from the committed example and reads
the identity back off the wire.

**Register map.** `src/modbus/registers.rs`, typed and in code. `SunS` at protocol address 40000,
then Common (id 1, length **65** — the Fronius length, not the textbook 66), then the `0xFFFF`
terminator at 40069. Addresses are 0-based on the wire; the ADR's tables are 1-based, and the module
documents the reconciliation.

**Chain traversal.** `tests/sunspec_discovery.rs` walks the chain through `tokio-modbus`, so the
framing sunnypot emits is read back by code that knows nothing about it rather than by our own
decoder. It does not run a real SunSpec discovery implementation — the walk is written out by hand,
and the library contributes framing only. It shows the chain is traversable by the standard
algorithm and lands exactly on the terminator, so no model is misaligned; confirming a real
scanner's implementation is ticket 11.

**Framing.** Hand-written (`src/modbus/frame.rs`), per the spec — the published crates are
client-shaped, and a honeypot needs to control exactly which byte it returns for input a real client
would never send. FC 3 only for now; 6 and 16 are ticket 05.

**Exceptions.** Unsupported function code → `0x01`; outside the mapped block, including a read that
starts inside and runs off the end → `0x02`; quantity of 0 or above 125, and an FC 3 payload too
short to name a range → `0x03`. Quantity is validated before the address, as the Modbus spec
sequences it; `tests/modbus_exceptions.rs` pins that precedence, which keeps the test valid as the
map grows in ticket 03.

**Capture.** One flat NDJSON stream, `schema_version` on every record. `tests/modbus_capture.rs`
pins the wire format, including that `exception_code` is absent rather than null on success so `jq`
filters read cleanly. The raw request PDU is base64-inlined on every request event — for function
codes sunnypot does not parse, it is the only evidence of what was attempted.

**Resource limits** (`tests/modbus_limits.rs`). The surface caps concurrent connections and closes a
peer that holds a socket without completing a request; both are config, defaulting to 256 and 120
seconds. A refused connection is still recorded, with `error: "at the connection limit"` — a peer
exhausting the limit is itself an observation. Without these, a peer that opens sockets and sends
seven bytes each walks the process to fd exhaustion, which `CLAUDE.md` puts in the default path
rather than in a warning.

**Config.** `sunnypot.toml`, gitignored, with `sunnypot.example.toml` committed. Unknown keys are
rejected rather than ignored. Identity strings are checked against their SunSpec field widths at
load, and empty ones are refused: a string too long would be truncated on the wire and an empty one
would serve an all-zero Common block — both invisible locally and obvious to a scanner.

## Comments

**Rust was not installed in this environment.** Installed via rustup (1.98.0, `stable`). The repo had
no `Cargo.toml` before this ticket; the crate skeleton, `.gitignore`, and dependency choices land
here.

**Dependencies.** tokio, serde/serde_json/toml, chrono (default features off, to keep the wasm and
Windows trees out of a static build), uuid, base64, thiserror. `tokio-modbus` is a **dev**-dependency
only — it never enters the shipped binary.

**The identity field is `product`, not `model`.** `CONTEXT.md` reserves *Model* for a SunSpec
register-block definition "and nothing else", and ADR 0002's device table calls this field `Product`.
It still serves SunSpec `Md`.

**The example config carries the ADR's identity, not blanks.** The manufacturer, product, options and
version are the same for every deployment and are public in ADR 0002, so a fresh checkout serves a
coherent device rather than an all-zero Common block. Only the deployment-specific values — serial,
latitude, longitude, timezone — are placeholders, and they are marked `FILL IN`. This is a slightly
looser reading of "documenting shape but not values" than the checkbox implies; the alternative left
the ticket 01 decision reachable only from ADR prose, which both reviews flagged.

**Register addresses are 0-based on the wire.** The ADR tabulates the identifier at 40001, as vendor
register maps do; that is the 1-based reference for protocol address 40000, which is what standard
SunSpec discovery probes. Worth confirming against a real Fronius banner in ticket 11, since a
one-register offset would break every client.

**Sunnypot answers any unit id and echoes it back**, where the ADR documents unit ID 1. Refusing other
unit ids would turn a scanner's unit-id sweep into silence, and engagement is the point. This is a
knowing deviation from the documented device, so it is recorded in ADR 0002's Consequences rather
than only here.

**Non-zero MBAP protocol ids are answered, not refused.** Strictly, protocol id 0 means Modbus.
Sunnypot echoes whatever it is given and answers anyway, on the same keep-them-talking reasoning.
Not asserted by any test; revisit if it ever matters.

**The binary writes captures to stdout.** `StdoutSink` is what makes the binary runnable now and is
not in the spec's two-implementation list; ticket 09 replaces it with the object-storage uploader
behind the same `CaptureSink` trait.

**Models 120, 121 and 122 were unowned.** Ticket 05's control addresses are only correct if those
blocks sit between Model 101 and Model 123, and no ticket built them — they are worth 106 registers,
which is exactly the offset between the ADR's 40228 and where Immediate Controls would otherwise
land. Now ticket 03a, which 05 is blocked by. Model 160 sits after 123 and shifts nothing that 05
touches, so it rides along in 03a rather than being load-bearing.
