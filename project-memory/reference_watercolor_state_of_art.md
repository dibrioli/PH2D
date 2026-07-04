---
name: reference-watercolor-state-of-art
description: Watercolor literature benchmark — the Painter fluid engine is canonical Curtis 1997 + extensions; reading list + next-step ranking
metadata: 
  node_type: memory
  type: reference
  originSessionId: 63f87272-2a56-4ba6-a970-0da82f8d28dc
---

Deep-research benchmark (2026-06-08, verified multi-source) of the PH2D Painter watercolor
engine vs. the academic state of the art. Full cited report:
[`docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/Painter_projeto/pesquisa_aquarela_estado_da_arte.md).

**Verdict:** the engine sits squarely in the canonical lineage. Its 4 layers map 1:1 to
**Curtis et al. 1997 "Computer-Generated Watercolor" (SIGGRAPH '97)** — the named algorithms
we already cite (`MoveWater`/`FlowOutward`/`TransferPigment`/`RelaxDivergence`/`SimulateCapillaryFlow`)
are his. Real-time-GPU lineage = Van Laerhoven & Van Reeth (CAVW 2005) + Scott TAMU 2004.

**Our one EXTENSION beyond the canon:** the capillary layer co-advects DISSOLVED PIGMENT.
Curtis's capillary is **water-only + backrun-only** — co-advecting pigment is novel (sound +
Enio-validated, but not a published model). The rigorous published path for a pigmented fringe
is MoXi (LBM percolation).

**Frontier / next-step ranking (engineering synthesis — no published ranking exists):**
1. **Kubelka–Munk multi-pigment mixing** (rest of S4) — biggest visual leap (blue+yellow=vibrant
   green, translucent glazing). Today we carry 1 linear-RGB pigment mass/cell; K–M needs per-pigment
   K/S. Practical path = **Mixbox / Sochorová & Jamriška 2021** (MIT code). This is [[project-vector-node-opaque-carrier]]-adjacent in spirit: a contained model upgrade.
2. BFECC/MacCormack advection — cheap sharpness win (2nd-order transport).
3. MoXi/LBM capillary — realistic fiber fringe, but a big substrate rewrite.
4. Thin-film substrate (Adobe "Dripping Thin Films", CGF/Eurographics 2026) — SOTA real-time,
   best for dripping; different fluid PDE, large rewrite.

Caveat: Mixbox-vs-K–M tradeoffs were an OPEN LEAD (not verified to primary). Specific Curtis
constants that circulate on mirrors (μ=0.1, ν=0.01) did NOT verify — only `FlowOutward` η∈[0.01,0.05]
is confirmed from the primary PDF. Related: [[project-painter-fluid-4k-perf-architecture]].
