# A tinta viva sobrevive (decisões do dono, 06/11)

Estado: **plano aprovado no essencial, nada construído.** Commit de partida: `9aa112caec` (Relief Eraser).

## Defeito reproduzido
Dabber → trocar para Digital/Watercolor/Impasto (com ou sem traço) → voltar ao Wet → Relief Eraser: corte seco em vez da borracha que amassa. Sonda sobre `com_oleo()` + `apaga()` de `wetpaint/borracha_de_fabrica_tests.rs`:

| Caso | sessão ao aplicar | px mudados | soma \|Δ\| | |
|---|---|---|---|---|
| vivo | sim | 2 422 | 67 398 | a referência |
| Digital ou Watercolor, sem traço | não | 1 036 | 196 654 | difere do vivo em 2 483 px |
| Digital, Watercolor ou Impasto, traço longe | não | 1 036 | 196 654 | apagamento igual ao «sem traço» |

Sair do Wet muda 0 px na tela: o relevo que se vê continua, assado em RGBA.

**Mecanismo** (leitura de código, sem correr caminho a caminho): `set_paint_media` → `set_wetpaint_armed(false)` → `mode_switch.rs:117` `wetpaint_end_session` → `session = None` (`authored_actions.rs:227`). A grade (`Grid.altura`, `relevo`, `espessura`, Óleo Real) só existe na `WetSession`. Ao voltar, `ensure_wet_session` nasce com `base = canvas_rgba` e grade vazia. Sem sessão, `wet_owns_the_dabs` (`ciclo.rs:24`) devolve a Borracha ao caminho plano.
Outros gatilhos de morte: `wetpaint_guard` (`ciclo.rs:255`, o `Arc` do canvas mudou: fill, outra ferramenta; troca de camada não confirmada), undo (`layers/undo.rs:182`), razão de grade/fluxo (`wetpaint_settings.rs:531,550`), blend (`brush_core_settings.rs:292`), macros. Abrir projeto: `persist.rs` não guarda estado Wet.

## Decisões do dono
1. **Manter a tinta viva** ao trocar de modo e de ferramenta, sem reduzir a velocidade dos outros modos.
2. Traço de outro modo por cima do óleo: **a Borracha leva os dois juntos**.
3. Ao **salvar o projeto, uma OPÇÃO** de manter a tinta viva (não sempre).
4. **Borracha inteligente:** age de verdade na tinta viva e **imita o comportamento certo na tinta morta** (arquivo antigo ou salvo sem a opção).

## Medições de custo (Dabber, 1 camada; zstd `-T0`, mínimo de 3, load ~10; `tinta` e `pigmento` vazias na lei antiga)

| | Full HD | 4096×2560 |
|---|---|---|
| RAM da grade viva | 214 MB | 1 081 MB |
| Imagem crua no `.ph2dproj` (postcard, sem compressão) | 7,9 MB | 40 MB |
| zstd, um traço | 1,3 MB | 5,4 MB |
| zstd, metade | 21,3 MB | 108,5 MB |
| zstd, camada cheia | 40,6 MB | 205,3 MB |
| zstd comprimir / descomprimir, cheia | 0,03 / 0,04 s | 0,12 / 0,19 s |
| Juntar os planos (flat_map, perfil smoke) | ~0,21 s | ~1,1–1,3 s |

Nível 1 e 3 do zstd dão o mesmo tamanho: use o 1. Não medido: gravar em disco e restaurar a grade.

