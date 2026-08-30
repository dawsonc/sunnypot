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
doubles, exactly as the spec allows. All 15 tests drive real TCP over loopback and assert on returned
bytes or on captured events.

**Register map.** `src/modbus/registers.rs`, typed and in code. `SunS` at protocol address 40000,
then Common (id 1, length **65** — the Fronius length, not the textbook 66), then the `0xFFFF`
terminator at 40069. `tests/sunspec_discovery.rs` walks the chain with `tokio-modbus`, a third-party
client, rather than with our own framing code: the question that test answers is whether standard
discovery logic finds the chain without special-casing, which our own encoder could not honestly
answer. The walk lands exactly on the terminator, so no model is misaligned.

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

**Config.** `sunnypot.toml`, gitignored, with `sunnypot.example.toml` committed. Unknown keys are
rejected rather than ignored, so a typo is not a silently-ignored setting. Identity strings are
checked against their SunSpec field widths at load: a string too long would be truncated on the
wire, which is invisible locally and obvious to a scanner.

## Comments

**Rust was not installed in this environment.** Installed via rustup (1.98.0, `stable`). The repo had
no `Cargo.toml` before this ticket; the crate skeleton, `.gitignore`, and dependency choices land
here.

**Dependencies.** tokio, serde/serde_json/toml, chrono (default features off, to keep the wasm and
Windows trees out of a static build), uuid, base64, thiserror. `tokio-modbus` is a **dev**-dependency
only — it never enters the shipped binary.

**Register addresses are 0-based on the wire.** The ADR tabulates the identifier at 40001, as vendor
register maps do; that is the 1-based reference for protocol address 40000, which is what standard
SunSpec discovery probes. `src/modbus/registers.rs` documents the mapping. Worth confirming against a
real Fronius banner in ticket 11, since a one-register offset would break every client.

**Sunnypot answers any unit id and echoes it back.** A real Datamanager is addressed as unit 1
(`DA = 1`), but refusing other unit ids would turn a scanner's unit-id sweep into silence. Answering
maximises engagement, which is the point. Flagged here rather than hidden: if ticket 11 finds this
reads as non-Fronius, restrict it.

**Non-zero MBAP protocol ids are answered, not refused.** Strictly, protocol id 0 means Modbus.
Sunnypot echoes whatever it is given and answers anyway, on the same keep-them-talking reasoning.
Not asserted by any test; revisit if it ever matters.

**The binary writes captures to stdout.** `StdoutSink` is what makes the binary runnable now;
ticket 09 replaces it with the object-storage uploader behind the same `CaptureSink` trait.

**The example config carries empty identity strings**, per this ticket's "shape but not values".
A deployment that never fills them in would serve an all-zero Common block — config load does not
reject that, since any non-empty placeholder that passed a check would defeat it. Ticket 11's
external fingerprint check is what catches an unfilled config.
