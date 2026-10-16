# 26 — Plano F16: a ② vira a *Control bar* do Illustrator (foco no traço)

> Pedido do dono (15/10): *«criar a barra de ajustes no topo como no illustrator, principalmente com os ajustes do
> stroke. Primeiro investigue e depois planeje»* + *«Os botões que estão presentes na barra do topo devem sair»*.
> Referência: [`R_illustrator_barra_de_controle_2026-10-15.png`](referencias/R_illustrator_barra_de_controle_2026-10-15.png).
> Estado: **PLANO, sem código** — vai ao dono; as decisões dele entram na §5 antes da 1.ª linha.

## 1 — O que o código já tem (verificado 15/10, base `902f25d327`)

**A barra.** Não é barra nova: é a ② (`ToolBars.row_lead/row_tail`, `ph2d-editor-core/src/interaction/tool_bars.rs:76`),
pintada por `paint_tool_bar` (`screens/hero/tool_bar.rs:245`), composta por `bar_rail` (`tool_bar.rs:180`):
`row_lead` (Vetor) → `rail_entries` (GENÉRICO, `left_rail.rs:462`: gizmo, SPACE, VIEW, UNDO/REDO, Mode) → `row_tail` →
pulldowns de área (`AreaMenu`, Snap e Path). Overflow `bar_split` (`tool_bar.rs:86`) → `⋯`; a ordem é a prioridade.
Itens possíveis (`ToolRailEntry`, `widget/tool_rail.rs:90`): `Icon` · `Compound` (cicla) · `Swatch` · `Glyph` · `Number`
(rótulo + `display`) · `Divider`. **Não há** item-texto (nome do objeto), item-dropdown com lista, nem popover genérico
ancorado a um chip; o mais perto é o `AreaMenu` (`types_menu.rs:91`, linhas toggle/compound).
Orçamento: a fila pede 1051 px em Edit+Node; iPad 12,9" dá 730 px (comentário `tool_bar.rs:189`; o número não está no
doc de medições — **re-medir na F16a**).

**Os botões que saem** (ids em `ids/chrome/rail.rs`):

| Botão | Outra porta hoje |
|---|---|
| MOVE/ROT/SCALE/PIVOT (`TOOL_TRANSLATE..PIVOT`) | dica diz G/R/S — **tecla não confirmada**; no Vetor a caixa do Select já move/roda/escala |
| SPACE Global/Local (`TOOL_SPACE`) | só a ② e a paleta de comandos |
| VIEW (`TOOL_HOME`) | menu View (`MENUBAR_VIEW_FRAME_ALL/SELECTED`) |
| UNDO/REDO (`TOOL_UNDO/REDO`) | menu Edit + Ctrl+Z |
| Edit Mode (`ObjectMode::row_id`) | Tab (`ph2d-i18n/src/object_mode.rs`) |
| Snap ✓ (`AreaMenu`) | painel do Vetor (`populate_ops.rs:179`); a barra de estado mostra «snap on» (só leitura) |
| Path (`AreaMenu`) | os verbos `VECTOR_PATH_*` já estão na ④ e na coluna Node |

⚠️ `rail_entries` é comum a todos os layouts — tirar no Vetor ≠ tirar em todos (pergunta P1).

**O traço** (`ph2d-vec-scene/src/stroke_style.rs:238`, `StrokeSpec`): `paint` Solid/Pattern/Brush · `width` · `cap` ·
`join` · `dash: Option<(f64,f64)>` (UM par, alinhado aos cantos por `dash_fit.rs`) · `marker_start/end` (8 formas) ·
`marker_scale`/`marker_round` (UM valor para as duas pontas) · `align` Centre/Inner/Outer. `MITER_LIMIT = 4.0` global
(`:302`). Pilha de pinturas `VecPath.paints` (vários traços/fills, cada um com opacidade e blend). **Largura viva**:
`VecStrokeProfile` (ADR-0148, `ph2d-app-vec/src/profile_live.rs`) com `PRESETS` Uniform/Taper/Both/Bulge
(`ph2d-stroke-width/src/lib.rs:329`), hoje na secção Expand do painel (`paint_expand.rs:204`). Pincel: `BrushStroke`
(`paint_brush.rs`). **Ausentes**: Graphic Style, dash com 2–3 pares, escala por ponta, limite de mitra por traço.
Painel: `paint_sections_stroke.rs` (`stroke_style`:14, `stroke_ends_and_dashes`:175), `paint_markers.rs`; ids em
`ph2d-tool-vector/src/ids/vector.rs:68-139`. Cor da ②: `register_picker_swatch` → `INSP_BLENDER_PICKER`.

## 2 — A barra proposta (esquerda → direita, só no Vetor)

```
Retângulo │ ■▾ │ ▢▾ │ Stroke: [ 3 ⇕ ] │ ━━ Uniform ▾ │ ━━ Basic ▾ │ Opacity: [100 %] │ (params da forma/ferramenta) │ ⋯
```

