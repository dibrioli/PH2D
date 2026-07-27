# 04 — FK + os cinco modos de joint (W-FK, W-JointTools)

> **Origem:** ordem do Enio, 2026-07-27, logo depois do smoke da IK — *"Já temos um belo IK mas
> creio que ainda não temos um FK… FK também é extremamente útil. Isso é tão importante que merece
> uma seção exclusiva no Painel Physics. Deixe a seção Interaction para a simulação da física. Crie
> outra seção de interação exclusiva para Joints. Coloque botões para 5 tipos de interação (3 nós já
> temos)."*
>
> **Arquitetura:** [ADR-0145 §10](../architecture/decisions/0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md)
> (emenda). Este doc é o **estado**; o ADR é o *porquê*.

---

## §1 — O que a wave entrega

Uma seção **Joints** no painel de física, irmã da Interaction, com um rádio de **cinco modos**:

| modo | o press abre | o arrasto move |
|---|---|---|
| **Body** (default) | o arrasto normal | só o corpo pego |
| **Rig** | o arrasto normal | o rig INTEIRO, âncoras inclusas |
| **Links** | o arrasto normal | só os elos móveis; âncoras ficam |
| **IK** | a cinemática INVERSA | a cadeia dobra atrás da ponta |
| **FK** | a cinemática DIRETA | o elo gira na junta; os filhos seguem |

- **Núcleo:** `ph2d-physics::world::ik_coords` — `FkDof`, `fk_dof`, `joint_coordinate_at`.
- **Ponte:** `ph2d-physics-ecs::bridge::fk` — `FkSession`, `PhysicsBridge::{fk_begin, fk_move,
  fk_end, is_posing_fk, fk_bodies, fk_session}`.
- **Modelo:** `ph2d-physics-ecs::joint_tool` — `JointTool`, `DragReach`, `JointGesture`;
  `InteractionSettings.joint`; `jointed_by` como porta única da política de arrasto.
- **Painel:** seção `PHYSICS_SEC_JOINT`, rádio `PHYSICS_JOINT_TOOL` (5 opções), o slider
  **Smoothing** e o rádio **Tip Angle** migrados para cá (são do IK), e **duas** linhas de dica —
  a do modo em mãos e a do Alt.
- **Shell:** `body_fk.rs` — `take_fk` no press, `advance_body_fk` no Move, `release_body_fk` no
  release; `joint_rig_drag` passa a receber o alcance em vez de um booleano.
- **Smoke:** cena **`PH2D_PHYSICS_SMOKE=55`**.

## §2 — As três decisões que decidem tudo

1. **A FK não tem solver** (ADR §10): girar um elo e levar os descendentes é um movimento rígido,
   então não há restrição a violar nem nada a resolver. O gesto colhe um pivô e as poses no press e
   daí em diante é aritmética exata.
2. **A hierarquia é a MESMA do IK** (`ik_plan`). Uma segunda política de raiz seria uma segunda
   resposta a *"para que lado desta cadeia é 'para cima'?"*.
3. **O Alt é UMA decisão com duas metades** (`drag_reach` + `gesture`): com ele apertado, o alcance
   é `Whole` e o gesto é `None`, em qualquer modo. Gate próprio, porque as duas metades separadas
   fazem o atalho "não funcionar às vezes".

## §3 — Estado

| # | Item | Estado |
|---|---|---|
| K1 | `FkDof` + `fk_dof` + `joint_coordinate_at` no wrapper | ✅ |
| K2 | Sessão de FK na ponte (`fk_begin`/`fk_move`/`fk_end`) | ✅ |
| K3 | Sobe pelo **Weld** até a junta com grau de liberdade | ✅ |
| K4 | Limite honrado, com o clamp no MAPEAMENTO (ADR §10.2) | ✅ |
| K5 | **Slider**: desliza no eixo, curso honrado (ADR §10.3) | ✅ |
| M1 | `JointTool` + `DragReach` + `JointGesture` + a lei do Alt | ✅ |
| M2 | `jointed_by` — a política de arrasto numa porta só | ✅ |
| U1 | Seção Joints: rádio de 5 + Smoothing + Tip Angle + 2 dicas | ✅ |
| U2 | A seção Interaction volta a ser só das 3 de simulação | ✅ |
| S1 | Gesto no `input_dispatch` (press / move / release) | ✅ |
| S2 | Cena 55 + sonda medida | ✅ |

**Gates:** 9 no gesto de FK + 1 no laço real (`fk_gesture_loop`) + 3 no `JointTool` + 4 no
`body_fk` + 20 no seam do painel + 6 arch-gates de shell + 2 na cena. **Mutações: 9, todas
sangram** — e uma delas (a que apaga o `swap_anchors`) **sobreviveu à primeira rodada** e derrubou
um gate meu, não o código (ADR §10.3).

## §4 — Os números, e onde a tabela mora

Medidos pela sonda `probe_smoke_55`, sobre as peças que o artista abre:

- `Rig` a partir da mão: carrega `UpperArm + Forearm + Hand + **Shoulder**`.
- `Links` a partir da mão: carrega `UpperArm + Forearm + Hand`, **sem** o ombro.
- FK girando a coxa 90°: a canela viaja **2,12 m**, a coxa **0,71 m**, e a distância entre elas
  fica **1,000** antes e depois — a peça é rígida.
- FK girando a canela: o conjunto movido é **só ela** (o pai não vai junto).

## §5 — Aberto, nomeado com o preço

- **Numa cadeia SOLTA a FK gira só o elo pego** (ADR §10.1). É consistente com a IK e com a
  ausência de "para cima" numa cadeia sem âncora; um rig com raiz autorada explicitamente resolveria
  os dois, e é decisão de produto que ninguém pediu.
- **`Body` não é um modo, é a ausência de um.** Ele existe no rádio para o artista poder VOLTAR ao
  comportamento normal sem decorar que "nenhum modo" é uma opção.
- **Nada liga FK/IK a keyframe.** Posar escreve `Transform`; com AutoKey armado a timeline o captura
  pela máquina que já existe (a mesma nota do plano 03).
- **Rotação/escala de um rig** continuam fora: o alcance só é consultado num `Translate`, porque
  girar um rig é uma decisão de PIVÔ que nenhuma wave tomou.
