# 06 — HTTP login and status dashboard

**What to build:** A vendor-plausible web interface. Real-world default credentials get in, everything
else is turned away, and behind the login a dashboard shows the same device state the Modbus surface
reports. Every attempt is captured, whether it succeeded or not.

**Blocked by:** 02, 03.

**Status:** ready-for-agent

- [ ] The login page is served with an appearance and `Server` header consistent with the chosen identity
- [ ] The configured default credential set is accepted; all other credentials are rejected
- [ ] The dashboard renders live plant state and does not contradict what the Modbus surface reports at the same instant
- [ ] Every request is captured with method, path, headers, and body
- [ ] Every credential attempt is captured, and successful logins are distinguishable from rejected ones
- [ ] Bodies are base64-inlined up to the size cap, with a truncation flag set beyond it
- [ ] Pages are rebuilt to look plausible, not copied from vendor firmware
