# Handoff de continuação — Animated Sprite · várias folhas: V1 feita, V2 em lei pura (08/11)

**Substitui** [`HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V1.md`](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V1.md).
Plano: [26](../26_animated_sprite_varias_folhas_plano.md) (decisões do dono no topo — não se refazem). Maquete aprovada:
[26_hoje_e_proposta.png](../maquetes/26_hoje_e_proposta.png).

## Feito (V1 completo, verde por nome, 10/10 mutações sangraram)

- **`ph2d-ecs`:** `FrameRef { Cell { sheet, cell }, Empty }` (+ `run`, `cell_on`, `fits`), `SpriteGrid.sheet`
  (+ `new`, `shown`, `show`, `EMPTY_SHEET`), `AnimationTag.sheet`. Componente **`SpriteSheets(SmallVec<[ExtraSheet; 2]>)`**
  em `sprite_sheets.rs` (⚠️ mudou de casa face ao plano §3.1: mora em `ph2d-ecs` ao lado do `SpritePixels`, porque é
  só `AssetId` e assim `anim_steps`/marcadores/tique contam as folhas sem depender do `ph2d-render`). `SHEETS_MAX = 255`.
  `sheet_cells_of(world, e)` / `cells_per_sheet(grid, sheets)` = células por folha. `anim_steps(world, e, name)` voltou a
  3 argumentos. Registo `118 → 119` (`registry_tests.rs`).
- **`ph2d-render/src/sprite/sheets.rs`:** `SheetTextures` (cache derivado, não registado), `SheetView {Own, Other, Nothing}`,
  `sheet_view` (a lei de um ponto só), `resolve_sheet_textures` (átlas ganha; senão `upload`), `upload_sheet_image`
  (uma vez por imagem, thread_local).
- **Shell (composição):** `fase_extract_inputs.rs` resolve as texturas antes do extract; `sim_extract_emit::sprite` troca
  `Sprite`/grelha pela folha `k` à ENTRADA e sai cedo no vazio.
- **Painel (`ph2d-editor-core`):** `AnimFrameRef { sheet, cell }` (espelho, em `inspector_model_anim_frame.rs`);
  `InspectorAnimRow.{frames: Vec<AnimFrameRef>, sheet}`, `InspectorAnimInfo.{frame: AnimFrameRef, sheets: Vec<u32>}`,
  `AnimFieldEdit::Frames(u8, Vec<AnimFrameRef>)`, `row_cells(row, &sheets)`, `plays_on(&sheets)`.
- **Editor:** `EditorSnapshot.shown_sheet` (hoje sempre `0`; V4 liga o dropdown), cliques juntam `AnimFrameRef` da folha
  mostrada, `shown_cell`, tira/faixas só pintam a imagem da folha mostrada (V4 pinta as outras).
- **Crate NOVA `ph2d-anim-sheets`** (folha, como `ph2d-anim-signals`: `ph2d-app-sprite-anim` estava no tecto 7 545 da
  catraca `the_shell_only_shrinks_and_every_app_crate_too`; hoje 7 512): `panel_frame` (motor ↔ painel), `split` (movido;
  `add_rows(.., sheet, sheets)`), e **`ops`** (V2/V3 em lei pura, abaixo).
- **Schema `219 → 220`** (escada + tripla); catálogo `SpriteSheets` («Sprite Sheets», máquina), porta de macro `Gravado`.
- Provas: `ph2d-ecs` crate inteira (motivo: formato/registo) 653 ✓; `sprite_anim_frames` (mistura, vazio dura, intervalo
  de outra folha), `sprite_sheets`, `ph2d-render sprite::sheets` (4), `ph2d-host-game-scenes sprite_varias_folhas` (tique
  REAL + `sheet_view`), `ph2d-anim-sheets` (10). Testes antigos migrados para folha 0 e verdes por nome (game-scenes
  `sprite_`/`editor_animacoes`, editor-core, panel-inspector `anim`, desktop `preview`/`sim_extract`/`autokey_pass`…).
- Mutações V1 M1–M10 (render ignora folha, vazio pinta, tique sem folha, índice k sem −1, intervalo sem folha, folha
  inexistente cabe, vazio saltado, cache ignora troca, split ignora folha, ponte perde folha): **10/10 sangraram**.

## `ph2d-anim-sheets::ops` (escrito, testado, AINDA SEM chamador)

