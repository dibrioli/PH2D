# Handoff de integração — `line/Painter` (jornada de PERFORMANCE), 2026-07-26

> **Para o agente INTEGRADOR.** A linha está **FECHADA**, todos os smokes foram **aprovados pelo Enio**, e
> a ordem de integrar é dele. Leia §0 e §1 antes de qualquer comando.

---

## §0 — O essencial em dez linhas

| | |
|---|---|
| **Branch** | `line/Painter` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| **Commits** | **86** à frente de `main` |
| **Diff** | 113 arquivos, +16.065 / −1.201 |
| **Rebase** | ⚠️ **JÁ CONTÉM `main`** — `git merge-base --is-ancestor main HEAD` responde SIM ⇒ **`--ff-only` é possível hoje**. Se `main` andar antes da integração, rebase primeiro. |
| **`PROJECT_SCHEMA`** | **31 nos dois lados** — a linha **não bumpou nada** |
| **Contratos congelados (§6)** | **INTACTOS** — `git diff main..HEAD` em `ph2d-editor-core/src/tool.rs` e `ph2d-nodegraph/` é **vazio** |
| **ADR novo** | **nenhum** ⇒ a linha fica **fora** de qualquer disputa de número desta janela |
| **Suíte** | 868 verdes em **debug E release**, clippy 0, LOC verde, typos 0 |
| **Natureza** | **quase toda PERFORMANCE**, com dois fixes de correção (o escorrido do Wet Paint no Ctrl+Z; o SIGSEGV do fechamento) |

---

## §1 — O que o integrador precisa saber ANTES de rodar o gate

### 1.1 ⚠️ Rode a suíte em **DEBUG também**, não só `--release`

Esta linha instalou uma **rede de verificação que só existe em `cfg(debug_assertions)`**: o motor de delta
do undo confere, a cada commit estrutural, que a janela declarada por quem escreveu contém a janela
verdadeira. Ela é o que torna a otimização da §5.19 segura, e **ela não roda no `ci-test`/`--release`**.

```
cargo test -p ph2d-tool-painter            # ~4 s — é AQUI que a rede corre
cargo test -p ph2d-tool-painter --release
```

O `ship.sh` usa o perfil `ci-test`, então **ele sozinho não exercita a rede**. Rode o debug à parte.

### 1.2 ⚠️ Gates de RAZÃO são sensíveis a CARGA — a família documentada no CLAUDE.md

Rodar `cargo test -p ph2d-tool-painter -- --ignored` (102 testes de medição em paralelo) faz falhar
`the_parallel_fork_is_actually_faster…`, `sculpt_perf_kill_criterion`, `smear_perf_kill_criterion`,
`warp_perf_kill_criterion`, `deform_*_perf_*` e `the_fold_costs_what_the_window_costs…`. **Todos passam
isolados** e passam com `--test-threads=1`. Re-rode sozinho antes de suspeitar do merge.

### 1.3 ⚠️ DOIS gates deliberadamente VERMELHOS, PRÉ-EXISTENTES (não são desta linha)

`watercolor_app_params_incremental_matches_full_diluted` e `…_mixer_on` falham. O próprio `#[ignore]`
deles diz: *"RED conhecido (doc 12 take 7) … Vira gate regular quando o residual for corrigido"*.
**Conferido no `HEAD` anterior à jornada: falham lá também.**

---

## §2 — O que a linha entrega (por ordem de tamanho do número)

Detalhe completo e reprodutível em **[`docs/Painter/28_otimizacoes_o_que_funcionou.md`](Painter/28_otimizacoes_o_que_funcionou.md)**; o CLAUDE.md §5 tem o resumo.

| Frente | Medido (4096²) | Doc |
|---|---|---|
| **O fold do relevo caminha por LINHAS** | `dispatch` **201,5 → 14,55 ms** (o *"delay do primeiro traço"* que o Enio reportou 3×) | §4.8.2 |
| **A porta única de fork do canvas** | pen-down do Blur **11,64 → 3,66 ms** | §5.15 |
| **O move do Wet Paint** | **13,71 → 1,82 ms**, e **plano na tela** | §5.12 |
| **O commit de undo (a janela declarada)** | **23,72 → 12,16 ms**; pen-up **37,0 → 24,0** | §5.19 |
| **O Ctrl+Z** | **46,56 → 23,41 ms** | §5.16 |
| **O AA do filme de impasto** | 2,60 → **1,43 ms/dab** | §4.6.1 |
| **O histórico de undo (U1, por delta)** | retido **1.627 → 242 MB**; por passo 67,8 → 2,36 | plano 26 §7.5 |
| **O pen-down do digital** | 10,3 → **3,9 ms** | §4.5 |

