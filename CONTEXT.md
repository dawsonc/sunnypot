# Sunnypot

A honeypot impersonating a grid-edge energy device, exposed to the public internet to observe
what attacks distributed energy resources actually attract.

## Language

**DER** (distributed energy resource):
A consumer- or building-scale device that generates, stores, or controls electrical power —
solar inverters, batteries, EV chargers. Sunnypot concerns itself only with the subset that
accepts *inbound* connections, since the rest are unreachable from the public internet.

**PV inverter**:
A grid-tied inverter converting DC from a rooftop array into AC, with no attached storage. The
device class sunnypot impersonates, chosen for the size of its exposed population.
_Avoid_: Hybrid inverter (has storage; a different device identity, out of scope for v1),
solar panel, PV system

**Curtailment**:
Deliberate reduction of an inverter's real power output below what the array could produce.
The most consequential thing an attacker can do to a PV inverter over Modbus, and the reason
the plant must respond to control writes rather than merely report.

**Low-interaction**:
The property that sunnypot never executes or interprets what an attacker sends. Realism comes
from emulated responses, never from running real firmware. A hard boundary, not a v1 shortcut.
_Avoid_: Fake, simulated (both blur into the plant's dynamic state, which is a separate axis)

**Plant**:
The simulated physical device — solar position, irradiance, generation — evolving over time and
responding to control writes. The plant holds the truth; protocol surfaces only expose it.
_Avoid_: Model (reserved for SunSpec Model), sim, digital twin

**Model**:
A SunSpec register-block definition and nothing else (Model 1 common, 101/103 inverter,
123 immediate controls). Never used for the plant or for any internal data structure.

**Surface**:
One protocol endpoint through which the plant is exposed and observed — the Modbus surface, the
HTTP surface. A surface serves values; it never owns them.
