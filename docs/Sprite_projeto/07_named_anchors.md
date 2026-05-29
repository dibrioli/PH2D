# 07 — Named Anchors — sistema unificado (socket + slice + image_point)

## 7.1 O problema unificado

Três engines resolvem o mesmo problema com nomes diferentes:

| Engine | Conceito | Propriedades |
|---|---|---|
| **Unreal Paper 2D** | Socket | `name + transform 2D (pos + rot + scale)` |
| **Aseprite** | Slice | `name + bounds (rect) + center (9-slice region opc) + pivot opc + data` |
| **Construct 3** | Image Point | `name + pos (vec2)` |

Todos resolvem casos como:
- **Anexar arma à mão do char** (socket)
- **Definir região de hitbox / face-visible** (slice)
- **Spawn point de projétil** (image point / muzzle)
- **9-slice region nomeado dentro de um sprite** (slice + center)
- **Camera follow point**

PH2D unifica num único `NamedAnchor` cobrindo os 3 casos.

## 7.2 `NamedAnchor` schema

**Tipos reusam infra ECS existente** (corrigido pós-audit — `Transform2D`/`Rect2` originalmente vapores de tipo):
- **Transform** = [`ph2d_ecs::Transform`](../../crates/ph2d-ecs/src/transform.rs) — já é 2D (Vec2 translation, f32 rotation, Vec2 scale, + skew_x/skew_y via [ADR-0025 amendment-1](../architecture/decisions/0025-amendment-1.md)).
- **Rect2** = `[f32; 4]` literal — ordem `[x, y, w, h]` em pixels intrínsecos. Consistente com `Sprite.region_rect`.

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NamedAnchor {
    /// Identificador único dentro do sprite. Convenção: snake_case.
    /// Exemplos: "muzzle", "hand_right", "face_box", "dialog_bg".
    pub name: String,
    
    /// Transform 2D no espaço LOCAL do sprite (pixels intrínsecos da imagem).
    /// Convenção: ORIGEM no anchor do sprite (`Sprite.anchor`); positivo = direita+baixo.
    pub transform: ph2d_ecs::Transform,
    
    /// Se `Some`, este anchor é uma SLICE (área retangular nomeada).
    /// Bounds em pixels intrínsecos da imagem. Ordem `[x, y, w, h]`.
    pub bounds: Option<[f32; 4]>,
    
    /// Se `Some` (e `bounds` for `Some`), este anchor é uma 9-SLICE REGION
    /// (rect interno ao bounds que define o "centro" do 9-slice).
    pub center: Option<[f32; 4]>,
    
    /// Payload livre (string/number/dict via Variant).
    /// Útil pra: damage values per-hitbox, tags ("hurt_box"/"hit_box"),
    /// metadata user-defined (e.g., "fires_when_pressed":true).
    #[serde(default)]
    pub user_data: AnchorData,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum AnchorData {
    #[default]
    None,
    Str(String),
    Int(i64),
    Float(f64),
    Color([f32; 4]),
    /// Dict: `SortedSmallVec` newtype (Lens D D15) — type-system enforce
    /// key-sorted invariant. API: `insert_sorted` / `get` / `iter` / `len`;
    /// SEM `push`/`insert_idx` que permitam out-of-order.
    /// **`HashMap` rejeitado** — viola [ADR-0022 no-HashMap-in-simulation](../architecture/decisions/0022-no-hashmap-in-simulation.md).
    /// `String::cmp` byte-wise → bit-identical cross-OS.
    /// Depth ≤ 4 (anti-recursion-DoS). Detalhes em [ADR-0072 §2.1](../architecture/decisions/0072-named-anchor-unification.md).
    Dict(SortedSmallVec),
}
```

**Semântica derivada do schema:**

| Padrão | Significado | Caso de uso |
|---|---|---|
| `bounds=None`, `center=None` | **Socket** (ponto de attach) | Muzzle, hand bone, foot bone |
| `bounds=Some(r)`, `center=None` | **Slice** (área retangular) | Hitbox, hurtbox, face-recognition zone |
| `bounds=Some(r)`, `center=Some(c)` | **9-slice region** | Dialog box pieces, button bar |
| `user_data` populado | Anchor com payload semântico | Per-hitbox damage values |

## 7.3 Onde mora

| Local | Quando |
|---|---|
| **Component `NamedAnchorList(SmallVec<[NamedAnchor; 4]>)` no Entity** | Anchors estáticos do sprite (constantes; não mudam com frame) |
| **Override em `SpriteFrames` per-frame** | Anchors que mudam com a animação (mão sobe/desce; muzzle gira) |

Resolução de runtime:
1. SpriteAnimator presente E frame atual tem override → usa override.
2. Senão → usa `NamedAnchorList` do entity.

## 7.4.1 MCP semantics canonical (Lens E E4 fix)

`sprite_anchor_*` ops têm semantics precisas pra evitar silent data-loss + adversarial input via LLM/MCP:

| op | precondition (name) | duplicate-name | requires_token (HR-11) |
|---|---|---|---|
| `sprite_anchor_get(e, n)` | UTF-8 ≤ 64 bytes; rejected control chars `< 0x20` exceto `[\t\n]`; `[a-zA-Z0-9_-]` recommended (não enforced) | n/a (read) | no |
| `sprite_anchor_list(e)` | n/a | n/a | no |
| `sprite_anchor_set(e, n, anchor)` | name UTF-8 ≤ 64 bytes + character class enforced | **REJECT com `Err(AnchorNameDuplicate)`** se name já existe (alinha com i18n string `sprite.error.anchor_name_duplicate`) | no |
| `sprite_anchor_set_or_replace(e, n, anchor)` | idem | **REPLACE silently** (use case raro; pra LLM workflow que sabe que está editando) | **yes (HR-11 destructive)** |
| `sprite_anchor_remove(e, n)` | idem | n/a (`Err(AnchorNotFound)` se ausente) | **yes (HR-11 destructive)** |
| `sprite_clear_all_anchors(e)` | n/a | n/a | **yes (HR-11 destructive)** |

**Character class enforcement** (rejeita LLM token injection):
- Allowed: `[a-zA-Z0-9_-.]` (Unicode letters/digits + ASCII underscore/hyphen/dot)
- Rejected: control chars (`< 0x20` exceto whitespace), `\0` NUL, `\x1b` ANSI escape, `\x7f` DEL.
- Empty name `""` → `Err(AnchorNameEmpty)`.

`validate_named_anchor` expandido (W5.T5.X):

```rust
pub fn validate_named_anchor(anchor: &NamedAnchor, list: &NamedAnchorList) -> Result<(), SpriteError> {
    if anchor.name.is_empty() { return Err(SpriteError::AnchorNameEmpty); }
    if anchor.name.len() > 64 { return Err(SpriteError::AnchorNameTooLong); }
    if anchor.name.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return Err(SpriteError::AnchorNameControlChar);
    }
    if list.iter().any(|a| a.name == anchor.name) {
        return Err(SpriteError::AnchorNameDuplicate);
    }
    Ok(())
}
```

Gate `validate_named_anchor_sanitizes_and_rejects_dup` (W5 expande de Lens C §11.2.8): 4 testes (length, control_char, empty, duplicate).

## 7.4 Per-frame override (Aseprite + Construct pattern)

Quando animação tem 8 frames de attack e a posição do muzzle muda em cada frame, cada frame em `SpriteFrames.frames[]` pode ter sua própria lista:

```rust
pub struct SpriteFrame {
    pub texture_ref: TextureRef,
    pub duration_ms: u32,
    
