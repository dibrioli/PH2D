# ADR-0073 — Sorting canonical order (Z + ZAsRelative + YSort + SortingGroup + ShowBehindParent + DFS)

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0069 — decisão-mãe](0069-sprite-inspector-v2.md), [ADR-0021 — SimWorld/PresentWorld](0021-simulation-presentation-boundary.md), [ADR-0025 — GameObject model](0025-gameobject-model.md).
**Spec normativa:** [`docs/Sprite_projeto/05_ordering_sorting.md`](../../Sprite_projeto/05_ordering_sorting.md).
**Tags:** sprite, sorting, ordering, determinism, hr-5

---

## 1. Contexto

Sorting de sprites 2D tem **múltiplos eixos** que precisam compor consistentemente:
- Z Index (override manual de ordem)
- Z As Relative (Z absoluto vs hierárquico)
- Y Sort (topdown/iso depth via coordenada Y)
- Sorting Layer (macro-camada nominal)
- Order in Layer (micro-ordering)
- Sorting Group (sub-hierarquia como bloco único)
- Show Behind Parent (flip local da ordem ant/dep do pai)
- DFS counter (fallback determinístico do scene tree)

**Sem ordem canônica** = bugs sutis cross-platform, regressões silenciosas, debug nightmare. Godot tem 5+ issues abertos relacionados a sorting (#74265 shadows cover sprites, batching reorder).

PH2D canoniza a ordem **fixa, lexicographic, determinística cross-OS**.

---

## 2. Decisão

### 2.1 Pipeline canônico (7 estágios)

Ordem FIXA de aplicação na extract phase:

```
1. Viewport                      (separação de telas)
2. SortingLayer (named)          (macro-camada — BG/Player/UI)
3. YSort cascateado dos ancestrais (topdown sort por Y global)
4. ZIndexOverride + ZAsRelative   (override absoluto/relativo)
5. SortingGroup                   (sub-hierarquia como bloco único)
6. ShowBehindParent               (flip local ant/dep do pai)
7. DFS counter                    (fallback determinístico do scene tree)
```

Resultado: `SortKey` lexicographic; sort estável.

### 2.2 SortKey struct

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    pub viewport: ViewportId,         // 1. Viewport
    pub sort_layer: LayerId,           // 2. SortingLayer (u32 stable)
    pub ysort: Option<i32>,            // 3. YSort (fixed-point Y * 1000 OR None)
    pub z: i32,                        // 4. ZIndex (computed com ZAsRelative)
    pub sort_group_root: EntityId,     // 5. SortingGroup root entity (self if not in group)
    pub show_behind: bool,             // 6. ShowBehindParent (boolean)
    pub dfs_index: u32,                // 7. DFS counter
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.viewport.cmp(&other.viewport)
            .then(self.sort_layer.cmp(&other.sort_layer))
            .then(self.ysort.cmp(&other.ysort))
            .then(self.z.cmp(&other.z))
            .then(self.sort_group_root.cmp(&other.sort_group_root))
            // show_behind: true means BEFORE parent (negate em compare)
            .then(other.show_behind.cmp(&self.show_behind))
            .then(self.dfs_index.cmp(&other.dfs_index))
    }
}
```

### 2.3 Cada estágio, semântica

**Estágio 1 — Viewport (contextual, NÃO Inspector do Sprite):** Camera2D distinta = "Layer separation" macro (HUD vs world). Não editável no Inspector do Sprite — vive em Camera. **Listado por completeness do pipeline; cap "7 estágios" inclui este contexto, dos quais 6 são editáveis pelo Sprite Inspector + 1 é contexto pré-Sprite.**

**Estágio 2 — SortingLayer:** Component `SortingLayer(LayerId)`. Project Settings define lista ordenada nominal ("Background", "Default", "UI"). Default = "Default" se Component ausente.

**Estágio 3 — YSort:** Component `YSort { enabled, axis, sort_point }` no ANCESTRAL cascateia pros descendentes. Descendant herda sort key. YSort axis = vetor 2D pra projeção (iso 45° = `Vec2(1,1)`). Sort point = Center / Pivot / Custom(Vec2).

Quando ZIndexOverride presente no descendant, QUEBRA o YSort cascade (Z primeiro buckets).

**Estágio 4 — ZIndexOverride + ZAsRelative:** Component `ZIndexOverride(i32)`. Quando presente:
- `ZAsRelative(true)` (default) → `Z_efetivo = ZIndexOverride + Z_efetivo_do_pai`.
- `ZAsRelative(false)` → Z absoluto, ignora pai.

Sem Component ZIndexOverride → não há override; estágio 4 não muda key (usa default 0). Ausência ≠ "Z=0 explícito".

**Estágio 5 — SortingGroup:** Component `SortingGroup { sort_at_root: bool }` num ancestral. Sub-hierarquia sorta como BLOCO ÚNICO no Z do SortingGroup raiz. Multi-piece char (body + arm + sword) renderiza como unidade.

`sort_at_root: true` no descendant → escape do bloco (sortado globalmente).

**Estágio 6 — ShowBehindParent:** Component `ShowBehindParent` (zero-size marker) no filho. Quando presente, filho desenha ANTES do pai (sem reordenar scene tree). Sombra atrás do char.

**Walkthrough canônico** (corrigido pós-audit — comparação invertida no SortKey é sutil):

Cenário: `Player` é entity-pai com Body + Shadow + Hat filhos:
```
Player [SortingGroup, default sort_group_root = Player.entity_id]
├─ Shadow [ShowBehindParent]  → show_behind=true
├─ Body                        → show_behind=false
└─ Hat                          → show_behind=false
```

DFS order: Player, Shadow, Body, Hat (DFS index 0, 1, 2, 3).

SortKey comparação dentro de `sort_group_root=Player`:
- `Shadow.show_behind = true`; `Body.show_behind = false`.
- `cmp` ordena: `other.show_behind.cmp(&self.show_behind)` → `false.cmp(&true) = Less` → Shadow ordena BEFORE Body em `Less` direction → Shadow desenha PRIMEIRO → fica ATRÁS visualmente.
- `Body.show_behind = false`; `Hat.show_behind = false` → tie em show_behind; cai pra `dfs_index` → Body (2) < Hat (3) → Body desenha antes de Hat → Hat fica visualmente NA FRENTE de Body.

Resultado: Shadow → Body → Hat ordem de pintura, que é o desejado.

**Pai (Player) vs Filho:** Quando Player também participa do sort (entity próprio), Player tem `sort_group_root = Player.entity_id` igual aos filhos; comparison entre Player e Shadow tem `show_behind` ambos `false` no Player (sem marker) vs `true` no Shadow → `false.cmp(&true) = Greater` em `other.cmp` (invertido) → Shadow ordena BEFORE Player → Shadow desenha antes do Player → fica atrás. ✓

**Estágio 7 — DFS counter:** fallback determinístico. Top-down DFS da scene tree. `RenderInstance.z_order` recebe esse counter já existente em [sprite.rs:217](../../../crates/ph2d-render/src/sprite.rs#L217).

### 2.4 Determinismo cross-OS

HR-5 aplica em SimWorld. SortKey é computada na extract phase (PresentWorld) **mas** consome estado canônico do SimWorld (positions, hierarchies). Para byte-identical cross-OS:

1. **YSort fixed-point** (corrigido pós-Lens-C H1): `ysort_key = ((global_y as f64 * 1000.0).round() as i64).saturating_cast::<i32>()` (1mm precision). Razão da promoção `f64`: multiplicação `f32 × 1000.0` pode ser FMA-contracted pelo compiler em x86_64 (com `target-feature=+fma`) mas não em aarch64 sem FMA → resultado ULP-divergent → `as i32` truncation produz boundary jumps (e.g., `global_y = 0.0005` → ysort=0 em um OS, ysort=1 em outro). Promover a `f64` antes da multiplicação reduz erro de arredondamento; `round()` em vez de truncation evita boundary jumps. `as i32` saturating no fim. **`global_y` deve já estar canônico** (passou por `Transform::compose` com `libm` — vide [ADR-0025-amendment-1 §2.4](0025-amendment-1.md)).
2. **Ordering de ties**: sempre lexicographic; `cmp(self, other)` é total order.
3. **DFS counter**: top-down traversal estável (children sortados por entity bits, não hash).
4. **SortingGroup root**: usa entity_bits (canonical identifier).

Gate `tests/sorting_pipeline_determinism.rs` (cross-OS matrix): hash blake3 do `Vec<RenderInstance>` produzido em cenário fixo é bit-identical Linux x86_64 / macOS aarch64 / Windows x86_64. **Test também inclui `ysort_quantization_boundary` com valores próximos a 0.0005, 1.0005** para garantir que YSort fixed-point não diverge cross-OS.

**Scene fixture canônica concreta (Lens E E11 fix — Lens C anterior tinha "10 sprites em hierarquia 3-níveis" sem valores; baseline arbitrária = gate vacuous):**

```
Entity    | Parent  | x      | y     | Z   | sort_layer | YSort.enabled | ShowBehindParent | SortGroup.sort_at_root
---       | ---     | ---    | ---   | --- | ---        | ---           | ---              | ---
sky       | None    |   0.0  |  0.0  |  0  | "BG"       | false         | false            | false
midground | None    |   0.0  |  0.0  |  0  | "Mid"      | false         | false            | false
world     | None    |   0.0  |  0.0  |  0  | "Default"  | true          | false            | false
player    | world   | 100.0  | 50.5  |  0  | (inherit)  | (inherit)     | false            | true
shadow    | player  |   0.0  |  2.0  | -1  | (inherit)  | (inherit)     | true             | false
body      | player  |   0.0  |  0.0  |  0  | (inherit)  | (inherit)     | false            | false
hat       | player  |   0.0  |-16.0  |  1  | (inherit)  | (inherit)     | false            | false
tree      | world   | 200.0  | 30.0  |  0  | (inherit)  | (inherit)     | false            | false
rock      | world   | 150.0  | 70.0  |  0  | (inherit)  | (inherit)     | false            | false
hud_bar   | None    |   0.0  |  0.0  |  0  | "UI"       | false         | false            | false
```

**Expected ordem RenderInstance (top-to-bottom de painting):**

```
[sky, midground, shadow, tree, body, hat, player, rock, hud_bar]
```

(player tem SortGroup; shadow tem ShowBehindParent dentro do grupo, body+hat dentro depois. tree y=30 sorta antes (yhigher = farther back); player y=50 sorta antes de rock y=70 dentro de world layer.)

**Expected hash blake3 do `Vec<RenderInstance>` serializado postcard:** `blake3:<hex em fixtures/sorting_determinism.expected>` (gerado em qualquer host via `libm` deterministic — vide ADR-0025-amendment-1 §2.4; cross-OS bit-identical).

Implementador W3.T3.19 carrega fixture, sortta, computa hash, compara. Sem o fixture concreto, baseline arbitrária = gate tautologicamente passa.

### 2.5 Order Debug Overlay (🆕 PH2D)

Component `OrderDebugOverlay(bool)` no entity OU global toggle "Debug ▶ Show Sort Order".

Quando ativo, cada sprite renderiza overlay:
- **Cor de fundo (40% alpha)** = cor da SortingLayer (dicionário Project Settings).
- **Label** = `"Z: 5 | Y: 12.3 | DFS: 47 | SortingGroup: Player"`.
- **Marcador de YSort axis** se YSort.enabled (linha + seta).

Resolve "por que esse sprite tá atrás daquele?" em 1 segundo. Gap universal nos engines existentes; só web tem (Edge DevTools 3D View).

Implementação: shader debug pass extra, ativado só com flag. Custo zero em release builds (compilado fora via `#[cfg(feature="editor")]`).

