# HANDOFF DE INTEGRAÇÃO — `line/Painter` FECHADA (2026-07-30)

> **Para o agente integrador**, munido pelo Enio (DIRETRIZ §1.5.9).
> Este é o documento **MESTRE** da linha: ele cobre os **74 commits** à frente do `main` e
> **indexa os dois handoffs parciais que ainda não foram integrados**. Leia este primeiro; os
> outros dois são o detalhe por frente.
>
> A linha está **FECHADA**. Não integrei e não pushei (CLAUDE.md §0.7).

## 1. Branch / HEAD / base

| | |
|---|---|
| branch | `line/Painter` (worktree `Worktrees/line-Painter/`) |
| commits à frente de `main` | **74** (`git log --oneline main..HEAD`) |
| diff | 137 arquivos, ~20.700 inserções |
| árvore | limpa |
| base | conferir `git rebase main` antes de tudo — a linha nasceu antes das integrações de physics/FLIP de 27/07 |

⚠️ **DOIS handoffs anteriores desta linha NÃO estão no `main`** e continuam válidos como detalhe:

| doc | commits | frente |
|---|---|---|
| [`..._undo_journal_2026-07-28.md`](HANDOFF_INTEGRACAO_line_Painter_undo_journal_2026-07-28.md) | 26 | o **journal por tile** (S3, degraus 1–3b) + o **Wet Paint a 4 FPS** |
| [`..._wet_perf_2026-07-30.md`](HANDOFF_INTEGRACAO_line_Painter_wet_perf_2026-07-30.md) | 46 | a frente de **PERFORMANCE** do Wet Paint (§1–§9) |

O último handoff desta linha **já no `main`** é o
[`..._undo_delta_2026-07-26.md`](HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md).

---

## 2. O que entra, em uma linha cada

### Frente A — o journal do undo (doc 28 §5.20–§5.30 + §7)

| # | O quê |
|---|---|
| A1 | **O journal captura o "antes" por TILE**, na hora da escrita — retido 67,11 → **0,13 MB**, constante na tela |
| A2 | **Portas que NOMEIAM o plano** + a rede que confere o invariante em toda rodada da suíte |
| A3 | ⚠️ **Um achado de produto:** `apply_material_to_stroke` escrevia o `mats` com `Arc::make_mut` **cru** — fora de toda porta, sem abrir acesso, com o passo de undo ficando INCOMPLETO |
| A4 | **O Wet Paint a 4 FPS** (independente do S3; veio de um smoke do Enio no meio da leva) |

### Frente B — a performance do Wet Paint (doc 28 §5.31–§5.45)

| # | O quê | doc |
|---|---|---|
| B1 | **A sim saiu da thread do frame** — água 15 → 33 Hz, pior tick 73 → 9 ms | [doc 29](Painter/29_offthread_sim.md) + §5.31–§5.38 |
| B2 | **Três passes row-disjuntos ao rayon** — passo 16,08 → 10,34 ms | **[ADR-0145](architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)** + §5.39 |
| B3 | **A CADÊNCIA** explica o 1,56× virar 1,10× — e o worker passa a REPORTAR | §5.40 |
| B4 | **`Grid Size`** — a grade do fluido desacopla do pixel; 31 → 83 Hz a 2:1, e o K–M curado pelo mesmo slider | §5.41 |
| B5 | **`Flow Grid`** — a multi-resolução: o fluxo é grosso, o pigmento é da tela | [plano 30](Painter/30_plano_multiresolucao.md) + §5.42 |
| B6 | **A secagem era a LIBM** — `drying_pass` 46,08 → 32,13 ms, **byte-idêntico** | §5.43 |
| B7 | **O `advect`** — a ablação atribuiu, o relógio negou (**resultado NEGATIVO, registrado**) | §5.44 |
| B8 | **O SOLVER FICOU INDEPENDENTE DE ORDEM** — 52,05 → **11,02 ms/passo**, água a **90,8 Hz** | **[ADR-0147](architecture/decisions/0147-wet-paint-order-invariant-solver.md)** + §5.45 |
| B9 | **A GPU tem ADR com o desenho inteiro, em PROPOSTA** (+ emenda 3, que a re-precifica) | **[ADR-0146](architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)** |

**Resultado de ponta a ponta, pela porta do artista** (poça de 4096², três faixas sobrepostas,
`Grid Size 1 + Flow Grid 4`): **~12,5 Hz → 90,8 Hz**, e a água **sai do regime work-limited** —
ela corre a 2,3× o nominal de 40 Hz da SPEC.

---

## 3. ⚠️ O que o integrador PRECISA saber (a lista de riscos)

### 3.1 Dep nova — UMA, e ela tem cerca

`rayon = "1"` na **`ph2d-wet-paint`** (+ a aresta no `Cargo.lock`). **Nenhum outro `Cargo.toml`
foi tocado; nenhuma crate nova.**

É a **2ª e a 3ª exceção** do repo ao *"sem rayon"* (a 1ª é o ADR-0109), e as duas são **decisões
distintas**, registradas no próprio `Cargo.toml`:

