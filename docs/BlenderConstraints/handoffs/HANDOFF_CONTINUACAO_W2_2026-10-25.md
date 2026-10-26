# Continuação — `line/BlenderConstraints`, onda W2 (ossos), 25/10/2026

> Anterior: [`W1d`](HANDOFF_CONTINUACAO_W1d_2026-10-25.md) (W1 fechada, smoke OK). Plano: [`01_plano.md`](../01_plano.md) §7 W2 ·
> desenho e recusas medidas: [`02_W2_ossos.md`](../02_W2_ossos.md) · ADR [`0192`](../../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md).

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou).
HEAD = o commit deste handoff, árvore limpa. Commits da onda: `53800503fa` oráculo+desenho · `3365e912cd` leis ·
`22b94045dd` preview-drive (camadas) · `63f691002d` IK na pilha · `c8d06da00a` nó 2 ossos → `reach` · `ec37000ba8` API do
item IK + conversão + ordem do quadro · `c3f815376a` migração do goal, cartão, Follow Path, cena 4 · `4b620dea6b` cena abre
o cartão · `f7c0901059` fecho (fmt/clippy, gate da conversão). Binário `target/smoke/ph2d-host-desktop` compilado no HEAD.
Shell: encolheu 8 linhas na onda (catraca `the_shell_only_shrinks` verde). `PROJECT_SCHEMA` continua 194 (Kind cresce ao
FIM do enum; o `IkGoal` continua registado para ler ficheiros antigos).

## Smoke do dono (cena 4) — por fazer

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-BlenderConstraints && PH2D_CONSTRAINT_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Abre com o braço (dois ossos cor de pele), a bola vermelha (alvo da IK) e o ponteiro cinzento «Rodador». O osso
   «Antebraço» vem escolhido e o cartão Constraints aberto: **IK** (alvo Bola) em cima, **Copy Rotation** (alvo Rodador) abaixo.
3. Arrastar a bola: o braço dobra para a seguir, o cotovelo sempre do mesmo lado. O antebraço fica paralelo ao Rodador.
4. Rodar o Rodador (ferramenta ROT): o antebraço roda com ele — ao contrário do Blender, onde o Copy Rotation some.
5. No cartão, «Move Down» na linha IK: agora a ponta do antebraço toca a bola (quem está em baixo vence).
6. Errado = o antebraço aponta para a bola com a IK em cima; o cotovelo troca de lado ou treme parado; o braço salta.

Foto conferida (`fotografa_cena.sh`, 1930×1040): antebraço paralelo ao Rodador, linhas tracejadas aos dois alvos, cartão com
Chain 2 · Softness 0 % · Bend Auto/CCW/CW/Mixed · Stretch No/Yes.

## Feito

