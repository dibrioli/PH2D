# 10 — Runtime + Gameplay (Vector como first-class asset de jogo)

> Spec do **runtime de jogo** do Vector Module. `ph2d-vector-runtime` crate ship-em-jogo (sucessor Rive-class). State machine. Bones + mesh deformation. ECS integration. Opt-in determinism. **Dynamic Rigid-Body Physics Vector Colliders** (Proposta 4 Antigravity — sword-cut → 2 corpos rígidos com momento preservado). **LOD vetorial dinâmico** (Proposta 2 Antigravity).
>
> **ADR ratificador:** ADR-0063 (Vector runtime + Dynamic Physics Colliders).
> **Sumário**: vector arte em gameplay como **first-class asset**, não decoration.

## 10.1 `ph2d-vector-runtime` crate API

### 10.1.1 Posicionamento arquitetural

```
crates/
├── ph2d-vector-doc/        (data model + edit log + CRDT)
├── ph2d-vector/            (Vello renderer wrapper)
├── ph2d-vector-fill/       (procedural fill shader graph)
├── ph2d-vector-runtime/    ← este spec (ship-em-jogo subset)
└── ph2d-tool-vector-*/     (editor tools — EXCLUÍDOS em release game build)
```

`ph2d-vector-runtime` é **subset shipable** em release game build. Importa:
- `ph2d-vector-doc` (read-only — load `.ph2d-vector` asset; sem edit log no runtime).
- `ph2d-vector` (Vello renderer).
- `ph2d-vector-fill` (procedural fills compile já cached em asset).
- `ph2d-physics` (Rapier 2D 0.28 — para physics colliders).

**Excluído em runtime**: editor tools, Studios, LLM bridges, Brush Studio (apenas asset consumption).

### 10.1.2 API surface

```rust
//! Runtime de jogo para Vector Module. Subset shipable, sem editor.

#![forbid(unsafe_code)]

pub struct VectorRuntime {
    asset: Ph2dVectorAsset,
    state_machine: Option<StateMachine>,
    skeleton: Option<Skeleton>,
    physics_handle: Option<PhysicsHandle>,
    lod_state: LodState,
    cached_render: CachedRender,
}

impl VectorRuntime {
    /// Load asset from binary (postcard).
    pub fn load(asset_bytes: &[u8]) -> Result<Self>;

    /// Tick per frame. Updates state machine + physics + LOD.
    pub fn tick(&mut self, dt: Duration, ctx: &TickContext);

    /// Render para scene Vello (chamado per frame from game render thread).
    pub fn render(&mut self, scene: &mut Scene, transform: Affine);

    /// State machine — chamado from ECS / Luau.
    pub fn set_state(&mut self, state_id: StateId);

    /// Trigger state transition.
    pub fn trigger(&mut self, trigger_id: TriggerId);

    /// Skeleton bone IK — chamado from ECS animation.
    pub fn set_bone_pose(&mut self, bone_id: BoneId, transform: Affine);

    /// Apply runtime boolean cut (gameplay sword-cut).
    /// Splits collider em N corpos rígidos com momento preservado.
    pub fn apply_cut(&mut self, cut_path: &VectorNetwork) -> Vec<RuntimePhysicsBody>;
}
```

### 10.1.3 Memory budget per platform (HR-13)

Vide [01 §1.13.3](01_data_model.md). Resumo:
- Desktop: 200 MB VRAM + 100 MB RAM.
- iPad Pro: 150 + 80 MB.
- Android top: 80 + 50 MB.
- Web: 40 + 20 MB.

### 10.1.4 Frame budget runtime

Sub-budget 3.5 ms (HR-4) shared com Vector Module overall budget. Specifically:
- Asset load (1×): ~50 ms cold start.
- Per-frame tick: ≤ 0.2 ms.
- Render: vide [03 §3.7](03_renderer.md).
- LOD adjust: ≤ 0.2 ms.

---

## 10.2 State machine model (Rive-inspired)

### 10.2.1 Conceito

Estado = preset de params (per node param do graph). Transitions com blend (linear / ease / spring). Triggers via ECS events.