* **ADR-0145** — `project`, `smooth_velocity` e a metade row-disjunta do `rebuild_active_region`.
  Row-disjuntos **por construção**, resultado **byte-idêntico** ao serial.
* **ADR-0147** — `solver::advect_jacobi` e `drying::drying_pass_jacobi`. Aqui o que mudou foi o
  **MODELO**; ⚠️ **não** são "row-disjuntos que ninguém tinha visto" — a §2 do ADR-0145 os recusava,
  e **corretamente**.

`build_flow_field` e a SAIA do `rebuild_active_region` ficam seriais por semântica. **Qualquer uso
novo de rayon nesta crate exige ADR novo** — a tabela por-passe vive em `crates/ph2d-wet-paint/src/par.rs`.

### 3.2 Schema / contrato / ids

* **`PROJECT_SCHEMA` = 37, INTOCADO.** ⚠️ É o número que colidiu **duas vezes** neste mês entre
  `line/physics` e `line/FLIP` ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]) — esta
  linha **não o move**, então **não há o que contar**. Se o `main` do dia estiver acima de 37, nada
  aqui muda.
* **Contrato congelado (§6): INTOCADO** — `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` /
  `PanelEvent=4`, conferido por **grep** e pelo gate `architecture_tool_contract_surface`.
* **Ids novos:** `crates/ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs` ganhou
  `PAINTER_WETPAINT_GRID` e `PAINTER_WETPAINT_FLOW`; `PAINTER_WETPAINT_FIELDS` foi de 7 para **9**.
  ⚠️ Esse array é **iterado** pelo gate que parece cobri-lo (`the_knob_rows_are_offered_only_while_armed`),
  logo ele é **auto-referente** e não pega um id fora da lista — quem pega nomeia o id por
  **literal**. Merge aditivo; a contagem se **CONTA**.

### 3.3 ADRs — os três números são PROVISÓRIOS

**0145 · 0146 · 0147** nasceram nesta linha. ⚠️ Se outra linha reivindicou qualquer um deles na
mesma janela, **quem chegou ao `main` primeiro fica com o número** (gate
`architecture_adr_numbers_are_unique`, sem allowlist — já aconteceu 4× no repo). Renumerar é
mecânico: o arquivo, os links no doc 28, no `par.rs`, no `Cargo.toml`, no `CLAUDE.md` §5 e as
referências cruzadas entre os três.

### 3.4 ⚠️ O pino do fingerprint MOVEU — e há um gate que o audita

`crates/ph2d-wet-paint/tests/fingerprint.rs` tem agora **três** pinos:

| const | rota | o que prova |
|---|---|---|
| `PINNED` | produto (`order_invariant = true`) | o modelo de hoje |
| `PINNED_GAUSS_SEIDEL` | ablação `= false` | **que nada além dos dois passes do ADR-0147 mudou** |
| `PINNED_PRE_DOC23` | ablação `= false` + `wetLift = 0` | o modelo pré-doc-23, ao byte |

⚠️ **Se `the_gauss_seidel_route_still_reproduces_its_own_pin` ficar VERMELHO pós-merge, PARE.**
Ele não é um pino a mais: ele é o que separa *"a wave trocou o modelo"* de *"algum outro passe
regrediu no merge"*. Um `PINNED` vermelho sozinho é esperado se outra linha tocar o motor; os dois
vermelhos juntos é semântica.

### 3.5 Renomes que conflitam TEXTUALMENTE (não semanticamente)

| de | para |
|---|---|
| `Sim::advect_gather` | **`Sim::order_invariant`** (o flag cobre os DOIS passes) |
| `AdvectScratch` | **`SolverScratch`** |
| `Grid::adv` | **`Grid::scratch`** |

### 3.6 Superfície fora da própria crate

* **Shell (2 arquivos):** `shells/desktop/src/render_loop/mod.rs` + `render_loop/wet_grid_look_probe.rs`.
* **Foundational (1 arquivo):** o de ids acima. Aditivo.
* **`project-memory/` (5 arquivos):** memórias novas/atualizadas. ⚠️ Lista compartilhada — **só
  ADICIONE**; remover é integração.
* **`CLAUDE.md` §5 e `docs/Painter/28_*.md`: só APENDAM.** Se conflitar, é conflito de vizinhança —
  mantenha as duas metades.

### 3.7 Arquivos com maior chance de conflito

`crates/ph2d-wet-paint/src/{sim,grid,par,drying,solver}.rs` · `crates/ph2d-tool-painter/src/tool/paint/wetpaint*` ·
`crates/ph2d-panel-painter-layers/src/*` · `CLAUDE.md` · `docs/Painter/28_*.md`.

⚠️ **Cargo.lock: NUNCA resolva à mão** — regenere (`cargo check --workspace`).

---

## 4. Bateria de fechamento — rodada agora, na worktree

