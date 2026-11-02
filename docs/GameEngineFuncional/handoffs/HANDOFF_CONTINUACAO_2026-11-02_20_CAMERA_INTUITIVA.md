# HANDOFF de continuação — GEF 20 câmera intuitiva + indicador de queda (2026-11-02)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base main `3df5894489`
(a linha estava a par do main na reabertura) · HEAD `500354c781`. **A linha continua; NÃO está integrada.**
Anterior: [GEF 19, 4.ª janela](HANDOFF_CONTINUACAO_2026-11-01_19_ORDEM_E_CLIQUE_4.md). Plano com a tabela de
estado (C1–C7 + C2b, todos «feito»): [`20_camera_intuitiva_plano.md`](../20_camera_intuitiva_plano.md).
Decisões do dono: `project-memory/project_camera_intuitive_owner_decisions_2026_11_02.md` — ⚠️ **esse
ficheiro vive no `project-memory/` do checkout principal e está por comitar lá**; o integrador o recolhe.
Smoke do dono: **OK em 02/11** (onda 20 e 20b).

## Commits (`git log 3df5894489..HEAD`)

| Commit | O quê |
|---|---|
| `2fe470dab5` | os dois testes que a onda 19 devia: gate de placa `as_duas_metades_entregam_a_faixa_motion_a_colagem` (`shells/desktop/src/render_loop/present_placa_tests.rs`, `#[ignore]` placa) + assert do tecto da câmera do nível 2 em `shells/game-scenes/tests/it/jogo_camara.rs` |
| `bf5828f316` | onda 20, C1–C7 |
| `046a10bbe4` · `795bbe4896` · `1e23193cf4` | 20b: indicador de compatibilidade da queda |
| `500354c781` | clippy `useless_conversion` no teste do conta-gotas |

## Feito — NÃO reconstruir

- **C1 dicas:** `populate_camera::dicas` (23 grupos); chaves i18n `panel.inspector.camera.tip.*`
  (`ph2d-i18n/src/inspector_game.rs`). Toda linha da câmera tem tooltip.
- **C2 alvo por arrastar / conta-gotas:** arrastar da Hierarchy → `DropTarget::CameraTarget` /
  `DropAction::SetCameraTarget`; botão `INSP_CAMERA_TARGET_PICK` → `CameraFieldEdit::PickTarget` →
  `CameraCanvasView::escolhe_alvo`; clique no canvas passa pelo guarda `joint_body_pick` →
  `ph2d_camera_frame::escolhe_alvo`; `Esc` desarma. **Limite conhecido:** o conta-gotas só apanha sprites
  (vector/Flip: por arrastar ou pelo nome). ⛔ **Lista/dropdown de objectos RECUSADA pelo dono** (cenas
  com objectos demais) — não reabrir.
- **C3 zona morta no meio da moldura:** `centro_da_zona = centro − offset` (`ph2d-camera-frame`).
- **C4 gizmo à Vector:** `Cara` por família, halo, chip de leitura junto ao ponteiro, chips «Limits» /
  «Dead Zone»; `Esc` restaura, duplo clique repõe (`padrao` + `UltimoPress`), `Shift` encaixa (`encaixa`).
- **C5 rótulos** dentro do canvas utilizável (`tinta::canvas_util`) e chip «follows X» a meio do fio.
- **C6 limites nascem no contorno do nível:** `contorno_do_nivel` + `semeia_limites`; nível 2 =
  `[-13,-1]..[17,7]`. **C7:** cena 2/4 abre a enquadrar o jogador (`SetViewFocus::Selected`).
- **C2b indicador de queda (como no Godot):** com um arrasto armado, todo campo do Inspector que ACEITA o
  payload ganha contorno fino verde; o campo sob o cursor ganha contorno GROSSO, verde (aceita) ou
  vermelho (recusa). **Contorno, nunca preenchimento** (dono: «um stroke ficaria melhor que o fill»).
  - Lei pura em `ph2d-app-components` `asset_drop::{field_target, verdict, field_marks}` = **porta única**
    dos três campos receptores (slot de textura, campo de objecto do Script, Target da câmera);
    `drop_target_at` da shell passa a usar `field_target` (três picks → um).
  - `ph2d-editor-core`: `DropFieldMark` (`interaction/drag_payload.rs`),
    `WidgetStore::set_asset_drop_fields`/`asset_drop_fields` (limpos em begin/end),
    `asset_drag_ghost::paint_drop_fields` pinta antes do fantasma, com o rect do `HitIndex`.
  - ⛔ **Defeito achado por FOTO:** pela porta do tema (`stroke_frame` + `Feel::Active`) o verde SUMIA — a
    família moderna apaga os quadros de estado. Agora `stroke_rounded_rect` directo, com `FRAME-RAW-OK`
    (gate `every_frame_goes_through_the_theme_door`). Também `paint_hover_tooltip` devolve `false` com
    arrasto armado (a dica cobria o fantasma).
  - Foto: `PH2D_GAME_SMOKE_FOTO=camera_larga` (`object_drag::smoke_ao_alvo` semeia o Player e leva o
    ponteiro ao Target sem soltar) — 1.ª foto de um *hover* de arrasto nesta linha.