```rust
pub struct StateMachine {
    states: HashMap<StateId, State>,
    transitions: Vec<Transition>,
    current_state: StateId,
    blend_progress: f32,  // 0..1 during transition
}

pub struct State {
    id: StateId,
    params: HashMap<NodePath, ParamValue>,  // overrides per-node-param
}

pub struct Transition {
    from: StateId,
    to: StateId,
    trigger: Option<TriggerId>,    // event-based
    condition: Option<Condition>,  // automatic
    blend_duration: Duration,
    blend_curve: BlendCurve,
}

pub enum BlendCurve {
    Linear,
    Ease,
    Spring { stiffness: f32, damping: f32 },
}
```

### 10.2.2 Triggers via ECS

ECS dispara `set_state` ou `trigger` quando game event acontece:

```rust
// Em ECS system (game code)
fn handle_hover(
    mut vector_runtime: Mut<VectorRuntime>,
    cursor: Res<CursorPos>,
    hit_test: Query<(&VectorAsset, &Transform)>,
) {
    if let Some(asset) = hit_test_cursor(&cursor, &hit_test) {
        vector_runtime.set_state(StateId::from("hover"));
    }
}
```

### 10.2.3 Blending

Durante transition, params interpolados via `BlendCurve`. Spring curve dá feel orgânico (bounce). Linear deterministic.

### 10.2.3-bis Reduced Motion runtime filter (Antigravity 3ª iteração L5F2 2026-05-29)

OS preference "Reduced Motion" (iOS Settings / macOS / Windows Ease of Access / Android System) sinaliza usuário com vestibular sensitivity OR cognitive load reduction need. WCAG 2.2 §2.3.3.

`VectorRuntime::tick` consulta `PlatformHost::reduced_motion_active()` per-frame. Quando true:

```rust
fn apply_state_transition(&mut self, transition: &Transition, dt: Duration) {
    if self.platform.reduced_motion_active() {
        // Snap immediate — no blending, no springs
        self.state = transition.to;
        self.blend_progress = 1.0;
        return;
    }
    
    // Normal smooth transition
    self.blend_progress += dt.as_secs_f64() / transition.blend_duration.as_secs_f64();
    if self.blend_progress >= 1.0 {
        self.state = transition.to;
        self.blend_progress = 1.0;
    }
    
    // Apply blend curve
    let t = transition.blend_curve.sample(self.blend_progress);
    self.interpolate_params(transition.from, transition.to, t);
}
```

**Aplica também a**:
- Animation curve playback: snap entre keyframes em vez de smooth interpolation.
- Bones IK: skip animated transitions, settle imediato.
- Variable font axes: snap em vez de smooth axis change.
- SDF Hybrid GPU morphing: skip animated boolean during draft preview.

Gate CI `vector_reduced_motion_compliance` valida com fixture animado + reduced_motion=true; result deve ser visually still (sub-frame discrepância ≤ 1 frame).

### 10.2.4 Multiplos state machines

Asset pode ter N state machines concorrentes (e.g., one para hover state, outro para gameplay damage state). Compostos via priority OR layering (foreground masks background).

---

## 10.3 Bones + vertex weighting (Rive-class)

### 10.3.1 Skeleton

```rust
pub struct Skeleton {
    bones: SmallVec<[Bone; 16]>,
    /// VertexId → bone influence mapping (per-vertex weighted).
    weights: HashMap<VertexId, BoneInfluence>,
}

pub struct Bone {
    id: BoneId,
    parent: Option<BoneId>,
    local_transform: Affine,  // rotation + translation + scale
    rest_pose: Affine,        // base position
}

pub struct BoneInfluence {
    bones: SmallVec<[(BoneId, f32); 4]>,  // up to 4 bones per vertex
}
```

### 10.3.2 Aplicação

```rust
fn apply_skeleton(network: &mut VectorNetwork, skeleton: &Skeleton) {
    for vertex in &mut network.vertices {
        let influence = &skeleton.weights[&vertex.id];
        let mut accum = Vec2::ZERO;
        let mut total_weight = 0.0;
        for &(bone_id, weight) in &influence.bones {
            let bone = &skeleton.bones[bone_id as usize];
            let transformed = bone.world_transform() * vertex.pos;
            accum += transformed * weight;
            total_weight += weight;
        }
        if total_weight > 0.0 {
            vertex.pos = accum / total_weight;
        }
    }
}
```

