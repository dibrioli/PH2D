---
name: project_wacom_pen_needs_tablet_v2_on_kwin_wayland
description: Intuos5 pen does nothing in PH2D on KWin 6.7 Wayland (no pointer emulation for clients without tablet-v2); XWayland works; pressure needs a tablet-v2 path
metadata:
  node_type: memory
  type: project
  originSessionId: 5fae2399-6b2b-43a0-9430-fa400cd49c9f
  modified: 2026-10-18T00:17:17.671Z
---

MEASURED 2026-10-17 (Intuos5 touch S, PTH-450, KWin 6.7.5, libinput 1.32, winit 0.30.13): the kernel and
libinput deliver the pen perfectly (evtest: ABS_PRESSURE 0..2047, BTN_TOUCH toggles; libinput
debug-events: TABLET_TOOL_TIP down/up). Nothing has to be installed: the `wacom` kernel module + libwacom
are enough; `xf86-input-wacom` is X11-only and useless on Wayland.

**But KWin does NOT emulate a pointer for a Wayland client that does not bind `zwp_tablet_manager_v2`.**
A bare winit 0.30.13 probe on native Wayland got only `CursorEntered/Left`: zero `CursorMoved`, zero
`MouseInput`. The cursor is drawn moving, so it *looks* like it works. winit 0.30 has no tablet code
at all, so PH2D on Wayland ignores the pen. The same probe with `env -u WAYLAND_DISPLAY` (XWayland)
got moves + Left Pressed/Released for every tip touch, but no pressure.

Plasma's own panel also ignores the pen (QtWayland logs `Can't send tablet event with no proximity
surface`); Dolphin works. That is a Plasma/Qt defect, not ours.

**Why:** this refutes "the pen works as a mouse, only the pressure is missing" (see
`pointer_dynamics` in `shells/desktop/src/vec_app_bridge.rs`). On Wayland the pen gives nothing.

**How to apply:** the stopgap is to launch with `env -u WAYLAND_DISPLAY`. The real fix (pen + pressure +
tilt) is a tablet-v2 path in the shell (e.g. the `octotablet` crate beside winit, or a newer winit with
tablet events). The owner ordered it on 17/10 as prompt 46 of `line/PainterWatercolor`
(`docs/Painter/NewFeaturesWetPaint/46_PROMPT_a_caneta_de_verdade.md`). The probe and the recordings live in
`docs/Painter/ferramentas/caneta/`. Related: [[reference_topic_installed_apps]].
