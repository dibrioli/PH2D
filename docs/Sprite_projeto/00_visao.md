# 00 — Visão executiva

## A frase

> Um Inspector de Sprite que casa **profundidade-de-Godot** com **per-vertex tint do Phaser**, **Self Tint independente** e **named anchors unificados** (socket Paper2D · slice Aseprite · image_point Construct) — mantendo `Sprite` struct enxuto (POD, schema versionado) e tudo ortogonal como Component ECS opcional.

## A pergunta que esta spec responde

**"Quando o artista seleciona um Sprite na cena, o que vê e edita no Inspector?"**

Resposta: **12 seções canônicas** (Identity · Transform · Render Source · Sprite Sheet · 9-Slice · Color & Tint · Ordering / Sorting · Visibility · Sampling · Material & Blend · Animation · Sockets/Slices Named Anchors) — cobrindo ~70 propriedades discretas que mapeiam para o `Sprite` struct (POD intrínseco) **ou** Components ECS opcionais (aspectos ortogonais).

## Por que isto importa

O Sprite é (provavelmente) o objeto mais importante de jogos 2D. Toda Image Tool edita Sprites; toda animação anima Sprites; toda hierarquia organiza Sprites. Inspector ruim de Sprite = atrito permanente para toda a sessão de produção. Inspector bom = base de produção fluida.

Engines existentes erram em:
- **Godot:** `clip_children` regrediu 5× em releases sucessivas; AnimatedSprite2D não fala com AnimationTree (Proposal #567 aberta há anos); `centered` é bool obtuso.
- **Unity:** sem `self_modulate` (só `Color` que herda); 9-Slice exige `Mesh Type=Full Rect` sem erro claro; `Pivot` per-sprite só no importer (não runtime).
- **Unreal Paper 2D:** Sockets brilhantes, mas o resto do Inspector é stale.
- **Phaser:** per-corner tint maravilhoso, mas é state-stuffed nos mixins.
- **GameMaker:** `image_blend`/`image_angle`/`image_alpha` como variáveis de instância = sopa.

PH2D corrige cada um desses, num único Inspector coerente.

## As 4 decisões estruturais

1. **`Sprite` struct enxuto, POD, schema versionado** ([01_anatomia_canonica.md](01_anatomia_canonica.md)) — só aparência intrínseca da imagem. Bumpa de v3 → v4 com novos campos `#[serde(default)]`.
2. **Tudo ortogonal vira Component ECS opcional** ([02_components_ortogonais.md](02_components_ortogonais.md)) — ausência ≠ default; Components anexáveis dão override semântico (`ZIndexOverride` ≠ "Z=0 explícito").
3. **4 canais de tint independentes, matemática multiplicativa canônica** ([04_color_tint_canais.md](04_color_tint_canais.md)) — Tint (herda) · Self Tint (local) · Per-corner (gradient) · Opacity (final).
4. **Named Anchor unifica 3 conceitos** ([07_named_anchors.md](07_named_anchors.md)) — socket (transform sem bounds) · slice (transform com bounds) · 9-slice region (transform com bounds + center) — num único tipo.

## Os 8 itens "pequenos com impacto desproporcional"

Quatro pares complementares + quatro toggles minúsculos. A diferença entre "Inspector bom" e "padrão-ouro" mora aqui:

### Pares complementares
1. **`tint` + `self_tint`** — herda vs não-herda. Você precisa dos dois (Godot acerta; ninguém mais expõe).
2. **`z_index` + `z_as_relative`** — absoluto vs hierárquico. Toggle decisivo (Godot acerta).
3. **`centered` + `offset`** — origem no centro vs offset arbitrário. Anchor logic separado do scene logic (Godot acerta).
4. **`tint` (flat) + `per-corner tint`** — flat vs gradient. Phaser tem; ninguém mais.

### Toggles minúsculos
5. **Show Behind Parent** — organização hierárquica sem reordenar tree.
6. **Top Level** — quebra cascata de transform/modulate sem reparentar.
7. **Use Parent Material** — batching brutal (10k filhos = 1 material instance = 1 draw call).
8. **Region Filter Clip** — sampler trava no rect; anti-bleed industrial em atlas.

## Os 5 cuidados estruturais

1. **God-struct anti-pattern.** `Sprite` NUNCA cresce além de aparência-intrínseca-universal. GameMaker virou sopa (`image_blend`/`image_angle`/`image_alpha`/`image_xscale`); Phaser virou mixins (Alpha/Tint/Crop/Mask) acumulados. PH2D resiste.
2. **Schema versionado v3 → v4 desde dia 1.** Migrator obrigatório; back-compat via `#[serde(default)]`. Sem isso, save-files quebram entre versões (HR-14).
3. **`RenderInstance` ABI sensível.** Per-corner tint adiciona 16 bytes; passa em `vertex_attr_offsets_match_struct` (gate existente em [tests/sprite.rs](../../crates/ph2d-render/src/sprite.rs#L343-L375)).
4. **Order Debug Overlay como gate visual.** Pipeline de ordenação tem 7 estágios; sem visualização, regressões viram bugs de 2-3 horas de debug. Overlay built-in = gate de regressão grátis.
5. **`ClipChildren` não pode regredir.** Godot teve 5 issues abertos sucessivos. PH2D adiciona arch-test específico + smoke obrigatório em cada wave que toca a feature.

## Diferenciais sobre o estado da arte

| Feature | Engines com | Engines sem | PH2D |
|---|---|---|---|
| Self Tint independente | Godot | Unity, Unreal, Phaser | ✓ |
| Per-corner tint | Phaser | Unity, Godot, Unreal, GameMaker | ✓ |
| Skew nativo | Godot | Unity, GameMaker | ✓ |
| Z Index + Z as Relative | Godot | Unity (só Order in Layer) | ✓ |
| Show Behind Parent | Godot | Unity, Unreal | ✓ |
| Texture Filter per-node hierarchical | Godot | Unity, GameMaker | ✓ |
| Use Parent Material | Godot | Unity (limitado) | ✓ |
| Region Filter Clip | Godot | Unity, Phaser | ✓ |
| Named Anchors unificados | nenhum (Paper2D só socket, Aseprite só slice, Construct só image_point) | todos os outros | 🆕 PH2D |
| OKLCH color picker | nenhum | todos | 🆕 PH2D |
| Order Debug Overlay | nenhum (web tem em CSS) | todos | 🆕 PH2D |
| AnimationTree ⇄ SpriteAnimator unificado | nenhum (Godot proposal aberto) | todos | 🆕 PH2D |

## A consequência operacional

Após W0 ratificada (6 ADRs Accepted), W1 pode rodar em **fan-out paralelo** (cada Component ECS novo = um Implementador), porque o `Sprite` struct + matemática dos canais de tint + ordering pipeline + NamedAnchor schema estão congelados.

Sem padrão-ouro na W0, cada wave futura ripple back em todas as anteriores — exato anti-padrão que ADR-0039 (nodegraph FREEZE) e ADR-0040 (tool FREEZE) impedem em outros contratos PH2D.
