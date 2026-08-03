# HANDOFF DE INTEGRAÇÃO — `line/physics`, jornada de 2026-08-03

> **A linha está FECHADA e NÃO integrou.** 6 commits, 4 waves, **gate impactado
> verde (7239/7239, 0 falhas)**. Aguarda ordem explícita do Enio.

---

## 1. O que entra, numa frase cada

| wave | o que é | cena |
|---|---|---|
| **W-SignalLeave** | a porta que **FECHA** — o nome de SAÍDA, o último aberto do W-Signal; e a row de CHEGADA, que shipou **write-only** | `=76` |
| **W-PartAdopt** | ⚠️ `Make Independent Body` **APAGAVA a forma autorada da peça, em silêncio** | `=70` |
| **W-RopeSays** | o readout de uma corda que não roteia **diz `no route`** em vez de `0 N` em âmbar | `=58`.. |
| **W-RailRope** | o **TRILHO** como elo de corda — o mastro telescópico | `=77` |

O detalhe de cada uma (o desenho, os números, os gates e as mutações) vive no
tracker [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md), nas duas últimas
seções. Este documento é o que o integrador precisa **antes** de fundir.

---

## 2. A tabela de COLISÃO — o que outra linha pode ter reivindicado

| eixo | valor | risco |
|---|---|---|
| **`PROJECT_SCHEMA`** | **INTOCADO por esta linha** | ⚠️ **nenhum.** Conferido por `git diff main --name-only`, que **não traz `project.rs`** — não pelo número, que envelhece. (O `main` de hoje diz `50`.) |
| **`VEC_SCENE_SCHEMA` · `DOC_VERSION` · `FLIP_SCHEMA`** | intocados | nenhum |
| **registro `ph2d-physics-ecs`** | **26 → 27** (`SignalOnLeave`) | ⚠️ **o contador é UM só nesta crate** (o espelho triplo é do `ph2d-ecs`, que esta linha **não** tocou) |
| **registro `ph2d-ecs`** | intocado | nenhum |
| **gizmo ids** | **nenhum novo** — o último segue **973**, próximo livre **974** | nenhum |
| **ADR** | **nenhum novo** | ⚠️ **esta linha fica FORA da disputa de número desta janela** |
| **`Cargo.toml`** | **ZERO tocados**, nenhuma dep nova, nenhuma crate nova | nenhum |
| **contrato congelado** | intacto (`architecture_tool_contract_surface` 4/4 · `architecture_contract_surface` 3/3, **rodados**) | nenhum |
| **`physics_ecs_c9`** | **`16ba80e8…`, 99 corpos, debug ≡ release** — **byte-idêntico ao `main`** | ⚠️ os quatro assuntos são **readout ou gesto de POSE**, e nenhum entra no hash |

**ids de painel novos:** `INSP_PHYS_SIGNAL_LEAVE` (hash de string, e **os DOIS**
ids de sinal entraram no `node_id_collisions` — o de chegada shipou fora daquela
lista, que é mantida à mão).

**Ação nova no barramento:** `EditorAction::InspectorSignalLeaveEdit`.

---

## 3. Os pontos de merge sensíveis (leia antes de resolver conflito)

1. **`shells/desktop/src/render_loop/physics_overlay_joints.rs` mudou de
   ASSINATURA e de tamanho.** `joint_marks` devolve
   `JointMarks { paths, not_acting }` em vez de um `Vec`, e o arquivo cruzou 600
   ⇒ **split por assunto** em `physics_overlay_joint_ghost.rs`. Se outra linha
   tocou esse overlay, o conflito é textual e a resolução é **manter as duas
   metades**; se ela ACRESCENTOU uma chamada a `joint_marks`, ela precisa de
   `.paths`.

2. **`inspector_physics_gesture_tests.rs` também cruzou 600** ⇒ split em
   `inspector_physics_gesture_zone_tests.rs` (os cinco gates de ZONA saíram; o
   pai fica com o corpo e o que ele grita). O helper `snapshot` virou
   `pub(super)`.

