# Handoff de integração — `line/Painter`, a frente de PERFORMANCE do Wet Paint (2026-07-30)

> **Para o agente integrador.** Este handoff cobre os **44 commits** desde o
> [handoff de 28-29/07](HANDOFF_INTEGRACAO_line_Painter_undo_journal_2026-07-28.md)
> (que fechou na doc 28 §5.30). Tudo aqui é **uma frente só**: *quem paga o
> tempo da simulação de aquarela, e quanto ele custa de fato.*
>
> **Todos os smokes foram aprovados pelo Enio.** A linha está **fechada** —
> não integrei e não pushei (CLAUDE.md §0.7).

---

## 1. O que entra, em uma linha cada

| # | O quê | doc |
|---|---|---|
| 1 | **A sim saiu da thread do frame** — água 15 → 33 Hz, pior tick 73 → 9 ms | [doc 29](Painter/29_offthread_sim.md) + 28 §5.31-§5.38 |
| 2 | **Três passes row-disjuntos ao rayon** — passo 16,08 → 10,34 ms | **[ADR-0145](architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)** + §5.39 |
| 3 | **A CADÊNCIA** explica o 1,56× virar 1,10× — e o worker passa a REPORTAR | §5.40 |
| 4 | **A grade do FLUIDO desacopla do pixel** (`Grid Size`, 1..30) — 31 → 83 Hz a 2:1; e o K–M curado pelo mesmo slider | §5.41 |
| 5 | **A GPU tem ADR com o desenho inteiro, em PROPOSTA** | **[ADR-0146](architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md)** |
| 6 | **A MULTI-RESOLUÇÃO** (`Flow Grid`) — o fluxo é grosso, o pigmento é da tela | [plano 30](Painter/30_plano_multiresolucao.md) + §5.42 |
| 7 | **A SECAGEM era a LIBM** — `drying_pass` 46,08 → 32,13 ms, byte-idêntico | §5.43 |
| 8 | **O `advect`** — a ablação atribuiu, o relógio negou (**resultado NEGATIVO, registrado**) | §5.44 |

**Resultado de ponta a ponta, pela porta do artista** (poça de 4096², três
faixas sobrepostas): **~12,5 Hz → 21,1 Hz** com `Flow Grid 4`, e a água saiu do
regime *work-limited* na thread do frame.

---

## 2. O que o integrador PRECISA saber

### 2.1 Dep nova — e ela tem cerca

`rayon = "1"` na **`ph2d-wet-paint`**. É a **2ª exceção** do repo ao *"sem
rayon"* (a 1ª é o ADR-0109), autorizada pelo Enio com a palavra *"rayon"*, e o
**ADR-0145 é a cerca**: ela entra em **exatamente três** passes, e
**qualquer uso novo exige ADR novo**. O racional por-passe está no
`crates/ph2d-wet-paint/src/par.rs` e repetido no `Cargo.toml`.

### 2.2 Schema, contrato, ids

* **`PROJECT_SCHEMA` = 37, INTOCADO.** ⚠️ Ele é o número que colide entre
  linhas nesta janela ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]):
  esta linha **não o move**, então não há o que contar. Se o `main` do dia
  estiver acima de 37, nada aqui precisa mudar.
* **Contrato congelado (§6): INTOCADO** — `Tool=12` / `RasterEditTool=5` /
  `CanvasPaintTool=1` / `PanelEvent=4`, conferido por grep e pelo gate
  `architecture_tool_contract_surface` (4/4).
* **Ids novos:** `crates/ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs`
  ganhou `PAINTER_WETPAINT_GRID` e `PAINTER_WETPAINT_FLOW`; `FIELDS` foi de 7
  para **9**. ⚠️ Esse array é **iterado** por um gate (`the_knob_rows_are_offered_only_while_armed`),
  então ele é **auto-referente** e não pega um id fora da lista — o gate que
  pega nomeia o id por **literal**. Se outra linha mexeu no mesmo arquivo, o
  merge é aditivo e a contagem se **CONTA**.
