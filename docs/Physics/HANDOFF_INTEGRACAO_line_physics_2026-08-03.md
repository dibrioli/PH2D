# HANDOFF DE INTEGRAÇÃO — `line/physics`, jornada de 2026-08-03

> **A linha está FECHADA e NÃO integrou.** 10 commits, 6 waves, **gate impactado
> verde (7541/7541, 0 falhas)**. Aguarda ordem explícita do Enio.

---

## 1. O que entra, numa frase cada

| wave | o que é | cena |
|---|---|---|
| **W-SignalLeave** | a porta que **FECHA** — o nome de SAÍDA, o último aberto do W-Signal; e a row de CHEGADA, que shipou **write-only** | `=76` |
| **W-PartAdopt** | ⚠️ `Make Independent Body` **APAGAVA a forma autorada da peça, em silêncio** | `=70` |
| **W-RopeSays** | o readout de uma corda que não roteia **diz `no route`** em vez de `0 N` em âmbar | `=58`.. |
| **W-RailRope** | o **TRILHO** como elo de corda — o mastro telescópico | `=77` |
| **W-JointAnim** | **um parâmetro de joint é uma entrada por TICK** — 4 canais de timeline, e a correção de que o número nunca chegava num replay | `=78` |
| **W-JointCustom** | o **joint descrito por EIXO** — Free/Limited/Locked por grau de liberdade, e o motor diz em qual deles age | `=79` |

O detalhe de cada uma (o desenho, os números, os gates e as mutações) vive no
tracker [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md), nas últimas
seções. Este documento é o que o integrador precisa **antes** de fundir.

---

## 2. A tabela de COLISÃO — o que outra linha pode ter reivindicado

| eixo | valor | risco |
|---|---|---|
| **`PROJECT_SCHEMA`** | **50 → 51** (o `PhysicsJoint` ganhou `custom`) | ⚠️ **ALTO, e é o item a conferir primeiro.** O 51 foi **CONTADO** contra o `main` de hoje, que dizia 50 — mas o valor se conta contra o `main` do **DIA DA INTEGRAÇÃO**. ⚠️ **E a colisão pode passar MUDA:** se outra linha escreveu o mesmo literal, o `project.rs` **não conflita** (o git não sabe o que o número significa) e o bump de uma das duas **evapora com a suíte verde** — foi assim com a `line/FLIP` em 01/08. **Confira o `project_schema_tests.rs`, que é onde o conflito aparece** |
| **tripla do pin** | **`(51, 13, 14)`** | `FLIP_SCHEMA` 13 e `VEC_SCENE` 14 **intocados por esta linha** — se o `main` os moveu, é só re-ler os dois |
| **`VEC_SCENE_SCHEMA` · `DOC_VERSION` · `FLIP_SCHEMA`** | intocados | nenhum |
| **registro `ph2d-physics-ecs`** | **26 → 27** (`SignalOnLeave`, da W-SignalLeave) | ⚠️ o contador é UM só nesta crate (o espelho triplo é do `ph2d-ecs`, que esta linha **não** tocou). As duas waves de hoje **não** acrescentam componente |
| **registro `ph2d-ecs`** | intocado | nenhum |
| **gizmo ids** | **nenhum novo** — o último segue **973**, próximo livre **974** | nenhum |
| **ADR** | **nenhum novo** | ⚠️ **esta linha fica FORA da disputa de número desta janela** |
| **`Cargo.toml`** | ⚠️ **DOIS tocados** — `crates/ph2d-timeline/Cargo.toml` (feature `physics` + dep OPCIONAL de path) e `shells/desktop/Cargo.toml` (liga a feature) | ⚠️ **nenhuma dep EXTERNA nova, nenhuma crate nova** — só uma aresta interna de path. O `Cargo.lock` ganha a aresta |
| **contrato congelado** | intacto (`architecture_tool_contract_surface` 4/4 · `architecture_contract_surface` 3/3, **rodados**) | nenhum |
| **`physics_ecs_c9`** | **`8c7ba62442f1d577…`, 101 corpos, debug ≡ release** | ⚠️ **MUDA** vs `main` (`16ba80e8…`, 99): a W-JointCustom acrescentou duas lanes (o servo e o Custom). ⚠️ E ele se moveu **DUAS vezes** na jornada — a segunda foi o fix do `has_motor`, **não** o split do arquivo |

