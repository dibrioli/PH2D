# Handoff de INTEGRAÇÃO — linha `line/FLIP` (2026-07-17)

> **Para o agente integrador** (DIRETRIZ §1.5.9), por ordem EXPLÍCITA do Enio. A linha está
> **fechada, com smoke OK, e PARADA**. O implementador não integra, não pusha, não roda ship.
> Este é o documento de integração; o [`…CONTINUACAO_2026-07-17`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-17.md)
> é o estado técnico do módulo (leia se precisar do porquê de cada decisão).

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/FLIP` |
| **Worktree** | `Worktrees/line-FLIP` |
| **HEAD** | `523127cd` (o commit acima é doc; o código é `c4fb6ade`) |
| **merge-base com `main`** | `4d203d48` |
| **`main` andou desde o fork?** | Sim: **5 commits, SÓ `project-memory/`** (memórias de outras linhas). **Zero interseção com código** — provado abaixo. |

**O que a linha entrega desde a última integração (a de 2026-07-13, que landou até FLIP=5 /
PROJECT=9):**
- **W7.5** — a pose da chave virou AFIM + o gizmo da POSE no Edit (rotate/escala da instância);
- **W8** — seleção no domínio POINT (meia-traço selecionável);
- **§4.A** — o gizmo da SELEÇÃO no Edit (rotate/escala assado nos pontos de arte exclusiva);
- **§4.B** — **Segment mode** (o 3º domínio: o pedaço entre dois cruzamentos). ← o bloco novo.

Todos com smoke do Enio OK.

---

## 2. ⚠️ Como integrar — **NÃO é `merge --ff-only`**, e o net-diff MENTE sobre 7 arquivos

A `main` **já contém** W5/W6 (a crate `ph2d-flip-reshape` e o Edit Mode existem lá,
`FLIP_SCHEMA=5`), mas os commits granulares desse trabalho **ainda aparecem em
`main..line/FLIP`** (foram squashados/recomitados na integração de 2026-07-13). Consequências:

1. **`git merge --ff-only line/FLIP` é impossível** — a `main` tem 5 commits que a branch não
   tem (as memórias) e a branch tem commits cujo conteúdo já está na `main`.
2. **O `git diff main..line/FLIP` tentaria DELETAR 7 arquivos** que a `main` adicionou depois
   do fork (todos em `project-memory/`). **NÃO aplique essas deleções.** São:
   `feedback_a_2_5d_analog_of_a_3d_operator_needs_the_lateral_recovered.md` ·
   `feedback_a_condition_that_enumerates_its_readers_rots.md` ·
   `feedback_a_hard_clamp_is_not_a_ceiling_it_is_an_eraser.md` ·
   `feedback_a_lateral_effect_needs_a_nonlocal_operator.md` ·
   `feedback_an_absolute_unit_that_should_feel_relative_must_scale_with_the_geometry.md` ·
   `feedback_growing_geometry_without_growing_matter_grows_nothing.md` ·
   `reference_display_topology_workstation.md`.

**Interseção REAL entre o que a linha autorou e o que a `main` autorou desde o fork: ZERO em
código.** A única sobreposição é `project-memory/`, e a **linha não autora memória** (foi
mantida fora da branch de propósito — as 2 lições novas desta sessão já estão escritas direto
na árvore primária, ver §7).

**Caminho recomendado:** rode o protocolo testado — `scripts/foundational-integrate.sh` (gate
da árvore combinada) + Mergiraf no resíduo textual (DIRETRIZ §1.5.3). Como não há interseção
de código, o merge `main` → branch (ou rebase da branch sobre `main`) resolve trivialmente nas
memórias; o único trabalho de verdade é reconciliar os **contadores de schema** (§4). Depois,
o gate da árvore combinada (`cargo check --workspace` + testes, NÃO `cargo check -p`) é quem
prova que o consumidor não quebrou.

**Tamanho do delta de código+docs:** 83 arquivos, +8402 / −335 (`git diff --stat main..line/FLIP -- ':!project-memory'`).

---

## 3. Foundational / compartilhado tocado — tudo APPEND-ONLY

Nenhuma assinatura existente mudou de forma (exceto o schema, §4). Os sítios em
`ph2d-editor-core` (foundational) são **6**, todos apendados por último:

| Arquivo | O quê | Risco |
|---|---|---|
| `ids/chrome/flip.rs` | **4 `NodeId`s novos** vs `main`: `FLIP_SCRUB` (W7.3), `FLIP_EDIT_DOM_STROKE`/`FLIP_EDIT_DOM_POINT` (W8), **`FLIP_EDIT_DOM_SEGMENT`** (§4.B, `hash_node_id("flip.edit.dom.segment")`). Arquivo **exclusivo do Flip**. | Baixo |
| `tests/node_id_collisions.rs` | as entradas dos ids acima na tabela do gate. **Lista compartilhada append-only.** | **Médio — UNIÃO** |
| `gizmo/drag.rs` | variantes **`GizmoTarget::FlipPose`** (W7.5) e **`GizmoTarget::FlipSelection`** (§4.A), apendadas por último. | Médio (se outra linha mexeu no `GizmoTarget`) |
| `gizmo/paint.rs` | 2 scramblers em `keyed_handle_id`: `0x_C3A5_C85C_97CB_3127` (pose) e `0x_5F1E_C7A0_2B94_D6E3` (seleção) · **`HANDLE_SIZE_PX` virou `pub`** (era `const` privado). | Médio |
| `gizmo/mod.rs` + `lib.rs` | `pub use` de `HANDLE_SIZE_PX`. | Baixo |
| `screens/hero/state.rs` + `screens/hero/paint.rs` | campos **`GizmoStateGroup.pose_view`** e **`.selection_view`** + os braços de pintura keyed. | Médio |

Colisão de mesmo-símbolo (outra linha mexeu no `GizmoTarget`/`GizmoStateGroup`/`keyed_handle_id`):
resolva pelos **ESTÁGIOS do índice** ([[feedback_resolve_conflicts_from_index_stages_not_markers]]),
NUNCA pelos marcadores, e rode `check --workspace` (merge limpo pode estar semanticamente
quebrado — [[feedback_clean_text_merge_can_be_semantically_broken]]).

**Sem crate nova neste delta** (a `ph2d-flip-reshape` já está na `main`). **Sem dep externa
nova. Cargo.lock não muda.**

---

## 4. ⚠️ Contadores de schema — CONTE, não escolha ([[feedback_numbers_that_sum_across_lines_count_dont_pick]])

| | `main` hoje | `line/FLIP` | delta da linha |
|---|---|---|---|
| `FLIP_SCHEMA_VERSION` (`crates/ph2d-flip/src/lib.rs`) | 5 | **7** | +2: W7.5 (`pose` afim, 5→6) · W8 (`point_sel`, 6→7) |
| `PROJECT_SCHEMA` (`shells/desktop/src/project.rs`) | 13 | **15** | +2, pelos dois acima |
| pin da tripla (`shells/desktop/src/project_tests.rs`) | `(13, 5, 8)` | `(15, 7, 8)` | (PROJECT, FLIP, VEC_SCENE) |

- `FLIP_SCHEMA = 7` é **meu sozinho** (ninguém mais toca `ph2d-flip`) — fica 7.
- `PROJECT_SCHEMA` **SOMA** todas as quebras de layout do arquivo de projeto. Se **outra linha
  também bumpou** PROJECT desde que a `main` chegou a 13, o valor integrado é
  `13 + 2 (meu) + N (dela)`, **não** 15 — e o pin `(…, 7, 8)` acompanha. **§4.A e §4.B não
  bumparam nada** (métodos + política de pick, não layout); os +2 são de W7.5/W8.
- `VEC_SCENE_SCHEMA = 8` (o `8` do pin) a linha **não toca** — se colidir, é outra linha.

---

## 5. Arquivos novos (todos exclusivos do Flip, prefixo `flip_`/`segment`)

- **Modelo:** `crates/ph2d-flip/src/segment.rs` (+ `segment_tests.rs`) — o motor do §4.B.
- **Shell (§4.B):** `flip_select_segment.rs` (+ `_tests`) · `flip_segment_smoke.rs`.
- **Shell (blocos anteriores deste delta):** `flip_pose_gizmo.rs` (+`_tests`) ·
  `flip_pose_smoke.rs` · `flip_selection_gizmo.rs` (+`_tests`) · `flip_selection_smoke.rs` ·
  `flip_select_pick.rs` · `flip_select_points.rs` (+`_tests`) · `flip_edit_smoke.rs`.
- **`.typos.toml`:** +2 palavras pt-BR (`acender`, `Repare`) na seção do Flip. **Chave
  duplicada mata o gate no parse** ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]])
  — se outra linha adicionou as mesmas, funda sem duplicar.
- Os arquivos `flip_*` que **toda linha encosta** (`main.rs`, `app_state.rs`,
  `input_dispatch.rs`, `render_loop/mod.rs`) mudam só por **linhas aditivas** (declarar
  módulos, campos de estado do gesto, roteamento de canvas/tecla) — união de blocos.

---

## 6. Gate da árvore combinada (o que PROVA a integração — não o `-p`)

Depois de fundir, na árvore combinada:
```bash
scripts/foundational-integrate.sh          # o gate da árvore combinada (DIRETRIZ §1.5.3)
# e, no fechamento da jornada, o ship COMPLETO (só por ordem do Enio):
scripts/ship.sh                            # paridade EXATA com o CI
```
`nextest-impacted` teve **false-green em RAM baixa** — no fechamento rode o `ship.sh` completo,
não o impacted. O replay-hash pode mudar (o postcard mudou de forma nos schema-bumps) — re-lock
esperado.

**Estado no fechamento da linha (gate batched, 1× sobre o diff acumulado):** 64 suítes verdes
(`ph2d-flip{,-fill,-render,-reshape}` · `ph2d-tool-flip` · `ph2d-panel-flip{,-frames}` ·
`ph2d-ui-testkit` · `ph2d-editor-core` · `ph2d-host-desktop`) · clippy `--all-targets` limpo ·
typos · `fmt` (rustup 1.95) · release builda · LOC caps OK (`flip_select.rs` a **568/600**, o
mais apertado). GPU `gpu_render`/`gpu_fill_fit` `--ignored` não foram tocados por este delta.

---

## 7. Memória — **fora da branch DE PROPÓSITO** (não a comite a partir daqui)

A linha **não carrega `project-memory/`**. O `project-memory/MEMORY.md` está **modificado e
não-comitado na árvore primária** (trabalho de outras sessões), e a cópia que a branch teria é
a ANTIGA — comitá-la reverteria as outras linhas e quebraria o `merge --ff-only` das memórias.

As 2 lições novas desta sessão **já estão escritas direto na árvore primária** (via o symlink
`~/.claude/projects/<key>/memory` → `project-memory/`), com o índice já atualizado:
- `feedback_the_representation_can_delete_the_special_case.md`
- `feedback_a_green_gate_may_be_green_by_accident.md`

O integrador **não precisa fazer nada com memória** — só não deixar o merge deletar os 7
arquivos que a `main` adicionou (§2).

---

## 8. Smoke — TODOS OK

- **§4.B:** `PH2D_FLIP_SEGMENT_SMOKE=1` — Enio 2026-07-17: *"smoke ok"*.
- W7.5: `PH2D_FLIP_POSE_SMOKE=1` · W8: `PH2D_FLIP_EDIT_SMOKE=1` · §4.A:
  `PH2D_FLIP_XFORM_SMOKE=1` — todos OK (ver o handoff de continuação).

---

**A linha fecha aqui. O integrador funde (§2), reconcilia os contadores (§4), roda o gate da
árvore combinada (§6), e só faz ship por ordem do Enio.**
