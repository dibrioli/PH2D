# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (conta-gotas/zoom + P10a, P10b, P10c: tilemaps) — 2026-11-06

> Leitor: o agente integrador. A linha **fecha e PARA**: não integra, não faz push. Plano: [P10_plano.md](../P10_plano.md) · medições: [P10_oraculos.md](../P10_oraculos.md).
> Mecanismo por onda: [P10a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-06_P10a.md) · [P10b](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-06_P10b.md) · [conta-gotas e zoom](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_CONTA_GOTAS_ZOOM.md).
> Antecessor no `main`: [P8d+P9](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-05_P9.md).

## 0. Leia primeiro

- ⛔ **Ordem do dono, 06/11: «Vamos integrar ao main. escreva handoff. Não integre.»** Integrar só por NOVA ordem explícita ao integrador.
- A linha leva **quatro entregas e integra como UMA**: conta-gotas/zoom · P10a (motor) · P10b (tela) · P10c (terreno 47, iso/hex, Export Tileset, split/trim).
- **Smoke do dono:** P10a e P10b aprovadas em 06/11 (P10b: cena 21, todos os passos); conta-gotas/zoom `Smoke OK` em 06/11. **P10c: smoke NÃO feito** — «faremos o smoke completo amanhã». **Integrar não é aprovar** (§8).
- **Não correram nesta linha:** a suíte completa e o `nextest-impacted` (são do integrador, regra do dono 23/10). Conferências baratas: ver abaixo.

