# ADR-0072 — Named Anchor unification (socket + slice + image_point num único tipo)

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0069 — decisão-mãe](0069-sprite-inspector-v2.md), [ADR-0022 — No HashMap in simulation](0022-no-hashmap-in-simulation.md), [ADR-0025 — GameObject model](0025-gameobject-model.md).
**Spec normativa:** [`docs/Sprite_projeto/07_named_anchors.md`](../../Sprite_projeto/07_named_anchors.md).
**Tags:** sprite, anchor, socket, slice, unification, llm-first

---

## 1. Contexto

Pesquisa multi-engine: três engines resolvem o **mesmo problema** com nomes diferentes:

| Engine | Conceito | Propriedades |
|---|---|---|
| **Unreal Paper 2D** | Socket | `name + transform 2D (pos+rot+scale)` |
| **Aseprite** | Slice | `name + bounds (rect) + center (9-slice opc) + pivot opc + data` |
| **Construct 3** | Image Point | `name + pos (vec2)` |

Casos de uso unificados:
- Anexar arma à mão do char (socket)
- Definir região de hitbox / face-visible (slice)
- Spawn point de projétil (image point / muzzle)
- 9-slice region nomeado dentro de um sprite (slice + center)
- Camera follow point

**Nenhum engine** unifica os 3 conceitos. Gaps reportados (Godot Proposal #14098, Unity Discussion "Sprite sockets requested feature" aberto há anos).

PH2D unifica num único `NamedAnchor` cobrindo os 3 casos.

---

## 2. Decisão

### 2.1 `NamedAnchor` schema canônico

**Tipos reusam infra ECS existente** (corrigido pós-audit — `Transform2D`/`Rect2` originalmente vapores):
- **`Transform`** = [`ph2d_ecs::Transform`](../../../crates/ph2d-ecs/src/transform.rs) (já é 2D: `Vec2 translation, f32 rotation, Vec2 scale`). Skew adicionado via [ADR-0025 amendment-1](0025-amendment-1.md).
- **`Rect2`** = `[f32; 4]` literal (consistente com `Sprite.region_rect`); ordem `[x, y, w, h]`.

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NamedAnchor {
    pub name: String,                    // snake_case, identificador único per-sprite
    pub transform: ph2d_ecs::Transform,  // pos + rot + scale (skew via amendment-1)
    pub bounds: Option<[f32; 4]>,        // se Some → slice (área retangular [x,y,w,h])
    pub center: Option<[f32; 4]>,        // se Some + bounds Some → 9-slice region interno
    #[serde(default)]
    pub user_data: AnchorData,           // payload livre (string/num/dict)
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum AnchorData {
    #[default]
    None,
    Str(String),
    Int(i64),
    Float(f64),
    Color([f32; 4]),
    /// Dict: SortedSmallVec newtype (lookup linear binary-search; cabe em 8 entries comum).
    /// `BTreeMap` rejeitado por hash determinism overhead em sim path;
    /// `HashMap` rejeitado por **violar ADR-0022 (no HashMap em SimWorld)**.
    /// **Key-sorted invariant ALWAYS via type-system** (Lens C H5 + Lens D D15):
    /// `SortedSmallVec` enforce by construction (API expose só `insert_sorted`/`get`/`iter`);
    /// sem `push`/`insert_idx` que violem invariant. Save/load roundtrip preserva ordem
    /// canônica `String::cmp` byte-wise (bit-identical cross-OS).
    Dict(SortedSmallVec),  // recursivo, depth ≤ 4
}

/// Newtype enforce key-sorted invariant by construction.
/// Sem API que permita inserir out-of-order.
#[derive(Clone, Debug, PartialEq)]
pub struct SortedSmallVec(SmallVec<[(String, AnchorData); 8]>);

impl SortedSmallVec {
    pub fn new() -> Self { Self(SmallVec::new()) }
    
    /// Insert ou substitui valor. Mantém invariant via binary-search.
    pub fn insert_sorted(&mut self, key: String, value: AnchorData) -> Option<AnchorData> {
        match self.0.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(idx) => Some(std::mem::replace(&mut self.0[idx].1, value)),
            Err(idx) => { self.0.insert(idx, (key, value)); None }
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&AnchorData> {
        self.0.binary_search_by(|(k, _)| k.as_str().cmp(key))
              .ok().map(|i| &self.0[i].1)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &(String, AnchorData)> { self.0.iter() }
    pub fn len(&self) -> usize { self.0.len() }
    
    // SEM: push, insert(idx), extend (qualquer API que permita out-of-order).
}

// serde via try_from valida invariant em deserialize:
impl TryFrom<SmallVec<[(String, AnchorData); 8]>> for SortedSmallVec {
    type Error = &'static str;
    fn try_from(v: SmallVec<[(String, AnchorData); 8]>) -> Result<Self, Self::Error> {
        if v.windows(2).all(|w| w[0].0 < w[1].0) { Ok(Self(v)) }
        else { Err("Dict keys must be lexicographically sorted") }
    }
}
```

**Razão `SortedSmallVec` newtype vs `SmallVec` cru (Lens D D15 + Lens E E10 fix):**
- `HashMap<K, V>` viola ADR-0022 — banido em SimWorld (`NamedAnchor` vai como Component ECS SimComponent).
- `BTreeMap<String, AnchorData>` é alternativa determinística, MAS recursão de Dict com BTreeMap inflate (estrutura interna B-tree); + alocação por insert.
- `SmallVec` cru expõe `.push()`/`.insert(idx)`/`.extend()` que **violam invariant trivialmente** — invariant declarado FROZEN sem type-system enforcement é mentira arquitetural (Lens D D15).
- **`SortedSmallVec` newtype** expõe apenas `insert_sorted`/`get`/`iter`/`len` — type system garante invariant; gate `anchor_dict_keys_sorted_invariant` torna-se redundante (compile-time enforce) mas mantido como smoke check no deserialize.
- Cabe inline (zero heap até 8 entries; caso comum ~3-4); **binary-search O(log N) lookup + O(N) shift on insert = O(N) amortized per insert** (Lens E E10 fix — afirmação anterior "O(log N)" era misleading; total fill O(N²)). Caso d'uso real (Dict ≤ 32 keys) torna constante prática negligente (~32×16 = 512 ops max).
- Para Dicts > 8 entries (raro), spill em heap mantém Vec linear (não BTree); manutenção fácil.

Semântica derivada do schema:

| Padrão | Significado | Caso de uso |
|---|---|---|
| `bounds=None, center=None` | **Socket** | Muzzle, hand bone, foot bone |
| `bounds=Some, center=None` | **Slice** | Hitbox, hurtbox, face zone |
| `bounds=Some, center=Some` | **9-slice region** | Dialog box, button bar |

### 2.2 Onde mora

```rust
// Component anexável ao Entity (per-sprite, anchors estáticos):
pub struct NamedAnchorList(pub SmallVec<[NamedAnchor; 4]>);  // inline 4 anchors

// Per-frame override em SpriteFrames asset:
pub struct SpriteFrame {
    pub texture_ref: TextureRef,
    pub duration_ms: u32,
    pub pivot_override: Option<[f32; 2]>,
    #[serde(default)]
    pub named_anchors: SmallVec<[NamedAnchor; 4]>,  // override do frame
}
```

Resolução em runtime:
1. SpriteAnimator presente AND frame atual tem override → usa override.
2. Senão → usa `NamedAnchorList` do entity.

### 2.3 Visual handles no canvas

Quando seção "Sockets / Slices" expandida no Inspector:
- **Socket** → cruz colorida (drag mover; alt+drag rotacionar; cmd+drag escalar).
- **Slice** → retângulo overlay (drag corners para resize; drag center para mover).
- **9-slice region** → retângulo externo (bounds) + interno (center), ambos editáveis.

Cor por anchor (hash do nome → cor estável). Label aparece próximo ao handle.

Per-frame mode (toggle "Drive by frame"): canvas mostra anchor do frame atual; scrubber mostra mudança.

### 2.4 Aseprite import lossless

```
Aseprite slice → NamedAnchor
  bounds (rect) → bounds
  center (rect, opcional) → center
  pivot (vec2, opcional) → transform.translation
  data (string) → user_data.Str
```

Lossless cross-tool. Aseprite slice per-frame → PH2D `SpriteFrame.named_anchors` per-frame.

### 2.5 Construct 3 + Paper 2D import lossless

```
Paper 2D Socket → NamedAnchor {bounds=None, center=None, transform=socket.local_transform}
Construct ImagePoint → NamedAnchor {bounds=None, center=None, transform.translation=image_point.pos}
```

### 2.6 Runtime API

```rust
// Rust:
let muzzle = entity.named_anchor("muzzle")?;
let world_pos = entity.transform.global() * muzzle.transform;

// Luau (HR-10):
local muzzle = ph2d.sprite.anchor(entity, "muzzle")
ph2d.spawn(bullet_prefab, muzzle.position, muzzle.rotation)

// MCP tools:
sprite_anchor_get(entity, name) → AnchorData
sprite_anchor_set(entity, name, anchor) → mutates
sprite_anchor_list(entity) → [Name]
```

HR-10 (cada tool exposto via lua_export). **HR-11 vide registry canon** em [README §7.1.2](../../Sprite_projeto/README.md): `sprite_anchor_get/list/set` mutative sem token; `sprite_anchor_set_or_replace`/`sprite_anchor_remove`/`sprite_clear_all_anchors` destructive **com token obrigatório** (Lens E E3 fix — afirmação anterior "HR-11 NÃO aplica" era parcial).

### 2.7 Caps congelados

Arch-gate `named_anchor_caps` em `crates/ph2d-render/tests/`:

| Cap | Valor | Razão |
|---|---|---|
| `NamedAnchorList` inline SmallVec | **4** | Shape comum (~3 anchors médio); >4 vai heap |
| `NamedAnchor.name` length | **≤ 64 chars** | Identificador, não payload |
| Total anchors por sprite | **≤ 64** | Sanity; >64 sugere refactor pra hierarquia ECS |
| `AnchorData.Dict` depth | **≤ 4** | Anti-recursion-DoS |
| `AnchorData.Dict` keys por nível | **≤ 32** | Anti-explosion |
| `AnchorData` variant count | **6 FROZEN** (None/Str/Int/Float/Color/Dict) — Lens E E23 | Bump exige ADR-0072-amendment |

Bump → ADR-0072-amendment.

### 2.8 Performance

- Lookup por nome: linear scan de SmallVec (≤4 anchors comum) → O(N) trivial.
- Heap fallback (≥4 anchors): ainda linear; otimização HashMap se gargalo (medir antes).
- Per-frame override: extra indirection via SpriteFrames lookup; ~1 cache miss adicional por anchor query — negligible.

Sem spatial index. Anchors são poucos (~3-10/sprite); queries raras (1× por spawn, 1× por hitcheck).

---

## 3. Consequências

### 3.1 Positivas

- **Unifica 3 conceitos paralelos** (socket Paper2D + slice Aseprite + image_point Construct) num único `NamedAnchor` — diferencial competitivo central.
- **Per-frame override** funciona automaticamente para anchors animados (Aseprite slice + Construct image_point per-frame).
- **Lossless import cross-tool** — Aseprite, Construct, Paper 2D todos preservam.
- **Visual handles in-canvas** — UX correta pra editar anchors (vs. só Inspector numeric).
- **LLM-first ready** — cada anchor exposto via `#[lua_export]` + MCP.
- **Hierarchical entity overhead evitado** — SmallVec inline cobre 99%; sem entity-per-socket explosion.

### 3.2 Negativas

- **`NamedAnchorList` Component opcional** adiciona ~50 bytes/sprite quando presente. Aceito (anchors são feature opt-in).
- **Per-frame override** dobra o lookup overhead em sprites animados com anchors. Negligible em workloads reais.
- **Visual handles in-canvas** demanda widget novo (`NamedAnchorEditor`). Custo W5+W6.

### 3.3 Neutras

- **Single source of truth** — anchor visível no Inspector + handle no canvas + API runtime. Sem duplicação.
- **Schema versionado** segue padrão Sprite v4 (HR-14 implícito quando NamedAnchorList ganhar `version`).

---

## 4. Alternativas consideradas

### 4.1 3 conceitos paralelos (NamedSocket + NamedSlice + ImagePoint) — rejeitada

Cada um Component próprio. **Por que rejeitada:** duplica registry; usuário tem que escolher tipo upfront; conversion entre tipos exige re-add. Schema unificado com Option<bounds>/Option<center> é mais flexível.

### 4.2 Anchors como entidades ECS filhas — rejeitada

Cada anchor vira Entity com Transform + Component Anchor. **Por que rejeitada:** entity overhead explosivo (50-100 anchors × 100 sprites = 5-10k entities só pra sockets). SmallVec inline cobre 99% sem entity churn.

### 4.3 Lookup via int ID em vez de nome string — rejeitada

`anchor_id: u32` em vez de `name: String`. **Por que rejeitada:** "anchor 5" ≠ "muzzle" semanticamente. Strings com snake_case são auto-documenting e Aseprite import preserva nome.

### 4.4 Per-frame override implícito (sem schema explícito) — rejeitada

Anchors mudam por frame mas via mecanismo invisível. **Por que rejeitada:** vira surpresa runtime. PH2D explicita via `SpriteFrame.named_anchors`.

### 4.5 user_data como `String` apenas (sem typing) — rejeitada

Single string payload. **Por que rejeitada:** Aseprite slice data permite estruturas (JSON-like); typed `AnchorData` enum é estritamente mais expressivo + LLM-friendly.

---

## 5. Implementação (Wave 5)

Tasks T-W.N vide [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.6`](../../Sprite_projeto/15_plano_de_implementacao.md).

W5 fecha quando:
- `NamedAnchor` struct + `AnchorData` enum compilados.
- Component `NamedAnchorList` anexável.
- `SpriteFrame.named_anchors` per-frame override funcional.
- Widget `NamedAnchorEditor` no Inspector seção 12.
- Visual handles no canvas (drag socket/slice).
- Aseprite import bridge lossless.
- API runtime `entity.named_anchor("muzzle")`.
- MCP tools `sprite_anchor_*`.
- Smoke do Enio (vide plano §15.6).

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| `name` ≤ 64 vs maior? | 64 cobre identificadores; payload vai em `user_data.Str`. Bump exige amendment. |
| `Dict` depth ≤ 4 vs maior? | Anti-recursion-DoS; 4 cobre estruturas reais (e.g., `{"damage":{"physical":10, "magic":5}}`). Bump exige amendment. |
| MCP `sprite_anchor_set` permite criar anchor novo? | **Sim** (mutative, não destructive — HR-11 não aplica). |
| Camera follow-point é caso especial? | **Não** — é só um Socket com nome "camera_target". Camera2D pode opt-in para follow esse anchor via Component. |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/07_named_anchors.md`](../../Sprite_projeto/07_named_anchors.md).
- ADR pai: [ADR-0069](0069-sprite-inspector-v2.md).
- Godot Proposal #14098 (sockets em AnimatedSprite2D): <https://github.com/godotengine/godot-proposals/issues/14098>.
- Unity Discussion "Sprite sockets requested feature": <https://discussions.unity.com/t/requested-feature-sprite-sockets/638175>.
- Unreal Paper 2D Sockets: <https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-sprite-sockets-in-unreal-engine>.
- Aseprite Slices: <https://www.aseprite.org/docs/slices/>.
- Construct 3 Sprite Plugin (Image Points): <https://www.construct.net/en/make-games/manuals/construct-3/plugin-reference/sprite>.