### 10.3.3 IK (inverse kinematics)

Bones support IK chain — game code sets end-effector position, IK solver computes intermediate bone transforms. Algoritmo: CCD (Cyclic Coordinate Descent) ou FABRIK.

### 10.3.4 Editor authoring

Skeleton authored em editor (W17+ stretch goal). Para v1.0, skeleton bake from external tool (Spine import? out v1.0).

---

## 10.4 Mesh deformation hybrid (path moves + raster UV warp)

### 10.4.1 Quando usar

- **Path moves** (skeleton-driven vertex updates) é canônico — escolha default.
- **Raster UV warp** (image meshes Rive-style) suportado para hybrid assets (vector + embedded image).

### 10.4.2 Image mesh

Quando asset contém embedded image (`embedded_assets[].kind == ImageMesh`), mesh tem UV coords associados aos vertices. Skeleton/state machine drive vertex positions; image samples per-pixel via UV → deformed image.

---

## 10.5 ECS integration

### 10.5.1 Components

```rust
#[derive(Component)]
pub struct VectorRuntimeComponent {
    pub asset: Handle<Ph2dVectorAsset>,
    pub current_state: StateId,
    pub physics_handle: Option<PhysicsHandle>,
}

#[derive(Component)]
pub struct VectorTransform(pub Affine);
```

### 10.5.2 Systems

```rust
fn vector_tick_system(
    time: Res<Time>,
    mut runtimes: Query<&mut VectorRuntimeComponent>,
) {
    for mut rt in &mut runtimes {
        rt.runtime.tick(time.delta(), &TickContext { /* ... */ });
    }
}

fn vector_render_system(
    runtimes: Query<(&VectorRuntimeComponent, &VectorTransform)>,
    mut scene: ResMut<VelloScene>,
) {
    for (rt, transform) in &runtimes {
        rt.runtime.render(&mut scene.0, transform.0);
    }
}
```

### 10.5.3 Bevy_ecs 0.18 compatibility

PH2D usa `bevy_ecs 0.18` standalone (vide SKILL_Stack §5). Vector runtime integra via standard `Component` derive.

---

## 10.6 Luau bridge

### 10.6.1 API exposta a Luau

```lua
-- Get current state
local s = ph2d.vector.state(handle)
-- => "idle" | "hover" | "press" | ...

-- Set state
ph2d.vector.set_state(handle, "hover")

-- Trigger transition
ph2d.vector.trigger(handle, "damage_taken")

-- Set bone pose
ph2d.vector.set_bone(handle, "head", { rotation = math.pi/8 })

-- Apply runtime cut (sword-cut, etc.)
local new_bodies = ph2d.vector.apply_cut(handle, cut_path_handle)
for _, body in ipairs(new_bodies) do
    -- body has rigid body handle for physics
    ph2d.physics.set_velocity(body, vec2(10, 5))
end
```

### 10.6.2 Determinismo

Quando runtime is deterministic (HR-5), Luau calls go via fixed-point + ordered reductions. Coroutines determinísticas via PH2D scheduler.

---

## 10.7 Determinismo opt-in (SimWorld vs PresentWorld)

### 10.7.1 Decisão por-asset

`.ph2d-vector` asset declara `deterministic: bool` em metadata. Asset com `deterministic=true`:
- Vive em `SimWorld` (ADR-0021).
- Boolean ops em fixed-point Q16.16.
- Linesweeper deterministic mode.
- SDF Hybrid com fixed resolution + ordered reductions.
- Replay determinístico cross-platform.

Asset com `deterministic=false` (default):
- Vive em `PresentWorld`.
- FMA + ordering relaxado.
- Mais performant.

### 10.7.2 Multiplayer rollback netcode

Quando rollback netcode (GGPO-style) ativo, vector runtime precisa estar em SimWorld + deterministic. Test: `tests/determinism/vector_rollback_replay.rs`.

