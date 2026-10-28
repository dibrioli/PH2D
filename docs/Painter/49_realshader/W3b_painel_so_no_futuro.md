# W3b item 3 — o painel do pincel só vale para o futuro

**Lei do dono (27/10):** «Uma vez que o traço foi feito, se o usuário muda as configurações no painel, não afeta a tinta
já colocada no papel — passa a valer para os próximos traços.» Decisão do dono para os ambíguos: o dente do papel
(Tooth / Paper depth) é do PAPEL (tela inteira) e pode mudar tudo. A Paint Texture (ex-«Texture on Paint») foi do papel até
28/10, quando o dono a fez do TRAÇO («Paint Texture deve ser por traço e não para o papel todo») — ver a linha dela abaixo. O Smooth Edges foi do traço
numa ronda (`65d6a6a16d`) e o 2.º smoke do dono revogou-o (D1, 27/10: «para todo o papel, de todos os traços já pintados
ou futuros»): é da TELA, mora no cartão **Paper**, não viaja no preset e não está no gate deste item
(`oraculo/README.md` §Smooth Edges na tela).

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
| Paint Texture (`PAINTER_WETPAINT_PAPER_VISUAL`, `paper_visual`) | `render::pigment_row` (o papel impresso; `PigmentVisual::paper` = há papel escolhido) | **sim** até 28/10 (tela inteira) | **do TRAÇO (dono 28/10)**: guardada por célula (`AparenciaCelula::textura`, `Sim::textura`/`Params::textura`, facto `textura` da sessão); o render imprime só onde a célula a pediu; o clique não recompõe; viaja no preset (`paintTexture` no `engine`, só ligada — fábrica ao byte) e por ferramenta (`textura_por_slot`); saiu do cartão Paper para o cartão **Wet Paint** (abaixo do Show Wet) |
| Smooth Edges (`PAINTER_WETPAINT_SMOOTH_EDGES`) | `composite.rs` → `oleo_real::amacia_a_silhueta` (com o óleo) e `smooth_edges::Suave` | tela inteira, por decisão (D1) | mudou-se para o cartão **Paper**; recompõe no tick |
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

`AparenciaCelula` = **16 B por célula** desde 28/10 (papel · grão · vidrado · **textura**, a Paint Texture do traço; o `suave` da ronda 2 das bordas saiu quando o Smooth Edges voltou a ser da TELA — `grid/aparencia.rs`, gate `size_of == 16`), só quando alocado — ligar a Paint Texture aloca-o (era 12 B: +4 B/célula, 4096² razão 1: +67 MB sobre os +201 MB; a medida abaixo é a dos 12 B) (medido no gate: a grade retém 67,1 B/célula; +12 =
+18 %; 4096² com razão 1: +201 MB). A fábrica paga **0** (gate). `u8` quantizado (3 B) foi pesado e recusado: os
valores dos presets (passo 0,01) deixavam de ser exactos (1/170 de passo) e a lei do vidrado `1` deixava de ser a de
velatura ao bit; o idioma da crate (relevo, vela, material) é `f32`.

## Gates (`crates/ph2d-tool-painter/src/tool/paint/wetpaint/painel_so_no_futuro_tests.rs`)

- `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` — TODO knob da tabela (knob novo entra sozinho) menos os 3
  `Rebuild::Paper`, e os 7 factos da sessão (+ `paint_texture`, 28/10); com e sem Óleo Real: 0 bytes.
- `o_traco_seguinte_usa_o_valor_novo` — Paper visibility, Visual grain, Glaze layering, Paint Texture: o traço seguinte muda, o de
  antes fica ao byte (mesma história com e sem a mudança).
- `o_vidrado_da_celula_e_a_lei_de_velatura_ao_bit` — uma tela pintada toda com o Glaze layering = a sessão cuja lei já
  é de velatura, ao byte, em toda célula pousada (secar, pintar por cima, ticks); ≤ 0,1 % de células só pela água.
- `a_fabrica_nao_aloca_a_aparencia` — `NEUTRA` = `WetKnobs::DEFAULT`; a fábrica não aloca; o campo custa 16 B/célula; `NEUTRA.textura = 0` (fábrica desligada);
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
| T1 render sem o `× textura` da célula (imprime em toda célula) | VERMELHO (`o_traco_seguinte_usa_o_valor_novo`) |
| T2 composite volta à caixa global (`paper_visual && papel escolhido`) | VERMELHO (`mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`) |
| T3 o clique da Paint Texture volta a recompor | VERMELHO (`um_controlo_do_pincel_nao_recompoe_a_tela`) |
| T4 o preset não lê `paintTexture` | VERMELHO (`a_paint_texture_viaja_com_o_preset_e_com_a_ferramenta`) |
| T5 `tool_link` não troca a da ferramenta | VERMELHO (idem) |
| T6 `do_pincel` pousa `textura` 0 | VERMELHO (`o_traco_seguinte_usa_o_valor_novo`) |
| T7 o facto `textura` da sessão fixo em `false` | VERMELHO (idem) |

13 mutações vermelhas (M1–M10, P1–P3); a mensagem do commit `151cf87c10` dizia 16.

