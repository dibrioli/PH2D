# ADR-0063 — Vector runtime + Dynamic Physics Colliders + Dormant Fractures + LOD

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0021 — SimWorld/PresentWorld](0021-simulation-presentation-boundary.md), [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0059 — Renderer pipeline](0059-vector-renderer-pipeline.md), [ADR-0050 — Device heterogeneity](0050-device-heterogeneity-layer.md) (Painter precedente), [ADR-0068 — Mobile Core tier](0068-mobile-core-tier.md) (irmã).
**Spec normativa:** [`docs/Vector Module/10_runtime_gameplay.md`](../../Vector%20Module/10_runtime_gameplay.md).
**Tags:** vector, runtime, physics, rapier, wave-0, contract, gameplay

---

## 1. Contexto

Inovação #5 + #8: **Vector Runtime determinístico + Dynamic Physics Colliders + Dormant Fracture Edges**. Sucessor Rive-class superior (state machine + bones + mesh deform) com adições únicas: Rapier 2D collider gen automático + dynamic split em runtime boolean cut + pre-computed fracture lines no editor.

Antigravity 1ª iter (Proposta 4 physics colliders) + 2ª iter (L1F5 collider temporal gap) + 3ª iter (L1F3 dynamic concave fallback + L7F1 LOD aggressive) absorvidos.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-vector-runtime`

```
crates/ph2d-vector-runtime/
├── Cargo.toml                       (no `editor` feature; tree-shake editor code)
├── src/
│   ├── lib.rs                       VectorRuntime API
│   ├── state_machine.rs             States + Transitions + Blending (Rive-style)
│   ├── skeleton.rs                  Bones + vertex weighting (Rive-class)
│   ├── mesh_deform.rs               Hybrid path moves + raster UV warp
│   ├── ecs_bridge.rs                bevy_ecs 0.18 integration
│   ├── luau_bridge.rs               ph2d.vector.* MCP API
│   ├── lod.rs                       Dynamic LOD curve-aware adaptive fit
│   ├── physics.rs                   Rapier 2D collider gen + dynamic split
│   ├── dormant_fractures.rs         Pre-computed fracture pipeline (Inovação #8)
│   └── reduced_motion.rs            OS preference snap (L5F2 Antigravity 3ª iter)
└── tests/
    ├── determinism_cross_os.rs
    ├── physics_momentum_conservation.rs
    └── dormant_fracture_smoke.rs
```

### 2.2 State machine model (Rive-inspired)

```rust
pub struct StateMachine {
    states: HashMap<StateId, State>,        // (BTreeMap em determinism mode)
    transitions: Vec<Transition>,
    current_state: StateId,
    blend_progress: f64,                    // f64 per L1F1 Antigravity 3ª iter
}

pub struct Transition {
    from: StateId,
    to: StateId,
    trigger: Option<TriggerId>,
    blend_duration: Duration,
    blend_curve: BlendCurve,                 // Linear | Ease | Spring
}
```

Reduced Motion runtime filter (L5F2 Antigravity 3ª iter): snap immediate quando `PlatformHost::reduced_motion_active() == true`.

### 2.3 Bones + vertex weighting (Rive-class)

```rust
pub struct Skeleton {
    bones: SmallVec<[Bone; 16]>,
    weights: BTreeMap<VertexId, BoneInfluence>,  // up to 4 bones per vertex
}
```

IK via CCD (Cyclic Coordinate Descent) ou FABRIK. Editor authoring W17+ stretch.

### 2.4 Rapier 2D physics collider integration (Proposta 4 Antigravity 1ª iter)

```rust
fn vector_to_rapier_collider(network: &VectorNetwork) -> Vec<rapier2d::Collider> {
    let mut colliders = Vec::new();
    for region in &network.regions {
        let polygon = region_outline_polygon(network, region);
        if is_convex(&polygon) {
            colliders.push(rapier2d::ColliderBuilder::convex_polyline(polygon).build());
        } else {
            // Earcut decomp em sub-pieces convexas
            let convex_pieces = earcut_decompose(&polygon);
            for piece in convex_pieces {
                colliders.push(rapier2d::ColliderBuilder::convex_polyline(piece).build());
            }
        }
    }
    colliders
}

fn compute_region_mass(region: &Region, network: &VectorNetwork, material: &MaterialDef) -> f32 {
    let area = compute_region_area(region, network);  // signed area via Green's theorem
    area * material.density
}
```

Joints opcionais entre regions (Rigid | Hinge | Cloth { breaking_force }).

### 2.5 Dynamic split em runtime boolean cut — 3-tier pipeline (L1F5 + L1F3 Antigravity 2ª+3ª iter)

```rust
impl VectorRuntime {
    pub fn apply_cut(&mut self, cut_path: &VectorNetwork) -> Vec<RuntimePhysicsBody> {
        // === TIER 0 (preferred): Dormant Fracture Edges ===
        if let Some(dormant_set) = &self.asset.dormant_fractures {
            if let Some(fracture_edge) = dormant_set.find_nearest(cut_path.midpoint()) {
                return self.activate_dormant_fracture(fracture_edge);  // sub-µs
            }
        }

        // === TIER 1 (fallback): CPU fast-slice + async exact reconcile ===
        let silhouette = self.sdf_boolean_subtract(&self.asset.network, cut_path);  // ≤ 0.5 ms GPU
        self.cached_render.update_silhouette(silhouette);

        // Dynamic concave fallback (L1F3 Antigravity 3ª iter):
        if self.asset.cached_convex_decomp.is_none() {
            let outer_hull = quickhull_2d(&self.asset.network.all_vertices());  // ~0.3 ms
            let approx_collider = rapier2d::ColliderBuilder::convex_polyline(outer_hull).build();
            let body = single_rigid_body_from_hull(approx_collider, mass, velocity);
            self.spawn_async_convex_decomp(&self.asset.network);  // refinement async
            return vec![body];
        }

        // Normal Tier 1: Sutherland-Hodgman per convex piece
        let fast_split_bodies = self.cpu_fast_slice_convex_hulls(cut_path);  // ~0.5 ms
        let original = self.physics_handle;
        let mut new_bodies = Vec::new();
        for fast_body in fast_split_bodies {
            let mass_ratio = fast_body.approx_mass / original.mass();
            new_bodies.push(RuntimePhysicsBody::new(
                fast_body.approx_colliders,
                original.linear_velocity() * mass_ratio,
                original.angular_velocity(),
                fast_body.approx_mass,
            ));
        }
        self.physics_handle.destroy();

        // === TIER 2 (async refinement): Linesweeper exact topology ===
        self.spawn_linesweeper_refinement(cut_path, |exact_networks| {
            self.replace_colliders_with_exact(exact_networks);
        });

        new_bodies
    }
}
```

### 2.6 Inovação #8 — Dormant Fracture Edges (L7F1 Antigravity 3ª iter)

Editor pre-computa fracture lines (Voronoi sample OR artist-painted breakaway paths) e salva como `DormantFractureSet` no `.ph2d-vector` asset:

```rust
pub struct DormantFractureSet {
    pub fracture_edges: Vec<VectorNetwork>,         // pre-decomposed paths
    pub fracture_regions: Vec<RegionId>,            // affected regions
    pub sub_bodies: Vec<RuntimePhysicsBody>,        // pre-built Rapier bodies (cooked)
    pub spatial_index: AabbTree,                    // O(log N) nearest lookup
}
```

Em runtime cut: O(log N) nearest dormant edge → activate atomicamente → momentum applied. **ZERO custo CPU/GPU no tick de colisão.**

Authoring: `vector-voronoi-fracture` node (ADR-0058) gera N fracture variants pre-cooked. Artist paints breakaway paths manualmente.

### 2.7 LOD vetorial dinâmico (Proposta 2 Antigravity 1ª iter)

Bézier-aware adaptive fit pré-Vello sparse-strips:

```rust
fn compute_lod(network: &VectorNetwork, camera: &Camera, asset_pos: Affine) -> LodLevel {
    let bbox_screen = asset_bbox_in_screen_pixels(network, asset_pos, camera);
    let coverage = bbox_screen.area();
    if coverage < 100.0 { LodLevel::VeryLow }     // skip rendering off-screen
    else if coverage < 1000.0 { LodLevel::Low }   // RDP threshold 2.0 px
    else if coverage < 10_000.0 { LodLevel::Medium }  // RDP threshold 0.5 px
    else { LodLevel::High }                       // full detail
}
```

Per-asset override (`lod_override: LodLevel::High` para heroi). Mantém frame budget 3.5 ms mesmo com 50+ assets em tela.

Gate `vector_runtime_lod_frame_budget`.

### 2.8 Reduced Motion runtime filter (L5F2 Antigravity 3ª iter)

`VectorRuntime::tick` consulta `PlatformHost::reduced_motion_active()` per-frame. Quando true:
- State transitions snap immediate (no blending).
- Animation curves skip interpolation.
- Bones IK settle imediato.
- Variable font axes snap.
- SDF morphing skip animated.

Gate `vector_reduced_motion_compliance` (WCAG 2.2 §2.3.3).

### 2.9 Determinismo opt-in (HR-5 + ADR-0021)

`.ph2d-vector` asset com `deterministic: true`:
- Boolean ops em fixed-point Q16.16.
- Linesweeper deterministic via app-layer (ADR-0059 §2.5).
- Ordered reductions.
- State machine transition fixed-step.

Test `tests/determinism/vector_rollback_replay.rs`.

### 2.10 Memory budget per tier (cross-ref ADR-0068)

| Tier | VRAM Vector Runtime | RAM Vector Runtime |
|------|---------------------|--------------------|
| Heavy | 100 MB | 50 MB |
| Standard | 80 MB | 40 MB |
| Lite | 50 MB | 30 MB |
| Mobile Core | **<12 MB** | **<8 MB** |
| Web | 30 MB | 20 MB |

### 2.11 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Skeleton max bones | **64** | Rive paridade |
| Bones per vertex | **4** | Rive paridade |
| State machine states max | **256** | Rive paridade |
| Transitions max | **1024** | Rive paridade |
| LOD levels | **4** (VeryLow / Low / Medium / High) | Balance simplicity vs granularity |
| Runtime cut tier latency targets | T0 sub-µs / T1 < 1 ms / T2 1-50 ms async | Gameplay responsiveness |
| Dormant fracture max edges per asset | **64** | Memory bound |

---

## 3. Consequências

### 3.1 Positivas

- **Vector arte em gameplay como first-class asset**, não decoration.
- **Dormant Fractures + Dynamic Split** ZERO custo em hot-path collision tick — diferencial brutal vs Rive.
- **3-tier pipeline collider** resolve L1F5 temporal gap (sub-ms physics + async exact refinement).
- **Dynamic concave fallback** evita crash em runtime-generated assets.
- **LOD adaptive** mantém 50+ assets cena performante em mobile.
- **Reduced Motion** garante WCAG 2.2 §2.3.3 compliance.

### 3.2 Negativas

- **3-tier pipeline** complexidade implementação (Dormant + CPU fast-slice + Linesweeper async). Tier 0 mitigates muito.
- **Editor cooker** precisa gerar `DormantFractureSet` (Voronoi OR artist-painted) — adds asset cook time.
- **Rapier dep** já no stack PH2D (M10) mas ABI binding com VectorNetwork novo.

### 3.3 Neutras

- Memory overhead Dormant Fractures ~2-5 MB per asset typical.

---

## 4. Alternativas consideradas

### 4.1 Sem physics integration (rejeitada — diminished ambition)

Vector como decoration only. **Por que rejeitada**: Rive prova mercado para runtime; sword-cut shape morph é gameplay diferencial unique.

### 4.2 Apenas Linesweeper async (rejeitada — temporal gap)

Sem CPU fast-slice + sem Dormant Fractures. **Por que rejeitada**: 50ms gap visual → physics = unacceptable. Vide L1F5 Antigravity 2ª iter.

### 4.3 Sem Dormant Fractures (rejeitada — gameplay-defining)

Sem pre-cooked fracture. **Por que rejeitada**: L7F1 brilliant catch Antigravity 3ª iter — Voronoi pré-compute em editor + activate sub-µs em runtime = ZERO custo. Inovação #8 ativada.

### 4.4 Skeleton authoring no editor v1.0 (rejeitada — stretch)

Full skeleton editor W17+. **Por que adiada**: bake from Spine import OR placeholder simple para v1.0.

---

## 5. Implementação (Wave 16)

- **T16.1**: `ph2d-vector-runtime` skeleton + asset loading.
- **T16.2**: State machine + ECS bridge.
- **T16.3**: Rapier collider gen.
- **T16.4**: Dynamic split em runtime boolean cut (3-tier pipeline).
- **T16.5**: LOD vetorial dinâmico.
- **T16.6**: Dormant Fractures activation pipeline.
- **T16.7**: Reduced Motion filter.

Gates: `vector_runtime_physics_momentum_conservation` + `vector_runtime_lod_frame_budget` + `vector_runtime_cut_latency` + `vector_reduced_motion_compliance` + `vector_dormant_fracture_smoke`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/10_runtime_gameplay.md`](../../Vector%20Module/10_runtime_gameplay.md) (714 linhas).
- Rive runtimes: <https://rive.app/runtimes>
- Rive Bones docs: <https://help.rive.app/editor/manipulating-shapes/bones>
- Rapier 2D 0.28: <https://crates.io/crates/rapier2d>
- Painter ADR-0050 device heterogeneity (pattern paralelo).
- Antigravity Proposta 4 (1ª iter) + L1F3/L1F5 (2ª iter) + L7F1 (3ª iter) absorvidos.
