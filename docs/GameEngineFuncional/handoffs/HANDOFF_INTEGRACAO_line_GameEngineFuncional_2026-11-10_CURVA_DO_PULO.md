# HANDOFF de integração — `line/GameEngineFuncional`, 10/11 (GEF 04: a curva do pulo no canvas)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, base `bc4a6ce930` (= ponta do `main` quando a linha abriu). Esta linha leva TAMBÉM os commits do Character ([handoff](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_CHARACTER.md)) e dos ABERTOS do Pivot ([handoff](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-10_ABERTOS_DO_PIVOT.md)); integre a linha inteira. Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10): são do integrador. Ficha: [`04_curva_do_pulo.md`](../componentes/04_curva_do_pulo.md), secção «O que foi construído».

## 1. Commits do item (`git log --oneline c28e15d5da..HEAD`)

| Commit | O que |
|---|---|
| `d2bd3f1781` | ficha: lei, alça, presets simulados, maquete A/B |
| `ad422d44ae` | a lei `ph2d_platformer::arc` + `feel` (Floaty·Snappy·Heavy conservam o ápice) + gate contra o player real |
| `39bfcc6af7` | caixa Show Jump Arc e seletor Feel no cartão Jump, alça `JumpApex`, quatro curvas no canvas, schema 233 |
| `65c8fe9673` | cor do pulo do ar em `ObjMacro` (o Accent sai azul no tema escuro e confundia-se com o parado); fotos; fmt |
| `82ce129d70` | ficha: o que foi construído, medido e fotografado |
| `3050d9a0ba` | cura do M13 (único sobrevivente de 13 mutações): a caixa e o Feel do cartão chegam ao componente e ao canvas |

## 2. ⚠️ Alerta: FUNDAÇÃO tocada (`ph2d-editor-core`)

- `gizmo/point.rs`: `PointHandleKind::JumpApex` (hit = diâmetro padrão, pintado como o ponto A, rank 10, multiplicador próprio).
- `gizmo/hit.rs`: `GIZMO_JUMP_APEX = NodeId(978)`.
- Outra linha que acrescente variante a `PointHandleKind` ou use `NodeId(978)` conflita. `match` exaustivo sobre `PointHandleKind` noutra linha precisa do braço novo (aqui: `joint_anchor_drag.rs::param_value`, `=> None`). `git grep -n 'PointHandleKind::'`.
- `screens/hero/inspector_model_player.rs`: `PlayerFieldEdit::{ShowJumpArc, Feel(u8)}`, `PlayerInfo.{show_jump_arc, feel}` — fixture `PlayerInfo { .. }` literal noutra linha precisa dos dois campos.

## 3. Crates e ficheiros tocados (só o item)

- `ph2d-platformer`: `arc` (lei) + `feel`; testes `arc::tests` (9).
- `ph2d-physics-ecs`: `PlatformPlayer.show_jump_arc` (e o feel) no FIM do tipo.
- `ph2d-component-desc` / `ph2d-i18n`: campo 62 (§4); 4 chaves i18n mortas apagadas de `inspector_player.rs` (`nobody_hears_him_he_lands`, `publish_what_he_does_as`, `player.off`, `player.on`) para o gate de chaves ficar verde.
- `ph2d-editor-core`: §2.
- `ph2d-app-physics`: `jump_arc.rs` (novo; `jump_arcs`, alça; `jump_arc::tests` 5), `joint_anchor_drag.rs`; **API nova** `inspector::player::apply_player_edit_in(sim, bits, edit, &PhysicsSettings)` — o antigo `apply_player_edit` delega com `PhysicsSettings::default()`.
- `ph2d-panel-inspector`: `player_pulo.rs` novo (a função `cartao` Walk/Jump saiu de `player_cartoes.rs`, 592 → 566); ids `INSP_PLAYER_SHOW_ARC`, `INSP_PLAYER_FEEL`, `INSP_PLAYER_FEEL_IDS`; `tests/it/seam_player_pulo` (4).
- `shells/desktop` (conflito textual provável): `fase_physics_overlay.rs` (desenho), `fase_snapshot_readouts.rs` (alça), `despacho_clique_gizmo.rs` (abrir arrasto), `joint_gestures_app.rs` (seguir), `despacho_clique_largar.rs` (soltar), `asset_drop_apply.rs` (foto), `fase_physics_edits.rs` (chama `apply_player_edit_in` com `physics.settings()`), `project_schema.rs` (+ `_tests`). `the_shell_only_shrinks` verde.

