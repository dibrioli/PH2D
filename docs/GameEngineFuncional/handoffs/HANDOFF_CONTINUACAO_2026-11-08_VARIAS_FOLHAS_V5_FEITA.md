# Handoff de continuação — Animated Sprite · várias folhas: V2–V5 feitas, falta o resto da V5 e a V6 (08/11)

**Substitui** [`HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V1_FEITA.md`](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V1_FEITA.md).
Plano: [26](../26_animated_sprite_varias_folhas_plano.md) (decisões do dono no topo — não se refazem). Maquete aprovada:
[26_hoje_e_proposta.png](../maquetes/26_hoje_e_proposta.png).

## Commits desta janela (todos verdes por nome, cada lei com mutação que sangrou)

| Commit | O quê |
|---|---|
| `49c68c2884` V2 | `ShownSheet` (estado de UI, não registado) + `shown_sheet()` presa; Pivot edita a folha VISÍVEL; numa folha extra só a âncora dela muda (origem, filhos, colisão e outras folhas ficam — decisão técnica: mexer a origem faria a folha 0 saltar); `ph2d_render::{sheet_sprite, sheet_geometry, drawn_geometry}`; a caixa do gizmo e o encaixe medem o quad NA TELA. Mutações: `ops` 11/11, V2 9/9. |
| `b9a1882962` V3 | Cartão Image: ▾ + dropdown das folhas, Load sheet (ícone Plus), Delete sheet (Trash, só com ≥ 2); cartão Animation: linhas em alerta «⚠ nome · N empty» e Remove empty frames · Delete empty animations. `ImageUseEdit::{ShowSheet, LoadSheet, AddSheet, DeleteSheet, RemoveEmptyFrames, DeleteEmptyAnimations}` → `ph2d_anim_sheets::apply`. Assets: `choose_sheet_for`, `Choosing::answer`. `slot_sheets::SheetCard` publicado por `shells/desktop/src/render_loop/snapshots_slot_sheets.rs`. Censo do Animated 17/48 → 18/49. Mutações 14/14. |
| `8ffb7372cb` V4 | Editor: dropdown da folha no título (a MESMA `ShownSheet`), tira por folha com marca quando mistura, vazios destacados, os dois botões ao pé da lista. `SheetCache` mudou-se para `ph2d_anim_sheets::sheet_cache` (re-export `sheet`); `editor_ui.rs` novo. `ph2d-app-sprite-anim` −121. Mutações 5/5. |
| `1a6a977a09` V5 | `ph2d_ecs::sheet_assets`; `ph2d_render::{images_of, drawn_texture, texture_of_entity}`; em uso, índice, retrato do quadro, Painter devolve (`aponta_folhas`), gravar/abrir (`wanted_pixels`), macro grava, duplicar e Stop provados. Mutações 7/7. |

## ABERTO (próxima janela, nesta ordem — §0.10: o resto da V5 é UM bloco)

1. **F13 — Edit in Painter da folha VISÍVEL extra:** `marca_edicao` abre sempre a imagem da folha 0. Com a folha `k ≥ 1` visível tem de abrir a `ExtraSheet.asset` dela (o retorno já reescreve a folha certa: `aponta_folhas`).
2. **F12 — aplicar macro noutro projeto:** gravar embute as folhas extra; aplicar não tem porta para pôr uma imagem crua no `AssetDb` com o seu id (`Portas` só tem `imagem`, que cria objeto). Porta nova em `Portas` + ligação na shell + prova.
3. **Por ler** (decidir: cura ou «folha 0 por definição», com o porquê no handoff): `region/mod.rs:182`, `hero_intents/texture_rebind.rs`, `texture_edit.rs`, `image_export.rs`, `cooked_texture_bridge.rs`.
4. **`recut_users`** (`image_use.rs`, 700/700): o Cut again não passa às folhas extra que usam a mesma imagem.
5. **Nome da folha:** o editor mostra o nome do ficheiro, o cartão o nome da biblioteca (diferem só com nomes repetidos «(2)») — uma fonte só.
6. **Cols/Rows** de uma folha (0 ou extra) não prende quadros que apontam além da grelha nova (já era assim na 0).
7. **V6:** cena `PH2D_GAME_SMOKE=5`, objeto «Two Sheets» (idle + run e uma animação misturada); FOTO (`docs/Components/ferramentas/fotografa_cena.sh`) lado a lado com a maquete ANTES do smoke (painéis 1, 2, 3, 4 e 6), acabamento incluído; smoke em passos numerados. Binário smoke compilado 2× (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`).

## Cuidados

- Tectos: `ph2d-app-sprite-anim` ~7 391/7 545 (folga ganha na V4); `ph2d-app-components` **sem folga** (80 856/80 869 depois do −13 da V5) — porta nova vai a uma folha (`ph2d-anim-sheets`, `ph2d-render`, `ph2d-ecs`) ou à shell como composição; `image_use.rs` 700/700; `fase_recipe_and_asset_verbs` a 200 por função.
- `cargo fmt -p`/`rustfmt` num `lib.rs` ou num ficheiro com filhos `#[path]` reformata ficheiros ALHEIOS: `rustfmt --edition 2024 --check <f>` e corrigir à mão; `scripts/commit-barato.sh` espelha o hook.
- Falha conhecida e ALHEIA: `no_hex_color_literals_in_widget_or_screens` (`command_palette/selecao.rs:108 Color::TRANSPARENT`) — não é desta linha.
- Ainda devido no FECHO da linha (só por ordem do dono): mutações do Add Object do 1.º commit, a rodada anterior sem handoff, o link do §5 no CLAUDE.md, `/pd-linha-fechar`.