### 2.6 Caps congelados

Arch-gate `sorting_caps` em `crates/ph2d-render/tests/`:

| Cap | Valor | Razão |
|---|---|---|
| `SortingLayer` count no projeto | **≤ 32** | Decisão pós-audit (drift inter-arquivo H7 reconciliado): 32 cobre 95% dos casos reais; AAA games observados 50-100 são EDGE (geralmente layers semânticos overlap). Bump exige amendment. SortKey comparison O(1) em u8. Spec original (§6 Open questions) já dizia "32 cobre todos cenários reais"; consolidado aqui. |
| `LayerId` tipo (Lens D D20) | `pub struct LayerId(pub u8);` newtype + `SortingLayerRegistry { layers: SortedSmallVec<[(LayerId, Box<str>); 32]> }` | u8 cobre cap 32; `LayerId::DEFAULT = LayerId(0)` reserved para "Default" layer; registry serializável via postcard |
| `ZIndexOverride` range | `i32::MIN..i32::MAX` (sem clamp) | Usuário decide |
| `YSort` cascade depth | até raiz (sem limit) | Recursive |
| `OrderDebugOverlay` ativo | só em build com feature `editor` | Performance |
| Pipeline estágios | **7 FROZEN** | Bump exige amendment |

Bump → ADR-0073-amendment.