Mais duas **correções**: o **escorrido do Wet Paint sobrevivia ao Ctrl+Z** (a divergência que o `undo.rs`
previa e declarava sem repro — era a água, e o smoke do Enio a achou) e o **SIGSEGV do fechamento** (a
superfície EGL morria depois do `wl_display`).

⚠️ **Uma boa parte do valor desta linha são NEGATIVOS medidos** — otimizações construídas e **rejeitadas
com número** para ninguém as refazer: o AA com menos amostras, colapsar a grade do AA, a coalescência de
eventos, a frente dos tiles, a extração paralela da janela, o atalho da janela pelo `mark_dirty`, o guard
com `Drop`. Elas estão no doc 28 com o motivo e a medição.

---

## §3 — Foundational e shell tocados (o que pode conflitar)

⚠️ Tudo **aditivo**; nenhum símbolo removido de API pública fora da própria `ph2d-tool-painter`.

**Foundational**
- `ph2d-painter-brush` — `dab.rs` (+ `dab/bands.rs`), `footprint.rs`, `height.rs`, `height_film.rs`,
  **`height_film_lut.rs` (novo)** + os dois arquivos de teste irmãos, `lib.rs` (re-exports).
- `ph2d-render` — `impasto_light.rs` (+ `_tests`), `tests/impasto_light_gpu.rs`,
  `tests/measure_first_stroke_pipelines.rs` (novo).
- `ph2d-editor-core` — `ids/menus.rs`, `screens/hero/pre_populate.rs`,
  `screens/hero/context_menu_dialogs.rs` (**4096 no modal New Image**, e a largura dele virou DERIVADA),
  `tests/every_new_image_choice_is_alive_under_the_mouse.rs` (novo).
- `.typos.toml`, `Cargo.lock` (só a aresta do `dhat` como dev-dep), `SKILL_Stack` (emenda do HR-13).

**Shell (`shells/desktop`)**
- `render_loop/` — `paint_perf.rs` (+ `_tests`), `painter_bridge.rs`, `painter_gpu_preview.rs`
  (+ `_tests`), `painter_preview_handoff_tests.rs`, `painter_preview_undo_tests.rs`,
  `measure_bridge_phases.rs`, `mod.rs`.
- `input_dispatch.rs` (+ `painter_canvas_input.rs`), `app_state.rs`, `main.rs`, `impasto_smoke.rs`.
- **4 gates novos em `shells/desktop/tests/`** (ver §4.2 — eles NÃO correm num `cargo test -p` por crate).

**A crate da linha** — `ph2d-tool-painter`: além do módulo de pintura, os arquivos **novos**
`plane_copy.rs`, `undo_window.rs`, `undo_delta.rs`/`undo_planes.rs`/`undo_absorb.rs` e os `_tests` irmãos.

---

## §4 — Gate de fechamento: o que rodar, e o que NÃO basta

### 4.1 A sequência

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo test -p ph2d-tool-painter              # DEBUG — a rede de verificação (§1.1)
cargo test -p ph2d-tool-painter --release
cargo test -p ph2d-painter-brush --release
cargo test -p ph2d-render --release
cargo test -p ph2d-editor-core --release
cargo test -p ph2d-host-desktop --release    # ⚠️ §4.2
cargo clippy --all-targets --all-features
```
…e depois o `./scripts/ship.sh` da árvore combinada.

### 4.2 ⚠️ Os gates de `shells/desktop/tests/` NÃO correm num fechamento por-crate filtrado

É a família que a `line/physics` e a `line/Vector` já documentaram (o miss do `file_loc_caps`, os dois
arch-gates vermelhos no próprio tip). Esta linha **adiciona quatro** deles:

- `the_close_gesture_tears_down_the_gpu_first.rs`
- `the_first_stroke_does_not_compile_shaders.rs`
- `the_impasto_fold_walks_the_dirty_rect.rs`
- `the_pointer_clock_starts_where_the_paint_starts.rs`

Rode `cargo test -p ph2d-host-desktop --release` **inteiro**, sem filtro.

### 4.3 Gates de GPU

`ph2d-render/tests/impasto_light_gpu.rs` é `#[ignore]` e precisa de adapter. Na RTX:
`cargo test -p ph2d-render --release -- --ignored`. **Sem adapter ele faz skip gracioso, que não é
verde** (a mesma armadilha que a `line/FLIP` documentou).