| Peça | O que é | Lê/escreve |
|---|---|---|
| Nome | o nome da forma; «3 objects» com vários; o nome da ferramenta sem seleção | Hierarchy (só leitura) |
| ■▾ ▢▾ | amostras Fill/Stroke (já existem) | picker partilhado (já existe) |
| **Stroke:** (sublinhado) | abre o **painel do traço** por baixo da barra (§3) | — |
| peso | o `W` de hoje, com ⇕ | `StrokeSpec.width` |
| Perfil ▾ | Uniform/Taper/Both/Bulge com o desenho de cada um | `VecStrokeProfile` (já vivo) |
| Pincel ▾ | Basic (sólido) · Pattern · Brush | `StrokeSpec.paint` |
| **Opacity:** (sublinhado) | campo; o rótulo abre o modo de mistura | opacidade/blend do objeto |
| params | Corner round, lados, gesto do Pencil, simetria — os AJUSTES ficam, depois do grupo do traço | como hoje |

Sem seleção, a barra mostra e edita o estilo da **próxima** forma (como o Illustrator). Com seleção, edita as
selecionadas — **um** passo de undo por gesto.

## 3 — O painel do traço (o *Stroke panel* que o Illustrator abre do «Stroke:»)

Solto por baixo da barra, não bloqueia o resto, fecha com clique fora (D1: o solto continua solto por declaração).
Weight · Cap (3 ícones) · Corner (3 ícones) · Align Stroke (3 ícones) · Dashed line ☐ + dash/gap **×3** · Align dashes
to corners · Arrowheads início/fim ▾ + ⇄ + **escala por ponta** · Profile ▾ + inverter.
**Limite de mitra fica fora**: o 4 é decisão MEDIDA (doc 19, `⛔ Recusas MEDIDAS`), não se reabre.
Técnico: precisa de um tipo de popover ancorado a um chip da ② (o `AreaMenu` só tem toggles); e o modelo ganha
dash ×3 e escala por ponta (formato gravado ⇒ versão + migração; `ph2d-vec-*` não é contrato congelado).

## 4 — Ondas

| Onda | Conteúdo | Gates a mexer/criar |
|---|---|---|
| F16a | tirar os botões (no âmbito que o dono escolher), achar casa para Snap; nome · amostras · peso · Opacity; re-medir 730 px | `the_snap_pulldown_holds_the_four_switches`, `the_tool_bar_is_one_line_on_every_tablet`, `the_longest_vector_rows_fit…`, `the_palette_reaches_the_vector_bars` |
| F16b | o popover ancorado + o painel do traço com tudo o que JÁ existe | `every_bar_button_reaches_the_panel_door`, novo «o popover do traço escreve nas selecionadas, 1 undo» |
| F16c | Perfil ▾ e Pincel ▾ na barra; dash ×3 e escala por ponta no modelo | formato: migração com fixture do ficheiro antigo |
| fecho | cena de smoke 23 (fotografada), suíte/clippy `PH2D_SUITE=fecho`, CLAUDE.md §5.1 | — |

## 4b — F16c: o mapa de impacto (medido 15/10, para a janela que a fizer)

`StrokeSpec.dash: Option<(f64,f64)>` → até 3 pares, e `marker_scale` → início/fim. ~60 sítios em 15 crates.
- **Formato**: postcard POSICIONAL ⇒ bump `VEC_SCENE_SCHEMA_VERSION` (`ph2d-vec-scene/src/schema.rs:80`, hoje 22) +
  a escada do `PROJECT_SCHEMA` (`shells/desktop/src/project_schema.rs:450`, hoje 189; história em
  `project_schema_history.rs`) — a tripla do §5.0, reconta com `python3 scripts/schema-recount.py`. Receita a
  copiar: `60db072e7b` (v22, `dilate`). Sem golden de bytes do `StrokeSpec`; não é contrato congelado (§6 só pina
  `ph2d-vector-doc`/`-traits`).
- **Tracejado não é drop-in**: o par está tipado em `dash_lengths()` (`stroke_style.rs:416`), `dash_fit::dash_lengths_for`
  (`dash_fit.rs:109`, `[f64;2]`), `dash_of`/`kurbo_stroke` (`ph2d-vec-render/src/lib.rs:417,538`),
  `stroke_uniform.rs:371` (`let &[traco, vao]` + `fit`), `brush_spans` (`brush_stroke.rs:171`), `align.rs:94-102`
  (halva o dash do Inner/Outer), `expand.rs:202-267`, `svg_export.rs:182` (`stroke-dasharray`), `placa.rs:61`,
  `miter_clip.rs:30`, `instance.rs:66`. O `kurbo::Stroke::with_dashes` já aceita N valores.
- **Setas**: só `marker.rs:543-575` (`stroke_head`, que já recebe `at_start`) lê `marker_scale`/`marker_round`;
  hit-test `vec_gizmo_pick.rs:192`.
