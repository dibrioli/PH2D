# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6j (a 2.ª rodada do Trial na linha do tempo) — 2026-11-01

> Leitor: a próxima LLM que assume a linha. A P6j está **feita e aprovada no smoke do dono (01/11)**.
> A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · anterior: [P6i](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-01_P6i.md) (o §5 tem o aberto que continua aqui).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Sobre o main `422fec2027`; P0–P6g no main; P6h, P6i e P6j só na linha.
- Commits P6j (`git log --oneline 7253796620..HEAD`):
  - `2249259e7a` ronda 2 do Trial: Ctrl ao largar copia, intervalo de cels move só cels, Reverse sobre o envelope dos soltos, um Esc limpa intervalo e selecção, Select All limpa o intervalo, #8 o sprite nascido leva a paleta em RGBA.
  - `9635d2da9a` sobreviventes #10 #12 #13 #18 #20.
  - `283323ef45` a moldura a meio do arrasto com Ctrl não estica até às cópias (ids-fantasma `u32::MAX - j`); gestos conduzidos `PH2D_PIXEL_SMOKE_GESTO=copia|cels`; fotos.
  - `3b44708025` oráculo ronda 2 (docs).
  - `03d639c72c` fecha os sobreviventes da rodada de mutações A3 A7 A9 B2 A11 B1.
  - `3604ff40b4` defeito do smoke do dono (undo do intervalo).
  - `f85fef993d` clippy `type_complexity`: `type ParCel`.
  - + este handoff e a emenda P6j do ADR-0196.

## 2. O que a P6j fez

1. **`ph2d-pixel-core::frames`**: `landing`, `block_copy_order`, `copy_block(s, &ids, before) -> Option<(Op, Vec<FrameId>)>` (quadros novos por `reserve_id`, cels próprios sem link, `tags_after_insert` por inserção — a tag que contém a fronteira cresce), `move_cels(s, pairs, copy)` (rehome; origens que não são destino esvaziam salvo `copy`; cel vazio na origem esvazia o destino).
2. **`ph2d-editor-core::pixel_panel`**: `PixelRequest::RangeMove { before, copy }` (campo `copy` NOVO) e `PixelRequest::CelsMove { pairs: Vec<((u32,u32),(u32,u32))>, copy }` NOVO.
3. **`ph2d-app-pixel`**: `intervalo.rs` (RangeMove copy → `copy_block`, o intervalo passa às cópias; CelsMove → `move_cels`, intervalo Cels nos destinos; `range_span` = envelope, Reverse com soltos inverte o envelope) · `selecao.rs` (Esc com intervalo também tira a máscara; Select All limpa `d.range`) · `object_add.rs` (`born_from_paste` aplica a paleta em todo modo de cor) · `smoke_intervalo.rs` (gestos `copia`/`cels`).
4. **Defeito do smoke do dono (01/11), `state.rs`**: copiar com Ctrl, Ctrl+Z e copiar de novo falhava — o `Doc.range` ficava nos ids das cópias desfeitas. Trial `mover_ctrl_undo_de_novo`: o undo devolve o intervalo aos originais 2..3 e a mesma largada copia de novo igual. Cura: `Doc.ranges: (Vec<PixelRange>, Vec<PixelRange>)` — `commit` empilha o intervalo de ANTES do passo (alinhado ao `history.undo_len()`, apara à frente quando o histórico corta por bytes); o undo troca-o pelo actual e empilha o actual para o redo; o redo faz o inverso. Vale para todo passo, como a transação do Aseprite. O `Doc.range` continua estado de vista (ADR-0196, emenda P6h–P6i); só o undo o restaura.
5. **`ph2d-panel-pixel-timeline`**: `gestos.rs` (`Arrasto::Bloco{alvo,de,a,copy}`, `Voo{info,dx,dy,vao}`, `bloco_previsto` — para Cels não reordena, vão nos destinos; quadros: `block_copy_order`/`block_order`; `cels_pares` arredonda o deslocamento à célula mais próxima, fila de grupo devolve `None`; no fim pede CelsMove ou RangeMove com `copy: mods.cmd`) · `paint_voo.rs` reescrito (`vao`, `voa`: em Cels voam só as filas do intervalo) · `paint_grid.rs`.
6. **Oráculo**: `docs/PixelArt/ferramentas/trial_timeline.py` (gesto `dk:` = Ctrl antes de largar; casos novos incl. `mover_ctrl_undo_de_novo`), `docs/PixelArt/ferramentas/oraculo_timeline.md` (Ronda 2), fixtura `shells/pixel-scenes/tests/fixtures/oracle_trial_timeline.json` (entradas novas `move_range_ctrl` + `undo_then_again`, `move_cels*`, `reverse_picks`, `clear_delete_key`, `selection.shift_click_cel_after_drag`, `esc_with_mask_and_range`, `select_all_clears_range`, `born_sprite_palette`). Shift+clique em cel e 1 undo do Delete JÁ coincidiam (sem cura).
7. **Movido**: `quadros.rs` de `crates/ph2d-app-pixel/tests/it/` para `shells/pixel-scenes/tests/it/` (folga da catraca: `ph2d-app-pixel` ~13 18x de tecto 13 424).
8. **Fotos** `smoke_P6j_cena16_{copia,cels}_{fabrica,dono}_2026-11-01.png`; `layout.txt` do dono igual antes/depois (sha256 `5316591f…`).
9. **Mutações** (rodada P6b/P6c/P6h/P6i = 47 mutações, 32 sangraram, 15 sobreviveram): 7 fechadas com testes (A3, A7, A9, A9 filas, B2, A11, B1); equivalentes A6, B3, B5b; só-pintura A18, B9, B10 (ficam abertas); apanhadas noutra crate A17/pontas, C4/C7. Mutação à mão do defeito do undo: sem a restauração reprova em «o undo devolve 2..3».