---

## 10.8 LOD vetorial dinâmico (Proposta 2 Antigravity)

### 10.8.1 Necessidade

Cena com **50+ vector elements** simultaneously → frame budget 3.5 ms quebra se cada element renderiza full detail.

### 10.8.2 Algoritmo

Curve-aware adaptive fit pré-Vello sparse-strips:

```rust
fn compute_lod(network: &VectorNetwork, camera: &Camera, asset_pos: Affine) -> LodLevel {
    let bbox_screen = asset_bbox_in_screen_pixels(network, asset_pos, camera);
    let coverage = bbox_screen.area();
    
    if coverage < 100.0 {
        LodLevel::VeryLow  // skip rendering (off-screen-ish)
    } else if coverage < 1000.0 {
        LodLevel::Low      // RDP threshold 2.0 px
    } else if coverage < 10_000.0 {
        LodLevel::Medium   // RDP threshold 0.5 px
    } else {
        LodLevel::High     // full detail
    }
}

fn apply_lod(network: &mut VectorNetwork, lod: LodLevel) {
    let threshold = match lod {
        LodLevel::VeryLow | LodLevel::Low => 2.0,
        LodLevel::Medium => 0.5,
        LodLevel::High => 0.0,  // no simplify
    };
    if threshold > 0.0 {
        // 1) Levien flatten to polyline
        let polyline = network.flatten(threshold);
        // 2) RDP simplify
        let simplified_poly = douglas_peucker(&polyline, threshold);
        // 3) Re-fit to fewer cubics (Levien Béz fitting)
        *network = fit_cubics_to_polyline(&simplified_poly);
    }
}
```

### 10.8.3 Per-asset override

Heroi sempre full detail (`lod_override: LodLevel::High`). Props distantes simplificam.

### 10.8.4 Performance

- LOD compute: ≤ 50 µs / asset.
- Simplification: ≤ 200 µs / 100-segment path.
- Mantém frame budget 3.5 ms com 50+ assets em tela.

### 10.8.5 Visual quality threshold

LOD aggressive em entry devices → user pode notar low-detail em zoom in. Acceptable trade-off.

---

## 10.9 Physics collider integration (Proposta 4 Antigravity)

### 10.9.1 Collider generation

VectorNetwork → Rapier 2D collider:

```rust
fn vector_to_rapier_collider(network: &VectorNetwork) -> Vec<rapier2d::Collider> {
    let mut colliders = Vec::new();
    for region in &network.regions {
        let polygon: Vec<rapier2d::Point2> = region_outline_polygon(network, region);
        
        if is_convex(&polygon) {
            // Direct convex hull
            colliders.push(rapier2d::ColliderBuilder::convex_polyline(polygon).build());
        } else {
            // Earcut decomp into convex pieces
            let convex_pieces = earcut_decompose(&polygon);
            for piece in convex_pieces {
                colliders.push(rapier2d::ColliderBuilder::convex_polyline(piece).build());
            }
        }
    }
    colliders
}
```

### 10.9.2 Mass derivation

```rust
fn compute_region_mass(region: &Region, network: &VectorNetwork, material: &MaterialDef) -> f32 {
    let area = compute_region_area(region, network);  // signed area via Green's theorem
    area * material.density  // ρ × A
}
```

### 10.9.3 Joints

Regions adjacentes podem ter joints (cloth-like, breakable):

```rust
pub enum RegionJoint {
    Rigid,  // welded
    Hinge { pivot: Vec2 },
    Cloth { stiffness: f32, breaking_force: f32 },
}
```

### 10.9.4 Dynamic split em runtime boolean cut — 3-tier pipeline (revisado Antigravity L1F5 2ª iteração 2026-05-28)

**A inovação chave da Proposta 4 Antigravity, com fix descompasso temporal L1F5.**

**Pipeline em 3 tiers** (cada tier reduz latência aceitando trade-off):

