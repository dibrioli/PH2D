# ADR-0070-amendment-6 — `UvTransform` component + `RenderInstance.uv_xform: [f32;4]` GPU @location(15) (UV tiling/scroll)

**Status:** Accepted (W3 §9, 2026-05-30) — render shipado + smoke do Enio (TextureRepeat demonstrável via tiling/scroll).
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI; complementa [ADR-0070-amendment-5](0070-amendment-5.md) (repeat mode bits).
**Slot rationale:** próximo livre após `-5`.
**Spec sections clarified:** `docs/Sprite_projeto/09_sampling.md` §9.2 (tiling/scroll), §1.7 ABI.
**Reference:** [`crates/ph2d-render/src/sprite.rs`](../../../crates/ph2d-render/src/sprite.rs) (`uv_xform` + `IDENTITY_UV_XFORM`), [`crates/ph2d-render/src/shaders/sprite.wgsl`](../../../crates/ph2d-render/src/shaders/sprite.wgsl) (`wrap_uv` + fragment in-rect tiling), [`crates/ph2d-ecs/src/uv_transform.rs`] (`UvTransform`).

---

## 1. Context

§9.2 expõe **UV tiling** (escala > 1 ladrilha a textura) + **scroll** (offset, ex. background contínuo). Pra ficar visível, a transformação precisa chegar ao **fragment shader** (é UV math por-fragmento, não seleção de bind group). Diferente de `sampling` (amendment-5, CPU-only), `uv_xform` é **GPU-read** → vertex attribute.

## 2. Decision

1. **Component opcional** `UvTransform { scale: [f32; 2], offset: [f32; 2] }` (ECS). `scale [1,1]` + `offset [0,0]` = identidade (sem tiling/scroll). Ausência = identidade.
2. **`RenderInstance.uv_xform: [f32; 4]`** = `[scale.x, scale.y, offset.x, offset.y]`, exposto como **vertex attribute @location(15)** (Float32x4). Fica ANTES do tail CPU pra manter os attrs GPU contíguos.
3. **Fragment** (`sprite.wgsl`): samplea `wrap(quad_uv * scale + offset)` **DENTRO do sub-rect do próprio sprite** (entre `atlas_uv.xy` e `.zw`) — o tiling nunca vaza pro vizinho no atlas. O **wrap mode** vem dos bits 3-4 de `flip_uv` (repeat resolvido, amendment-5): `2 Enabled`→fract, `3 Mirror`→triangle wave, senão clamp. O scroll sempre wrap-a (`fract`) pra continuidade, mesmo em Clamp (caso background-scroll, §9.2).

- **ABI:** GPU 148→**164 B** (11→**12 attrs**), total 160→**176 B** (13→**14 campos**). O `vertex_attr_offsets_match_struct` re-locka 11→12 attrs (uv_xform @15). Demais attrs inalterados.

## 3. Why a GPU attr (vs CPU + per-draw uniform)

O tiling varia por-instância e é math de fragmento; um uniform por-sprite quebraria o batching (1 draw por sprite). Como vertex attr ele viaja com a instância no mesmo buffer, batch-stable. O custo (16 B/instância) é justificado: é GPU-read genuíno, ao contrário de `sampling` (CPU-only, amendment-5).

## 4. Consequences

- Identidade bit-a-bit: `scale [1,1] offset [0,0]` → `wrap_uv` + `select` no-op → sprites legados renderizam idênticos.
- O smoke-fix 2026-05-30 (`UV Offset wraps continuous scroll`) garantiu que scroll funciona até em Clamp (o `fract` do offset é incondicional quando offset≠0).
- Forward: rotação de UV / shear de UV estenderiam pra um `[f32;6]` (mat2x3) num amendment futuro, com re-lock de attrs.

## 5. Provenance

W3 §9 UV tiling/scroll, 2026-05-30 (commits `48e3ee8` render + `d615fc8` UI + `2fbc207` smoke-fix). Doc retroativa (Phase 8 ADR debt closure, 2026-05-31).
