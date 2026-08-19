# HANDOFF DE INTEGRAÇÃO — `line/physics` (2026-08-01)

**Status:** FECHADO 2026-08-01 · no `main` em `527d0b51a` (o commit que trouxe este arquivo).

> **Para o agente integrador.** A linha está **FECHADA**. Sete commits, quatro
> waves, **todos os smokes aprovados pelo Enio**. Ela **não** integrou e **não**
> pushou — DIRETRIZ §1.5.9.
>
> **Tracker por-wave:** [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) ·
> **Planos:** [`00_plano_waves.md`](../00_plano_waves.md) ·
> [`02_plano_joints_ui_authoring.md`](../02_plano_joints_ui_authoring.md) §11-§12

---

## §1 — O que integrar

**Branch:** `line/physics` · **tip `6ff45b79d`** · **base `98eb502a2`**

⚠️ **O `main` NÃO andou desde o fork** (medido: `git log $(git merge-base HEAD main)..main` = **0
commits**). Se ele tiver andado quando você ler isto, a §5 abaixo lista o que re-conferir.

| # | commit | wave |
|---|---|---|
| 1 | `c8e411f25` | docs de reabertura (o §8 encolheu de 7 para 5) |
| 2 | `eeec73e22` | **W-JointCopy** — afine UM joint e carimbe o rig inteiro (cena `=66`) |
| 3 | `6896fa29f` | **W-Rig** — o rig sai da HIERARQUIA (cena `=67`) |
| 4 | `a5b53c186` | as três correções do smoke da `=67` (Reset · emenda · batentes) |
| 5 | `74aa76845` | a régua existe — o prólogo abre a timeline para toda cena |
| 6 | `2bca8717a` | **W-SoftWeld** — a solda que CEDE (cena `=68`) |
| 7 | `6ff45b79d` | **W-Compound** — um corpo, VÁRIAS formas (cena `=69`) |

**33 arquivos novos · zero `Cargo.toml` tocado** (nenhuma dep nova, nenhuma crate nova).

---

## §2 — Os números de identidade (MEDIDOS agora, não lembrados)

| o quê | valor | como conferir |
|---|---|---|
| **`PROJECT_SCHEMA`** | **46 → 47** | `grep 'const PROJECT_SCHEMA' shells/desktop/src/project.rs` |
| **tripla-pin** | **`(47, 12, 13)`** | `project_schema_tests.rs` |
| **registro `ph2d-physics-ecs`** | **fica em 24** | `registers_every_physics_component` (`lib.rs`) |
| **`physics_ecs_c9`** | **99 corpos**, `556cb652…` | roda o bin; **debug ≡ release**, conferido |
| **ids novos** | `INSP_PHYS_ADD_SHAPE` · `INSP_JOINT_SOFT_GROUP` · `INSP_JOINT_SOFT[2]` | (mais `INSP_PHYS_RIG` e o par Copy/Paste, dos commits 2-3) |
| **gizmo ids** | **nenhum novo** (o último segue **971**, próximo livre **972**) | — |
| **ADR** | **nenhum novo** | tudo sob a ADR-0131 |
| **contrato congelado** | **intacto** | `grep -rn "NodeOp\|OpResolver\|NodeManifest" crates/ph2d-physics*` = **0** |

### ⚠️ O `PROJECT_SCHEMA` é o número que COLIDE

**46 → 47** é o único bump da linha (o campo `PhysicsJoint.soft`, W-SoftWeld). Se outra linha
bumpar na mesma janela, **o valor se CONTA a partir do `main` do dia, não se escolhe** —
[[feedback_numbers_that_sum_across_lines_count_dont_pick]]. Esta linha já pagou isso duas vezes
com a `line/FLIP` (o 30 em 25/07 e os 32/33/34 em 27/07). A **tripla-pin** em
`project_schema_tests.rs` tem de acompanhar.

---

## §3 — As quatro waves, uma frase cada

1. **W-JointCopy** (`=66`) — dois botões na §12 copiam/colam as **doze** propriedades de afinação
   de um joint. O Paste é a única edição da §12 que faz **fan-out**, e é a razão de ele existir.
   ⚠️ A porta `with_properties_of` **desestrutura `PhysicsJoint` exaustivamente**: um campo novo
   **não compila** até ser classificado — foi ela que decidiu o desenho da W-SoftWeld.

