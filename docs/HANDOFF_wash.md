# HANDOFF — Wash (núcleo mínimo de aquarela, ADR-0086/0087)

Tracker vivo do modo **Wash** (crate `ph2d-painter-wash` + bridge `painter_wash_bridge.rs`).
Build: `cargo run -p ph2d-host-desktop --features wash` · toggle "Wash" no Brush Studio.

## Estado (2026-06-13)
- **Fases 1-3 DONE** (ADR-0087): crate (solver + WashCompositor) + seletor brush/tool + bridge no
  shell + toggle UI. Pintável, mutuamente exclusivo com Fluid v2 (que fica intacto).
- **Perf:** hot path zero-readback (textura override + slot copy), region-scoped (janela ativa),
  backdrop só no início do traço, finalize-on-idle (1 bake + para). Regime ~0.8ms CPU + ~2.3ms GPU
  (super-estimado pelo poll de profile). `PH2D_WASH_PROFILE=1` imprime cpu/gpu/seed/dirty/err.
- **Primeiro-traço delay (~0.5s): RESOLVIDO** — `fluid_prewarm_paper` agora gera o papel no hover
  pro wash também (era O(grid) no clique).

## BUGS CONHECIDOS (a resolver)

### B1 — pintar repetido no mesmo lugar → vira PRETO (Enio 2026-06-13)
**Sintoma:** sobrepor traços no mesmo ponto escurece sem limite até o preto (screenshot).
**Causa provável:** o pigmento é absorbância Beer–Lambert (`Dab::from_color_mass`: `a = −ln(c)·mass`)
e cada dab **SOMA** absorbância no campo **sem teto**. Overlap acumula `a` → `exp(−a)` → 0 = preto.
Difere do v2, cujo `coverage_k`/`alpha = 1−exp(−amount·k)` **satura** a cobertura por célula.
**Direções de fix (quando voltarmos):**
1. **Saturar a deposição/cobertura por célula** — limitar a massa/absorbância acumulada (ex.: a
   absorbância tende a um teto por canal, ou `mass` satura como no v2) → overlap converge a uma cor
   sólida, não a preto.
2. Ou compor com cobertura saturante (alpha) em vez de multiplicação pura sobre fundo.
3. Casa com a re-introdução de **K–M/Mixbox** (cor subtrativa real) — a saturação de pigmento é
   parte do modelo K–M; resolver junto evita refazer o composite duas vezes.

## PRÓXIMAS ETAPAS (roteiro, ADR-0086 §8)
1. **EM ANDAMENTO:** seção "Wash" enxuta na UI (sliders relevantes vs os 17 da seção Watercolor).
2. Cor subtrativa real (K–M / Mixbox) — fecha a limitação RGB **e** o B1 (saturação).
3. Franja capilar water-only (Curtis-faithful) — se faltar a borda suave além do traço.
4. Perf residual: dobrar o wash no encoder do render principal (1 submit/frame).