    // Per-frame overrides (Aseprite slice per-frame + Construct image_point per-frame):
    #[serde(default)]
    pub named_anchors: SmallVec<[NamedAnchor; 4]>,
    
    // ... outros campos ...
}
```

Runtime: `Sprite.frame == N` → resolver `NamedAnchor` lookup vai pra `SpriteFrames.frames[N].named_anchors` se presente; senão fallback para `NamedAnchorList` do entity.

## 7.5 Inspector UX

```
▼ Sockets / Slices  (12)
  ┌─ muzzle ─────────────────────────────────────────────┐
  │ name: muzzle                                          │
  │ transform: pos=(28, -4)  rot=12°  scale=(1, 1)        │
  │ ▸ bounds: (none — Socket)                              │
  │ ▸ user_data: {"dmg":10, "fires":"projectile_basic"}   │  (keys lex-sorted)
  │ [Drive by frame] ☐  [Add Slice]                       │
  └────────────────────────────────────────────────────────┘
  
  ┌─ face_box ────────────────────────────────────────────┐
  │ name: face_box                                         │
  │ transform: pos=(0, 12)                                 │
  │ bounds: x=8, y=4, w=24, h=24                           │
  │ center: (none — flat slice)                            │
  │ user_data: {"emotion":"happy"}                         │
  │ [Drive by frame] ☐  [Add 9-Slice Region]              │
  └────────────────────────────────────────────────────────┘
  
  ┌─ dialog_bg ─────────────────────────────────────────────┐
  │ name: dialog_bg                                          │
  │ transform: pos=(0, 0)                                    │
  │ bounds: x=0, y=0, w=128, h=64                            │
  │ center: x=16, y=16, w=96, h=32                           │
  │ user_data: (none)                                        │
  │ [Drive by frame] ☐                                       │
  └──────────────────────────────────────────────────────────┘
  
  [+ Add Anchor]