* **Nenhuma crate nova. Nenhum ADR renumerado** (0145 e 0146 nasceram aqui;
  ⚠️ se outra linha reivindicou 0145/0146 na mesma janela, quem chegou ao
  `main` primeiro fica com o número — gate `architecture_adr_numbers_are_unique`).

### 2.3 A rede de segurança que decide tudo

**`crates/ph2d-wet-paint/tests/fingerprint.rs`** — a sessão roteirizada,
byte a byte. **Toda** wave desta frente é reescrita de hot loop e se prova por
ele; ele está **intocado** desde o ADR-0134. Se ele ficar vermelho depois do
merge, **pare**: não é conflito textual, é semântica.

Rode também `tests/acceptance.rs`, `product_doors.rs`, `parallel_rows.rs`,
`resumable_step.rs`, `spans.rs` e `flow_grid.rs`.

### 2.4 Arquivos com maior chance de conflito

| arquivo | por quê |
|---|---|
| `crates/ph2d-tool-painter/src/tool/paint/wetpaint.rs` + `wetpaint/` | 6 módulos novos (`offthread`, `grid_map`, `dab_route`, `session`, `flow_ratio_tests`, `grid_ratio_tests`) |
| `crates/ph2d-panel-painter-layers/src/paint_wetpaint.rs` | os dois sliders + o readout no TOPO da seção |
| `crates/ph2d-editor-core/src/ids/chrome/painter_wetpaint.rs` | `FIELDS` 7 → 9 |
| `shells/desktop/src/render_loop/mod.rs` | o `[frame]` do `PH2D_FLUID_PROFILE` |
| `docs/Painter/28_otimizacoes_o_que_funcionou.md` | 14 seções apendadas (**só apende**) |
| `CLAUDE.md` §5 | idem — **só apende**, remover é integração |

---

## 3. Os smokes (todos aprovados, para re-conferência pós-merge)

