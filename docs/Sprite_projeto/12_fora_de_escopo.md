# 12 — Fora de escopo (não-objetivos explícitos)

> Cada "não" aqui é **decisão consciente**, não esquecimento. Razões documentadas; reabrir um destes itens exige **ADR novo** estendendo 0069 OU módulo dedicado próprio.

> **Filtro:** Inspector do Sprite v2 cobre propriedades canônicas editáveis pelo artista, num único Inspector coerente, mantendo `Sprite` struct enxuto (POD). Tudo que excede esse escopo vai pra outros módulos.

## 12.1 FX / Shader chain por-sprite

**Decisão:** **OUT**. Vai pro **módulo Shader FX** dedicado, futuro.

**O que isso significa estar OUT:**
- Outline / contorno automático
- Drop shadow automático
- Palette swap / colorização dinâmica
- Dither
- Glow / Bloom per-sprite
- Blur per-sprite
- Pixelate
- Distortion / wobble / wave
- Chromatic aberration
- Hue Shift / Saturation / Brightness / Contrast
- Vignette
- Mosaic
- Liquify per-sprite
- FX chain composable (Phaser pre/postFX style)

**Razão:**
1. FX são uma família grande e ortogonal (~30+ efeitos). Cada um tem parâmetros próprios; juntos formam uma chain composável (Phaser pre/postFX). Módulo dedicado é arquiteturalmente mais limpo.
2. Inspector do Sprite ficaria poluído se cada FX virasse uma seção/checkbox. Seria 20+ seções extra.
3. FX dependem de pipeline render mais sofisticado (pre/post FBO, pass ordering, shader graph). Inspector v2 não precisa esperar.
4. Padrão Phaser confirma viabilidade: `sprite.preFX.addGlow()` API separada do Inspector base.

**Quando talvez:** módulo `Shader_FX_projeto/` dedicado, com FXChain Component anexável ao Sprite. Inspector v2 adicionará uma nova seção "Effects" SÓ se FXChain Component presente — esta seção será gerenciada pelo módulo Shader FX, não duplicada aqui.

**Mas no Inspector v2:** Material slot existe (seção 10), permite shader custom único per-sprite. Não é "FX chain", é "1 material override". Cobre casos simples (sprite-único com shader water).

## 12.2 Sistema de luzes e sombras 2D

**Decisão:** **OUT**. Vai pro **módulo Lighting 2D** dedicado, futuro.

**O que isso significa estar OUT:**
- `Light2D` component (Point/Directional/Sprite/Spot/Global)
- Normal maps + Specular maps per-sprite (`CanvasTexture` style)
- `LightMask` bitmask
- `Shadow Caster 2D` (occluder polygons)
- `Light Blend Styles` (4 channels R/G/B/A)
- Light range Z/Layer
- `light_mode` (Normal/Unshaded/LightOnly)
- Secondary Textures named (`_NormalMap`, `_MaskTex`)
- Composite Shadow Caster 2D

**Razão:**
1. Sistema de luz é arquitetura inteira (gathering lights per-cluster, shadow pass, normal map sampling, blend styles). Inspector v2 NÃO depende.
2. Lighting tem proprio Inspector (Light2D entity, Shadow Caster entity). Sprite Inspector apenas anexa `MaskInteraction` (mask) e `Material` (shader, que pode ser lit-aware). Não cabe expor 10 propriedades de lighting per-sprite.
3. Padrão Godot/Unity: Light2D são entidades separadas; Sprite apenas tem `light_mask` bitmask (qual luz afeta). PH2D futuro segue.

**Quando talvez:** módulo `Lighting_2D_projeto/` dedicado. Inspector v2 adicionará Component opcional `LightMask(u32 bitmask)` quando módulo lighting existir — vai aparecer como sub-seção em "Visibility" ou nova seção "Lighting".

## 12.3 Física / collision geometry

**Decisão:** **OUT**. Vai pro **módulo Física** dedicado, futuro.

