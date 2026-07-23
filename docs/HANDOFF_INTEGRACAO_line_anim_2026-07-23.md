# HANDOFF DE INTEGRAÇÃO — `line/anim` (duração explícita + containers como aba + bugs)

> **Modo L.** A linha está **FECHADA**, **todos os smokes aprovados pelo Enio**
> (*"Tudo ficou maravilhoso. Todos os bugs corrigidos e as animações complexas com
> múltiplos containers funcionam perfeitamente."* — 2026-07-23), **aguardando ordem
> explícita de integração**. A linha NÃO integra nem pusha sozinha (CLAUDE.md §0.7):
> integração e ship só por ordem do Enio, via um agente integrador dedicado munido
> deste handoff.

- **Branch:** `line/anim`
- **Base (merge-base com `main`):** `13a04c7aab` (`style(vector): fmt canonico do pin 1.95`) — **= HEAD do `main`** no fork, sem drift. Já inclui a integração do nesting (`line/anim-fixes`, ADR-0133) e o toggle **Physics** do transporte (physics W4b): esta linha foi construída **em cima** deles.
- **Tamanho:** 25 commits, 83 arquivos, **+7203 / −994**.
- **Data de fechamento:** 2026-07-23.

---

## 1. O que a linha entrega

A **duração explícita** (modelo composition-duration do After Effects) e a reforma de
navegação/transporte que ela puxou, mais 8 bugs de smoke fechados.

**Duração explícita.** Cada clip, cada container e o Arranje (a cena) tem um tamanho
**autorado** que define "o fim" (go-to-end, o loop recém-armado, a fatia de uma
instância nova) e **CORTA o excedente sem destruí-lo** — keys/strips além do fim ficam
autorados, o avaliador só clampa o relógio no corte (`clip_cut`/`container_cut`/
`cut_scene`). Editável pela caixa **Dur(s)** no transporte (stepper ±0,2 s) **e** por
uma **alça ↔ na régua** que arrasta o véu (grab-relative, snap igual ao playhead). Uma
duração autorada é uma duração **VISÍVEL**: o **véu** (a máscara escura pós-fim) e o
**clamp do playhead** aparecem desde o 1º frame. **Toda composição nasce com 4 s**
(clip novo, container novo, cena — o default do Enio).

**Containers como aba.** Entrar num container adiciona uma **ABA nomeada** no grupo
`[Keys | Containers | <container> | Arrange]`, nascida marcada; sai-se pelo mesmo
grupo. O breadcrumb solto (os botões "Scene"/container fora do grupo) **morreu**.
Dentro de um container o transporte é o **relógio DELE**, com loop independente por
modo.

**8 bugs de smoke fechados** (detalhe em [`HANDOFF_line_anim_duration_autokey_bugs_2026-07-23.md`](HANDOFF_line_anim_duration_autokey_bugs_2026-07-23.md)):
autokey/K minando keys além do corte (seed==sample) · Dur(s) autorando a vista ·
lane solta após o último strip · pingpong sem gap cíclico · additive de container
inerte · scrub pausado saltando na saída de strip com fade · foco em chip numérico
seleciona-tudo · e o default de 4 s.

---

## 2. Schema — **DOC_VERSION 9 → 11**, `PROJECT_SCHEMA` INTOCADO (29)

⚠️ **Duas coisas distintas, não confunda:**

- **`ph2d-timeline::DOC_VERSION` 9 → 11** (dois campos apendados na linha: o relógio
  próprio do container — nest field, 9→10 — e os `length_override` de clip/container/
  cena — 10→11). Postcard é **posicional**, então é **quebra dura**: docs de timeline
  v9/v10 são **recusados no load** (a política ADR-0133 que todo bump deste documento
  seguiu — *"v8 recusado no load"*). **Dev-only: nenhum save shipado é afetado.**
- **`PROJECT_SCHEMA` NÃO mudou (fica em 29).** O `TimelineDoc` viaja como **blob DENTRO**
  do `ProjectFile` e **carrega a própria versão** — a forma do `ProjectFile` não mudou,
  então o número global não anda (racional ADR-0133, verificado no diff). ⚠️ **Isto é
  o que evita colisão com as linhas de physics/vector/painter**, que bumpam
  `PROJECT_SCHEMA`: o número delas e o meu não competem porque **eu não mexo nele**
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]] não se aplica aqui — não
  há número somando).

