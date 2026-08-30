# 08 — Firmware upload capture

**What to build:** A firmware update page that accepts a file, reports plausible success, and stores
the upload untouched. This is the highest-value capture in the project — and the one place where the
low-interaction boundary matters most, so nothing about the file is ever interpreted.

**Blocked by:** 06.

**Status:** ready-for-agent

- [ ] The upload page accepts a file and reports plausible success to the uploader
- [ ] The file is stored whole as a capture artifact
- [ ] The file is never unpacked, parsed, decompressed, executed, or inspected beyond recording its size and declared type
- [ ] Uploads beyond the size cap are recorded as truncated rather than dropped silently
- [ ] The upload is captured with filename, declared content type, size, and a hash