---

## §5 — Riscos de merge, nomeados

1. **`docs/Painter/28_*.md` e `CLAUDE.md` §5** são os dois arquivos mais prováveis de conflitar (outras
   linhas escrevem no mesmo parágrafo do §5). ⚠️ **Só ADICIONE** — remover linha alheia é decisão de
   integração, não de merge.
2. **`Cargo.lock`** — regenerar (`cargo check --workspace`), **nunca** resolver à mão.
3. **`.typos.toml`** — lista compartilhada; funde por adição.
4. **`ids/menus.rs`** — a linha acrescentou id(s) para o 4096 do New Image. Se outra linha acrescentou
   ids, **o número se CONTA, não se escolhe**: reconferir o próximo livre e o gate de colisão.
5. **Nenhum schema, nenhum ADR, nenhum contrato congelado** ⇒ esta linha não disputa número com ninguém.

---

## §6 — Smokes (todos APROVADOS pelo Enio nesta jornada)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
env PH2D_WETPAINT_SMOKE=1                  cargo run -p ph2d-host-desktop --release
env PH2D_MASK_SMOKE=1                      cargo run -p ph2d-host-desktop --release
```

Canvas **4096²** é onde os números desta linha vivem. O `PH2D_PAINT_PERF=1` imprime o split do frame
PIOR — é o instrumento que nomeou a causa do *"delay do primeiro traço"* e é ele que o integrador deve
olhar se algo parecer lento depois do merge: **`dispatch max` tem de ficar em ~1 ms, não em 232**.

---

## §7 — O que fica ABERTO (com número, para o handoff seguinte)

1. **12,16 ms ainda no commit de undo** — extração dos dois lados da janela + os planos que nenhum sítio
   declarou. **Número aberto, não estimativa.**
2. **S3 — o journal por tile**: guardar os PIXELS da região e fazer o Ctrl+Z **aplicar o patch ao plano
   VIVO** em vez de instalar um snapshot materializado. Aí o `cursor` larga os planos, a contagem de
   donos cai para **um**, e **o fold (11,9 ms) e o fork do pen-down somem juntos**. Plano em três degraus
   no doc 28 §7 — S1 e S2 já landaram.
3. **O outlier de 134,8 ms num único evento (frente R)** segue sem atribuição; o maior evento
   reprodutível é o pen-up, agora em 24 ms.
4. **Semear os planos da luz no bind (frente P)**: **12,7 ms medidos** contra ~218 MB de VRAM em todo
   bind — **decisão de produto do Enio**, com os dois lados já precificados.
5. **`EVENTO→FRAME` 16,8 contra o alvo 9 (frente S)** — **não é compute** (`p50 ≈ período real`); é
   cadência/pipeline, de outro dono.
6. ⚠️ **Achado LATENTE fora do escopo:** o `mark_dirty` do Inflate declara *onde a imagem mudou*, não
   *onde bytes foram escritos* — então a pista de **upload parcial** recebe a mesma reivindicação curta.
   Invisível hoje (o excedente tem delta abaixo do `RELIEF_EPS`), mas real. **Não corrigido**: alargar o
   `moved` muda o custo de repaint e é decisão do dono do sculpt.

---

## §8 — Recado do implementador

Duas coisas que valem mais que o diff:

- **A rede de verificação em debug é o padrão que esta linha deixa.** Ela reprovou uma otimização
  minha na **primeira rodada da suíte** — antes de qualquer gate especializado, antes de qualquer smoke.
  Custa 4 s e cobre todo gesto que algum teste encena. Quem for mexer no motor de delta do undo, mantenha-a.
- **Metade desta jornada foi otimização REJEITADA por medição.** O doc 28 registra cada uma com o número
  que a matou, e a §6 dele tem as lições de método. A mais cara: *uma sonda que constrói os próprios
  dados logo antes de medi-los mede o cache, não o produto* — foi ela que quase me fez paralelizar algo
  que rende **zero**.
