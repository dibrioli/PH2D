# Continuação — line/PainterWatercolor, onda de 07/11: pânico do undo, refazer, lote de medição

Base: `main` `652b506b58` (reaberta por `reset --keep`; `git cherry` = 0 `+`, 34 `-`; ponto de volta `backup/line-PainterWatercolor-pre-reabertura-2026-11-07`). Commits: `c887156cbb` (item 1), `3fc47dc81d` (item 3), e o desta onda (sondas e este doc). Não integrada.

## Missão autónoma de 08/11 (noite): Wet blend à CSP + 16 pincéis CSP

**SUPERA** «Color mixing → Pigment mix» e o item 0 de «Próximos» abaixo. Sem smoke do dono (é amanhã); suíte completa NÃO correu (é do integrador).
- **Ordem do dono (08/11):** mistura do Digital tão boa ou melhor que o Clip Studio (foto dele: costura dura/pixelada da tinta de baixo dentro do traço do Pigment mix); depois os principais pincéis do CSP; autónomo.
- **Backup:** branch `backup/PainterWatercolor-2026-11-08-antes-mistura-csp` (a3f6b20b78) + `/home/enio/Documentos/Projetos/PH2D-backups/PainterWatercolor-2026-11-08-antes-mistura-csp.tar.gz`. Checkpoint da lei: `f77c01494e` (inclui o trabalho sem commit de 07–08/11).

### Lei «Wet blend» (`ph2d-painter-brush/src/mistura_opaca.rs`)
- `ModoMisturaOpaca::{Off, WetBlend}`; PigmentMix/Blend/RunningColor carregam como WetBlend.
- Por carimbo, amostra = média da tela de ANTES do traço + película VELHA do próprio traço (1.º depósito há > `DESFASAGEM`=1,0 raio de caminho) sob a pegada. Reservatório pelo Color stretch (memória L = 0,1·e^(4s) diâmetros). Tinta P = lerp(reservatório, pincel, Amount); Pigment 0 = RGB do CSP, 1 = espectral.
- Película f32 (`Sombra`, ladrilhos 64², 5 f32/texel: RGB, cobertura, arco do 1.º depósito). TECTO = coverage do dab × grão/silhueta (Shape e Grain vão ao tecto como no Off, via pegada `sem_grao`); TAXA = Density × Flow × pegada, normalizada pelo spacing NOMINAL. Accumulate ligado = tecto 1. Amount < 0,25 (`AMOUNT_QUE_COBRE`) não cobre o vazio (esbatedor).
- Passo efetivo dos carimbos `PASSO_DA_MISTURA` = 0,03 diâmetro (`BrushSpec::spacing_do_traco`, `stroke/walk.rs`). Rota `stamp_route/mistura.rs` reescrita (sem cobertura/força/tecto antigos); cauda (`traco_plano/cauda.rs`) recomeça a sombra; `escala_do_plano` exclui a mistura.
- Painel: ids `painter_brush.mix.wet_blend`, `painter_brush.mix.pigment_amount`; ordem [0,1,2,5,3,4]; i18n `mix_wet_blend` «Wet blend», `mix_pigment_amount` «Pigment», `mix_stretch` «Color stretch»; JSON `mistura_opaca` com chave "pigment". Visível: Color mixing · Off · Wet blend · Amount · Density · Color stretch · Pigment.

### Medido contra o CSP corrido (Xvfb; `material_2026-11-08/wetblend_vs_csp_*.png`, `wetblend_vs_csp_medidas.txt`)
- Fração do fundo a Amount 0,8 = 0,072 com amostra viva pura (CSP 0,07).
- Varredura da DESFASAGEM (erro médio por região, 4 pincéis): 0,75→95 · 1,0→79 · 1,25→69 · 1,5→67; Round mixing 7·12·16·20 → escolhido 1,0.
- ⛔ Recusas MEDIDAS: amostra viva pura (G-Pen não misturava e dependia do spacing); Density como tecto (véu chapado; CSP com Density 45–50 cobre 88–98 %).
- Custo (sonda `custo_do_wet_blend_por_tamanho`, arrasto 300 px, 100 eventos): Hard Round r8 2,9→4,6 ms, r30 8,8→38,1, r100 7,1→73,4; Sketch Pencil r100 17,3→129,0 ms (≤ 1,3 ms por evento).
- No app (Xvfb, `PH2D_WETPAINT_SMOKE=1`, xdotool): mescla lisa na borda, sem costura. Fotos `material_2026-11-08/no_app_wet_blend_*.png`.