**ids de painel novos:** `INSP_PHYS_SIGNAL_LEAVE` · os quatro
`TIMELINE_ADDPROP_JOINT_*` · `INSP_JOINT_AXIS_GROUP`/`_MODE`/`_MIN`/`_MAX` ·
`INSP_JOINT_MOTOR_AXIS`(`_GROUP`) — todos hash-de-string, cobertos pelo
`node_id_collisions`.

**Ação nova no barramento:** `EditorAction::InspectorSignalLeaveEdit`.

**Arrays cujo TAMANHO se conta, não se escolhe:**
`ADDPROP_BUTTONS` **12 → 13** (`ph2d-panel-timeline/src/ids.rs`) ·
`INSP_JOINT_KIND` **8 → 9** · `INSP_PHYS_JOIN_KIND` **8 → 9** ·
`JointKind::ALL` **8 → 9**.

---

## 3. Os pontos de merge sensíveis (leia antes de resolver conflito)

1. **`PropKind` ganhou QUATRO variants apendados** (`JointMotorTarget = 9` ..
   `JointMaxLength = 12`). ⚠️ Toda outra linha que apendar um variant ali colide
   no NÚMERO — e a resolução é **contar**, nunca escolher. Dois `match`
   exaustivos fora desta linha (`prop_readback.rs`, `clip_stack_autokey.rs`)
   **deixam de compilar** quando um variant novo chega: isso é o desenho
   funcionando, não um conflito.

2. **A `ph2d-timeline` passou a ter uma feature `physics`.** Se outra linha
   tocou o `Cargo.toml` dela, as duas listas de feature fundem por **adição**; e
   o `shells/desktop/Cargo.toml` traz `features = ["render", "physics"]` numa
   linha só — um merge que perca o `"physics"` deixa os quatro canais
   **pintados e mudos**, com a suíte da crate verde (ela roda sem a feature).

3. **Quatro assinaturas mudaram, e cada uma tem UM ou POUCOS chamadores:**
   `motor_axis(kind)` → `motor_axis(desc)` · `motor_units(kind_tag)` →
   `motor_units(&info)` · `motor_out`/`motor_in` passaram a receber o `info` ·
   `JointView` ganhou `rotation_free`. Um conflito aqui é textual.

4. **QUATRO splits por LOC**, todos por assunto — se outra linha tocou os pais,
   o conflito é textual e a resolução é manter as duas metades:
   `ph2d-physics-ecs/src/joint.rs` → `joint/clamp.rs` (o `clamped()`) e
   `joint/ports.rs` (as duas perguntas que viraram da INSTÂNCIA) ·
   `ph2d-panel-inspector/src/sections/joint.rs` → `joint_custom.rs` (as rows de
   eixo) · `ph2d-panel-timeline/src/tracks.rs` → `tracks_label.rs` (os rótulos) ·
   `ph2d-physics-ecs/src/bin/physics_ecs_c9.rs` → `physics_ecs_c9/joints.rs`.

5. **`physics_overlay_joints.rs` mudou de ASSINATURA** (jornada da manhã):
   `joint_marks` devolve `JointMarks { paths, not_acting }` em vez de um `Vec`,
   e o arquivo cruzou 600 ⇒ split em `physics_overlay_joint_ghost.rs`. Quem
   ACRESCENTOU uma chamada precisa de `.paths`.

6. **`inspector_physics_gesture_tests.rs` também cruzou 600** ⇒ split em
   `inspector_physics_gesture_zone_tests.rs`; o helper `snapshot` virou
   `pub(super)`.

7. **`inspector_commits::dispatch` ganhou UM parâmetro** (`signal_leave_edit`) —
   um chamador só, no `render_loop/mod.rs`.

8. **A baseline do `hr15_no_hardcoded_ui_strings` PERDEU uma entrada.** ⚠️ Não é
   dívida paga: os dois placeholders de sinal foram para uma **TABELA** e o
   scanner só vê literais dentro de `.placeholder("…")`. As strings continuam
   hardcoded, **nomeadas no comentário da baseline**. Se outra linha acrescentou
   uma entrada, as duas listas fundem por adição.

9. **`ph2d-ui-testkit` ganhou `set_text`** (aditivo, irmão exato do
   `set_number_value`/`set_toggle_on`).

---

## 4. Rode ISTO, e não confie no meu relato