`FirstSheet` + `NewSheetImage` → `new_sheet` (m/px da folha 0; Pivot = mesma fração da caixa opaca, com espelho;
`carry_pivot`) · `after_deleting` / `delete_sheet(lib, frames, sheets, k)` (buracos no lugar; intervalo da folha k vira
lista de vazios; reindexa) · `empty_count`, `all_empty`, `remove_empty_frames` (⚠️ decisão técnica: NÃO toca a animação
toda vazia — tirar-lhe os buracos devolvia-a ao intervalo; é o Delete empty animations que a apaga) ·
`empty_animations`. Testes: `os_pes_ficam_no_sitio_ao_trocar_de_folha` (contra o oráculo Godot), `o_pixel_tem_o_mesmo_tamanho_nas_duas`,
`a_folha_nova_copia_o_pivot_da_primeira_pela_caixa_opaca`, `delete_sheet_deixa_quadros_vazios_no_lugar`,
`a_animacao_com_vazios_fica_em_alerta`, `remove_empty_frames_tira_so_os_vazios`, `delete_empty_animations_so_apaga_as_todas_vazias`.
**Mutações de `ops` ainda por correr** (Pivot = centro · tamanho da 1.ª · espelho ignorado · apagar em vez de esvaziar ·
sem reindexar · intervalo não vira lista · remove tira da toda vazia).

## Próximo (na ordem)

1. **V2 a fio:** o Pivot editado mexe na folha VISÍVEL (`pivot.rs::set_pivot/pivot_de/arrasta` sobre a `ExtraSheet` quando a
   visível é `k ≥ 1`) — prova `o_pivot_mexe_na_folha_visivel`. A «folha visível» é estado de UI (não save/undo).
2. **V3 cartão Image** (`ph2d-panel-inspector/src/cards_sprite_cabeca.rs`, `InspectorSpriteInfo`): ▾ no canto da miniatura →
   dropdown das folhas (moldura com a imagem inteira, nome, `C × R`, ✓), linha `W × H px · C × R · sheet i of n`, fila
   **+ Load sheet** (sempre em Animated) · **Delete sheet** (só com ≥ 2). Load = o painel de Assets de hoje com um alvo «nova
   folha» → `ops::new_sheet` + push em `SpriteSheets` (NÃO cria animação). Delete `k ≥ 1` = `ops::delete_sheet` + tirar
   `SpriteSheets[k-1]`; Delete da folha **0** = promover a folha 1 ao `Sprite`/`SpriteGrid`/`SpritePixels` pela porta
   `image_choice` e depois o mesmo `delete_sheet(.., 0)`. Um passo de undo (diff de componentes). Alerta no cartão
   Animation (`⚠` + «N empty», token de aviso) e a fila **Remove empty frames · Delete empty animations** (a 2.ª só se há
   uma toda vazia; apagar pela porta `AnimFieldEdit::Remove`, que leva marcadores e listas). Strings novas em inglês, i18n.
   Censo `o_censo_dos_campos_do_animated_em_uso` (17/48) sobe no máximo 1.
3. **V4 editor:** dropdown da folha no título (só com ≥ 2) liga `EditorSnapshot.shown_sheet` + grelha/imagem dessa folha
   (`SheetCache` por folha); tira pinta cada quadro com a imagem da sua folha + marca de folha quando mistura; vazios
   destacados; os dois botões ao pé da lista.
4. **V5:** F2 (usos da imagem contam `SpriteSheets`: `image_use.rs:674`, «used by», apagar asset), F3/F13 (`ph2d-sprite-edit/src/imagem.rs::aponta`
   reescreve a `ExtraSheet.asset` que casa), F4 (gravar: `project_sprite_pixels.rs`/`atlas_assets`, `asset_index_build.rs:322,402,625`,
   `asset_card_verbs::users_of`), F12 (macros embutem as folhas extra), F5 duplicar, F6 miniaturas (Hierarchy/Assets/prévia
   via `sheet_view`), F8 Stop repõe folha+quadro (o `Driven::SpriteAnim.frame` já é `FrameRef`). Antes: um explorador lista
   TODOS os leitores de `Sprite.source` fora de `ph2d-render`.
5. **V6:** cena `PH2D_GAME_SMOKE=5`, objeto «Two Sheets» (idle + run e uma animação misturada), foto
   (`docs/Components/ferramentas/fotografa_cena.sh`) lado a lado com a maquete ANTES do smoke; smoke em passos numerados.

## Cuidados

- Tectos: `ph2d-app-sprite-anim` 7 512/7 545 (o novo vai para `ph2d-anim-sheets` ou outra folha); `editor/paint.rs` 687/700,
  `inspector_model_anim.rs` 690/700, `component_catalog.rs` 700/700 (i18n novo noutro ficheiro do `ph2d-i18n`),
  `cards_sprite.rs` 520/600, `image_use.rs` ~698/700, shell 187 539/189 041.
- `cargo fmt -p` reformata ficheiros alheios (command_palette, look_tests, recibo): formatar SÓ os ficheiros tocados.
- Ainda devido no FECHO da linha (só por ordem do dono): mutações do Add Object do 1.º commit, a rodada anterior sem handoff,
  link do §5 no CLAUDE.md, `/pd-linha-fechar`.