2. **W-Rig** (`=67`) — a terceira rota de criação de joint: **uma aresta pai→filho da Hierarquia
   É um joint**. Uma corrente é uma FILA, um ragdoll é uma ÁRVORE, e a árvore o artista já
   desenhou. Um GRUPO no meio é transparente (o filho liga ao AVÔ).
   ⚠️ E ela achou **dois defeitos PRÉ-EXISTENTES** ([BUGS](../BUGS_physics.md) #5 e #6): o **Reset não
   devolvia corpo parenteado** (o readback escrevia o filho antes do pai — 4,910 m; vinha do W5 e
   valia 3,2 mm no play, que é como sobreviveu um ano) e a **âncora nascia no ponto médio dos
   centros** (agora vai para a **EMENDA**, sem geometria nova — a `radial_fraction` é
   função-calibre). Mais os **batentes de ±60°** com que um rig nasce.

3. **W-SoftWeld** (`=68`) — a solda que CEDE, o espelho do vão que o Rod preencheu.
   ⚠️ **A receita que o plano prescrevia foi construída e REPROVADA por medição** (*"não travar
   nada + três molas"* separa as peças **0,92 m**); o que shipou trava os lineares e amolece só o
   ANGULAR. `SOFT_WELD_ANGULAR_GAIN = 20` é medido.

4. **W-Compound** (`=69`) — **um corpo, várias formas**. Um filho com `Collider` e **sem**
   `RigidBody` é mais uma forma do corpo ancestral (o `CollisionShape2D` do Godot sobre a nossa
   árvore). ⚠️ **Zero schema** — o `Collider` já existia; o que mudou é **quem o lê**.

---

## §4 — O que a integração precisa saber para não se surpreender

### 4.1 Mudanças de comportamento (todas smokadas)

- **Um `Collider` num filho sem `RigidBody` deixou de ser inerte.** Antes ele não fazia nada;
  agora vira uma forma do corpo ancestral. ⚠️ **Varri o repo** e nenhuma fixture/cena dependia do
  estado antigo (ele era invisível ao solver por construção), mas é a mudança com maior alcance da
  linha — se algum teste de OUTRA linha spawnar um filho com collider e sem corpo, ele passa a
  colidir.
- **O contorno (`B`) desenha as peças**, e a query dele deixou de exigir `RigidBody`.
- **O prólogo do `physics_smoke` abre a timeline** para toda cena de física (18 delas pediam a
  régua e nenhuma a mostrava).
- **`InspectorPhysicsInfo` deixou de ser `Copy`** (o nome do dono é `String`) e o thread-local
  dele virou `RefCell`, como o do joint ao lado. ⚠️ **Isto toca a `ph2d-panel-inspector` inteira**
  — é o ponto mais provável de conflito textual com outra linha que mexa naquele painel.

### 4.2 Foundational tocado (tudo aditivo)

| crate | o que |
|---|---|
| `ph2d-editor-core` | 3 ids novos · `PhysicsFieldEdit::AddShape` · `JointFieldEdit::Soft` · `InspectorPhysicsInfo.part_owner` (+ perda do `Copy`) · `InspectorJointInfo.soft` |
| `ph2d-physics` | `world/parts.rs` (novo) · `JointDesc.soft` + `SOFT_WELD_ANGULAR_GAIN` · `world/joints.rs` o braço soft |
| `ph2d-physics-ecs` | `rig.rs`/`seam.rs`/`bridge/parts.rs`/`bridge/joint_desc.rs` (novos) · `PhysicsJoint.soft` · `JointKind::can_be_soft` · `PhysicsJoint::breaks_on_torque` |
| `ph2d-panel-inspector` | `sections/physics_doors.rs`/`physics_join_rows.rs` (novos) |

**Superfície pública nova:** `ph2d_physics_ecs::{RIG_LIMIT_DEG, rig_edges, rig_limits,
subtree_parts, ColliderPose, seam_between, seam_point}` · `ph2d_physics::world::parts::{attach_part,
detach_part, part_local}`.

### 4.3 Seis splits de LOC, todos por RESPONSABILIDADE

`bridge/joints.rs` → `bridge/joint_desc.rs` · `sections/physics.rs` → `physics_doors.rs` ·
`render_loop/physics_overlay.rs` → `physics_overlay_shapes.rs` · (mais os três da W-Rig).
⚠️ **Se outra linha tocar os mesmos arquivos, o conflito será de MOVIMENTO** — resolva pelos
estágios do índice ([[feedback_resolve_conflicts_from_index_stages_not_markers]]), não pelos
marcadores.

---

## §5 — O gate de fechamento que EU rodei (reproduza na árvore combinada)

Tudo verde no tip, **em release E em debug** (a lição do `ph2d-flip-colorize`: rodar só com
`--release` esconde pânico):

```
cargo test -p ph2d-physics -p ph2d-physics-ecs -p ph2d-panel-inspector -p ph2d-editor-core -p ph2d-ecs --release
cargo test -p ph2d-physics -p ph2d-physics-ecs -p ph2d-host-desktop          # DEBUG
cargo test -p ph2d-host-desktop --release
cargo clippy --workspace --all-targets     # 0 warnings
cargo fmt --all --check                    # limpo
```

⚠️ **Os gates que um `cargo test -p` filtrado NÃO alcança** (a família do miss do `file_loc_caps`
que esta linha já documentou duas vezes) — rode-os **isolados**:

```
cargo test -p ph2d-editor-core --release --test architecture_workspace_file_loc_cap \
  --test architecture_panel_loc_cap --test architecture_adr_numbers_are_unique \
  --test node_id_collisions --test arch_safe_clamp_only
cargo test -p ph2d-host-desktop --release --test file_loc_caps \
  --test handle_scenes_start_paused --test every_physics_component_is_authorable
```

E o **hash de determinismo**, que é o que prova que a física atravessou o merge sem mover um bit:

```
cargo run -p ph2d-physics-ecs --release --bin physics_ecs_c9   # 99 corpos, 556cb652...
cargo run -p ph2d-physics-ecs           --bin physics_ecs_c9   # o MESMO hash em debug
```

⚠️ **Se o hash mudar na árvore combinada e nenhuma linha tiver tocado física, PARE** — é o sinal de
que outra coisa alcançou o caminho determinista.

---

## §6 — Os smokes (todos APROVADOS; re-rodar é opcional)

```
env PH2D_PHYSICS_SMOKE=66 cargo run -p ph2d-host-desktop --release   # copy/paste de propriedades
env PH2D_PHYSICS_SMOKE=67 cargo run -p ph2d-host-desktop --release   # o rig a partir da hierarquia
env PH2D_PHYSICS_SMOKE=68 cargo run -p ph2d-host-desktop --release   # a solda que cede
env PH2D_PHYSICS_SMOKE=69 cargo run -p ph2d-host-desktop --release   # um corpo, várias formas
```

Cada cena **imprime o que montou** com os números MEDIDOS ao lado. Se a linha de resumo não
aparecer, pare: o resto do smoke não significa nada.

---

## §7 — O que fica ABERTO (nada bloqueia a integração)

**Do plano 02 §8 — dois itens, os dois condicionados:**

- **Params de joint keyframáveis** — cross-line com a timeline; `PropKind` é enum FECHADO de 7
  variants, todas de POSE. **Decisão do Enio.**
- **Custom/GenericJoint** — *"só se um caso real pedir"*. Cerca de Chesterton.

**Dívidas menores, cada uma com o preço nomeado no tracker:** um pino de mundo e um pino
corpo↔corpo leem igual na tela · não há alça para *onde no corpo* o pino de mundo prende ·
`axle_pair` recusa 3+ contatos num eixo · o readout `0 N` de corda degenerada quer i18n · **um
Ctrl+Z para as duas metades do bake** (dois roteadores de undo — a cura mora no roteador do editor,
outro domínio) · uma **peça** não tem seção própria no Inspector · o `part_views` da ponte existe e
**não tem consumidor** (o overlay desenha do ECS, que é a fonte que o artista arrasta).

---

## §8 — As lições desta jornada que valem para a PRÓXIMA linha

1. **Uma nota de pendência velha faz a próxima LLM construir o que existe.** Três listas deste
   tracker diziam *"aberto"* sobre coisas fechadas — uma delas **vinte linhas acima da seção que a
   fechou**. Corrigidas nesta jornada; a regra é atualizar a lista no MESMO commit que fecha.
2. **Uma mutação que sobrevive acusa um gate faltando OU uma afirmação minha.** Nesta jornada
   houve as duas: no W-SoftWeld ela provou que um guard que eu chamara de correção é **higiene**
   (o doc foi corrigido); no W-Compound ela provou que **o desenho das peças e o botão novo não
   tinham gate nenhum** — a suíte inteira ficava verde com a feature neutralizada.
3. **O CONTROLE é o que separa "produto quebrado" de "minha fixture errada".** Uma cena minha pôs
   mesas passando da borda do chão do smoke e uma delas caiu para **−74,88 m**; o que impediu isso
   de virar um defeito reportado foi a outra mesa medir certo.
4. **No Modo L, todo comando começa com o `cd` da worktree.** A cwd escorregou para a árvore
   primária **duas vezes** nesta jornada, uma delas fazendo `git rev-parse HEAD` responder pelo
   `main`. Nada foi commitado lá.
