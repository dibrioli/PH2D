# 03 — IK multibody: posar arrastando a ponta (W-IK)

> **Origem:** horizonte do [plano 02 §8](02_plano_joints_ui_authoring.md) — *"IK multibody
> (`inverse_kinematics_delta` pronto no rapier): posar corrente arrastando a ponta e bakear —
> diferencial de verdade para animação; arquitetura separada (multibody set), ADR próprio"*.
> Escalonado pelo Enio em 2026-07-27.
>
> **Arquitetura:** [ADR-0145](../architecture/decisions/0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md).
> Este doc é o **estado**; o ADR é o *porquê*.

---

## §1 — O que a wave entrega

Uma quarta ferramenta de ponteiro, **Pose**, na seção Interaction do painel de física. Com ela em
mãos e o relógio **parado**, arrastar um corpo que pertence a uma cadeia de joints rígidos dobra a
cadeia inteira atrás dele.

- **Núcleo:** `ph2d-physics::world::ik` — `IkChain` (a árvore transitória), `IkLink`, `IkPose`,
  `IkOptions`, `is_rigid_link`. `PhysicsWorld::ik_chain` / `ik_solve`.
- **Ponte:** `ph2d-physics-ecs::bridge::ik` — `IkPlan` (quem é a raiz, quais arestas),
  `IkSession`, `PhysicsBridge::{ik_plan, ik_begin, ik_move, ik_end, is_posing, posing_tip,
  posing_bodies}`.
- **Modelo:** `InteractionTool::Pose` + `runs_at_rest()` + `InteractionSettings::{ik_damping,
  ik_match_angle}` + `ik_options()`.
- **Painel:** o chip **Pose** no rádio de ferramenta, o slider **Smoothing**, o rádio
  **Tip Angle** (Free/Match) e uma **dica própria** (*"Paused + drag a jointed body"*).
- **Shell:** `body_pose.rs` — `take_pose` no press, `advance_body_pose` no Move,
  `release_body_pose` no release.
- **Smoke:** cena **`PH2D_PHYSICS_SMOKE=54`**.

## §2 — As três decisões que decidem tudo

1. **O multibody é ferramenta, não estado** (ADR §3). Nada persiste, o `step` não o vê, o c9 não
   se mexe.
2. **A raiz decorre da CENA** (ADR §5): um corpo Static/Kinematic alcançável é a raiz; sem
   nenhum, o mais distante da ponta, e a cadeia flutua.
3. **Posar exige o relógio PARADO** — a única ferramenta desta seção que trabalha assim, e o
   predicado é um (`runs_at_rest`), lido pelo painel (a dica) e pelo shell (o gesto).

## §3 — Estado

| # | Item | Estado |
|---|---|---|
| K1 | Árvore transitória + solve (`world/ik.rs`) | ✅ |
| K2 | Projeção de limites de dobradiça (ADR §6.1) | ✅ |
| K3 | Teto de passo adimensional (ADR §6.2) | ✅ |
| K4 | Clamp de alcance — a cadeia esticada APONTA (ADR §6.3) | ✅ |
| B1 | `IkPlan`: política de raiz + árvore geradora | ✅ |
| B2 | Sessão de pose na ponte (`ik_begin`/`ik_move`/`ik_end`) | ✅ |
| U1 | 4ª ferramenta + Smoothing + Tip Angle + dica | ✅ |
| S1 | Gesto no `input_dispatch` (press / move / release) | ✅ |
| S2 | Cena 54 + sonda medida | ✅ |
| S3 | Pill **PHYS** no topbar, ao lado do IMG (o painel de mundo não tinha abridor visível) | ✅ |
| F1 | Semeadura das coordenadas — a árvore nasce na pose REAL | ✅ |
| F2 | Re-montagem quando os handles ficam obsoletos (o crash do gesto) | ✅ |

**Gates:** 15 no núcleo (+5 varreduras `#[ignore]`) · 6 na ponte · 5 no shell · 4 arch-gates ·
16 no painel (2 novos) · 2 na cena. **Mutações:** o swap de âncoras, a projeção de limites, o
clamp de alcance e o teto de passo — todas sangram, cada uma no seu gate.

## §4 — Os números, e onde a tabela mora

Todos MEDIDOS, tabelas no ADR §6-§7 e nas varreduras de `world/ik_tests.rs`:

- `damping` **0,1** (não o 1,0 do rapier — 0,0787 m contra 0,0004 m de erro), faixa 0,05..1,0.
- `max_iters` **10**, **sem slider** — medido inerte acima de 10.
- Teto de passo **1 comprimento de elo** (a degradação começa em 2).
- Custo: build 0,005–0,015 ms, solve 0,002–0,006 ms para 3..32 elos.

## §5 — Aberto, nomeado com o preço

- **Limites de Slider não são honrados ao posar** (ADR §6.1). O `local_frame1` do prismático
  carrega a rotação do eixo, então a pose relativa não entrega a distância percorrida sem desfazer
  aquele frame. Gateado (`limit_is_a_coordinate`), não escondido.
- **Um joint criado no MESMO frame do gesto não entra na árvore** (ADR §9) — a travessia lê o
  estado reconciliado para reusar o `JointDesc` do solver. Não é alcançável pela mão.
- **Nenhuma marca de overlay para a pose.** A cadeia se desenha sozinha ao dobrar (os joints já
  têm glifo, o contorno já existe), e uma marca que só repete o que já está na tela é ruído. Se um
  smoke mostrar que falta ver *qual corpo é a raiz*, é wave própria — a resposta seria um realce
  no corpo-raiz, não uma anotação no cursor.
- **Posar não tem *ghost* da pose anterior.** O onion da timeline faz isso para animação; para
  autoria de rig é decisão de produto.
- **Nada liga posar a keyframe.** Posar escreve `Transform`; com AutoKey armado a timeline o
  captura pela máquina que já existe. Se o fluxo pedir um "K de pose", é da timeline, não daqui.
