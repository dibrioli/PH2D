═══════════════════════════════════════════════════════════════════
AUDIT → Vector W3 · T3.5 · `vector.boolean` + `vector.offset` (motores exatos CPU)
Autor: Implementador Vector · 2026-06-04 · baseline `98da9d3` (+ commits locais T3.3/T3.4)
Escopo: o **reconcile path exato** (CPU Linesweeper/kurbo). O SDF draft GPU NÃO existe
ainda (Coord) → lente B fica bloqueada. Gates executáveis > claims verbais.
═══════════════════════════════════════════════════════════════════

## Lente A — robustez do Linesweeper (onde Clipper falha) ✅ PASSA

Gate executável: `cargo test -p ph2d-node-vector-boolean --test edge_cases` — **12/12 verde**.
Cobre os casos degenerados clássicos que quebram engines Clipper-family:

| Caso | Resultado |
|---|---|
| **Coincident edge** (2 rects compartilham aresta x=2) | Union dissolve a borda → 1 rect; Intersect → ∅ (sem área) ✅ |
| **Shared vertex** (2 quadrados tocam só no ponto (2,2)) | sem panic; Intersect → ∅ ✅ |
| **Tangent contact** (2 círculos tocam em (1,0)) | Union/Intersect/Exclude válidos, sem panic ✅ |
| **Boundaries idênticas** (A op A) | A∖A→∅, A∪A=A, A∩A=A, A⊕A→∅ ✅ (o degenerado mais difícil p/ sweep-line) |
| **Nested/hole** (rect grande − rect interno) | emite outer+hole = 2 contours; Intersect→inner; Union→outer ✅ |

**Veredito:** o dep **early-beta `linesweeper` 0.3 sobrevive aos casos que o T3.5 lente A mira.**
O risco que sinalizei na ratificação está **substancialmente mitigado** para a nossa geometria
(shapes de source/boolean/offset). Recomendação p/ depois (T5+): **fuzz** antes de confiar em
input arbitrário de usuário (o crate é "minimal maintenance"); se achar bug é upstream → reporta.

## Lente C — perf (medido em `--release`, M-series Mac) ✅ ADEQUADO

Gate: `cargo run --release -p ph2d-node-vector-boolean --example perf`. Números medidos:

| Cenário | Tempo | Por-op |
|---|---|---|
| 100 Union ops independentes (pares de 24-gon) | 14.4 ms | **0.144 ms/op** |
| 500 ops mistos (stress §plano) — sem crash | 73.4 ms | 0.147 ms/op |
| Accumulate-union 50 polys (49 ops encadeados, blob final 404 segs) | 48.9 ms | ~1 ms/op (complexidade crescente) |

**Veredito:** o exato é o **reconcile (async/on-commit, ADR-0059 §2.4)**, NÃO o per-frame
(esse é o SDF draft GPU). Op única de 2 shapes = **sub-ms** (parece instantâneo). 100 ops em
14ms cabe folgado num commit interativo. Frame-budget per-frame é responsabilidade do SDF draft
(≤0.5ms, ainda Coord). **Adequado pro papel.**

## Golden cross-OS ✅

`golden_intersection_corners_are_exact_on_the_grid` (em `edge_cases.rs`): Intersect([0,2]²,[1,3]²)
= [1,2]² com cantos **inteiros exatos** (1,1)/(2,1)/(2,2)/(1,2) após snap Q16.16, e byte-stable em
re-run. O snap erra a drift f64 last-ULP → bytes idênticos cross-OS (mesmo mecanismo do source).

## Lente B — SDF vs Linesweeper consistency ⛔ BLOQUEADA

O SDF draft shader (`boolean_sdf.wgsl` + wiring no renderer) **não existe** (cross-crate, Coord —
vide HANDOFF_vector_w3_t33_boolean_coord §3.A). Não dá pra comparar silhueta SDF vs topology
Linesweeper sem ele. **Reabrir quando o SDF landar** (T5.2 / Coord).

## Achados / limitações documentadas

- **F1 (renderer, → Coord): holes não renderizam como holes.** Boolean nested emite outer+hole
  como **2 regions** no carrier (fiel ao resultado). Mas `draw_vector_network` preenche cada region
  independente e aditivamente → o hole é pintado por cima (some). Corrigir exige **suporte a
  multi-loop/parent-region (ou even-odd cross-region) no renderer** — fora do meu isolamento.
  Sem impacto nos casos sem-hole (Union/Subtract de shapes não-aninhados), que são o smoke W3.
- **F2 (escopo W3, documentado): semântica de ops compostos.** Merge≡Union, Crop≡∩, Divide=
  {A∖B,A∩B,B∖A}, Trim={A∖B,B∖A}, Outline=boundary da Union. Distinção Pathfinder de cor/bias e
  width-expansion outlining ficam pra outline-stroke/styling (T-later).
- **F3 (offset): só regiões fechadas.** Open-path offset → region é do `vector.outline-stroke`
  (§2.2.4). Region-less input → vazio (documentado, testado).

## Estado de fechamento W3

- **Motores exatos (boolean+offset) DONE + auditados** (engines isolados, 28 unit + 12 edge-case
  + perf + golden, todos verde; clippy/machete/contract/staleness verdes).
- **Falta pra W3 fechar de fato (tudo Coord/cross-crate, vide HANDOFF coord):** SDF draft +
  wiring, bridge multi-nó + memoização do Cook, panel chrome, e então lente B reaberta.
═══════════════════════════════════════════════════════════════════
