# 09 — Object storage sink

**What to build:** Captures leave the device continuously and land in remote storage the honeypot
cannot read back or alter. A network outage delays delivery; it never loses data. Runs in parallel
with the surface work — it needs only the sink interface.

**Blocked by:** 02.

**Status:** ready-for-agent

- [ ] The sink interface has an in-memory implementation for tests and an S3-compatible implementation for production
- [ ] Events are batched and gzipped, and uploaded on a size or time trigger
- [ ] Captures buffer locally through a simulated outage and deliver once it clears
- [ ] Object keys are unique, so no upload ever overwrites an existing object
- [ ] The endpoint is configurable, so R2, S3, and B2 are interchangeable without code changes
- [ ] The implementation requires only object-put permission — it never lists, reads, or deletes
