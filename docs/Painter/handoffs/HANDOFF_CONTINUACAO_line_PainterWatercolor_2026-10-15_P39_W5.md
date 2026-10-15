# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 39: a W5 do plano 27 (2026-10-15)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (merge-base, sem rebase). Sem push. [Prompt 39](../NewFeaturesWetPaint/39_PROMPT_w5_flat_fan_saem_doc_do_shape.md)
· [plano 27 §6.10](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md) · lista [26](../NewFeaturesWetPaint/26_ABERTOS.md)
(A1 feito; B6 com a medida, resposta pendente) · [doc 40](../NewFeaturesWetPaint/40_o_shape_casa_dos_pinceis.md) ·
[prancha 39](../NewFeaturesWetPaint/39_prancha_b6_ponta_lisa_e_cerdas.png). Anterior: [P38](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P38_COR_DO_SHAPE_EM_CAMADAS.md).
Commits: `27c75b50e3` (corte) · `2a604229d1` (portão D7, doc 40, prancha) · o deste fecho (docs).

## §2 — Flat/Fan saem do motor (`ph2d-wet-paint`)

Censo reconferido no código de 15/10 (as linhas do 27 §6.4 tinham andado): `Engine::shape` só escrito no construtor
(`Round`), lido só por `pressure_dab` → `Dab::shape` → `for_each_stamp_pixel` (5 ferramentas + as 2 rotas do
`Trail`); a elipse só corria sem silhueta, e o produto passa sempre `Some(&sil)`; zero presets/ficheiros gravados
os nomeiam (o `BrushShape` não tinha `serde`). A direção do traço (`Stroke::dir_*`, `StrokeEvent::Dab::dir_*`,
`Dab::dir_*`) só alimentava a elipse ⇒ saiu também.

- **API mudou:** `dispatch_pressure_dab(x, y, b, r)`, `_lane(lane, x, y, b, r, sil, grain, padrao)`,
  `_erase(x, y, b, r, sil, grain)`, `_lane_blend(lane, x, y, b, r)`, `_tool(tool, x, y, b, r, prev)` — os dois `dir`
  saíram. Chamadores: `wetpaint/dab_route.rs` (4), `macro_track/tests.rs`, 16 testes do motor. Outra linha que chame
  estas portas com `dir` não compila — o erro diz o sítio (arity).
- `for_each_stamp_pixel` = só o disco; os docs órfãos do `brush.rs` (o do iterador estava colado ao `CellFn`) foram
  arrumados.
- **Prova ao bit:** `ph2d-wet-paint` `fingerprint` (a impressão FIXA da sessão) e
  `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (12 ao byte) verdes; mais `product_doors`,
  `product_tool_doors`, `deposit_rows`, `the_window_follows`, `the_accumulate_declares`, `glaze_vertical`,
  `acceptance` (38 ✓) e `lei_b_tests`, `mascara_tests`, `shape_bristles`, `macro_track`, `dab_route` (24 ✓).

## §3 — Mutação nas leis W1/W2 (nunca tinham prova) e o portão que faltava

Agente `mutacao`, controlo verde antes de cada uma:

| M | lei | resultado |
|---|---|---|
| M1a/b | W1: tapete em PÍXEIS; os 4 knobs na ordem | sangram (`o_ladrilho_do_shape_e_o_do_motor_ao_bit`, `as_cerdas_amostram`, `os_presets_antigos…`) |
| M2 | W2: Bristles → `texv` | sangra (`o_wet_paint_le_as_cerdas_do_shape`, `cada_transformacao_do_shape_muda_a_ponta_de_cerdas`) |
| M3 | D4: Shape ≠ Bristles = ponta LISA | sangra (`o_wet_paint_le…` «a ponta lisa pinta igual», `lei_b …_ponta_lisa_nao_perde_os_vales`) |
| M4 | migração dos presets antigos | sangra (`os_presets_antigos_chegam_com_as_cerdas_no_shape`) |
| M5 | **D7: o Grain SUBSTITUI as cerdas** | **sobreviveu** (223 testes verdes) ⇒ portão novo |
| M6 | `Dependente::CerdasDaAgua` | sangra (`censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age`) |

Portão novo: `tool/paint/tests/shape_bristles.rs` `um_grain_armado_substitui_as_cerdas_do_shape` — Soft Pastel (grão
armado), Shape Bristles com as cerdas do Wet Round = Shape None ao byte; Count 0,1 idem; controlo «o pastel pintou».
Vermelho sob M5 («grão × cerdas» em `dab_route.rs` `grain_arg`), verde restaurado. A mensagem conta píxeis (o
`assert_eq!` de `Vec` despejava ~1 MB).

## §4 — Doc e prancha

- [40_o_shape_casa_dos_pinceis.md](../NewFeaturesWetPaint/40_o_shape_casa_dos_pinceis.md): as 30 formas por classe, os
  três canais do motor (`sil` · `texv` · `padrao`), o painel por mídia (Relief/Shine só onde `impasto_applies`: Digital
  e Impasto), a Shape Color, a tabela lei → portão. Indexado no `00_README`.
- B6: `sonda_shape_casa.rs` `sonda_prancha_b6_lisa_e_cerdas` (azul do oráculo, `PH2D_SONDA_OUT`) +
  `ferramentas/prancha_b6.py`. lisa ÷ cerdas: Wet Round 6,23 · Wet Wash 4,28 · Wet Dry Brush 4,80 · Wet Liner 1,44 ·
  Acrylic Round 1,66 · Oil Bristle 1,00. O Round do Rebelle (célula molhada, água 10) fica ENTRE as duas.

## §5 — Fecho

`cargo fmt --check` (wet-paint, tool-painter) ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`) em
`ph2d-wet-paint`, `ph2d-tool-painter`, `ph2d-painter-brush`, `ph2d-panel-wet-tuning`, `ph2d-pigment` ✓ ·
`nextest-impacted` (base = merge-base `0aab2c1cb6`) **18 458/18 458** ✓ em 460 s, `load 4,1` · `grep BrushShape`
em `crates/` = só o `UseAsBrushShape` da Hierarchy (outra coisa). Smoke ao dono: §7.

## §7 — Smoke ao dono (15/10)

Na resposta da janela, passos numerados: (a) o Shape Noise e o Bristles no Wet Paint (o #47 fechado); (b) o painel
por mídia (Use Texture Colors esmaecido na água; Shape Color some no Wet Paint); (c) a pergunta B6 com a prancha 39.

## §6 — ABERTO

- **B6** (dono): manter a lisa, ou calibrá-la (`texv` da lisa = um número só, contra `fixtures/rebelle/ponta_textura_round_*`).
- O resto do Shape: lista 26 B9 · B12 · C16 · C23 · C24; biblioteca de pontas PNG (D5).
- `docs/Painter/00_INDEX.md` não cita a pasta `NewFeaturesWetPaint` (pré-existente).
