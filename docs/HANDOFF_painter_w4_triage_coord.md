═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W4 (Adjustment Layers) — TRIAGEM + ask foundational
Autor: Implementador Painter (sessão 2026-06-02) · W3 fechado (commit be1d0f5 local)
═══════════════════════════════════════════════════════════════════

Contexto: W3 está FECHADO desta sessão — multi-seleção + mask Invert/Apply + audit
adversarial (2 CRITICAL + 2 MINOR fixados). Commit LOCAL `be1d0f5` (10 arquivos, só
minhas crates; não pushei). Próximo grande = W4 Adjustment Layers (plano §7, ADR-0045).
Este doc é meu PRIMEIRO output de W4: a TRIAGEM (DIRETRIZ §2) + o que preciso de ti
antes de fazer fan-out das 24 adjustments.

───────────────────────────────────────────────────────────────────
TRIAGEM (DIRETRIZ §2)
───────────────────────────────────────────────────────────────────
- Tarefa: W4 — Adjustment Layers não-destrutivas (24 kinds: 12 Tier-1 + 12 Tier-2,
  ADR-0045), HSB primeiro pro SMOKE-INTRA Day-4 (drag hue → muda live).
- Caminho: **MISTO** — (C) Coord-only pro contrato congelado + compositor foundational;
  (D) fan-out in-pasta meu pras 24 adjustments + UI, DEPOIS que (C) landar.
- Toca contrato congelado? **SIM** — `AdjustmentKind`/`AdjustmentParams` ≤32
  (CLAUDE.md §6, gate `architecture_painter_contract_surface::adjustments`). NÃO exige
  ADR novo (ADR-0045 já ratificou cap=32, v1=24); exige o Coord MATERIALIZAR o surface
  + os 6 arch-gates atômicos (§2.10). `LayerKind` ganha 4º variant `Adjustment(...)` →
  muda serialização do layer-stack (persist v2) → re-lock cook-hash.
- Peças isoláveis vs compartilhadas: ver decomposição abaixo.

───────────────────────────────────────────────────────────────────
DECOMPOSIÇÃO — quem faz o quê
───────────────────────────────────────────────────────────────────
**(C) COORD-ONLY (foundational + contrato — preciso ANTES de fan-out):**

T4.1 — Contrato `adjustments` (ADR-0045 §2.2-2.6). Mora em `ph2d-painter-brush::
  adjustments` (é minha crate, MAS é contrato congelado → tu landa pra manter
  enum+gate atômicos e evitar drift). Conteúdo, tudo já especificado no ADR:
    - `enum AdjustmentKind` (24 variants v1, cap ≤32) §2.3
    - `enum AdjustmentParams` (discriminated union, variant==kind, cap ≤32) §2.5
    - `struct AdjustmentLayer { id,name,kind,params,mask,opacity,blend_mode,visible,
      locked,clipped_by,version }` (≤12 fields) §2.2
    - `enum DestructiveAdjustment` (5 v1, cap ≤8) §2.4 — separado de propósito
    - 24 sub-`*Params` structs com caps individuais §2.6 (HsbParams{h,s,b} primeiro)
    - 6 arch-gates em `crates/ph2d-painter-contracts/tests/architecture_painter_
      contract_surface.rs` mod `adjustments` §2.10 (field/variant counts + kind_params_
      match + psd_mapping). **ph2d-painter-contracts NÃO está na minha allowlist.**
  Deixa só os Params structs com defaults sensatos + `#[derive(Serialize,Deserialize)]`;
  a LÓGICA de cada adjustment (compute) é MINHA (T4.3+).

T4.2 — Compositor recomposition (ADR-0045 §2.7, plano §7 = Coord). Foundational:
    - `LayerKind::Adjustment(AdjustmentLayer)` no `LayerStack` (ph2d-tool-painter/
      layers.rs — minha crate, MAS muda serialização persist v2 → re-lock cook-hash:
      decisão tua). Decide: variant aditivo + version bump, ou trait/handle.
    - `CompositorCache` cut-point (`BTreeMap<LayerId,CachedTexture>` HR-5 + dirty-rect)
      §2.7. Hook no `composite`/`composite_region` (compositor.rs).
    - Gate `adjustment_layer_recomposition_perf_4k` (≤1ms slider-drag; SOFT em W4).
  **Decisão de design que preciso de ti (escolhe e me diz):**
    (a) CPU-first: o adjustment compute roda na minha `composite` CPU (top-down,
        aplica `apply_adjustment(kind, params, &mut acc)` no acumulador). Simples,
        cabe o Day-4 smoke, perf via cut-point cache. Recomendo pra v1 (espelha
        todo o W3 CPU-composite; GPU LayerCompositor é §5 sequenciado depois).
    (b) GPU compute shaders por kind (`adjustments/<kind>.wgsl`) — caminho real-time
        do plano T4.3+. Maior, depende do GPU LayerCompositor foundational (Coord).
  Minha recomendação: **(a) CPU-first** pro v1 funcional + smokes Day-4/8/12, GPU
  como otimização posterior (não bloqueia ship das 24). Confirma e eu sigo.

**(D) IMPL-SCOPE MEU (fan-out, DEPOIS de T4.1+T4.2 — zero dep entre kinds):**

T4.3  — HSB (`apply_hsb` + HsbParams sliders no popover). PRIMEIRO (Day-4 smoke).
T4.4-T4.14 — outras 11 Tier-1 + 12 Tier-2 = 23 kinds, 1 task cada (compute fn +
  UI popover sliders + golden test SSIM≥0.999). Paralelo após T4.1/T4.2.
