# ADR-0074 — Sprite struct vs Component ECS — princípio operacional

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0069 — decisão-mãe](0069-sprite-inspector-v2.md), [ADR-0025 — GameObject model](0025-gameobject-model.md), [ADR-0025 amendment-1 — Transform 2D skew](0025-amendment-1.md).
**Spec normativa:** [`docs/Sprite_projeto/02_components_ortogonais.md`](../../Sprite_projeto/02_components_ortogonais.md).
**Tags:** sprite, ecs, boundary, anti-pattern, foundational

---

## 1. Contexto

Sprite Inspector v2 (ADR-0069) lista ~70 propriedades editáveis em 12 seções. Decidir **onde cada uma mora** é foundational — escolha errada cria débitos arquiteturais permanentes:

**Anti-padrões observados:**
- ❌ **GameMaker** espalha `image_blend`/`image_angle`/`image_alpha`/`image_xscale`/`image_yscale` como variáveis de instância do objeto = sopa, zero type safety.
- ❌ **Phaser** acumula mixins (Alpha, Tint, Crop, Mask, Flip, Origin, Pipeline) num objeto único = state-stuffed; Inspector vira sopa visual.
- ❌ **Construct 3** mantém 9-Patch como objeto SEPARADO de Sprite = duplica registry; conversões entre tipos exigem re-criar.
- ❌ **Unity SpriteRenderer.color** herda sem `self_modulate` = força artista a duplicar tint em material override pra hurt-flash local.

**Padrões corretos observados:**
- ✅ **Godot CanvasItem vs Sprite2D**: visual base (modulate, z_index, clip_children) vive em CanvasItem; sprite-specific (texture, centered, hframes) vive em Sprite2D. Separação por concern.
- ✅ **Defold**: slice-9 é flag do Sprite Component, não objeto separado.
- ✅ **Unity SortingGroup component**: feature opcional via Component anexável; ausência = comportamento default.

PH2D adopta **princípio operacional explícito** pra ditar todas as decisões futuras sobre "onde mora?".

---

## 2. Decisão

### 2.1 Regra dos 3 lugares (decisão tree)

```
┌─ É aparência intrínseca da imagem, universal,
│  todo sprite tem, default benigno, POD pequeno?
│  └─ SIM → Campo do Sprite struct (versionado, serde)
│
├─ É aspecto ortogonal, opcional, nem todo sprite carrega,
│  ausência ≠ default explícito, pode ter UI/dispatch própria?
│  └─ SIM → Component ECS anexável (presença = override)
│
└─ É derivado por algoritmo per-frame, produto de avaliador?
   └─ SIM → Output de Sistema ECS (extract) ou Nó do grafo (Motion/Shader)
```

### 2.2 Teste decisivo: "ausência ≠ default explícito?"

Se você consegue dizer "este sprite **não tem** essa propriedade" e isso ter significado DISTINTO de "tem com valor X" → **Component**.

Exemplos:
- "Sprite sem `ZIndexOverride`" → use DFS counter (NÃO "Z=0 explícito"). → Component opcional.
- "Sprite sem `tint`" → impossível, tint sempre tem valor. → Campo do struct.
- "Sprite sem `ShowBehindParent`" → "default = false comportamento". → Component marker opcional.

### 2.3 Critérios para campo do struct

Todos satisfeitos:
1. **Universal** — TODO sprite tem (independente de uso).
2. **Default benigno** — valor padrão não muda visual (`WHITE` tint, `1.0` opacity).
3. **POD pequeno** — couvre em ~4-16 bytes.
4. **Schema-versioned** — bump exige migrator (HR-14).
5. **Cabe na ABI `RenderInstance`** — campo provável de chegar ao GPU.

### 2.4 Critérios para Component ECS

Pelo menos UM satisfeito:
1. **Aspecto ortogonal** — pode existir/não-existir sem afetar outras propriedades.
2. **Ausência ≠ default explícito** — distinção semântica importante.
3. **UI/dispatch própria** — seção dedicada no Inspector ou handler de evento próprio.
4. **Lifecycle independente** — anexa/remove sem reconstruir sprite.

### 2.5 Critérios para derivado (Sistema OU nó do grafo)

Pelo menos UM satisfeito:
1. **Computado per-frame** — não persiste como estado canônico.
2. **Função de outros componentes** — derivado de Transform global, hierarchy, etc.
3. **GPU-bound** — vive em buffers de GPU, não em CPU heap.
4. **Nó do grafo Motion/Shader** — produzido por avaliador procedural.

