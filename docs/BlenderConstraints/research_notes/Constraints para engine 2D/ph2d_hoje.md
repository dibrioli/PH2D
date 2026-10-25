# O que o PH2D já tem (levantamento do código, 25/10/2026)

> Fonte: varredura read-only da árvore `line/BlenderConstraints` (base `c3662cd8e4`). Endereços `file:line`.

## Conclusão

O PH2D já tem bastante do que um módulo de constraints proporia: parentesco (`ChildOf` + `StableId`), IK de dois ossos e FABRIK (nós de rig e `ph2d-skeleton`), IK/FK interativos na física, joints com motor/limites/quebra, `look_at`, `path`, `pin_constraint`, `follow`, `spring` e `drive` (valor→canal). **Não existe** um sistema GENÉRICO de constraints: não há Copy Transform, Child Of (com offset/inverso animável), Limit, Distance como peça própria, nem constraints geométricas no vetor (paralelo, perpendicular, horizontal). A timeline tem caminho para propriedade dirigida por outra (expressões, ADR-0144/0152), estado de entrega na UI não confirmado.

## Tabela

| Capacidade | Estado | Onde |
|---|---|---|
| Parentesco / hierarquia | existe | `ph2d-ecs/src/lib.rs:392` (`ChildOf`/`Children`); `ph2d-ecs/src/transform_inverse.rs:26` (`parent_world_transform`); ADR-0025 proíbe wrapper `Node` próprio |
| Referências por identidade | existe | `ph2d-ecs/src/stable_id.rs:86` (`StableId`), `:278`; `ph2d-app-components/src/instance_refs.rs:3` |
| Seguir um objeto com offset | só via hierarquia | `ph2d-app-components/src/tween_bridge.rs:107` |
| IK de dois ossos | existe (nó) | `crates/ph2d-node-rig-ik-2bone/src/lib.rs:1-30` |
| FABRIK (N ossos) | existe (nó) | `crates/ph2d-node-rig-fabrik/src/lib.rs:1-25` |
| Cadeia no `ph2d-skeleton` (reach, fabrik, bend, centro, correção, fold) | existe | `crates/ph2d-skeleton/src/reach.rs:411`, `reach_fabrik.rs`, `bend.rs`, `centro.rs`, `correccao.rs` |
| Pose / gesto de alcance (IK interativa), limites de osso | existe | `ph2d-app-skeleton/src/bone_gesture.rs:283`; `ph2d-app-skeleton/src/bone_limit.rs:39` (`BoneLimit` em `ph2d-skeleton-ecs`) |
| Skin (LBS) | existe | `crates/ph2d-skeleton/src/lib.rs:1-15`; `ph2d-skeleton-ecs/src/skin_bind.rs` |
| FK de rig / pose de física | existe | `ph2d-node-rig-fk`; `ph2d-physics-ecs/src/bridge/fk.rs:202`, `:336` |
| IK na física (gesto transitório) | existe | `ph2d-physics-ecs/src/bridge/ik.rs:322/392/450`; ADR-0149 |
| Joints de física para o usuário (motor, limites, quebra, mola, rigidez) | existe | `ph2d-physics-ecs/src/joint/kind.rs:18`; `ph2d-app-components/src/marcas_inspector.rs:76-131`; `ph2d-physics-ecs/src/joint_tool.rs:69-80` |
| Polia | existe | `ph2d-physics-ecs/src/bridge/joint_break.rs:206` |
| Quebra de junta | existe | `ph2d-physics-ecs/src/bridge/joint_break.rs:35` |
| `look_at` (curto arco, falloff) | existe (nó Motion) | `crates/ph2d-node-motion-look-at/src/lib.rs:1-30`, `:124` |
| Seguir curva (`path`) | existe (nó Motion) | `crates/ph2d-node-motion-path/src/lib.rs:1-20`; `ph2d-app-components/src/path_follow_bridge.rs` |
| `pin_constraint` | existe (nó Motion) | `crates/ph2d-node-motion-pin-constraint/src/lib.rs:1-52` |
| `follow` (Linear/Smooth/Spring) | existe (nó valor) | `crates/ph2d-node-value-follow/src/lib.rs:1-25` |
| `spring` | existe | `crates/ph2d-node-motion-spring/src/lib.rs` |
| `drive` (valor → canal) | existe | `crates/ph2d-node-motion-drive/src/lib.rs:1-25` |
| `proximity` / `spline_wrap` | parcial | `crates/ph2d-node-motion-proximity/`, `ph2d-node-motion-spline-wrap/` |
| Distância, Limit Location/Rotation/Scale, Copy Transforms, Child Of | **não existe** como peça | — |
| Expressões / drivers da timeline | parcial, a confirmar | ADR-0144, ADR-0152; `ph2d-timeline/src/expr_owed.rs:84`, `apply.rs` |
| Pontes de driver (script, tween, parallax, hud, path) | existe | `ph2d-preview-drive` (ledger por `(entidade, driver)`) |
| Snap e guias (vetor) | existe | `crates/ph2d-vec-edit/src/snap.rs:1-12`, `:329`; `quickshape_geom.rs:116` |
| Constraints geométricas no vetor | **não existe** | nada em `ph2d-vec-edit`/`ph2d-vec-entities` |
| Câmera segue alvo | parcial | `ph2d-app-components/src/camera_inspector.rs` |

## Docs/ADRs relacionados

- `docs/Components/00_levantamento_componentes.md:176` (`IkSolver2d` parcial), `:305` (LookAt/Aim como candidato a preset).
- ADR-0149 (IK = pose transitória), ADR-0169 (esqueleto em módulo próprio), ADR-0144/0152 (expressões). Nenhum ADR com «constraint» no título.

## Contratos congelados que um módulo de constraints pode encostar

- Nós (ADR-0039): `NodeOp=2`, `OpResolver=1`, `NodeManifest=8` — nó novo de drop-crate não muda o contrato; mudar portas/EvalCtx muda.
- Tools (ADR-0040/0041): `Tool=12`, `PanelEvent=4`.
- Vector antigo (ADR-0056..0068) congelado; o motor `ph2d-vec-*` não.

## Onde se encaixaria

- Constraint = componente ECS sobre `ChildOf`, alvo por `StableId` (padrão `PhysicsJoint.body_a`).
- Escrita de pose por um `Driver` novo em `ph2d-preview-drive` (padrão `tween_bridge.rs`), para não haver dois passes a escrever o `Transform`.
- Restrição negociada com a física → `ImpulseJoint` (`ph2d-physics-ecs/src/joint`); IK de pose no padrão ADR-0149.
- Constraint de elemento no Motion → nó de cook (padrão `pin_constraint`/`look_at`).
- Ordem do schedule (animação → constraints → propagação → física) **não confirmada**: ler o schedule antes de cravar.
