# W3b item 3 — o painel do pincel só vale para o futuro

**Lei do dono (27/10):** «Uma vez que o traço foi feito, se o usuário muda as configurações no painel, não afeta a tinta
já colocada no papel — passa a valer para os próximos traços.» Decisão do dono para os ambíguos: o dente do papel
(Tooth / Paper depth) e a Texture on Paint são do PAPEL (tela inteira) e podem mudar tudo; o Smooth Edges passa a ser do
traço e é do agente das BORDAS (fora deste item).

## Censo (medido, não lido)

Pergunta por controlo: *o painel escreve onde · quem lê · lê ao pintar, ao compor ou na simulação?* Medição: o gate
`mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` (todos os knobs de `KNOB_DEFS` + os factos da sessão, um a um, tela
inteira recomposta, com e sem Óleo Real) correu ANTES da cura como censo exploratório.

| Controlo | Leitor | Mudava o passado? | O que foi feito |
|---|---|---|---|
| Paper visibility (`paperVisibility`) | `render::pigment_row` (com Texture on Paint e papel escolhido) | **sim** | guardado por célula (`AparenciaCelula::papel`) |
| Visual grain (`visualGrain`) | `render::pigment_row` (idem) | **sim** | guardado por célula (`AparenciaCelula::grao`) |
| Glaze layering (K-M) (`km_glaze` → `Sim::camadas_transparentes`) | `Sim::gather_params` → `Ecra::Cobre::esconde`, lido no render E em todas as transferências verticais | **sim** | guardado por célula (`AparenciaCelula::vidrado`); a lei da célula é `Ecra::com_vidrado` em TODA porta que lia `com_vela` (render, Dry canvas, secagem, assentar do Glaze, levantar do traço, Wet/Smear) |
| Emboss (`emboss`) | render lê `Grid::relevo` | não (já da célula, Bug #48) | — |
| Glaze (knob `glaze`) | `Grid::vela` | não (já da célula, prompt 42) | — |
| Material (11 knobs) / Transparency | `Grid::material` | não (já da célula) | agente TRANSPARÊNCIA |
| Live Water (`agua_viva`) → `glaze_filme` | `Ecra::Cobre::filme` no render | **latente**: `VELATURA_DO_FILME = VELATURA_DO_FILME_VIVA = 0` | gate que reprova se as duas divergirem (`a_fabrica_nao_aloca_a_aparencia`) |
| Ext rake light / valley gran / wet sheen / grain dither / edge tint (grupo Look) | só `render::render_region` — **nenhum caminho do produto o chama** | não: não fazem NADA no produto | ronda 2: controlos mortos SAEM do Tuning (o grupo `KnobGroup::Look` deixa de existir; os 5 knobs vão para `Hidden`, censo `MORTOS` em `os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao`; Tuning 92 → 87 linhas). O `render_region` FICA: é a referência SPEC §13 e 4 testes dependem dele (`perf_experimental::{the_glaze_checkbox_leaves_unpainted_paper_byte_identical, the_glaze_checkbox_is_free_where_there_is_no_paint, …}`, `measure_experimental::measure_render_region_glaze_cost`) |
| Paper contrast / fibres / grooves (`Rebuild::Paper`) | `rebake_paper` (só sem papel escolhido) | não ao compor (medido: com e sem óleo, sem papel escolhido, 0 bytes) | do papel — fora do gate, por nome (`os_knobs_do_papel_sao_os_que_refazem_a_folha`) |
| Tooth / Paper depth (`brush.paper_depth` → `set_paper_tooth`) | `rebake_paper` → `Grid::paper` | só com papel impresso (Texture on Paint) — tela inteira, por decisão | mora no cartão **Paper** (`paint_paper_section`). Ronda 2: o preset de pincel deixou de o levar (`CAMPOS_DO_PINCEL` 38 → 37, `CAMPOS_FORA` `("paper_depth", DA_OBRA)`; a gravação tira-o também da base antiga; um preset antigo que o traga é lido e ignorado — os 65 de fábrica traziam todos `1.0`, a fábrica não muda) e o setter carimba-o em TODOS os slots (`set_paper_field`: antes pegar um pincel Digital devolvia o dente a 1,0) |
| Texture on Paint (`PAINTER_WETPAINT_PAPER_VISUAL`) | `composite.rs` → `PigmentVisual::paper` | tela inteira, por decisão | já mora no cartão **Paper** — não mexido |
| os outros ~100 knobs (Paint, Water, Physics, Tools, Bristles, Stroke, WetEffects, Pen, Tilt…) | depósito, solver, secagem, mão | não ao compor (gate: 0 bytes) | — |
| `km_mixing`, `borda_que_pesa`, `opacidade`, tilt | sim / depósito | não ao compor (gate) | — |

Os knobs da simulação (Evaporation, Leveling, Gravity, ExtGranulation, ExtStaining…) agem só sobre a tinta MOLHADA que
ainda corre — é física em curso, não a aparência da tinta assentada; a tinta seca não é simulada.

## A cura

`crates/ph2d-wet-paint/src/grid/aparencia.rs`: `Grid::aparencia: Vec<AparenciaCelula>` (papel, grão, vidrado; `f32`),
campo PREGUIÇOSO no idioma do `Grid::material`: VAZIO = `NEUTRA` (= os valores de fábrica) até o primeiro pincel fora
deles pintar. Escrito só no pouso do rasto (`Trail::transfer_paint`, ao lado do relevo e do material), misturado pela
massa; viaja no instantâneo, no `GridPatch` (undo de janela), no `clear_canvas` e em `bytes_retidos`.

- **Orla virgem** (`veste_virgens`, `ORLA_VIRGEM = 2`): a advecção não move este campo (como relevo, vela e material).
  Medido: sem ela 1 060 das 6 549 células pintadas (16 %) eram pigmento levado pela água a uma célula sem aparência —
  um aro de grão de fábrica à volta do traço. Distância ao rasto: 1 043 a 1 célula, 17 a 2 (40 ticks); 1 083 / 75 (240
  ticks); 0 a ≥ 3. A cura veste as células VIRGENS da caixa do pouso ± 2 com a aparência nova (uma célula virgem não se
  desenha: nunca muda um byte). Depois: 1 célula em 6 549.
- `Sim::gather_params` deixou de trocar o `esconde` pelo flag da sessão: dá `Params::vidrado` (pousado) e a lei de cada
  célula aplica-o (`Ecra::com_vidrado`, `1` = `TINTA_ESCONDE_EM_VELATURA` ao bit, `0` = a da sessão ao bit).

## Custo de memória

`AparenciaCelula` = **12 B por célula**, só quando alocado (medido no gate: a grade retém 67,1 B/célula; +12 =
+18 %; 4096² com razão 1: +201 MB). A fábrica paga **0** (gate). `u8` quantizado (3 B) foi pesado e recusado: os
valores dos presets (passo 0,01) deixavam de ser exactos (1/170 de passo) e a lei do vidrado `1` deixava de ser a de
velatura ao bit; o idioma da crate (relevo, vela, material) é `f32`.

## Gates (`crates/ph2d-tool-painter/src/tool/paint/wetpaint/painel_so_no_futuro_tests.rs`)

- `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` — TODO knob da tabela (knob novo entra sozinho) menos os 3
  `Rebuild::Paper`, e os 6 factos da sessão; com e sem Óleo Real: 0 bytes.
- `o_traco_seguinte_usa_o_valor_novo` — Paper visibility, Visual grain, Glaze layering: o traço seguinte muda, o de
  antes fica ao byte (mesma história com e sem a mudança).
- `o_vidrado_da_celula_e_a_lei_de_velatura_ao_bit` — uma tela pintada toda com o Glaze layering = a sessão cuja lei já
  é de velatura, ao byte, em toda célula pousada (secar, pintar por cima, ticks); ≤ 0,1 % de células só pela água.
- `a_fabrica_nao_aloca_a_aparencia` — `NEUTRA` = `WetKnobs::DEFAULT`; a fábrica não aloca; o campo custa 12 B/célula;
  Live Water latente.
- `os_knobs_do_papel_sao_os_que_refazem_a_folha` — a excepção do gate, nomeada.
- `escolher_um_pincel_nao_muda_o_dente_da_tela` (ronda 2) — os 65 presets de fábrica e um antigo com `paper_depth`
  1,9: o dente da tela fica; gravar não o leva.
- `ph2d-wet-paint/tests/it/a_aparencia_da_tinta.rs::o_vidrado_da_celula_e_a_lei_de_velatura_em_cada_porta` (ronda 2) —
  motor com o flag × sessão de velatura, ao bit nas grades, pelo Paint com Glaze 0,6 (o assentar do Glaze), o Blend
  (o levantar), o Wet e o Smear.
- Adaptados à lei nova: `tests_doc22::km_flags_reach_the_sim_and_the_composite`,
  `product_tool_doors::{the_glaze_stacks_the_film_over_the_dried_paint, the_paper_visual_prints_the_tooth_into_the_pigment_alpha_intact}`,
  `glaze_vertical_transfers` (a cena semeada à mão pousa o vidrado na célula; a régua lê a lei da célula).

## Mutações

| Mutação | Resultado |
|---|---|
| M1 render lê `p.k(PaperVisibility/VisualGrain)` em vez da célula | VERMELHO (gate principal: `visualGrain`, `paperVisibility`; e o do traço seguinte) |
| M2 render lê o vidrado do pincel (`p.vidrado`) | VERMELHO (`km_glaze`) |
| M3 o pouso não escreve a aparência | VERMELHO (`o_traco_seguinte_usa_o_valor_novo`) |
| M4 Dry canvas (`canvas_ops`) sem o vidrado da célula | VERMELHO (`o_vidrado_da_celula…`) |
| M5 secagem (`dry_cell`) sem o vidrado | VERMELHO |
| M6 sem a orla virgem | VERMELHO |
| M7 assentar do Glaze no pouso (`transfer.rs`, `vela > 0`) sem o vidrado | VERMELHO (ronda 2, `a_aparencia_da_tinta`, Paint) |
| M8 levantar do Blend (`transfer_blend` → `lift_settled`) sem o vidrado | VERMELHO (ronda 2, Blend) |
| M9/M10 Wet e Smear (`tools.rs`, as duas portas) sem o vidrado | VERMELHO (ronda 2, Wet · Smear) |
| P1 a gravação deixa o `paper_depth` da base | VERMELHO (`escolher_um_pincel_nao_muda_o_dente_da_tela`) |
| P2 `paper_depth` de volta a `CAMPOS_DO_PINCEL` | VERMELHO |
| P3 o setter do dente só no pincel vivo | VERMELHO |

## Aberto

- O pigmento que a água leva além de 2 células (escorridos longos com gravidade) fica com a aparência NEUTRA.
- O arrasto, o Smear e o Blend não movem a aparência (como o relevo).
- Os 5 knobs mortos continuam no registo (`Hidden`) e nos presets (`engine`): ligá-los ao produto ou apagá-los com o
  `render_region` é outra decisão.
