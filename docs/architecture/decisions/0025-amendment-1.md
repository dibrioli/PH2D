# ADR-0025 amendment-1 — Transform 2D skew (skew_x, skew_y) — bump `Transform::VERSION` 1→2

**Status:** Accepted (2026-05-28) — ratificado pelo Enio junto com ADRs 0069..0074 pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Amenda:** [ADR-0025 GameObject model](0025-gameobject-model.md).
**Pré-requisitos:** [ADR-0069 — Sprite Inspector v2](0069-sprite-inspector-v2.md) (motiva a feature), [ADR-0021 — SimWorld/PresentWorld](0021-simulation-presentation-boundary.md).
**Precedente de pattern:** [ADR-0020 amendment-1](0020-amendment-1.md), [ADR-0040 amendment-1](0040-amendment-1.md), [ADR-0046 amendment-1](0046-amendment-1.md).
**Tags:** transform, ecs, foundational, hr-5, hr-14, skew

---

## 1. Contexto

[ADR-0069 Sprite Inspector v2](0069-sprite-inspector-v2.md) §2.1 Seção 2 inclui **Skew X / Skew Y** como propriedades editáveis do Transform. Pesquisa multi-engine: Godot Node2D tem `skew` nativo desde 2018 (diferenciador real — Unity NÃO tem skew nativo); LÖVE expõe `kx`/`ky` no draw call. Casos de uso: fake-3D barato (sombra de personagem deformada), wind sway em árvores 2D, "tilt" de cartão, banner dinâmico, cartoon "lean".

[ADR-0074](0074-sprite-component-boundary.md) §2.6 decidiu: **skew vai no Transform, não no Sprite** — skew é decomposição da matriz 2D (igual rotação/scale), não da imagem.

