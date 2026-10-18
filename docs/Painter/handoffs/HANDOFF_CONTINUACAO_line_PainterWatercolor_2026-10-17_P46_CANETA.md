# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 46: a caneta de verdade (2026-10-17)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 46](../NewFeaturesWetPaint/46_PROMPT_a_caneta_de_verdade.md)
· [ADR-0189](../../architecture/decisions/0189-a-caneta-chega-pelo-tablet-v2-ao-lado-do-winit.md) (número PROVISÓRIO — o
integrador reconta) · [medições](../ferramentas/caneta/README.md) · lista [26](../NewFeaturesWetPaint/26_ABERTOS.md) C29–C32.
Anterior: [P45](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P45_ESMAECER.md) (+ o Randomize Color com a Shape
Color, §4b). Commits `9c9953a7c1` → `c7262d87af` + os docs.

## §2 — Decidido (ADR-0189), medido e não lido

- **(b) o `zwp_tablet_v2` à mão**, na ligação do winit (o padrão do `ph2d-wayland-dnd`), zero crates novas, bind
  CAPADO na versão 1. Recusados: `octotablet` 0.1.0 (liga a v2 do servidor com código v1 → o libwayland mata a
  ligação NESTA máquina) e o winit 0.31 beta (perde o tempo de cada amostra, tilt em graus inteiros, sem X11, API
  nova em 3 crates). A pressão e a borracha/tilt: `PointerEvent::pressure` + `PointerSource::Pen(PenInfo)` (o
  `Pencil` não tinha leitor; um campo `tilt` seriam 180 construções).
- **Sonda ao vivo** (sem a caneta na mão): 20 s no KWin do dono, sem erro de protocolo; ligou o manager v1 e o
  `wp_cursor_shape_manager_v1`; o KWin anunciou a mesa, o pad e a ferramenta (pressão, distância, tilt).

## §3 — O que existe

- **`ph2d-pen`**: `sessao` (pura: eventos do protocolo → `PenEvent`, UM por `frame`, com o tempo; a pressão persiste
  entre frames; toque dentro de um frame = Down+Up) · `wayland` (o fio, a superfície, o cursor por cursor-shape,
  `PH2D_PEN_LOG=<ficheiro>` regista cada evento) · `ponteiro` (o `Conversor`: a caneta como o rato a veria — ponta
  = Primary, botão de baixo = Middle, de cima = Secondary, hover = Move pressão 0; `do_rato` = a lei do rato, 1.0 e
  `Mouse`) · `porta` (o estado do receptor fora da `App`, que está no tecto de campos).
- **Shell**: `on_cursor_moved`/`on_mouse_input` são pontes para `pointer_moved`/`pointer_button`; a caneta
  (`input_drop/caneta.rs`) entra pelas MESMAS duas portas (gate `the_mouse_and_the_pen_end_in_the_same_pointer_doors`),
  logo conduz a UI inteira. A fila drena-se no TOPO do quadro (antes do Move coalescido do Painter). O cursor da
  caneta espelha o do rato. A shell ficou 31 linhas MENOR (movidos `canvas_down_accepts` → `ph2d-app-painter`,
  ajudantes do gradiente → `ph2d-app-vec`).
- **Consumidores**: Painter — Move, Move coalescido e o tilt (`CanvasPointer`, `[-1, 1]`); o `Up` fecha com a pressão
  do último contacto (a caneta levanta a 0 e as figuras re-carimbam no Up); a ponta de BORRACHA liga a borracha só
  nesse traço (`toggle_brush_eraser`, a porta que a macro grava). Flip — `flip_canvas_down/move(x, y, pressure)` e o
  sculpt. Vector — `pointer_dynamics` com a pressão e o carimbo do evento.
- **Calibrações contra a pressão real do dono** (180 s gravados: no contacto p50 0,59, p5 0,24, p95 0,86; < 0,2 só
  3 %): Flip `Response` 0,5 → **0,65** (o traço típico a 0,48 do Size em vez de 0,61), `Min Width` fica 0,05. Vector,
  fonte Pressure, passa a ABSOLUTA `0,35 + 1,10·p` (antes esticava a faixa de cada traço: a força deitava-se fora; a
  mediana 0,59 dá 0,999 = a largura que o Width promete).
- **Macro**: o `Passo::Ponteiro` já gravava a pressão/tilt resolvidos; gate novo — uma rampa gravada toca, ao bit, a
  tela pintada ao vivo em Digital, Watercolor, Impasto e Wet Paint (o Composite passa pelas rotas do Digital).

## §4 — Portões

`ph2d-pen` 28 (fixtures da sessão e do conversor; **oráculo libinput**: os 180 s repetidos linha a linha —
10 107 amostras, 105 Down/105 Up nas linhas TIP, 16 botões, 7 saídas, pressão a 1/65535, e pelo conversor cada toque
= um clique primário) · `caneta_tests` da shell · Flip `the_factory_response_puts_the_owners_median_pressure_at_half_the_size`
· Vector `pressure_is_absolute_so_a_firm_steady_stroke_stays_firm` · a macro nos 4 meios. **Mutação 8/8 vermelhas**
(a ponta a virar Secondary no conversor e na shell; o rato a 0,5; o rato a herdar a pressão da caneta; a porta que
esquece a tinta; o Move do Painter fixo em 1.0; a macro a gravar 1.0; o lápis normalizado de volta; o Response 0,5).
`the_desktop_shell_has_no_pen_pressure.rs` ficou vermelho e foi APAGADO depois da calibração; as notas «sem pressão»
corrigidas (vec_app_bridge, pencil_width, i18n, painter_canvas_input, reshape, docs Vector 25 e Painter 37/38).

## §5 — Fecho

fmt ✓ · clippy `-D warnings --all-targets` nas 13 crates ✓ · `nextest-impacted` (base `92b42ffa03`, cobre também o
Randomize) **19 083/19 093**: 7 defeitos desta onda, curados (`70e5f9fc72`: um comentário citava o cabeçalho GPL do
kernel; `c7262d87af`: seis gates da shell liam o corpo de `on_mouse_input`, que passou às portas — apontados às portas
+ três que passavam por lerem até ao fim do ficheiro) e refeitos por nome ✓; 3 de carga verdes sozinhos 3/3
(`the_cost_of_a_player_is_linear_in_their_number`, `the_mask_stroke_cost_does_not_follow_the_canvas`,
`g4_o_interior_de_um_opaco_nao_custa`).

## §6 — Aberto (só a caneta na mão prova)

**Smoke do dono 17/10: OK** — a Wacom (P46), salvar com a Shape Color e o Airbrush (P45), o Randomize Color com a
Shape Color (§4b do P45). Pedidos novos na fila: lista 26 A13–A15 (apagar, repor, a prévia em todos os modos).


- O smoke do dono: clique em menu/cartão/aba com a ponta; a pressão em cada meio, no lápis e no Flip; a borracha; os
  botões laterais; o cursor; os tempos reais do compositor (`PH2D_PEN_LOG` × `libinput debug-events` da mesma
  sessão — também responde à C31).
- Sem gate (precisam de janela): a pressão no Move coalescido e no Up, o tilt e a borracha a chegarem ao Painter pela
  shell.
- Em X11/XWayland a caneta continua rato sem pressão (C32). Fora do prompt: C29 (pad), C30 (dedo), C31 (curva do KDE).