`cargo machete` limpo (tirou `ph2d-pen` e `ph2d-pen-law` de `ph2d-app-pixel`, em `89c0cc3a19`: a ponte os levou na P10b) · `check-standalone-optional.sh` verde · `check-workflow-packages.sh` verde · `target/*/incremental` reclamado · binário `smoke` compilado, 2.ª corrida `Finished` em 0,30 s, sem `Compiling` · depois do último commit de código: `ph2d-host-pixel-scenes tiles::` 26 verdes

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`.
- Merge-base `bbfefaf393` (= `main` hoje, 0 commits atrás); último commit de código `89c0cc3a19` (só tira 2 dependências de `app-pixel`), e por cima só este handoff. Os números a seguir são de `2dc8479b90`: `git diff --stat bbfefaf393..2dc8479b90`: **19 commits, 192 ficheiros, +47 007 / −641** (docs, fotos e fixturas incluídas: 102 `A`, 89 `M`, 1 `R`).
- Ondas: conta-gotas/zoom `4b83b84fd7`,`286ce1ea95` · P10a `4952bcf0ee`,`4d0258915a`,`84e720b958`,`940c8a07e5`,`b8060437f8`,`c30817c2a6` · P10b `94941afa39`,`cd67695794`,`5b67ad67ed`,`88fc4ffc60`,`46a80e6788`,`31e98054d9`,`0e6595ecfe` · P10c `0177ec637e`,`66a695f114`,`8f17478063`,`2dc8479b90`.
- Sem rebase (ordem: não integrar). Como o `main` não andou, o rebase é nulo hoje; a árvore combinada é do integrador.

## 2. O que entrega

- **Conta-gotas/zoom**: ver o handoff próprio (o seletor partilhado amostra em Draw; o zoom não salta ao sair do Pixel).
- **P10a (motor)**: `Tileset`/`TileGrid`/`TileRef`/`Op::Tiles`; lei: `Cel.tiles` é a verdade, `Cel.data` deriva; `.ase` lê e grava tilemap (o Trial abre); Manual, Auto, Stack, Convert To; gabarito de 47.
- **P10b (tela)**: janela flutuante TILEMAP (crate nova), New Tilemap Layer, Convert To, Pixels/Tiles, Draw Tiles com Flip, `Space+Tab`/`Space+1/2/3`; traço ao vivo igual ao resultado; tilesets servem a qualquer camada; M6 2 531 → 38 ms.
- **P10c**: cartão **Terrain** (gabarito de 47 pincelável, Template, vários terrenos sem transição); **formas Square/Iso/Hex/Hex Flat** por tileset; **File > Export Tileset…** (PNG dos tiles + JSON do mapa, prévia ao vivo); **Export Sheet** ganha *Split grid* e *Trim by grid*; cena **22**.

## 3. Superfície de colisão (confira com `git diff --stat bbfefaf393..HEAD -- <caminho>`)

| Onde | O quê | Por que é aditivo |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/pixel.rs` | `PIXEL_TILES_PANEL` = `"pixel_tiles.panel"` · `CTX_MENU_PIXEL_EXPORT_TILESET` = `"ctx_menu_pixel_export_tileset"` | ids novos, hash de string |
| `ph2d-editor-core/src/pixel_panel/mod.rs` | `PixelFileAction::ExportTileset` | **variante nova em enum público: `match` exaustivo noutra linha parte na compilação** |
| `ph2d-editor-core/.../pixel_panel/layers.rs`, `pixel_edit_ops.rs`, `screens/hero/{menu_tables,paint,panel_ids,pre_populate}.rs` | `LAYER_ACTIONS` 14→17 (`pixel.layer.new_tilemap` → `NewTilemap`, `pixel.layer.to_tilemap` → `ConvertToTilemap`, + `ConvertToImage`; textos «New Tilemap Layer» / «Convert to Tilemap»); linha do menu; painel no hero | tabelas append-only, +1..5 linhas cada |
| `ph2d-panel-pixel-export/src/ids.rs` | `SEC_TILES_OUT` `"pixel_export.sec.tileset"` · `SPLIT_GRID` `"pixel_export.split_grid"` · `TRIM_BY_GRID` `"pixel_export.trim_by_grid"` · `COLUMNS` `"pixel_export.columns"` · `tileset_option(i)` = `"pixel_export.tileset.{i}"`; `NUMEROS` 15→16, `CHECKS` 11→13 | ids novos; os arrays crescem |
| `ph2d-panel-pixel-tiles/src/ids.rs` (crate nova) | `TILES_DRAG/CANCEL/OK`, `SEC_DRAW/SEC_TILESET/TILES_SEC_GRID/SEC_TERRAIN`, `TARGET_{PIXELS,TILES,TERRAIN}`, `MODE_{MANUAL,AUTO,STACK}`, `FLIP_{X,Y,D}`, `ADD_TILE`, `DELETE_TILE`, `TILE_W/H`, `MATCH_FLIPS`, `NEW_SET`, `NEW_TERRAIN`, `DELETE_TERRAIN`, `TERRAIN_TEMPLATE`, `SHAPE_{SQUARE,ISO,HEX,HEX_FLAT}`; strings `pixel_tiles.*`; dinâmicos `pixel_tiles.{set,tile,terrain}.{i}`. `BOTOES: [NodeId; 21]` | ids novos |
| `ph2d-i18n` | `pixel_tiles.rs` NOVO (89 l), `lib.rs` +2, `chrome_menus.rs` (`chrome.menu.pixel_export_tileset`), `pixel.rs`, `pixel_export.rs` (+title_tileset, columns, split_grid, trim_by_grid) | chaves novas; **foundational** |
| `ph2d-pixel-render` | `lib.rs`, `view.rs` +62 | **foundational**: grelha do mundo em iso/hex |
| `ph2d-render/src/camera.rs` (+tests) | +11 / +17 | zoom ao sair do Pixel (conta-gotas/zoom) |
| `ph2d-pixel-core` | `forma.rs` NOVO (`Forma`), `rect.rs`, `tiles.rs` + `tiles/expande.rs`, `codec/grade.rs` NOVO, `canvas/compose/convert/frames/layers/op/sprite/stroke/transform` | módulos irmãos para caber no tecto de 700 linhas/ficheiro |
| `ph2d-pixel-io` | `ase/saida_tiles.rs`, `tiles_saida.rs` NOVOS, `exporta.rs`, `sheet.rs`, `abrir.rs`, `ase.rs` | **`Export` ganhou `Tiles(ExportTiles)`**: `match` exaustivo noutra linha parte |
| `ph2d-aseprite/src/{file,write}.rs` | cel/camada tilemap | ler/gravar |
| `ph2d-app-pixel` | `tiles.rs`, `smoke_tiles.rs`, `conta_gotas.rs` NOVOS; `ajustes.rs` MOVIDO (R094) para `ph2d-pixel-ponte`; `state.rs`, `smoke.rs` (`max_level` 20→**22**), `exportar.rs`, `input_teclas.rs`, … | **`input_teclas.rs`: conflito provável com `828051653b` de `line/GameEngineFuncional` (Tab)**: aqui `Tab` sozinho ainda abre a timeline e `Space+Tab` mexe no mesmo sítio |
| `shells/desktop` | `Cargo.toml` +2 (feature `panel-pixel-tiles`), `input_dispatch/despacho_pixel.rs`, teste `every_panel_the_shell_drives_is_in_its_registry` | |
| `ph2d-panel-registry-init` | `Cargo.toml` +3 (feature `panel-pixel-tiles`), `lib.rs`, testes (`the_pixel_tiles_answers_a_real_pointer` NOVO) | feature nova |
| `ph2d-app-registry-init` | `Cargo.toml` (+`ph2d-pixel-core`), `tests/it/tilemap_recusa.rs` NOVO | |
| `shells/pixel-scenes` | `Cargo.toml` +2, `tests/it/{tiles,conta_gotas}.rs` NOVOS, `edit/fecho/main` | |

