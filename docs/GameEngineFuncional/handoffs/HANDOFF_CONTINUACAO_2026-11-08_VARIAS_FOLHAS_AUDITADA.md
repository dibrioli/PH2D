# Continuação — GEF 26 (várias folhas no Animated Sprite) FEITA, auditada, smoke do dono OK · 08/11

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`. Plano: [`26_animated_sprite_varias_folhas_plano.md`](../26_animated_sprite_varias_folhas_plano.md). Anterior: [`HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V5_FEITA.md`](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V5_FEITA.md).

## O que está feito (commits `c652d296c7..3f75fb43e1`, todos com mutações que sangraram)

- **V5 resto + V6** (`c652d296c7`, `229e66d496`): Painter na folha visível, recut/Cols-Rows em todas as folhas, `SheetNames` uma fonte, folhas na macro de outro projeto, cena «Two Sheets» (`PH2D_GAME_SMOKE=5`).
- **Rodada de smoke 08/11** (`3ec7ad4fbb..44d5d2ec93`): prévia do Painter pela folha visível (`ph2d_render::open_preview_cell`); faixa ▾ na miniatura; «SHEET» em destaque; × 2× (margem Xxs); barra de rolagem da tira (`strip_bar.rs`); **marca de inserção** (`doc_strip.rs`: `insert_at`, `playhead_gap`, `spliced`, `markers_follow_insert`); **voo da inserção** (`doc_flight.rs`, mola `Role::Travel`, `AnimDocs::wants_frame`); roda pelo `strip_content`; clicar num quadro põe a marca à direita; o inserido fica escolhido (+`SetStep`); moldura de acento em toda linha escolhida.
- **Auditoria de 2 lentes + 2.ª rodada** (`0a2e653876`, `057cf46739`, `3f75fb43e1`):
  - recut compara a imagem pelos DOIS nomes (biblioteca e célula de átlas via `SheetTextures`);
  - marca da animação recém-criada no começo (`playhead_gap` pela `click_base`);
  - `sheet_label` (`ph2d-anim-sheets::editor_ui`) = um nome no editor e no cartão;
  - troca de folha com o Painter ligado: lei `ph2d_sprite_screen::imagem_aberta::{decide_troca,troca}`, ponte `ph2d_app_painter::imagem_aberta::folha_trocou`, cola `hero_intents::troca_de_folha` (na `fase_sim_extract`): assa na folha ANTIGA e solta o documento;
  - editor mostra a tinta por assar ao fim do traço: `sheet_cache::follow_live` + `painter_bridge_queries::revisao_viva`;
  - catraca `the_shell_only_shrinks_and_every_app_crate_too` VERDE (estava vermelha desde a V5): **crate nova `ph2d-anim-editor`** com o documento do editor (a família `ph2d-app-sprite-anim` só liga; tecto 7 545 → 5 209); `publish_sheet_names` → `ph2d-anim-sheets::names` (`stale_ids` + `publish`).

## ABERTO — fazer num bloco, ANTES de integrar (ordem do dono 08/11: «consertar tudo antes de integrar»)

1. **DECISÃO DO DONO 08/11: as ferramentas de imagem agem na folha VISÍVEL**, como o Painter: Trim, Padding, Make Square, Upscale, Real Size, Rasterize, BG removal, Color equalization, Equalize sizes e afins (`shells/desktop/src/render_loop/fase_image_edit_apply.rs`, `hero_intents/image_edit/*`, funil `texture_edit.rs`). Hoje escrevem a folha 0. Modelo: `read_shown_extra`/`commit_shown_extra` (`ph2d-anim-sheets/src/shown_image.rs`) e `pinta_folha_extra`. Undo de folha extra = o do documento (`SpriteSheets` registado), não `ImageEditSnapshot`. Censar TODOS os verbos de imagem antes (lista aberta: meça a composição antes de construir).
2. **Roda durante o voo** é sobrescrita: `layout.rs` recalcula `strip_scroll` de `start_scroll` a cada quadro. Cura provável: a roda/barra cancelam o voo (como o `press` já faz) — `ph2d-app-sprite-anim/src/editor/mod.rs::wheel`.
3. **Cartão Animation sem folha 0 descodificada** esconde prévia e miniaturas de quadros de outras folhas: `ph2d-panel-inspector/src/sections/anim_card.rs:33-35` e `:158-160` exigem `info.card.sheet`; deviam olhar `sheet_of(k)`.
4. **Falha de upload da folha extra no Painter** diz «tela vazia»: `hero_intents/image_edit/painter.rs` `pinta_folha_extra` devolve `Some(None)` nos dois casos; separar e dar a mensagem certa (i18n).
5. **`LoadSheet` sem `hero_screen`** cai em `apply` que devolve `false` calado (`asset_drop_apply.rs:126`, `ph2d-anim-sheets/src/apply.rs:27`): avisar ou tornar impossível.
6. **Macro noutro projeto, folhas extra**: conferir que `mundo/embute.rs` embute as imagens das folhas extra e o flag `premultiplicado` delas (`ph2d-app-macros/src/sessao.rs:545`); gate que prove cor igual na chegada.
7. **Herdado, baixa**: `Documents::wants_frame` não tem chamador na shell (o laço é `ControlFlow::Poll`, então hoje não importa). Decidir: ligar ao laço ou apagar o contrato morto — não é desta feature; registar se não couber.
8. Escolha a conhecer (já dita ao dono): trocar de folha com o Painter ligado ACHATA as camadas da folha antiga (o documento sai). Não há documento por folha no Painter.

## Depois: fecho da linha (só por ordem do dono)

Mutações do Add Object do 1.º commit · handoff da rodada anterior sem handoff · link do §5 no `CLAUDE.md` · `/pd-linha-fechar` (gate batched 1×; `fmt`/`clippy` das crates tocadas, incluindo a NOVA `ph2d-anim-editor`). Push e integração só por ordem do dono.

## Como medir

- Editor: `bash scripts/cargo-test-narrow.sh ph2d-host-game-scenes editor_animacoes` (63 verdes).
- Catraca: `bash scripts/ph2d-run.sh cargo test -p ph2d-editor-core --test it the_shell_only_shrinks` (verde; folgas pequenas: `ph2d-app-painter` e `ph2d-app-components` perto do tecto — código novo vai para crate não-`app`).
- Smoke: `PH2D_GAME_SMOKE=5 ./target/smoke/ph2d-host-desktop` («Two Sheets»); fotos `docs/Components/ferramentas/fotografa_cena.sh` com `PH2D_GAME_SMOKE_FOTO=animado_folhas|animado_folhas_editor|animado_folhas_apagada|animado_folhas_trim`.
