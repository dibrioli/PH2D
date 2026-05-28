# ADR-0067 — `ph2d-brush-traits` decoupling crate (resolve circular dep Painter ↔ Vector)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0040 — Tool isolation](0040-tool-as-isolated-feature-crate.md), [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md), [ADR-0062 — Painter ↔ Vector bridge](0062-painter-vector-bridge.md).
**Spec normativa:** [`docs/Vector Module/README.md §7.1`](../../Vector%20Module/README.md) + [`docs/Vector Module/08_painter_bridge.md`](../../Vector%20Module/08_painter_bridge.md).
**Tags:** vector, painter, wave-0, contract, decoupling, foundational

---

## 1. Contexto

L6F2 Antigravity 3ª iter (HIGH severity) catch: Vector Module `vector-pattern-along-path` node (W4/W8) consome `ph2d-painter-brush` library; AND Painter "Vectorize Layer" command (W12) chama `vector-auto-trace` node. Risco de **circular crate dependency** que impede compilation.

Solução: crate foundational `ph2d-brush-traits` desacoplado.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-brush-traits`

```
crates/ph2d-brush-traits/
├── Cargo.toml                       (deps mínimas: serde, glam, ph2d-color)
├── src/
│   ├── lib.rs                       Re-exports + invariants
│   ├── brush_ref.rs                 BrushRef + BrushHandle types
│   ├── stamp_spec.rs                StampSpec interface
│   └── brush_engine.rs              trait BrushEngine
└── tests/
```

**Crítico**: zero deps em `ph2d-painter-*` OR `ph2d-vector-*`. Linear import position — ambos crates downstream importam linearly.

### 2.2 BrushRef + StampSpec (data types)

```rust
/// Opaque handle para brush. Painter implementations resolvem BrushRef
/// para internal Brush struct. Vector consumes BrushRef sem conhecer
/// internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrushRef(pub u64);                // blake3 hash do .ph2d-brush

#[derive(Debug, Clone, Copy)]
pub struct StampSpec {
    pub pos: glam::Vec2,
    pub tangent: glam::Vec2,
    pub pressure: f32,                       // [0..1]
    pub tilt: glam::Vec2,                    // [-π/2, π/2] per axis
    pub azimuth: f32,                        // [0, 2π)
    pub jitter: glam::Vec2,                  // random offset per-stamp
    pub size_scale: f32,                     // multiplier (typical 1.0)
}
```

### 2.3 BrushEngine trait (behavior interface)

```rust
pub trait BrushEngine {
    /// Stamp brush at sample into target texture.
    fn stamp(&self, target: &mut RenderTexture, sample: StampSpec);

    /// Reference handle do brush.
    fn brush_handle(&self) -> BrushRef;

    /// Optional metadata para UI display.
    fn display_name(&self) -> Cow<'_, str> { Cow::Borrowed("Unnamed Brush") }
}
```

`ph2d-painter-brush` **implementa** `BrushEngine` for `PainterBrush`. `ph2d-node-vector-pattern-along-path` **consume** trait sem conhecer internals.

### 2.4 Dependency graph cleaned

**Antes (circular risk)**:
```
ph2d-painter-brush ─────?────► ph2d-node-vector-auto-trace
       ▲                              │
       │                              ▼
       └──── ph2d-node-vector-pattern-along-path
```

**Depois (linear via brush-traits)**:
```
ph2d-brush-traits  ◄─────────  ph2d-painter-brush (impl BrushEngine)
       ▲
       │
       └──────  ph2d-node-vector-pattern-along-path (consume BrushEngine)
       └──────  ph2d-node-vector-auto-trace (output VectorNetwork; no Painter dep)
```

Arch-gate `no_circular_dep_painter_vector` (verifica via `cargo-deps` ou `cargo-modules` em CI).

### 2.5 Migration path para Painter existing

Painter ADR-0043/0044 já especificam `Brush` + `Stamp` structs internos. Adição zero-impact:
- `ph2d-painter-brush` adds `impl BrushEngine for PainterBrush { /* delegate to existing stamp pipeline */ }`.
- Existing Painter code unchanged.

### 2.6 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| `BrushEngine` trait methods | **3** (stamp + brush_handle + display_name) | Minimal surface |
| `StampSpec` fields | **7** | Cobre pressure/tilt/azimuth/jitter; expansão exige amendment |
| Crate deps | **3** (serde, glam, ph2d-color) | Foundational; zero domain deps |

---

## 3. Consequências

### 3.1 Positivas

- **Zero circular dep risk** Painter↔Vector (L6F2 catch certeiro absorbed).
- **Minimal surface** — 3 trait methods + 2 data types.
- **Migration path zero-impact** para Painter existing code (ADR-0044 não amenda).
- **Reusabilidade futura**: outros crates (3D modeling? sound brushes?) podem implementar `BrushEngine`.

### 3.2 Negativas

- **Crate novo** adds ~1 dep linha no Cargo workspace. Trivial.
- **Indirect dispatch** via trait object adds vtable lookup ~5 ns per stamp. Negligible vs stamp compute cost.

### 3.3 Neutras

- `BrushHandle` opaque type (vs full `Brush` struct) preserve encapsulation; consumers don't see internals.

---

## 4. Alternativas consideradas

### 4.1 Direct import Vector → Painter (rejeitada — L6F2 circular risk)

Vector node imports `ph2d-painter-brush` direct. **Por que rejeitada**: se Painter algum dia consume Vector (já é o caso com Vectorize Layer), circular dep impede compilation.

### 4.2 Concrete `PainterBrush` re-export (rejeitada — leak abstraction)

Re-export concrete type. **Por que rejeitada**: Vector consume Painter internals; tight coupling; difficult to evolve.

### 4.3 Generic over `BrushEngine` no node (rejeitada — runtime polymorphism necessário)

`vector-pattern-along-path<B: BrushEngine>`. **Por que rejeitada**: brush escolhido em runtime via UI dropdown; precisa `dyn BrushEngine`.

---

## 5. Implementação (Wave 1)

- **T1.1b** (W1 day 1): `ph2d-brush-traits` crate skeleton (foundational paralelo a `ph2d-vector-traits`).
- **T1.X** (W1 future): Painter `impl BrushEngine for PainterBrush`.
- **T4.5** (W4): `vector-pattern-along-path` consume `dyn BrushEngine`.

Gates: `no_circular_dep_painter_vector` + arch-gate ativo desde W1.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/README.md §7.1`](../../Vector%20Module/README.md) + [`docs/Vector Module/08_painter_bridge.md`](../../Vector%20Module/08_painter_bridge.md).
- ADR-0044 Brush Engine GPU (Painter base).
- ADR-0062 Painter ↔ Vector bridge (parent context).
- Antigravity L6F2 (3ª iter) absorbed.