### 16 pincéis de fábrica novos (84 → 100; o CLAUDE.md §5 diz 78: o integrador reconta)
- `tests/it/familias/csp_tinta_grossa.rs`: Round Mixing, Gouache, Dry Gouache, Thin Gouache, Gouache Blender, Oil Paint, Paint And Apply, Thick Oil Paint (pasta Digital/Oil/Thick paint; cerdas pelo Shape Bristles, macro `cerdas!`).
- `tests/it/familias/csp_secos.rs`: Crayon, Pastel, Chalk (Digital/Pastel); Pointillism (Digital/Texture); Wet Wash Mix e Watercolor Splash em Multiply (Digital/Watercolor); Brush Pen (Digital/Ink/Brush); Finger Tip (Digital/Blend).
- Sondas `sonda_csp_tinta_grossa.rs`, `sonda_csp_secos.rs`; folhas `csp_lote_A_tinta_grossa.png`, `csp_lote_B_secos.png`. Miniatura dos esbatedores: faixa vermelha das ferramentas (`wetpaint_preset/preview.rs`, `so_arrasta`). Gerados com `PH2D_REGERA_SO` só dos 16; nenhum ficheiro antigo mudou.
- Limites de motor vs CSP (NÃO construídos): spray espalha ≤ 2 diâmetros (Splash CSP 3,25×); jitter de cor entra antes da mistura (Pointillism compensa a amplitude); sem pontas de imagem (Splash sem contorno recortado; Wet Wash lóbulos mais esfumados); Paint and apply sem o borrão próprio do CSP; Gouache cobre um pouco menos; pontas de traço arredondadas.

### Verde por nome e ABERTO
- Verde: núcleo wet_blend 10 · ferramenta lib mistura+preview 77 · `it` a_mistura 13 (hash do Watercolor Basic idêntico) · `cada_preset_de_fabrica_e_a_sua_receita`, `the_factory_presets_are_counted`, os 2 alarmes de pincel, `cada_familia_e_um_ficheiro` · painel mistura/color_mixing.
- ABERTO: smoke do dono (amanhã); fmt/clippy 1× no fecho; suíte completa (integrador).
- ⚠️ A fila pesada (`scripts/fila-pesada.sh`) tem itens 1–3 velhos desta onda: o 1 regravaria os 6 pincéis CSP de 07/11. **NÃO correr `correr`; limpar antes.**

## Item 1: FEITO, pânico do undo com `flow_ratio` > 1
- **Mecanismo:** `FotoDoMotor::de` passava `g.s` a `flow_x/flow_y`, que moram em `g.flow`. `foto_em_tiles` truncava e `repoe` encolhia o plano; `apply_boundaries` (`project.rs:269`) entrava em pânico na thread da água. RED com a mensagem exata do dono (`1052676` × `1051650`).
- **Geometria do dono, por exclusão:** com fluxo de 1024, a conta só fecha em largura fina 2048 e `flow_ratio` 2 (varridos rf 1..16). Não é a tela do `PH2D_COMPOSITE_SMOKE` (1024²): ele estava noutra tela, ou 4096 com grade 2.
- **Cura:** `g.flow.s`; `PlanoEmTiles::de` recusa comprimento não múltiplo de `s`.
- **Varredura da família, sem outro achado:** `oleo_antes` é fino; `tira/poe_os_tiles` e `dormente` levam `s`, `rows` e comprimentos; `enterra_celula` só toca planos finos.
- **Portões:** `undo_matriz_tests`, 26 testes do undo vivo em 1×2, 1×3, 1×4, 2×1, 2×2, 4×4, com controlo de que as razões chegam à sessão; a cena de 2048; a foto ao bit em rf 1, 2, 3, 4, 16. **Mutação 6/6 detectadas.**
- **1f, as razões reais:** o app nasce 1×1 em qualquer tamanho (`DEFAULT_RATIO` 1, `flow_ratio` 1). Só o slider «Flow Grid (x)» ou o estado de uma macro as mudam; preset não muda (só a prévia, que repõe). Os números de 07/11 foram medidos a 1×1, logo valem para o padrão. Com fluxo 2 (`target/lote/preco.log`, não versionado; sonda `sonda_o_preco_de_um_passo_no_fluxo_do_dono`): pen-down 10–14 ms a 1080p (traço curto), 34–38 ms a 4K na aguada larga, 7–8 passos vivos com ~3,4–3,7 GB de registos nessa cena. Comparado aos de 1×1 só a olho: **não é medição intercalada**.
- **Duas réguas só valem na grade 1, com o motivo no teste:** a tinta do 1.º traço para lá do tecto e `z < cru/4` da compressão.
- ⚠️ **Correção ao handoff de 07/11:** `para_la_do_tecto_so_o_traco_desfeito_acaba` só passava por margem. A 1×1 a tinta do 1.º traço cai de 253 489 para 46; a 2×1 e 4×4, para 0. É o ABERTO 1 de lá (`tocada` enterra células longe), em toda razão, e pior do que o texto dele sugere: «a tinta que sobrou continua viva» é falso para aguada para lá do tecto. **Não curado.**