**Crates NOVAS** (glob `crates/*`): `ph2d-panel-pixel-tiles`, `ph2d-pixel-ponte` (código puro que saiu de `app-pixel`), `ph2d-pixel-tiles` (leis de desenhar, terreno, grelha, converte). `Cargo.lock` **+45 linhas**: ao colidir, REGENERE, nunca à mão.
**Dev-deps novas**: `ph2d-grid` (+`serde_json`) em `ph2d-pixel-tiles`; `ph2d-pixel-tiles` em `ph2d-pixel-io`.

**API pública que muda** (outras linhas que as usem partem): `Tileset.forma`; `TileGrid.col/row`; `TileGrid::rect` **removida** em favor de `caixa(&Tileset)`; `Carimbo.tile` virou `marca`; `PixelFileAction::ExportTileset`; `Export::Tiles`.

**Foundational tocado**: `ph2d-editor-core`, `ph2d-i18n`, `ph2d-pixel-render` (todos append-only).

Colisões prováveis: `ph2d-i18n/src/lib.rs`, `menu_tables.rs`, `pre_populate.rs`, `registry-init` (Cargo + lib), `shells/desktop/Cargo.toml`, `Cargo.lock`, `hero/paint.rs`, `input_teclas.rs`, o tecto de `ph2d-app-pixel` em `architecture_the_shell_only_shrinks.rs`.

## 4. Contratos e schemas: NÃO tocados

`git diff --name-only bbfefaf393..HEAD` não lista ficheiro de `PROJECT_SCHEMA`, `ph2d-nodegraph`, `ph2d-tool-*`, `ph2d-vec-*` nem de gates `*contract*` (grep vazio). Contratos §6 do CLAUDE.md intactos.

**Formato gravado**: `PH2DPIXEL` fica em **MINOR 4** (`main` está em MINOR 3: o 4 nunca chegou ao `main`; nada a re-degraus). Campos APENDADOS: tileset **tag 7 = forma**; grade **tags 6 e 7 = `col`, `row`** (`codec/grade.rs`). Binário MINOR 3 lê o MINOR 4 e vê camada de imagem com a mesma arte; regravar perde só os tiles. Fixtura real MINOR 3: `pixel_minor3_bbfefaf393.bin`.

## 5. Números e tectos (DELTA)

| Item | Valor |
|---|---|
| `ph2d-app-pixel` (`architecture_the_shell_only_shrinks.rs:222`) | tecto **11 415** (era 11 415 no P9), `wc -l` de `crates/ph2d-app-pixel/src/*.rs` = **11 385** (o P10b anotou 11 400 antes dos últimos cortes; a catraca pode contar de outro modo — confira rodando o gate). ⚠️ SOMA entre linhas: recontar com `python3 scripts/recontar.py`, nunca subir à mão |
| Cena de smoke | `max_level` 20 → **22** (cena 21 = P10b, 22 = P10c). Se outra linha tomou 21/22, RECONTE |
| `LAYER_ACTIONS` | 14 → 17 · `NUMEROS` 15→16 · `CHECKS` 11→13 · `BOTOES` do painel novo 21 |
| `Cargo.lock` | +45 linhas · 3 pacotes novos |
| Fixturas novas (bytes) | `oracle_tile_edges_m9.json` **522 577** (≈ 0,5 MB) · `oracle_godot_grelhas.json` 266 473 · `oracle_godot_p10c.json` 77 657 · `oracle_pixelorama_modos.json` 35 653 · `oracle_pixelorama_grelhas.json` 17 157 · `oracle_godot_autotile.json` 15 724 · `oracle_trial_flips.json` 3 717 · `oracle_godot_remonta.json` 2 028 · `pixel_minor3_bbfefaf393.bin` 6 526 |
| M6 (256×256 células × 64 quadros, 1 px) | 2 531 → **38 ms**; desfazer 33 |