## Aberto

- O pigmento que a água leva além de 2 células (escorridos longos com gravidade) fica com a aparência NEUTRA.
- O arrasto, o Smear e o Blend não movem a aparência (como o relevo).
- Os 5 knobs mortos continuam no registo (`Hidden`) e nos presets (`engine`): ligá-los ao produto ou apagá-los com o
  `render_region` é outra decisão.

## Depth, Gloss e Real Oil por traço (dono 28/10)

**Ordem:** «o Depth e o brilho [do cartão Real Oil] passam a ser POR TRAÇO» e, na mesma família, «a caixa Real Oil
também é POR TRAÇO: ligar/desligar não muda o que já está pintado». Luz, Shadow, altura/direcção da luz, fenda,
alcance, ambiente e o material de 8 campos continuam no cartão (TELA) e aplicam-se à tinta com óleo.

**Onde ficaram na tela:** **Depth** e **Gloss** saíram do cartão Real Oil e abrem o grupo **Tuning › Material**
(knobs `oilDepth` `0..3` e `oilGloss` `0..1`, fábrica `1`/`1` — as faixas e o neutro do cartão de antes). Decisão
pelo idioma do painel: o Material é o grupo onde a ronda 3 (D3) já pôs a Transparency/Transmission/Paper Relief que
saíram do mesmo cartão — os «pesos» do óleo do pincel. A caixa **Real Oil** fica onde estava (cartão Wet Paint), mas
é do pincel; o cartão mostra os seus números com a caixa ligada **ou** com tinta com óleo na tela (decisão: são da
tela — com a caixa desligada e óleo pintado, o artista ainda move a luz dele).

**A lei.** `grid::oleo_celula` — `Grid::oleo: Vec<OleoCelula { oleo, profundidade, brilho }>` (**12 B/célula**, só
depois do primeiro traço com Real Oil; a fábrica e a tinta sem óleo pagam **0**: o traço sem óleo pousa a `NEUTRA`
`{0, 1, 1}` e não aloca, logo «o campo existe» = «houve óleo nesta grade»). Pousado em `Trail::transfer_paint` ao
lado da aparência, misturado pela massa, orla virgem `ORLA_VIRGEM` = a do material (4, medida lá), no instantâneo, no
`GridPatch` (um desfazer que deixa a grade sem óleo LARGA o campo), no `clear_canvas` e em `bytes_retidos`.
`Sim::oleo`/`Params::oleo` e o facto `oleo` da sessão (o idioma do Glaze e da Paint Texture).

- **Campo de altura** (`CampoAltura::de_com`): a massa de cada camada × `Depth × Real Oil` da célula ANTES do desfoque
  (com as três alturas, os fios e a crista); a tinta sem óleo é lisa para o campo. Sem o campo em camada nenhuma
  (réguas, testes do motor) a chamada é a de antes: toda a tinta é óleo de Depth 1. Os planos `brilho` e `oleo` do
  campo são o que se VÊ (a camada de baixo com tinta e as de cima pela cobertura), no cache por linhas como os outros.
- **Sombreado:** `o.brilho × brilho da célula` nas quatro portas do especular (`termos_com`).
- **Cor** (`render::pigment_row`): por célula, `w` = a fracção com óleo — `1` = a óptica (ao bit da de antes), `0` = a
  tinta sem ela (ao bit da tela sem óleo: o emboss volta), entre os dois a mistura das duas cores e coberturas, com o
  emboss da parte sem óleo acumulado à parte. A tinta sem óleo não recebe a sombra do óleo (nem o papel por baixo da
  orla dela); o dente do papel na tinta (`grao_do_papel`) × `w`.
- **Composite:** a óptica corre quando ALGUMA camada visível tem o campo (`oleo_na_tela`), com os números do cartão;
  o cache do campo só se larga sem óleo em célula nenhuma. Os números do cartão recompõem (varredura do tick) só com
  óleo na tela; a caixa, o Depth e o Gloss não recompõem.
- **Smooth Edges numa tela com óleo:** a orla escolhe-se POR PÍXEL (`Suave::aplica_com`): a tinta sem óleo (e o
  papel à volta só dela) guarda a orla de sempre; a com óleo, a silhueta do campo. Sem isto, o primeiro traço com óleo
  mudava a orla de toda a tinta sem óleo (gate: o caso «realOil ligado com Smooth Edges» do traço seguinte).
- **A caixa só à vista de quem pousa** (`LinhaDaFerramenta::DepositoMolhado`, como a Paint Texture): a Borracha e as
  ferramentas da água não a lêem (`o_painel_mostra_o_que_a_ferramenta_le`); os números do cartão aparecem nelas só com
  óleo na tela.

**Parede entre traços (medido, `grid/oleo_celula_tests.rs::a_parede_…`, faixas de 6 000 de massa lado a lado):**

| Fronteira | Depth ANTES do desfoque (escolhido) | Depth DEPOIS do desfoque (recusado) |
|---|---|---|
| Depth 1 · Depth 3 | maior salto 2,571 px, monótona | degrau de 18,000 px numa célula |
| óleo · sem óleo | 1,286 px, monótona | 5,143 px |