**O que isso significa estar OUT:**
- Custom Physics Shape (Unity Sprite Editor)
- Collider 2D auto-gen do sprite alpha
- Collision masks (GameMaker 7 modos: Rectangle, Rotated Rect, Ellipse, Diamond, Precise, Precise per-frame, User-defined)
- Render Geometry ≠ Collision Geometry (Paper2D `GeometryType` enum: Diced, ShrinkWrapped, etc.)
- Collision mask types separadas do visual mesh
- Trigger shapes
- Layer masks de colisão

**Razão:**
1. Física é sistema próprio (Rapier 2D integration, collision detection, response). Inspector v2 NÃO depende.
2. Collider 2D são entidades-componentes próprias (Collider, RigidBody, Sensor). Sprite apenas tem o gizmo visual; collision lives in physics layer.
3. Render Geometry (Diced, ShrinkWrapped) é optimization que vai no asset cooker, não Inspector runtime.

**Quando talvez:** módulo `Fisica_2D_projeto/` dedicado, com Components `Collider2D`, `RigidBody2D`, etc. Sprite Inspector NÃO ganha seção física — usuário adiciona Collider2D Component separadamente.

## 12.4 Onion skin

**Decisão:** **OUT do Inspector**. Vai pra **timeline editor** (módulo Animation futuro).

**Razão:** Onion skin mostra contexto temporal (frame anterior + frame seguinte em transparência). Inspector mostra estado atual. Conceitos ortogonais.

Timeline editor é entidade UI separada (panel docado próprio, NÃO Inspector). Inspector seção Animation só mostra `frame`, `frame_progress`, `current_animation` — estado pontual, não contexto.

**Quando talvez:** módulo `Animation_Timeline_projeto/` dedicado, panel próprio. Inspector adicionará botão "Open in Timeline" na seção Animation.

## 12.5 Pixel-perfect camera + sub-pixel smoothing

**Decisão:** **OUT**. Vai pro **módulo Camera 2D** dedicado, futuro.

**Razão:** Pixel-perfect é propriedade da **Camera2D / Viewport**, não do Sprite. Camera define resolução-alvo + upscale render texture + snap. Sprite herda comportamento da camera.

Inspector do Sprite NÃO tem campos relacionados (nenhuma engine séria coloca lá; sempre na Camera).

**Quando talvez:** módulo `Camera_2D_projeto/` dedicado. Camera Inspector terá esses campos. Sprite Inspector continua agnostic.

## 12.6 Frame events com payload tipado

**Decisão:** **OUT do Inspector**. Vão pra **timeline editor** (módulo Animation futuro).

**Razão:** Frame events (footstep SFX no frame 3 com `surface_type` payload) são keyframes da timeline, não estado runtime do sprite. Inspector mostra ESTADO; timeline mostra CONTEXTO temporal + eventos.

Inspector v2 emite signals BÁSICOS de transição (`SpriteFrameChanged`, `SpriteAnimationFinished`, `SpriteAnimationLooped`) — vide [08_animation_inline.md §8.10](08_animation_inline.md). Frame events COM PAYLOAD são feature do módulo Animation.

**Quando talvez:** módulo `Animation_Timeline_projeto/`. Cada frame de `SpriteFrames` ganhará `events: Vec<FrameEvent>` com payload struct tipado. Será editado no timeline editor, não no Inspector.

## 12.7 SpriteShape / spline-based terrain

**Decisão:** **OUT**. Subsistema próprio, não Inspector.

**Razão:** SpriteShape (Unity, Cavalry) gera geometria via spline + Angle Ranges → sprite swap por ângulo. Não é "sprite isolado"; é renderer especializado. Inspector do Sprite v2 NÃO depende.

**Quando talvez:** módulo `SpriteShape_projeto/` se demanda emergir. Será Tool isolado (caminho A drop-crate ADR-0040).

## 12.8 PSD importer / Aseprite importer completo

**Decisão:** **OUT do Inspector**. Vai pro **asset cooker**.

**Razão:** Importers (PSD, Aseprite `.ase`, Procreate `.brush`, etc.) são parte do asset pipeline (offline cooker). Inspector consome os assets cookados. Não cabe expor "Import PSD" no Inspector do Sprite.

