# caneta — a sonda e as medições de 17/10 (Wacom Intuos5 touch S, PTH-450)

Máquina do dono: KWin 6.7.5 Wayland · libinput 1.32.0 · kernel 6.18 (`wacom`) · winit 0.30.13.

- `penprobe/` — janela winit 0.30.13 + softbuffer que só REGISTA `CursorMoved`/`MouseInput`/`Touch`/entrar/sair.
  Fora do workspace: `CARGO_TARGET_DIR=/tmp/penprobe bash scripts/ph2d-run.sh cargo build --manifest-path docs/Painter/ferramentas/caneta/penprobe/Cargo.toml`.
  Nativo: `…/penprobe > log`; XWayland: `env -u WAYLAND_DISPLAY …/penprobe > log`.
- `medicoes_2026-10-17/penprobe_wayland_nativo.log` — caneta por cima e a riscar (18–33 s): só `CursorEntered/Left`,
  **zero** `CursorMoved`, **zero** `MouseInput`; depois o rato (33–37 s) chega inteiro. ⇒ o KWin NÃO emula ponteiro
  para cliente sem `zwp_tablet_manager_v2`.
- `medicoes_2026-10-17/penprobe_xwayland.log` — só caneta: movimentos + 5 pares `Left Pressed/Released`; nenhuma
  pressão (o winit 0.30 não a tem em X11).
- `medicoes_2026-10-17/libinput_debug_events_event14.txt.zst` — `sudo libinput debug-events --device /dev/input/event14`,
  180 s, o dono a riscar com força variada: 210 `TABLET_TOOL_TIP`, 9 890 `TABLET_TOOL_AXIS`, 14 proximidades,
  16 botões. A pressão normalizada vai de `0.00` a `0.97` (o máximo, 15 linhas; não chega a `1.00`). É o ORÁCULO
  do gate `crates/ph2d-pen/tests/oraculo_libinput.rs`, linha a linha (amostra, ponta, botão, saída, pressão).
- `PH2D_PEN_LOG=<ficheiro>` na app: uma linha por evento da caneta (`ms ferramenta fase x/y pressure: … tilt: …`),
  para pôr ao lado de um `libinput debug-events` gravado ao mesmo tempo.
- Medido à parte (`evtest`, ficheiro não guardado: 5 MB): `ABS_PRESSURE` 0..2047, `BTN_TOUCH` alterna, `BTN_STYLUS`
  e `BTN_STYLUS2` respondem, `ABS_TILT_X/Y` e `ABS_DISTANCE` presentes.
