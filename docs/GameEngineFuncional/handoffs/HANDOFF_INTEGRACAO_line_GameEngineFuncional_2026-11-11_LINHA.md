# HANDOFF de integração — `line/GameEngineFuncional`, 11/11 (a LINHA inteira: Pivot + Character + curva do pulo)

Leitor: o agente integrador. Este é o documento de ENTRADA; o mecanismo, as mutações e as listas de ficheiros vivem nos três handoffs de onda (§1) — leia-os se um conflito pedir.

- Branch `line/GameEngineFuncional` · worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional`.
- Merge-base `bc4a6ce930` = ponta do `main` (conferido: `git merge-base HEAD main` == `main`; **sem rebase, `--ff-only` serve** se o `main` não andar).
- HEAD `e099f9be1b` · **30 commits** · **178 ficheiros** · +5942 −1139 (antes deste doc) · árvore limpa.
- **Nada integrado, nenhum push.** Os três smokes do dono: **OK em 10/11** (Pivot + Align ao criar + onda dos abertos; Character; curva do pulo).
- **NÃO correram (são do integrador):** suíte completa, `nextest-impacted`, `machete`, B2.

## 1. As três ondas

| # | Commits (`git log --reverse`) | Entrega | Handoff |
|---|---|---|---|
| 1 | `a271b50367`..`cc9f1a1eb8` (11) | ABERTOS do Pivot: executor da cena 5 → `ph2d-sprite-anim-scene`; `FolhasAVista` atravessa o Ctrl+Z; «+» escolhe a nova; Align sai ao criar; `AnimDoc::arriving`; onda `401d4d5e3c` (`Cmd+`/`Ctrl+` por sistema, estouro de pilha do `TocadorDoPainter`, censo da cena 5, `Commit::FeitoComAviso`, «INSERT HERE») | [ABERTOS_DO_PIVOT](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_ABERTOS_DO_PIVOT.md) |
| 2 | `c77af873d2`..`c28e15d5da` (7) | Character em cartões (versão B): sete `*_on` em `PlatformPlayer`, catálogo 55–61; **fundação** `corte_da_dobra` e `chave_da_altura` (`ph2d-editor-core`) | [CHARACTER](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_CHARACTER.md) |
| 3 | `d2bd3f1781`..`e099f9be1b` (12) | Curva do pulo no canvas: lei `ph2d_platformer::arc`+`feel`, caixa Show Jump Arc, seletor Feel, alça `JumpApex`, catálogo 62; desenho desce a `ph2d-panel-physics` | [CURVA_DO_PULO](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_CURVA_DO_PULO.md) |

Fichas: [`03_character.md`](../componentes/03_character.md) · [`04_curva_do_pulo.md`](../componentes/04_curva_do_pulo.md).

## 2. Superfície de colisão AGREGADA

Colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional/scripts/collision-surface.sh` (11/11):

```
merge-base bc4a6ce930 · 30 commit(s) · 178 arquivo(s)
PROJECT_SCHEMA 233 (base 231) · tripla (233, 13, 29) (base (231, 13, 29)) · VEC 29 · FLIP 13 · DOC 25 = base
REGISTRO ph2d-ecs 121 (base 121) · espelhos render/script: SONDA CEGA (✗, como nos três handoffs)
CONTRATO node.rs / tool.rs: intocado · ADR: nenhum (próximo livre 0201)
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
LOC: ✗ 1034/600 shells/desktop/src/main.rs — PRÉ-EXISTENTE (1031 linhas no merge-base; a linha soma +4 −1, a isenção «Ctrl+Q» da onda 1)
```

Ficheiros foundational/partilhados tocados (`git diff --name-only bc4a6ce930..HEAD`):