## Item 3: FEITO, dois mecanismos
1. **`Registo::restaura` contava `oleo_real` antes de pôr os tiles.** Refazer o 1.º traço de óleo recria `g.oleo` neutro, a conta dá `false`, o quadro seguinte compõe sem `sombra` nem `grao`: 13 796 px, Δ até 187, a 240×160; a grade ao bit. O refazer seguinte acertava. Cura: recontar depois dos tiles.
2. **`wet_volta_do_undo` não invalidava a imagem no ar.** Pedida antes do comando, chegava depois e ficava na tela um passo atrás (`refaz→6: 5`, `desfaz→6: 7>6>7`), intermitente: 1 falha em 3 corridas com o relógio do app, 0 em 6 depois. Cura: `toca_tudo` em todo restauro aplicado.

Impasto: certo em todo passo, até 1024 e com os comandos seguidos. A suspeita inicial (`undo.rs`, `undo_planes.rs`) não se confirmou. Pilha mista: cor e imagem certas; desfazer através do outro meio devolve a tinta morta, por desenho. Portões: `cada_passo_devolve_cor_relevo_e_imagem_do_mesmo_estado`, `refazer_o_primeiro_traco_nao_perde_o_relevo` (4 óleos), `desfazer_e_refazer_recusam_a_imagem_no_ar`. Mutação: 3 detectadas; sem `reconta_teto` sobrevive, equivalente (`suave_teto` só sobe). Alarmes 2/2.

## Item 2: MEDIDO; as lições NÃO estão feitas
Sonda `sonda_oleo_x_aguada_por_fase`: mesmo traço, ferramenta nova por corrida (réplicas, J13 §5), relógio do app, 60 Hz, ordem rodada, CPU por fio via `schedstat` entre as fases. Mínimo de 3 rodadas, 2 no tamanho 30. A última rodada de 4K 1×2 foi cortada pelo prazo de 30 min (`rc=143`): n=2 em alguns. Carga ~3–9. `rayon=16` (quota da fatia): os valores absolutos de CPU são da fatia, não do app.

**O óleo NÃO ganha em tudo. Ganha ao assentar e parado; perde durante o traço e ao soltar.**

Mediana do quadro no fio da tela, ms (`p50 / p95`), Size proporcional à tela (120 a 1080p, 240 a 4K):
| Fase | Dabber | Dabber sem relevo | Flat Bristle | Flat s/ relevo | Wet Round | Wet Flat | Wet Wash |
|---|---|---|---|---|---|---|---|
| traço 1080p | 34/45 | 12/22 | 172/245 | 110/168 | 0/13 | 6/9 | 5/8 |
| traço 4K | 130/199 | 68/133 | 1037/1580 | 795/1323 | 0/50 | 18/30 | 15/23 |
| soltar 4K | 130 | 70 | 1154 | 864 | 0,1 | 18 | 14 |

Size 30 (p50/p95): traço 1080p Dabber 8,5/10 · Flat 20/25 · Wet Round 2,4/3 · Wet Flat 4,2/6. Traço 4K Dabber 12/15 · Flat 26/33 · Wet Round 3,9/5,3 · Wet Flat 7,5/10.

Depois de soltar (CPU total por segundo; imagens na tela por segundo; células com água):
| | 1080p óleo | 1080p aguada | 4K óleo | 4K aguada (Size 240) | 4K aguada (Size 30) |
|---|---|---|---|---|---|
| Assentando, CPU | 0,2–2 núcleos, 2 s | ~8 núcleos | 0,7–8 | **~13 núcleos** | ~8 |
| Parado (7 s depois), CPU | **0** | 7–9 núcleos | **0** | **11–14 núcleos** | 8–9 |
| Imagens/s, parado | 0 | 20–29 | 0 | **0–1 (Wash 3–4)** | 9–18 |
| Células com água | 0 | 570–720 mil | 0 | 2,4–2,9 milhões | 310–390 mil |
| O worker ocupado (parado) | 0 % | 34–44 % | 0 % | **96–98 %** | 58–65 % |

- **Por que o óleo parece rápido:** a água dele acaba em ≤ 2 s (`has_fluid` cai; caixa 0) e depois o custo é ZERO. A aguada continua a correr além dos 7 s medidos.
- **Gargalo real da aguada:** a simulação fora do fio da tela. Passos: 1080p 40/s (nominal), 4K Size 240 só 22–25/s. A óptica mede 15 ms por imagem, e ainda assim saem 0–1 imagens/s a 4K com o worker a 97 %. **Hipótese, não medida:** o compositor espera na fila do mesmo pool rayon que a simulação satura.
- **Razões:** 1×1 e 1×2 sem diferença material em nenhuma fase: o fluxo não é o custo.
- **Relevo (Real Oil):** dobra o Dabber no traço; custa 1 imagem de 222–1252 ms ao assentar.
- ⚠️ **Achado à parte:** Flat Bristle no traço a Size 240 em 4K leva ~1 s por quadro. É óleo, não aguada; o dono não perguntou.

Não medido: área por fase é o `take_preview_upload_bbox`, que pode faltar (conta a tela inteira quando `None`). Envio à tela = só o `take_preview_arc`; a subida à placa não está aqui.