**Quando talvez:** Asset cooker territory. Inspector v2 SUPORTA NamedAnchor + SpriteFrames + Tags importados de Aseprite (vide [07](07_named_anchors.md), [08](08_animation_inline.md)) — schema lossless. Mas o IMPORT em si vive no cooker.

## 12.9 Hot-reload de sprites em runtime

**Decisão:** **OUT do Inspector**. Vai pra **app-level config / asset cooker integration**.

**Razão:** Hot-reload é propriedade global da app (file watcher + asset re-cook + entity refresh). Não cabe no Inspector do Sprite (cada sprite individual).

**Quando talvez:** feature da app PH2D (Preferences ▶ "Enable hot-reload"). Inspector NÃO terá toggle per-sprite. Quando arquivo source muda em disco, cooker recompila + scene refresh emite `EditorAction::SpriteAssetReloaded { entity }` automaticamente.

## 12.10 Multi-user collaborative editing do Sprite

**Decisão:** **OUT**.

**Razão:** Multi-user editing live (Figma-style) é arquitetura inteira (CRDT, presence, server). PH2D não é serviço hospedado. Inspector é single-user por design.

Vide spec do Vector Module ADR-0057 que tem CRDT data model para mutações vetoriais; aquele caso é específico (vetor é edit-heavy, sprite é mais bound-by-bytes). Sprite Inspector v2 NÃO replica esse trabalho.

**Quando talvez:** **nunca** no Inspector v2. Se PH2D virar plataforma cloud-hospedada futura, seria sub-projeto inteiro.

## 12.11 Vector graphics dentro do Sprite

**Decisão:** **OUT**.

**Razão:** Sprite é raster (`source: Atlas/Individual/Handpacked`). Vector é módulo separado (vide `docs/Vector Module/`). Sprite NÃO carrega vector paths editáveis.

**Bridge bidirecional Painter ↔ Vector** existe ([Painter §3.8](../Painter_projeto/README.md), Vector ADR-0062): Vector pode RASTERIZAR pra sprite; sprite pode ser VECTORIZADO para vector. Mas no Inspector do Sprite, é raster final, opaco ao usuário.

## 12.12 Macros / scripts editando o Sprite

**Decisão:** **OUT do Inspector UI**. Acessível via Luau / MCP (HR-10).

**Razão:** Toda propriedade exposta no Inspector é também exposta via `#[lua_export]` → MCP toolset `sprite_*`. Power-user automation usa Luau, não macros UI.

**Caso de uso:** `ph2d.sprite.bulk_set_tint(filter="enemies", color=[1, 0.5, 0.5, 1])` em Luau é equivalente a multi-select + edit no Inspector. PH2D já tem o caminho via scripting; UI macro recorder duplica surface.

## 12.13 Multiple Inspectors lado-a-lado

**Decisão:** **OUT v1**. Inspector é singleton (1 ativo na zona Right do chrome 4-zonas).

**Razão:** UI panel docked é singleton em cada zona (ADR-0023). Comparar 2 sprites side-by-side é nicho. Bulk-edit multi-select (vide [03 §3.14](03_inspector_secoes.md)) cobre 95% do caso de comparar.

**Quando talvez:** feature de "Pin Inspector to entity" (Inspector secundário flutuante) — pós-v1.

## 12.14 Property animation no Inspector (sem timeline)

**Decisão:** **OUT**.

**Razão:** Animar propriedades (tween de `opacity`, `tint`, `per_corner_tint`) é feature do **AnimationPlayer / SpriteAnimator + Timeline**. Inspector mostra ESTADO atual + permite scrub manual (drag value); não tem botão "record keyframe".

**Quando talvez:** módulo Animation/Timeline futuro. Inspector v2 manda valor → módulo timeline registra keyframe se gravando.

## 12.15 Render geometry ≠ Collision geometry (Paper2D `GeometryType`)

**Decisão:** **OUT do Inspector**. Vai pro **asset cooker / render optimization**.