| Zona | Ficheiros | Onda |
|---|---|---|
| `ph2d-editor-core` (fundação) | `widget/section_header/body.rs`, `panel/cards_compacta.rs`, `widget/mod.rs` (dobra: `corte_da_dobra`, `chave_da_altura`); `gizmo/point.rs` (`PointHandleKind::JumpApex`), `gizmo/hit.rs` (`GIZMO_JUMP_APEX = NodeId(978)`); `screens/hero/inspector_model_player.rs` (`PlayerFieldEdit::{ShowJumpArc, Feel(u8)}`, `PlayerInfo.{show_jump_arc, feel}`) | 2, 3 |
| `ph2d-ecs` / `ph2d-host-presos` / `ph2d-anim-editor` | `FolhasAVista` (aditivo), `AntesDoRestauro` com campos privados + `repoe_a_sessao`, `align_off`, `AnimDoc::arriving` | 1 |
| Schema | `shells/desktop/src/project_schema.rs` + `project_schema_tests.rs` | 2, 3 |
| Catálogo físico | `ph2d-component-desc/src/catalog/physics_fields.rs` (campos 55–62) + `ph2d-i18n/src/component_catalog.rs` | 2, 3 |
| `ph2d-i18n` | `atalho.rs` (novo), `lib.rs`, `chrome_menus/chrome_panes/code_editor/shell_media.rs` (33 entradas `Cmd+`→`Ctrl+`), `inspector_player.rs`, `app_physics.rs`, `anim_editor.rs`, teste `a_tecla_dos_atalhos_vem_da_porta` | 1, 2, 3 |
| Shell `shells/desktop` | `undo_app.rs`, `render_loop/inspector_commits.rs`, `janela_do_jogo_animado.rs`, `main.rs`, `every_word_the_shell_shows_comes_from_the_string_table.rs`; Character: `physics/inspector_player*_tests.rs`; Curva: `fase_physics_overlay.rs`, `fase_snapshot_readouts.rs`, `fase_physics_edits.rs`, `despacho_clique_gizmo.rs`, `despacho_clique_largar.rs`, `joint_gestures_app.rs`, `asset_drop_apply.rs` | 1, 2, 3 |
| `Cargo.toml` / `Cargo.lock` | `ph2d-anim-sheets` (+dep `ph2d-physics-ecs`), `ph2d-sprite-anim-scene`, `shells/presos` (dev-dep `ph2d-asset`), `ph2d-panel-physics` (`ph2d-ecs` dev→dep, `+ph2d-render`), `Cargo.lock` (sem pacote externo novo) | 1, 3 |

**Não-colisão com as outras linhas** — `git diff --name-only bc4a6ce930..HEAD | grep -E 'bridge/|physics_apply|ph2d-input|input_map_ops'` → **vazio (grep exit 1)**. A linha NÃO tocou nos paths da `line-GameEngineFuncional02` (`ph2d-physics-ecs/src/bridge/*`, `physics_apply.rs`, juntas, formas de colisão, Outline, cartão Trigger) nem da `line-InputMap` (`ph2d-input`, `input_map_ops.rs`). Ressalva: `joint_anchor_drag.rs` (`ph2d-app-physics`) e `joint_gestures_app.rs` (shell) FORAM tocados pela onda 3 (braço `=> None` de `param_value`; arrasto da alça) — ficheiros de juntas fora da lista do grep; se a GEF02 também mexeu neles, conflito textual.

## 3. Degraus do `PROJECT_SCHEMA`

A linha soma **2 degraus**: 231 → **232** (Character: sete `*_on` no FIM de `PlatformPlayer`) → **233** (curva: `show_jump_arc` no FIM). Hoje no código: `PROJECT_SCHEMA = 233` (`project_schema.rs:432`), tripla `(233, 13, 29)` (`project_schema_tests.rs:374`). A onda 1 não mexeu (231). Sem leitor do antigo (§0.11). Registos: DELTA 0. **O integrador RECONTA, não escolhe:** `python3 scripts/schema-recount.py` contra o `main` do dia; os dois blocos de comentário («`231 → 232`», «`232 → 233`») e a tripla são três sítios irmãos. O degrau 233 só vale se o 232 vier antes, na mesma linha.

## 4. Catracas de LOC e folga