`Transform` atual ([crates/ph2d-ecs/src/transform.rs:65](../../../crates/ph2d-ecs/src/transform.rs#L65)) tem 3 campos:
- `translation: Vec2`
- `rotation: f32`
- `scale: Vec2`

`Transform::VERSION = 1` (const). Schema serde (postcard) tem layout estável; mudar exige migrator (HR-14) — Transform é Component foundational ECS usado em **todas as entidades com Transform** (sprites, vector paths, painter strokes, particles, audio sources com posição).

Adicionar `skew_x: f32, skew_y: f32` é **cascata foundational**:
1. **Bump `Transform::VERSION` 1 → 2** com migrator obrigatório (HR-14).
2. **`compose()` matemática muda** — skew altera shear; produto Transform com skew não é `(T, R, S)` simples; precisa decomposição SkewRS ou SRSkew documentada.
3. **`propagate_transforms` pass** lida com novo campo; hierarchy de skew cascateia? (Pai com skew + filho com scale — produto correto?).
4. **Memory layout muda** (Copy hoje = 20B; com skew = 28B); todo ECS store realoca; benchmark `transform_hot_path_no_alloc` precisa revalidar.
5. **Determinismo HR-5** — `f32::tan(skew)` ou `f32::sin_cos(skew)` introduz ponto onde precisão cross-platform precisa ser provada (já temos `Transform.rotation.sin_cos()` deterministic; skew adiciona).
6. **Sprite Inspector v2 W2.T2.2** depende — implementação só após esta amendment Accepted.

---

## 2. Decisão

### 2.1 Adicionar `skew_x: f32, skew_y: f32` ao `Transform`

```rust
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    /// Skew X em radianos. Shear ao longo do eixo X.
    /// Default 0.0 (sem skew). Range típico: [-π/4, +π/4] (-45°..+45°).
    #[serde(default)]
    pub skew_x: f32,
    /// Skew Y em radianos. Shear ao longo do eixo Y.
    /// Default 0.0 (sem skew). Range típico: [-π/4, +π/4].
    #[serde(default)]
    pub skew_y: f32,
}

impl Transform {
    pub const VERSION: u32 = 2;  // bump 1 → 2
    
    pub const IDENTITY: Self = Self {
        translation: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        skew_x: 0.0,
        skew_y: 0.0,
    };
}
```

### 2.2 `compose()` matemática canônica (T·R·Sk·S ordering)

Ordem de aplicação (LOCAL → WORLD via parent traversal):

```
M_local = Translate × Rotate × Skew × Scale × p
```

`Skew × p` =
```
| 1     tan(skew_x) | × | p.x |
| tan(skew_y)     1 |   | p.y |
```

`compose(parent, child)` atualizado (usa `libm` cross-OS — vide §2.4):

```rust
pub fn compose(parent: Self, child: Self) -> Self {
    // libm para bit-identical cross-OS (vide §2.4 fix Lens C C1).
    let (sin, cos) = libm::sincosf(parent.rotation);
    let tan_sx = libm::tanf(parent.skew_x);
    let tan_sy = libm::tanf(parent.skew_y);
    
    // Apply child translation in parent's space:
    // child_t' = parent.T + parent.R · parent.Sk · parent.S · child.t
    let s_tx = child.translation.x * parent.scale.x;
    let s_ty = child.translation.y * parent.scale.y;
    let sk_tx = s_tx + s_ty * tan_sx;       // skew_x mixes Y into X
    let sk_ty = s_tx * tan_sy + s_ty;       // skew_y mixes X into Y
    let rx = sk_tx * cos - sk_ty * sin;
    let ry = sk_tx * sin + sk_ty * cos;
    
    Self {
        translation: Vec2::new(parent.translation.x + rx, parent.translation.y + ry),
        rotation: parent.rotation + child.rotation,
        scale: Vec2::new(parent.scale.x * child.scale.x, parent.scale.y * child.scale.y),
        // ⚠️ Skew additive: APROXIMAÇÃO documentada — vide §2.2.1.
        skew_x: parent.skew_x + child.skew_x,
        skew_y: parent.skew_y + child.skew_y,
    }
}
```

**Razão T·R·Sk·S** (não T·R·S·Sk): skew aplicado **depois** de scale produz visual mais natural (cartas inclinadas mantêm proporção); skew **antes** distorce escala. Godot usa T·R·S·Sk; Three.js usa T·R·Sk·S. PH2D adopta T·R·Sk·S por consistência visual.

### 2.2.1 ⚠️ Limitação documentada: skew additive em cascade não-trivial (Lens C C5)

`compose(parent, child).skew_x = parent.skew_x + child.skew_x` é **APROXIMAÇÃO** matemática, NÃO exata.

**Erro:** o produto real de matrices `M_parent × M_child` (cada uma com T·R·Sk·S) NÃO decompõe trivialmente como `Translate(p+c) × Rotate(r_p+r_c) × Skew(sk_p+sk_c) × Scale(s_p*s_c)` quando rotation ≠ 0 ou scale ≠ (1,1) em pai ou filho. Rotação **não comuta** com skew; o produto tem termos cruzados que não cabem em (sk_x_new, sk_y_new) ortogonais.

**Prática:** o `compose` produz Transform cuja aplicação a um ponto NÃO é igual a `parent * (child * point)` em casos com skew + rotation/scale combinados.

**Decisão pós-audit:**

1. **Aceitar a limitação como visualmente plausível.** Spec documenta explicitamente; usuário com `parent.skew + parent.rotation` pode ver leve drift visual em filhos profundos. 99% dos casos reais (skew em sprite folha + rotation/scale em ancestrais) cabem na aproximação porque skew é **leaf-only** em prática (artist aplica skew na imagem final, não no rig do char).

2. **Constraint recomendada (não enforced):** "skew em entities folha; rotation/scale em ancestrais." Doc explica esta convenção em [`docs/Sprite_projeto/03_inspector_secoes.md §3.2`](../../Sprite_projeto/03_inspector_secoes.md) (UX hint no tooltip do skew slider).

3. **Rejeitado: matriz completa Mat3 affine.** Custo per-compose ~6-9 multiplicações extras em hot-path; perde semântica de animação por-componente (artist anima `rotation` discretamente, não matrix entries). Trade-off contra exatidão matemática.

4. **Rejeitado: arch-gate `transform_skew_only_in_leaf`.** Runtime cost + falsos positivos em scenes legítimas; convenção doc é suficiente para v2.

Se demanda emergir para skew exato em cascade (futuro), ADR-0025-amendment-2 pode introduzir `MatrixTransform` Component alternativo para entities que precisam exatidão.

### 2.3 Migrator v1 → v2 (HR-14) — wrapper enum PRIMARY

**Decisão pós-audit (Lens C C3):** postcard wire format declara explicitamente "Backwards/forwards compatibility between revisions of a postcard schema are considered outside of the scope of the postcard wire format" ([postcard.jamesmunns.com/wire-format](https://postcard.jamesmunns.com/wire-format)). `#[serde(default)]` em campo trailing **NÃO** é cobertura silenciosa em postcard — deserialize retorna `Err(DeserializeUnexpectedEnd)` ao tentar ler campo seguinte.

**Wrapper enum `TransformVersioned` é PRIMARY PATH** (mesma política de ADR-0070 §2.3 para Sprite):

```rust
#[derive(Serialize, Deserialize)]
pub enum TransformVersioned {
    V1(TransformV1),     // 3 campos legados
    V2(Transform),       // 5 campos v2
}

pub fn load_transform(bytes: &[u8]) -> Result<Transform, Error> {
    let versioned: TransformVersioned = postcard::from_bytes(bytes)?;
    match versioned {
        TransformVersioned::V1(v1) => Ok(migrate_v1_to_v2(v1)),
        TransformVersioned::V2(v2) => Ok(v2),
    }
}

pub fn save_transform(transform: &Transform) -> Vec<u8> {
    postcard::to_allocvec(&TransformVersioned::V2(*transform)).unwrap()
}

pub fn migrate_v1_to_v2(v1: TransformV1) -> Transform {
    Transform {
        translation: v1.translation,
        rotation: v1.rotation,
        scale: v1.scale,
        skew_x: 0.0,
        skew_y: 0.0,
    }
}
```

`#[serde(default)]` nos campos `skew_x`/`skew_y` é **defesa-em-profundidade** (fallback caso wrapper enum corrompa em corner case), não primary path. Empirical test validate em W0.T0.13.

### 2.4 Determinismo HR-5 — `libm` para transcendentals (corrigido pós-Lens-C)

**Lens C C1 critical fix:** `f32::tan`, `f32::sin_cos` **NÃO** são garantidos bit-identical cross-OS via IEEE 754. IEEE 754-2008 cobre operações básicas (`+ - × / abs copysign mul_add sqrt`); transcendentals (`sin/cos/tan/exp/log`) ficam **explicitamente fora**. A doc oficial [`std::primitive.f32`](https://doc.rust-lang.org/std/primitive.f32.html) afirma textualmente: "the precision of these functions varies by platform and Rust version, and can even differ within the same execution from one invocation to the next." Em Linux glibc → `tanf`; macOS Apple libm → `tanf`; Windows UCRT → `tanf` — três implementações distintas com ULP divergence documentada.

**Decisão pós-audit:**

1. **`Transform::compose()` v2 usa `libm` crate** ([rust-lang/libm](https://github.com/rust-lang/libm)) — pure Rust libm port com **bit-identical guarantee** cross-platform documentada. Substitui `f32::tan` por `libm::tanf` e `f32::sin_cos` por `libm::sincosf`. Adicionar `libm = "0.2"` em `crates/ph2d-ecs/Cargo.toml`.

2. **Aplicação imediata em `compose()` v1 atual** ([transform.rs:109](../../../crates/ph2d-ecs/src/transform.rs#L109)) — substituir `parent.rotation.sin_cos()` por `libm::sincosf(parent.rotation)`. O comentário em `transform.rs:29-32` ("bit-identical across Linux/Mac/Windows") era aspiracional e **continua aspiracional** com `f32::sin_cos`; com `libm::sincosf` vira **provado** (libm tem cross-OS test suite).

3. **Gate `transform_compose_with_skew_determinism`** atualizado:

```rust
#[test]
fn transform_compose_skew_byte_identical_cross_os() {
    // Fixture canônico: 3 cenários (rotation-only, scale-only, full TRS+skew).
    let cases = [
        Transform::compose(
            Transform { translation: Vec2::new(10.0, 20.0), rotation: 0.7853981, scale: Vec2::new(2.0, 1.5), skew_x: 0.1, skew_y: 0.0 },
            Transform { translation: Vec2::new(5.0, -3.0), rotation: 0.2, scale: Vec2::new(0.5, 0.5), skew_x: 0.0, skew_y: 0.05 },
        ),
        // ... 2 mais ...
    ];
    let bytes = postcard::to_allocvec(&cases).unwrap();
    let hash = blake3::hash(&bytes);
    // Hash canônico fica em fixtures/transform_skew_compose.expected
    // (single file, idêntico em todos OSes — gerado por libm em qualquer host).
    let expected = include_str!("fixtures/transform_skew_compose.expected").trim();
    assert_eq!(hash.to_hex().as_str(), expected,
        "Transform compose with skew is not cross-OS bit-identical. \
         Either libm migration is incomplete OR fixture is stale.");
}
```

**Fixture single source of truth:** `crates/ph2d-ecs/tests/fixtures/transform_skew_compose.expected` (single file, idêntico em todos OSes). PR W2.T2.2 que adiciona o test também adiciona o fixture (gerado em qualquer host porque libm é cross-platform); pre-merge run em CI matrix 3-OS valida (não single OS).

**CI matrix 3-OS** (Linux x86_64 com/sem AVX2 + macOS aarch64 + Windows x86_64) valida bit-identical. Gate falha se `libm` migration estiver incompleta OU se fixture estiver stale.

**Política long-term:** TODOS os transcendentals (`sin/cos/tan/exp/log/cbrt/...`) em código que escreve SimWorld (HR-5) DEVEM usar `libm::*` em vez de `f32::*`. Aplicar em PRs futuros (audit-sweep em W1 ou W2 separadamente).

**Custo aceito:** `libm` adiciona ~50KB ao binário; performance equivalente a `f32::*` em hardware moderno (compiler inline + LLVM optimizer). Trade-off documentado.

### 2.5 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| `Transform` fields | **5 (FROZEN v2)** | T·R·Sk·S × 2 axes |
| `Transform::VERSION` | **2** | Bump v1→v2 |
| `Transform` size_of | **28 bytes** | 8+4+8+4+4, 4-align |
| `skew_x/y` range | `[-π/2, +π/2]` (clamped no setter pra evitar tan() → ∞) | Sanity |
| **Fixtures v1 binárias** | 3 (translation-only, rotation-only, full TRS) | Gate `migrate_transform_v1_to_v2` |

Bump → ADR-0025-amendment-2.

### 2.6 Impactos cascade

- **`Transform::IDENTITY` const** atualizado (5 campos).
- **`from_translation(t)` const fn** atualizado (skew=0).
- **`Mul` impl** segue `compose` sem mudança.
- **`propagate_transforms` pass** em [transform.rs](../../../crates/ph2d-ecs/src/transform.rs) usa `compose` — automaticamente correto pós-update.
- **`GlobalTransform`** (PresentComponent) também precisa skew? **Sim**, pois deriva de Transform local cascade. Bump GlobalTransform field count → revisar separadamente em W2.T2.2.
- **`ph2d-vector`, `ph2d-painter-stroke`, `ph2d-render` extract phases** que decompõem Transform pra rotation: agora também precisam decompor skew se relevante. Maioria não — extract para `RenderInstance` usa rotation apenas. Skew aplica via vertex shader extra step (W2.T2.x decide).

---

## 3. Consequências

### 3.1 Positivas

- **Sprite Inspector v2 W2.T2.2 destrancado** com schema correto.
- **Diferencial sobre Unity** — skew nativo (sem shader manual).
- **Cascade hierárquico** — pai com skew propaga pros descendentes via `compose()`.
- **Determinístico cross-OS** — `tan()` IEEE 754 + sem FMA.
- **Memory overhead aceitável** — +8B/Transform (20→28). Cenas com 10k entities = +80KB; negligible.

### 3.2 Negativas

- **`compose()` matemática mais complexa** — 4 multiplicações + 2 tans a mais por composition. Hot path em `propagate_transforms`; bench `transform_compose_no_alloc` precisa validar.
- **Migrator obrigatório** — TODOS os save files Transform v1 precisam migrar; cooker bump.
- **GlobalTransform follow-up** — schema também muda; ADR separado em W2.
- **HR-14 ripples** — qualquer outro crate que serializa Transform diretamente (vector, painter, etc.) precisa recompilação + cooker test.

### 3.3 Neutras

- **Order T·R·Sk·S** documentada explicitamente; pre-W0 prevent debate.
- **skew_x/y default 0** — comportamento v1 preservado para 99% das entidades (que não usam skew).

---

## 4. Alternativas consideradas

### 4.1 Skew como Component opcional (`Skew { x, y }`) — rejeitada

Component separado anexável. **Por que rejeitada:** skew É decomposição da matriz 2D; coexiste obrigatoriamente com translation/rotation/scale em `compose()`. Component opcional força lookup `Option<Skew>` em CADA traversal de `propagate_transforms` — overhead em massa. Campo no struct é zero-overhead quando skew=0.

### 4.2 Skew como propriedade do Sprite struct — rejeitada

Vide ADR-0074 §2.6. Skew é matriz 2D, não imagem. Outras entidades sem Sprite (vector paths, painter strokes) também precisam skew.

### 4.3 Order T·R·S·Sk (skew depois de scale) — rejeitada parcialmente

Godot usa este ordering. **Por que rejeitada:** skew depois de scale produz aspect-ratio dependent (escalar non-uniformly + skew = distorção dobrada). T·R·Sk·S é estritamente mais previsível visualmente.

### 4.4 Matrix completa 2x3 affine em vez de TRS+Skew — rejeitada

Substituir TRS+Skew por `Mat2x3` direto. **Por que rejeitada:** perde semântica de animação (artist pensa em rotação, scale, skew separadamente); animation curves precisam atributos discretos para keyframe. TRS+Skew é estado canônico; matrix é derivada.

---

## 5. Implementação (Wave 2.T2.2)

Vide [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.3`](../../Sprite_projeto/15_plano_de_implementacao.md).

W2.T2.2 fecha quando:
- `Transform` v2 compila com 5 fields.
- 3 fixtures v1 binárias congeladas em `crates/ph2d-ecs/tests/fixtures/`.
- `migrate_transform_v1_to_v2` gate verde.
- `transform_compose_with_skew_determinism` cross-OS verde.
- `propagate_transforms` smoke verde (cascade hierarquia 3-níveis com skew).
- `cargo bench transform_compose` dentro do orçamento HR-4.
- Sprite Inspector seção Transform editável com Skew X/Y.

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| `tan(skew)` perto de ±π/2 → +∞? | Sim; clamp range `[-π/2 + ε, +π/2 - ε]` no setter. ε = 0.01 rad. |
| Animation timeline path para skew? | `transform.skew_x` / `transform.skew_y` — convenção dot, padrão Bevy Reflect. |
| `GlobalTransform` precisa de skew? | **Sim** — bump separado em W2.T2.2 (sub-task) ou subsequent amendment-2. |
| Aseprite import preserva skew? | Aseprite NÃO tem skew nativo; import = 0.0 sempre. Construct 3 tem; ignora se cross-importer. |

---

## 7. Referências

- ADR-0025 original: [GameObject model](0025-gameobject-model.md).
- Spec normativa Sprite Inspector v2: [`docs/Sprite_projeto/03_inspector_secoes.md §3.2`](../../Sprite_projeto/03_inspector_secoes.md).
- Godot Node2D skew: <https://docs.godotengine.org/en/stable/classes/class_node2d.html#property-skew>.
- LÖVE love.graphics.draw kx/ky: <https://love2d.org/wiki/love.graphics.draw>.
- Affine 2D math (T·R·Sk·S ordering): standard linear algebra; reference Three.js [`Object3D.applyMatrix4`](https://threejs.org/docs/#api/en/core/Object3D.applyMatrix4).
- Precedente ADR amendment pattern: [ADR-0020 amendment-1](0020-amendment-1.md), [ADR-0040 amendment-1](0040-amendment-1.md).
- **Lens C audit (Determinism + Multi-OS) fix sources:**
  - [Rust `std::primitive.f32` — transcendentals non-deterministic](https://doc.rust-lang.org/std/primitive.f32.html)
  - [RFC 3514 — Float Semantics](https://rust-lang.github.io/rfcs/3514-float-semantics.html)
  - [Postcard wire format — back-compat OUT-OF-SCOPE](https://postcard.jamesmunns.com/wire-format)
  - [rust-lang/libm — pure Rust libm with bit-identical guarantee](https://github.com/rust-lang/libm)