## Item 4: MEDIDO, à espera do dono (⛔ nada mudou no produto)
`Size` é o RAIO (`set_brush_size_px` → `radius_px`): Size 60 = D 120. **Tile de cerdas tem período 128**: coincide com o limiar; correlação, não causa provada.
Folhas: [hoje por tamanho](material_2026-11-07/folha_A_hoje_por_tamanho.png) · [variantes a 200](material_2026-11-07/folha_B_variantes_size200.png) · [variantes a 120](material_2026-11-07/folha_B_variantes_size120.png). Sonda `sonda_folha_de_tamanhos`: `F` hoje, `T` textura, `B` fios, `E` relevo (ref D = 120, γ = 1), `G` grade da água = `round(Size/60)`.

Detalhe relativo ao Size 60 (1 = igual), ao reduzir à mesma altura, a Size 200:
| | hoje | T | B | E | G | TBE | tudo |
|---|---|---|---|---|---|---|---|
| Flat Bristle | 2,27 | 2,26 | 0,61 | 2,27 | 0,96 | 0,87 | 0,55 |
| Thick Impasto | 2,16 | 1,93 | 1,82 | 2,16 | 0,35 | 1,54 | 0,37 |
| Knife Textured | 0,87 | 0,87 | 0,91 | 0,87 | 0,96 | 0,91 | 0,82 |
| Fan Brush | 1,39 | 1,12 | 1,42 | 1,39 | 0,65 | 1,17 | 0,53 |
| Dabber 4 | 1,24 | 1,24 | 1,11 | 1,24 | 0,96 | 1,15 | 0,88 |
| Wet Wash | 1,00 | 1,00 | 1,00 | 1,00 | 1,01 | 1,00 | 1,01 |
| Wet Round | 0,77 | 0,80 | 0,77 | 0,77 | 0,78 | 0,80 | 0,83 |

- **Quebra de verdade:** Thick Impasto e, menos, Flat Bristle e Fan. Aguadas não quebram. Abaixo de 60 também muda (detalhe 0,4–0,7 a Size 20), mas essa é a cara aprovada.
- **Grandeza que quebra:** detalhe preso a px, resolvido por célula. `G` é a que mais aproxima o detalhe, e **custa 3× menos** (Flat a 200: 19,1 → 6,8 s, intercalado). Mas a Thick Impasto com `G` fica lisa demais (0,35), sem o empaste do 60; e `G` em `round` salta em Size 90. O relevo `E` não muda nada visível. `T` estria o Wet Round, recusar.
- **Nenhuma variante reproduz a cara do 60.** Por isso a lei NÃO foi proposta como pronta: pergunta ao dono na mensagem final. Lei candidata, a confirmar depois da escolha: grade da água contínua em função do Size, só acima de 60 (idêntica ao byte até 60 por construção: razão 1). `SUAVE_RAIO_MAX_PX = 6` não tem override em execução e não foi separado dentro de `G`.

## Vereditos do dono (07/11)
Smoke A, B e C: **OK**. Item 4: **as duas leis no painel, «agua/alisar» por padrão**, acima de Size 60; até 60 ao byte.

⛔ **Trava do item 4 (lida no código, não testada):** a sonda pôs `grid_ratio` por tela, com um só tamanho. No produto a razão é da SESSÃO: `set_wet_grid_ratio` chama `wetpaint_end_session` (`wetpaint_settings.rs:523`) e o acordar recusa grade diferente (`adormecida.rs:351`). Trocar o Size entre traços acabaria a tinta viva, contra a decisão de 06/11. Não ligue `grid_ratio` ao Size.

## Item 4, protótipo: MEDIDO e ⛔ RECUSADO pelo dono em 07/11 («deixar como está»; o código saiu, as tabelas ficam)
Sonda `sonda_prototipo_do_carimbo`, `wetpaint/carimbo_grosso.rs` (só `cfg(test)`), `b = Size ÷ 60` fracionário, grade da sessão 1. Variantes: `C` carimbo em blocos (silhueta, grão/tapete, pente) · `O` a tinta pousada em blocos (massa, cor por massa, relevo), UMA vez ao fim do traço, sem a água depois · `CO` as duas · `GS` a grade sem o `Suave`. Folhas: [zoom 100 %](material_2026-11-07/folha_C_zoom100_size200.png) · [traço inteiro](material_2026-11-07/folha_C_traco_inteiro.png). Imagens em `target/lote/proto` (não versionadas).

Detalhe relativo ao Size 60, a Size 200 (régua NOVA: gradiente médio na tinta, `target/lote/proto.log`; os números não casam com a tabela de cima, só comparam entre si):
| | hoje | `G` | `C` | `O` | `CO` |
|---|---|---|---|---|---|
| Flat Bristle | 3,01 | 1,09 | 2,61 | 1,96 | 2,07 |
| Thick Impasto | 2,29 | 0,28 | 1,54 | 1,44 | 1,16 |
| Knife Textured | 1,03 | 1,02 | 1,04 | 0,77 | 0,77 |
| Fan Brush | 1,15 | 0,67 | 0,95 | 0,93 | 0,79 |
| Dabber 4 | 1,12 | 0,70 | 0,95 | 0,65 | 0,68 |
| Wet Wash | 1,06 | 1,05 | 1,05 | 1,04 | 1,04 |
| Wet Round | 0,75 | 0,74 | 0,77 | 0,63 | 0,64 |