## 3. Superfície de colisão NOVA (soma-se à P6i §3)

- `ph2d-editor-core/src/pixel_panel/mod.rs`: **`RangeMove` ganhou o campo `copy`** (quem a constrói tem de o dar) e **`CelsMove` é variante nova** (`match` exaustivo sobre `PixelRequest` tem de a tratar).
- `ph2d-pixel-core/src/frames.rs`.
- `ph2d-app-pixel/src/{intervalo,selecao,object_add,smoke_intervalo,state,quadros}.rs` (folga da catraca ainda curta).
- `ph2d-panel-pixel-timeline/src/{gestos,paint_voo,paint_grid}.rs`.
- `shells/pixel-scenes/tests/it/{intervalo,sobreviventes,quadros}.rs` (`quadros.rs` movido de `ph2d-app-pixel`), fixtura `oracle_trial_timeline.json`; `crates/ph2d-pixel-core/tests/it/fast_tie.rs` NOVO.

## 4. Prova (por nome, verde, 01/11)

- `ph2d-host-pixel-scenes`: `intervalo` 18, `sobreviventes` 16, `quadros` 4. `ph2d-panel-pixel-timeline`: `seam` 14. `ph2d-editor-core`: `the_shell_only_shrinks` 2. `ph2d-pixel-core`: `fast_tie`. `ph2d-host-desktop`: `pixel` 26, `the_key_blocks` 4. `ph2d-app-pixel`: `pontas`, `fecho`, `edit`, `catalogo`, `ajustes`, `viva`.
- Testes novos: `seam.rs` (`ctrl_at_the_drop_copies_and_a_cel_range_moves_only_its_cels`, `mid_ctrl_drag_the_outline_stays_on_the_originals`, `drags_round_to_the_nearest_cell_a_group_row_refuses_and_shift_never_duplicates`) · `intervalo.rs` (`ctrl_at_the_drop_copies_the_block_and_the_copies_become_the_range` com undo/de novo/redo, cels movem, Reverse dos soltos, Esc/Select All, Delete 1 undo, `a_copy_inside_the_tag_grows_it_and_an_empty_cel_empties_its_target`) · `sobreviventes.rs` (#8, #12 `mid_handle_drag_the_box_paints_only_its_outline`, #13 `the_shown_handle_of_scene_14_paints_hot_until_the_cursor_moves`, #20 `leaving_draw_through_the_mode_drive_settles_only_its_own_live_shape`, #18 `a_px_field_dragged_by_a_hair_stays_on_the_nearest_cell`) · `fast_tie.rs` (#10).
- **`fmt --all` e `clippy -D warnings` das 5 crates tocadas VERDES** (`ph2d-pixel-core`, `ph2d-editor-core`, `ph2d-app-pixel`, `ph2d-panel-pixel-timeline`, `ph2d-host-pixel-scenes`).

### Smoke (cena 16)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=16 ./target/smoke/ph2d-host-desktop`
2. Ctrl+arrastar a moldura 2..3 para entre 5|6 → 8 quadros, cópias 6..7 escolhidas. Errado: ficam 6 quadros, ou a moldura estica até às cópias.
3. Ctrl+Z → 6 quadros e moldura em 2..3; repetir a mesma largada → igual ao 2; Ctrl+Y → cópias escolhidas.
4. Células: escolher A2..A3 e arrastar duas colunas → só os cels andam (nenhum quadro novo).
5. Shift+clique em 2, 4, 6 + Reverse → 600, 500, 400, 300, 200 em 2..6.
6. Ctrl+A + intervalo + um Esc limpa os dois.

## 5. ABERTO

- Mutações só-pintura **A18, B9, B10**.
- **Não medido no Trial**: cópia com Ctrl numa camada contínua (ligado ou cópia? — nós fazemos cópias próprias); bloco de cels largado fora da grelha / fila de grupo (prendemos à grelha, grupo recusa); Paste as New Sprite em si (cinzento no Xvfb; medido no irmão).
- `a_altura_de_abertura_de_um_painel_so_encolhe` reprova com vizinhos (P6 §2a).
- O §10 do [handoff de integração 28/10](HANDOFF_INTEGRACAO_line_PixelArt_2026-10-28.md): RotSprite, empates do Fast, elipse > 150×96, tecto do Modify, `LengthDisplay`, paletas, P5, P7–P10.
- Fecho da linha: `/pd-linha-fechar`, `cargo machete` (deps novas desde a P6i), `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 6. Próximo item

A decidir pelo dono na abertura da janela nova; proposta: o dono disse «vamos seguir» depois do smoke OK, então o próximo bloco é o §10 do handoff de integração 28/10, pela ordem lá dada:

1. Smoke do dono da cena 15 (P6g).
2. Pedidos sem controlo: `ConstantFrameRate`, `TagRange`, `LayerContinuous` (`quadros.rs`); selecção de intervalos na timeline; P5 (subtags, grupos de tags, velocidades, Preview flutuante).
3. RotSprite fora de 90°; as 36 células de empate do Fast; elipse > 150×96 e a tabela `{4,8,12,22}`; foto da cena 9 com a lei Fast nova; tecto do Modify (100) e arredondamento %→px; `a_altura_de_abertura_de_um_painel_so_encolhe`.
4. Etiqueta do Pixel segue `LengthDisplay`; gravar com forma viva a assenta (P6d §6).
5. Paletas: arrastar-reordenar, Tint/Shade/Tone, Spectrum, rodas, presets; `.ase` import/export = P9; OKLCH/cycling = P13; GPU dos 19 modos de mistura.
6. Mutações da linha por correr (§5 acima).

Se o dono preferir integrar: `/pd-linha-fechar`. (Parte do item 2 já foi feita nas P6h–P6j: confira o código antes de acreditar na ausência.)