## 6. Prova de fecho (por nome; a suíte completa é do integrador)

**Crates inteiras, verdes na P10c** (motivo: formato gravado / crate alterada): `ph2d-pixel-core`, `ph2d-pixel-io`, `ph2d-panel-pixel-tiles`, `ph2d-panel-pixel-export`, `ph2d-pixel-tiles`.

**Por nome, verdes no HEAD final `2dc8479b90`**: `ph2d-host-pixel-scenes tiles::` **26** · `ph2d-panel-registry-init the_pixel_` **19** · `ph2d-pixel-tiles --test p10c` **12** · `ph2d-pixel-io --test aceitacao_tiles` **7** · `architecture_workspace_file_loc_cap` **2**.

**Verdes antes dos últimos commits (só moveram LOC)**: `exportar::`, `tilemap_recusa`, `the_shell_only_shrinks`, `panel_functions_under_loc_cap`, `no_magic_numeric`, `literal_color`, `wiring_parity`, `behavioral_test`, `a_minor_3_file…`, `node_id_collisions` (14, refeito depois do renome). ⚠️ **`no_magic_numeric`, `wiring_parity` e os behavioral NÃO foram refeitos depois dos dois últimos commits pequenos** (`8f17478063`, `2dc8479b90`).

**Clippy `-D warnings` verde** em: `ph2d-pixel-core`, `ph2d-pixel-tiles`, `ph2d-pixel-io`, `ph2d-pixel-render`, `ph2d-panel-pixel-tiles`, `ph2d-panel-pixel-export`, `ph2d-app-pixel`, `ph2d-host-pixel-scenes`. ⚠️ **`ph2d-i18n`, `ph2d-editor-core` e `ph2d-panel-registry-init` NÃO entraram nessa corrida**, embora tenham edições de tabela e de teste.

**Mutações, por agente**: leis da P10c, **25 mutantes, todos sangraram no fim**. Sobreviventes da 1.ª ronda (10, 17, 18, 19 e depois o 22) eram **fixturas que não separavam hipóteses**, curadas assim: canto do guia por posição · obra com a grelha deslocada (origem 5,3; M8 refeito, 0 píxeis) · píxel isolado no trim · fase no split · tinta na coluna 0 do canvas. P10a: 31 mutantes (2 sobreviventes resolvidos). P10b: todos sangraram em três rondas.

**Oráculo (M2iso, M3b, M8a, M8, M10; [P10_oraculos.md](../P10_oraculos.md))**
- **Godot 4.7.2 real** (xvfb, `opengl3`, llvmpipe) remonta o mapa a partir do NOSSO PNG + JSON com **0 píxeis diferentes**, `square`/`iso`/`hex`/`hex_flat` × 2 quadros. **Só arte opaca; alfa parcial não foi comparado.**
- ⚠️ O gate `o_godot_remonta_o_mapa` compara o sha do que o Godot leu: **qualquer mudança na saída exige refazer** `docs/PixelArt/ferramentas/oraculo_godot_remonta.py` e regravar `oracle_godot_remonta.json`.
- Trial abre o `.ase` da aceitação (foto `handoffs/P10_trial_aceitacao.png`, só o 1.º quadro visto).
- **M10 (Trial, split/trim)**: ver a tabela em P10_oraculos. Não medido: Ignore Empty com vazias (preview preso), Source: Tilesets, várias camadas de tilemap, JSON/ficheiro exportado, flags de linha de comando.

**Diferenças DE PROPÓSITO do Trial**: a grelha do split/trim é a do **tile** (o documento não guarda grelha do sprite); *Source Sprite Grid + Trim Sprite* (célula sem o píxel / preview preso) NÃO se copia; a célula parcial à esquerda/em cima tem `origem` x/y **clampada a 0** no JSON (`u32`). Ordem de desenho em hex vs. `rendering_quadrant_size = 16` do Godot: só diverge em arte que sai da forma do tile (coluna, depois linha = 0).

**Cura fora do tilemap**: o desfazer que removia a camada activa deixava `doc.layer` pendurado; o passo em `state.rs` chama agora `camadas::sane_active(doc)`. ⚠️ Pode mudar o resultado de testes de outras linhas na árvore combinada.

## 7. O que um merge pode partir