Tempo do traço a 200, mínimo de 3 intercaladas, carga ~12 (`hoje / G / C`): Flat 19,6 / 6,9 / 68,8 s · Thick 4,8 / 1,2 / 5,3 s · Wet Round 1,42 / 0,26 / 1,50 s.

- **O que dá a cara da `G` NÃO é o carimbo nem o alisar:** `C` muda pouco e nenhum canal sozinho basta; `GS` igual a `G` a olho (0–59 mil px de 1,9 M, detalhe igual). Com a grade fixa, `O` e `CO` não reproduzem a `G` no Flat, Fan e Dabber. Falta a água a correr em células grossas, que a grade fixa não exprime.
- **Com a grade fixa o ganho que sobra é ZERO:** vem todo da grade. `C` é mais lento; no Flat 3,5×, por uma tabela por dab. Otimizável, não medido.
- **Saltos:** `G` só tem razões inteiras e ondula no Flat (90→200: 0,70 · 0,80 · 0,99 · 0,71 · 0,96 · 1,09). `C` e `O`, de `b` fracionário, sobem sem degrau.
- **Só o zoom 100 % mostra, e a folha reduzida escondeu:** `G` borra o Thick, põe pontos (furos) no Dabber e um xadrez no Knife. `CO` guarda o caráter e fica o mais perto do 60 no Thick.
- **Controlos:** F = C ao byte no 60 (por construção). `O` foi um único passe no fim: não prova o traço ao vivo, nem a água depois. `C` não cobre a cor por célula (`cor_arg`).

## A escovada do Dabber ao crescer (dono 07/11): MEDIDO

Sondas `sonda_a_escovada_do_dabber` (cada knob fora do padrão de volta ao padrão, Size 51 e 200) e `sonda_o_relevo_gravado_por_tamanho` (`Grid::altura` no miolo, traço recto). Foto: [folha D](material_2026-11-07/folha_D_dabber_relevo_zoom100.png). A régua de coerência (tensor de estrutura, janela 9 px) só compara dentro de si e sobe também quando a tinta alisa: ler junto com o gradiente.

- **Causa:** o relevo dos fios, `bristleHeight` 0,75 (`Tuning › Bristles › Bristle relief`). Sem ele o Size 200 fica granulado: coerência 0,471 → 0,349 (Size 51: 0,386 → 0,322). Os fios têm largura fixa (`bristleLaneWidth` 1,5 px) e o Shape não entra: os sliders do Shape não movem a escovada.
- **O que cresce com o Size é o COMPRIMENTO, não a altura:** ÷ declive através pelo declive ao longo: 6,2 · 8,1 · 14,0 · 22,5 (Size 25 · 51 · 100 · 200), com desvio 0,32–0,39. É a queda macia da ponta: a 200, Hardness 0,2 · 0,6 · 0,8 · 1 dá 29,3 · 22,5 · 16,9 · 3,6 (a Hardness do painel prende em 1; o «2» das sondas é igual a 1). Como isto aciona o comprimento é inferência (a fronteira macia ∝ raio penteia mais células iguais), **não medido por dentro**.

| ⛔ Recusas MEDIDAS | Resultado a 200 |
|---|---|
| Hardness 1 | risca curta, mas vira tijolinho, não granulado |
| pente com cobertura ×4–×16 só nele | comprimento 14,8–10,2: tijolinho |
| pente limitado ao comprimento (L60) | comprimento 8,7, a foto continua riscada |
| Spacing ÷2, ÷4, ÷8 | coerência 0,466–0,477 |
| jitter no mesmo px do Size 51 (0,0765) | coerência 0,456, e jitter 0 são carris (0,72) |
| fios que crescem com o pincel | fios mais largos, ainda riscas |

**O que dá a cara:** `Bristle relief` 0,25 reduz, 0,12 fica granulado (coerência 0,404 · 0,364), com gradiente 12,8 contra 16,3 do Size 51: um pouco mais plano. Tempo do traço igual. Candidato, se o dono aprovar depois de provar: o relevo do Dabber a cair com o tamanho só acima de 60, com o abaixo ao byte. Não construído.

## Bristle Size até 16 (dono 07/11): FEITO, sem commit

