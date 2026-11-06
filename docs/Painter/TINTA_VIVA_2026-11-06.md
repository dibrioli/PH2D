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

**Lição para o nosso:** os dois guardam o molhado por camada, no formato nativo. A nossa grade tem ~108 B por célula em `f32`; a do Rebelle parece muito menor (~3,8 B/px, se o autosave for 1920×1080). Antes de fixar o formato, MEDIR quantos planos são deriváveis ou quantizáveis (a tela cheia tem de cair de 41 MB; alvo a medir, sem número inventado).

## Plano (um bloco, §0.10)
1. **Tinta adormecida:** sair do Wet (modo, ferramenta) não larga a sessão: adormece-a. Zero tique e zero custo nos outros modos, comprimida em RAM (zstd 1). Medir o custo ANTES de ligar: traço Digital/Watercolor com × sem sessão adormecida, intercalado no mesmo processo ([MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md)). **Kill-criterion:** diferença acima do ruído = não entrega.
2. **Volta ao Wet:** a nova `base` = a tela de agora, com os traços de outros modos, e a grade acorda. Sobreposição: onde o outro modo pintou, a Borracha leva os dois (decisão 2). Rever `wetpaint_guard`, undo vivo (`undo_vivo.rs`) e a marca do undo; o `Arc` do canvas muda.
3. **Por camada:** confirmar se trocar de camada mata a sessão e tratar no mesmo desenho, com orçamento de RAM.
4. **Borracha inteligente:** sem tinta viva, imitar. Referência a medir: `vivo` da tabela (orla macia, Δ baixa, amassa). Candidato: uma sessão que nasce da própria imagem, sobre o traço. Provar pelo olho do dono: foto vivo × imitado.
5. **Opção ao salvar:** campo novo no `PaintedDocument`, **no FIM** (postcard é posicional), degrau de `PROJECT_SCHEMA` por `schema-recount.py`, e migração. Arquivo antigo ou sem a opção abre com a tinta morta e a Borracha imita. Ida e volta salva: a tinta reaberta igual ao bit.
6. Reprovações a cobrir: o preço da compressão, 4K com várias camadas, redo (item 0 da [próxima fase](PROXIMA_FASE_2026-11-06.md); o mesmo código).

**Fila depois disto:** itens 0–3 de [PROXIMA_FASE_2026-11-06.md](PROXIMA_FASE_2026-11-06.md). A foto 5 refeita pelo dono ainda não chegou (sem anexo; pasta sem alteração).