- **Tool/ponte/painel**: `params_snapshot.rs:50-110`, `tool.rs:172-391`, `params.rs:35,165,186`;
  `vector_bridge_style.rs:37-146,420`; `vector_bridge.rs:266-354`; `paint_sections_stroke.rs` (Dash/Gap),
  `paint_markers.rs:54-139`, `populate_style.rs:83`, `event.rs:266-293`. Motion grava `spec.dash`
  (`motion_shape_gen.rs:398`). Fixtures literais: `vector_bridge_tests.rs`, `vector_bridge_opacity_tests.rs`,
  `vec_selection.rs:346`, `brush_stroke_fixtures.rs:52`, `stroke_present_tests.rs:149`, `motion_shape_*_tests.rs`.

### 4c — F16c FEITA (15/10, `line/UIUX`) — o que ficou decidido

- **Modelo**: `StrokeSpec.dash: Option<DashPattern>` (`[[f64; 2]; 3]`, `ph2d-vec-scene/src/dash_pattern.rs`) e
  `marker_scale_start`/`_end`. Lei dos pares: o 1.º conta sempre (1.º traço `0` = contínuo, a lei de antes);
  os seguintes contam até ao 1.º par `[0, 0]`; um traço `0` depois do 1.º é um PONTO (com ponta redonda).
  `DashLengths` (em comprimento) é o que o traçador, o Outline Stroke, o pincel e o SVG leem; o ajuste aos
  cantos (`dash_fit::fit`) escala os N comprimentos pelo mesmo `k` e, num aberto, acaba no 1.º traço.
- **Ficheiros antigos abrem iguais** (`VEC_SCENE` `22 → 23`, `PROJECT_SCHEMA` `189 → 190`): só o
  `StrokeSpec` tem forma antiga (`stroke_wire.rs`, `StrokeSpecV22`); o leitor de um ficheiro `< 190` abre o
  âmbito `StrokeLayoutV22` (thread-local), e as formas guardadas DENTRO de componentes (fonte do Envelope e
  da pele) são regravadas no load. Gates sobre BYTES gravados pelo código v22/v189 (commit `d138b96401`):
  `stroke_v22_fixture` (vec-scene), `traco_v189` (shell), `envelope_traco_v22_tests` (app-vec) — cada um com
  o controlo «sem o âmbito, os mesmos bytes não leem igual».
- **Painel** (e o popover do «Stroke ▾», que pinta as mesmas linhas): *Dash / Gap 1..3*, uma linha por par
  com [traço][vão] — «um nome, N caixas» do Inspector (os SLIDERS do 1.º par saíram, como no Illustrator,
  que só tem caixas); *Head Size* [começo][fim]; *Head Round* na mesma coluna. `paint_dashes.rs`.
  ⛔ Duas linhas de três caixas (*Dash* [1][2][3] / *Gap* [1][2][3]) foram a 1.ª redacção: a FOTO mostrou a
  fila a partir no painel solto (a 3.ª caixa debaixo da 1.ª).
- **Lápis com farpas desenhadas**: cada ponta tem o tamanho da SUA farpa (era a média das duas).
- **SVG**: importa até 6 números do `stroke-dasharray` (a nota de perda só acima disso); exporta os pares ativos.
- ⏳ **Por fazer, nomeado**: a placa (`ph2d-shape-gpu`, `tracejado_do_eixo`) só percorre UM par; um padrão de
  2–3 pares cai na cerca que já existia (`FLAG_SO_CONFORME`: a placa desenha as cópias conformes com o
  tracejado cortado na CPU, o Vello as outras) — mesma imagem, caminho mais lento só sob escala não uniforme.
  O Motion continua com um par (o nó `ph2d-node-motion-shape` tem `DASH`/`DASH_GAP`).
- Cena de smoke **24** (`PH2D_OBJECT_MODE_SMOKE=24`, `vector_mode_tracejado.rs`).

## 5 — Perguntas ao dono (respostas entram aqui)

- **P1** Os botões saem só na tela do Vetor ou em todas? (recomendado: só no Vetor agora)
- **P2** O Snap vai para onde? (recomendado: o «snap on» da barra de baixo vira clicável e abre os quatro interruptores)
- **P3** O painel do traço solto por baixo do «Stroke:» conta como «modal»? (recomendado: não — não trava a tela)
- **P4** «Style» (estilos guardados) — agora ou numa onda própria? (recomendado: onda própria; não existe nada hoje)

**Respostas do dono (15/10, verbatim):** *«1: por enquanto só de vector · 2: Snap volta pro painel lateral como era
antes. · 3: Não entendi, mas procure fazer o melhor tendo com referência o Illustrator · 4: Etapa para o futuro ·
Duplicate sai. «Live: Off/On» por enquanto fica»*.
⇒ P1 só o layout Vetor perde os botões genéricos · P2 o Snap sai da ② e vive só no painel · P3 popover à Illustrator
(dropdown-painel ancorado, não bloqueante) · P4 Style fora da F16 · o Duplicate sai da ④ (fica no menu da Hierarchy) ·
o «Live: Off/On» do painel fica.