---

## 3. Foundational compartilhado tocado (a lista de risco de MERGE)

Fora das crates da timeline, a linha toca foundational compartilhado. Tudo é
**append-only / isolado** (CLAUDE.md §0.2 — foundational novo desenhado para
isolamento), mas é **aqui que um merge textual pode acontecer** se `main` andou:

| Arquivo | Mudança | Isolamento |
|---|---|---|
| `ph2d-editor-core/.../interaction/types.rs` | `TimelineHitKind::DurationHandle` (variant **sem payload**) + 1 braço de match | append no enum de hit-kinds da timeline |
| `.../dispatch/{mod,pointer_down,focus,key}.rs` | foco em chip numérico **seleciona tudo** (`init_number_buffer` → `selection_anchor=Some(0)`; `NumberInput` pula `place_text_caret`) | comportamento novo do `NumberInput`, guardado por gate próprio |
| `.../interaction/state/{mod,store_core}.rs` | suporte ao select-all-on-focus | append |
| `ph2d-editor-core/src/ids/{chrome/timeline,menus_timeline}.rs` | `TIMELINE_DUR_HANDLE`, `TIMELINE_TRACK_MENU`, `TIMELINE_CRUMB[]` | ids novos (arrays const — cobertos por `node_id_collisions`) |
| `ph2d-i18n/src/lib.rs` | **1 chave apendada**: `"panel.timeline.length" => "Dur(s)"` | append |
| `shells/desktop/src/{main,app_state,input_dispatch,input_handlers}.rs` | wiring do handle + cursor `EwResize` + default 4 s no boot | edições pontuais |
| `shells/desktop/src/render_loop/{mod,autokey_pass,timeline_bridge}.rs` | corte do relógio de autoria; ponte de container/duração | ver §4 |

**Contrato congelado (§6): NENHUM tocado.** `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`
/`NodeOp`/`OpResolver`/`NodeManifest` intactos (grep confirmou zero toque na superfície).

---

## 4. Overlap cross-linha (o integrador MEDE — DIRETRIZ §1.5, [[feedback_integration_order_comes_from_measured_overlap]])

Como o base já é o HEAD de `main`, **se `main` não andou desde o fork o merge é
`--ff-only` limpo** — nada a medir. Se `main` andou (outra linha integrou primeiro),
os pontos de atrito prováveis, por já serem tocados por outras famílias:

- **`shells/desktop/src/render_loop/mod.rs`** e **`ph2d-panel-timeline/src/transport.rs`** —
  também são território da **física** (o toggle Physics já está no meu base; uma wave
  futura de física que volte a mexer em `transport.rs`/`render_loop` colide). Regiões
  provavelmente distintas, mas confira.
- **`crates/ph2d-editor-core/src/interaction/types.rs`** e **`ids/`** — qualquer linha
  que apende hit-kinds/ids no mesmo enum é colisão de mesmo-símbolo (DIRETRIZ §1.5.5 —
  **PARE e reporte ao Enio** se o rebase conflitar aqui fora dos meus arquivos).
- **`Cargo.lock`** — 1 linha (dep de dev do panel); resolve trivial.

Regra: **só ADICIONE** em listas compartilhadas; se o conflito for de mesmo-símbolo
fora dos arquivos desta linha, é escalada pro Enio, não renegociação.

---

## 5. Passos de integração (para o agente integrador, sob ordem do Enio)

1. `git fetch` + conferir se `main` andou desde `13a04c7aab`.
   - **Não andou:** `git merge --ff-only line/anim` (limpo).
   - **Andou:** rebase/merge medindo o overlap da §4; resolva pelos **estágios do
     índice**, não pelos marcadores ([[feedback_resolve_conflicts_from_index_stages_not_markers]]);
     varra marcadores em CADA commit.
2. **Gate da árvore combinada** — `check --workspace` (merge limpo pode estar
   semanticamente quebrado, [[feedback_clean_text_merge_can_be_semantically_broken]]).