### 2.7 Gates de regressão obrigatórios

Sorting é frágil. Godot tem 5 issues abertos. PH2D adiciona:

| Test | O que verifica |
|---|---|
| `tests/sorting_pipeline_determinism.rs` | Mesmo cenário (10 sprites, 3 níveis, Z + YSort + ShowBehindParent) produz ordem byte-identical cross-OS |
| `tests/y_sort_cascade.rs` | YSort em ancestral propaga pros descendentes corretamente; quebra correta em ZIndex divergente |
| `tests/sorting_group_block.rs` | Multi-piece char sortado como bloco; descendente com SortAtRoot foge do bloco |
| `tests/show_behind_parent.rs` | Filho com ShowBehindParent renderiza antes do pai (DFS ordem invertida) |
| `tests/sort_layer_macro_buckets.rs` | "BG" renderiza antes de "Default" antes de "UI" |
| `tests/z_relative_vs_absolute.rs` | ZAsRelative(true) cascada com pai; ZAsRelative(false) ignora pai |

Smoke do Enio (W3): cenário visual fixo de 10 sprites canônicos com 5 configurações. Capturar screenshot golden; comparar pixel-a-pixel em CI.

---

## 3. Consequências

### 3.1 Positivas

- **Pipeline determinístico** (cross-OS hash bit-identical) — HR-5 honrado.
- **7 estágios cobrem todos os casos** identificados na pesquisa multi-engine.
- **Lexicographic SortKey** é total order; sort estável.
- **Order Debug Overlay** built-in é diferencial UX raro.
- **Gates de regressão obrigatórios** previnem Godot's bug pattern (5 issues abertos).
- **ZIndexOverride ausente vs explícito** distingue "use DFS" vs "Z=0 forçado".