`Knob::BristleSize.max` 4 → 16. ⛔ O Shape guarda `u` normalizado e os presets gravaram `u = 1` = 4: **a escala não foi refeita** (`valor_em_um`, `Escala::um`); a faixa cresce acima de 1 (topo 4,158), e os setters prendem em `Escala::u_max`, sem migração. Mapa antigo do motor prende em 4 em `wet_preset_aplicar`: sem isso, Fan Brush e Express Oil (receita 7) mudam, e os dois alarmes apanharam (mutação natural: Fan 14,74 → 20,14). Custo do ladrilho, mínimo de 5, por fio e por mudança: 0,21 ms (Size 2, Count 2000) · 3,73 ms (16, 2000) · 7,29 ms (16, 4000). Portões: `o_bristle_size_vai_a_16_sem_mudar_o_que_esta_gravado`, `o_bristle_size_do_painel_chega_a_16`, `o_bristle_size_16_atravessa_o_ficheiro`; os dois alarmes de pincel verdes, nada regravado. Falta o fecho (fmt/clippy 1×, suíte na integração).

## Bristle Combing no cartão Shape (dono 07/11): FEITO, sem commit

Ordens de 13/10 e 01/11 incompletas: o plano 27 §1.6 só levou os 4 sliders do tapete. Agora `bristleLaneWidth`, `bristleHeight`, `bristleTipRagged`, `bristleLaneSizeExp` são `KnobGroup::Hidden`, num grupo próprio abaixo do tapete (`cards_penteado.rs`, `ids::CERDAS_DO_PENTEADO`). Os outros 17 ficam no Tuning (dono: «apenas os 4»). Formato gravado igual (chave do knob); ids do Tuning mantidos e resolvidos por `wet_tuning_id_map` (macros). Três leis novas: `PenteadoForaDoMolhado` (meio que não lê), `PenteadoSemFios`, `PenteadoSemLuz` (o relevo só acende com emboss ou Real Oil: quem apontou foi o censo). A prévia do Shape continua só o tapete. Foto: `bristle_combing_foto`.

## Clip Studio: estudo, 6 pincéis novos e a mistura opaca (07/11): FEITO, sem commit

**Ordem do dono (07/11):** estudar os pincéis do Clip Studio e aumentar o nosso número; «PLANEJE, ESCOLHA E SIGA»; não estragar o que existe (sobretudo o Wet Paint); uma lei nova de mistura de cor entra como opção EXTRA, nunca no lugar de outra. Os 4 valores de cerdas do dono (width 3, relief 0, ragged 0,64, follow 1,5) ficam só como referência para uma futura missão autónoma noturna: presets por tamanho no menu Size, para o pincel parecer o mesmo em todos os tamanhos. **Nenhum pincel de fábrica foi regravado por eles.**

### Estudo
[`ESTUDO_CLIP_STUDIO_PINCEIS_2026-11-07.md`](../ESTUDO_CLIP_STUDIO_PINCEIS_2026-11-07.md): leitura só-leitura do `EditImageTool.todb` (SQLite) do CSP 1.13.2; nenhuma ponta ou textura da Celsys copiada; o app não foi corrido. ⚠️ O mapeamento dos bits de efetor P/T/V/R é DEDUZIDO, não confirmado.

### Seis pincéis de fábrica novos (78 → 84; Digital 21 → 26, Watercolor 1 → 2)
- Receitas em `crates/ph2d-tool-painter/tests/it/familias/{canetas,aerografo,lapis,pastel_carvao,aguarela}.rs`; ficheiros em `assets/brushes/Digital/{Ink/G-Pen,Ink/Turnip,Airbrush/Soft,Pencil/Sketch,Pencil/Charcoal}` e `assets/brushes/Watercolor/Watercolor/Round`.
- Gerados com `PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO=... gera_os_presets_de_fabrica`.
- Correção: as receitas traziam `"area": false` em `lei`, que o pincel não serializa; removido.
- Grão procedural + ponta redonda. **A desenhar:** pontas próprias de grafite/carvão e papéis rugoso/desenho/areia.
- O motor não exprime tilt→size, nem pressão/velocidade→opacidade no Watercolor.
- Folha: `material_2026-11-07/seis_pinceis_novos.png`.
- ⚠️ CLAUDE.md §5 diz «78 pincéis de fábrica»: vira 84 na integração (contado; não editado agora).