- `ph2d-app-physics` (`the_shell_only_shrinks_and_every_app_crate_too`): chegou a 38 126 > tecto 38 085; cura = desenho da curva desce a `ph2d-panel-physics` → **37 823 / 38 085** (folga 262).
- `ph2d-app-sprite-anim`: 5 285 > 5 209; cura = `tiled.rs` e `previa.rs` → `ph2d-anim-sheets` → **5 173 / 5 209** (folga 36; a catraca conta `src/` E `tests/`).
- `ph2d-panel-inspector`: `sections/player.rs` −363; `player_cartoes.rs` 592 → 566 (depois de `player_pulo.rs`), `player_grupos.rs` novo — tectos de ficheiro cumpridos.
- Shell: `janela_do_jogo_animado.rs` 356 → 145; `the_shell_only_shrinks` verde nos três fechos. Tabela de strings da shell: −3 isenções (onda 1) e +1 (`main.rs` «Ctrl+Q»).
- 14 linhas de comentário do `component_catalog.rs` foram para o [arquivo](../../archive/integracao-jornadas/COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-10.md).

## 5. Portões corridos no fecho (pela linha)

- **Onda 1:** fmt + clippy `-D warnings` de 9 crates, depois 9 na onda `401d4d5e3c`; `the_shell_only_shrinks*`, `no_dependency_climbs_a_layer`, LOC, órfãos, HR-15, `no_literal_color`, `no_tofu_glyphs`, os 3 da tabela da shell, `the_only_restore_door_of_the_shell_calls_the_two_halves_in_order`; testes `editor_animacoes` 89/89, `ph2d-anim-sheets` 61/61, `inspector` 22, `tutorial_painter`, `smoke` do app-game.
- **Onda 2:** fmt/clippy das crates tocadas; LOC, HR-15, cor, números mágicos; `seam_` **254/254**.
- **Onda 3:** fmt/clippy `-D warnings` de 11 crates (inclui `ph2d-panel-physics`); `arc::tests` 9, `o_arco_do_desenho_e_o_do_jogo` (12 casos), `jump_arc::tests` 5, `curva_do_pulo`, `point_gizmo`, `seam_player`, `seam_player_pulo` 4, `o_inspector_armado`, `an_undo_that_keeps_the_cards_moves_nothing`, `project_schema`, `inspector_player`, gates de LOC/cor/órfão/NodeId e os dois do catálogo. Não existe gate que compare o catálogo com os campos de `PlatformPlayer` (campo 62 conferido à mão).
- **O integrador corre:** suíte completa (`scripts/suite-completa.sh` / `integrar-rodada.sh`), `nextest-impacted`, `machete`, B2; e recontar o schema (§3). Custo medido da curva: 11,8 µs por `jump_arcs` (release), 2 por quadro.

## 6. Mutações (verdes antes, vermelhas com a mutação, restauradas)

- Onda 1: M1–M7 do Pivot (M6 sobreviveu na 1.ª rodada, fechada por `a_porta_do_restauro_leva_a_folha_a_vista_so_num_passo`) + M1–M8 da onda `401d4d5e3c` + 1 em `c93ce28b8c` (`arriving` com intervalo vazio).
- Onda 2: **16/16 sangram** (M1c e M10 sobreviviam e ganharam teste).
- Onda 3: **13/13 sangram** (M13 curado em `3050d9a0ba`).

## 7. O que um merge pode partir

- `PlatformPlayer { .. }` literal completo noutra linha: faltam os 8 campos novos (`walls_on`, `dash_on`, `crouch_on`, `swim_on`, `ledge_on`, `glide_on`, `fall_limit_on`, `show_jump_arc`) → `..Default::default()`.
- Teste/cena que ligue Dash, Walls, Crouch, Swim, Ledge, Glide ou Fall Limit só pelo número: agora a caixa `*_on` também (≈45 sítios curados aqui).
- `PlayerInfo { .. }` literal sem `show_jump_arc`/`feel`; `match` exaustivo sobre `PointHandleKind` (braço `JumpApex`) ou sobre `inspector::Commit` (variante `FeitoComAviso`); quem use `NodeId(978)`.
- `AntesDoRestauro` por literal (campos privados); `AnimEditorAlignRequests` com `u64` (agora `(u64, bool)`); implementadores de `Mesa` (`editor` → `alvos`).
- Quem escreva `"Cmd+"` num texto (o censo reprova: `Ctrl+`).
- `git grep -n 'section_body_h\|remember_section_body_h'` (a memória da altura é por modo): testes de geometria de dobra de outra linha podem mudar de número.
- Campos novos no catálogo da física (intervalo 55–62) noutra linha: recontar. Conflito textual provável em `seam_player.rs`, `component_catalog.rs`, `inspector_player.rs`, `sections/player.rs`, `player_cartoes.rs` e nos ficheiros da shell do §2.
- Quem chame `apply_player_edit` continua a compilar (delega), mas sem os `PhysicsSettings` da cena; a API nova é `apply_player_edit_in`.

