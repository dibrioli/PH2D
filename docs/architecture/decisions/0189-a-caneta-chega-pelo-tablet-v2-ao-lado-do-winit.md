# ADR-0189 — A caneta chega pelo `zwp_tablet_v2` ao lado do winit, numa crate nossa

- **Status:** Accepted — o produto (a caneta conduz a UI inteira, a pressão chega a todos os meios) é ordem do
  DONO (prompt 46, 17/10); o desenho técnico é delegado.
- **Data:** 2026-10-17
- **Linha:** `line/PainterWatercolor` (prompt 46)
- **Amparo:** [prompt 46](../../Painter/NewFeaturesWetPaint/46_PROMPT_a_caneta_de_verdade.md) ·
  [medições de 17/10](../../Painter/ferramentas/caneta/README.md) · o precedente `ph2d-wayland-dnd` (MiroClone W14).

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 17/10 contra o máximo do `main` e de todas as worktrees (`0188`).
> Quem integrar reconta-o.

## Contexto — medido na máquina do dono

Wacom Intuos5 touch S, KWin 6.7.5 Wayland, winit 0.30.13. O kernel e o libinput dão pressão, tilt, ponta e botões;
um cliente Qt com tablet-v2 clica; **o winit 0.30 em Wayland nativo não recebe NADA da caneta** (só entrar/sair:
o KWin não emula ponteiro para quem não liga `zwp_tablet_manager_v2`); em XWayland clica sem pressão. O KWin
anuncia o `zwp_tablet_manager_v2` na **versão 2**. A pressão morre na shell: `input_dispatch.rs` crava `1.0`.

## Decisão

1. **Ligamos o `zwp_tablet_v2` nós mesmos** (`wayland-client` 0.31 + `wayland-protocols` 0.32 `wp::tablet::zv2`,
   estável, sem feature; `wayland-backend` `client_system` — os três já no lock pelo winit: **zero crates
   novas**), na MESMA ligação do winit (`RawDisplayHandle::Wayland` → `Backend::from_foreign_display`), com fila
   e fio próprios — o padrão do `ph2d-wayland-dnd`. **O bind é capado na versão 1** (`1..=1`): ligar a versão do
   servidor (2) com código gerado que não a conhece mata a ligação inteira (`bustype`, o crash do octotablet).
2. **A crate `ph2d-pen`** (drop-crate; a shell só compõe): `sessao` — a máquina pura que transforma os eventos do
   protocolo (proximidade, ponta, movimento, pressão `0..65535`, distância, tilt, botões, `frame(time)`) em
   `PenEvent`s, UM por `frame`, sem coalescer, com o tempo em ms — testável sem compositor; `wayland` (Linux) — o
   fio que fala com o compositor, filtra a superfície da janela e põe o cursor da ferramenta. X11 e Windows
   entram depois pela MESMA `sessao` (fontes novas, mesma saída).
3. **A porta na shell**: a fila drena-se no TOPO de cada quadro — antes do carimbo coalescido do Painter, onde os
   eventos do winit já chegaram (no ponto do receptor de «soltar» o Move coalescido esperaria um quadro) — e cada
   `PenEvent`, convertido por `ph2d_pen::ponteiro`, entra pelas MESMAS portas do rato (`pointer_moved`/
   `pointer_button`, as que o `on_cursor_moved`/`on_mouse_input` do winit também chamam) — logo a caneta conduz a
   UI inteira. A ponta é o botão primário; o botão de baixo (`BTN_STYLUS`) é o do meio, o de cima (`BTN_STYLUS2`) o
   secundário (a convenção do GNOME/Krita). Coordenadas lógicas × o factor de escala.
4. **A pressão viaja no `PointerEvent::pressure`; a inclinação e a ponta de borracha viajam na FONTE**:
   `PointerSource::Pencil` (sem leitor) passa a `Pen(PenInfo { tilt_cdeg: [i16; 2], eraser: bool })`. Nenhuma
   lei de pincel lê o tilt hoje: ele chega ao `CanvasPointer::tilt` do Painter (que a macro já grava) e o resto
   espera um leitor. O rato continua `pressure 1.0`, `PointerSource::Mouse`, ao byte.

## Alternativas recusadas (o preço)

| alternativa | medido / lido | porquê não |
|---|---|---|
| `octotablet` 0.1.0 (MIT) | liga o manager na versão do servidor (2): o KWin 6.7 manda `bustype` e o libwayland mata a ligação (`zwp_tablet_v2 has no event 5`; issue #18 upstream). A cura só no `master`, ainda sem tecto de versão nem dials; X11 por fundir; Windows com 3 defeitos abertos | derruba o app NESTA máquina com ESTA mesa |
| winit 0.31.0-beta.3 | tem caneta em Wayland/Windows, mas deita fora o tempo de cada `frame`, trunca o tilt a graus `i8`, sem distância nem ferramenta; `BTN_STYLUS` colide com a ponta; X11 nada; a API muda (`Window` vira trait, `Touch` sai) em 3 crates | beta, perde o que o traço precisa (todas as amostras com tempo) e é a fundação de janela de todo módulo |
| um campo `tilt` no `PointerEvent` | 180 construções em 10+ crates | colisão entre linhas por um dado sem leitor; a fonte é o sítio dele |

## Consequências

- O gate `the_desktop_shell_has_no_pen_pressure.rs` fica vermelho de propósito: re-calibrar o `Min Width`/
  `Response` do Flip contra pressão real e apagá-lo.
- Fora deste ADR (lista 26): os botões e o anel da MESA (`Pad`), o toque com dedo, e a curva de pressão própria
  (o KDE já aplica a dele).
