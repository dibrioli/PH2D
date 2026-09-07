# HANDOFF DE INTEGRAÇÃO — `line/motion-value` · 2026-09-07

> **Ordem do Enio (2026-09-07): integrar ao main.** ⚠️ **Esta linha FICOU DE FORA da rodada
> anterior** — as outras integraram a 06/09 e já voltaram a implementar. Ela traz, portanto,
> **duas jornadas** de trabalho contra um `main` que as outras linhas vão mover na mesma rodada.
>
> Este documento é **auto-suficiente para a integração**. O
> [handoff de 06/09](HANDOFF_INTEGRACAO_line_motion_value_2026-09-06.md) continua a valer para o
> *mecanismo* da primeira metade (o substrato do cartão + os ciclos 1 e 2) e traz no §14 o
> adendo desta segunda; aqui está o que a integração precisa, medido hoje.

---

## §1 — Identidade

| | |
|---|---|
| ramo | `line/motion-value` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| merge-base | `004150bea` — **já rebasada em cima do `main` de hoje** (`git rev-list --count HEAD..main` = **0**) |
| commits | **57** |
| ficheiros | **170** |
| integração | `--ff-only` possível a partir deste ponto |

---

## §2 — O que a linha ENTREGA (uma frase por bloco)

1. **Os params dos nós vivem DENTRO dos cartões** (ciclo 1 da dinâmica dos ciclos, doc 103) — a
   faixa, o LOD, as secções dobráveis, o arrasto, o clique, a caixa de escrita.
2. **O ciclo 2 (ANIMADORES)** — 5 waves + a medição do grupo + o tutorial em PDF.
3. **O cartão passou a ALCANÇAR todos os controlos**: o censo `what_the_card_still_cannot_reach`
   foi de **26 → 0** sobre 683 rows do catálogo.
4. **O painel lateral de params SAIU** (a ordem do Enio de 05/09, cuja condição era esse zero).
5. **Uma crate-folha nova, `ph2d-param-editors`** — os três editores ricos com dois hospedeiros.

---

## §3 — ⚠️ SUPERFÍCIE PARTILHADA (a saída do `collision-surface.sh`, não de memória)

**Schemas: nenhum se mexe.** `PROJECT_SCHEMA` 121 = base 121 · a tripla `(121, 13, 22)` = base ·
`FLIP_SCHEMA` 13 · `DOC_VERSION` 18. **Registos de componentes:** 82 = base. **Contrato congelado
(§6): intocado** nos dois ficheiros. **ADR: esta linha não cria nenhum** ⇒ fora de toda disputa de
número. **Marcadores de conflito: nenhum.** **Tectos de LOC: nenhum ficheiro da linha passa.**

### §3.1 — ⭐ A CRATE NOVA

`crates/ph2d-param-editors` — **a membresia é por glob** (`crates/*`), então o `Cargo.toml` da
workspace **não muda**. O `Cargo.lock` ganha `+ph2d-param-editors` (o único pacote novo; as
outras linhas do lock são arestas internas). Se o lock conflituar, **regenere** em vez de fundir.

### §3.2 — Os ficheiros FORA do módulo, um a um

| ficheiro | linhas | o quê | risco |
|---|---:|---|---|
| `ph2d-editor-core/src/screens/task_layout.rs` | +9 −1 | o layout `Nodes` passa a **nomear `"inspector"`** | ⚠️ **quem mexer nos layouts colide**; a linha é obrigatória (ver §5.2) |
| `shells/desktop/tests/a_layout_never_commands_a_panel_a_bridge_owns.rs` | +8 −1 | o controlo perde `"motion"` | ⚠️ é o gate que **obriga** à linha acima |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | +14 | isenção do `paint_card.rs` | ⛔ vermelho **pré-existente de 06/09** |
| `ph2d-editor-core/tests/architecture_motion_chrome_never_wraps_a_row_label.rs` | +10 −1 | a crate nova entra no `SCANNED_CRATES` | a lei segue o código |
| `ph2d-editor-core/src/text_elide.rs` | +75 | `title_elided_width` (a largura no peso em que se PINTA) | append-only |
| `ph2d-editor-core/src/paint_shapes.rs` | +25 | `fill_polygon` | append-only |
| `ph2d-editor-core/src/interaction/types.rs` | +9 | um campo no `GraphGesture` | ⚠️ tipo partilhado |
| `ph2d-ui-testkit/src/lib.rs` | +20 −1 | o arnês aceita eventos de painel | append-only |
| `ph2d-motion-region/src/lib.rs` | +11 | o doc/`#[must_use]` do `is_rect` (cura de 06/09) | trivial |
| `shells/desktop/src/main.rs` | +6 | registo das duas sondas novas | ⚠️ **ficheiro-hub** |
| `scripts/tutorial-pdf.sh` | +66 | a ferramenta do PDF | novo |
| `ph2d-node-registry-init/tests/spring_ceiling.rs` | ver §5.3 | o gate mudou de canto | ⚠️ o gate **já estava no main** |

