# Continuação — line/PainterWatercolor, onda de 07/11: pânico do undo, refazer, lote de medição

Base: `main` `652b506b58` (reaberta por `reset --keep`; `git cherry` = 0 `+`, 34 `-`; ponto de volta `backup/line-PainterWatercolor-pre-reabertura-2026-11-07`). Commits: `c887156cbb` (item 1), `3fc47dc81d` (item 3), e o desta onda (sondas e este doc). Não integrada.

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

## Próximos, por ordem
1. Item 4, protótipo por override, sem mudar produto: separar DENTRO de `G` o que dá a cara (o carimbo do pincel em resolução menor, a água, o alisar `SUAVE_RAIO_MAX_PX`) e achar a mesma cara com a grade da sessão fixa, a tinta de outros tamanhos viva. Medir o que sobra do ganho de 3–4×. Corrigir os saltos (Size 90, 150). Foto antes × depois, com zoom 100 %, e PARE.
2. Item 4, depois da foto aprovada: o controle no painel em cartões (seletor, inglês, HR-15, todas as pontas e teste de costura), padrão «agua/alisar»; definir onde mora (por pincel ou da ferramenta), macro e preset sem mudar os de fábrica; «tex+fios» sem as estrias do Wet Round. Alarmes: só regravam com aprovação.
3. Item 2: até 3 protótipos com ganho medido e pixels iguais. Primeiro: separar o compositor do pool da simulação (medir a hipótese; a imagem não muda). Respeitar J13 §5 e a recusa de compor por linhas.
4. ABERTO 1 do handoff de 07/11 (corrigido acima), se o dono quiser.

Fora deste pedido, seguem pendentes: redução de memória do 1.º traço, «manter a tinta viva» ao salvar, `PROXIMA_FASE_2026-11-06.md`.

## Prova do fecho da onda
fmt ✓ · clippy `ph2d-tool-painter`, `ph2d-wet-paint` ✓ · por nome `undo_matriz_tests redo_relevo_tests foto_em_tiles` 16 verdes · família do undo 53 verdes antes do lint · alarmes 2/2, **nenhum pincel mudou** · suíte completa NÃO correu. Binário: `Finished smoke in 0.38s`, `target/smoke/ph2d-host-desktop`.
