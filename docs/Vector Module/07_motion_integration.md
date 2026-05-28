# 07 — Motion nodes integration (cross-domain `motion` ↔ `vector`)

> Spec da **integração cross-domain entre motion nodes e vector nodes**. PH2D já tem domain `motion` no `ph2d-nodegraph` (motion-grid / motion-clone / motion-transform). Vector Module adiciona domain `vector` (vide [02](02_geometry_graph.md)). Esta integração permite **motion nodes driving vector params** (animation procedural) e **vector paths como input de motion nodes** (Cavalry-style).
>
> **ADRs ratificadores:** ADR-0058 (Vector geometry graph) + ADR-0039 (Nodegraph contract freeze; já permite cross-domain).
> **Inovação relacionada**: [`14 §14.6`](14_inovacoes_extraordinarias.md) §8.5 (Vector Runtime) + motion-driven typography (§8.6).

## 7.1 Motion nodes driving vector params

### 7.1.1 Mecanismo

`ph2d-nodegraph` já suporta cross-domain connections (motion.wave outputs `f32`; vector-roughen.amplitude accepts `f32`). Vector Module valida + ratifica end-to-end.

### 7.1.2 Example: motion-wave → vector-roughen

```
motion-wave (frequency=2 Hz, amplitude=10)
   ↓ output: f32 oscillating [-10..10]
   → vector-roughen.amplitude
```

Result: vector path com roughen amplitude oscilando em 2 Hz. Visual feedback: borda do path "respira" sinusoidalmente.

### 7.1.3 Example: motion-grid → vector-scatter pattern

```
motion-grid (rows=5, cols=8, spacing=50)
   ↓ output: array of positions
   → vector-scatter.targets
```

Result: VectorNetwork duplicado em 40 instâncias em grid.

### 7.1.4 Type compatibility

| Motion output | Vector param input | Compatibility |
|---------------|-------------------|---------------|
| `f32` | Numeric param (amplitude, frequency, radius, ...) | ✓ direct |
| `Vec2` | Vec2 param (center, offset, ...) | ✓ direct |
| `Vec3` | Color (RGB)? | ⚠ conversion needed |
| `Array<Vec2>` | Targets param (scatter, etc.) | ✓ direct |
| `Affine` | Transform param | ✓ direct |
| `Color` | Color param | ✓ direct |

Connect with mismatch type → graph editor mostra error visual + tooltip "incompatible types".

### 7.1.5 Smoke W11

User adiciona motion-wave + connects para vector-roughen → vê path "respirar" no canvas em real-time.

---

## 7.2 Reverse: vector path como input de motion nodes

### 7.2.1 Mecanismo

VectorNetwork (output de vector-source, vector-pencil, etc.) pode ser input para motion-scatter-along-path, motion-clone-pathwise, etc.

### 7.2.2 Example: vector path → motion-scatter-along-path

```
vector-source (spiral, turns=8)
   ↓ output: VectorNetwork (spiral path)
   → motion-scatter-along-path.path
   + sprite_input (e.g., painter brush stamp)
   ↓ output: 50 sprites distributed along spiral
```

Result: sprites scattered along spiral path. Useful para particle-like effects, character trails, etc.

### 7.2.3 Path sampling API

Motion-scatter-along-path samples N points along VectorNetwork (vide [01 §1.7](01_data_model.md)):
- `VectorNetwork::sample_at(t)` → Vec2 (interp position at t in [0..1] along total length).
- `VectorNetwork::tangent_at(t)` → Vec2 (tangent direction).

Used by motion nodes to distribute sprites following path direction.

### 7.2.4 Edge cases

- Empty VectorNetwork: motion-scatter-along-path outputs zero sprites; no error.
- Self-intersecting path: sampling correct (uses total arc length along single traversal).
- Multi-region network: sample uses dominant region (largest area) OR concat all (configurable).

---

## 7.3 Cascading determinism

### 7.3.1 Princípio

Se motion graph é **determinístico** (SimWorld, ADR-0021), vector output is determinístico **automaticamente**.

### 7.3.2 Implementação

```rust
fn evaluate_cross_domain(motion_node: &MotionNode, vector_node: &VectorNode, ctx: &EvalCtx) -> Result<VectorNetwork> {
    let motion_output = motion_node.eval(ctx)?;
    if ctx.deterministic_required() && !motion_node.is_deterministic_capable() {
        return Err(Error::NonDeterministicInDeterministicGraph(motion_node.id()));
    }
    
    // Pass motion output as param to vector node
    let vector_output = vector_node.eval_with_input(motion_output, ctx)?;
    
    // Recursive — vector output must also be deterministic
    if ctx.deterministic_required() && !vector_node.is_deterministic_capable() {
        return Err(Error::NonDeterministicInDeterministicGraph(vector_node.id()));
    }
    
    Ok(vector_output)
}
```

