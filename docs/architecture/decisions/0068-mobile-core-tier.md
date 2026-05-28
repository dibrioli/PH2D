# ADR-0068 — DeviceTier Mobile Core (Vector Module <12 MB rival Rive)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0053 — Cross-platform tier policy](0053-cross-platform-tier.md) (Painter precedente), [ADR-0063 — Runtime + Physics](0063-vector-runtime-physics-dormant-fractures.md), [HR-13 — Memory budget](../../SKILL_Stack_PH2D_Definitiva.md).
**Spec normativa:** [`docs/Vector Module/10_runtime_gameplay.md §10.10`](../../Vector%20Module/10_runtime_gameplay.md).
**Tags:** vector, wave-0, contract, mobile, tier, performance

---

## 1. Contexto

L5F4 Antigravity 2ª iter: 80MB VRAM mobile initial estava muito alto vs Rive ~10MB. Solução: **Tier System** com tier **Mobile Core <12MB** competitivo. Padrão espelhando Painter ADR-0053.

L1F6 Antigravity 3ª iter (graceful fallback) + L8F2 (build-time validator) integram nesta ADR.

---

## 2. Decisão

### 2.1 Tier matrix per device

| Tier | Devices alvo | VRAM | RAM | Features ativas | Trade-off |
|------|--------------|------|-----|-----------------|-----------|
| **Heavy** | Desktop Mac M-series, Win RTX 30+, Linux RDNA2+ | 200 MB | 100 MB | Tudo (SDF Hybrid full, procedural shaders complex, diffusion curves full-res) | Quality máxima |
| **Standard** | iPad Pro M-series | 120 MB | 60 MB | SDF Hybrid + procedural shaders + diffusion curves 0.5× res | Quality alta |
| **Lite** | Android top-tier (Adreno 660+/Mali-G78+), iPad std | 60 MB | 30 MB | SDF Hybrid limitado (≤ 20 paths concurrent), procedural fills limited, diffusion curves 0.25× res | Quality decente |
| **Mobile Core** ✨ | Android entry-tier, devices entry-level | **12 MB** | **8 MB** | SDF only (no boolean preview), no procedural fills (solid + gradient only), no diffusion curves, fluid sim off, LOD aggressive | Quality compete com Rive |
| **Web** | WebGPU Chrome 121+/Safari 18+/Firefox 141+ | 40 MB | 20 MB | Standard subset minus offline cache | Quality alta com web budget |

### 2.2 Detection automática

`PlatformHost::device_tier()` (extension de ADR-0050 Painter Device heterogeneity layer) retorna `DeviceTier::{Heavy, Standard, Lite, MobileCore, Web}`. Assets podem `min_tier` override em metadata (e.g., heroi requer Standard+).

### 2.3 Mobile Core variant `ph2d-vector-runtime` (feature flag `mobile-core`)

Tree-shake aggressive:
- **Excluído**: shader graph compiler (`ph2d-vector-fill`), fluid sim, diffusion curve solver (Poisson PDE), Linesweeper exato (apenas Linesweeper-naïve clipper para commits raros).
- **Preservado**: state machine, bones + skeleton, LOD, simple SDF rendering (Vello GPU OR Vello CPU SIMD fallback), solid/gradient fills.

```toml
# crates/ph2d-vector-runtime/Cargo.toml
[features]
default = []
mobile-core = []  # tree-shake aggressive
```

### 2.4 Asset cooking para Mobile Core (L8F2 build-time validator)

Editor compiler (`ph2d-asset-cooker vector` sub-command):
- Asset com `target_tier = MobileCore` flag → pré-render diffusion curves + procedural shaders para texture atlas estático.
- Stub fluid sim params (replaced com solid fill).
- Asset carrega 5-10 MB em vez de 50-100 MB.

**Validator estático** (L8F2): se asset declara Mobile Core target mas usa unavailable features sem pre-render atlas:

```rust
fn validate_for_tier(asset: &Ph2dVectorAsset, target_tier: DeviceTier) -> Result<(), TierViolation> {
    if target_tier == DeviceTier::MobileCore {
        if asset.has_dynamic_diffusion_curves() && !asset.has_pre_rendered_atlas() {
            return Err(TierViolation {
                tier: target_tier,
                msg: "Asset uses dynamic diffusion curves but no pre-rendered atlas. Re-cook with --tier=mobile-core.",
            });
        }
        if asset.has_complex_procedural_shaders() && !asset.has_pre_rendered_atlas() {
            return Err(TierViolation { /* ... */ });
        }
        if asset.uses_fluid_sim() {
            return Err(TierViolation {
                tier: target_tier,
                msg: "Fluid sim not supported in Mobile Core tier.",
            });
        }
    }
    Ok(())
}
```

