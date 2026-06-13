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

### B1 — pintar repetido no mesmo lugar → vira PRETO — **RESOLVIDO (2026-06-13)**
**Sintoma:** sobrepor traços no mesmo ponto escurecia sem limite até o preto.
**Causa:** o pigmento é absorbância Beer–Lambert (`a = −ln(c)·mass`) e o splat **SOMA** `(absorb,mass)`
no campo **sem teto**. Overlap acumula `a` → `exp(−a)` → 0 = preto.
**Fix:** **saturação de papel no composite** (`composite.wgsl`, `MASS_MAX=1.0`). O hue por unidade de
massa é `absorb/mass` (= −ln(c)); capamos a massa efetiva em `MASS_MAX`, então uma célula saturada
glaza para `exp(−(absorb/mass)·MASS_MAX) = c` — o masstone do pigmento — e nunca mais escuro. Física
crua (conservativa) intacta; mudança de **um kernel só**. Edge-darkening sobrevive (a borda concentra
mais massa que o interior, então ainda lê mais escura, só limitada na cor do pigmento).
**Gate:** `inv_overlap_saturates_to_pigment_not_black` (50× overlap → masstone (218,89,89), não preto).
Casa com K–M/Mixbox futuro (a saturação é parte do modelo) — quando entrar, é troca do mesmo kernel.

## PRÓXIMAS ETAPAS (roteiro, ADR-0086 §8)
1. **EM ANDAMENTO:** seção "Wash" enxuta na UI (sliders relevantes vs os 17 da seção Watercolor).
2. Cor subtrativa real (K–M / Mixbox) — fecha a limitação RGB **e** o B1 (saturação).
3. Franja capilar water-only (Curtis-faithful) — se faltar a borda suave além do traço.
4. Perf residual: dobrar o wash no encoder do render principal (1 submit/frame).
