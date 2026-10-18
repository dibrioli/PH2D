# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 45: a Shape Color no pincel salvo e o esmaecer da pressão (2026-10-17)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 45](../NewFeaturesWetPaint/45_PROMPT_shape_color_no_pincel_e_o_esmaecer.md)
· medição e leis: [45_medicao](../NewFeaturesWetPaint/45_medicao_o_esmaecer.md) · lista [26](../NewFeaturesWetPaint/26_ABERTOS.md)
B13 ✅ · C27 ✅ · C28 (a pressão ✅; a textura a 20 px fica). Anterior: [P44](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P44_FORMAS.md).
Commits `a5134220e5` (identidade das 22 cenas, ANTES da lei) · `e2401f0ac1` (o esmaecer) · `d4d706303a` (a Shape
Color no pincel) · `ed7ed42a85` (a opacidade como argumento da porta) · `92b42ffa03` + `340d8c00b9` (o fecho) · os docs.

## §2 — Medido primeiro (doc 45 §1–§2)

- **O mecanismo** (perguntado ao código): a pressão → opacidade entrava no `b` do motor (`dab_route.rs`), que vira
  `intensity = min(3, 3,45·p)` ANTES do tecto `stamp = min(1, ·)` (`trail/deposit.rs::cell`): o miolo do dab satura
  para `p ≥ 0,29` e só a borda esmaecia. No pó a média por célula é a do contínuo. Nenhum orçamento por dab nem
  renormalização pelo traço — é o tecto.
- **Um lote** (uma compilação, um processo por variante, 10 variantes em 5 s no gesto; depois a cena seca): o lugar
  (a porta de antes · o carimbo · o depósito) × a curva γ. O depósito é o único que aceita a curva sem o toque leve
  deixar de pousar; a linear fica acima do Rebelle (a massa satura na cor) ⇒ curva por pincel.

## §3 — As leis

- **A opacidade do dab** (`trail::Dab::opacity`, argumento de `dispatch_pressure_dab_lane_cor`; as outras portas
  pousam com `1`): multiplica o depósito depois do portão e do `sv`. Na porta do PINCEL (Paint, não a borracha) o `b` é
  o do toque inteiro (`10 · overlap`) e a opacidade `coverage ÷ (strength · overlap)`; o resto lê o `b` de sempre.
  O overlap do método é UM predicado (`StrokeMethod::is_spaced_fill`, `BrushSpec::method_overlap_factor`).
- **A curva** `pressureOpacityCurve` (Tuning › Paint, 0,25–4, omissão 1 = linear; `KNOB_COUNT` 59):
  `Dynamics::strength_curve`, `min + (1 − min)·p^γ`; γ 1 = o `p` sem `powf`, ao bit. O painel conta 33 knobs.
- **Fábrica**: Wet Flat γ 1,75 → rampa 0,77/0,41 (Rebelle 0,83/0,42) · Gouache Flat γ 2 → 0,77/0,42 (0,87/0,43) ·
  Airbrush tamanho 0,5 + γ 1,5 → 0,60/0,35 (Spray 2 0,63/0,34). ⚠️ Os números literais do Spray 2 (tamanho 0)
  esmaecem igual mas NÃO afinam (largura 1,00) — ficou o tamanho 0,5 (a ordem era «como no Rebelle»).
- **A Shape Color no pincel** (B13): `Parametros::shape_color` (opcional; `SCHEMA` fica 1 — o leitor velho
  guarda-a no `extra`). UMA lei por nome, `EstadoDaShapeColor::para_json`/`ler_json` (`shape_ramp.rs`), lida pelo
  preset e pela macro (o estado inicial da macro já não a grava no topo; lê a das macros de antes). Grava só quando
  difere da de fábrica; aplicar SEM a chave repõe a de fábrica (`reset_shape_ramp` usa o mesmo `Default`). A
  miniatura pinta-a (o ficheiro devolve-a).

## §4 — Portões

