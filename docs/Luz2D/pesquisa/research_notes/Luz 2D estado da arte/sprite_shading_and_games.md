# Sprite-level 2D shading, and how acclaimed 2D games get their lighting look

> Research notes, compiled 2026-10-08 for the PH2D Luz2D line (Rust/wgpu engine).
> Every claim carries an inline source and, where known, the source's date. **Confirmed** = stated by the
> developer or official documentation. **Community/unverified** = forum posts, wikis, reviewer observation.
> Anything without a fetched source is in *Gaps* or flagged in *Inferences*.
>
> Source dates used below (for quick reference):
> - Godot 2D lights & shadows docs (stable, Godot 4.x, current as of 2026-10) · Unity URP 2D light docs (Unity 6000.0 manual, current)
> - Thomas Vasseur, *Dead Cells* deep dive, Game Developer, **2018-01-25**
> - Svyatoslav Cherkasov, *Graveyard Keeper* effects, Game Developer, **2018-10-23**
> - Noel Berry, *Remaking Celeste's Lighting*, **2017-09-25**
> - TheGamer interview, *Ori and the Will of the Wisps* lightmaps, **2020-10-13**
> - Holographic Radiance Cascades (Freeman, Sannikov, Margel), arXiv 2505.02041, **May 2025**; Osborne & Sannikov, arXiv 2408.14425, **Aug 2024**
> - Sprite Lamp product page (text dates from ~2013–2014); SpriteIlluminator knowledge base (undated, current product)

---

## (A1) Normal-mapped sprite lighting math and how engines expose it

### Takeaway
Every mainstream 2D engine exposes the same small model: per-sprite normal map (+ optional specular map) lit by 2D lights that carry a **virtual height/Z** (Godot `height`, Unity `Normal Map Distance`), a **radius with an inner/outer falloff** (or a falloff texture), colour × energy, a blend mode (add / multiply / mix), and optional 2D shadow casters with a softness filter. Artists tune the *shape* of falloff mostly by gradient textures or "falloff strength" sliders, not by physical inverse-square.

### Cited Findings
**Godot 4 (CanvasItem lighting)** — [Godot docs: 2D lights and shadows](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html) (current stable docs, fetched 2026-10-08):
- `PointLight2D.texture` defines the light's *shape* (a light "cookie" in all but name; the docs don't use the word "cookie"); its size sets the light size; `offset` moves the texture without moving shadows; `texture_scale` multiplies size and "larger lights have a higher performance cost".
- `height` = the light's virtual height for normal mapping; it only affects surfaces that have normal maps. `DirectionalLight2D.height` ranges 0.0 (parallel to surface, default) to 1.0 (perpendicular).
- Custom falloff is done by assigning a `GradientTexture2D` in Radial fill mode (opaque white centre → transparent white edge) — i.e. falloff curve = an artist-drawn gradient.
- Blend modes: **Add** (default), **Subtract** (negative lights), **Mix** (lerp between light texture and pixels under it).
- Normal/specular maps are attached via a `CanvasTexture` (Diffuse, Normal Map, Specular texture + Specular colour + **Shininess**: low = broad highlight, high = tight "wet" highlight).
- Practical tip from the docs: "lights may look weaker after enabling normal mapping" — raise the light's Height and possibly Energy.
- Directional shadows are always infinitely long; for shorter shadows the docs recommend a custom shader reading the **2D SDF** built from `LightOccluder2D`s (SDF collision is on by default and costs nothing unless a shader reads it).
- Shadow filters: **None** (fastest, blocky — "suits pixel art"), **PCF5** (moderate), **PCF13** (softest, most expensive — "use on only a few lights"); `filter_smooth` > 0 can band/streak.
- The docs recommend the free tool **Laigter** for generating maps.