3. **`inspector_commits::dispatch` ganhou UM parâmetro**
   (`signal_leave_edit`) — um chamador só, no `render_loop/mod.rs`.

4. **A baseline do `hr15_no_hardcoded_ui_strings` PERDEU uma entrada.** ⚠️ Não é
   dívida paga: os dois placeholders de sinal foram para uma **TABELA** e o
   scanner só vê literais dentro de `.placeholder("…")`. As strings continuam
   hardcoded, **nomeadas no comentário da baseline** com arquivo e texto, e o
   gate segue afiado para o próximo literal ali. Se outra linha acrescentou uma
   entrada, as duas listas fundem por adição.

5. **`ph2d-ui-testkit` ganhou `set_text`** (aditivo, irmão exato do
   `set_number_value`/`set_toggle_on`).

---

## 4. Rode ISTO, e não confie no meu relato

```bash
cd Worktrees/line-physics
bash scripts/nextest-impacted.sh              # 7239/7239 aqui
cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9            # e em --release
cargo test -p ph2d-editor-core --test node_id_collisions
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
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
| **`PH2D_PHYSICS_SMOKE=76`** | *A PORTA QUE FECHA.* Três andarilhos: um sensor com os DOIS nomes (abre e fecha), um sólido com os dois (bate e desencosta) e o **CONTROLE** marcado só na chegada (abre e cala). ⚠️ **O passo 3 é o conserto do write-only:** selecione a porta e as duas rows **mostram** `door_open` e `door_close` |
| **`PH2D_PHYSICS_SMOKE=77`** | *O MASTRO TELESCÓPICO.* **Alt**+arraste a cabeça. Em cima, PARA OS LADOS: `2,0 / 1,5 / 1,0 / 0,5`. No meio, PARA CIMA: a corrente vai inteira. Embaixo, o CONTROLE soldado |
| **`PH2D_PHYSICS_SMOKE=70`** | *A PEÇA.* Autore uma peça fina com offset e densidade próprios, e aperte **Make Independent Body**: a forma tem de FICAR (antes ela virava a caixa do sprite com tudo zerado) |
| **`PH2D_PHYSICS_SMOKE=58`** | o readout de uma corda degenerada tem de dizer **`no route`** em vermelho, não `0 N` em âmbar |

---

## 6. O que fica ABERTO, e por quê (com o número)

⚠️ **Nenhum dos três é dívida desta jornada — os três são decisões que não são de
engenharia**, e a evidência está medida no tracker:

- **o consumidor de GAMEPLAY do sinal** — `AppGfx.script` é um `Option<ScriptHost>`
  **nunca tickado**; não há superfície de autoria de script, e a de áudio é
  desenho de produto. E a mesma outbox recebe os sinais da TIMELINE, então o
  consumidor é **cross-cutting dos dois produtores**;
- **um Ctrl+Z para as duas metades do bake** — o `timeline_key` consome o acorde
  antes, e o `undo_owner` não tem dono Timeline; unir as filas é redesenho do
  roteador de undo, com risco real de desfazer um passo global alheio;
- **params de joint keyframáveis** — ⚠️ **a premissa registrada estava FALSA e foi
  corrigida**: o `PropKind::Morph` já dirige um campo de componente, então apendar
  variant **é** a forma. O bloqueio real é que o `PhysicsJoint` mora na
  `ph2d-physics-ecs`, que a `ph2d-timeline` não conhece — **três saídas nomeadas**
  no tracker.

E os que continuam sendo **cercas de Chesterton, não defeitos**: peça-dentro-de-peça ·
salto balístico do contrapeso · corpo que passa da própria roldana · a corda de IK
não colide · a trava ultrapassa por um sub-passo (0,3685 contra 0,5, pinado no gate).