Monótona = sem ressalto; `o_relevo_nao_sai_do_pigmento` verde (a tinta sem óleo é para o campo como a tinta de
Depth 0, e o papel continua a 0). Consequência medida (`o_traco_seguinte_…`): o PAPEL a ≤ 24 células (o alcance da
fenda) de um traço mais alto escurece pela fenda dele — é a física; a tinta já pousada não muda.

**Migração.** Preset de pincel: nunca levou o cartão; sem `oilDepth`/`oilGloss` = fábrica `1` (os 65 de fábrica e
miniaturas ao byte). A caixa no preset: `engine.realOil` gravada só ligada (`CHAVE_OLEO`, o idioma do Glaze —
ausente = desligada; **escolher um preset de fábrica desliga o Real Oil**) e por ferramenta (`oleo_por_slot`).
Macro de antes (`oleo_real_valores.profundidade`/`brilho` no cartão e o preset sem os knobs): o valor do cartão vira
o do PINCEL (`macro_estado::cartao_antigo_no_pincel`) — o traço reproduzido pousa a altura e o brilho com que foi
gravado; a macro nova já não os escreve no cartão. `OleoReal::profundidade`/`brilho` ficam como ganho GLOBAL do motor
(réguas e testes), a `1` no produto.

**Gates.** `ph2d-wet-paint`: `grid::oleo_celula::tests::{a_parede_entre_traços_de_depth_diferentes_e_uma_rampa_sem_degrau,
o_oleo_de_fabrica_e_o_campo_de_antes_ao_bit, o_campo_nasce_com_o_oleo_mistura_se_e_larga_se_sem_ele}`. `ph2d-tool-painter`
`wetpaint/oleo_por_traco_tests.rs`: `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos` (Depth, Gloss, desligar
sobre óleo, ligar sobre sem óleo), `pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte` (a lei da caixa de antes,
`oleo_plano::lei_da_caixa`, 0 píxeis), `o_cartao_segue_a_tinta_com_oleo_na_tela`, `o_real_oil_viaja_no_preset_e_por_ferramenta`,
`a_macro_com_o_depth_e_o_gloss_no_cartao_passa_os_ao_pincel`. O gate principal deste doc percorre os dois knobs pela
tabela e o facto `real_oil` (FACTOS); o censo `um_controlo_do_pincel_nao_recompoe_a_tela` corre com e sem óleo e
leva o clique `real_oil`. Painel: `seam_wetpaint_oleo_real` (14 linhas; o cartão com óleo na tela e a caixa
desligada, pintado e clicável; `o_cartao_real_oil_nao_mostra_os_pesos_da_tinta` + «Depth»/«Gloss»), Tuning
`a_ordem_do_material_…` (Depth, Gloss à cabeça) e `every_visible_knob_has_a_row` (Material 13 → 15). A elisão
(`cada_nome_deste_painel_cabe_na_coluna_da_seccao`) não mudou (o cartão perdeu dois nomes curtos).

Catracas do registo apertadas (medidas): `wet_tuning` 15 → 14 comandos, `painter_layers` 1 570 → 1 568 px.

**Mutações** (`ph2d-run.sh cargo test`, perfil dev):

| Mutação | Resultado |
|---|---|
| O1 o pouso não escreve o óleo | VERMELHO (`o_traco_seguinte_…`) |
| O2 o campo ignora `Depth × óleo` da célula | VERMELHO |
| O3 o especular ignora o Gloss da célula | VERMELHO |
| O4 a cor ignora a fracção com óleo (`w = 1`) | VERMELHO |
| O5 o composite volta à caixa | VERMELHO (`mudar_o_painel_…`, facto `real_oil`) |
| O6 sem a orla virgem do óleo | VERMELHO (`pintar_tudo_com_o_real_oil_…`) |
| O7 o cartão só com a caixa | VERMELHO (seam) |
| O8 sem a migração da macro | VERMELHO |
| O9 o preset não grava `realOil` | VERMELHO |
| O10 `tool_link` não troca a caixa | VERMELHO |
| O11 os números do cartão só recompõem com a caixa | VERMELHO (`o_cartao_segue_…`) |
| O12 a caixa volta a recompor | VERMELHO (`um_controlo_do_pincel_nao_recompoe_a_tela`) |
| O13 o desfazer não larga o campo | VERMELHO |
| O14 a orla do Smooth Edges volta a ser da tela toda | VERMELHO (o caso «com Smooth Edges» do traço seguinte) |
| O15 a caixa à vista na Borracha | VERMELHO (`a_borracha_em_cada_meio_so_ve_o_que_o_meio_lhe_muda`) |

15 mutações, 15 vermelhas.

Folha: [`out/Depth_brilho_por_traco_folha.jpg`](out/Depth_brilho_por_traco_folha.jpg) — à esquerda dois traços com
óleo (o de cima Depth 2,6 · Gloss 0,1: mais alto, mate); à direita o mesmo pincel com e sem óleo, encostados (a
fronteira sem degrau; a tinta sem óleo lisa).