```

### Botões contextuais

| Estado atual | Botões oferecidos |
|---|---|
| `bounds=None, center=None` (Socket) | `[Add Slice]` (preenche bounds) |
| `bounds=Some, center=None` (Slice) | `[Add 9-Slice Region]` (preenche center) `[Remove Slice]` (vira Socket) |
| `bounds=Some, center=Some` (9-slice region) | `[Remove Region]` (vira Slice) `[Remove Slice]` (vira Socket) |

## 7.6 Visual handles no canvas (UX crítico)

Quando seção "Sockets / Slices" expandida no Inspector, canvas mostra:

- **Socket** → cruz colorida (drag para mover transform.translation; alt+drag rotaciona; cmd+drag escala).
- **Slice** → retângulo overlay (drag corners para resize; drag center para mover).
- **9-slice region** → retângulo externo (bounds) + retângulo interno (center), ambos editáveis.

Cor por anchor (hash do nome → cor estável). Label do nome aparece próximo ao handle.

Per-frame mode (toggle): canvas mostra anchor do frame atual (vai mudando com scrubber); fora desse mode, mostra fallback do entity.

## 7.7 Use cases concretos

### 7.7.1 Anexar projétil ao muzzle de uma arma
```rust
let muzzle = entity.named_anchor("muzzle").unwrap();
let bullet_spawn = entity.transform.global() * muzzle.transform;
spawn_bullet(bullet_spawn.translation, bullet_spawn.rotation);
```

Quando arma anima e muzzle muda de posição por frame, `entity.named_anchor("muzzle")` automaticamente retorna o override do frame atual. Zero código de animação.

### 7.7.2 Hitbox físico de char
```rust
let face = entity.named_anchor("face_box").unwrap();
let hurt = entity.named_anchor("hurt_box").unwrap();
// Slices com bounds → áreas retangulares pra collision check.
if check_collision(face.bounds.unwrap(), enemy.hit_area()) {
    apply_damage_to_face(face.user_data.get("emotion"));
}
```

### 7.7.3 9-slice region para sub-dialog
```rust
let dialog = entity.named_anchor("dialog_bg").unwrap();
draw_nine_slice(dialog.bounds.unwrap(), dialog.center.unwrap(), texture, target_size);
```

### 7.7.4 Camera follow point (Lens E E15 fix — Component canonical)

```rust
// Component anexado à Camera2D entity:
pub struct CameraFollowAnchor {
    /// Entity-alvo da qual a câmera segue um anchor named.
    pub target_entity: Entity,
    /// Nome do anchor no target_entity (e.g., "camera_target", "head", "focal_point").
    pub anchor_name: String,
}

