# Alternative (non-Radiance-Cascades) techniques for real-time 2D lighting and shadows — quality vs. cost scaling

> Researched 2026-10-08. Every source carries a date (publication / last edit) where one could be found.
> Consumer: Rust/wgpu 2D engine with compute shaders, mobile a priority.
> Hard rule kept: numbers only appear with the resolution/GPU/light count the source gave; where a source gave none, that is said explicitly.

## Q1 — SDF-based 2D GI: JFA distance field each frame + sphere-traced rays per pixel (Bigos, GM Shaders, "Noita-like", The Crypt, bevy_magic_light_2d)

### Takeaway
JFA itself is cheap and well-measured (~67–75 µs per full-screen pass at 1080p on an RTX 2080 Super, ≈0.6–0.7 ms for a full flood; ⌈log2(N)⌉ passes), but the per-pixel ray march that consumes it is the dominant and badly-scaling cost: published implementations use 32 rays/px at 360×240 (Bigos) up to 1000 rays/px at 320×240 (The Crypt, self-described "very demanding"), and none of the hobby write-ups publish frame times. Practical systems reduce it with probes (1 of 64 px) plus temporal accumulation (8 frames).

### Cited Findings
**JFA algorithm and pass count**
- Rong & Tan (I3D 2006) present jump flooding as a constant-number-of-rounds GPU algorithm to approximate the Voronoi diagram of seeds on a 2D grid and, from it, the distance transform; the rounds are reduced to logarithmic by halving the step length — [Rong & Tan 2006, project page](https://www.comp.nus.edu.sg/~tants/jfa.html) · [paper PDF](https://www.comp.nus.edu.sg/~tants/jfa/i3d06.pdf)
- Variants add one extra pass with step 1 at the end (JFA+1) to reduce errors — [Wikipedia: Jump flooding algorithm](https://en.wikipedia.org/wiki/Jump_flooding_algorithm) (accessed 2026-10-08)
- Bigos (2020-10-28): pass count = log2 of largest viewport dimension rounded up to a power of two; 768×512 → 10 passes; each pass samples a 3×3 neighbourhood at a halving offset — [Sam Bigos, "2D Global Illumination in Godot" part 1, 2020-10-28](https://samuelbigos.github.io/posts/2dgi1-2d-global-illumination-in-godot.html)

**JFA measured cost (the only hard numbers found)**
- Ben Golus (2020-07-18), RTX 2080 Super, 1920×1080: JFA outline **~0.6 ms at 30 px radius, ~0.7 ms at 100 px radius**, "full screen 1080p outlines for less than 1 ms"; cost is roughly independent of radius (it grows only with log2 of the radius/pass count) vs. brute force O(n²) (~10 ms at 20 px radius) and separable blur (~1.1 ms at 30 px) — [Ben Golus, "The Quest for Very Wide Outlines", 2020-07-18 (archived copy)](https://web.archive.org/web/2023/https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)
- Same source, per-pass breakdown: switching from RG32F to RG16F dropped the **init pass from 47 µs to 28 µs** and **each jump-flood pass from 75 µs to 67 µs**; the whole 50 px effect from 630 µs to 565 µs. RG16F caused sub-pixel offset precision issues, so he used **R16G16_SNorm** (same footprint, "ever so slightly slower"). "Both of these shader passes are completely hardware texture unit limited" — ALU tricks (squared distance instead of length) gave no measurable change — [Golus 2020](https://web.archive.org/web/2023/https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)
- Golus also notes that at 8K the pass count would only rise slightly, "ignoring the extra costs from memory bandwidth increases (hint: it'll be a lot more than 1 ms)" — i.e., JFA is bandwidth-bound and scales with pixel count × log2(N) — [Golus 2020](https://web.archive.org/web/2023/https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)
- For 3D volumes (not 2D), RTSDF reports a 256³ JFA volume at 21.89 ms and that "the duration of jump flooding becomes much higher with increasing SDF resolution" — [RTSDF, arXiv 2210.06160, 2022](https://arxiv.org/pdf/2210.06160) (3D; context only)

**Per-pixel GI ray march on the SDF**
- Bigos pipeline: emitters/occluders drawn into a viewport (any colour > 0 emits, any non-transparent pixel occludes) → JFA seed (UV in RG) → JFA passes → distance field → GI pass casting **32 rays/pixel, max 128 march steps**, hit threshold ≈0.001 (limited by fp16 precision), per-pixel/per-frame random start angle + golden-angle offset; example at **360×240** upscaled to 1280×720. **No timing numbers**; JFA described as cheap relative to the GI pass; each march step is a texture sample, "expensive"; no temporal accumulation, no denoise, **no bounce** (direct emissive light only; bounce promised for part 2, never linked); low-light pixels noisy; SDF in UV space is skewed on non-square viewports — [Bigos 2020-10-28](https://samuelbigos.github.io/posts/2dgi1-2d-global-illumination-in-godot.html)
- The Crypt (BuilderBot, Unity/HTML5, date not determinable from page): JFA distance field rebuilt every frame; **1000 rays per pixel by default at 320×240**; bounce light re-emitted tinted by surface colour; transparent emissives, partial shadowing, normal-map directionality, no baking. No measured numbers; "relatively powerful GPU" recommended; a commenter called it "very demanding" and the developer agreed it is "unacceptable" for now — [The Crypt – Lighting Demo, itch.io](https://builderbot.itch.io/the-crypt)
- bevy_magic_light_2d (2022–2023 copyright; supports Bevy 0.14–0.16): SDF of occluders in one channel (JFA listed as TODO, i.e. not yet implemented), direct light computed **at probes only (8×8 → 1 of every 64 pixels, "the main speedup")**, bounce pass sampling probe irradiance, **temporal blend of previous 8 frames**, optional edge-aware smoothing. No performance numbers; "heavily under construction"; emissive materials and offscreen-occluder SDF still TODO — [zaycev/bevy-magic-light-2d, GitHub](https://github.com/zaycev/bevy-magic-light-2d)
- Robert-K/light2D: Unity real-time 2D ray tracing on SDFs with JFA Voronoi distance field — [Robert-K/light2D, GitHub](https://github.com/Robert-K/light2D) (no numbers seen)
- Slembcke (2021-08-01) on this family: "a bit of a rabbit hole on how complicated you want to make it and how much GPU power you want to burn. It also requires representing your scenes as distance fields which is not trivial" — [Slembcke, "2D Lighting Techniques", 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Godot 4 ships an engine-generated 2D SDF from LightOccluder2D ("SDF Collision", on by default, zero cost if no shader reads it), queryable by `texture_sdf`/`screen_uv_to_sdf` for circle tracing; Linietsky lists 2D GI among suggested uses and GPU-particle collision — [Godot, "Godot's 2D engine gets several improvements for upcoming 4.0", Juan Linietsky, 2020-11-28](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40) · [Godot docs: 2D lights and shadows (stable)](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)
- Theory: Jarosz et al. 2012 analyse 2D light transport ("Theory, analysis and applications of 2D global illumination") — [Jarosz et al., ACM TOG 2012](https://cs.dartmouth.edu/~wjarosz/publications/jarosz12theory.html)
- Xor's well-known WebGL 2D GI shader shared in the Construct community is radiance-cascades-based, not SDF ray marching (multiple blur passes, no viewport scaling) — [Construct forum thread](https://www.construct.net/en/forum/game-development/tools-and-resources-27/global-illumination-ray-183906) (secondary/forum)

### Inferences
- Cost model: JFA ≈ (pixels) × ⌈log2(max dim)⌉ × 9 texel reads, texture-unit-bound; GI march ≈ (pixels or probes) × rays × steps texel reads. At Bigos's settings (32 rays × ≤128 steps) the march is up to ~4096 reads/pixel versus ~99 (11 passes × 9) for JFA at 1080p, which is consistent with "JFA cheap, GI pass expensive". This is my arithmetic, not a measured number.
- Quality: per-pixel Monte-Carlo direct light from arbitrary emissive shapes gives physically-shaped soft shadows/penumbrae for area emitters for free, and bounces with probes/feedback — but noise ∝ 1/√rays forces either many rays, low resolution (all examples are ≤ 360×240), or temporal + spatial denoise. This is the noise/cost problem RC was designed to solve (out of scope here).
- Mobile: a 16-bit RG JFA buffer (R16G16_SNorm/UNorm) halves bandwidth vs RG32F, and Golus' result says it's bandwidth/texture-bound — directly relevant to tile-based mobile GPUs. wgpu note: R16G16Snorm storage support is not universal; R16G16Uint or packed u32 coordinates would be the portable choice (inference).

### Gaps
- No source gave a measured frame time for SDF ray-marched 2D GI (Bigos, The Crypt, bevy_magic_light_2d, light2D all publish none). Mobile numbers for JFA: none found.
- Samuel Bigos' part 2 (bounces) was never found; the repo `github.com/samuelbigos/godot_2d_global_illumination` is referenced but its contents were not verified.
- GM Shaders (Xor) specific 2D-GI article was not located; Noita's lighting technique has no primary source found.

## Q2 — Per-light SDF soft shadows (one SDF, cone/sphere-traced soft shadow per light, Quilez-style)

### Takeaway
One shared SDF (built once by JFA) lets each light's shadow be a single march from pixel to light with the k·h/t penumbra estimator; cost is roughly (lit pixels per light) × (march steps), with no per-occluder term. It's an approximation, not an exact penumbra, with banding/leaking at sharp corners that Aaltonen's improved estimator reduces.

### Cited Findings
- Quilez: march toward the light and keep min(k·h/t) where h = SDF distance, t = distance travelled; k ≈ inverse light angular size (larger k = harder). Cost is essentially the cost of the raymarch (loop capped at 256 in his example, one ray per pixel); the arithmetic is negligible vs evaluating the SDF — [Inigo Quilez, "soft shadows in raymarched SDFs" (original example captioned 2010)](https://iquilezles.org/articles/rmshadows/)
- Artifacts: discrete steps can miss the darkest point → light leaking/banding following the step pattern, worst at sharp corners. Sebastian Aaltonen's improvement (GDC, "7 years after" the original, i.e. ~2017) triangulates the true closest approach from the current and previous SDF samples; an inner-penumbra extension (after Shadertoy user nurof3n) is "pretty close" to ground truth — only visual comparisons, no error numbers — [Quilez](https://iquilezles.org/articles/rmshadows/)
- Rong & Tan also applied JFA to image-based soft shadows ("Utilizing jump flooding in image-based soft shadows", ACM VRST 2006) — [p173-rong.pdf](https://www.comp.nus.edu.sg/~tants/softShadow/p173-rong.pdf) (3D image-space; not read in detail)
- Godot 4's 2D SDF is explicitly intended for shader circle tracing (example: long drop-shadows/light shafts stepping through `texture_sdf`) — [Linietsky 2020-11-28](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40)

### Inferences
- Scaling: O(lights × covered pixels × steps), independent of occluder count once the SDF exists (SDF cost is per-frame, shared). With a tiled light list (Q5) each pixel marches only for the lights binned to its tile. Shadows from emissive *surfaces* lighting neighbours are not handled — each light is still a point/disc with an analytic radius.
- In 2D a march from pixel to light is bounded by the pixel-to-light distance in texels; the sphere-tracing speedup is large in open space and degrades near walls/along grazing corridors (general sphere-tracing behaviour, inference).

### Gaps
- No 2D-specific measured per-light cost (ms per light at a resolution/GPU) was found.

## Q3 — Geometry shadows: shadow volumes / visibility polygons from occluder edges, soft-edge tricks (penumbra wedges/fins, Slembcke's SFSS), box2dlights

### Takeaway
Edge-extruded shadow quads (hard) and Slembcke's Super Fast Soft Shadows (one quad per segment per light, penumbra/umbra/antumbra computed as additive linear gradients in the vertex/fragment shader) give the most accurate area-light penumbrae for polygonal occluders at the lowest cost — no per-pixel search, ~O(lights × edges) vertex work plus fill. SFSS runs with soft shadows + light fields at 60 Hz on a Raspberry Pi 4 (author's claim). Its known inaccuracy: occlusion is combined by subtraction, which is wrong for non-adjacent occluders overlapping in the light's angular extent. CPU raycast visibility polygons (box2dlights, Red Blob) are CPU-bound and give hard edges unless blurred.

### Cited Findings
**Slembcke series (2021-08) — Super Fast Soft Shadows (SFSS)**
- Survey of 2D techniques with pros/cons: forward (draws ≈ lights × objects), deferred (lights + objects), screen-space lightmaps ("extremely fast", colour × light, no normals), hard shadow geometry (each segment → quad extruded from light; used by Gish), shadow fins ("too fiddly", penumbra "popping"), SFSS, screen-space light fields (per-pixel Fourier-series probe for normal mapping), light-space shadows ("very expensive, many passes per light"), grid raycasting (Starbound), SDF ray marching — [Slembcke, "2D Lighting Techniques", 2021-08-01 (updated 2025-01-31)](https://www.slembcke.net/blog/2DLightingTechniques/)
- "I render shadows at 1/4 resolution for performance on low end machines such as the Raspberry Pi"; Veridian Expanse developed "to run with full light fields and soft shadows at 60 hz on a Raspberry Pi 4" (author claim; resolution and light count not stated) — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Visibility polygons: "I don't think this approach makes a lot of sense anymore though as it uses the CPU heavily instead of GPU, and is not easy to implement robustly"; physics-raycast polygons are "very expensive and has popping artifacts if you don't shoot enough rays" — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- SFSS mechanism: penumbra bounds by lines through the occluder corner tangent to the light disc; shadow strength = fraction of light area covered; along any line through the corner the fraction is constant → it is linear math; umbra/antumbra = sum of gradients from both endpoints; **one quad per segment**, vertex attributes = both endpoints + 2D shadow coord, infinite projection via homogeneous w; CPU batches geometry once, "No further processing or per-light calculations are required" on CPU; needs indexed or instanced geometry (no triangle strips) — [Slembcke, "2D Lighting with Soft Shadows", 2021-08-09](https://www.slembcke.net/blog/SuperFastSoftShadows/)
- SFSS history: born ~2015 in Cocos2D-SpriteBuilder (mobile-oriented framework), sold as a Unity asset, revived for Veridian Expanse — [Slembcke 2021-08-09](https://www.slembcke.net/blog/SuperFastSoftShadows/)
- SFSS limitations: uses a tangent-point *approximation* ("not quite correct"); adjacent segment masks add exactly but numerical precision leaves speckled cracks → fudge `penumbra -= 1/64`; **"Subtracting Shadows is Wrong"** for two separate objects covering the same half of the light at different distances (over-darkening; "there isn't an easy solution"; "very subtle"); clipping when the light straddles a segment's plane causes subtle artifacts; HDR lightmaps need a second-pass trick with "alpha saturate" blending; area lights exacerbate the overlap problem — [Slembcke 2021-08-09](https://www.slembcke.net/blog/SuperFastSoftShadows/)
- SFSS "light penetration" (light enters surfaces slightly) lets the lightmap be rendered at reduced resolution "without sacrificing any noticeable details and vastly decreasing the performance cost" — [Slembcke 2021-08-09](https://www.slembcke.net/blog/SuperFastSoftShadows/)
- A SpriteKit/Metal port projects shadow geometry per edge in the vertex shader and computes per-pixel shadow opacity in the fragment shader, based on Lembcke's algorithm — [Apple Developer Forums, "2D Soft Shadows with SpriteKit and Metal"](https://developer.apple.com/forums/thread/841671)

**Penumbra wedges (3D origin, the academic ancestor)**
- Akenine-Möller & Assarsson (EGWR 2002): one penumbra wedge per silhouette edge replaces the shadow-volume quad — [Chalmers PDF](https://www.cse.chalmers.se/~uffe/softshadows_egrw.pdf); Assarsson & Akenine-Möller (ACM TOG 2003): results "most often indistinguishable" from the average of **1024 hard shadow images** — [Graphics Hardware 2003 PDF](https://www.cse.chalmers.se/~uffe/soft_gfxhw2003.pdf) · [LTH](https://fileadmin.cs.lth.se/graphics/research/papers/soft2003); GH 2003 optimizations: >150 fps simple models, ~50 fps complex (2003 hardware) — same source
- Older 2D geometric soft shadows (fins, penumbra size from the light's physical size, not its range) — [Orangy Tang, "Dynamic 2D Soft Shadows", GameDev.net](https://archive.gamedev.net/archive/reference/articles/article2032.html)

**Visibility polygons (Red Blob, Sight & Light)**
- Red Blob (2012, updated 2018/2020): rotational sweep over wall endpoints sorted by angle keeps the nearest wall, emits triangles; run once per light, union for lighting; GPU AND = multiply, OR = add+clamp; low-res light bitmap can be upscaled; output is **hard** ("post-process the output to get soft shadows"); CGAL's algorithm was measured "50X faster" than his sweep; no Big-O stated — [Amit Patel, Red Blob Games, "2d Visibility"](https://www.redblobgames.com/articles/visibility/)
- ncase's "Sight and Light" (cast rays to wall vertices) recommended as an optimisation in The Crypt comments — [The Crypt](https://builderbot.itch.io/the-crypt)

**box2dlights (libGDX)**
- CPU Box2D raycasts per light: "For each ray box2dLights have to use 1 raycast and 1 vertex + 2 for soft light"; recommended **5–128 rays** per dynamic light; `xray` (no raycasting) cuts CPU load "by about 80%"; static lights very cheap; soft-light distance costs CPU+GPU — [box2dlights wiki, Performance Tuning (wiki home last edited 2015-02-17)](https://github.com/libgdx/box2dlights/wiki)
- Lightmap FBO size: 800×480 screen at full-size FBO with 3 blur passes = >10 M texture lookups; default **200×120 (quarter size) + 1 blur pass = 240 000 lookups** and looks smoother; each blur pass = 5+5 fetches — [box2dlights Performance-Tuning.md](https://raw.githubusercontent.com/wiki/libgdx/box2dlights/Performance-Tuning.md)

### Inferences
- Cost scaling: hard/SFSS = O(lights × occluder segments) vertices (instancing removes CPU cost) + O(lights × shadowed pixels) fill; no per-pixel loops. With per-light culling of segments to the light radius, cost ≈ Σ_lights (segments in radius). Fill dominates at high resolution — hence everyone renders the lightmap at ¼–½ res (Slembcke ¼, box2dlights ¼ per axis, Unity ½).
- Quality: correct analytic penumbra/umbra/antumbra for disc lights and polygon occluders; no GI, no bounce, emissive *surfaces* cannot light neighbours (lights are explicit discs). Needs occluder outlines (polylines) — a content cost SDF methods avoid.
- On mobile this family is the cheapest high-quality option for many dynamic lights with soft shadows (SFSS was designed on Cocos2D for mobile).

### Gaps
- No published ms-per-light or ms-per-1000-edges benchmark for SFSS or box2dlights with GPU + resolution was found.
- Nicky Case's "Sight & Light" page was not fetched directly.

## Q4 — 1D polar shadow maps (Catalin Zima / mattdesl; Godot)

### Takeaway
Per light: render occluders into a light-centred square FBO, reduce to a 1D polar "nearest occluder per angle" texture, then shade a light quad by comparing radius vs stored distance with an angular blur for soft edges. Pixel-perfect for sprite alpha occluders, but needs ≥ 2 passes per light and is fill-rate bound: ~20 lights on a GeForce 9400 (2011-era) before fill-rate limits.

### Cited Findings
- mattdesl (LibGDX): occlusion map in a 256×256 FBO around the light (alpha only, threshold 0.75) → 256×1 shadow map (256 angular rays, nearest normalised distance, 1.0 when empty) → light quad with polar lookup and additive blending; soft shadows via **9 samples along the angle axis** (Gaussian weights) with blur width growing with distance from light, plus smoothstep radial falloff; author reports **~20 lights before fill-rate becomes the bottleneck on a GeForce 9400 (Mac OS X 10.6.6)**; code "unoptimized", poor on Android; suggested optimisations: atlas the occlusion maps, pack shadow maps into RGBA channels, smaller maps + upscale, separable blur, disc meshes for lights — [mattdesl, "2D Pixel Perfect Shadows", lwjgl-basics wiki, last edited 2013-06-15](https://github.com/mattdesl/lwjgl-basics/wiki/2D-Pixel-Perfect-Shadows)
- Godot: 3.x drew each light in a separate pass ("main constraint … performance"); 4.0 draws all lights in a single pass and puts **all 2D shadows and light textures in one atlas** — [Linietsky 2020-11-28](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40)
- Godot 4 shadow filters: None (cheapest, blocky), PCF5, **PCF13 ("most expensive", use on only a few lights at once)**; high Filter Smooth causes banding/streaking; directional shadows always infinitely long ("limitation of Godot's 2D shadow method"); larger lights cost more (screen pixels covered); lighting computed at viewport resolution — [Godot docs, 2D lights and shadows (stable)](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)

### Inferences
- Cost per light = occluder raster into light FBO + (if done by search) up to R² texel reads to build an R-wide polar map + light-quad fill × blur taps. With compute, the polar reduction could be done by rasterising occluder *edges* directly into angular bins (atomicMin of distance), removing the R² search (inference, not from sources).
- Penumbra is a filter approximation (blur grows with distance), not physically tied to light size; contact hardening is approximate. Godot's polar approach is why its directional shadows are infinite.
- Godot's exact polar map resolution is a project setting not documented on the fetched page (gap).

### Gaps
- Catalin Zima's original XNA article was not fetched. No modern (post-2015) per-light ms numbers for 1D shadow maps with GPU/resolution found.

## Q5 — Deferred 2D lighting, reduced-res light buffers, light volumes, tiled/clustered culling for thousands of lights

### Takeaway
2D engines universally render lighting into a reduced-resolution light buffer (Unity default ½ screen, box2dlights ¼ per axis, Slembcke ¼ on Raspberry Pi). For many lights, tiled compute binning from 3D Forward+/tiled-deferred carries over directly (2D can skip the depth min/max); the measured reference points are 3D: 1024 point lights at 1280×720 ≈ 9.3 ms → 7.4 ms on a mid-range DX11 GPU. No 2D-specific thousands-of-lights benchmark exists in the sources found.

### Cited Findings
- Deferred in 2D reduces draws from lights × objects to lights + objects; Cryptark is a 2D deferred example; cons: hard with multiple materials, no alpha blending, higher base cost — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Screen-space light fields (Slembcke): per-pixel Fourier-series lit-sphere probe stored in MRTs, sampled with the sprite normal; "bandwidth intensive, but can be sub-sampled", diffuse only; works with lightmap shadows; runs at 60 Hz on a Raspberry Pi 4 in Veridian Expanse — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Unity URP 2D: light render texture **Render Scale default 0.5** ("half screen size resolution provides a good performance with almost no noticeable artifact in most situations"); each distinct blend style = own light render texture (use 1–2); **Max Light Render Textures default 4, Max Shadow Render Textures default 1**; light batching merges sorting layers that need exactly the same lights — [Unity 6 docs, "Optimize 2D lights"](https://docs.unity.com/en-us/engine/6000.7/manual/unity2d/2d-urp/2d-index/2d-light-batching-debugger) · [Unity, Optimizing 2D lights](https://docs-alpha.unity3d.com/Manual/urp/2d-lights-optimize-methods.html); a user reports the Frame Debugger still shows one batch per added light despite the debugger claiming batching (unconfirmed) — [Unity Discussions](https://discussions.unity.com/t/why-arent-2d-lights-_actually_-batched/922877)
- Godot 4: all 2D lights in a single pass (vs. per-light passes in 3.x) — [Linietsky 2020-11-28](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40)
- 3D tiled deferred reference: **1024 densely packed point lights, 1280×720, mid-range DX11 card: 9.34 ms (107.7 fps), later 7.4 ms**; culling only by the tiled compute pass (GPU model unspecified) — [GameDev.net blog, "Tile-based deferred shading"](https://www.gamedev.net/blog/1546/entry-2256528-tile-based-deferred-shading/)
- Forward+ hobby benchmark, 1024 lights: 16×16 tiles best; 8×8 more accurate but diminishing returns; large radii reduce culling effectiveness — [shakesoda/Forward-Plus-Renderer, GitHub](https://github.com/shakesoda/Forward-Plus-Renderer)
- Tile size: Black Rock (Game Engine Gems) found 4×4 best for their classification/shader-complexity balance — [MCV/Develop excerpt](https://www.mcvuk.com/development-news/tutorial-deferred-shading/); Intel's SIGGRAPH 2010 compute tiled deferred sample — [Intel, "Deferred Rendering for Current and Future Rendering Pipelines"](https://www.intel.com/content/www/us/en/developer/articles/technical/deferred-rendering-for-current-and-future-rendering-pipelines.html); Wicked Engine on optimising tile culling — [Wicked Engine blog, 2018-01](https://wickedengine.net/2018/01/optimizing-tile-based-light-culling); clustered shading — [Olsson et al., Chalmers](https://www.cse.chalmers.se/~uffe/clustered_shading_preprint.pdf)
- WebGPU clustered lighting (luma.gl): atomically marked candidate-light bitmasks compacted into a bounded per-cluster index list with an overflow fallback — [luma.gl clustered lighting docs](https://luma.gl/docs/api-reference/experimental/clustered-lighting)

### Inferences
- For 2D, tiles need no depth bounds; light-vs-tile test is circle-vs-rect, so binning cost ≈ Σ tiles touched by lights (atomic append per overlap), not lights × tiles. A WGSL compute binning pass + shading pass is the direct port. Shadows remain the hard part: tiled lists solve unshadowed lights; shadowed lights need one of Q2–Q4 per light (SDF march per binned light is the natural compute pairing).
- Mobile: tile-based GPUs favour a single pass over a reduced-res light buffer with few render-target switches; Godot's single-pass + atlas and Unity's ½-res texture are consistent with that.

### Gaps
- No 2D benchmark with thousands of lights (ms, GPU, resolution) was found; the closest are 3D numbers from ~2011–2013 hardware.

## Q6 — Light propagation on a grid (Terraria/Starbound/Minecraft-2D flood fill, LPV-style)

### Takeaway
Tile games propagate light per tile (BFS flood from each source, attenuating per step; Minecraft keeps separate sky and block channels). It's cheap at tile resolution and handles light bending around corners and coloured light, but gives blocky, Manhattan-diamond falloff and no sharp shadows; no primary developer source with numbers was found.

### Cited Findings
- Flood-fill lighting = BFS propagation of light values across faces; Minecraft tracks two channels (sky light by time of day, block light from torches); bit-level word parallelism can speed it up — [Mikola Lysenko, 0fps.net, "Voxel lighting", 2018-02](https://0fps.net/2018/02)
- Terraria's lighting modes: "Color" computes light at sub-tile pixel clusters (smoother, heavier), "Retro" per tile (blocky); lighting mode affects performance (no algorithm documented) — [Terraria Wiki, Lighting mode](https://terraria.fandom.com/wiki/Lighting_mode)
- Slembcke: grid raycasting "still used in some tile based games … Starbound's tile based lighting" with texture filtering and multisampling — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Community Unity re-implementations: Starbound-style = queue-based box fill into a world-overlay texture, ~500 lines, no shaders; Manhattan distance causes the diamond shape — [Unity Discussions, "Terraria / Starbound Style Lighting (Round 2)"](https://discussions.unity.com/t/terraria-starbound-style-lighting-round-2/590349) · [Unity Discussions, "Terraria like 2D block based lighting"](https://discussions.unity.com/t/terraria-like-2d-block-based-lighting-terraria-like-block-based-lighting/248547) (forum, low authority)

### Inferences
- Cost ∝ (grid cells reached) × (sources), or ∝ cells × iterations for a Jacobi-style GPU propagation; at tile resolution (e.g. 120×68 tiles for 1080p at 16 px tiles) it's negligible on any GPU. Quality ceiling is low (no penumbra correctness, light wraps around corners — behaves like diffusion, not transport). An LPV-style directional propagation would reduce wrap-around but no 2D implementation with numbers was found.

### Gaps
- No Terraria/Starbound/Re-Logic/Chucklefish primary source; no 2D LPV implementation with measured numbers found.

## Q7 — Lightmaps / baking for static 2D scenes and hybrids; industry/academic comparisons 2015–2026

### Takeaway
No dedicated 2D baking source was found in this pass. The closest "static" levers are engine-level: box2dlights static lights ("very cheap", re-update only on change) and Godot's engine SDF shared by all lights. No GDC talk comparing 2D lighting families was found; Slembcke's 2021 survey is the best practitioner comparison, and he notes his SFSS GDC proposal was rejected twice.

### Cited Findings
- box2dlights: static flag makes a light "very cheap" (recomputed only when its state changes); disabled lights have zero overhead — [box2dlights Performance-Tuning.md](https://raw.githubusercontent.com/wiki/libgdx/box2dlights/Performance-Tuning.md)
- Slembcke pitched the SFSS article "as a GDC presentation twice" and it was rejected ("2D lighting is pretty niche") — [Slembcke 2021-08-09](https://www.slembcke.net/blog/SuperFastSoftShadows/)
- Practitioner comparison of 2D families (forward, deferred, screen-space lightmaps, hard geometry, fins, SFSS, light fields, light-space shadows, grid raycasting, SDF marching) — [Slembcke 2021-08-01](https://www.slembcke.net/blog/2DLightingTechniques/)
- Academic 2D GI theory — [Jarosz et al. 2012](https://cs.dartmouth.edu/~wjarosz/publications/jarosz12theory.html)

### Inferences
- A hybrid for a wgpu engine: cache static occluders' SDF/JFA seeds (or static shadow quads) and re-flood only when dirty; per-light shadow buffers for static lights can be cached across frames (box2dlights' static-light idea generalised).
- Summary comparison (synthesised from findings above, not a single source):

| Family | Shadow quality | Penumbra correctness | GI/bounce | Emissive surfaces light neighbours | Cost scaling |
|---|---|---|---|---|---|
| Hard shadow quads | hard | n/a | no | no | lights × edges (vertex) + fill |
| SFSS (Slembcke) | soft, analytic | near-correct for disc light; wrong for non-adjacent overlapping occluders | no | no | lights × edges + fill; ¼-res OK; RPi4 60 Hz claim |
| Visibility polygon (CPU) | hard (blur for soft) | none | no | no | CPU: lights × edges·log; GPU fill |
| box2dlights | ray-fan, blur-soft | none (blur) | no | no | CPU: lights × rays (5–128) raycasts |
| 1D polar shadow map | blur-soft | approximate | no | no | ≥2 passes per light; ~20 lights / GeForce 9400 |
| Per-light SDF march (Quilez) | soft | approximate (k·h/t) | no | no | JFA once + lights × pixels × steps |
| SDF GI (JFA + rays/px) | soft from area emitters | physically shaped, noisy | yes (with feedback/probes) | yes | JFA ≈ px × log2 N (~0.6–0.7 ms 1080p RTX 2080S) + px × rays × steps (dominant; examples ≤360×240) |
| Tiled/deferred many lights | (shadows separate) | — | no | no | binning ≈ light-tile overlaps; 1024 lights 720p ≈ 7–9 ms (3D, 2013 mid DX11) |
| Grid flood fill | none (diffuse wrap) | none | crude (light bends) | yes (as sources) | cells × sources at tile res; tiny |

### Gaps
- No 2D baked-lightmap tool/paper found; no GDC Vault talk (2015–2026) comparing 2D lighting families located; no mobile-GPU measurements for any of the families. These are the main holes for the downstream report — the only way to fill them is to measure in-engine.