### Lei de mistura opaca (opção EXTRA, Off por omissão)
- Núcleo: `crates/ph2d-painter-brush/src/mistura_opaca.rs` (+`_tests.rs`). `BrushSpec.mistura_opaca` = {modo Off|Blend|RunningColor, paint_amount, paint_density, color_stretch, amount_pressure, density_pressure}.
- Por carimbo: amostra da tela ponderada pelo falloff; reservatório `r = lerp(amostra, r, stretch)`; `m = lerp(r, cor, amount)`; deposita com `density × falloff × coverage × flow`. Blend mantém `m` no reservatório; Running mantém a cor da tela.
- `DE_FABRICA.paint_amount` mudou 1,0 → 0,5: com 1,0 Blend/Running pintavam igual a Off e o censo `o_censo_dos_controlos_so_encolhe` reprovou.
- Preset JSON grava o campo só quando ligado (`wetpaint_preset/{campos.rs (CAMPOS_FORA 80→81), json.rs, json/meio.rs}`).
- Fiação: `crates/ph2d-tool-painter/src/tool/paint/stamp_route/mistura.rs` + ramo em `stamp_dabs_inner` (só ligado, modo Paint, sem apagar). `stamp_device.rs::eligible()` recusa mixer ligado (só CPU). Reset ao baixar a caneta (`stroke_lifecycle.rs`) e a cada re-carimbo de forma inteira (`stamp_preview.rs`).
- Pressão = último evento da caneta por lote (o `Dab` não tem pressão; 173 construtores). Simetria: um reservatório por espelho. Tiling: as cópias partilham o estado inicial. Alpha lock tratado dentro da rota. Undo via `fork_canvas`. Testes `mistura_opaca_ferramenta_tests.rs`.
- Painel: grupo «Color mixing» no fim do cartão Basics (cartões e clássico), `cards_mistura.rs`; ids `PAINTER_BRUSH_MIX_MODE/_MODES[3]/_FIELDS[5]` em `ids/painter.rs`; setters `mistura_settings.rs`; `Dependente::MisturaDesligada` (49 entradas); portas de macro; 11 chaves i18n em `ph2d-i18n/src/painter_layers.rs`. Só aparece para pincel Digital sem flags watercolor/Wet (escondido nos outros, como Fine Thread); sliders esmaecidos em Off. Testes `tests/it/color_mixing.rs`.
- Fotos em `material_2026-11-07/`: `painel_mistura_{off,blend}.png`, `mistura_opaca_{off,blend,running,comparacao,ferramenta}.png`.

### Verde (por nome)
Painel inteiro 454/455 e a 1 falha curada · censo · `color_mixing` · ferramenta `mistura` (50) · pincel `mistura_opaca` (7) · os 2 alarmes de pincel · `the_factory_presets_are_counted` · `cada_preset_de_fabrica_e_a_sua_receita`. Suíte completa NÃO correu.

### ABERTO
1. Blend/Running apanham o PRÓPRIO depósito fresco (carimbos sobrepostos): Blend vai à cor pura do pincel depressa e Running nunca apanha o azul. Cura candidata: amostrar um snapshot da camada de antes do traço. Espera o olho do dono no smoke.
2. Shape/Grain não moldam o carimbo do mixer (só falloff; o código da pegada é privado na crate do pincel).
3. Sem testes de re-carimbo/simetria/tiling com mixer ligado.
4. Lote 2 de presets (Round mixing, Gouache, Dry/Thin Gouache, Oil paint, Paint and apply, Crayon, Pastel, Chalk) não feito: depois do smoke do dono.
5. Lote 3 (versões Wet Paint) só se o dono quiser.
6. Pressão é peso linear, sem curvas.
7. fmt/clippy das crates tocadas ainda não correram (passo de fecho).

## Color mixing → Pigment mix (07–08/11): o que foi feito e o que o dono quer

> ⛔ **SUPERADA** pela «Missão autónoma de 08/11 (noite)» no topo: Pigment mix virou a lei Wet blend. Fica só como história.

Tudo sem commit. Esta secção SUPERA «Lei de mistura opaca» acima (Blend/Running já não existem como modos).

### 1. Blend/Running (lei opaca por carimbo): corrigidos, depois substituídos
Smoke do dono: «sem efeito». Causas achadas e curadas:
- Auto-apanha: a amostra passa a vir dos ladrilhos de ANTES da caneta descer (`Ladrilhos`, `stamp_route/mistura.rs`). Fecha o ABERTO 1 acima.
- Taxas por distância (`DIAMETROS_REF`, `mistura_opaca.rs`), não por carimbo.
- A ponta dual vazava para o mixer: `dual.rs:229` força Off.
- G-Pen, costura: o re-pintar da cauda (taper) restaura o reservatório por lote (`Lote.mistura`, `traco_plano/cauda.rs`); anéis: `perfil_da_cor` (⚠️ nome dado pela janela principal, não achado por grep — conferir).
- Grão/forma agora moldam o depósito (máscara de `stamp_dab_textured_masked` + teto de cobertura `Pegada`): fecha o ABERTO 2.
- Camada vazia não lava a cor; bug do caminho Accumulate ligado (alpha 10) curado.
- Comparação com o Composite Brush: [`COMPARACAO_COMPOSITE_VS_COLOR_MIXING_2026-11-07.md`](../COMPARACAO_COMPOSITE_VS_COLOR_MIXING_2026-11-07.md), folhas `material_2026-11-07/comparacao_composite_vs_mixing_*`, sonda `crates/ph2d-tool-painter/tests/it/sonda_composite_vs_mixing.rs`. Defeito do Composite NOTADO, não aberto: G-Pen com riscos brancos e cauda redonda.