**Razão:** Render geometry choice (Diced N×N, ShrinkWrapped, Custom polygon) é optimization de fillrate. Asset cooker decide automaticamente baseado em alpha distribution. Inspector NÃO expõe escolha manual (override só via config do asset).

**Quando talvez:** caso emergir necessidade, ADR-0070-amendment adiciona Component `RenderGeometryOverride` opcional. Default = cooker-decided.

## 12.16 Sub-pixel positioning toggle

**Decisão:** **OUT**. Propriedade da Camera2D / Viewport, não do Sprite.

**Razão:** Sub-pixel snap é decisão global do projeto (pixel-art game = snap; HD = no snap). Camera define. Sprite herda.

## 12.17 Visual Hot-reload preview

**Decisão:** **OUT**.

**Razão:** "Live preview do shader mudando antes de aplicar" é editor-wide feature, não Inspector do Sprite. Cobertura no Material editor (módulo Material) e Shader FX editor (módulo futuro), não aqui.

## 12.18 Sprite Stacking (técnica pseudo-3D voxel)

**Decisão:** **OUT**.

**Razão:** Sprite Stacking (N sprites empilhados com Y-offset por sprite → pseudo-3D voxel; GTA1/Kingdom/Brotato/Bouncy Castle Sims aesthetic) é **técnica de game design**, não feature de engine. Implementável pelo usuário com hierarquia de sprites + Y-offset incremental + rotação compartilhada — não precisa de Inspector v2 cap especial.

**Quando talvez:** se um número significativo de PH2D games usar, considerar feature "Stack" como Component opcional `SpriteStack { layers: u32, y_offset_per_layer: f32 }` em wave futura — adicionar via fan-out drop-crate (DIRETRIZ §3.A) sem ADR-amendment de v2.

## 12.19 CanvasTexture (multi-layer textures: diffuse + normal + specular)

**Decisão:** **OUT do Inspector v2**.

**Razão:** Godot CanvasTexture (sprite com `diffuse_texture + normal_texture + specular_texture` num único asset) é primariamente feature de **lighting** (vide §12.2 Sistema Lighting OUT). Sprite v2 carrega 1 texture source (Atlas/Individual); multi-layer textures vivem no módulo Lighting futuro.

**Quando talvez:** módulo `Lighting_2D_projeto/` adicionará Component opcional `LightingTextures { normal: Option<TextureRef>, specular: Option<TextureRef>, mask: Option<TextureRef> }` anexável ao entity. Sprite v2 NÃO ganha campo "normal_texture" — Component fora cobre.

## 12.20 Aseprite Linked Cels (memory-shared frames)

**Decisão:** **IN no asset cooker (lossless import); OUT do Inspector**.

**Razão:** Aseprite linked cels (frames compartilhando mesmos pixels via reference) é optimization de **memory autoria** (Aseprite file size). Para preservar no PH2D: asset cooker faz **dedup-hash automático** (frames com mesmo blake3 de texture_ref compartilham GPU memory). Inspector v2 não expõe — usuário não sabe que linked cels existiram; sees just N frames.

**Implementação cooker:** Aseprite import → cada frame → texture content hash; frames com mesmo hash compartilham `TextureRef`. Runtime: 0 overhead (mesma `texture_id`).

## 12.21 Free-Form Deformation / Mesh Distortion

**Decisão:** **OUT**.

**Razão:** Mesh distortion (Construct 3 `Set Mesh Point(x, y, ox, oy, z)`; Unity Sprite Shape FFD; livro virando página, bandeira ao vento) é feature de **Vector Module** territory (vide `docs/Vector Module/`). PH2D Sprite v2 é raster com transform 2D rígido + skew; deformação custom é vetorial.

**Quando talvez:** **nunca** no Sprite v2. Vector Module W6+ cobre via spline-based deform de vector paths.

## 12.22 Bake-from-3D pipeline (Dead Cells aesthetic)

**Decisão:** **OUT do Inspector** (pipeline externo).

