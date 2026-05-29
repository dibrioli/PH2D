# 13 — Referências

## 13.1 Pesquisa multi-engine (4 agentes paralelos, 2026-05-27)

### Godot 4 (2D)
- [Godot Docs — Node2D](https://docs.godotengine.org/en/stable/classes/class_node2d.html)
- [Godot Docs — CanvasItem](https://docs.godotengine.org/en/stable/classes/class_canvasitem.html)
- [Godot Docs — Sprite2D](https://docs.godotengine.org/en/stable/classes/class_sprite2d.html)
- [Godot Docs — AnimatedSprite2D](https://docs.godotengine.org/en/stable/classes/class_animatedsprite2d.html)
- [Godot Docs — SpriteFrames](https://docs.godotengine.org/en/stable/classes/class_spriteframes.html) — Godot 4 suporta **per-frame relative duration multiplier + animation_fps global multiplier** (não "só FPS global" como originalmente alegado). PH2D adopta `duration_ms` absoluto por simplicidade e Aseprite parity.
- [Godot Docs — AtlasTexture](https://docs.godotengine.org/en/stable/classes/class_atlastexture.html)
- [Godot Docs — CanvasItemMaterial](https://docs.godotengine.org/en/stable/classes/class_canvasitemmaterial.html)
- [Godot Docs — CanvasLayer](https://docs.godotengine.org/en/stable/classes/class_canvaslayer.html)
- [Godot Docs — VisibleOnScreenNotifier2D](https://docs.godotengine.org/en/stable/classes/class_visibleonscreennotifier2d.html)
- [Godot Forum — TileMapLayer y-sort/z-index](https://forum.godotengine.org/t/tilemaplayer-nodes-tileset-and-z-index-y-sorting/102531)
- [Bugnet Blog — Fix Z-Index in Godot 2D](https://bugnet.io/blog/fix-z-index-not-working-correctly-godot-2d)

### Unity 2D / URP 2D
- [SpriteRenderer reference (Unity 6)](http://docs.unity3d.com/Manual/sprite/renderer/sprite-renderer-reference.html)
- [SpriteRenderer Scripting API](https://docs.unity3d.com/ScriptReference/SpriteRenderer.html)
- [Texture Type Sprite import settings](https://docs.unity3d.com/Manual/texture-type-sprite.html)
- [9-Slicing sprites](http://docs.unity3d.com/Manual/sprite/9-slice/9-slicing.html)
- [Sorting Group reference (Unity 6)](https://docs.unity3d.com/6000.0/Documentation/Manual/sprite/sorting-group/sorting-group-reference.html)
- [Sprite Mask reference](https://docs.unity3d.com/6000.3/Documentation/Manual/sprite/mask/sprite-mask-reference.html)
- [Sprite Atlas reference (V2)](https://docs.unity3d.com/6000.3/Documentation/Manual/sprite/atlas/sprite-atlas-reference.html)
- [2D Animation Skinning Editor](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.0/manual/SkinningEditor.html)
- [Custom physics shape (Sprite Editor)](https://docs.unity3d.com/6000.1/Documentation/Manual/sprite/sprite-editor/custom-physics-shape/custom-physics-shape-landing.html)

### Unreal Paper 2D
- [PaperSprite Python API (UE 5.4)](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/PaperSprite?application_version=5.4)
- [Paper 2D Sprite Editor (UE 5.7)](https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-sprite-editor-in-unreal-engine)
- [Paper 2D Sprite Sockets (UE 5.7)](https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-sprite-sockets-in-unreal-engine)
- [Sprite Source Region & Render Geometry (UE 4.27)](https://docs.unrealengine.com/4.27/en-US/AnimatingObjects/Paper2D/Sprites/RenderGeometry)
- [Paper 2D Sprite Material (UE 5.7)](https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-sprite-material-in-unreal-engine)
- [PaperFlipbookComponent API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/Paper2D/UPaperFlipbookComponent)

### Defold
- [Defold Sprite Manual](https://defold.com/manuals/sprite/)
- [Defold Atlas Manual](https://defold.com/manuals/atlas/)
- [Defold Render Pipeline](https://defold.com/manuals/render/)
- [Defold Sprite Lua API](https://defold.com/ref/stable/sprite-lua/)

### GameMaker
- [GameMaker Sprite Editor](https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Sprites.htm)
- [GameMaker Nine Slice (beta)](https://manual.gamemaker.io/beta/en/The_Asset_Editors/Sprite_Properties/Nine_Slices.htm)
- [GameMaker Sprite Instance Variables](https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Sprites/Sprite_Instance_Variables/Sprite_Instance_Variables.htm)
- [GameMaker Texture Groups](https://manual.gamemaker.io/lts/en/Settings/Texture_Groups.htm)

### Construct 3
- [Construct 3 Sprite Plugin](https://www.construct.net/en/make-games/manuals/construct-3/plugin-reference/sprite)
- [Construct 3 Animations Editor](https://www.construct.net/en/make-games/manuals/construct-3/interface/animations-editor)
- [Construct 3 Effects](https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/objects/effects)
- [Construct 3 Mesh Distortion blog](https://www.construct.net/en/blogs/construct-official-blog-1/heard-meshes-1567)

### LÖVE
- [LÖVE love.graphics.draw](https://love2d.org/wiki/love.graphics.draw)
- [LÖVE SpriteBatch](https://love2d.org/wiki/SpriteBatch)
- [LÖVE SpriteBatch:attachAttribute](https://love2d.org/wiki/SpriteBatch:attachAttribute)
- [LÖVE BlendMode](https://love2d.org/wiki/BlendMode)

### Phaser 3 / 4
- [Phaser 3 Tint Component](https://docs.phaser.io/api-documentation/namespace/gameobjects-components-tint) — `setTintFill` em Phaser 3; Phaser 4 deprecou e move para `setTint(color).setTintMode(Phaser.TintModes.FILL)`. PH2D adopta o conceito (tint_fill bool toggle no Sprite struct).
- [Phaser 3 FX overview](https://docs.phaser.io/phaser/concepts/fx) — para Shader FX module futuro (OUT do Inspector v2).
- [Phaser 3 PostPipeline](https://newdocs.phaser.io/docs/3.80.0/Phaser.GameObjects.Components.PostPipeline) — para Shader FX module futuro.

### Aseprite (DCC)
- [Aseprite Slices](https://www.aseprite.org/docs/slices/)
- [Aseprite Tags](https://www.aseprite.org/docs/tags/)
- [Aseprite Linked Cels](https://www.aseprite.org/docs/linked-cels/)
- [Aseprite Blend Mode API](https://www.aseprite.org/api/blendmode)
- [Aseprite blend_mode.h (source)](https://github.com/aseprite/aseprite/blob/main/src/doc/blend_mode.h)

## 13.2 Levantamento de comunidade (Reddit + GitHub Issues + Forums)

### Godot Proposals (pedidos formais não-implementados)
- [#4282 — Add a Mask2D node](https://github.com/godotengine/godot-proposals/issues/4282)
- [#9222 — Pivot origins on Sprite2D](https://github.com/godotengine/godot-proposals/issues/9222)
- [#10937 — Frame-Specific Offsets for AnimatedSprite2D](https://github.com/godotengine/godot-proposals/issues/10937)
- [#14098 — Dynamic Tracking Points (sockets) on AnimatedSprite2D](https://github.com/godotengine/godot-proposals/issues/14098)
- [#567 — AnimatedSprite + AnimationTree integration](https://github.com/godotengine/godot-proposals/issues/567)
- [#11466 — Standard outlines for CanvasItems](https://github.com/godotengine/godot-proposals/issues/11466)
- [#11845 — Signal track type for AnimationPlayer](https://github.com/godotengine/godot-proposals/issues/11845)
- [#10160 — Zoom function in Texture preview inspector](https://github.com/godotengine/godot-proposals/issues/10160)

### Godot Issues (bugs abertos com regressões)
- [#41324 — 2D normal mapping rotations incorrect](https://github.com/godotengine/godot/issues/41324)
- [#70517 — Normal Map doesn't flip with flip_h/v](https://github.com/godotengine/godot/issues/70517)
- [#18299 — 2D Sprite Normal Maps inverted](https://github.com/godotengine/godot/issues/18299)
- [#79885 — Clip Children in Sprite2D not working](https://github.com/godotengine/godot/issues/79885)
- [#102190 — Sprite2D Clip children no longer works (regression)](https://github.com/godotengine/godot/issues/102190)
- [#102224 — Clip Children no longer masks by alpha (Control)](https://github.com/godotengine/godot/issues/102224)
- [#35606 — 2D Sprite jittering with camera smoothing](https://github.com/godotengine/godot/issues/35606)
- [#71074 — Jitter even with Snap 2D Transform to Pixel](https://github.com/godotengine/godot/issues/71074)
- [#97376 — Undo/Redo symmetry breaks in inspector](https://github.com/godotengine/godot/issues/97376)
- [#74265 — Shadows cover sprites when set to show behind sprite](https://github.com/godotengine/godot/issues/74265)

### Unity Discussions (feature requests)
- [Sprite sockets requested feature](https://discussions.unity.com/t/requested-feature-sprite-sockets/638175)
- [Skew support](https://discussions.unity.com/t/skew-support/526685)
- [Multi-Select editing for 2D Sprite Editor](https://discussions.unity.com/t/multi-select-editing-and-auto-name-for-2d-sprite-editor/880701)
- [Free Form Deformation for 2D Animation](https://discussions.unity.com/t/free-form-deformation-for-2d-animation/892102)
- [Animation Events multi-parameter best practices](https://discussions.unity.com/t/looking-for-best-practices-with-animation-events-multiple-parameters/843791)
- [Tight Packing in Sprite Atlas issue](https://discussions.unity.com/t/tight-packing-in-sprite-atlas-issue/748655)

### Defold Forum
- [Pain Points Survey](https://forum.defold.com/t/defold-pain-points-survey/81367)
- [Normal map lighting for 2D Pixel Art sprites](https://forum.defold.com/t/normal-map-lighting-for-2d-pixel-art-sprites/70967)

## 13.3 Tutoriais populares (sinais de gap no produto)

- [Outline Shader for Complex Sprites — Medium](https://medium.com)  (genérico, tutorial-heavy)
- [godot-color-dither — Multicolored dithering shaders](https://github.com/Donitzo/godot-color-dither)
- [Catlike Coding — True Top-Down 2D Lighting/Shadow](https://catlikecoding.com/godot/true-top-down-2d/4-light-and-shadow/)
- [pixel-beef itch.io — 4 things we did to add light/depth to our Pixel Art Game](https://pixel-beef.itch.io/xdasher/devlog/192949/4-things-we-did-to-add-light-and-depth-to-our-pixel-art-game)
- [Demystifying Sprite Atlas Variants](https://gametorrahod.com/demystifying-sprite-atlas-variants/)
- [Connor Wolf — Sprite Stacking in Godot](https://www.connorwolf.com/post/sprite-stacking-in-godot)
- [Alan Zucconi — Sprite Doodle Shader Effect](https://www.alanzucconi.com/2019/04/16/sprite-doodle-shader-effect/)

## 13.4 Jogos AAA 2D — soluções customizadas (apontam gap em engines)

- [GameDeveloper — Dead Cells Art Pipeline deep dive](https://www.gamedeveloper.com/production/art-design-deep-dive-using-a-3d-pipeline-for-2d-animation-in-i-dead-cells-i-) — 3D render → 2D bake, normal maps custom
- [GameDeveloper — Animating Cuphead](https://www.gamedeveloper.com/art/animating-i-cuphead-i-the-verve-of-the-1930s-with-the-tech-of-now) — frame-by-frame em massa, atlas streaming
- [Toon Boom — Cult of the Lamb pipeline](https://www.toonboom.com/half-giants-bill-northcott-on-channeling-enthusiasm-for-cult-of-the-lamb) — paper-stack sprites via Toon Boom Harmony → Spine

## 13.5 Tools externos referenciados

- [SpriteIlluminator — Normal map editor](https://www.codeandweb.com/spriteilluminator)
- [SpriteStack.io — Pseudo-3D sprite stacking](https://spritestack.io/)
- [evilmartians — OKLCH Color Picker](https://github.com/evilmartians/oklch-picker)
- [Microsoft Edge DevTools 3D View](https://blogs.windows.com/msedgedev/2020/01/23/debug-z-index-3d-view-edge-devtools/) — z-index visualization (web)

## 13.6 PH2D — refs internos

- [SKILL_Stack_PH2D_Definitiva.md](../../SKILL_Stack_PH2D_Definitiva.md) — Stack canônico, HR-1..HR-18.
- [DIRETRIZ.md](../IntegracaoMultiAgente/DIRETRIZ.md) — Workflow operacional.
- [ADR-0021 SimWorld/PresentWorld boundary](../architecture/decisions/0021-simulation-presentation-boundary.md).
- [ADR-0025 Transform component (componentes ECS)](../architecture/decisions/0025-transform-component.md) — base ortogonal a Sprite.
- [ADR-0023 UI/UX baseline (4 zonas)](../architecture/decisions/0023-ui-ux-baseline.md).
- [ADR-0029 Trait-driven panel host (Phase C.1)](../architecture/decisions/0029-trait-driven-panel-host.md) — base do ph2d-panel-inspector existente.
- [ADR-0039 Nodegraph contract FREEZE](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md) — Sprite NÃO é nó do grafo.
- [ADR-0040 Tool as isolated feature crate](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) — Inspector é Panel, não Tool.
- [crates/ph2d-render/src/sprite.rs](../../crates/ph2d-render/src/sprite.rs) — Sprite struct v3 (atual).
- [crates/ph2d-panel-inspector/](../../crates/ph2d-panel-inspector/) — Inspector panel atual (alvo de expansion).
- [docs/Painter_projeto/](../Painter_projeto/) — spec gêmea raster.
- [docs/Vector Module/](../Vector%20Module/) — spec gêmea vetorial.

## 13.7 Memory persistente do PH2D (`~/.claude/.../memory/`)

- [feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md) — padrão-ouro absoluto.
- [feedback-audit-lens-diversity](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md) — auditoria adversarial rotacionada.
- [feedback-no-industrial-claims-without-verification](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md) — verificar afirmações externas. **Aplicado pós-audit:** correções em Godot AnimatedSprite2D (per-frame relative + FPS, não "só FPS"), GameMaker 9-slice (4 modos canônicos: Stretch/Repeat/Mirror/BlankRepeat — "Hide" não existe), Phaser setTintFill (Phaser 3 OK; Phaser 4 deprecou → TintModes.FILL).
- [feedback-audit-internal-state-grep](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_internal_state_grep.md) — sweep grep por vapores internos. **Aplicado pós-audit:** verificações de paths (`crates/ph2d-color` vs `ph2d-painter-color` fantasma), tipos (`Transform2D`/`Rect2`/`StringKey`/`Value` vapores → `ph2d_ecs::Transform`/`[f32;4]`/`Box<str>`/`InstanceParamValue`), ADRs (`0025-transform-component` fantasma → `0025-gameobject-model`; `0051-painter-color-pipeline` fantasma → `0051-color-profile-pipeline`), variants (`PanelEvent::TextChanged` fantasma → `WidgetEvent::TextChanged`).
- [project-painter-w0-ratified-2026-05-26](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_painter_w0_ratified_2026_05_26.md) — precedente W0 ratificada.
- [project-vector-module-w0-ratified-2026-05-29](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_vector_module_w0_ratified_2026_05_29.md) — precedente W0 mais recente.
