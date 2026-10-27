# Continuação — `line/BlenderConstraints`, W5 «A e B colados» FECHADA (smoke do dono OK, 27/10)

> Anterior: [`W5.1`](HANDOFF_CONTINUACAO_W5_1_2026-10-27.md). Desenho, oráculo, mutações, kill e perfil:
> [`05_W5_diferenciais.md`](../05_W5_diferenciais.md) §3.1, §4.1–§4.3. ADR: [0193 (provisório)](../../architecture/decisions/0193-a-cola-e-uma-relacao-simetrica-resolvida-por-grupo-antes-da-pilha.md). **Não duplicado aqui.**

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`.
`PROJECT_SCHEMA` = **196** (195→196 = `Colas` registado; reconta-se na integração), ADR **0193** provisório (reconta-se),
`VEC_SCENE_SCHEMA_VERSION = 24`, família de canal `CAD = 9`, `Driver::ColaPose` / `Driven::ColaPose` em append no
`ph2d-preview-drive`. Cenas de smoke 1–8; **a próxima é a 9**.

## Feito nesta janela (não refazer)

| Commit | O quê |
|---|---|
| `20477fa276` | Oráculo FreeCAD 1.1.4 Assembly: `ferramentas/oraculo_freecad_assembly.py` + `fixtures/freecad_1_1_4_assembly_cola.json` (5 famílias de casos, resíduo por caso) |
| `4d04e3a756` | Crate `ph2d-constraints-cola`: dado, grupos, sólidos (Rígido contraído), fecho analítico de árvore, `kcl-ezpz` só p/ laços ou ≥2 condutores; 9 passos do quadrilátero = FreeCAD a 1e-6 |
| `7c93d29706` | `ph2d-constraints-ecs::colar()` antes da pilha em `ph2d_app_constraints::quadro()`: mão → documento (undo por diff agrupa), motor → `PreviewDrive` `ColaPose`, `Locked` = chão; `AlvosDasFamilias` (família CONSTRAINT) |
| `7a7860c93e` | ADR-0193 + mutações (05 §4.1: 3/5 na crate — 2 equivalentes documentados —, 4/4 no mundo) |
| `bd6cea2c96` | Gravar/abrir: schema 196, descritor `machinery` (antes de Constraints, ordenado), porta das macros `Recusado(SEM_REMAP)` |
| `9d49e530e0` | UI: Add Constraint ▸ Glue · Rigid / Glue · Pin, cartão nos DOIS objectos com Unglue, pino arrastável no gizmo, cena 8 «Braço articulado» |
| `6af350dcd6` | A mão CEDE quando o grupo não fecha (prioridade menor na ezpz, `R::new(k,1)`) — a manivela presa ao chão roda em vez de rasgar |
| `d40782048a` · `8c7cc87cb8` | Kill medido (5,30/6,18 ms) e curado: **cola 0,44 ms · pilha 0,29 ms por 1 000** (tecto 0,5) — construção exacta antes da ezpz (`construcao.rs`), Kahn com contagens, atalho de ângulo zero no `Transform` (mesmo bit, gate) |
| `6bb6f90f44` | Fecho da onda: fmt + clippy `-D warnings` verdes nas 12 crates tocadas |

Smoke do dono 27/10 (cena 8): **OK**. Decisão de produto: **«deixe como está»** — colar só pelo cartão Constraints
(não vai para o clique direito). Notas do agente da UI que o dono aceitou como estão: o estado «Parado» não mostra frase;
colar o mesmo par duas vezes é aceito.

## ⚠ Para o integrador

- `ph2d-ecs` (`transform.rs`, `transform_inverse.rs`) e `ph2d-preview-drive` (`consultas.rs`, `lib.rs`) mudaram —
  crates de que muitas dependem: a suíte inteira da integração tem de as cobrir.
- Schema 196 e ADR 0193 são provisórios: `python3 scripts/schema-recount.py`; a tripla está em
  `project_schema_tests.rs` `(196, 13, 24)` e o degrau em `project_geometria_v194_tests.rs` (`hoje[0] = 0xC4`).
- Herdados do `main`, NÃO curar na linha: `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`.
- A suíte (nem `nextest-impacted`) NÃO correu na linha (dono 23/10: só antes da integração).

## Portões por nome verdes

Crate: `a_rigid_glue_keeps_the_relative_pose` · `a_pin_keeps_the_points_together_and_each_its_rotation` ·
`two_drivers_that_disagree_are_a_conflict_not_a_fight` · `the_four_bar_matches_freecad_step_by_step` ·
`a_loop_that_does_not_close_is_a_conflict` · `a_hand_that_cannot_go_there_yields_to_the_glue` · os de `construcao_tests`.
Mundo: `dragging_either_glued_object_moves_the_other` · `glued_play_and_scrub_are_the_same_bit` ·
`autokey_records_every_glued_partner` · `a_glue_survives_save_and_open` · `dragging_a_pinned_piece_turns_it_instead_of_tearing_it`.
UI: `the_glue_card_shows_on_both_objects` · `dragging_the_pin_moves_it_on_both_objects` ·
`the_articulated_arm_scene_closes_its_four_pins_on_a_locked_ground` · `every_registered_component_has_a_descriptor`.
Kill (ignorado, só sozinho em `--profile smoke`): `glue_frame_under_half_a_millisecond_per_thousand`.

## Próximo passo — o resto da W5 (plano §7)

Três peças, **desenho antes do código** (como o 05 fez para a cola), num doc novo `06_W5_resto.md`:

1. **Física parâmetro→parâmetro** — **MEDIR ANTES** se `drive` + `spring` (W3) já o exprimem; só construir o que a
   composição não exprime (CLAUDE.md §5.0).
2. ***Behaviours* de um clique** — presets dos nós que já existem (`wiggle`, `oscillator`, `orbit`, `falloff`,
   `force-attractor`); conferir no CÓDIGO o que cada nó já faz antes de acreditar numa ausência.
3. **Layout de HUD** — `kasuari` 0.4.13 (MIT/Apache): triagem de licença por artefacto, oráculo se houver app que o faça.

Perguntar ao dono só produto (ordem das três, onde moram na UI). Cena de smoke seguinte = 9. Kill de cada peça nova
na mesma rodada intercalada.

## Armadilhas novas desta janela

- FreeCAD 1.1 Assembly: refs do joint enraizadas no COMPONENTE (`setJointConnectors`), não no Assembly; criar o joint
  pré-alinha os corpos ⇒ repor as poses autoradas DEPOIS, e só então `GroundedJoint`; `solve()` devolve 0 num laço que
  não fecha — quem decide é o resíduo.
- `Pose2` não tem cisalhamento: pai com escala não-uniforme e filho rodado não faz ida-e-volta (mesmo limite do
  Child Of) — gates com escala uniforme.
- Um grupo onde a mão não pode chegar: sem `cede`, Conflito e rasga; com `cede`, a mão vira alvo macio.
- Um catálogo de descritores é ORDENADO (`every_registered_component_has_a_descriptor`).
- Descritor `authored` para um componente sem campo na paleta vira controlo morto: `machinery`.
- O palpite da ezpz é a pose AUTORADA, para tocar × arrastar a timeline dar o mesmo bit.