### 7.3.3 Capabilities matrix

| Node | deterministic_capable |
|------|----------------------|
| motion-wave | ✓ (sinusoidal exact) |
| motion-grid | ✓ (positions exact) |
| motion-clone | ✓ |
| motion-transform | ✓ |
| motion-noise | ✓ se fixed seed |
| vector-source | ✓ |
| vector-boolean | ✓ se Linesweeper deterministic mode |
| vector-offset | ✓ |
| vector-outline-stroke | ✓ |
| vector-roughen | ✓ se fixed seed |
| vector-twist | ✓ |
| vector-bend-path | ✓ |
| vector-pattern-along-path | depends on brush (Painter brush deterministic by HR-5) |
| vector-scatter | ✓ se fixed seed |
| vector-width-profile | ✓ |
| vector-hatch | ✓ |
| vector-mirror | ✓ |
| vector-corner-round | ✓ |
| vector-warp | ✓ |
| vector-recolor | ✓ |
| vector-llm-shape | ❌ (LLM non-deterministic by default) |
| vector-luau-script | depends on script |

### 7.3.4 Gate CI

`tests/determinism/vector_cross_domain.rs` — fixture com motion → vector chain rodando em SimWorld + asserting bit-identical hash cross-OS.

### 7.3.5 LLM node exclusion

`vector-llm-shape` is excluded from deterministic graphs by default. UI warns user "LLM node prevents deterministic replay; consider baking result first (right-click → Bake)".

---

## 7.4 Visual feedback no graph editor

### 7.4.1 Cross-domain edge styling

Edges entre domains diferentes (motion → vector) com cor distinta no graph editor (e.g., motion = blue; vector = green; cross-domain = gradient blue→green).

### 7.4.2 Determinism indicator

Cada node mostra indicator (small icon) em corner:
- ✓ green: deterministic_capable + in deterministic chain.
- ⚠ yellow: non-deterministic mas em non-deterministic chain (OK).
- ❌ red: non-deterministic em deterministic chain (error; user must fix).

---

## 7.5 Performance

### 7.5.1 Cross-domain eval overhead

Negligible. Motion eval + vector eval rodam em standard `ph2d-nodegraph` pipeline; cross-domain é só passing values via connections.

### 7.5.2 Cache strategy

Same as single-domain (vide [02 §2.6](02_geometry_graph.md)). Hash inputs propaga cross-domain.

---

## 7.6 Future: shader nodes integration

### 7.6.1 Status atual

Shader nodes (Blender-style texture nodes for 2D fills) NÃO existe ainda no PH2D. Vector Module roadmap inclui `ph2d-vector-fill` (procedural fill shader graph, vide [05](05_procedural_fill.md)).

### 7.6.2 Future integration (V2.0+)

Quando shader nodes ecosystem amadurece em PH2D, motion nodes podem drive shader params (e.g., motion-wave → shader-noise.frequency em fill). Same pattern as motion → vector.

Vector Module **already prepared** via trait abstrações (`ph2d-vector-traits::ProceduralFillShader`, mock W1). Real implementation em W6+ desbloqueia integration.

---

## 7.7 Example workflows completos

### 7.7.1 "Logo with breathing motion"

```
vector-source (spiral, turns=5)
   → vector-roughen (amplitude DRIVEN BY motion-wave(2Hz, amp=2))
   → vector-recolor (harmony=Complementary)
   → fill = procedural-shader (noise + ramp, frequency DRIVEN BY motion-wave(0.5Hz))
```

Result: spiral with breathing edge + breathing fill texture. All animated procedurally.

### 7.7.2 "Particle trail along character path"

```
character_path = vector-pencil (user drew traced)
   → motion-scatter-along-path (count=50, path=character_path)
   → sprite_input = painter brush stamp
```

Result: 50 brush stamps distributed along character path. As character moves (path updates), stamps reposition.

### 7.7.3 "Letterform morph driven by mouse proximity"

```
text_path = vector-text-on-path ("HELLO")
   → variable_font.weight DRIVEN BY motion-radial-falloff(mouse_pos, radius=100px)
   → variable_font.slant DRIVEN BY motion-radial-falloff(mouse_pos, axis="slant", radius=100px)
```

Result: text gets bolder + slanted near mouse cursor.

---

## Fim do motion integration spec

Cross-domain `motion` ↔ `vector` permite animation procedural + path-as-input para motion nodes. Cascading determinism preserved. Visual feedback no graph editor. Performance negligible overhead.

**Next:** [`13_fora_de_escopo.md`](13_fora_de_escopo.md) (OUT-list explicit justifications).
