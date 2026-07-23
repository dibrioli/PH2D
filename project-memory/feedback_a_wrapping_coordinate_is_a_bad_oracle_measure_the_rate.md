---
name: feedback_a_wrapping_coordinate_is_a_bad_oracle_measure_the_rate
description: When the measured quantity can exceed a coordinate's wrap period (angle mod 2pi), the wrapped value is noise; assert on the unwrapped RATE, not the accumulated coordinate
metadata:
  type: feedback
---

A coordinate that **wraps** — an angle stored mod 2π, a phase, a looping time — is a
useless oracle the moment the thing you are measuring can exceed one wrap period. Two runs
that differ by exactly one revolution read as *identical*; a run that turned 14 revolutions
reads as whatever fraction is left over. The gate can pass on a coincidental leftover and
fail on a correct-but-large result, and neither says anything true.

**Concrete (physics AreaTorque, W-AreaTorque 2026-07-22):** a torque zone spins a body, and
the readback writes `Transform.rotation`, which wraps at ±π. A strong torque spins a light
box **many** revolutions in the test window, so the wrapped rotation was noise — measured a
*compact* box at 2.688 rad and a *long bar* (8× the moment of inertia, so it should barely
move) at **−1.254 rad**: negative, larger in magnitude, meaningless as a spin rate. The
world-level gates were correct because they read the raw `angvel` (angular VELOCITY, never
wrapped) and could push hard; the ECS/gesture/smoke gates read `rotation` and had to keep the
spin **sub-revolution** (a modest torque on a body big enough that it turns a fraction of a
turn) for `> threshold` to mean "it turned" rather than "it wrapped to a lucky value".

**Why:** the gate's oracle must model the OBSERVABLE, and a wrapped coordinate is not the
observable — the *unwrapped rate* is (how fast it spins, how far it travelled per unit time).
The trap hides because for the COMMON small case (a gentle spin, one revolution or less) the
wrapped coordinate and the true angle coincide; it only fails once the quantity is large, which
a weak-torque / heavy-body fixture never reaches.

**How to apply:** when the oracle reads a coordinate that can wrap (angle, phase, mod-N
counter, looping clock), either (a) assert on the unwrapped RATE if you have access to it
(here, `angvel`), or (b) size the fixture so the quantity stays within one wrap period and the
sub-period value is monotone. Never assert `wrapped_value > k` when the true quantity can be
`k + 2π·n`. Measure before you pick the fixture: run a probe, and if the number is bigger than
the wrap period, the coordinate is the wrong axis.

Sibling of [[reference_topic_oracle_discipline]] (the oracle must model appearance/observable,
not a convenient internal number) and [[reference_topic_fixture_discipline]] (a fixture only
proves what it contains — a weak-torque fixture never reaches the wrap and hides the flaw).