Build-time CI gate `vector_mobile_core_asset_compat` valida 100% fixtures em `tests/fixtures/mobile-core/`.

### 2.5 Runtime graceful fallback (L1F6 Antigravity 3ª iter)

Quando asset com unavailable features carrega em Mobile Core device em runtime (e.g., asset dinâmico Luau-gen):

```rust
fn load_or_validate_asset(asset: &Ph2dVectorAsset, tier: DeviceTier) -> Result<()> {
    if tier == DeviceTier::MobileCore && asset.needs_unavailable_features() {
        // Don't crash — substitute degraded but functional render
        asset.degrade_fills_to_solid_avg();
        log::warn!("Asset {} degraded to solid fills (Mobile Core tier)", asset.id);
    }
    Ok(())
}

impl Ph2dVectorAsset {
    fn degrade_fills_to_solid_avg(&mut self) {
        for region in &mut self.network.regions {
            if let Some(FillStyle::MeshGradient(_)) | Some(FillStyle::DiffusionCurve(_)) | Some(FillStyle::ProceduralShader(_)) = region.fill {
                let avg_color = sample_avg_color_from_curves(region, &self.styles);
                region.fill = Some(FillStyle::Solid(avg_color));
            }
        }
    }
}
```

UI toast notification "Mobile Core tier — quality reduced".

### 2.6 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Tiers total | **5 FROZEN** (Heavy / Standard / Lite / MobileCore / Web) | Cap; expansão exige amendment |
| Mobile Core VRAM | **12 MB** | Rival Rive (~10 MB target) |
| Mobile Core RAM | **8 MB** | Aggressive |
| Mobile Core SDF resolution multiplier | **1× canvas DPI** | Quality vs cost minimum |
| Mobile Core asset sub-budget typical | **1-2 MB** | Cena com 5 assets = 5-10 MB total |

---

## 3. Consequências

### 3.1 Positivas

- **Rival Rive directly** em mobile entry — Vector Module competitivo em footprint.
- **Heavy tier preserva quality máxima** sem compromise para Desktop / iPad Pro M-series.
- **Build-time validator + Runtime graceful fallback** mitigam tier mismatch crashes.
- **Tier system extensível** — adicionar futuros tiers (e.g., "VR HMD") via amendment.

### 3.2 Negativas

- **Editor cooker complexity** — pré-renderiza atlas para Mobile Core target tier; adds cook time.
- **5 tiers** = matriz de testing complexity. CI runs `vector_mobile_core_asset_compat` mais outros tier gates.
- **Mobile Core graceful degrade** quality lower than competitor; trade-off para footprint.

### 3.3 Neutras

- Tier detection via `PlatformHost` é runtime-only; build-time defaults to Heavy.

---

## 4. Alternativas consideradas

### 4.1 Single tier (rejeitada — 80MB unacceptable Rive comparison)

Sem tier system. **Por que rejeitada**: L5F4 catch certeiro — mobile entry users won't accept 80MB para feature parity Rive ~10MB.

### 4.2 Strict crash on tier mismatch (rejeitada — L1F6)

Validate apenas em build-time; runtime crash if unavailable feature. **Por que rejeitada**: Luau scripts em gameplay podem gerar assets dinâmicos; graceful degrade preserve UX.

### 4.3 Spec-time only validator (sem runtime graceful) (rejeitada — incomplete)

Build-time only. **Por que rejeitada**: dynamic assets via Luau bypassam build-time check; runtime graceful necessário.

### 4.4 Mobile Core tier disabled by default (rejeitada — must support)

Apenas Heavy + Standard. **Por que rejeitada**: target mercado mobile entry expressivo; Vector Module precisa rival Rive.

---

## 5. Implementação (Wave 10 + Wave 16)

- **T10.X** (W10): `PlatformHost::device_tier()` extension (cross-ref ADR-0050 amendment).
- **T16.5** (W16): LOD aggressive integration por tier.
- **T16.6** (W16): Mobile Core variant runtime build (`feature mobile-core`).
- **Editor cooker** (Wave 18 export interop): build-time validator + atlas pre-render.

Gates: `vector_mobile_core_asset_compat` (build-time) + `vector_mobile_core_runtime_smoke` (12MB budget verify) + `vector_tier_graceful_fallback` (no crash on mismatch).

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/10_runtime_gameplay.md §10.10`](../../Vector%20Module/10_runtime_gameplay.md).
- Painter ADR-0053 Cross-platform tier policy (pattern paralelo).
- Rive performance benchmark: <https://rive.app/blog/rive-renderer-now-open-source-and-available-on-all-platforms>
- Antigravity L5F4 (2ª iter) + L1F6 + L8F2 (3ª iter) absorbed.
