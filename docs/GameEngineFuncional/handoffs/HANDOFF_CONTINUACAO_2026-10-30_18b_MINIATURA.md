# HANDOFF de continuação — line/GameEngineFuncional, onda 18b (A6, a miniatura da câmera) — 30/10

**Smoke do dono: APROVADO (30/10)** — `PH2D_GAME_SMOKE=4`, Camera seleccionada, arrastar, Preview Off|On, Player
apaga; `PH2D_LIGHT2D_SMOKE=7` (luz + Flip). Fotos em [`fotos_18/`](fotos_18/).

## Árvore

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional`, ramo `line/GameEngineFuncional`.
- Base: main `422fec2027`. Sobre ele, 7 commits **não integrados**: os 5 da onda 18 (`39cb3c81fd`..`a9f5957ffe`),
  `32ae7a3363` (plano da A6) e `4faf5458df` (a miniatura). Antes de qualquer rebase/reset: `git cherry main HEAD`.

## Feito (não reconstruir)

Desenho, medição e custo: [`18_miniatura_da_camera_plano.md`](../18_miniatura_da_camera_plano.md) §«Feito — medido».

- `shells/desktop/src/render_loop/vista_da_camera.rs` — `Miniatura`/`Vista`/`Retrato`: a vista da câmera guarda
  as instâncias por vista e faz `mem::swap` com o `AppGfx` (`Vista::troca`) à volta do MESMO present
  (`present_world_sprites` → `present_flip_fx` → `present_mundo_por_cima`), com `na_vista_da_camera = true`.
  Retrato `Rgba8Unorm` registado no Vello; **`mark_texture_dirty` por quadro** (o atlas do Vello é persistente).
- `crates/ph2d-app-game/src/miniatura.rs` — `escolhida`, `vista` (máscara da câmera), `caixa` (canto inferior
  direito, acima de `bottom_hud.y`, abaixo da régua), `desenhar` (fio `ColorToken::Selection`; ponteiro por cima
  ⇒ só o fio), `sob_o_ponteiro`.
- `crates/ph2d-app-light2d/src/quadro_vista.rs` — `LuzDoQuadro::da_vista()` + `codificar_no_alvo`.
- Codificação na `fase_vector_bands.rs` (bloco da miniatura); pedido em `fase_game_camera.rs`; desenho do retrato em
  `fase_selection_highlight.rs`.
- Cartão Camera: «Preview: Off | On» (`CameraCanvasView.preview`, omissão `true`; `CameraFieldEdit::Thumbnail`;
  ids `INSP_CAMERA_THUMB*`; i18n `panel.inspector.camera.preview*`). Foto `Foto::CameraSemPrevia`.
- Gates: `shells/game-scenes/tests/it/jogo_miniatura.rs` (5) · `jogo_encaixe.rs` (testes movidos do
  `ph2d-app-game`, por causa da catraca do crate: 3 179 / tecto 3 202) · agulha em `present_bands_grid_tests.rs`.
- Custo medido (load 18): CPU da vista 1,62–2,29 ms · placa 5,6–6,1 ms (luz da vista 3,29) · 62 fps.
  Instrumentos: `PH2D_MINIATURA_CUSTO=1`, span `render.miniatura` (`PH2D_FLUID_PROFILE=1`), `PH2D_LIGHT2D_CUSTO=1`.

## ABERTO, por ordem

1. **Gaps da A6 — UMA família** (o que o Motion codifica com a câmera do EDITOR fora da placa), num só bloco:
   - **Palcos das macros** (`motion_shell.palcos`): a vista salta-os (`na_vista`). Cura: `PalcoEmCena.placa` por
     vista, trocada como a `PlacaDeFormas` do editor — SEM 2.º estado de reprodução (re-simularia).
   - **Formas do Motion fora da placa**: rota do Vello (`do_editor = motion_tool_active && !placa.ativa()`,
     `fase_vector_overlays.rs:48`) e a camada filtrada (`motion.filtro.alvo_da_cena`). Cura: a mesma
     `motion_shape_gen::encode` com a afim da vista numa cena da vista, colada como faixa.
   - Gate por nome + foto (`PH2D_MOTION_OBJETO_SMOKE=1` + uma cena com macro) antes do smoke.
2. **Piso da grade da luz da vista** (512×320 para um alvo 366×205; 3,29 ms de placa): medir se a grade pode seguir
   o alvo; **sem** limite escrito (§0) — só se pesar no quadro.
3. **Fechar a linha** (DIRETRIZ §1.5.9, `/pd-linha-fechar`): gate batched sobre o diff acumulado, testes por nome,
   fmt + clippy das crates tocadas com `PH2D_SUITE=fecho`, auditoria, `bash scripts/agent-loop-profile.sh`,
   `rm -rf target/*/incremental`, handoff de INTEGRAÇÃO (substitui o link do §5 do CLAUDE.md no fecho).
   Sem suíte inteira, sem push, sem integrar.
