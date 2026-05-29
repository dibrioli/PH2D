# 05 — Ordering / Sorting — pipeline canônico

## 5.1 Pipeline canônico (ordem fixa)

Sprite tem **múltiplos eixos de ordem** que se compõem. PH2D define a ordem FIXA:

```
1. Viewport               (separação de telas)
2. CanvasLayer/SortingLayer named  (BG / Player / FG / UI)
3. YSort cascateado dos ancestrais (topdown)
4. ZIndexOverride + ZAsRelative (override absoluto/relativo)
5. SortingGroup (sub-hierarquia como bloco único)
6. ShowBehindParent (flip local da ordem ant/dep do pai)
7. DFS counter (fallback determinístico do scene tree)
```

**Critical:** o resultado é determinístico cross-platform (HR-5) porque cada passo é uma chave de sort estável.

## 5.2 Cada passo, detalhado

### Passo 1: Viewport
Mundos completos separados. Ex: HUD vs world. Implementado via Camera2D distinta. Não é Inspector do Sprite.

### Passo 2: SortingLayer (named)
Macro-camada nominal. Project Settings define a lista ordenada (default: "Background", "Default", "UI"). Sprite com Component `SortingLayer(LayerId)` indica qual layer. Sem Component → "Default".

Layer com index menor renderiza primeiro (mais ao fundo). Convenção:
- "Background" — sky, parallax-far
- "Midground" — props, terrain
- "Default" — gameplay entities (player, enemies, items)
- "Foreground" — particles, foreground decoration
- "UI" — HUD, menus

Razão named (não int): "Sorting Layer 5" não comunica intent. "UI" comunica. Unity acerta nesse design.

### Passo 3: YSort cascateado
Quando ancestral tem Component `YSort { enabled: true, ... }`, os DESCENDENTES diretos são sortados por sua coordenada `y` global (mais Y = mais à frente). Sort point opções:

- **Center** — usa centro da bbox (default).
- **Pivot** — usa pivot do sprite (`anchor` aplicado). Recomendado pra topdown RPG (pé do char).
- **Custom(Vec2)** — projeta posição num eixo arbitrário. Iso 45° = `Vec2(1, 1)`.

YSort cascateia recursivamente: filho-de-filho-y-sorted herda sort key... ATÉ encontrar quebra por ZIndex (passo 4).

**Cascateamento exemplo:**
```
World [Y-sort=true]
├─ Tree    (y=10)  → 2º
├─ Player  (y=5)   → 1º (atrás)
└─ Rock    (y=15)  → 3º (frente)
```

### Passo 4: ZIndexOverride + ZAsRelative
Component `ZIndexOverride(i32)` força ordem manual. Quando presente, sobrescreve YSort dentro do mesmo SortingLayer.

`ZAsRelative(bool)`:
- `true` (default) — Z efetivo = `ZIndexOverride + Z_efetivo_do_pai`. Hierarquia decorativa (sombra atrás do char, arma na frente).
- `false` — Z absoluto, ignora pai. Overlay no topo independente de hierarquia.

**Pegadinha histórica (Godot):** Z != YSort vivem em mundos paralelos. Z primeiro buckets, dentro do bucket o YSort ordena. PH2D mantém a mesma semântica (porque é o que faz sentido — ambos são úteis e ortogonais).

### Passo 5: SortingGroup
Component `SortingGroup { sort_at_root: bool }` num ancestral diz "sub-hierarquia inteira sorta como BLOCO ÚNICO".

Útil para: personagem multi-piece (corpo + roupa + arma + escudo). Sem Sorting Group, peças do personagem misturam com sprites do mundo (Z de cada parte interfere). Com, sub-hierarquia toda renderiza como uma unidade no Z do `SortingGroup` raiz.

`sort_at_root: true` no DESCENDENTE — descenden é tirado do bloco e sortado globalmente. Escape hatch.

### Passo 6: ShowBehindParent
Component `ShowBehindParent` (zero-size marker) no filho. Quando presente, filho desenha ANTES do pai (sem precisar reordenar scene tree). Sombra do char como filho do char, mas atrás dele:

```
Player
├─ Shadow [ShowBehindParent]  → desenha 1º (atrás)
├─ Body                        → desenha 2º (em cima)
└─ Hat                          → desenha 3º (frente)
```

Sem `ShowBehindParent`, ordem seria Body → Shadow → Hat (DFS), e Shadow sobreporia o Body.

### Passo 7: DFS counter (fallback)
Quando nada acima decide, ordem é DFS top-down da scene tree. Determinístico, hierarchy-aware, free.