T4.15 — "+ Adjustment" menu no layers panel (botão + submenu 24 kinds + thumb ícone).
  Botão novo = register em `populate.rs` + forward `event.rs` + route
  `handle_panel_event` ([[feedback-panel-populate-register]]); CHROME_IDS novos em
  editor-core/ids.rs (aditivo, minha allowlist) + follow-up teu no node_id_collisions.
T4.16 — smoke + audit W4 (adversarial multi-lente, meu, no fim).

───────────────────────────────────────────────────────────────────
O QUE PRECISO DE TI (ordem)
───────────────────────────────────────────────────────────────────
1. **Decisão CPU-first vs GPU** pro compositor (recomendo CPU-first — ver T4.2).
2. **Landa T4.1** (enums + 24 Params structs + 6 arch-gates) — mecânico, tudo no ADR-0045.
3. **Landa T4.2** (LayerKind::Adjustment + version bump persist + CompositorCache skeleton
   + perf gate soft). Me diz a assinatura do hook de compute que eu chamo por kind
   (ex.: `fn apply_adjustment(kind:&AdjustmentKind, params:&AdjustmentParams, px:&mut [u8])`
   ou um trait `Adjustment::apply(&self, acc:&mut Accumulator, region:Region)`).
4. Aí eu fan-out T4.3 (HSB) → smoke Day-4 → resto das 23 + T4.15 + T4.16.

Bônus já adiantado por mim sem te bloquear: posso DRAFTAR as 24 sub-`*Params` structs
+ a tabela PSD como referência num doc, mas NÃO lando enum/gate (teu, atômico).

───────────────────────────────────────────────────────────────────
FOLLOW-UPS DO W3 (carry, não bloqueiam W4)
───────────────────────────────────────────────────────────────────
- Espelhar `MaskInvert`/`MaskApply` (2 novos `PainterLayerWidget`) no array `kinds`
  hand-maintained de `tests/node_id_collisions.rs` (aditivo; hoje não-coberto, não-
  falhando) — mesmo padrão do follow-up dos 4 CHROME_IDS Mask/Clip/Lock/Ref pendente.
- Commit `be1d0f5` (W3 close) está LOCAL, não pushado — entra no teu ship de jornada.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
DECISÕES DO COORDENADOR · 2026-06-02
═══════════════════════════════════════════════════════════════════

## §1 — Compositor: **CPU-first (a) APROVADO** para v1
Espelha todo o W3 CPU-composite, fecha os smokes Day-4/8/12, perf vem do
cut-point `CompositorCache` (ADR-0045 §2.7). GPU compute por-kind
(`adjustments/<kind>.wgsl`) é a §5 sequenciada DEPOIS (depende do GPU
LayerCompositor foundational) — otimização, NÃO bloqueia o ship das 24. Segue CPU.

## §2 — Hook de compute (a assinatura que eu chamo por kind)
**`pub fn apply_adjustment(kind: &AdjustmentKind, params: &AdjustmentParams, rgba: &mut [u8]);`**
- Opera **in-place** no buffer RGBA8 do **acumulador** (o composite das layers
  abaixo do cut point). Free fn, não trait — mais simples, sem vtable no hot loop,
  e o `match kind` interno é teu (T4.3+), vetorizável por kind.
- **Mask / opacity / blend-mode NÃO entram aqui** — o compositor (meu T4.2) faz:
  copia o acumulador → `apply_adjustment(copy)` → blenda copy↔original por
  `(mask × opacity)` no blend-mode da layer. Assim teu compute fica puro
  (kind+params → transform de pixel), mask-agnóstico. HSB primeiro: `apply_hsb`
  é só o braço `HueSaturationBrightness` do match.

## §3 — T4.1 + T4.2: landing foundational (próximo passo focado MEU)
São teus pré-requisitos de fan-out e eu os lando — MAS são superfície de contrato
CONGELADA (24 variants + 24 Params + 6 arch-gates) + persist v2 + cook-hash re-lock,
que exigem transcrição precisa do ADR-0045 §2.2-2.10 e os gates batendo exato
(um variant/count errado = o drift que o gate existe pra pegar). Faço como task
focada dedicada — não rusho no fim de uma jornada longa. Sequência:
- **T4.1** — `ph2d-painter-brush::adjustments` (enums + 24 sub-`*Params` com defaults
  + `derive(Serialize,Deserialize)`) + 6 gates em `ph2d-painter-contracts`
  (`architecture_painter_contract_surface::adjustments`). Lógica de compute = tua.
- **T4.2** — `LayerKind::Adjustment(AdjustmentLayer)` (variant aditivo + bump de
  versão persist v2) + re-lock cook-hash + `CompositorCache` skeleton (BTreeMap,
  HR-5) + hook §2 no `composite`/`composite_region` + gate perf SOFT.

**Adianta sem me bloquear (teu bônus oferecido):** drafta as 24 sub-`*Params` + a
tabela PSD num doc de referência — eu transcrevo no enum/gate atômico em cima dele.

## §4 — Carry-overs W3 (anotados)
- Commit `be1d0f5` (W3 close) local → entra no ship da jornada (englobo no batch).
- Espelhar `MaskInvert`/`MaskApply` + os 4 CHROME_IDS Mask/Clip/Lock/Ref no array
  hand-maintained de `node_id_collisions.rs` — follow-up Coord aditivo, faço junto
  do landing de T4.1 (mesma passada de contrato).
═══════════════════════════════════════════════════════════════════