---

## §4 — Ordem sugerida e dependências

Esta linha **não depende de nenhuma outra**. Ela toca `ph2d-editor-core` em quatro sítios
(dois deles TESTES de arquitectura que medem os painéis de toda a gente), então:

- ⚠️ **integre-a DEPOIS de quem mexe em `task_layout.rs`** — a linha dela é uma entrada numa
  lista de strings, trivial de re-aplicar, e não o contrário;
- ⚠️ **os dois gates de arquitectura (`hr12_widgets_a11y`, `..._never_wraps_a_row_label`) têm
  contagens e listas que OUTRAS linhas também editam** — se conflituarem, **a fusão é a UNIÃO
  das entradas**, nunca um dos lados;
- ⛔ **o `Cargo.lock` regenera-se**, não se funde.

---

## §5 — ⛔ As SETE coisas que uma leitura rápida do diff entende ao contrário

1. **«O painel de params foi apagado»** — não foi: está **desligado**
   (`PH2D_MOTION_PANEL=1` traz-no de volta) e a crate continua a ser a casa das rows que a shell
   constrói (o gerador de tutoriais, os params de um subgrafo, a leitura de volta do selector).
   **Apagá-la parte o cartão.**
2. **«O layout ganhou o inspector por gosto»** — não: o gate
   `a_layout_names_the_inspector_exactly_when_its_canvas_owner_does_not_take_it_over` varre as
   PONTES à procura de `insert("inspector", !…)` e **exige** que o layout de quem não a faz
   nomeie o inspector. Apagar a linha da ponte reprovou o gate até o layout ser corrigido.
3. **«O `publish` do painel já não é preciso»** — ⛔⛔ ele é quem **DRENA as intenções de param**,
   e desde o ciclo 1 a maioria vem do **CARTÃO**. Saltá-lo com o painel fora pararia o cartão
   inteiro: todo arrasto, toda caixa, todo selector ficariam mudos, e nada diria porquê. Há gate
   (`with_the_side_panel_out_the_card_still_writes`) e a mutação mata-o.
4. **«O censo a ZERO autoriza tudo»** — ele mede o que um clique numa row FAZ e é **cego** a se a
   escolha CHEGA ao documento. A outra metade tem gates próprios na shell
   (`motion_bridge_color_card_tests`).
5. **«A mudança de casa dos editores é um refactor cosmético»** — ela existe porque o painel
   SAI, e os ids são FNV de uma **string**: uma string diferente compilaria, desenharia, e faria
   o arrasto falar de um widget que ninguém pintou. Há gate (`moving_house_moves_no_id`).
6. **«O teto de `friction` da mola mudou»** — **não mudou** (continua 20). O que mudou foi a
   RAZÃO: ver §5.3.
7. **«O `stagger` só ganhou um param»** — ele tinha o par `"order", "seed"` **duplicado** na
   lista do kernel, e por isso **não tinha shader no dispositivo** desde a W2 do ciclo 2.

### §5.2 — Por que o Inspector volta

A tomada de conta da coluna da direita existia por uma razão só: o `motion_params` ocupava-a.
Sem ninguém a tomar o lugar, escondê-lo passaria a ser **tirar uma superfície e não pôr
nenhuma** — o artista ficaria sem o Inspector *e* sem os params. É também o que o Enio pediu em
31/08 (*«em animate o inspector está sendo escondido; por padrão deve ficar visível»*).

### §5.3 — ⚠️ O teto da mola: o número ficou, a razão inverteu-se

O gate `the_friction_ceiling_holds_at_the_lowest_tension_not_just_the_highest` **já estava no
`main`** e reprovava desde a W1 do ciclo 2 (que é desta linha). Re-medido pela sonda do produto,
no relógio do pior caso:

| tensão | último `friction` sadio | primeiro mau |
|---|---:|---|
| 0,1 (piso) | 1.280 | 1.400 EXPLODE |
| 20.480 | 40 | 45 SALTA |
| **1.600.000 (o teto)** | **20** | **40 EXPLODE** |