## 4. Schema e catálogo

- `PROJECT_SCHEMA` **232 → 233**: `PlatformPlayer.show_jump_arc` no FIM (postcard posicional). Sem leitor do antigo (§0.11). Registos: DELTA 0; a tripla não vê o degrau.
- **Reconte pelo `main` do dia:** `python3 scripts/schema-recount.py`. Degrau em `shells/desktop/src/project_schema.rs`, bloco «`232 → 233`»; linha da tripla `(233, 13, 29)` em `project_schema_tests.rs`. Este degrau parte de 232 (o do Character): se o Character entrar antes, os dois contam em sequência.
- Catálogo: **campo 62** `component.field.platform_player.62` «Show Jump Arc» (`ph2d-component-desc/src/catalog/physics_fields.rs`, `ph2d-i18n/src/component_catalog.rs`). Outra linha com campos novos no catálogo da física reconta.

## 5. O que um merge pode partir

- Alerta do §2 (variante, `NodeId(978)`, `PlayerInfo` literal).
- Conflito textual nos sete ficheiros da shell do §3 e em `player_cartoes.rs`/`player_pulo.rs` se outra linha mexeu no cartão do player.
- Literal `PlatformPlayer { .. }` completo noutra linha: falta `show_jump_arc` (use `..Default::default()`).
- Quem chamar `apply_player_edit` continua a compilar (delega), mas sem os `PhysicsSettings` da cena.

## 6. Prova de fecho

Corrido (pela linha): gate `o_arco_do_desenho_e_o_do_jogo` (12 casos; altura ≤ 1,3 cm, pouso no mesmo tique, alcance < 1 mm); `arc::tests` 9; `jump_arc::tests` 5; `seam_player_pulo` 4 (clique real, Feel aceso/None, animação da dobra); mutação **13/13 sangram** (M13 curado em `3050d9a0ba`); custo 11,8 µs por `jump_arcs` (release), 2 por quadro; fmt/clippy das crates tocadas; `the_shell_only_shrinks` verde. Smoke do dono OK em 10/11 (versão B, linhas coloridas).

**NÃO corrido (é do integrador):** suíte completa, `nextest-impacted`, `machete`, B2.

## 7. Smoke (já feito pelo dono em 10/11; para repetir)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Hierarchy ▸ Player; no cartão Platform Player, dobra Jump ▸ marque **Show Jump Arc**: aparecem no canvas as curvas (pulo inteiro forte, curto, parado e correndo tracejados, e no ar com Air Jumps 1).
3. Arraste a bolinha do topo: para cima muda Jump Height, para o lado muda Speed. **Feel** Floaty / Snappy / Heavy: a curva muda de forma e o ápice fica.
4. Errado: caixa que não desenha nada, bolinha que não arrasta, ou curva que não bate com o pulo no Play.

Fotos: [`fotos_04/`](../componentes/fotos_04/) (`PH2D_GAME_SMOKE_FOTO=fisica_player_pulo`).

## 8. ABERTO

- A curva termina na linha dos pés (não segue o terreno).
- Floaty/Snappy/Heavy: valores são decisão técnica aprovada no smoke, afináveis.
- O campo Jump Height mostra a altura de gravidade neutra (2,00); a alça mostra o ápice real (2,04).
- Herdados do Character: o gate de geometria do `seam_player` prova menos; as dobras sem dica de hover; `open_all_sections` não abre as dobras novas.
