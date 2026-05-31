# ADR-0074-amendment-1 — Clip & Mask = stencil (não back-buffer); `Mask2D` como Component opcional mínimo

**Status:** Accepted (W3 Phase 6, 2026-05-30) — ratificado pelo Enio (smoke ClipChildren + Mask Interaction OK).
**Amends:** [ADR-0074 — Sprite struct vs Component ECS](0074-sprite-component-boundary.md) §2.1 (regra dos 3 lugares).
**Spec sections superseded/clarified:** `docs/Sprite_projeto/06_mask_clip.md` §6.2 ("back-buffer based"), §6.6 (Mask2D futuro).
**Related:** [ADR-0070-amendment-7](0070-amendment-7.md) (ABI `clip_group`/`clip_meta`).
**Reference:** [`crates/ph2d-render/src/clip_pass.rs`](../../../crates/ph2d-render/src/clip_pass.rs), [`crates/ph2d-render/src/pipeline.rs`](../../../crates/ph2d-render/src/pipeline.rs), [`crates/ph2d-ecs/src/masking.rs`](../../../crates/ph2d-ecs/src/masking.rs), gates `clip_children_regression.rs` + `mask_interaction_regression.rs`.

---

## 1. Decisão A — técnica de render = **stencil**, não back-buffer

A spec §6.2 descreve ClipChildren como "back-buffer based" (1 render-target por clip-parent + pass de composição + sample da máscara). **Substituído por stencil por silhueta**, a técnica canônica:

- **Custo:** 1 attachment de stencil (`Stencil8`) compartilhado + sub-passes, vs 1 RT por clip-parent + composição. Stencil é mais barato e escala com o número de grupos sem alocar RTs.
- **Alocação lazy:** o attachment só existe em frames que têm clip/mask (`has_clip || has_mask`). Cena sem clip/mask = caminho single-pass byte-idêntico ao legado (zero-regressão).
- **3 (clip) + 1 (mask) pipelines** compartilham a mesma layout/shader: `mark` (Replace ref, discard por cutoff, color OFF), `test` (Equal ref), `test_outside` (NotEqual ref, pro Mask2D VisibleOutside). ClipAndDraw desenha a cor do mask pela `test` pipeline (Equal-ref coincide com a própria silhueta) → **3 pipelines, não 4** pro clip.
- **Refs incrementais por span** (sem clears inter-grupo) pro clip; **ref global = 1** pro mask (escopo global, §3). `Stencil8` não tem aspecto de depth → o render pass não precisa de depth ops.

Detalhe de transporte do cutoff (limite de 16 vertex attrs) está em [ADR-0070-amendment-7 §3](0070-amendment-7.md).

## 2. Decisão B — `Mask2D` é um **Component opcional** (regra dos 3 lugares, ADR-0074 §2.1)

MaskInteraction (no responder) já era Component (§02). Pra a feature ficar **demonstrável** faltava a FONTE. Pela árvore de decisão do ADR-0074 §2.1, "fonte de máscara" é **aspecto ortogonal, opcional, nem todo sprite carrega, ausência ≠ default explícito** → **Component anexável**, não campo do `Sprite` struct:

```rust
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mask2D { pub alpha_cutoff: f32 }   // DEFAULT_CUTOFF = 0.5
```

- O `Sprite` da entidade fornece a textura cuja silhueta (alpha > cutoff) é a máscara. O Mask2D **não desenha cor própria** (é molde, como o SpriteMask default da Unity).
- Este é o **Mask2D MÍNIMO**. O módulo completo (§6.6 — source `SpriteRef` vs `SupportedRenderer`, `sprite_sort_point`, Inspector próprio) segue futuro; ele NÃO cabe no Sprite Inspector (§6.6 razão de separação preservada). O componente mínimo só destranca o render + o toggle "Mask Source" na seção §8.
- Registrado no `ComponentRegistry` (contagem ecs 19→20, render/script 20→21).

## 3. Decisão C — escopo de máscara = **global** no W3

Todo responder `MaskInteraction` reage à UNIÃO de todas as `Mask2D` (um stencil ref compartilhado = 1). Sem fonte presente, o stencil fica 0 → Inside não desenha em lugar nenhum, Outside desenha por todo lado (o default da Unity). O scoping por intervalo de SortingLayer (`MaskCustomRange { front, back }`, spec §6.5) é **refinamento futuro opcional** — múltiplas máscaras sem brigar. Documentado como não-implementado.

## 4. Decisão D — gate de regressão é **obrigatório** (spec §6.3)

Godot teve 5 issues sucessivos de regressão de clip (#79885, #102190, #102224, #91068, #90793). PH2D **exige** testes pixel-exatos headless:

- `clip_children_regression`: 4 px canônicos × 3 modos (Disabled/ClipOnly/ClipAndDraw) + variação de cutoff.
- `mask_interaction_regression`: 3 px × {VisibleInside, VisibleOutside, source-only} — prova também que a fonte não pinta cor.

Ambos rodam num target offscreen `Rgba8Unorm` + readback; skip gracioso em CI sem adapter; rodam em Mac dev + Mac CI (onde o smoke do Enio também roda).

## 5. Consequences

- **Spec §6.2** "back-buffer based" vira contexto histórico; a implementação é stencil (Coord aplica a nota na spec).
- **Combinar clip + mask no mesmo sprite** é fora de escopo W3 (stencil reusado por pass; o extract dá precedência ao clip).
- **z-ordering:** clip groups e mask responders compõem POR CIMA do normal pass (limitação documentada do multi-pass); aceitável pros casos canônicos (avatar circular, HP-bar, fog-of-war, spotlight).
- **Forward path:** Mask2D completo + MaskCustomRange + clip+mask combinados reusam a mesma infra de stencil (`clip_pass.rs`).

## 6. Provenance

- W3 Phase 6, handoff `docs/HANDOFF_sprite_w3_phase6_clipchildren.md` (§1.1 mandava ratificar stencil via este amendment) + pedido do Enio pós-smoke pra tornar Mask Interaction demonstrável (Mask2D + render).
- Decisão de técnica (stencil vs back-buffer) e de 3-pipelines validada ao construir+validar via naga; gates pixel-exatos passando em GPU; smoke do Enio OK (ClipChildren + Mask).