| Peça | Onde | Prova (por nome, verde) |
|---|---|---|
| **Oráculo W2** antes do código: `ik_dois_ossos` (+ espelho), `ik_estica`, `ik_cadeia_3`, `stretch_to` (4 volumes × 2 distâncias), `follow_path` (L e meio círculo × 5 fracções). Os casos W1 saem iguais ao bit | `ferramentas/oraculo_blender.py`, fixture | — |
| **Recusas medidas**: o stretch do Blender encolhe o braço ao alcance (0,631/0,952) e dobra-o fora; a tangente da poligonal roda 19,69° a meio de um trecho recto. A Softness não tem oráculo corrível (o Blender não a tem; o Spine só com interface): fica `softened_distance` | `02_W2_ossos.md` §1 | gates do lado contrário |
| **Leis** (`ph2d-constraints`): `Kind::Ik/StretchTo/FollowPath` ao fim do enum; `solve_chain` (reach + Stretch: resolve para o alvo ÷ k, estica pela raiz; contínuo no joelho), lado forçado troca sob espelho (`bend_in_world`); Stretch To (aponta +X, `sx·d/rest`, largura `1/√s`·`1/s`·`1`); Follow Path (fracção/distância, fechado dá a volta, tangente do segmento); `resolve_with`/`apply_with` levam a poligonal | `ossos.rs`, `caminho.rs`, `kind.rs` | `oraculo_ossos`: `ik_matches_the_blender_two_bone_reach` (resíduo do solver iterativo do Blender medido: `TOL_IK` 3e-3; ao alcance a nossa ponta toca ao bit), `the_forced_side_is_the_chains_and_flips_with_the_mirror`, `a_chain_of_three_reaches_with_rigid_bones`, `the_stretch_only_grows_past_the_knee_and_reaches_the_target`, `stretch_to_matches_blender_volume`, `follow_path_goes_by_arc_length_and_faces_the_tangent` |
| **A IK é um item da pilha** (`ph2d-constraints-ecs/src/ik.rs`): corrente da ponta para cima (`governed`), parte do autorado (ou do estado da pilha para quem já foi avaliado/escrito neste quadro), `solve_chain`, escreve os ossos acima pelo ledger `ConstraintPose`, devolve a pose do dono à pilha. Alvo na corrente = `Ciclo`; dono sem `Bone` = `NaoEOsso`. `ordem()`: quem lê um osso da corrente depende da ponta. IKs com correntes sobrepostas repetem-se NO QUADRO até ≤ 4 ulp (medido: 9,8e-3 · 2,3e-4 · 5,4e-6 · 2,5e-7 · 1 ulp ⇒ 5 passagens; guarda 64) | `ik.rs`, `avalia.rs`, `ordem.rs` | `ossos.rs`: `ik_is_an_ordinary_item_in_the_stack`, `bend_side_flips_with_the_mirror`, `a_reader_of_the_chain_runs_after_the_ik`, `a_target_inside_the_chain_is_a_cycle`, `a_half_ik_is_a_function_of_the_authored_pose`, `ik_on_an_object_that_is_not_a_bone_says_so`, `the_stretch_reaches_a_far_target_by_scaling_the_chain_root`, `an_old_ik_goal_becomes_the_ik_item`; app-skeleton `two_anchors_on_one_chain_reach_a_fixed_point` |
| **O goal virou o item IK**: API única `ik_de/iks/com_ik/por_ik/tirar_ik` + `add_ik/add_look_at/aponta` (`ik_novo.rs`: a app-vec também cria IK e família não chama família); `converter_ik_goals` no início de cada `quadro` (ficheiro/prefab/colagem). `goal::solve`/`schedule`/`goal_authored` saíram; anéis, âncora, knobs do painel do esqueleto, tecla K, espelho (`app-skeleton/espelho.rs` tira a IK da cópia), timeline `IkBendSide` (a curva antiga continua a endereçar) — tudo pela API | `ik_item.rs`, `ik_novo.rs`, app-skeleton, timeline `apply_prop.rs`, shell (5 fases) | bateria por nome: 192 verdes |
| **Quadro**: osso inteligente → `ph2d_app_constraints::quadro(sim, &vec_scene, preview)` (objectos e ossos, topológica) | `fase_skeleton_drives.rs` | `architecture_the_pose_motors_run_before_the_mesh` |
| **Duas camadas no mesmo osso** (osso inteligente + pilha): `driven` não lê a escrita de outra camada como «outra mão»; `substitute_authored` lê todos os vivos primeiro e escreve a de baixo por último (o restore devolvia a pose errada) | `ph2d-preview-drive/src/lib.rs` | `two_motors_on_one_entity_keep_the_authored_pose`, `an_artists_edit_under_two_motors_is_still_the_document`; testes da shell `preview_drive_tests` |
| **Uma lei dos 2 ossos**: o nó `rig.ik_2bone` chama `reach` (a cópia f32 saiu). A mira e o limite de junta descem para `ph2d_skeleton_live::bone` (gesto e IK, o mesmo bit); a mira segue a lei da composição da casa (sob pai espelhado apontava ao lado errado) | node, `skeleton-live/bone.rs` | `the_node_solves_with_the_skeleton_law` + 7 antigos do nó |
| **Cartão**: menu + IK/Stretch To/Follow Path; `MAX_OPCOES` 3→4; unidades `Porcento`/`Contagem` (sem sufixo); canais 10..18 (append); i18n; estado «IK works on bones» | campos, inspector, `constraint_edits.rs`, painel ids/sync | `seam_constraints`, wiring parity, `node_id_collisions`, canais |
| **Follow Path sobre a forma**: a poligonal tem UM dono — `ph2d_vec_scene::world_polyline` (movida do `motion.path`, que delega, bits iguais); a família guarda a poligonal LOCAL (recurso `Caminhos`), a fase aplica o afim do alvo na hora | `world_polyline.rs`, app-constraints `caminhos`, `avalia.rs` | `follow_path_walks_the_drawn_shape_where_it_is` |
| **Cena 4** + sonda headless | `smoke.rs::braco` | `the_arm_scene_turns_the_hand_below_the_ik`, `every_level_up_to_the_router_ceiling_answers` |

