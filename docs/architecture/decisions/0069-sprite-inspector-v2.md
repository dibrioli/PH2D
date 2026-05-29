# ADR-0069 — Sprite Inspector v2 (decisão-mãe)

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais (147 findings, 31 CRITICALs fechados a erro-zero).
**Decisor(es):** Enio + Claude (sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0021 — SimWorld/PresentWorld](0021-simulation-presentation-boundary.md), [ADR-0022 — No HashMap in simulation](0022-no-hashmap-in-simulation.md), [ADR-0023 — UI/UX baseline 4 zonas](0023-ui-ux-baseline.md), [ADR-0025 — GameObject model (Transform component)](0025-gameobject-model.md), [ADR-0029 — Trait-driven panel host](0029-trait-driven-panel-host.md).
**Sub-contratos congelados por ADRs irmãs:** 0070 (Sprite schema v4), 0071 (Tint channels), 0072 (Named Anchor unification), 0073 (Sorting canonical order), 0074 (Sprite-vs-Component boundary).
**Spec normativa:** [`docs/Sprite_projeto/`](../../Sprite_projeto/) — 16 arquivos (README + 14 seções + 15_plano).
**Tags:** sprite, wave-0, contract, inspector, foundational, padrão-ouro

---

## 1. Contexto

O Sprite é (provavelmente) o objeto **mais importante** de jogos 2D. Toda Image Tool edita Sprites; toda animação anima Sprites; toda hierarquia organiza Sprites. Sprite ruim = atrito permanente. Sprite bom = base de produção fluida.

O Sprite atual ([crates/ph2d-render/src/sprite.rs](../../../crates/ph2d-render/src/sprite.rs), v3) carrega 5 campos canônicos (source, size, tint, anchor, premultiplied). O Inspector atual ([crates/ph2d-panel-inspector/](../../../crates/ph2d-panel-inspector/)) tem 4 seções (Name, Visibility, Transform, Render Source).

Pesquisa de estado-da-arte (4 agentes paralelos, 2026-05-27, [docs/Sprite_projeto/13_referencias.md](../../Sprite_projeto/13_referencias.md)) catalogou 110+ features candidatas em 8 engines (Godot 4 · Unity 2D / URP · Unreal Paper 2D · Defold · GameMaker · Construct 3 · LÖVE · Phaser 3 · Aseprite) + levantamento de comunidade (Godot Proposals, Unity Discussions, Reddit, GitHub Issues). Achados:

1. **Lacunas universais** que NENHUM engine resolve completamente — Self Tint independente (só Godot), per-vertex tint (só Phaser), Named Anchors unificados (Paper2D só socket; Aseprite só slice; Construct só image_point — nunca os 3 num único conceito).
2. **Features pedidas-e-não-entregues há anos** — Godot Proposal #4282 (Mask2D), #9222 (custom pivot origins), #10937 (frame-specific offsets), #14098 (sockets em AnimatedSprite2D), #567 (AnimationTree ⇄ AnimatedSprite2D).
3. **Bugs crônicos** — Godot ClipChildren regrediu 5× em releases sucessivas (#79885, #102190, #102224, #91068, #90793).
4. **Padrões mal-feitos a evitar** — GameMaker `image_*` instance variables = sopa; Phaser mixins = state-stuffed; Construct 9-Patch como objeto separado de Sprite = duplica registry.

Antes de tocar código (W1+), **a anatomia do Sprite struct + a fronteira Sprite-vs-Component-ECS + o layout do Inspector + a matemática dos canais de tint + a ordem de sorting precisam estar congeladas** porque:

1. **Schema bump v3 → v4 toca foundational.** `crates/ph2d-render/src/sprite.rs` é foundational (Coord-A only). Cada mudança no schema = migrator obrigatório (HR-14) + ABI `RenderInstance` bump + gate `vertex_attr_offsets_match_struct`. Sem padrão-ouro na W0, ripple em cada wave futura.

2. **Múltiplas waves paralelas vão expandir o Inspector.** W2 adiciona Color & Tint; W3 adiciona Sorting + Visibility; W5 adiciona Named Anchors. Sem fronteira fixa Sprite-vs-Component, cada wave duplica decisão "isto é campo do struct ou Component?".

3. **Fan-out paralelo só funciona com escopo definido.** Cada Component ECS novo (~20 candidatos: ZIndexOverride, SortingLayer, SliceNine, NamedAnchorList, etc.) pode virar drop-crate isolado (DIRETRIZ §3.A) DESDE QUE a interface com `Sprite` esteja congelada.

4. **Padrão-ouro absoluto** ([feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)). Sprite é objeto central; precedente Painter (11 ADRs Accepted) + Vector Module (13 ADRs Accepted) estabelece a barra.

### 1.1 O que diferencia esta ADR

- **Não é Tool isolado** (ADR-0040 não aplica — Sprite não é Tool, é entidade ECS canônica que toda Tool edita).
- **Não é Nó do grafo** (ADR-0039 não aplica — Sprite vive em SimWorld, não em `ph2d-nodegraph`).
- **Não é Painter / Vector / Image Tool** — é o **objeto comum** que todos editam.
- **É foundational** — mexe em `ph2d-render` + `ph2d-ecs` + `ph2d-panel-inspector`.

### 1.2 Por que sub-contratos separados (0070..0074)

Esta ADR fixa **só o escopo + decisão-mãe + lista das 12 seções**. Sub-sistemas adjacentes têm ADRs próprias:

- **ADR-0070** — schema v4 (`Sprite::VERSION`, fields, ABI bump, migrator).
- **ADR-0071** — matemática multiplicativa dos 4 canais de tint.
- **ADR-0072** — `NamedAnchor` unification (socket + slice + image_point).
- **ADR-0073** — pipeline canônico de sorting (Z + YSort + SortingGroup + ShowBehindParent + DFS).
- **ADR-0074** — princípio operacional Sprite struct vs Component ECS.

Razão: cada sub-contrato tem evolução distinta. Adicionar 12ª seção no Inspector não pode forçar bump em `Sprite::VERSION`. Mudar matemática de tint não pode forçar revisão de NamedAnchor.

---

## 2. Decisão

### 2.1 Escopo IN — **12 seções canônicas** do Inspector do Sprite v2

1. **Identity** — Name · Tags · Notes
2. **Transform** — Position · Rotation · Scale · Skew X/Y · Top Level · Reset · Look At
3. **Render Source** (ampliada) — Strategy · Storage detail · Source W×H · Region toggle + Rect · Region Filter Clip · Pixel Format · Reimport
4. **Sprite Sheet** (inline) — Centered · Offset · Flip H/V · H/V Frames · Frame · Frame Coords
5. **9-Slice** — Draw Mode · Borders L/T/R/B · Size · Per-region tile mode (8 regiões) · Tile Mode · Stretch Value · Fill Center
6. **Color & Tint** — Tint · Self Tint · Per-corner tint · Tint Fill · Opacity
7. **Ordering / Sorting** — Z Index · Z as Relative · Show Behind Parent · Sorting Layer · Order in Layer · Y Sort · Sorting Group · Sort At Root · Translucency Priority + Distance Offset · Order Debug Overlay
8. **Visibility** — Visible · Visibility Layer (bitmask) · Clip Children mode · Mask Interaction · Alpha Cutoff · On-Screen Enabler
9. **Sampling** — Texture Filter · Texture Repeat · Anti-halo (read-only)
10. **Material & Blend** — Material slot · Use Parent Material · Instance Shader Params · Blend Mode (6 modos)
11. **Animation** (collapsible) — SpriteFrames · Current Animation (tag) · Frame · Frame Progress · Speed Scale · Playing · Autoplay · Direction · Loop · Hold ms · Repeat Delay
12. **Sockets / Slices** (Named Anchors) — lista NamedAnchor + per-frame override + visual handles

**Cap arch-gate** (`inspector_section_count_canonical`): **12 FROZEN**. Pre-W0 audit corrigiu "11" inconsistency.

Detalhamento de cada seção em [`docs/Sprite_projeto/03_inspector_secoes.md`](../../Sprite_projeto/03_inspector_secoes.md).

### 2.2 Escopo OUT — explicitamente fora

Vide [`docs/Sprite_projeto/12_fora_de_escopo.md`](../../Sprite_projeto/12_fora_de_escopo.md). Resumo:

- **FX / Shader chain** → módulo Shader FX dedicado, futuro.
- **Lighting 2D + normal maps + LightMask + ShadowCaster** → módulo Lighting dedicado, futuro.
- **Física / Collision geometry** → módulo Física dedicado, futuro.
- **Onion skin** → timeline editor (módulo Animation futuro).
- **Pixel-perfect camera** → módulo Camera dedicado, futuro.
- **Frame events com payload tipado** → timeline editor.
- **SpriteShape (spline terrain)** → subsistema próprio.
- **PSD/Aseprite full import** → asset cooker.
- **Hot-reload runtime** → app-level config.

### 2.3 Princípio Sprite-vs-Component-ECS

Detalhado em ADR-0074. Resumo:

```
APARÊNCIA INTRÍNSECA da imagem  →  Sprite struct (POD, schema versionado)
ASPECTO ORTOGONAL opcional      →  Component ECS anexável (ausência ≠ default)
DERIVADO por sistema/grafo      →  Sistema ECS extract OU nó do grafo
```

Anti-padrões evitados:
- ❌ GameMaker `image_*` instance variables (sopa)
- ❌ Phaser mixins acumulados (state-stuffed)
- ❌ Construct 9-Patch como objeto separado de Sprite (duplica registry)
- ❌ `Sprite.material: Option<MaterialRef>` no struct (Component opcional resolve)

### 2.4 Os 8 itens "pequenos com impacto desproporcional"

4 pares complementares + 4 toggles minúsculos. **Diferenciam "Inspector bom" de "padrão-ouro":**

**Pares complementares:**
1. **`tint` + `self_tint`** — herda vs não-herda (Godot acerta; ninguém mais expõe os 2).
2. **`z_index` + `z_as_relative`** — absoluto vs hierárquico (Godot acerta).
3. **`centered` + `offset`** — origem no centro vs offset arbitrário (Godot acerta).
4. **`tint` (flat) + `per-corner tint`** — flat vs gradient (só Phaser tem; ninguém mais).

**Toggles minúsculos:**
5. **Show Behind Parent** — organização hierárquica sem reordenar tree.
6. **Top Level** — quebra cascata de transform/modulate sem reparentar.
7. **Use Parent Material** — batching brutal (10k filhos = 1 material instance = 1 draw call).
8. **Region Filter Clip** — sampler trava no rect; anti-bleed industrial em atlas.

### 2.5 Inovações 🆕 PH2D no Inspector do Sprite v2

1. **OKLCH ColorPicker** nativo (web tem desde 2023 — Chrome 111+, Safari 15.4+, Firefox 113+, Edge 111+; engines de jogo não).
2. **Order Debug Overlay** built-in (color por sorting layer, depth labels) — só web tem (Edge DevTools 3D View pra CSS z-index).
3. **NamedAnchor unificado** (socket Paper2D + slice Aseprite + image_point Construct num único tipo).
4. **AnimationTree ⇄ SpriteAnimator** unificados desde dia 1 (Godot Proposal #567 aberto há anos).
5. **Bulk-edit multi-select inspector** first-class (Unity Discussion `multi-select editing` aberto há anos).

> Inovações de fora do Inspector (Hot-reload runtime, Undo granular global) NÃO estão na lista — vivem em módulos próprios da app (vide [`docs/Sprite_projeto/12_fora_de_escopo.md`](../../Sprite_projeto/12_fora_de_escopo.md) §12.9 Hot-reload OUT). Pre-W0 audit corrigiu inconsistência prévia.

### 2.6 Roadmap waves (7 waves)

Detalhado em [`docs/Sprite_projeto/15_plano_de_implementacao.md`](../../Sprite_projeto/15_plano_de_implementacao.md):

- **W0** — Spec freeze + 6 ADRs (estamos aqui).
- **W1** — Schema bump strategic-only (`Sprite::VERSION=4` + ABI + migrator).
- **W2** — Inspector seções 1-6 + OKLCH ColorPicker.
- **W3** — Seções 7-9 (Sorting · Visibility · Sampling) + ClipChildren regression gate.
- **W4** — Seções 10-11 (Material&Blend · Animation).
- **W5** — Seção 12 (Named Anchors).
- **W6** — Foundational widgets novos.
- **W7** — Polish + i18n + a11y + bug bash.

---

## 3. Consequências

### 3.1 Positivas

- **Inspector do Sprite v2 = padrão-ouro absoluto** — supera Godot/Unity/Unreal/Paper2D/Defold/GameMaker/Construct/Phaser/Aseprite combinados nos pontos identificados.
- **Schema v3 → v4** com `#[serde(default)]` + migrator: back-compat preservada; sem quebra de save files.
- **Fan-out paralelo W2-W6** habilitado pela fronteira Sprite-vs-Component fixa.
- **Multi-engine pesquisa absorvida** — sem reinventar o que outros já fazem bem; sem repetir os erros (GameMaker sopa, Phaser state-stuffed, Construct duplicação).
- **8 "itens pequenos com impacto desproporcional"** explicitados como pillars — gate visual em cada wave verifica.
- **Order Debug Overlay** e **OKLCH ColorPicker** são features 🆕 raras que diferenciam PH2D.

### 3.2 Negativas

- **RenderInstance v4 ABI ~144 bytes** (era 72) — dobra upload bandwidth do instance buffer. Mitigação opcional dual-buffer documentada em ADR-0070.
- **`Sprite` struct v3 → v4 com 13 campos novos** — schema maior; migrator obrigatório.
- **12 seções no Inspector** — UI mais densa. Mitigação: 7 colapsáveis por default.
- **Arch-gate `architecture_sprite_inspector_surface`** novo — força disciplina (cap **20 fields FROZEN**).

### 3.3 Neutras

- **Spec gêmea (16 arquivos `docs/Sprite_projeto/`)** preserve sincronizada com este ADR via cross-references.
- **Memory budget** sem mudança significativa.
- **6 ADRs novos** (0069-0074) — adiciona governance, mas é padrão Painter/Vector.

---

## 4. Alternativas consideradas

### 4.1 Estender Sprite v3 in-place (sem bump) — rejeitada

Adicionar novos campos sem bump de `VERSION`. **Por que rejeitada:** quebra HR-14 (migrator obrigatório); save files antigos viram inválidos silenciosamente. Bump v3 → v4 com `#[serde(default)]` é estritamente melhor.

### 4.2 Todos os campos novos como Components ECS opcionais (zero campo novo no Sprite) — rejeitada

Manter Sprite v3 5-field; tudo novo vira Component (tint per-corner, opacity, flip, etc.). **Por que rejeitada:** propriedades intrínsecas da imagem (tint, opacity, flip) precisariam de Component para CADA sprite (95% dos casos) — overhead inútil. Padrão-ouro: campos intrínsecos no struct; aspectos ortogonais como Components.

### 4.3 Inspector com 20+ seções (uma por feature) — rejeitada

Cada feature ortogonal vira seção própria. **Por que rejeitada:** Inspector vira sopa visual (Phaser mixins anti-pattern). 12 seções canônicas + 7 colapsáveis por default é o ponto certo.

### 4.4 Sprite carrega FX chain inline — rejeitada

Adicionar `fx_chain: Vec<FXPass>` ao Sprite struct. **Por que rejeitada:** FX é arquitetura inteira (30+ effects, pipeline pre/postFX, shader graph). Módulo Shader FX dedicado é separação correta. Inspector v2 não bloqueado por isso.

### 4.5 Postcard schema sem versionamento explícito — rejeitada

Confiar que `#[serde(default)]` cobre back-compat. **Por que rejeitada:** alguns campos têm lógica condicional (`region_filter_clip = (source == Atlas)`); migrator explícito é necessário. Hybrid `#[serde(default)]` + migrator é canônico.

---

## 5. Implementação (Wave 0 → Wave 7)

Tasks T-W.N materializando esta ADR vide [`docs/Sprite_projeto/15_plano_de_implementacao.md`](../../Sprite_projeto/15_plano_de_implementacao.md).

W0 fechada quando:
- 6 ADRs Accepted (0069-0074).
- Spec completa (16 arquivos `docs/Sprite_projeto/`).
- Auditoria adversarial ≥2 lentes rotacionadas com erro-zero.
- Ratificação Enio.

---

## 6. Open questions resolved during W0

| Q | Resolução |
|---|-----------|
| Skew vai no `Sprite` ou no `Transform`? | **`Transform`** — skew é decomposição da matriz 2D, não da imagem. ADR-0025 amendment via ADR-0070. |
| Per-corner tint vai no `Sprite` ou em Component? | **Sprite struct** — aparência intrínseca; default WHITE × 4 = zero overhead; vertex color ABI sempre presente. |
| Self Tint vai no `Sprite` ou em Component? | **Sprite struct** — aparência intrínseca; default WHITE; campo simples é estritamente mais expressivo. |
| Z Index vai no `Sprite` ou em Component? | **Component opcional** (`ZIndexOverride`) — ausência ≠ "Z=0 explícito"; DFS counter fallback determinístico. |
| Opacity separada de tint.a? | **Sim** — opacity é visibility multiplier; tint.a é blend channel; animáveis independentemente. |
| FX chain está nesta ADR? | **Não** — vai pro módulo Shader FX dedicado futuro. Inspector v2 v1.0 fora de escopo. |
| Onion skin está nesta ADR? | **Não** — vive no timeline editor (módulo Animation futuro). Inspector v2 só mostra estado atual. |
| Render Geometry choice (Diced/ShrinkWrapped) está nesta ADR? | **Não** — vai pro asset cooker. Não Inspector. |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/`](../../Sprite_projeto/) — 16 arquivos canônicos.
- Levantamento multi-engine: [`docs/Sprite_projeto/13_referencias.md`](../../Sprite_projeto/13_referencias.md).
- 4 agentes paralelos pesquisa (2026-05-27): Godot 4 · Unity 2D/URP · Unreal Paper 2D + Defold + GameMaker + Construct + LÖVE + Phaser + Aseprite · Community forums + GitHub Issues + Reddit + Godot Proposals.
- ADR Painter precedente: [ADR-0043](0043-painter-contract.md).
- ADR Vector Module precedente: [ADR-0056](0056-vector-network-data-model.md).
- Memory: [feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md), [feedback-audit-lens-diversity](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md).