3. **`./scripts/ship.sh`** (paridade CI: fmt, clippy `--all-targets`, machete, deny,
   audit, nextest `--cargo-profile ci-test`, typos). O ship do integrador **drena
   latentes** de outras linhas — 2 a 4 iterações são normais ([[project_integrator_ship_catches_latents_budget_iterations]]).
4. **Rode a suíte em DEBUG e RELEASE** — release-only esconde pânico ([[feedback_a_ship_x_can_be_the_environment_not_the_code]] e a lição do Flip).
5. Ship + push + babysit CI **só nesta última integração da jornada**, por ordem do Enio.

---

## 6. Verificação — gates e smokes

**Gates (todos verdes no fechamento, per-crate):**
- `ph2d-timeline` **306** · `ph2d-panel-timeline` **298** · editor-core (number-input
  focus, nesting seam) verdes · shell (timeline_bridge/autokey/orphan/nest) verdes.
- LOC caps (workspace + panel + shell HR-18) verdes; clippy `-p` **0**; release compila.
- Chaves red-first + mutação-provadas por bug (ver o handoff de bugs §, e
  `explicit_duration.rs`, `pingpong_scrub_exit.rs`, `duration_chip_gesture.rs`,
  `duration_drag_tests.rs`, `default_duration_tests`).

**Smokes (rode `--release`; a cena imprime o que montou):**
- **`PH2D_NEST_SMOKE=1`** — nesting básico (relógio do pai, elisão por fora).
- **`PH2D_NEST_SMOKE=2`** — o container "Jump" (3 clips, 2 lanes, 3 instâncias).
- **`PH2D_NEST_SMOKE=3`** — a **biblioteca** de containers (3 assets, um aninhado, um
  vazio) — o cenário das *"animações complexas com múltiplos containers"* que o Enio
  aprovou.
- Comando: `env PH2D_NEST_SMOKE=3 cargo run -p ph2d-host-desktop --release`
  ([[feedback_run_command_include_cd]] — a partir de `Worktrees/line-anim`).
- **Confira à mão:** abrir a timeline (`L`) mostra **Dur = 4** e o véu já visível;
  criar clip 2 → **4**; entrar em container vazio → **4**; entrar num container adiciona
  a **aba nomeada** no grupo; arrastar a alça ↔ na régua redimensiona a duração com snap.

⚠️ **Flake conhecida, PRÉ-EXISTENTE** (não desta linha): `the_cost_of_depth_is_linear_not_explosive`
(`ph2d-timeline/tests/nesting_clock.rs`) é gate de RAZÃO sensível a carga — passa
isolado; **re-rode sozinho antes de suspeitar do merge**.

---

## 7. Aberto (NÃO-bloqueante — Enio aprovou o conjunto)

- **Bug B (state-bleed percebido):** ao entrar em Arrange depois de configurar
  Clips/Containers, os checkboxes loop/pingpong e os tempos pareciam "mexidos". A
  metade **"Arrange dur = 0"** foi fechada (a cena nasce 4 s). A metade **"botões
  loop/pingpong mexidos"** seria o acoplamento tab↔stack de 2026-07-22 (`keys_mode`),
  um refactor que eu **não toco sem smoke que o reproduza** — e o Enio **não reportou
  recorrência** no smoke final. Fica nomeado; se voltar, o gate red-first precisa de
  *qual aba, com/sem pilha, qual campo*.
- **Higiene herdada** (CLAUDE.md §5, não desta linha): `vec_history` morto por limpar ·
  undo por-painel.

---

## 8. Atualizar a CLAUDE.md §5 (o integrador faz, na árvore combinada)

A entrada **Timeline** da §5 deve ganhar a duração explícita (modelo AE, `DOC_VERSION`
9→11, `PROJECT_SCHEMA` intocado), os **containers como aba** (fim do breadcrumb solto),
a **alça ↔ do véu**, o **default de 4 s**, e os 8 bugs — **só ADICIONE** contra a `main`
de hoje ([[feedback_a_shared_list_is_merged_against_todays_main]]); remover linha alheia
é trabalho de integração, não desta.