```
cd <worktree ou main>
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel grande, **Wet Paint no dropdown**, faixas
**SOBREPOSTAS** (numa risca fina a água já bate o nominal e o número não diz
nada). O que olhar:

1. A seção abre com **dois sliders** — `Grid Size (px)` e `Flow Grid (x)` — e um
   **readout em inglês** logo abaixo (`fluid 4096x4096 - flow 1024x1024`).
2. `Grid Size 1 + Flow Grid 4` ⇒ a água corre **lisa** e a **borda fica FINA**;
   compare com `Grid Size 4 + Flow Grid 1`, que granula o pigmento.
3. O log `[frame]` traz `worker: busy X% away Y% sleep Z% | TAXA DA AGUA N Hz`.
4. Trocar qualquer razão **encerra a água viva** (a tinta fica; o escorrido em
   voo, não).
5. O Tuning → **Pigment Mixing (K–M)** fica utilizável com `Grid Size` 4.

---

## 4. Aberto, com o preço medido ao lado

* **A alavanca de CPU deste módulo está ESGOTADA.** Composição do passo hoje:
  `advect` **70,4%** · `drying_pass` 21,9% · `rebuild` 5,2% · `build_flow` 1,7%
  · **portável-exato 0,7%**.
* **A próxima alavanca é a GPU**, e ela é **decisão do Enio + ADR próprio**:
  o **ADR-0146 está em PROPOSTA** com o desenho inteiro, as 5 fases, os 3
  gatilhos que a re-abrem, e **duas emendas** que a re-precificaram para PIOR
  (a metade portável-exata encolheu de 2,5% para 0,7%). Ele quebra o port 1:1 e
  o fingerprint pinado do ADR-0134.
* **`MAX_FLOW_RATIO = 16`** não é teto de recurso: é onde a grade de fluxo deixa
  de resolver o pincel. O número final é do smoke.
* **A aparência da amostra**: com `Flow Grid` alto o *backrun* fica esparso (um
  sítio de nucleação por bloco). Deliberadamente **sem gate numérico** — o
  oráculo é o olho.
* **⛔ Não refaça** (medidos e rejeitados, cada um com o parágrafo): tabular a
  razão K/S (§5.41 do doc 24) · reusar a moldura bilinear do `advect` (§5.44) ·
  paralelizar `advect`/`build_flow_field`/`drying_pass` (ADR-0145 §2).


---

## 9. O SOLVER INDEPENDENTE DE ORDEM (ADR-0147 · doc 28 §5.45) — a wave que fechou a frente

Acrescentada **depois** da versão original deste handoff. Ordem do Enio: *"GPU do Wet Paint"*.

**O que mudou:** `advect` e `drying_pass` passam a ler o estado do **início do passe** (forma de
Jacobi) e vão para o `rayon`. O `advect` vira **gather conservativo**; a secagem materializa o
**fator de borda** num pré-passe.

**Por que — e não é velocidade:** numa folha espelhada (cena cuja física é simétrica por
construção) o Gauss-Seidel desloca **1189,29** unidades de massa no advect e **554,82** na secagem;
o independente de ordem desvia **0,000000** (`tests/solver_symmetry.rs`).

**Ganho, pela porta do produto** (4096², ciclo de 12 passos, mesmo processo):

| `Flow Grid` | antes | depois | |
|---|---:|---:|---|
| 1 | 60,19 ms (16,6 Hz) | **29,29 ms (34,1 Hz)** | 2,06× |
| 4 | 52,05 ms (19,2 Hz) | **11,02 ms (90,8 Hz)** | 4,72× |

### O que o integrador PRECISA saber

1. ⚠️ **`rayon` em mais dois passes = 3ª exceção do repo** — **ADR-0147** (número **PROVISÓRIO**,
   conte-o contra o `main` do dia). A `ph2d-wet-paint` **já** tinha a dep (ADR-0145); **nenhum
   `Cargo.toml` foi tocado**.
2. ⚠️ **O pino do fingerprint MOVEU, com o protocolo do doc 23.** `PINNED` é o do produto; o pino
   ANTIGO virou `PINNED_GAUSS_SEIDEL`, executável na rota `Sim::order_invariant = false`. **Se o
   `the_gauss_seidel_route_still_reproduces_its_own_pin` ficar vermelho pós-merge, PARE** — ele é o
   que prova que nada além destes dois passes mudou.
3. **`PROJECT_SCHEMA` = 37, INTOCADO.** A shell **não foi tocada** (`git diff shells/` vazio).
   Contrato congelado intacto (gate + grep).
4. **Arquivos novos:** `src/solver/advect_jacobi.rs` · `src/grid/scratch.rs` ·
   `tests/solver_symmetry.rs` · `tests/measure_transport_range.rs` ·
   `docs/architecture/decisions/0147-*.md`.
5. **Renomes que podem conflitar textualmente:** `Sim::advect_gather` → **`Sim::order_invariant`**
   (o flag cobre os DOIS passes) · `AdvectScratch` → **`SolverScratch`** e `Grid::adv` →
   **`Grid::scratch`**.
6. ⚠️ **Um gate teve o ORÁCULO trocado, não a barra:** `flow_grid.rs::gravity_carry` passou da
   célula mais extrema acima de um limiar para o **centroide de massa**. Ele passa nos **dois**
   modelos — é isso que prova que a troca é honesta.

### O que o SMOKE decide

⚠️ **Uma mudança de comportamento, medida e nomeada: o escorrido corre ~18% menos longe**
(0,64–0,96×, média 0,82×, uniforme e sem viés de direção). O knob **Gravity** traz o alcance antigo
de volta. **É a única pergunta de olho desta wave.**

Comando: `env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release`
— canvas **4096**, pincel grande, faixas **SOBREPOSTAS**, `Grid Size 1 + Flow Grid 4`.