```bash
cd Worktrees/line-physics
bash scripts/nextest-impacted.sh              # 7541/7541 aqui
cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9            # e em --release
cargo test -p ph2d-editor-core --test node_id_collisions
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test project_schema_tests
```

⚠️ **Rode a suíte em DEBUG e em RELEASE.** Esta linha tem precedente registrado
(o `ph2d-flip-colorize` panicava só em debug, e a nota sobreviveu ao fato por três
integrações).

⚠️ **Conte os binários** (`grep -c "test result: ok"`), **não corte a saída** — um
`head -40` numa varredura me fez ler verde onde havia vermelho na jornada de
02/08.

---

## 5. Smokes (todos `--release`)

| cena | o que julgar |
|---|---|
| **`PH2D_PHYSICS_SMOKE=78`** | *A MÁQUINA ANIMADA.* Quatro máquinas keyframadas (servo · guincho · músculo · giro) e o **CONTROLE cinzento** ao lado, idêntico e sem chaves. ⚠️ **O braço cinzento fica em `−0,005 rad` enquanto o keyframado varre `1,54`** — se os dois se moverem, o canal não é o que se pensa. Arraste a régua para TRÁS: o replay tem de repetir o mesmo caminho |
| **`PH2D_PHYSICS_SMOKE=79`** | *O JOINT DESCRITO POR EIXO.* O carrinho do Custom **desliza E gira**; o Slider ao lado, com o mesmo motor, **só desliza** (13,72 rad contra 0,00). A calha vertical **para no batente** (cai exatamente 1,50 m). E as duas barras têm a MESMA configuração, só o eixo do motor difere: uma **gira** (15,00 rad contra 0,00), a outra **desliza** (2,00 m contra 0,00) |
| **`PH2D_PHYSICS_SMOKE=76`** | *A PORTA QUE FECHA.* Três andarilhos; o passo 3 é o conserto do write-only |
| **`PH2D_PHYSICS_SMOKE=77`** | *O MASTRO TELESCÓPICO.* **Alt**+arraste a cabeça |
| **`PH2D_PHYSICS_SMOKE=70`** | *A PEÇA.* `Make Independent Body` tem de PRESERVAR a forma autorada |
| **`PH2D_PHYSICS_SMOKE=58`** | o readout de uma corda degenerada diz **`no route`** |

---

## 6. O que fica ABERTO, e por quê (com o número)

**O §8 do plano 02 tem UM item**, e ele segue condicionado — *rows de readout
tingidas*, cuja condição **não** está satisfeita (o readout de carga do W-J7b
vive no OVERLAY, não em row).

⚠️ **Nenhum dos dois abaixo é dívida desta jornada — os dois são decisões que
não são de engenharia**, e a evidência está medida no tracker:

- **o consumidor de GAMEPLAY do sinal** — `AppGfx.script` é um `Option<ScriptHost>`
  **nunca tickado**; não há superfície de autoria de script, e a de áudio é
  desenho de produto. E a mesma outbox recebe os sinais da TIMELINE, então o
  consumidor é **cross-cutting dos dois produtores**;
- **um Ctrl+Z para as duas metades do bake** — o `timeline_key` consome o acorde
  antes, e o `undo_owner` não tem dono Timeline; unir as filas é redesenho do
  roteador de undo, com risco real de desfazer um passo global alheio.

E os três que as waves de hoje deixam **nomeados, com o mecanismo**:

- **`GenericJoint` por eixo com ACOPLAMENTO** (`coupled_axes`) não é oferecido: o
  Rod já mediu que o limite acoplado do rapier é unilateral, e expor um knob que
  só funciona num sentido é pior que não o ter;
- **um Custom não vira elo de árvore de IK/FK** (`is_rigid_link` o recusa) — o
  `FkDof` modela UM grau de liberdade e um Custom pode oferecer três; escolher
  por ele seria a mesma mágica que o `motor_axis` autorado existe para não fazer;
- **o glifo do Custom não desenha as direções dos eixos lineares** — elas vivem no
  frame do próprio joint e a `JointView` não o carrega; desenhá-las a partir do
  ângulo do corpo A seria uma figura que **mente** em todo joint rotacionado.

E os que continuam sendo **cercas de Chesterton, não defeitos**: peça-dentro-de-peça ·
salto balístico do contrapeso · corpo que passa da própria roldana · a corda de IK
não colide · a trava ultrapassa por um sub-passo (0,3685 contra 0,5, pinado no gate).