Identidade: as 19 cenas de `a_lei_da_tinta` sem a pressão → opacidade ao bit (a lei de antes reproduz as 20 de
`HEAD`; o Wet Flat e a Gouache Flat pinados com ela em `a5134220e5`); Airbrush, Wet Flat, Gouache Flat re-pinados
por ordem do dono; só esses três ficheiros regerados. `ph2d-wet-paint` `a_opacidade_do_dab` (2) ·
`as_formas::a_opacidade_da_pressao_morde_no_po` + `a_curva_esmaece_mais_no_toque_leve` (vermelhos com a lei de
antes) · `the_coverage_curve_shapes_the_pressure_only` · `midias_de_fabrica` `RampaCobertura` (3) +
`RampaLargura` no Airbrush · `wetpaint_preset/shape_color_tests.rs` (4) · `a_miniatura_pinta_a_shape_color`.

## §4b — Pedido do dono antes do smoke: o Randomize Color com a Shape Color

Com uma rampa dona da cor (Shape Color, e a rampa de cor do Grain) o desvio `Dab::color_shift` nunca chegava à cor:
a rampa dava a cor e o Randomize só mexia na tinta do pincel, que ela substitui. Lei única (a do Bug #51):
`apply_color_shift(cor_da_rampa, d.color_shift)`, o alfa fica, zero = ao bit. Rotas: `RampPaint::shift` lido em
`ramp_texel` (o por-pixel, por dab) · o carimbo assado desloca no BLIT (`blit_color_stamp`; a cache é do traço,
não se assa o desvio de um dab) · a rota da LUT com uma cópia deslocada por dab (`shift_ramp_lut`) · Wet Paint e
Watercolor por `RampaDaAguada::cor_do_dab`. Impasto e Composite passam pelas mesmas rotas. Commits `4bd26b6d4b`
(lei) · `adfd93d709` (portões `tests/rampa_randomize.rs`, 6, cada um vermelho sem a cura e com a ROTA verificada
pela cache que encheu) + o lint de clippy do fecho. Identidade (`a_lei_da_tinta`, miniaturas de fábrica) intacta.

## §5 — Mutação

**13/14 sangram** + 1 que não é buraco (agente `mutacao`, controlos verdes antes, âncoras 1×). Sangraram: o depósito
sem a opacidade, a porta sem ela, a lei de antes (`false &&`), o `b` com a pressão de volta, a curva ignorada no
`Dynamics` e no tool, a captura sem a chave, o aplicar que não repõe, o `ler_json` sem `blend`/`bw`, a macro de antes
ignorada, a miniatura sem a rampa. ⚠️ **M7 sobreviveu** («repor a opacidade a 1 no fim do lote»): o campo
`Engine::opacidade_do_dab` era estado que todas as portas liam. Curado pela FORMA (`ed7ed42a85`): a opacidade é
argumento de `dispatch_pressure_dab_lane_cor`, as outras portas pousam com 1 — nada a repor; tirar a linha nova da
porta → vermelho. M10 (gravar a chave sempre) sobrevive ao `cada_preset_de_fabrica_volta_ao_byte` (que não captura)
e sangra em `um_pincel_sem_rampa_tira_a_do_anterior`.

## §6 — Aberto

- A 20 px o Airbrush lê textura 28,0 contra 22,5 (C28, não juntado).
- `cada_preset_parece_a_sua_midia` / `a_largura_e_a_prometida` passam do prazo de 360 s sob carga (faixa lenta).
- O Airbrush com os números literais do Spray 2: decisão do dono no smoke se preferir o tamanho 0 (não afina).

## §7 — Fecho

`cargo fmt --all --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; wet-paint, painter-brush,
tool-painter, brush-presets, i18n, panel-wet-tuning, panel-painter-layers, app-painter) ✓ depois de 1 defeito
(`neg_cmp_op_on_partial_ord` na curva, `92b42ffa03`) · `nextest-impacted` (base `ea0d7a8672`) **18 909/18 910** em
888 s (`load` 120 → 9): 1 defeito desta onda — `ph2d-panel-wet-tuning` `every_visible_knob_has_a_row` contava 32
knobs visíveis (a curva é o 33.º, o 14.º do Paint), vermelho também sozinho; curado em `340d8c00b9`, refeito por
nome ✓. Nenhum timeout; o tecto de LOC passou.