### 3.2 Negativas

- **`SortKey` é struct grande** (~32 bytes) — sort em vector de N=10000 sprites ainda é O(N log N), aceito.
- **7 Components opcionais** novos (ZIndexOverride, ZAsRelative, SortingLayer, OrderInLayer, YSort, SortingGroup, ShowBehindParent, TopLevel, OnScreenEnabler, OrderDebugOverlay) — adiciona ~10 types em ph2d-ecs. Drop-crate fan-out cobre.

### 3.3 Neutras

- **YSort fixed-point** (1mm precision) suficiente; degrada precision além de 1000m de raio (mundos enormes), aceito.
- **DFS counter** já existe em extract phase ([sprite.rs:217](../../../crates/ph2d-render/src/sprite.rs#L217)); reaproveita.

---

## 4. Alternativas consideradas

### 4.1 Z único (sem ZAsRelative) — rejeitada

Z absoluto somente. **Por que rejeitada:** "Z relativo ao pai" é caso comum (sombra atrás do char, arma na frente). Godot acerta com toggle; PH2D segue.

### 4.2 YSort flat (sem cascade) — rejeitada

Unity Order in Layer não cascateia; força repetir em cada child. **Por que rejeitada:** topdown RPG canônico usa YSort cascateado (`world` tem `y_sort_enabled=true`; chars filhos sortam automaticamente).

### 4.3 SortingLayer como int — rejeitada

Unity legacy aceita int. **Por que rejeitada:** "Sorting Layer 5" não comunica; "UI" comunica. Named string + ordering em Project Settings é estritamente melhor.

### 4.4 ShowBehindParent fora de Component (só via reordenação manual de scene tree) — rejeitada

Forçar usuário a reordenar tree. **Por que rejeitada:** quebra organização hierárquica (sombra como FILHO do char mas atrás dele). Component marker é zero-overhead.

### 4.5 Sem Order Debug Overlay built-in — rejeitada

Deixar pra plugin third-party. **Por que rejeitada:** diferencial UX importante; built-in é zero-cost em release (feature gate); gap universal nos engines.

### 4.6 SortKey compacto (bit-packed u64) — rejeitada

Compress SortKey em 8 bytes. **Por que rejeitada:** Z range -i32::MAX..i32::MAX exige 32 bits sozinho; outros fields não cabem. Lexicographic comparison em struct é fast (CPU branch predictor lida bem).

---

## 5. Implementação (Wave 3)

Tasks T-W.N vide [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.4`](../../Sprite_projeto/15_plano_de_implementacao.md).

W3 fecha quando:
- 7 Components ECS criados (ZIndexOverride, ZAsRelative, SortingLayer, OrderInLayer, YSort, SortingGroup, ShowBehindParent).
- Pipeline canônico extract phase implementado.
- 6 testes de regressão verdes cross-OS.
- Order Debug Overlay funcional.
- Seção 7 do Inspector completa.
- Smoke do Enio (vide plano §15.4).

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| YSort axis customizable per-sprite OR per-camera? | **Per-sprite (Component YSort)** — diferentes regiões podem ter diferentes axes (iso area vs topdown area). |
| Sorting Layer count > 32? | Bump exige amendment; 32 cobre todos cenários reais. |
| `OnScreenEnabler` faz parte deste pipeline? | **Não** — culling de processing, não de ordering. Vive na seção Visibility (ADR-0069 §3 sec 8). |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/05_ordering_sorting.md`](../../Sprite_projeto/05_ordering_sorting.md).
- ADR pais: [ADR-0069](0069-sprite-inspector-v2.md), [ADR-0021](0021-simulation-presentation-boundary.md).
- Godot CanvasItem z_index, z_as_relative, y_sort_enabled: <https://docs.godotengine.org/en/stable/classes/class_canvasitem.html>.
- Unity Sorting Group: <https://docs.unity3d.com/6000.0/Documentation/Manual/sprite/sorting-group/sorting-group-reference.html>.
- Unity Sorting Layers + Order in Layer: <http://docs.unity3d.com/Manual/2d-renderer-sorting.html>.
- Godot Issue #74265 (shadows cover sprites): <https://github.com/godotengine/godot/issues/74265>.
- Bugnet — Fix Z-Index in Godot 2D: <https://bugnet.io/blog/fix-z-index-not-working-correctly-godot-2d>.