## Mutação: todas sangram

Ronda 1 (agente): M1 IK desligada · M3 espelho · M4 realimenta · M6 stretch · M7 mira · M8 ordem do substitute · M9 camada
— sangram; **M2** (aresta da ordem) e **M5** (autorado) sobreviveram: M2 era fixtura com bits ao contrário (o leitor nasce
DEPOIS), M5 escondia um defeito real (os ossos da corrente eram devolvidos ao autorado no início de cada quadro) — curados e
reaplicados: vermelhos. Ronda 2: M10 sem repetir o grupo · M11 osso escrito não conta · M12 volume · M13 tangente · M14 fechado
· M15 stretch ao alcance · M16 lado do nó · M17 eixo da mira · M18 conversão — 9/9 sangram. Caminho sem a família: vermelho.

## Para a integração

- Superfície de colisão (append em partilhados): `Kind` (+3 ao fim), `Estado::NaoEOsso`, `Inerte::Corrente`, canais
  `canal_constraint` 10..18, `TIPOS` 8→11, `MAX_OPCOES` 4 (ids `insp_constraint_opcao_*_3_*`, `insp_constraint_tipo_8..10`),
  `Unidade::{Porcento, Contagem}`, `Driven::pose()`, `ph2d_vec_scene::world_polyline`, `chain_to_in`/`governed_in`,
  `ph2d_app_constraints::quadro` ganhou `cena`. Dependências novas: timeline(feature skeleton)→constraints-ecs, app-vec→
  constraints-ecs, constraints-ecs→skeleton-live/skeleton-ecs/vec-entities. Shell: `fase_skeleton_drives` (ordem), 5 fases do
  esqueleto pela API.
- Mudança de comportamento intencional: a IK parte do AUTORADO a cada quadro (antes: da pose viva, convergindo ao longo dos
  quadros) — arrastar a timeline dá o mesmo bit.
- Suíte COMPLETA não corrida (regra do dono): é do integrador.

## Falta / aberto

1. Smoke do dono da cena 4.
2. A IK de 1 osso (Look At do esqueleto) e o `Kind::LookAt` são dois conceitos no menu — unificar ou esconder um (decidir).
3. Os abertos do W1d (2–7) continuam.
4. Próxima onda: **W3** (Spring determinístico, Transformation, Action/Slider, Multi-parent, Pivot, Floor, Spline IK) — plano §7.

## Armadilhas

- Dois agentes no mesmo worktree: partilham `lib.rs`/`Cargo.lock` — só um comita, no fim (memória «hunks de N agentes»).
- `quadro` pede a cena vectorial; nos testes `&VecScene::new()`.
- Sob a herança TRS da casa, a mira afim só concorda com escala uniforme positiva — use `ph2d_skeleton_live::bone::aim_rotation`.
- A bola da cena tem de estar ao alcance (o braço mede 4 m) — a sonda prova o cotovelo pela distância à bola.