```rust
impl VectorRuntime {
    pub fn apply_cut(&mut self, cut_path: &VectorNetwork) -> Vec<RuntimePhysicsBody> {
        // === TIER 0 (preferred): Dormant Fracture Edges (§8.8) ===
        // Se asset tem DormantFractureSet pré-computado e impact casa,
        // activate instantâneo — ZERO custo (sub-µs).
        if let Some(dormant_set) = &self.asset.dormant_fractures {
            if let Some(fracture_edge) = dormant_set.find_nearest(cut_path.midpoint()) {
                return self.activate_dormant_fracture(fracture_edge);
            }
        }

        // === TIER 1 (fallback): CPU fast-slice + async exact reconcile ===
        // SDF GPU silhueta imediata (preview ≤ 0.5 ms)
        let silhouette = self.sdf_boolean_subtract(&self.asset.network, cut_path);
        self.cached_render.update_silhouette(silhouette);

        // CPU fast-slice de convex hulls em sub-ms (NEW Antigravity L1F5)
        // Aproxima collider split em CPU usando convex pieces decomposition
        // — não exato como Linesweeper, mas dispatch IMEDIATO no physics tick.
        let fast_split_bodies = self.cpu_fast_slice_convex_hulls(cut_path);
        let original_velocity = self.physics_handle.linear_velocity();
        let original_angular = self.physics_handle.angular_velocity();
        let original_mass = self.physics_handle.mass();

        let mut new_bodies = Vec::new();
        for fast_body in fast_split_bodies {
            let mass_ratio = fast_body.approx_mass / original_mass;
            new_bodies.push(RuntimePhysicsBody::new(
                fast_body.approx_colliders,  // convex pieces approx
                original_velocity * mass_ratio,
                original_angular,
                fast_body.approx_mass,
            ));
        }
        self.physics_handle.destroy();

        // === TIER 2 (async refinement): Linesweeper exact topology ===
        // Spawn background worker para topology exata; quando ready,
        // substitui colliders aproximados pelos exatos (visual + physics smooth).
        self.spawn_linesweeper_refinement(cut_path, |exact_networks| {
            self.replace_colliders_with_exact(exact_networks);
        });

        new_bodies
    }
}
```

**Trade-off**:
- **Tier 0** (dormant): zero latência, **artista controls fracture pattern** em editor. Best for "designed destruction" (e.g., breakaway prefabs).
- **Tier 1** (CPU fast-slice): sub-ms latência, **physics responsive immediately**, convex approximation pode parecer ligeiramente diferente da silhueta SDF visual mas indistinguível para gameplay feel.
- **Tier 2** (async exact): refinement happen "behind the scenes"; user pode ou não perceber transition de approx → exact (geralmente não — diff é sub-pixel).

**Algoritmo CPU fast-slice**:
1. Convex decomposition do collider original (cached em asset cook time).
2. Para cada convex piece, fast 2D plane clip (Sutherland-Hodgman) com cut_path average direction.
3. Resultantes pieces são novos convex sub-bodies.
4. Custo: O(N convex pieces × M average clip edges) ≈ < 0.5 ms para 100 pieces.

**Fallback dynamic concave (Antigravity 3ª iteração L1F3 2026-05-29)**: Se asset é runtime-generated (procedural via Luau OR boolean ops) e NÃO tem convex decomposition cached em cook-time, Tier 1 não pode aplicar Sutherland-Hodgman direto sobre N pieces. Pipeline alternativo sub-ms:

```rust
if asset.cached_convex_decomp.is_none() {
    // Dynamic concave fallback — single convex hull approximation
    let outer_hull = quickhull_2d(&asset.network.all_vertices());  // ~0.3 ms
    let approx_collider = rapier2d::ColliderBuilder::convex_polyline(outer_hull).build();
    let body = single_rigid_body_from_hull(approx_collider, mass, velocity);

    // Spawn async exact decomposition para próximo cut OR refinement
    self.spawn_async_convex_decomp(&asset.network, |decomp| {
        asset.cached_convex_decomp = Some(decomp);  // future cuts use cached
    });

    return vec![body];  // single fast body, refinement async
}
```