1. `match` exaustivo sobre `PixelFileAction` ou `Export` noutra linha (variantes novas).
2. Quem use `TileGrid::rect` ou `Carimbo.tile` (removidos/renomeados).
3. `Cargo.lock`, features do `shells/desktop` e do `ph2d-panel-registry-init`: regenerar/recontar.
4. Cena de smoke 21/22 e `max_level` se outra linha criou cenas do Pixel.
5. `input_teclas.rs` × `828051653b` (Tab).
6. Tecto de `ph2d-app-pixel` e o de LOC por ficheiro (700): somam entre linhas.
7. `project-memory`: `MEMORY.md` com 18 080 bytes (alvo 17 KB). `git -C /home/enio/Documentos/Projetos/PH2D status --short project-memory` (leitura) mostra `M MEMORY.md` e 4 `??` na árvore PRIMÁRIA (`project_sfx_w4_owner_delegated_decisions_2026_11_06`, `project_tilemap_rule_discovery_not_intuitive_2026_11_06`, `project_tiles_any_layer_oracle_decides_2026_11_06`, `project_two_image_objects_pixel_and_painter_2026_11_06`): **são do integrador/dono comitar; esta linha não os tocou**.

## 8. Smoke do dono — cena 22 (P10c, ainda NÃO feito)

⚠️ Depois da integração o binário tem de ser reconstruído no worktree/`main` onde o dono for abrir (`--profile smoke`).

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=22 ./target/smoke/ph2d-host-desktop`
2. **Abre** um mapa com buracos e corredores estreitos, feito de tiles de terreno; a janela TILEMAP aparece com **Terrain** aceso. *Errado:* mapa vazio, sem a janela, ou sem o cartão Terrain.
3. Na janela TILEMAP escolha o destino **Terrain**, pegue o lápis e **arraste numa área vazia** do mapa: cada célula recebe o tile que combina com os vizinhos, bordas incluídas. Botão direito (ou Borracha) apaga. **Ctrl+Z** desfaz o arrasto inteiro de uma vez. *Errado:* bordas que não se ajeitam; Ctrl+Z que desfaz só um pedaço.
4. No cartão **Terrain** clique **Template**: abre a camada «Terrain 1 template» com as 47 formas. Em **Pixels**, pinte numa forma: as células do mapa que usam aquela forma mudam. **Ctrl+Z duas vezes** volta tudo. *Errado:* o mapa não muda, ou sobra a camada de gabarito.
5. No cartão Tileset, **Grid shape**: **Iso** vira losangos; **Hex** recusa com uma mensagem (terreno não funciona em hex); **Ctrl+Z** volta. *Errado:* Hex aceita calado, ou a arte muda de lugar.
6. Camada **New Tilemap Layer** → **New tileset** → **Hex** → **Create**: não deve haver cartão Terrain; **Pixels** desenha o tile, **Tiles** carimba arrastando. *Errado:* Terrain aparece em hex, ou o carimbo cai fora da célula.
7. Menu **File → Export Tileset…**: a prévia muda ao vivo; ajuste **Tiles per row**; **Export** grava um `.png` e um `.json` ao lado. *Errado:* prévia parada, um só ficheiro, ou erro mudo.
8. Menu **File → Export Sheet…**: ligue **Split grid** (recorta pela grelha do tile). **Trim by grid** só aparece com um trim ligado. *Errado:* a caixa aparecer sem trim, ou folha com pedaços fora de ordem.

Regra do dono: o smoke de hoje é de amanhã; a linha não decide sozinha o veredito da P10c.

## 9. ABERTO (honesto)

- **Smoke da P10c** (§8).
- Desempenho da grelha iso/hex num canvas enorme e a prévia ao vivo na GPU: **não medidos**.
- *Ignore Empty* com vazias no split, e várias camadas de tilemap no split: não medidos.
- Terreno não se renomeia; terreno em hex é recusado.
- Miniaturas do Tileset em iso mostram a caixa do tile (retângulo), não o losango.
- Abertos da P10b que continuam: dois cliques depressa só curados em `ph2d-panel-pixel-tiles`; a janela Export sem prova por ponteiro de flutuar/arrastar; sem apagar um tileset inteiro; sem a opção de cortar células de fora no Canvas Size.
- O `.ase` não guarda a forma: iso/hex sai como imagem, com aviso `FormaVaiComoImagem`; o tileset fica fora do ficheiro.
- Não medido no M9 (P10b): Pixelorama por ponteiro real nos casos M9b; desfazer em M9b; caso 1 invertido.