## Pesquisa: outros apps
- **Rebelle** ([manual](https://escapemotions.com/products/rebelle/manual/8.2/creating-opening-saving/): «REB contains all the information about the artwork and the layers»; nada sobre o molhado). O `.reb` é um ZIP com planos por camada (`layer0_structure.dat`, `layer0_flow.dat`, `canvas.png`). Lista de 4 de 21 arquivos locais: só o `AutoSave-2026-11-04 22-46-28.reb` traz `layer0_mida.dat`, `layer0_f_01..12.dat` e `layer0_drops.dat` (~7,8 MB crus, ~0,36 MB no ZIP, arquivo inteiro 5,7 MB, com 3,9 MB de papel no irmão medido). **Inferência:** é estado molhado, só presente quando há água. Não confirmado: o que cada plano é, a dimensão daquele autosave, e se a água continua ao reabrir. Sem teste: SAVE/LOAD não correm por WebSocket e o trial acaba ~06/11; `RebelleArtwork.reb` é 1920×1080. [Fórum](https://www.escapemotions.com/community/forum/t/442/rebelle-3-crashed): `.reb` com muitas camadas fica grande; fundir reduziu 30 %.
- **Corel Painter** ([ajuda](https://product.corel.com/help/Painter/540232071/Main/EN/Doc/Corel-Painter-Importing-and-exporting-layers.html): RIFF guarda camadas «live»; [guia do Painter 12](https://www.painterartist.com/static/ptr/img/content/community/tutorials/p12/Painter12_Tour_PS_Users.pdf): camadas de Watercolor exigem RIFF «to retain the live "wet" capabilities»; [escolha de formato](https://product.corel.com/help/Painter/540235477/707000/EN/Doc/Choosing_a_file_format.html): compressão sem perda, o «Uncompressed» desligado). [Addon de terceiro](https://blenderartists.org/t/addon-import-corel-painter-riff-files-for-impasto-depth-3d-texture/647557): Watercolor e Liquid Ink levam imagens extra de umidade. Não testado; não instalado aqui.

⛔ **Correções desta tabela:** os 40,6 MB acima não contam as fotografias do início do traço (`sett_do_traco`, `oleo_antes`, `chao`) nem o `paper`; a contagem completa dá **59,9 MB** (Dabber cheio, Full HD, 140 B/célula). E o «~3,8 B/px» do Rebelle estava errado: aquele autosave é 1000×640, 17 B/célula crus, 10,4 B por pixel PINTADO no ZIP.

## O «segredo» do Rebelle: medido em 21 `.reb` locais (só dados, sem abrir o executável)
- **Formato:** plano vazio = `ZERO` + contagem (8 bytes); não vazio = `BBOX` x0 y0 x1 y1 W H + tipo (`UCHA`, `UI_2`, `FL_2`) + linhas densas. `comprimento = 36 + bw·bh·elem` em 76 de 76. Sem tiles nem RLE. Grade 1:1 (W+8 × H+8): **não há resolução reduzida**. O tamanho segue a área pintada, não a tela.
- **Quem gasta** (5 695 710 B, 11-04 22:46): papel 68 %, preview e assets 23 %, **tinta 8,4 %**. Planos: densos 11,1 MB → caixa 8,2 MB → não-zeros 0,60 MB → ZIP 0,37 MB. A tinta cobre só 5–23 % da tela.
- **B por pixel pintado:** molhado 6,7–10,4; seco ~1,5. **Extrapolado, não medido:** tela inteira molhada em Full HD 14–21 MB; 4K 63–87 MB.
- `FL_2` = float16 big-endian. As cores (`f_05..07`) são máscara + constante quando a cor é uma só.
- **Só 4 de 21 têm planos molhados, todos autosaves.** O único `.reb` que não é autosave não os tem. **Não determinado:** se um Save manual os guarda e se a água volta ao reabrir (trial expirou; Painter parado no EULA por aceitar, nada acionado; dono desistira do Corel em 28/10).
- Corel: nada público sobre o formato ou tamanhos do molhado.

## Plano a plano: o nosso (Full HD, 1 camada, zstd -1, MB, sem a imagem composta)
| Estado | Água | Hoje | Hoje, bytes separados | Sem o derivável | 24 bits massa, resto half* | Cor e relevo u8* |
|---|---|---|---|---|---|---|
| Dabber | não | 59,9 | 51,4 | **37,0** | 17,6 | 10,5 |
| Wet Round, fim da cena | sim | 56,0 | 50,4 | 47,5 | 28,9 | 28,9 |
| Wet Round, +10 s | sim | 95,3 | 79,3 | 76,5 | 38,4 | 32,4 |
| Wet Round, Dry canvas | não | 49,6 | 24,2 | **21,4** | 9,9 | 6,3 |
| Misto (Dabber + Wet Round) | sim | 107,3 | 93,1 | 90,3 | 51,2 | 42,4 |
| Misto, Dry canvas | não | 83,2 | 62,6 | 47,0 | 21,5 | 14,7 |
| Knife Smooth 3 | não | 53,7 | 42,7 | 27,8 | 13,1 | 7,9 |

\* Nunca idênticos ao bit. Pixels mudados (de 518 400 a 960×540) / maior Δ de canal: 24 bits + half: Dabber 27k/2, Misto 25–45k/42; cor e relevo u8: Dabber 278k/3, Misto 240–252k/69.
- **Derivável, testado sem mudar pixel:** `paper` (pela porta `reconcilia_o_papel`, ⛔ não por `Engine::rebake_paper`: muda até 197) em qualquer estado; `flow_x/y`, `active`/faixas e as fotografias só **sem água**. Com água as fotografias são reprovadas (240–265k px / 184), e `flow`/`active` mudam pixel.
- ⭐ **Cura de graça:** `grid/canvas_ops.rs:71-73` zera `susp` e deixa `susp_rgb`: depois de Dry canvas, 19,4 MB de cor morta (Wet Round 49,6 → 24,2).
- ⛔ **Recusas medidas:** massa half (Δ 69), massa log8 (Δ 146), pincel u8 (Δ 85), meia resolução de velocidade (Δ 96) e de relevo (Δ 164). A massa exige ≥ 16 bits de mantissa. Planos «constantes do pincel» deixam de o ser quando dois pincéis se misturam.
- Esperar não encolhe: Wet Round +10 s cresce de 56 para 95 MB; só o Dry canvas tira a água.
- **Não verificado:** nenhum restauro real correu (mutação no mesmo processo; `Sim.frame`, trilho e o passo em voo também iriam no arquivo). O combinado do derivável não foi medido com a porta certa do papel. Pixel a 540p, tamanho a 1080p. O olho do dono não viu.

**Lição:** não há truque. O Rebelle ganha por cobertura pequena, caixa + zeros, 17 B/célula (float16) e só guardar água quando há água. **Ordem adotada:** sem perda primeiro (cor morta, derivável, caixa e máscara, bytes separados); quantizar só com a foto aprovada pelo dono.

## Plano (um bloco, §0.10)
1. **Tinta adormecida:** sair do Wet (modo, ferramenta) não larga a sessão: adormece-a. Zero tique e zero custo nos outros modos, comprimida em RAM (zstd 1). Medir o custo ANTES de ligar: traço Digital/Watercolor com × sem sessão adormecida, intercalado no mesmo processo ([MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md)). **Kill-criterion:** diferença acima do ruído = não entrega.
2. **Volta ao Wet:** a nova `base` = a tela de agora, com os traços de outros modos, e a grade acorda. Sobreposição: onde o outro modo pintou, a Borracha leva os dois (decisão 2). Rever `wetpaint_guard`, undo vivo (`undo_vivo.rs`) e a marca do undo; o `Arc` do canvas muda.
3. **Por camada:** confirmar se trocar de camada mata a sessão e tratar no mesmo desenho, com orçamento de RAM.
4. **Borracha inteligente:** sem tinta viva, imitar. Referência a medir: `vivo` da tabela (orla macia, Δ baixa, amassa). Candidato: uma sessão que nasce da própria imagem, sobre o traço. Provar pelo olho do dono: foto vivo × imitado.
5. **Opção ao salvar:** campo novo no `PaintedDocument`, **no FIM** (postcard é posicional), degrau de `PROJECT_SCHEMA` por `schema-recount.py`, e migração. Arquivo antigo ou sem a opção abre com a tinta morta e a Borracha imita. Ida e volta salva: a tinta reaberta igual ao bit.
6. Reprovações a cobrir: o preço da compressão, 4K com várias camadas, redo (item 0 da [próxima fase](PROXIMA_FASE_2026-11-06.md); o mesmo código).

**Fila depois disto:** itens 0–3 de [PROXIMA_FASE_2026-11-06.md](PROXIMA_FASE_2026-11-06.md). A foto 5 refeita pelo dono ainda não chegou (sem anexo; pasta sem alteração).