### 2. Pigment mix (substitui Blend/Running)
- O dono viu o Watercolor Basic com Mixing=Pigment (azul+amarelo=verde) e achou muito superior. Estudo: [`ESTUDO_MISTURA_DO_WATERCOLOR_PARA_O_DIGITAL_2026-11-07.md`](../ESTUDO_MISTURA_DO_WATERCOLOR_PARA_O_DIGITAL_2026-11-07.md).
- `ModoMisturaOpaca::{Off, PigmentMix}` (`ph2d-painter-brush/src/mistura_opaca.rs`); presets/macros com Blend/Running carregam como PigmentMix. Knobs: Amount, Pickup (novo), Pull, Density + 2 pesos de pressão.
- Lei: por pixel, mistura ESPECTRAL do pigmento da tela sob a caneta com a cor do pincel sujada pelo reservatório; branco/vazio NÃO é pigmento.
- Testes `tests/it/a_mistura_pigmento_digital.rs`, com a **prova por hash do Watercolor Basic** (gravada ANTES das edições, verde): Watercolor Basic segue byte-idêntico (regra permanente do dono).
- Custo por traço (G-Pen): Off 17,95 ms · Pigment mix 24,13 ms · Watercolor Basic 258,98 ms.
- Fotos: `material_2026-11-07/pigment_mix_digital.png`, `painel_pigment_mix.png`.
- Conhecidos: azul sobre vermelho dá bordô quase preto (lei espectral, amount 0,5, saturado); no painel, Pickup, Pull e Amount-by-pressure mostram todos a letra «P»; `painel_mistura_blend.png` está obsoleta.

### 3. Veredito do dono em 08/11: ARTIFICIAL
«O que eu gostaria de ter trazido do watercolor era o modo como ele mescla as tintas, o smear, o blur, etc. não apenas a cor.» A foto dele: amarelo sobre azul = placas empilhadas de borda dura. Ele quer a mescla ESPACIAL do Watercolor (smear, blur, sangrar, molhado-sobre-molhado) na tinta opaca do Digital, no lugar do Pigment mix (que é só cor). Um agente de estudo corre: `docs/Painter/ESTUDO_MESCLA_ESPACIAL_DO_WATERCOLOR_2026-11-08.md` + folha de ablação `ablacao_watercolor_basic.png` (resultado pendente ao escrever; os ficheiros NÃO existiam no disco nesse momento). Ler o estudo ANTES de desenhar.

### 4. Outras coisas de hoje (todas sem commit)
- 6 pincéis CSP novos (84 no total), estudo `ESTUDO_CLIP_STUDIO_PINCEIS_2026-11-07.md`. Os 4 valores de cerdas do dono ficam só de referência para a missão noturna de presets por tamanho.
- Lote 2 (pincéis de tinta grossa à CSP) espera a lei de mistura que o dono aceitar.

## Próximos, por ordem
0. ⛔ SUPERADO pela missão de 08/11 (noite), no topo (Wet blend + 16 pincéis CSP; resta o smoke do dono). Texto antigo: ⏳ Mescla espacial no Digital (secção acima, §3): ler o estudo de 08/11, desenhar a lei que traz smear/blur/bleed do Watercolor ao opaco (Pigment mix sai ou vira só a parte da cor), manter Watercolor Basic byte-idêntico (hash), medir custo, fotografar lado a lado com a foto do dono ANTES do smoke. Depois do veredito dele: lote 2. Fecho: fmt/clippy das crates tocadas 1×. Smoke dos 6 pincéis CSP segue pendente. Missão noturna futura: presets por tamanho no menu Size.
1. ⏳ Smoke do dono do grupo Bristle Combing. Abertos, decisão dele: que pincéis a óleo ganham width 3, relief 0, ragged 0,64, follow 1,5 (foto antes de regravar alarme); depois levar os fios ao Digital e ao Impasto, um por vez, Watercolor por último. O item 2 (aguada) segue parado; base: [estudo de custo](../ESTUDO_CUSTO_WET_PAINT_2026-11-07.md).
2. Item 4, depois da foto aprovada: o controle no painel em cartões (seletor, inglês, HR-15, todas as pontas e teste de costura), padrão «agua/alisar»; definir onde mora (por pincel ou da ferramenta), macro e preset sem mudar os de fábrica; «tex+fios» sem as estrias do Wet Round. Alarmes: só regravam com aprovação.
3. Item 2: até 3 protótipos com ganho medido e pixels iguais. Primeiro: separar o compositor do pool da simulação (medir a hipótese; a imagem não muda). Respeitar J13 §5 e a recusa de compor por linhas.
4. ABERTO 1 do handoff de 07/11 (corrigido acima), se o dono quiser.

Fora deste pedido, seguem pendentes: redução de memória do 1.º traço, «manter a tinta viva» ao salvar, `PROXIMA_FASE_2026-11-06.md`.

## Prova do fecho da onda
fmt ✓ · clippy `ph2d-tool-painter`, `ph2d-wet-paint` ✓ · por nome `undo_matriz_tests redo_relevo_tests foto_em_tiles` 16 verdes · família do undo 53 verdes antes do lint · alarmes 2/2, **nenhum pincel mudou** · suíte completa NÃO correu. Binário: `Finished smoke in 0.38s`, `target/smoke/ph2d-host-desktop`.