**Unity URP 2D Renderer** — [Unity 6 manual: Light 2D component reference](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/2DLightProperties.html) (current, fetched 2026-10-08):
- Light types: **Spot** (point light with inner/outer radius and inner/outer spot angle), **Freeform** (editable polygon), **Sprite** (light shaped by a sprite asset — Unity's equivalent of a cookie), **Global** (no attenuation; one per blend style and sorting layer).
- **Inner radius** = full intensity; **Outer radius** = where light stops; intensity fades between; **Falloff Strength** controls how fast it drops. Freeform: `Falloff` (size) + `Falloff Strength` (softness).
- **Normal Map Quality**: Disabled / **Fast** (cheaper, can cause artefacts) / **Accurate** (best, costs more). **Normal Map Distance** = the light's distance above the sprite plane, "without moving the light or changing its intensity" (i.e. light Z for N·L).
- Shadows: Strength (0–1), Softness, Falloff Strength (softer shadow edge); shadow casters are `ShadowCaster2D`.
- **Volumetric**: Volumetric Intensity (light visible in empty space) and Volumetric Shadow Strength.
- Blend Styles: Multiply, Additive, Multiply-with-Mask(R), Additive-with-Mask(R); **Overlap Operation**: Additive (default) or Alpha Blend; **Light Order** matters for Alpha Blend.
- Lights targeting different sorting layers reduce performance because Unity renders more light and shadow textures — [Unity 6 manual: 2D light properties explained](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/2d-light-properties-explained.html).

**Graveyard Keeper's custom vertical attenuation** (shipped, Unity built-in lights) — [Cherkasov, Game Developer, 2018-10-23](https://www.gamedeveloper.com/programming/graveyard-keeper-how-the-graphics-effects-are-made): a shader measures each sprite's *vertical* distance from the light; if the light is in front the sprite is lit normally, if the sprite occludes the light, lighting "fades quickly rather than cutting off instantly" — a top-down-specific depth term on top of N·L.

### Inferences
- The minimum feature set artists will expect from PH2D lights is the union of Godot+Unity: colour, energy/intensity, inner+outer radius with a falloff-strength curve **and** an optional falloff/cookie texture, light height (Z) for N·L, add/multiply/mix blend, per-light layer masks, shadows with strength + softness, and an optional volumetric term.
- Neither engine exposes "inverse-square" as the default: both default to an artist-shaped radial ramp with a hard cutoff radius. A physically-based inverse-square with a windowing function (as in 3D engines) could be offered as one curve among several, but the artist-expected default is a bounded, shapeable ramp. (Inference from the documented parameter sets; no source states artist preference directly.)
- Half-Lambert (`(0.5·N·L + 0.5)²`, introduced by Valve for Half-Life) is the usual way to keep the "dark side" of a sprite from going black. I could not fetch Valve's page (HTTP 403) this session, so treat the formula as standard knowledge, not a verified citation here.

### Gaps
- GameMaker and Defold have no built-in normal-mapped 2D light system documented in what I fetched; their lighting is shader/asset-based. I did not fetch their docs — unverified.
- No source found that surveys which falloff curves artists *prefer*; the evidence is indirect (engine defaults).

---

## (A2) Tools and pipelines for generating sprite normal maps

### Takeaway
There are four families: (1) **luminance/alpha-derived automatic** generators (SpriteIlluminator, Laigter, Sprite DLight) — fast, but guess depth from colour and get it wrong on dark details; (2) **painted lighting profiles** (Sprite Lamp: 2–5 greyscale drawings lit from different directions → normal/depth/AO; Graveyard Keeper used a home-grown 4-direction version); (3) **3D-rendered** (Dead Cells, Eastward rebuilds assets in 3D and paints bump maps); (4) **fully hand-painted light-direction maps** (Ori WotW: ~30,000 hand-drawn lightmaps, explicitly rejecting generated normals as "plasticky"). The highest-acclaim results come from (2)–(4).

### Cited Findings
- **SpriteIlluminator (CodeAndWeb)**: effect tools generate normal maps by image processing that "analyze the sprite's transparency and structure"; the **Emboss** tool treats brighter pixels as raised and darker as recessed; **Bevel** adds volume from the alpha silhouette; a structure brush stamps patterns; existing normal maps can be imported and edited; batch publish writes `_n`-suffixed maps; tutorials target Unity, SpriteKit, cocos2d-x — [CodeAndWeb: emboss tool KB](https://www.codeandweb.com/spriteilluminator/knowledgebase/emboss-tool), [CodeAndWeb: import normal map KB](https://www.codeandweb.com/spriteilluminator/knowledgebase/import-normal-map), [CodeAndWeb tutorials](https://www.codeandweb.com/spriteilluminator/tutorials) (current product pages, undated).
- **Laigter** (free, open source, **GPL-3.0**, Win/macOS/Linux, OpenGL 2.x): auto-generates normal maps from luminance and detail of the source; also generates **parallax, specular and ambient-occlusion** maps; a custom heightmap can override the auto result (author's own example: moustaches came out as depressions because "laigter uses color for that"); real-time preview with multiple movable lights and animation; headless CLI with presets; tileset neighbour support; grayscale maps can be packed into the normal map's alpha; plugin system — [Laigter docs](https://laigter.readthedocs.io/en/latest/Introduction/intro.html), [Laigter itch.io](https://azagaya.itch.io/laigter), [devlog v1.10.3](https://azagaya.itch.io/laigter/devlog/219830/laigter-1103-is-out), [GameFromScratch: Laigter 1.7](https://gamefromscratch.com/laigter-1-7-released/). Godot's own docs recommend it ([Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)).
  - Licence note for PH2D (§0.9 of CLAUDE.md): GPL-3.0 ⇒ usable only as an **oracle run by script**, never as source to read/port.
- **Sprite DLight (2Dee Game Art)**: automatic generation, works from sprite sheets; a third-party reviewer called it the best automatic generator, close to Sprite Lamp quality but hard to control and sometimes inaccurate on large images — [Construct tutorial "Enlighten games", page 3](https://www.construct.net/en/tutorials/enlighten-games-dynamic-1213/page-3) (undated; describes it as beta/pre-order, so probably stale).
- **Sprite Lamp (Snake Hill Games)**: normal maps created from 2–5 hand-drawn greyscale **lighting profiles** (subject lit from one direction each); also produces **depth maps** (demonstrated mainly via self-shadowing), **ambient occlusion**, an edge-exposure **specularity** map, and (Pro) **anisotropy** maps from a sketched flow map; Pro has depth-map editing tools — [Sprite Lamp page](https://www.snakehillgames.com/spritelamp/) (page text ~2013–14).
- **Graveyard Keeper**: "every sprite has its own normal map"; the artist roughly paints light on **four sides** of the sprite with a brush and a **script merges the four into a normal map**; author recommends Sprite Lamp — [Cherkasov, 2018-10-23](https://www.gamedeveloper.com/programming/graveyard-keeper-how-the-graphics-effects-are-made).
- **Dead Cells**: 3DS Max model + skeleton → FBX → in-house renderer draws the mesh "at a very small size with no antialiasing"; each frame exported as PNG **plus its normal map**, lit in-game with a "basic toon shader"; characters ~50 px tall; flickering pixels remained an accepted, unresolved artefact — [Vasseur, Game Developer, 2018-01-25](https://www.gamedeveloper.com/production/art-design-deep-dive-using-a-3d-pipeline-for-2d-animation-in-i-dead-cells-i-). The tool was never released; community recreations exist ([80.lv Dead Cells pipeline case study](https://80.lv/articles/case-study-dead-cells-character-art-pipeline/)).
- **Eastward (Pixpil)**: sprites drawn in Aseprite, split into parts by natural structure (roof vs wall), **rebuilt in a 3D environment and hand-painted in the bump map one by one** "to ensure they have the correct lighting performance in-game" — [Game Developer: Eastward's creators share insights](https://www.gamedeveloper.com/art/eastward-s-creators-share-insights-on-making-pixel-art-adventures).
- **Ori and the Will of the Wisps**: a **light direction map per asset**; ~7,000 assets → ~30,000 hand-drawn lightmaps; the team avoided script-generated normal maps which can look "very cheap [and] plasticky" (lead artist Daniel van Leeuwen) — [TheGamer, 2020-10-13](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/).

### Inferences
- The tool spectrum maps to a quality/cost trade-off: auto-from-luminance (minutes, plasticky, wrong on dark features) → painted profiles (hours, artist-controlled) → 3D re-build (days, correct volumes) → per-asset hand-painted direction maps (Ori; very high cost, highest acclaim). PH2D should accept **all** of these as inputs (a normal map is a normal map) and ideally offer an in-editor generator with an **override height layer** (Laigter's lesson) and a **"paint N directional profiles → normal"** mode (Sprite Lamp / Graveyard Keeper lesson).
- "Ori-style" multi-directional light maps are not strictly normal maps — they encode painted response per light direction — which suggests a representation more general than N·L (e.g. a few painted directional basis images blended by light direction). Inference only.

### Gaps
- **AI-generated normal maps for sprites**: I found no reliable, dated source on shipped use or tool quality in this session. Not researched further.
- Sprite DLight current status, price and platforms: not verified.

---

## (A3) Height/depth maps, self-shadowing, contact shadows, AO for 2D

### Takeaway
Self-shadowing on sprites is done either with a **per-pixel ray march over a height map** toward the light (cheap enough at sprite resolution) or a **precomputed horizon map** (angle at which the sprite occludes itself, tested against light elevation — free at runtime but self-only). Cross-object shadows in shipped games are usually cheaper tricks: rotating **shadow sprites** (Graveyard Keeper), **2D occluder polygons / SDF** (Godot, Unity ShadowCaster2D), or per-wall shadow quads (Celeste).

### Cited Findings
- Precomputed **horizon maps** (per pixel: the elevation angle at which the sprite occludes light), tested against light elevation in the shader; "pre-calculated, so none of it has to be generated at run-time", but self-shadow only — [GameMaker forum: 2D light and self-shadowing](https://forum.gamemaker.io/index.php?threads/2d-light-and-self-shadowing.29479/) (community, undated).
- **Height-map ray march**: store per-pixel height (e.g. a 10-px-tall building → value 10); for each pixel step toward the light until a taller pixel is found — [Matt Greer: Dynamic lighting and shadows in my 2D game](https://mattgreer.dev/blog/dynamic-lighting-and-shadows/) (dev blog, undated in fetch). Same idea as a fixed-step shader with step count/size/strength controls — [Godot Shaders: basic self-shadowing from heightmap](https://godotshaders.com/shader/basic-self-shadowing-from-heightmap/).
- **Sprite Lamp** depth maps exist mainly to drive self-shadowing; Pro allows hand-refinement of depth — [Sprite Lamp](https://www.snakehillgames.com/spritelamp/).
- **Laigter** produces **ambient-occlusion maps** ("define which parts ambient light should reach") and parallax maps (mainly for backgrounds) — [Laigter docs](https://laigter.readthedocs.io/en/latest/Introduction/intro.html), [LinuxLinks on Laigter](https://www.linuxlinks.com/laigter-generate-lighting-maps-2d-game-artwork/).
- **Graveyard Keeper** cross-object shadows: each object has up to **four shadow sprites** (one sun + nearest three dynamic lights, chosen per-frame in `Update()`), rotated in the **vertex shader** with the angle passed via a colour channel; ~10 generic shadow sprites reused; stretched objects (fences) look less convincing — [Cherkasov, 2018-10-23](https://www.gamedeveloper.com/programming/graveyard-keeper-how-the-graphics-effects-are-made).
- Alpha-tested shadow casting so transparent quad pixels don't cast — [pixi-lights-and-shadows README](https://github.com/dobrado76/pixi-lights-and-shadows) (community project).
- Godot recommends the real-time **2D SDF** for custom shadow shaders (e.g. finite-length directional shadows) — [Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html).

### Inferences
- For PH2D a "height" channel per sprite unlocks three features from one asset: self-shadow (ray march in sprite texel space), contact shadows (march a short distance in screen space against a scene height buffer), and parallax. Marching in **whole texel steps** keeps shadow edges on the pixel grid for pixel art. (Synthesis; not from a single source.)
- Horizon maps are a good precompute path when the light set is many and cheap is critical, at the cost of no cross-sprite shadows.

### Gaps
- No shipped acclaimed game was found to document screen-space 2D contact shadows or 2D SSAO specifically.

---

## (A4) Specular/rim light and "pixel-art safe" lighting

### Takeaway
Specular is exposed as a per-sprite specular map + shininess (Godot) and is used by shipped pixel games (Children of Morta, per a review). The core pixel-art hazard is that lighting is computed at **screen resolution** and so smears smooth gradients across chunky pixels; the documented fix is to **snap the light/shadow sample position to the art's texel grid**, plus optional quantisation of the lit value. Dead Cells used a **toon shader** (banded) on normals; Octopath's team noted that too much resolution/saturation "lost the appeal of pixel art".

### Cited Findings
- Godot: lighting is computed at viewport resolution so Nearest filtering does **not** pixelate it; fix = a canvas-item shader that snaps `LIGHT_VERTEX` and `SHADOW_VERTEX` to a grid with `floor()` — [Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html). Shadow filter **None** gives blocky edges that "suit pixel art" (same source).
- Dead Cells: normal maps let "a basic toon shader render the model's volume" — [Vasseur, 2018-01-25](https://www.gamedeveloper.com/production/art-design-deep-dive-using-a-3d-pipeline-for-2d-animation-in-i-dead-cells-i-).
- Octopath Traveler (HD-2D, UE4): early attempts "went too far on the resolution and saturation, losing the appeal of pixel art"; the look took ~1.5 months to reach — [Siliconera (2017/18)](https://www.siliconera.com/project-octopath-traveler-developers-answer-project-started-troubles-developing-hd-2d/), [Wikipedia: HD-2D](https://en.wikipedia.org/wiki/HD-2D).
- Noita applies **Dithering**, Lighting, Fog of war, Glow, Refraction and Gamma correction as post-processing passes (the map-capture mod disables each) — [noita-mapcap README](https://github.com/Dadido3/noita-mapcap/blob/master/README.md) (community tool; inference about pipeline order).
- Children of Morta (Unity) shows dynamic lighting **and specular maps** — per a review, [JumpDashRoll review](https://www.jumpdashroll.com/article/children-of-morta-review) (reviewer observation, not developer).
- Slynyrd pixel-art lighting guide (hand-painted light/shadow principles) — [Pixelblog 6: Light and Shadow, 2018-06-15](https://www.slynyrd.com/blog/2018/6/15/pixelblog-6-light-and-shadow).

### Inferences
- A "pixel-art mode" for PH2D lights should be a first-class switch: (1) evaluate lighting at the sprite's texel centre (snap), (2) optional N-band quantisation of the light term (toon ramp / posterise) applied before colour, (3) optional ordered dithering between bands, (4) shadow filter = none. Dead Cells (toon), Godot (snap) and Noita (dither pass) each show one piece. The combination is my synthesis.
- Rim light was not documented by any shipped-game source found; it is usually an N·V-style term or a separate "back light" — inference.

### Gaps
- No developer source found for specific rim-light implementations in the listed games.
- No dated source on quantised/posterised light in Sea of Stars or Eastward specifically.

---

## (A5) Rendering cost

### Takeaway
Cost scales with **lit pixel area × number of lights** and with **shadow filter width**; shipped games keep it cheap by atlasing light masks (Celeste: up to 256 lights in one 2048² texture, one pass), limiting shadow casters per object (Graveyard Keeper: max 4), or skipping "real" lights for additive sprites (Hollow Knight-style). Tile light propagation (Terraria) is CPU-side and gets expensive with many sources.

### Cited Findings
- Godot: bigger `texture_scale` ⇒ more pixels ⇒ more cost; **PCF13** expensive, few lights only; directional `max_distance` culls casters; **additive sprites are faster than lights** but can't cast shadows, can't light fully dark areas, and ignore normal/specular — [Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html).
- Unity: Normal Map Quality "Accurate" costs more than "Fast"; lights on multiple sorting layers cost extra light/shadow textures — [Unity Light 2D reference](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/2DLightProperties.html), [Unity 2D light properties explained](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/2d-light-properties-explained.html).
- Celeste (320×180): moved from per-light generated meshes (floating-point edge cases) to a **cutout** method: light drawn into a single channel of a mask, wall shadow quads erase it; 4 lights per RGBA texture; final **2048×2048 grid of 64 cells (≤256×256 each)** = up to 256 lights drawn in one pass; "much less CPU work and only a minor GPU cost", "lights are now some of the fastest code in the game" — [Noel Berry, 2017-09-25](https://noelberry.ca/posts/celeste_lighting/).
- Tile-based (Terraria-style) lighting overlays get expensive quickly with many sources — [Unity forum: Starbound/Terraria style lighting](https://forum.unity.com/threads/starbound-terraria-style-lighting.325086/) (community).

### Gaps
- No ms/frame numbers found for any shipped title's lighting pass.

---

## (B) Case studies of shipped 2D games with acclaimed lighting

### Takeaway
Two camps dominate the acclaimed set. **Painted/baked** (Hollow Knight, Ori Blind Forest, Hyper Light Drifter): light is mostly authored art — soft transparent shapes, additive glows, painted light/shadow — with real-time used for accents. **Real-time normal-mapped + dynamic** (Dead Cells, Graveyard Keeper, Eastward, Children of Morta, Sea of Stars, Octopath/HD-2D, Ori WotW): sprites carry normals (generated, painted or 3D-derived) and are lit by moving lights, often with a day/night cycle. The frontier is **2D global illumination** (Path of Exile 2's Radiance Cascades) and per-pixel simulated worlds (Noita). What players praise is consistently: soft shadows, coloured emissive glow that lights surroundings, time-of-day colour shifts (LUTs), fog layers and light shafts.

### Cited Findings (per game; technique · engine · source · real-time vs painted)

**Dead Cells (Motion Twin, 2017/2018)** — **real-time**, custom engine (Heaps/Haxe is common knowledge, not in fetched source). 3D-to-pixel pipeline with per-frame normal maps lit by a toon shader; built "so shadows wouldn't need to be hand-drawn on every frame", a cost they had paid on the earlier ScarKrow prototype — [Vasseur, Game Developer, 2018-01-25](https://www.gamedeveloper.com/production/art-design-deep-dive-using-a-3d-pipeline-for-2d-animation-in-i-dead-cells-i-).

**Graveyard Keeper (Lazy Bear Games, 2018)** — **real-time**, Unity built-in lights. Per-sprite painted normal maps (4-direction paint → script); torches/ovens/lamps as dynamic lights; ambient brightness and colour from a time-of-day gradient, lights turn darker and **bluer at night**; **10 LUTs** for times of day; up to 4 rotating shadow sprites per object; fog from layered horizontal cloud sprites with fewer layers higher up to fake altitude — [Cherkasov, 2018-10-23](https://www.gamedeveloper.com/programming/graveyard-keeper-how-the-graphics-effects-are-made). Sequel moved to a full 3D world partly because of "limitations of 2D graphics" — [TOKUGAME note](https://note.com/tokugame/n/nb9693274c275?hl=en) (2025/26 interview article; translated).

**Celeste (Maddy Makes Games, 2018)** — **real-time 2D light masks with hard geometry shadows** (lights that "actually hit walls"); cutout method + light atlas (details in A5) — [Noel Berry, 2017-09-25](https://noelberry.ca/posts/celeste_lighting/). No normal maps described.

**Ori and the Blind Forest (Moon Studios, 2015)** — mostly **painted**, Unity. Over 7,000 hand-painted graphics, each assignable its own lighting, colour and movement settings (lead artist Max Degen); prototype began in Construct and was ported to Unity in a week — [Xbox Wire: The Art of Ori](https://news.xbox.com/en-us/?p=4339), [MCV: Unity focus — making Ori](https://www.mcvuk.com/development-news/unity-focus-making-ori-and-the-blind-forest/) (both ~2014–15).

**Ori and the Will of the Wisps (2020)** — **hybrid**: per-asset hand-painted **light direction maps** (~30,000 for ~7,000 assets) so painted assets react to "various lighting systems"; generated normals rejected as plasticky — [TheGamer, 2020-10-13](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/). Thomas Mahler described a dynamic system where "every texture in the game reacts to lighting sources" — [Wccftech: WotW dynamic lighting](https://wccftech.com/ori-and-the-will-of-the-wisps-dynamic-lighting/amp/) (2019, secondary).

**Hollow Knight (Team Cherry, 2017)** — **painted/sprite-based**, Unity. Unity's case study: hand-drawn frames → PNG; stock `sprite_default`/`sprite_diffuse` shaders with small modifications; the team avoided complex 3D lighting and used **soft, transparent shapes** for light — [Unity case study: Hollow Knight](https://unity.com/case-study/hollow-knight) (undated). That these light shapes are *additive* is a **community guess**, not confirmed — [Unity forum: sprites as light](https://forum.unity.com/threads/sprites-as-light.536041/). Background blur appears to be depth-of-field (player observation, Steam). **Silksong (2025)**: no technical source found.

**Hyper Light Drifter (Heart Machine, 2016)** — GameMaker ([GameMaker showcase](https://gamemaker.io/en/showcase/hyper-light-drifter)). Glow: full-screen pulsing glow via shaders and soft light layers over pixel art — **community guess only** ([Unity forum thread, 2016](https://forum.unity.com/threads/can-you-make-games-like-hyper-light-drifter-in-unity.407227/)). Treat as mostly painted + glow post.

**Eastward (Pixpil, 2021)** — **real-time deferred lighting** in Pixpil's own engine (lead programmer Tommo Zhou names it "Maoi"); "3D game with a 2D perspective"; assets split by structure, rebuilt in 3D, bump maps hand-painted; fog layers and dawn sun beams for atmosphere — [Game Developer: Eastward's creators](https://www.gamedeveloper.com/art/eastward-s-creators-share-insights-on-making-pixel-art-adventures); Pixpil posts "Pixel art + Deferred lighting" ([Facebook](https://www.facebook.com/pixpil/photos/a-wip-scene-from-eastwardpixel-art-deferred-lighting/894060787367832/)) and "retro Pixel Art + 3D Lighting" ([X, 2015-11](https://twitter.com/pixpilgames/status/671177081515044864?lang=en)).

**Sea of Stars (Sabotage Studio, 2023)** — **real-time**, "full-on dynamic lighting" via a **custom render pipeline**; day/night is a player-controlled mechanic with moving shadows — [Sabotage press kit](https://sabotagestudio.com/presskits/sea-of-stars/), [GodIsAGeek preview, 2022-03](https://godisageek.com/2022/03/sea-of-stars-seems-like-a-must-play-already-hands-off-preview/), [MobileSyrup interview, 2023-09-11](https://mobilesyrup.com/2023/09/11/sea-of-stars-level-design-sabotage-studio-thierry-boulanger-interview/). The 2020 Kickstarter period plans reportedly listed dynamic sprite lighting with normal mapping, pixel depth writing, shadow receiving, volumetric lighting and screen-space indirect lighting — **I could not fetch the Kickstarter page (403); this list comes from a search summary and is pre-release, so unverified for the shipped game.** Players report charging Zale's fireball casts shadows from other characters — [ResetEra thread](https://www.resetera.com/threads/its-frankly-insane-how-good-sea-of-stars-pixel-art-is.1048128/) (anecdotal). Engine (Unity) **not confirmed** by any fetched source.

**Octopath Traveler (Square Enix/Acquire, 2018) — "HD-2D"** — **real-time 3D**, Unreal Engine 4: pixel sprites in a 3D world with dynamic lighting, depth of field and tilt-shift for a diorama look — [Wikipedia: HD-2D](https://en.wikipedia.org/wiki/HD-2D); UE4 Blueprints/optimisation credited by Acquire's lead programmer — [Unreal spotlight](https://www.unrealengine.com/spotlights/octopath-traveler-s-hd-2d-art-style-and-story-make-for-a-jrpg-dream-come-true?lang=ja); sequel interview — [Unreal: Octopath II](https://www.unrealengine.com/en-us/developer-interviews/octopath-traveler-ii-builds-a-bigger-bolder-world-in-its-stunning-hd-2d-style) (2023).

**Children of Morta (Dead Mage, 2019)** — Unity, dynamic lighting and specular maps (reviewer) — [JumpDashRoll](https://www.jumpdashroll.com/article/children-of-morta-review); "lighting effects and shadows add richness and depth" — [Engadget hands-on, 2018-03-22](https://www.engadget.com/2018/03/22/children-of-morta-hands-on-indie-xbox-gdc/). **No developer source on normal maps found** (TechRaptor art article returned 403).

**Kingdom Two Crowns (Raw Fury / Stumpy Squid co-dev, 2018)** — Unity; "intricate lighting system" (Gordon Van Dyke) — [Game Developer: Kingdom Two Crowns and pixel art](https://www.gamedeveloper.com/design/-i-kingdom-two-crowns-i-and-the-practical-intersection-of-pixel-art-and-roguelike-design); a 2.0 update listed "improved lighting systems" — [Kingdom wiki patch notes](https://kingdomthegame.fandom.com/wiki/Patch_notes_for_Two_Crowns). Implementation undocumented.

**Noita (Nolla Games, 2020)** — custom "Falling Everything" engine; every pixel simulated ([GDC Vault 2019: Exploring the Tech and Design of Noita](https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-Design) — talk focuses on simulation/physics; no lighting detail found in summaries). Light reveals **fog of war** temporarily: the Light modifier emits a 267-px-diameter light that is "actually just a particle (`fog_of_war_hole_128.xml`)" also used by explosions — [Noita wiki: Light](https://noita.wiki.gg/wiki/Light). Fog stored as a byte per 32×32 world chunk — **unofficial** ([wiki talk page](https://noita.wiki.gg/wiki/Talk:Modding:_Fog_of_War)). Lighting, glow, fog, dithering, refraction, gamma are post-process passes — [noita-mapcap](https://github.com/Dadido3/noita-mapcap/blob/master/README.md).

**Terraria (Re-Logic, 2011–)** — **tile/sub-tile light propagation**, CPU. Four modes: **Color** (sub-tile pixel clusters, smooth, coloured — default on desktop), **White**, **Trippy** (old per-tile engine with colour), **Retro** (per-tile, white, pre-1.1 look); several screen effects only run in Color/White — [Official Terraria wiki: Lighting mode](https://terraria.wiki.gg/wiki/Lighting_mode). Algorithm is reportedly a flood fill with attenuation per step, stronger through solid blocks, max-combine at overlaps — **community description** ([Unity forum](https://forum.unity.com/threads/starbound-terraria-style-lighting.325086/), [JVM Gaming](https://jvm-gaming.org/t/starbound-lighting-techneques/38799)). **Starbound**: ray-cast with slow falloff in air, fast in blocks — attributed to a developer on a forum, unverified ([JVM Gaming](https://jvm-gaming.org/t/starbound-lighting-techneques/38799)).

**Path of Exile 2 (Grinding Gear Games, early access 2024-12)** — **Radiance Cascades** GI (Alexander Sannikov, presented ExileCon 2023); described as a 2D+1D cascade in screen space; core idea: near light needs fine spatial resolution, far light needs fine angular resolution ("penumbra criterion": spatial Δs = a, angular Δω = 1/b for a shell [a,b]) — [Osborne & Sannikov, arXiv 2408.14425 (2024)](https://arxiv.org/pdf/2408.14425), [LIRIS talk slides](https://projet.liris.cnrs.fr/origami/topics/gdl/gdl_talk_84/radiance_cascades.pdf). The 2025 Holographic RC paper states Sannikov's 2023 method "is currently used in the game Path of Exile 2"; known RC artefacts: ring artefacts around lights (fixed by the "bilinear fix"), and HRC blurs small penumbras by ~4 px — [Freeman, Sannikov, Margel, arXiv 2505.02041 (May 2025)](https://arxiv.org/pdf/2505.02041). Rust/Bevy community crate: [solis_2d](https://lib.rs/crates/solis_2d) (WIP). Note PoE2 is a 3D game; RC was applied in screen space — it's the reference for **2D GI** because RC is natively a 2D-plane technique.

**Owlboy (D-Pad Studio, 2016)** — XNA ([Wikipedia](https://en.wikipedia.org/wiki/Owlboy)); **no lighting technique source found**.
**Blasphemous 1/2 (The Game Kitchen, 2019/2023)** — Unity ([Wikipedia](https://en.wikipedia.org/wiki/Blasphemous_(video_game))); **no lighting technique source found**.

### Inferences
- **What "state-of-the-art 2D look" means to artists today** (synthesis across the above): (1) coloured lights with soft, shapeable falloff and emissive sources that visibly light neighbours (Celeste masks, Eastward, Sea of Stars); (2) sprites that respond to light direction with believable volume — via painted, 3D-derived, or generated normals, with painted ≫ generated in perceived quality (Ori WotW, Eastward, Graveyard Keeper); (3) time-of-day colour grading via LUTs and tinted ambient (Graveyard Keeper, Sea of Stars); (4) soft/real shadows from moving lights (Sea of Stars, Graveyard Keeper's shadow sprites, Celeste geometry shadows); (5) atmosphere: layered fog, god rays/volumetric (Eastward dawn beams, Unity volumetric 2D light); (6) indirect/bounce light as the new frontier (PoE2 RC; Sea of Stars' reported screen-space indirect — unverified).
- **Real-time vs painted split**: Dead Cells, Graveyard Keeper, Celeste, Eastward, Sea of Stars, Octopath, Terraria, Noita = real-time light; Hollow Knight, Ori BF, HLD = predominantly painted/authored light with real-time glow/additive accents; Ori WotW = painted light *data* driven by real-time lights. A PH2D lighting system must serve both: authored additive/multiply light shapes with no normal maps (cheap, the Hollow Knight path) **and** normal-mapped dynamic lights.
- For pixel art specifically, the shipped evidence says pixel-snapping and banded/toon response matter more than physical accuracy (Godot snap guidance, Dead Cells toon shader, Octopath's "lost the appeal" warning).

### Gaps
- No primary developer source found for: Hollow Knight/Silksong blend modes, Sea of Stars' shipped feature list (Kickstarter blocked) or engine, Children of Morta normal-map pipeline, Hyper Light Drifter glow, Blasphemous and Owlboy lighting, Kingdom's lighting implementation, Starbound's algorithm.
- The Noita GDC 2019 talk content on lighting was not accessible; only wiki/mod-derived evidence.
- No player-perception study found; the "what players perceive as great" inferences rest on what developers chose to highlight in interviews and press kits.