Trade-off: dynamic concave shape recebe approximation por outer convex hull no primeiro cut (slight overestimate of collision area até refinement). Aceito para gameplay responsiveness; runtime ganha exact decomposition em background para cuts subsequentes. Editor pode mark assets sem cached decomp com warning "Dynamic concave: first cut approximated".

**Gate CI**: `vector_runtime_cut_latency` valida tier 0/1 ≤ 1 ms sub-cut visual-to-physics sync. Memory `feedback_pipeline_inject_dont_cap`: pipeline injeta no buffer correto, não cap o resultado final.

### 10.9.5 Smoke W16 (gameplay diferencial)

Espada vetor corta tábua vetor → vê:
1. Espada animation toca tábua (collision detected via Rapier).
2. SDF silhueta atualiza imediato (sword cut visual feedback).
3. Linesweeper async completa em ~5 ms; topology exata.
4. Tábua collider split em 2 sub-collider; cada um recebe parte do momento.
5. 2 metades caem com física correta (gravity + cada uma com sua velocidade angular).

### 10.9.6 Edge cases

- Cut produz N corpos com N=1: nenhum split, cut foi superficial. Collider modify mas não split.
- Cut produz fragmentos minúsculos (área < threshold): merge em vizinho OR descartar (puff effect).
- Cut com `deterministic=true`: Linesweeper deterministic + ordered split → bit-identical em CI cross-OS.

---

## 10.10 Memory budget per platform — Tier System (revisado Antigravity L5F4 2ª iteração 2026-05-28)

**Crítica L5F4**: 80MB VRAM mobile inicial era altíssimo vs Rive < 10MB. Solução: **Tier System** espelhando Painter DeviceTier (ADR-0053 Painter precedent).

### Tier matrix per device

| Tier | Devices alvo | VRAM | RAM | Features ativas | Tradeoffs |
|------|--------------|------|-----|-----------------|-----------|
| **Heavy** | Desktop Mac M-series, Win RTX 30+, Linux RDNA2+ | 200 MB | 100 MB | Tudo (SDF Hybrid full, procedural shaders complex, diffusion curves full-res) | Quality máxima |
| **Standard** | iPad Pro M-series, iPad standard | 120 MB | 60 MB | SDF Hybrid + procedural shaders + diffusion curves 0.5× res | Quality alta |
| **Lite** | Android top-tier (Adreno 660+/Mali-G78+), iPad std | 60 MB | 30 MB | SDF Hybrid limitado a 20 paths concurrent, procedural fills limited, diffusion curves 0.25× res | Quality decente |
| **Mobile Core** | Android entry-tier, devices entry-level | **12 MB** | **8 MB** | SDF only (no boolean preview), no procedural fills (solid + gradient only), no diffusion curves, fluid sim off, LOD aggressive | Quality compete com Rive (<10MB target alcançado) |
| **Web** | WebGPU Chrome/Safari/Firefox | 40 MB | 20 MB | Standard subset minus offline cache | Quality alta com web budget |

### Mobile Core specifics (rival do Rive ~10MB)

- **Variant `ph2d-vector-runtime` com feature flag `mobile-core`**: tree-shakes caches robustos, shader graph compiler, fluid sim, diffusion curves; deixa apenas (state machine + bones + LOD + simple SDF rendering + solid/gradient fills).
- **Asset cooked for Mobile Core**: editor build-time pré-renderiza diffusion curves e procedural shaders para texture atlas estático; asset carrega 5-10 MB ao invés de 50-100 MB.
- **Sub-budget per asset Mobile Core**: ~1-2 MB. Cena com 5 assets = ~5-10 MB total. ✓ Rival do Rive.

### Mobile Core graceful fallback dynamic assets (Antigravity 3ª iteração L1F6 + L8F2 2026-05-29)

**Risco**: Luau gameplay script gera VectorNetwork em runtime com diffusion curve fill, mas device tier é Mobile Core (sem solver Poisson). **Sem graceful handling = crash imediato** ("missing diffusion shader function").

**Pipeline canônico**:

```rust
fn load_or_validate_asset(asset: &Ph2dVectorAsset, tier: DeviceTier) -> Result<()> {
    // Editor-side validator (cook time) — L8F2 absorbed
    if tier == DeviceTier::MobileCore {
        if asset.has_dynamic_diffusion_curves() && !asset.has_pre_rendered_atlas() {
            return Err(Error::TierViolation {
                tier,
                msg: "Asset with dynamic diffusion curves must include pre-rendered atlas for Mobile Core. Re-cook with --tier=mobile-core."
            });
        }
    }

    // Runtime-side graceful fallback (L1F6 absorbed)
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

**Editor build-time validator** (L8F2 — `ph2d-asset-cooker` extension):
- Asset com `target_tier = MobileCore` flag forçada → cooker valida e refuses build se features inviáveis sem pre-render.
- CI gate `vector_mobile_core_asset_compat` valida 100% assets em pasta `tests/fixtures/mobile-core/`.

**Runtime fallback** (L1F6) — graceful degradation, não crash. Visual quality lower mas asset roda.

### Detection automática

`PlatformHost::device_tier()` (extension de ADR-0050 Painter Device heterogeneity layer) retorna `DeviceTier::{Heavy,Standard,Lite,MobileCore,Web}` em runtime. Asset metadata pode `min_tier` override (e.g., heroi requer Standard+).

Sub-budget per asset (média):
- Heavy/Standard: ~5-15 MB.
- Lite: ~3-8 MB.
- Mobile Core: ~1-2 MB.

Cena worst-case Heavy com 10 assets = ~150 MB. Cabe.

---

## 10.11 Frame budget runtime

### 10.11.1 Per-asset breakdown

- LOD compute: 0.01 ms / asset.
- Skeleton apply: 0.05 ms / asset.
- State machine tick: 0.02 ms / asset.
- Render (via Vello): vide [03 §3.7](03_renderer.md).

### 10.11.2 Scene-level worst case

Cena com 50 assets active:
- LOD: 50 × 0.01 = 0.5 ms.
- Skeleton: 50 × 0.05 = 2.5 ms (se todos têm skeleton).
- State machine: 50 × 0.02 = 1.0 ms.
- Render aggregado: dirty-rect propagation, only changed re-render.

Mitigação: skeleton em parallel rayon. State machine simples.

---

## 10.12 Asset loading

### 10.12.1 Postcard parsing

```rust
pub fn load_vector_asset(bytes: &[u8]) -> Result<Ph2dVectorAsset> {
    let asset: Ph2dVectorAsset = postcard::from_bytes(bytes)?;
    
    // Migrator chain HR-14
    if asset.version != CURRENT_VERSION {
        migrate(&mut asset)?;
    }
    
    // Validate
    asset.network.validate()?;
    
    Ok(asset)
}
```

### 10.12.2 Caching

- Load 1× per asset; cache em memory.
- Hot reload via blake3 hash detection (HR-6) — file watcher detecta change, re-load atomically.
- `EmbeddedAsset` (images, fonts) extraídos para texture cache do Vello.

### 10.12.3 Streaming

Assets grandes (> 5 MB) marcados `streamable=true`; carregados em chunks. v1.0 não-prioritário (most vector assets são pequenos).

---

## 10.13 Hot reload

### 10.13.1 Workflow editor → runtime

Editor (`ph2d-editor-core`) salva `.ph2d-vector` → `ph2d-asset` watcher detecta change → blake3 hash novo → notifica subscribers (jogo em modo dev) → `VectorRuntime::reload(new_asset_bytes)`.

### 10.13.2 State preservation

Hot reload preserva state machine current state se compatible (state id match between old/new asset). Caso incompatible, reset to default state.

### 10.13.3 Determinismo durante reload

Em deterministic mode, hot reload é apenas em dev mode (não em multiplayer game). Reload causa rollback marker no replay log.

---

## Fim do runtime spec

Runtime ship-em-jogo com state machine + bones + ECS + Luau + LOD + **Physics colliders dinâmicos**. Diferencial competitivo brutal vs Rive / Lottie / Spine.

**Next:** [`11_pencil_pipeline.md`](11_pencil_pipeline.md) (multi-platform input) + [`12_ux_chrome.md`](12_ux_chrome.md) (UI/UX layout).
