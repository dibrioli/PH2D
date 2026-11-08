# Handoff de continuação — Animated Sprite · várias folhas, V1 a meio (08/11)

**Estado:** desenho fechado com o dono; código no **início de V1**. Plano: [26](../26_animated_sprite_varias_folhas_plano.md)
(leia-o inteiro: decisões do dono no topo, §3 técnico, §4 família F1–F13, §5 etapas, §6 provas). Maquete:
[26_hoje_e_proposta.png](../maquetes/26_hoje_e_proposta.png). Memória:
[decisões](../../../project-memory/project_animated_sprite_many_sheets_owner_decisions_2026_11_08.md).
Oráculos corridos: [`oraculos/varias_folhas/`](../oraculos/varias_folhas/).

## Feito (commit `wip(GEF 26 V1)`)

Só `ph2d-ecs`, e só a lib compila; **o resto da workspace NÃO compila ainda** (assinaturas mudaram):

- `FrameRef { Cell { sheet: u8, cell: u32 }, Empty }`, `FrameRef::cell`, `fits(sheets)`, `sheet_cells(sheets, k)` em
  `sprite_anim_frames.rs`; `TagFrames.cells: Vec<FrameRef>`; `get`/`set` em `FrameRef`.
- `tag_cells`, `advance_frames(_visiting)`, `entry_cell`, `advance_frames_marked` recebem `sheets: &[u32]` (células por
  folha, índice 0 = a do `Sprite`) e `frame: &mut FrameRef`. No caminho de intervalo, um quadro de outra folha entra
  pela ponta (como o `u32::MAX` da lista).
- `anim_steps(world, e, name, sheets)` — `sheets` vazio = só a grelha da entidade.
- `SpriteGrid.sheet: u8` (+ `EMPTY_SHEET = u8::MAX`, `new(h, v)`, `shown()`, `show(f)`; `is_single` olha `sheet`).
- `AnimationTag.sheet: u8` no FIM (`new` põe 0).

## Próximo (na ordem)

1. **Fechar a compilação** de quem chama (V0 do explorador, 08/11): `ph2d-app-sprite-anim` (`tick.rs:61-127,213-224`,
   `inspector.rs:53-62,290,300,361`, `card.rs:82-87,145,171,180-186,287-300,398`, `editor/split.rs`, `image_use.rs:176,379`),
   `ph2d-anim-signals/src/lib.rs` (`anim_steps`), `ph2d-ecs/src/image_use_memory.rs:17,60`, e os literais
   `SpriteGrid { … }` (8 de produção: `image_choice.rs:62`, `image_use.rs:163,688`, `ph2d-app-vec smoke_bone_media*.rs`,
   `hero_sheet.rs:115`, `ph2d-app-game/src/smoke.rs:457`, `ase_import.rs:311`, `anim_smoke.rs:95`; 24 ficheiros de
   teste) → usar `SpriteGrid::new`. O tique escreve `grid.show(frame)` só quando muda. As células por folha vêm de
   uma função em `ph2d-render` (`sheet_cells_of(world, e)`: grelha + `SpriteSheets`).
2. **`SpriteSheets` + `SheetTextures`** em `ph2d-render/src/sprite/sheets.rs` (§3.1: só `AssetId`; cache derivado
   não registado, na lei de `ImageOverride`/`upload` de `ph2d-app-components/src/script_image.rs`). Registo:
   `ph2d-ecs/src/scene/registry.rs:559` (modelo `SpriteAnimFrames`; censo `registry_tests.rs:262` 118 → 119),
   `ph2d-app-macros/src/mundo/portas.rs:94`, `ph2d-component-desc/src/catalog/image.rs:225` + i18n.
3. **Render num ponto só** à entrada de `shells/desktop/src/render_loop/sim_extract_emit.rs::sprite` (cópia de `Sprite` +
   `SpriteGrid` com os campos da folha k; `EMPTY_SHEET` não emite).
4. **Schema 219 → 220** (`shells/desktop/src/project_schema.rs:511`, tripla `project_schema_tests.rs:360`,
   `python3 scripts/schema-recount.py`). Sem leitor de formato velho (CLAUDE §0.11).
5. Depois V2–V6 do plano.

## Cuidados

- Tectos de LOC: `image_use.rs` 696/700, `cards_sprite.rs` 520/600, `card_field.rs` 499/500,
  `fase_object_add_tests.rs` 600/600. Cura = mudar de casa.
- Testes por NOME, com mutação; suíte inteira só na integração. Crate inteira só com `motivo:` (formato gravado,
  registo de componente — vale aqui uma vez, no fecho de V1).
- Ainda devido no FECHO da linha (só por ordem do dono): mutações do Add Object do 1.º commit, a rodada anterior sem
  handoff, link do §5 no CLAUDE.md, `/pd-linha-fechar`.