A derivação original dizia que o pior caso era a tensão **mínima** (`2 / MAX_DT = 20`); depois de
a W1 curar o buraco de estabilidade, no piso a mola aguenta 1.280 e quem prende passou a ser o
canto **oposto**. O gate mudou de canto (mesma propriedade: *o teto é a borda, não folga*) e a
tabela do doc passa a dizer a verdade de hoje. ⛔ O teto **não sobe para 40** — a 20.480 o 40 é
sadio, no canto ele explode, e o canto é o que um teto tem de sobreviver.

---

## §6 — ⛔⛔ O que o portão de fecho apanhou, e a CAUSA (leia isto antes de fechar outra linha)

**Nove vermelhos**, todos desta linha, **todos em crates que ela não editou** — e por isso
invisíveis a todas as corridas por-crate que fiz:

| vermelho | onde vive o gate |
|---|---|
| 4 tectos de LOC (`geom.rs` 661 · `interact_param_row_tests.rs` 633 · `state.rs` 607 · `params/snapshot.rs` 610) | `ph2d-editor-core/tests` |
| censo de elisão 30 → 27 (os editores mudaram de crate) | `ph2d-editor-core/tests` |
| HR-12 a11y do `paint_card.rs` (**pré-existente de 06/09**) | `ph2d-editor-core/tests` |
| 2 números sem marcador | `ph2d-editor-core/tests` |
| **o `stagger` sem shader no device** | `ph2d-gpu-cook/tests` |
| **o teto da mola** | `ph2d-node-registry-init/tests` |

⇒ ***Um fecho que só corre as crates que a linha EDITOU é cego aos gates de arquitectura, que
vivem noutra.*** Os quatro tectos foram curados por **corte por responsabilidade** (nunca por
tolerância): `geom_card.rs` · `state_menu.rs` · `interact_param_row_selector_tests.rs` ·
`snapshot_channel.rs`.

⚠️ **E o `collision-surface.sh` foi quem os achou.** Ele é a primeira coisa que a integração deve
correr, e devia ter sido a primeira coisa que a SESSÃO correu.

---

## §7 — Portão de fecho (batched, 1×)

| passo | resultado |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `scripts/nextest-impacted.sh` | ✅ **13.397 testes, 13.397 verdes** (1.373 saltados) |
| clippy `--all-targets` nas **16** crates do diff | ✅ zero |
| `collision-surface.sh` | ✅ (§3) |
| `doc-index.sh --check` | ✅ 16 índices em dia |

⚠️ **Flake NOMEADA, confirmada e não corrigida:**
`flip_smooth::…::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` — membro declarado da
família do §5.0. **3 de 3 verde sozinha a `load 8,67`**, zero linhas do diff naquela área, e a
suíte inteira verde com `--no-fail-fast`.

---

## §8 — O que só o `ship.sh` apanha (o gate de integração não corre)

`typos` · `machete` · `deny`/`audit` · o `doc-index --check` da **árvore fundida** · e a matriz
3-OS do `physics_ecs_c9`. ⚠️ **Esta linha não toca física nem schema**, então o risco ali é o de
sempre (os três OS discordarem), não um risco dela.

---

## §9 — O que fica ABERTO (para o §5 do `CLAUDE.md`, não para esta linha)

1. ⏳ **O smoke do TUTORIAL do ciclo 2** (`docs/Motion Nodes/tutoriais/02_animadores.pdf`) — o
   Enio ainda não o percorreu, e é a aceitação que fecha o ciclo. **É o único bloqueio do ciclo
   3** (Transformes & Deformadores).
2. ⏳ Os ciclos **3–9** e os três do fim (o carimbo no device · a avaliação de performance · os
   tectos confortáveis) — doc 103 §5.
3. ⏳ **Retirar a crate `ph2d-panel-motion-params`** só é possível depois de mudar de casa o que
   a shell ainda lhe pede: `ParamRow`/`ParamsSnapshot` (o gerador de tutoriais, os params de um
   subgrafo) e o canal `MotionParamIntent` (que hoje é do **módulo**, não daquele painel — ele
   já vive em `snapshot_channel.rs`).
4. ⏳ O `Bug #7` (a fileira de 4 ondas da cena `=95`) segue aberto, adiado pelo Enio.

---

## §10 — Limpeza

`rm -rf target/*/incremental` corrido no fecho (DIRETRIZ §1.5.9 item 7).