## Superfície de colisão (`collision-surface.sh`, caminho absoluto, merge-base `3df5894489`)

38 ficheiros, +1 595 / −144, 6 commits. Novos: `crates/ph2d-app-components/src/asset_drop_field_tests.rs`,
`docs/GameEngineFuncional/20_camera_intuitiva_plano.md`, `shells/game-scenes/tests/it/jogo_camera_intuitiva.rs`.

- Schemas **sem delta**: `PROJECT_SCHEMA 213` (tripla `(213,13,27)`), `VEC_SCENE_SCHEMA 27`, `FLIP_SCHEMA 13`,
  `DOC_VERSION 25`. Registo de componentes **sem delta** (116 / 117 / 117). Nenhum `+name` novo no `Cargo.lock`.
- ADR: nenhum criado (próximo livre 0197). Contratos §6 (`node.rs`, `tool.rs`) intocados. Sem marcadores de conflito.
- Crates tocadas: `ph2d-app-components`, `ph2d-app-game`, `ph2d-camera-frame`, `ph2d-editor-core`,
  `ph2d-i18n`, `ph2d-panel-inspector`, `shells/desktop`, `shells/game-scenes`.
- Tectos de LOC: o mapa marcou dois `✗`. O gate `architecture_workspace_file_loc_cap` reprovou
  `ph2d-camera-frame/src/lib.rs` (738/700) — **curado por corte**: os gestos das alças e os limites do contorno
  foram para o irmão `src/gestos.rs` (lib 557 · gestos 188); gate verde. O outro
  (`despacho_metodos_picks_e_arrastos.rs` 603/600, shell) o gate não censa (os ficheiros da shell respondem à
  catraca `the_shell_only_shrinks`, verde); margem de 2 linhas na catraca.
- Shell: `the_shell_only_shrinks` verde (tecto 189 041; a lógica da foto foi para `app-components`, comentários
  da shell compactados).

## O que um merge pode partir

- **Mesmo símbolo:** `drop_target_at` (shell), `asset_drag_wire.rs`, `camera_no_canvas.rs`,
  `inspector_model_camera.rs`, `ph2d-camera-frame/src/{lib,tinta}.rs` — quem tocar a câmera, o arrasto de
  assets ou o Script object field noutra linha colide aqui.
- Linha que adicione um 4.º campo receptor de queda deve entrar por `asset_drop::field_target`/`field_marks`,
  não por um pick novo na shell.
- Qualquer `stroke_frame` novo sobre estado `Active` na família moderna some (lição do verde).

## Prova de fecho (verde, POR NOME)

`jogo_camera_intuitiva` 7 · `jogo_moldura` 15 · `jogo_camara` 3 · `the_camera_section_is_alive` 11 ·
`asset_drop` `field_tests` 2 · `every_frame_goes_through_the_theme_door` 3 · `the_shell_only_shrinks` 2 ·
`smoke_mao` `a_mao_desce` 1. Mutação: onda 20 **14/14 RED**; 20b **3/3 RED**. `clippy -D warnings` nas 8
crates tocadas e `fmt` verdes. A suíte completa NÃO correu (só na integração).
`fmt` com diffs **que NÃO são desta linha** (já no main): `ph2d-panel-pixel-preview/src/paint.rs`,
`ph2d-panel-registry-init/tests/it/the_vector_control_bar.rs`, `ph2d-vec-entities/src/connector_handles.rs`.

## Smokes (comando inteiro)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Hierarchy → **Camera**; Inspector → **Game Camera**.
3. Arrastar **Player** até **Target**: contorno grosso verde, sem dica; soltar escreve o nome.
   Errado se: nenhum contorno · a dica aparece · uma imagem de Assets fica verde (tem de ser vermelha).
4. Gizmo: `Esc` durante o arrasto de uma alça restaura · duplo clique na alça repõe · `Shift` encaixa.
- Foto: `PH2D_GAME_SMOKE_FOTO=camera_larga` (mesma pasta/binário).

## ABERTO

1. Catraca da shell com 2 linhas de margem — a próxima onda que tocar a shell tem de MOVER antes de crescer.
2. Conta-gotas só apanha sprites (limite conhecido, C2).
3. Gate de placa `as_duas_metades_entregam_a_faixa_motion_a_colagem` (`#[ignore]`): **verde na placa** 02/11
   (`PH2D_GPU=1 … --profile smoke -- --ignored`, 0,33 s).
4. As abertas da 4.ª janela da onda 19 que não constam acima (mutações em lote, rodada de perf do
   `LayerCompositor`) seguem no [handoff 19/4](HANDOFF_CONTINUACAO_2026-11-01_19_ORDEM_E_CLIQUE_4.md).
5. **Próximo item: J5 moedas** (moedas com som + contador no HUD), plano
   [`03_plano_jogo_de_plataforma.md`](../03_plano_jogo_de_plataforma.md) — a onda seguinte começa em
   **janela NOVA**. Integrar/shipar só por ordem do dono.
