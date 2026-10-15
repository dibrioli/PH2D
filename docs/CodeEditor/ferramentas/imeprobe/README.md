# imeprobe — o que o winit 0.30.13 entrega para tecla morta (´ + a)

Sonda da T6 (Onda 0, 15/10/2026). Corre fora do workspace:
`cargo run --manifest-path docs/CodeEditor/ferramentas/imeprobe/Cargo.toml` num Xvfb, com `xdotool` a mandar as teclas
(layout `us intl`). Logs medidos em `medicoes/`: `x11.log` (XIM por omissão), `x11b.log` e `xkb.log` (sem método de
entrada: a composição do próprio winit, a mesma função que ele corre no Wayland). A tabela e a conclusão estão no
comentário de `crates/ph2d-editor-core/src/ime.rs`.