## 8. Smokes para reconferir depois do rebase

Compilar uma vez: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop` (sem `Compiling` na 2.ª corrida). Depois, do mesmo diretório:

1. Cena 5 (Pivot, Align, atalhos): `PH2D_GAME_SMOKE=5 ./target/smoke/ph2d-host-desktop` — Edit diz «Ctrl+Z»/«Ctrl+Shift+Z»; Ctrl+Z na folha `run` mantém a `run`; «+ Add» escolhe a nova e sai do Align; «INSERT HERE» depois do último quadro.
2. Character e curva (cena do plataforma): `PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop` — Hierarchy ▸ Player ▸ Platform Player: 12 dobras abrem pelo clique; Dash marcado + Play + Q arranca; dobra Jump ▸ Show Jump Arc desenha quatro curvas; a bolinha do topo muda Jump Height (cima) e Speed (lado); Feel Floaty/Snappy/Heavy mantém o ápice.
3. Fotos: `PH2D_GAME_SMOKE_FOTO=player_cartoes` (`PH2D_PLAYER_DOBRAS=1` abre as dobras) e `PH2D_GAME_SMOKE_FOTO=fisica_player_pulo`; saídas em [`fotos_03/`](../componentes/fotos_03/) e [`fotos_04/`](../componentes/fotos_04/); da cena 5 em [`fotos_30/`](../fotos_30/), [`fotos_31/`](../fotos_31/).
4. Errado: dobra que não abre, campo atrasado, caixa que perde o número, curva que não bate com o pulo no Play, «Cmd» no menu Edit, `ShownSheet` que volta a 0 depois do Ctrl+Z.

## 9. ABERTO (consolidado, sem duplicar)

- **[onda 1]** `janela_do_jogo_smoke.rs` (214 linhas na shell), da GEF02: a `Mesa` serve-lhe de molde. `fotos_29/animado_pivot_segue_desfaz.log` desatualizado (a 11.ª linha mudou por desenho). Cancel de prefab deixa de levar a folha à vista (aceite).
- **[ondas 2 e 3]** O gate de geometria do `seam_player` prova menos («nenhuma row se sobrepõe» no lugar do passo da moldura por card); as 12 dobras sem dica de hover (isentas, como câmera e HUD); `open_all_sections` não abre as dobras novas.
- **[onda 3]** A curva termina na linha dos pés (não segue o terreno); Floaty/Snappy/Heavy são decisão técnica aprovada no smoke, afináveis; o campo Jump Height mostra a altura de gravidade neutra (2,00) e a alça o ápice real (2,04).
- **[integração]** Reconto do schema (§3); suíte completa, `nextest-impacted`, `machete`, B2; sonda cega dos espelhos `ph2d-render`/`ph2d-script` do registo (caminho do script, pré-existente).

## 10. Divergências entre os três handoffs (conferidas)

- O handoff do Pivot abre com «6 commits, 36 ficheiros» e cola uma superfície de «5 commits»: está congelado antes das ondas `c93ce28b8c` e `401d4d5e3c`; o intervalo real da onda 1 é 11 commits (§1). Os números do §2 deste doc são os do HEAD.
- O handoff da curva diz em §3 «`show_jump_arc` (e o feel)» no FIM de `PlatformPlayer`: só `show_jump_arc` é campo (`player.rs:389`); o Feel deriva-se do ajuste da física (`feel_of` em `ph2d-platformer`), não é gravado. `PROJECT_SCHEMA` 233 cobre só `show_jump_arc`.
- `editor_animacoes`: 88/88 (fecho do Pivot) × 89/89 (depois de `c93ce28b8c`) — evolução, não contradição.
- O handoff do Character diz «versão B» e fala de 5 commits do item; o `git log` mostra 7 no intervalo (`c77af873d2`..`c28e15d5da`: +`8d05ad1682` ficha e `c28e15d5da` docs).