`RenderInstance.z_order` recebe esse counter na extract phase (já existe em [sprite.rs:217](../../crates/ph2d-render/src/sprite.rs#L217)).

## 5.3 Algorítmo composto (extract phase)

```rust
fn compute_sort_key(entity: Entity, ...) -> SortKey {
    let viewport = camera_of(entity);
    let sort_layer = component::<SortingLayer>(entity).unwrap_or_default();
    let ysort_key = y_sort_cascade(entity);                  // None se nenhum ancestral tem YSort
    let z_override = component::<ZIndexOverride>(entity);    // Option<i32>
    let z_relative = component::<ZAsRelative>(entity).map_or(true, |z| z.0);
    let sort_group = ancestor_with::<SortingGroup>(entity);  // Option<Entity>
    let show_behind = component::<ShowBehindParent>(entity).is_some();
    let dfs_index = dfs_counter(entity);

    SortKey {
        viewport,
        sort_layer: sort_layer.id,
        ysort: ysort_key,
        z: compute_z(z_override, z_relative, parent),
        sort_group: sort_group_or_self(sort_group, entity),
        show_behind,           // boolean affecting tie-break with parent
        dfs_index,
    }
}

// Comparison ordering: lexicographic com fields acima na ordem.
impl Ord for SortKey { ... }
```

## 5.4 Translucency Sort Priority + Distance Offset (Paper2D)

Paper2D tem **dois ajustes finos** úteis no caso de `BlendMode != Mix`:
- **Priority (int)** — prioridade absoluta sobre Z (HUD overlay vence Z do mundo).
- **Distance Offset (float, world m)** — empurra distância-da-câmera computada (CPU side, antes de translucency sort).

PH2D adopta como Components opcionais raros (`TranslucencySortPriority(i32)`, `TranslucencyDistanceOffset(f32)`). Caso de uso: glow add-blended que deve sempre ficar na frente da mesh do enemy mesmo com Z igual.

## 5.5 Order Debug Overlay (🆕 PH2D)

Component `OrderDebugOverlay(bool)` no Sprite OU global toggle em "Debug ▶ Show Sort Order".

Quando ativo, cada sprite recebe **overlay visual** no canvas:
- **Cor de fundo (40% alpha)** = cor da SortingLayer (dicionário Project Settings).
- **Label** = `"Z: 5 | Y: 12.3 | DFS: 47"`.
- **Marcador de YSort axis** se YSort.enabled.

Resolve "por que esse sprite tá atrás daquele?" em 1 segundo. Gap universal nos engines existentes; só web tem (Edge DevTools 3D View pra z-index CSS).

Implementação leve: shader debug pass extra, ativado só com flag. Custo zero em release builds (compilado fora via `#[cfg(feature="editor")]`).

## 5.6 Gates de regressão

Sorting é frágil — Godot tem 5 issues recentes (#74265 sombras cobrem sprite, batching reorder etc.). PH2D adiciona:

| Test | O que verifica |
|---|---|
| `tests/sorting_pipeline_determinism.rs` | Mesmo cenário (10 sprites, 3 níveis hierarquia, Z + YSort + ShowBehindParent) produz ordem byte-identical cross-OS. |
| `tests/y_sort_cascade.rs` | YSort em ancestral propaga corretamente; quebra correta em ZIndex divergente. |
| `tests/sorting_group_block.rs` | Multi-piece char sortado como bloco; descendente com SortAtRoot foge do bloco. |
| `tests/show_behind_parent.rs` | Filho com ShowBehindParent renderiza antes do pai (DFS ordem invertida). |

Smoke do Enio em cada wave: cenário visual fixo (10 sprites canônicos) com 5 configurações. Capturar screenshot golden, comparar pixel-a-pixel em CI.

## 5.7 Caps gateados

| Cap | Valor | Razão |
|---|---|---|
| `SortingLayer` máx no projeto | **32 (FROZEN; H7 reconciliado)** | Caps explosão de comparação; cobre 95% cenários reais. Bump exige amendment ADR-0073-amendment-N. |
| `ZIndexOverride` range | `i32::MIN..i32::MAX` | sem clamp; usuário decide |
| `YSort` cascade depth | até raiz | sem limit |
| `OrderDebugOverlay` ativo | só em build com feature `editor` | Performance |

## 5.8 Interação com `CanvasLayer` (Godot legacy concept)

Godot tem `CanvasLayer` separado de SortingLayer. PH2D simplifica:
- Camera2D distintas = "Layer separation" macro (HUD vs world).
- `SortingLayer` Component = "Layer hierarchy" dentro de uma camera.

`CanvasLayer.layer (int)` do Godot vira nossa `SortingLayer(LayerId)` Component direta. Não temos duplicação Godot.

## 5.9 Z_INDEX_MAX / MIN

Limites de `ZIndexOverride`:
- **MAX**: `i32::MAX / 4` (1 bilhão) — folga grande pra somas hierárquicas sem overflow.
- **MIN**: `-i32::MAX / 4`.

Implementação: saturating arithmetic em `ZAsRelative=true` cascade. Sem panic em overflow; clampa.

Diferente de Godot que limita ZIndex a `-4096..4095` por causa do sort key compacto. PH2D usa SortKey lexicographic (não compacto), permitindo range maior.

## 5.10 Compatibilidade com Hierarchy panel

O `ph2d-panel-hierarchy` mostra entidades em ordem DFS (cima → baixo = render order). Quando `Order Debug Overlay` ativo, Hierarchy também colore cada linha pela SortingLayer correspondente. UX coerente.

## 5.11 Anti-padrões evitados

1. **Z absoluto único (sem relative)** ❌ — Godot history mostra que artistas pedem ambos. Component com toggle é canônico.
2. **YSort flat sem cascade** ❌ — Unity Order in Layer não cascateia, força repetir em cada child. PH2D cascada com YSort do ancestral.
3. **SortingLayer como int** ❌ — Unity legacy; named string + ordering em Project Settings é mais legível.
4. **Show Behind Parent fora de Inspector (só code-side)** ❌ — feature acessível artistas; toggle no Inspector é canon.
5. **Order Debug só via plugin third-party** ❌ — built-in, free, sem instalar nada.