**Razão:** "Modelar em 3D, renderizar offline em pequena resolução sem antialias, importar como sprite com normal map" (Dead Cells) é **pipeline de autoria externo** (Blender / Maya / Houdini → PNG output → PH2D import). Inspector v2 consome o resultado raster como qualquer sprite. Não há feature específica a expor.

**Quando talvez:** **nunca** no Inspector v2. Asset cooker pode oferecer "bake-from-3D" tooling em wave futura, mas não Inspector.

## 12.23 Frame-by-frame mass animation (Cuphead aesthetic, 800-1400 frames per character)

**Decisão:** **IN parcial** (Inspector v2 suporta o que infra atual permite; performance é asset cooker territory).

**Razão:** SpriteFrames v1 cap `frames count ≤ 4096` ([08_animation_inline.md §8.11](08_animation_inline.md)) cobre Cuphead character (800-1400 frames). Texture streaming + atlas dinâmico **OUT do Inspector** (vão pro asset cooker / streaming module futuro).

Inspector v2 mostra `frame: 0..N-1`; **N pode ser grande** desde que cooker entregue atlas eficiente. Não há gate específico no Inspector.

## 12.24 Frame events com payload tipado (footstep SFX no frame 3 com surface_type) — já em §12.6

Reiterado: vai pro timeline editor (módulo Animation futuro). Inspector v2 emite só signals básicos (FrameChanged, AnimationFinished, AnimationLooped).

## 12.25 GIF reading com edição frame-a-frame sofisticada

**Decisão:** **IN parcial** (Inspector v2 carrega GIF como SpriteFrames com 1 frame por GIF frame; edição sofisticada é OUT).

**Razão:** Asset cooker converte GIF → `SpriteFrames` (cada frame com `texture_ref` + `duration_ms` extraído do GIF). Inspector v2 mostra tags (se importer cria tags por GIF metadata), frame scrubber. Edição de easing entre cores, entries por frame → timeline editor.

## 12.26 RAW image format support

**Decisão:** **OUT**.

**Razão:** RAW workflow (CR2/NEF/ARW) é Lightroom/RawTherapee/darktable territory. Sprite v2 aceita PNG/JPEG/TIFF/EXR/WebP (via ph2d-imageio); usuária converte RAW fora.

## 12.27 Resumo — o que NÃO está aqui

| Categoria | Status | Razão |
|---|---|---|
| FX / Shader chain | OUT | Módulo Shader FX dedicado, futuro |
| Lighting 2D | OUT | Módulo Lighting dedicado, futuro |
| Física / Collision | OUT | Módulo Física dedicado, futuro |
| Onion skin | OUT | Timeline editor (módulo Animation) |
| Pixel-perfect camera | OUT | Módulo Camera dedicado |
| Frame events tipados | OUT | Timeline editor |
| SpriteShape (spline terrain) | OUT | Subsistema próprio |
| PSD/Aseprite full import | OUT do Inspector | Asset cooker territory |
| Hot-reload runtime | OUT do Inspector | App-level config |
| Multi-user editing | OUT | Não-objetivo PH2D |
| Vector dentro do Sprite | OUT | Módulo Vector separado |
| Macro UI recorder | OUT | Luau / MCP cobrem |
| Multiple Inspectors | OUT v1 | Bulk-edit cobre |
| Property animation in Inspector | OUT | Timeline editor |
| Render Geometry choice | OUT do Inspector | Asset cooker / opt |

## 12.28 Princípio operacional

Após o feedback do Enio absorvido (2026-05-27), estamos construindo a **versão padrão-ouro absoluto do Inspector do Sprite** — não engine inteira nem todos features. Tudo que é melhor que estado-da-arte (Godot/Unity/Unreal/Paper2D/Defold/GameMaker/Construct/Phaser/Aseprite combinados) **dentro do escopo do Inspector** e cabe nos princípios do PH2D (HR-1..HR-18, multi-plataforma, LLM-first) **entra**.

Cada item OUT acima tem **razão concreta**. Se demanda futura surgir, ADR-amendment 0069 (ou módulo dedicado novo) reabre o item formalmente.