| gate | resultado |
|---|---|
| `ph2d-wet-paint` **release** | **108 passed, 0 failed** |
| `ph2d-wet-paint` **DEBUG** | **107 passed, 0 failed** |
| `ph2d-tool-painter` release | **931 passed, 0 failed** |
| clippy (`wet-paint` + `tool-painter` + `editor-core`, `--all-targets`) | **0 diagnósticos** |
| `architecture_workspace_file_loc_cap` | ok |
| `architecture_tool_contract_surface` | ok (4/4) |
| `architecture_adr_numbers_are_unique` | ok |

⚠️ **As DUAS profiles, de propósito** — a `line/FLIP` pagou a lição de que rodar só `--release`
esconde pânico (o `voronoi.rs` do colorize). Rode as duas na árvore combinada.

⚠️ **Gates que um `cargo test -p` filtrado NÃO alcança** e que só a árvore combinada expõe:
`shells/desktop/tests/file_loc_caps.rs` e os arch-gates de `shells/desktop/tests/` — a
`line/Vector` e a `line/physics` já fecharam com eles vermelhos no próprio tip por causa disso.

---

## 5. As mudanças de COMPORTAMENTO (o que o smoke julga)

### 5.1 ⚠️ O escorrido corre **~18% menos longe** (ADR-0147)

Medido pelo deslocamento do **centroide de massa** do filme, varrendo `Flow Grid` 1..8: o solver
independente de ordem transporta **0,64–0,96× (média ~0,82×)** do que o Gauss-Seidel transportava —
uniformemente, sem colapso, e **sem viés de direção** (a hipótese óbvia, *"a varredura de cima para
baixo cascateia com a gravidade"*, foi **REFUTADA por medição**: 1,14× e 1,09×).

**O knob `Gravity` traz o alcance antigo de volta.** É a única pergunta de olho da frente B.

### 5.2 Dois sliders novos na seção Wet Paint

`Grid Size (px)` **tem de ser a 1ª row da seção**, `Flow Grid` a 2ª, com o readout derivado
(`fluido 1024x512 - fluxo 256x128`) embaixo.

### 5.3 Memória

**+25 B por célula do fluido** (o rascunho derivado do solver), alocado **preguiçosamente no 1º
passo**. A 4096² grade 1:1 são +420 MB — e o slider `Grid Size` é a resposta, como já era para os
43 B/célula que o grid custava antes.

---

## 6. Smokes

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel grande, faixas **SOBREPOSTAS** (numa risca fina a água bate o nominal e o
número não diz nada). O que olhar:

1. `Grid Size 1 + Flow Grid 4` → a água corre **lisa**; o log mostra a taxa e o `busy/away/sleep`.
2. **O escorrido** (§5.1) — corre menos longe; suba o `Gravity` e confira que ele volta.
3. `Grid Size 4 + Flow Grid 1` é o contraste: ali a **borda granula** (o preço nomeado da §5.41).
4. O **Pigment Mixing (K–M)** do Tuning fica utilizável a partir de `Grid Size 2`.

Os smokes do undo/journal (frente A) estão na §8 do handoff `..._undo_journal_2026-07-28.md`.

---

## 7. O que fica ABERTO (nomeado, não escondido)

* **A GPU (ADR-0146) segue em PROPOSTA**, e a **emenda 3** a re-precifica: a metade cara (o modelo)
  foi paga na CPU, então um port hoje é **tradução, não redesenho** — mas o ganho tem de ser medido
  contra **11 ms, não contra 52**, e os dois bloqueadores que nunca foram sobre o solver continuam
  de pé (o **stamp** recebe a silhueta do Painter por closure; a **residência** dos 14 planos é
  all-or-nothing).
* **A composição do passo pós-§5.45 NÃO foi re-medida.** Toda wave desta jornada moveu a fronteira
  de lugar; escolher o próximo alvo sem re-medir é o erro que a §5.13 documentou.
* `MAX_FLOW_RATIO = 16` não é teto de recurso — é onde a grade deixa de resolver o pincel. O número
  final é decisão de smoke.
* O **backrun fica esparso** em `Flow Grid > 1` (um sítio de nucleação por bloco) — mudança de
  desenho **para o smoke**, sem gate numérico de propósito.
* A cura da granulação do `Grid Size` (ler a tile de cerdas em escala de **canvas**) é **wave
  própria com smoke próprio** — nomeada, não contrabandeada.

## 8. ⛔ Medido e REJEITADO — não refaça

* **Tabular a razão K/S** junto com a transferência sRGB (doc 24): mal-condicionada no branco, e uma
  lavagem **parada** deixava de ser parada.
* **Reusar a moldura bilinear do `advect`** (§5.44): o LLVM já fundia por CSE; A/B deu **nada**.
* **Reduzir (em vez de amostrar) os planos finos** para a grade de fluxo (§5.42 F1): `O(finas)`, não
  encolhe com `rf`, e custa 12,7× mais.
* **Fatorar o `build_flow_field`** (§5.42 F1): 99,4% dele é o NÚCLEO; o backrun custa 0,6%.
* **Somar deslocamento no smear** / **clamp só de um lado do gather** — os dois criam ou destroem
  massa; o porquê está no doc-comment de cada porta.