// Sistema de update da câmera (W5+ task):
fn update_camera_follow_anchor(
    mut cameras: Query<(&mut Transform, &CameraFollowAnchor)>,
    sprites: Query<(&Transform, &NamedAnchorList)>,
) {
    for (mut cam_xform, follow) in &mut cameras {
        if let Ok((target_xform, anchors)) = sprites.get(follow.target_entity) {
            if let Some(anchor) = anchors.iter().find(|a| a.name == follow.anchor_name) {
                cam_xform.translation = target_xform.translation + anchor.transform.translation;
            }
        }
    }
}
```

Conta como **31º Component opcional** no cap `≤ 32` ([§2.7 ADR-0074](../architecture/decisions/0074-sprite-component-boundary.md)). W5.T5.X task adicionada no plano §15.6. Gate `camera_follow_anchor_canonical` testa lookup + update.

## 7.8 MCP / Luau exposure

Cada NamedAnchor exposto via `#[lua_export]`:

```lua
local muzzle = ph2d.sprite.anchor(entity, "muzzle")
ph2d.spawn(bullet_prefab, muzzle.position, muzzle.rotation)
```

MCP toolset `sprite_anchor_*`:
- `sprite_anchor_get(entity, name)` → AnchorData
- `sprite_anchor_set(entity, name, anchor)` → mutates
- `sprite_anchor_list(entity)` → [Name]

(HR-10; HR-11 não aplica — não há ação destrutiva implícita.)

## 7.9 Aseprite import flow

Quando importer carrega `.ase` com slices:
- Aseprite `bounds` → PH2D `bounds`.
- Aseprite `center` → PH2D `center`.
- Aseprite `pivot` → PH2D `transform.translation` (interpretado como anchor point dentro do bounds).
- Aseprite slice data → PH2D `user_data.Str(...)`.

Conversão lossless. Aseprite tags → PH2D `SpriteFrames.tags` (vide [08_animation_inline.md](08_animation_inline.md)).

## 7.10 Construct 3 import flow

Image Point → NamedAnchor com `bounds=None, center=None` (Socket); transform.translation = image point pos.

Loss-less.

## 7.11 Unreal Paper 2D import flow

Socket → NamedAnchor com `bounds=None`; transform copiado direto.

Loss-less.

## 7.12 Caps gateados

| Cap | Valor | Razão |
|---|---|---|
| `NamedAnchorList` inline SmallVec | 4 | Shape comum (~3 anchors médio); >4 vai pra heap |
| Maximum anchors por sprite | 64 | Sanity; >64 sugere refactor pra hierarquia ECS |
| `NamedAnchor.name` length | **≤ 64 bytes UTF-8** (não chars) | Identificador, não payload. Bytes é determinístico cross-OS; chars Unicode pode inflar (1 emoji = 4 bytes) e quebrar SmallVec assumptions. Postcard serializa length-prefixed bytes (varint). H6 fix Lens C. |
| `AnchorData.Dict` depth | ≤ 4 | Anti-recursion-DoS |
| `AnchorData.Dict` keys | ≤ 32 por nível | Anti-explosion |
| `AnchorData.Dict` key-sorted invariant | ENFORCED | Lens C H5 — keys lexicograficamente ordenadas always; gate `anchor_dict_keys_sorted_invariant` |

## 7.13 Performance

- Lookup por nome: linear scan de SmallVec (≤4 anchors comum) → O(N) trivial.
- Heap fallback (≥4 anchors): ainda linear, mas pode ser otimizado com `bevy_ecs::EntityHashMap` (deterministic) se virar gargalo — `std::HashMap` proibido (ADR-0022).
- Per-frame override: extra indirection via SpriteFrames lookup; negligible (1 cache lookup adicional por anchor query).

Sem necessidade de spatial index. Anchors são poucos (~3-10 por sprite); query é raro (1× por hitcheck, 1× por spawn, etc.).

## 7.14 Anti-padrões evitados

1. **3 conceitos paralelos (socket + slice + image_point) como sub-assets distintos** ❌ — Paper2D / Aseprite / Construct cada um tem o seu. PH2D unifica.
2. **Anchors como entidades ECS filhas** ❌ — explode hierarquia; cada socket vira entity overhead. SmallVec inline cobre 99%.
3. **Per-frame override implícito (sem schema explícito)** ❌ — Aseprite slices "podem mudar por frame" mas isso vira surpresa runtime. PH2D explicita via `SpriteFrame.named_anchors`.
4. **Nome como int** ❌ — "anchor 5" ≠ "muzzle". Strings com snake_case convention.
5. **Visual handles só em DCC externo** ❌ — handles in-canvas pra editar transform/bounds direto.