### 2.6 Tabela canônica de classificação

Vide [`docs/Sprite_projeto/02_components_ortogonais.md §2.2`](../../Sprite_projeto/02_components_ortogonais.md) para tabela completa. Resumo:

| Categoria | Onde mora | Exemplos |
|---|---|---|
| **Sprite struct** | aparência intrínseca | source, size, tint, self_tint, per_corner_tint, tint_fill, opacity, anchor, flip_x/y, centered, offset, hframes/vframes, frame, region_*, premultiplied, version |
| **Component ECS** | aspecto ortogonal | Transform, Visibility, Name, ZIndexOverride, SortingLayer, YSort, SortingGroup, ShowBehindParent, TopLevel, ClipChildren, MaskInteraction, VisibilityLayer, TextureFilter, TextureRepeat, Material, UseParentMaterial, InstanceShaderParams, BlendMode, OnScreenEnabler, SliceNine, SpriteAnimator, NamedAnchorList |
| **Derivado** | sistema ECS / nó | RenderInstance.world_pos (via propagate_transforms), RenderInstance.z_order (via DFS), RenderInstance.rotation (via Transform decomposition), RenderInstance.tint collapsed (via cascade collapse extract phase) |

### 2.7 Anti-padrões explicitamente banidos

| Anti-padrão | Onde foi visto | Por quê banido |
|---|---|---|
| **`Sprite.material: Option<MaterialRef>`** | Hypothetical | Component opcional preserva POD enxuto; ausência = usa default |
| **`Sprite.z_index: i32`** | Godot CanvasItem | Não distingue "DFS" de "z=0 explícito" |
| **`image_blend`/`image_angle`/...` no Sprite struct** | GameMaker | Sopa, type-unsafe, schema versioning impossível |
| **9-Slice como Object separado** | Construct 3 | Duplica registry; PH2D usa `SliceNine` Component anexável |
| **Mixins acumulados (`with Alpha, with Tint, with Mask, ...`)** | Phaser | State-stuffed; Inspector vira sopa |
| **Sprite carrega FX chain inline** | Hypothetical | FX é módulo separado; FXChain vira Component opcional anexável |
| **`Sprite.skew_x/y` no struct** | Não-observado | Skew é decomposição da matriz 2D, não da imagem; vive em `Transform` |

### 2.8 Implicação para o asset cooker

Sprite POD + Components ECS opcionais = serialização HÍBRIDA:

```
.ph2d-scene.postcard
└── Entity { 
      Sprite { ... POD v4 ... },                  // sempre presente
      Transform { ... },                          // sempre presente
      Visibility { ... },                         // sempre presente (default)
      Name(String),                                // sempre presente (default)
      ZIndexOverride(5),                           // opcional — só serializa se presente
      MaskInteraction { mode, alpha_cutoff },     // opcional
      // ...
    }
```

Cada Component opcional é serializado **só se anexado**. Sprite POD ocupa ~80 bytes; com 5 Components opcionais médios, ~150 bytes/entity total.

### 2.9 Implicação para fan-out paralelo (DIRETRIZ §3.A)

Adicionar Component novo (e.g., `ClipChildren` em wave futura) = **drop-crate isolado** (sessão Implementador-só) que NÃO toca arquivos centrais. Se fosse campo do `Sprite`, cada feature ortogonal viraria PR no `Sprite` struct = serial.

Esta ADR **habilita** fan-out paralelo W2-W6 do Sprite Inspector v2 (vide plano §15.1).

### 2.10 Implicação para LLM/MCP

Cada Component ECS exposto via `#[lua_export]` → MCP toolset `sprite_component_*`:
- `sprite_attach_component(entity, "ClipChildren", {mode: "ClipOnly"})`
- `sprite_remove_component(entity, "ClipChildren")`
- `sprite_query_components(entity)` → [ComponentName]

HR-10 + HR-11 (destructive ops com confirmation token).

### 2.11 Caps congelados

Arch-gate `architecture_sprite_inspector_surface`:

| Cap | Valor |
|---|---|
| `Sprite` struct fields | **20 (FROZEN v4)** — Lens C M1 reconciliado |
| Total Components opcionais para Sprite Inspector | **≤ 32** |
| Cap individual de cada Component fields | seguir HR-18 + caps específicos por ADR |

Bump → ADR-amendment.

---

## 3. Consequências

### 3.1 Positivas

- **Regra explícita** previne discussões cíclicas em PRs futuros ("isto é campo ou Component?").
- **Anti-padrões banidos** capturam erros conhecidos de Godot/Unity/Phaser/GameMaker.
- **Fan-out paralelo habilitado** — cada Component novo é drop-crate (W2-W6 paralelo).
- **Sprite struct enxuto** preservado long-term — não vira sopa.
- **LLM-friendly** — Components ECS são lookup table; lua_export trivial.

### 3.2 Negativas

- **20 campos no Sprite + ~20 Components opcionais** = surface grande pra documentar e auditar. Mitigação: spec gêmea + ADRs separadas por concern.
- **Decisão "campo vs Component" pode ser contestável** em casos limite (skew, flip, opacity). ADR documenta cada caso ambíguo (§2.6 + spec §02 §2.4 "Casos-limite").

### 3.3 Neutras

- **Princípio operacional** = não bloqueia; só guia decisões.
- **HR-18 + caps** já existiam; esta ADR consolida.

---

## 4. Alternativas consideradas

### 4.1 Sprite struct gigante (god-struct) — rejeitada

Acumular tudo no struct. **Por que rejeitada:** anti-pattern observado em GameMaker; schema bump custa; sopa visual no Inspector; impede fan-out paralelo.

### 4.2 Sprite ZST + tudo Components — rejeitada

Sprite seria marker zero-size; toda propriedade em Component. **Por que rejeitada:** aparência intrínseca (tint, opacity, anchor) precisa de Component em CADA sprite (95% dos casos) — overhead inútil. Padrão-ouro: campos intrínsecos no struct.

### 4.3 Componentes "fat" vs "thin" — neutro

Granularidade dos Components: 1 Component "Sorting" com {z, layer, ysort, group, behind} vs 5 Components separados. **Decisão híbrida:** Components agrupados POR CONCERN (`YSort { enabled, axis, sort_point }` é 1 Component porque os 3 campos co-variam; mas `ZIndexOverride`, `SortingLayer`, `ShowBehindParent`, `TopLevel` são markers separados porque ausência ≠ default).

### 4.4 Documentação implícita (sem ADR explícita) — rejeitada

Confiar que devs vão "inferir" o princípio. **Por que rejeitada:** discussões cíclicas em PRs já aconteceram em outros projetos; ADR explícita previne.

---

## 5. Implementação

Esta ADR é **principiológica, não tem código**. Implementação espelha em:
- [`docs/Sprite_projeto/01_anatomia_canonica.md §1.3`](../../Sprite_projeto/01_anatomia_canonica.md) — campos que NÃO vão no Sprite.
- [`docs/Sprite_projeto/02_components_ortogonais.md`](../../Sprite_projeto/02_components_ortogonais.md) — matriz canônica completa + razão.
- ADR-0070 — schema Sprite struct.
- Cada wave (W2-W6) implementa Components segundo esta ADR.

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| Caso limite: `BlendMode` no Sprite ou Component? | **Component** — nem todo sprite muda blend mode; default Mix é benigno; Component preserva POD enxuto. |
| Caso limite: `centered` é "ortogonal opcional"? | **Não — campo do Sprite.** TODO sprite tem origem; default true; sem ambiguidade ausência/presença. |
| Caso limite: `frame: u32` no Sprite vs `SpriteAnimator.frame`? | **Sprite carrega frame ATUAL** (sprite-sheet inline); `SpriteAnimator` é estado runtime de animação (pode override). Sem conflict — animator escreve em Sprite.frame a cada tick. |
| Adicionar Component vs amendment ADR? | Component novo SE não conflita com cap (`≤ 32 Components`) — sem amendment. Bump de cap → amendment. |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/02_components_ortogonais.md`](../../Sprite_projeto/02_components_ortogonais.md).
- ADR pais: [ADR-0069](0069-sprite-inspector-v2.md), [ADR-0025](0025-gameobject-model.md), [ADR-0025-amendment-1](0025-amendment-1.md).
- DIRETRIZ §3.A (drop-crate fan-out): [docs/IntegracaoMultiAgente/DIRETRIZ.md](../../IntegracaoMultiAgente/DIRETRIZ.md).
- Bevy ECS: <https://bevyengine.org/learn/quick-start/getting-started/ecs/>.
- Anti-pattern GameMaker `image_*`: <https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Sprites/Sprite_Instance_Variables/Sprite_Instance_Variables.htm>.
- Anti-pattern Phaser mixins: <https://docs.phaser.io/api-documentation/namespace/gameobjects-components-tint>.
- Anti-pattern Construct 9-Patch separate object: <https://www.construct.net/en/make-games/manuals/construct-3/plugin-reference/9-patch>.
