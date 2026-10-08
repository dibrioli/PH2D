# Best-in-class 2D lighting beyond Godot 4 — middleware, shipped games, mobile, research frontier (as of 2026-10-08)

> Scope note: Godot 4, Unity URP 2D basic Light2D, Radiance Cascades (RC) basics, JFA-SDF GI, Lembcke soft
> shadows and polar 1D shadow maps are covered in sibling notes; they are only referenced here.
> Research budget: ~35 tool calls. Many primary sources (GDC Vault slides, Google-Docs manuals, Steam
> announcements) were behind logins or returned 403, so a lot of the "how" for shipped games remains a gap —
> said explicitly below rather than filled from memory.

## 1. Middleware / plugins / libraries regarded as better than the engine default

### Takeaway
No middleware found publishes independent, device-named mobile measurements. The commercial leader on Unity
(Smart Lighting 2D, FunkyCode) only offers a vendor claim ("3–5× faster than URP 2D lights"); the open-source
leaders are Bevy crates (Firefly, bevy_light_2d, bevy_magic_light_2d, solis_2d) and GameMaker's Bulb (MIT) —
all permissively licensed (usable as reference code for PH2D, not only as oracles), none with published
numbers. The only technically distinctive designs are those that do GI (bevy_magic_light_2d: SDF + probe
irradiance cache with bounces; solis_2d: RC); the rest are direct-light + occluder-shadow systems.

### Cited Findings
**Smart Lighting 2D (Unity, FunkyCode) — proprietary, $29**
- Asset Store listing: publisher FunkyCode, $29, latest version 2022.11.0 released 14 Nov 2022, 224 reviews,
  2,390 favourites, original Unity version 2018.4.36; page itself (as fetched) states no technique — [Unity Asset Store, fetched 2026-10-08](https://assetstore.unity.com/packages/slug/112535)
- Vendor claim (snippet of the same listing via search): designed for "as little performance impact as possible
  for mobile devices", culls to on-screen lights/casters, output (lightmap) resolution tweakable, and "extensive
  testing showed SL2D lighting is 3-5 times faster than Unity URP 2D lights" — developer claim, unverified,
  no device/resolution given — [Unity Asset Store](https://assetstore.unity.com/packages/slug/112535)
- Architecture per listing: "create multiple lightmaps with personalized light logic"; lighting displayed as
  overlay or via shaders/Shader Graph; directional "day lighting" light type; itch.io page lists soft shadows,
  day shadows driven by sun angle, day-shadow height, custom darkness — [itch.io Smart Lighting 2D](https://qzia.itch.io/smart-lighting-2d)
- 80.lv calls it mobile-friendly ("great performance on Android & iOS devices", no GC after init) — this is a
  promotional partner piece, not a benchmark — [80.lv, "5 Great Tools for Lighting Setup in Unity"](https://80.lv/articles/5-great-tools-for-lighting-setup-in-unity)
- Anecdote on the baseline it competes with: an indie dev measured Unity's built-in Light2D at 6 fps on their
  phone and a SpriteRenderer-based additive light at 40–50 fps (device unnamed) — [itch.io devlog "2D Mobile Optimized Lighting System"](https://gamba04.itch.io/my-personal-devlog/devlog/145714/2d-mobile-optimized-lighting-system)

**Bulb (GameMaker, Juju Adams) — MIT**
- "2D lighting and shadows for GameMaker", version 22.0.7, targets GameMaker 2024.8 and 2022 LTS, MIT license,
  106 stars; README states no technique or performance numbers; docs at jujuadams.github.io/Bulb — [GitHub JujuAdams/Bulb, fetched 2026-10-08](https://github.com/JujuAdams/Bulb)

**Firefly (`bevy_firefly`, Bevy) — MIT/Apache-2.0**
- Features: point lights, round and polygonal occluders, transparent and coloured occluders, soft shadows
  (`Softness` 0..1) and hard-shadow/"light-banding" looks, normal maps, occlusion z-sorting (shadows only on
  sprites with lower z than the caster), multiple lightmaps, render layers; WebGPU-only; no normal maps/z-sort
  for meshes yet; occluder scaling unsupported; versions track Bevy 0.16–0.19; showcase videos dated
  2025-11-19 → 2026-01-01; no performance numbers (a "stress" example exists without metrics); author building it
  as a college thesis — [GitHub PVDoriginal/firefly](https://github.com/PVDoriginal/firefly); [docs.rs bevy_firefly](https://docs.rs/bevy_firefly/)

**bevy_light_2d — MIT**
- v0.10.0 (16 Sep 2026), Bevy 0.19; point lights, occlusion, dynamic shadows, per-camera ambient; WebGL2 and
  WebGPU; technique not stated in docs — [docs.rs bevy_light_2d](https://docs.rs/bevy_light_2d/)

**bevy_magic_light_2d — 2D GI (experimental)**
- Pipeline: (1) SDF of occluders into one-channel texture (JFA); (2) direct irradiance at screen-space probes,
  occlusion via SDF sampling, 1/d² falloff; (3) bounce pass sampling irradiance stored in probes with
  exponential sampling; (4) composite with temporal blend of the previous 8 frames + optional edge-aware smoothing.
  Lighting computed for only 1 pixel per 8×8 probe block (1/64), interpolated; configurable bounces, transparent
  occluders, colour transfer from occluders; "heavily under construction" — [GitHub zaycev/bevy-magic-light-2d](https://github.com/zaycev/bevy-magic-light-2d)

**solis_2d — RC 2D GI**
- "Realistic 2D light, shadow and normal map calculation using an optimized version of radiance cascade";
  v0.1.0/0.1.1 published 12 Oct 2024; Bevy 0.14; MIT/Apache-2.0; WIP, effectively unmaintained — [lib.rs solis_2d](https://lib.rs/crates/solis_2d); [docs.rs solis_2d](https://docs.rs/solis_2d)

**bevy_lit**
- Uses SDFs for occluder distance and blurs to soften; its author says this "is not ideal and might have
  limitations in terms of performance and visual accuracy" — [docs.rs bevy_lit](https://docs.rs/bevy_lit)

**Other open RC 2D implementations with numbers (desktop only)**
- Raylib RC 2D GI implementation: 60 fps at 1200×900 with 6 cascades on "mid-range GPU (e.g. GTX 1660)";
  cost linear in cascade count and ray range; claims OpenGL 3.3+ desktop & mobile but warns older GPUs may stall
  on cascade-merge texture lookups — [GitHub Hybrid46/RadianceCascade2DGlobalIllumination](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination)
- GM Shaders write-up (Xor & Alex): direction-first probe layout doubled performance vs position-first on an
  RTX 3080 — [GM Shaders "Radiance Cascades 2"](https://mini.gmshaders.com/p/radiance-cascades2)
- Mytino (indie dev, X post, Nov 2023): RC 2D GI "ca. 0.3 ms per frame (GTX 970)", no denoising/temporal,
  from scratch each frame (resolution not stated) — [X @MytinoDev](https://x.com/MytinoDev/status/1722365866979774530)
- GLLight2D: small open real-time 2D path tracer with GI (SFML/OpenGL), hobby-grade — [GitHub 222464/GLLight2D](https://github.com/222464/GLLight2D)

### Inferences
- "Best" middleware is a reputation for workflow + speed vs URP 2D, not for a different light-transport
  model: Smart Lighting 2D, Bulb, Firefly and bevy_light_2d are direct-light + occluder-shadow systems (the
  family already covered by the Lembcke / shadow-map notes). The quality ceiling above Godot comes only from
  GI-class methods (probe caches, RC).
- Smart Lighting 2D's "multiple lightmaps with custom logic + adjustable lightmap resolution" is the concrete
  mobile lever: decoupling light resolution from screen resolution. That is a design lesson, not a number.
- Every permissively licensed candidate (Bulb MIT, Firefly MIT/Apache, bevy_light_2d MIT, solis_2d MIT/Apache)
  can be read as code by PH2D; Smart Lighting 2D is proprietary → black-box oracle only (Unity runtime needed).

### Gaps
- No device-named mobile measurement for any middleware. Smart Lighting 2D's Google-Docs manual and forum
  thread were not reachable; its shadow algorithm is undocumented in what I could fetch.
- Bulb's technique (README silent) — docs site not fetched.
- Not researched for lack of budget: Defold lighting extensions, Heaps/HaxeFlixel lighting, SpriteLamp runtime,
  "Ultimate 2D Lighting", HardLight2D, "Nova". No evidence collected either way on whether they are regarded as
  best-in-class.

## 2. Shipped 2D/2.5D games with the most admired lighting — how they render it

### Takeaway
The most-admired shipped 2D/2.5D lighting is overwhelmingly **art-directed and faked**, not physically
simulated: INSIDE uses hand-placed decals for bounce/AO/cast shadows ("no global illumination or even automated
AO"), Ori 2 uses ~30,000 hand-painted per-asset light-direction maps. The only confirmed shipped real-time GI
from this family is **Path of Exile 2** (RC in screen space, Sannikov) — an isometric 3D game, not 2D. No shipped
2D game using ReSTIR/path-traced GI was found.

### Cited Findings
**INSIDE (Playdead, heavily modified Unity, 2016)**
- Lighting = layered hand-placed effects: decals for bounce light, specular, rim light, AO and cast shadows; cube
  maps for reflections; "there is no global illumination or even automated AO at play here"; point-light shadow
  casting "would have been much too expensive"; torch shadows at the boy's feet are decals that track the torch,
  rotate and scale with its height; cookies used later for textured shadows. Rationale: painterly look, predictable
  side-scroller camera allows precise tuning — [Playdead blog, Marek Bogdan, 2016](https://blog.playdead.com/articles/the_lighting_of_inside/lighting_of_inside.html)
- GDC Europe 2016 talk (Gjøl & Svendsen): lighting split into diffuse, specular and bounce components; analytic
  primitive-based AO; SSR; locally shadowed volumetrics; dithering to kill banding in subtle gradients — [GDC Vault "Low Complexity, High Fidelity – INSIDE Rendering"](https://gdcvault.com/play/1023002/Low-Complexity-High-Fidelity-INSIDE); [80.lv summary](https://80.lv/articles/the-rendering-of-inside-low-complexity-high-fidelity/?amp=1)

**Ori and the Will of the Wisps (Moon Studios, Unity, 2020)**
- "Every texture in the game reacts to lighting sources"; light masking painted on every asset; a light-direction
  map made per asset — ~7,000 assets, ~30,000 hand-drawn light maps; Van Leeuwen said hand-painting beat
  generating normal maps from scripts or 3D models — [TheGamer](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/); [Wccftech](https://wccftech.com/ori-and-the-will-of-the-wisps-dynamic-lighting/amp/)
- GDC 2021 talk "The Art of Ori and the Will of the Wisps" (Gritton, van Leeuwen) — slides not accessible — [GDC Vault](https://gdcvault.com/play/1027035/The-Art-of-Ori-and)

**Path of Exile 2 (Grinding Gear Games, custom engine)**
- ExileCon 2023 talk on PoE2 rendering by Alexander Sannikov (Senior Graphics Programmer), 1 Nov 2023 — [Inara summary](https://inara.cz/morps/article/31103)
- PoE2 uses RC for screen-space illumination (secondary statement in a search summary; Sannikov is the RC
  author) — see also [Chalmers thesis on RC SSGI](https://odr.chalmers.se/items/9e42007a-bb60-440f-ba28-04410822758f/full)
- No published per-pass GPU timing found; a 2026 Japanese settings guide recommends disabling "global
  illumination" for stable endgame fps (not explicitly tied to RC) — [gaming-st.com, 2026](https://gaming-st.com/trouble-settings/poe2-performance-fix/)

**Planet of Lana I/II (Wishfully, Unity)**
- Unity blog interview (14 May 2026; PoL II launched 5 Mar 2026): target 60 fps "on most platforms", low-spec
  baseline profiling, draw calls flagged, Sprite Atlas size limits, per-device quality tiers; "Unity's built-in
  settings" with a custom quality system. No lighting/render-pipeline detail. Platforms: PC, Xbox One/Series,
  PS4/PS5, Switch — **no mobile** — [Unity blog](https://unity.com/blog/wishfully-planet-of-lana-II)

**Children of Morta (Dead Mage, Unity)**
- Hand-painted pixel art "partnered with modern lighting techniques"; reviewers cite dynamic lighting and specular
  maps; team lead Amir Fassihi: chose pixel art but "didn't constrain themselves technology-wise" (advanced
  lighting, shadows, post) — [Co-Optimus interview](https://www.co-optimus.com/editorial/1448/page/1/indie-ana-co-op-and-the-children-of-morta-interview.html); [Engadget, 2018](https://www.engadget.com/2018/03/22/children-of-morta-hands-on-indie-xbox-gdc/)
- Academic context: normal-mapped pixel art needs specific methods because 3D normal-map generation "may not yield
  correct results" on pixel art — [SBGames 2022 paper](https://sol.sbc.org.br/index.php/sbgames/article/view/23463)

**The Last Night (Odd Tales)**
- Pixel art "upgraded" with lighting & post-processing since the jam version; canvas raised from 360×150 to
  640×270 per scene with "complex dynamic lighting" as a target — [Thread Reader, Tim Soret posts](https://threadreaderapp.com/thread/976819508441681920.html)

**Hades / Hades II (Supergiant, custom engine)**
- Custom C# engine ported to C++ for performance from ~2020; in-house rendering code for Hades II — [DigiPen](https://www.digipen.edu/showcase/news/hades-2-graphic-programmer)
- Claim that the engine is built on ConfettiFX "The Forge" comes only from a Steam user post — unconfirmed — [Steam community](https://steamcommunity.com/app/1145350/discussions/0/4361248648377497796/?l=tchinese)

**Rayman Legends (UbiArt Framework)**
- Only marketing-level: "new lighting design and new rendering system" for Legends; Ancel: consoles limited for
  lights/shadows, motivating a full-2D approach — [GamingTrend](https://gamingtrend.com/news/rayman-legends-features-the-same-2d-hand-drawn-visuals-with-new-3d-models-for-spice); [Shacknews](https://shacknews.com/article/71388/rayman-origins-spawned-from-beyond-good-evil-2)

**Noita / Rain World** — Noita: per-pixel simulated "Falling Everything" engine; 2019 GDC talk by Petri Purho
exists — [Game Developer](https://www.gamedeveloper.com/game-platforms/road-to-the-igf-nolla-games-i-noita-i-); [Gorilla Newsletter 53](https://gorillasun.de/blog/gorilla-newsletter-53).
Rain World: Unity, Joar Jakobsson did art + programming — [Wikipedia](https://en.wikipedia.org/wiki/Rain_World). No primary source on either lighting model was found.

### Inferences
- The aesthetic bar in admired 2D/2.5D games (INSIDE, Ori 2) is set by **authored light data per asset**
  (painted light-direction maps, decals) rather than by simulated transport; a PH2D system that beats Godot on
  *perceived* quality must give artists authored channels (light-direction/normal, rim, bounce tint, AO decal),
  not only a better solver.
- Where real-time GI ships (PoE2), it is screen-space RC by the RC inventor — consistent with RC being the
  production-proven 2D/screen-space GI choice; nothing more advanced has shipped in this category.

### Gaps
- No technical primary source found for: Trine series, Rayman Legends lighting internals, Hades/Hades II,
  Rain World, Noita lighting, Gris, Inmost, Ultros, Sea of Stars, Hollow Knight Silksong, Moonlighter,
  Unsighted, The Last Night renderer. GDC Vault slides were login-gated.
- PoE2 RC cascade count/resolution/ms not public in what I could reach (the Steam ExileCon announcement was not fetched).

## 3. Mobile specifically — praised lighting, techniques, budgets

### Takeaway
The mobile state of the art in 2026 is **3D, not 2D**: Love and Deepspace (Infold/Papergames) ships real-time GI
on phones with **surfels + 3D radiance cascades** (GDC 2026, Arm-sponsored); Arm is pushing ray-traced lighting +
neural upscaling at 60 Hz. No 2D/2.5D mobile title with published lighting budgets was found; the generic mobile
guidance remains "bake when possible".

### Cited Findings
- GDC 2026 (10 Mar 2026, Arm Partner Developer Summit), "The Evolution of Real-Time Global Illumination in
  'Love and Deepspace'", Tianlong Ruan (Infold Games): real-time GI built on **surfels and 3D radiance cascades**;
  runs "smoothly on mobile", handles moving objects and lights, "comparable to mainstream PC approaches"; surfels
  precomputed at asset import so no runtime/edit-time baking; motivated by UGC (players edit scenes/lighting) that
  defeats static baking. No ms/device numbers in the abstract — [GDC schedule (abstract via search)](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336); [Arm Newsroom, GDC 2026](https://newsroom.arm.com/blog/gdc-festival-of-gaming-2026-mobile-gaming-trends)
- Arm Newsroom: other GDC 2026 sessions target 60 Hz with ray-traced lighting + neural graphics; Arm ASR and
  Neural Super Sampling cited as temporal upscalers for quality within power budgets; Mali G1-Ultra mentioned for
  mobile RT (headline only) — [Arm Newsroom](https://newsroom.arm.com/blog/gdc-festival-of-gaming-2026-mobile-gaming-trends)
- Sky: Children of the Light — only art talks found (GDC Summer 2020, Yuichiro Tanabe); a Gnomon 2020 event
  mentioned the proprietary engine raising mobile graphics bar (no detail) — [GDC Vault](https://gdcvault.com/play/1026903/Art-of-Sky-Children-of); [ArtStation Magazine](https://magazine.artstation.com/2020/01/gnomon-event-the-art-of-sky-children-of-the-light/).
  A GameDev.net "GDC Rewind" listing "Dynamic Environment Lighting in Sky" (credited Oliver Castaneda) exists but
  returned 403 — unverified — [GameDev.net](https://gamedev.net/news/994-dynamic-environment-lighting-in-sky-children-of-the-light-gdcrewind-gdc2026/)
- Generic mobile guidance (Android/Unity): realtime lights update every frame and cost more on mobile; baked
  preferred over mixed for mobile titles — [Android Developers: Lighting for mobile games with Unity](https://developer.android.com/games/optimize/lighting-for-mobile-games-with-unity)
- No published RC timings on Mali/Adreno found in any search — [search note; closest desktop data: HRC paper](https://arxiv.org/abs/2505.02041)

### Inferences
- Love and Deepspace is the strongest public evidence that **RC-family GI fits a mobile budget** in a shipped
  game (in 3D, with precomputed surfels as the geometry proxy). The 2D problem is strictly smaller (one fewer
  spatial dimension, flat probe grids), so mobile-feasibility of 2D RC is plausible — but no device number exists;
  PH2D would have to measure it itself (CLAUDE.md §0 rule 0).
- The transferable mobile lessons are structural: decouple GI resolution from screen resolution, precompute the
  geometry proxy offline (surfels ↔ 2D: precomputed SDF/occluder atlas for static art), and use temporal upscaling.

### Gaps
- No talk/slides with ms numbers for Love and Deepspace; no Grimvalor or Alto's Odyssey lighting source found
  ("Direlight" yields nothing); no Genshin/Honkai 2D-transferable lighting source collected; no Qualcomm/Apple case
  study on 2D lighting found.

## 4. Research frontier 2024–2026 beyond classic RC for 2D GI

### Takeaway
The frontier is dominated by Sannikov/Freeman's RC line: **Holographic RC (May 2025)** is the current 2D
accuracy reference (visually indistinguishable from 2D reference, 1.85 ms @ 512² / 7.67 ms @ 1024² on an RTX 3080
Laptop); **Split RC (Jul 2026)** is 3D (sparse hashed world probes + "ray splitting"), not 2D, ~8.6–11.5 ms on an
unnamed laptop GPU, with no mobile mention. No 2D ReSTIR, 2D neural radiance caching or mobile-targeted GI paper
was found.

### Cited Findings
- **Holographic Radiance Cascades for 2D GI** (Freeman, Sannikov, Margel; arXiv 2505.02041, 4 May 2025):
  single-shot, scene-agnostic radiance transfer; multi-level probes, rays built by combining short intervals
  instead of raytracing; "visually indistinguishable from the 2D reference solution at realtime framerates";
  **1.85 ms @ 512×512, 7.67 ms @ 1024×1024, RTX 3080 Laptop**, constant cost per scene size; positions itself
  against stochastic raytracing + spatiotemporal denoising — [arXiv 2505.02041](https://arxiv.org/abs/2505.02041)
- **Split Radiance Cascades** (Freeman, Sannikov; arXiv 2607.20384, 22 Jul 2026, 13 pp.): adapts RC to 3D
  diffuse GI with world-space probes in a sparse hashmap; "ray splitting" traces from visible surfaces and weights
  cascade contribution by hit distance; works single-frame and temporally accumulated — [arXiv 2607.20384](https://arxiv.org/abs/2607.20384)
- Split RC third-party review: only number **8.6–11.5 ms/frame on a laptop GPU** (GPU, resolution, memory not
  named); compared visually vs a self-implemented DDGI, path-traced reference, OIDN; no RMSE/PSNR; limitations:
  light leaking, detail below probe spacing missed, over-blurred shadows, foliage/thin-rod coverage failures (up
  to 4× ray-count spread), parameter rule mismatch (predicted t0≈5.09Δs0 vs empirical 1.0Δs0, impl 1.6Δs0); no
  mobile; no 2D evaluation — [Pith review](https://pith.science/paper/2607.20384)
- RC origin/formal paper (Osborne & Sannikov) — [arXiv 2408.14425](https://arxiv.org/abs/2408.14425)
- RCPG: "Real-Time Product Path Guiding Using Radiance Cascades" (ACM, not read) — lead that RC is being used to
  guide path tracing — [ACM DL](https://dl.acm.org/doi/pdf/10.1145/3840291)
- ReSTIR work remains 3D/RTX (ReSTIR GI 2021; ReSTIR PT in NvRTX for Zorah 2025) — [A Graphics Guy's Note](https://agraphicsguynotes.com/posts/understanding_the_math_behind_restir_gi/); [Guru3D](https://www.guru3d.com/story/nvidia-previews-faster-restir-pt-enhanced-realtime-path-tracing-technology/)

### Inferences
- For a 2D engine, Holographic RC is the "correctness" target (no stochastic noise, no denoiser, constant cost);
  Split RC's ideas (sparse probes, surface-originated rays weighted by hit distance) are relevant only if PH2D
  wants sparse/large worlds or 2.5D layers.
- Screen-probe-style caches (Lumen-like) already exist in 2D form in bevy_magic_light_2d (8×8 probe irradiance
  cache + 8-frame temporal) — a cheaper, noisier, temporally-lagged alternative to RC.

### Gaps
- No 2D ReSTIR, 2D neural radiance cache, or explicitly mobile-targeted 2D GI paper found (2024–2026).
- HRC vs vanilla RC quantitative comparison (ringing/leaking) not extracted — full PDF not read.

## 5. Per-candidate summary (technique · achieves · cost · numbers · maturity · license)

### Takeaway
Only two candidates have trustworthy numbers (HRC paper; Mytino/Raylib RC anecdotes — all desktop). Mobile proof
exists only for RC-family GI in 3D (Love and Deepspace, no numbers). For PH2D, all open 2D candidates are
permissive; Smart Lighting 2D and shipped-game engines are proprietary → oracle-only.

### Cited Findings
| Candidate | Technique | Achieves | Cost / numbers (device, res) | Maturity | License |
|---|---|---|---|---|---|
| Smart Lighting 2D | Multiple lightmaps, custom logic, culling, adjustable lightmap res — [Asset Store](https://assetstore.unity.com/packages/slug/112535) | Soft + "day" shadows, many lights | Vendor: "3–5× faster than URP 2D" (no device) | Last update Nov 2022 | Proprietary ($29) |
| Bulb (GameMaker) | Not documented in README — [GitHub](https://github.com/JujuAdams/Bulb) | 2D lights + shadows | None | v22.0.7 | MIT |
| Firefly (Bevy) | Occluder shadows, soft/hard, normal maps, z-sorted shadows — [GitHub](https://github.com/PVDoriginal/firefly) | Coloured/transparent occluders | None (stress demo, no metric) | Early, thesis | MIT/Apache-2.0 |
| bevy_light_2d | Occlusion + dynamic shadows — [docs.rs](https://docs.rs/bevy_light_2d/) | Point lights, WebGL2 | None | v0.10, Sep 2026 | MIT |
| bevy_magic_light_2d | JFA-SDF + 8×8 screen probes, bounces, 8-frame temporal — [GitHub](https://github.com/zaycev/bevy-magic-light-2d) | GI w/ bounce, colour bleed | 1/64 pixels lit directly; no ms | Experimental | (repo; not checked) |
| solis_2d | Optimised RC — [lib.rs](https://lib.rs/crates/solis_2d) | GI, shadows, normal maps | None | Unmaintained (Bevy 0.14) | MIT/Apache-2.0 |
| Raylib RC 2D | RC, 6 cascades — [GitHub](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination) | GI | 60 fps @1200×900, GTX 1660 | Demo | (not checked) |
| Holographic RC | Multi-level probes, combined intervals — [arXiv](https://arxiv.org/abs/2505.02041) | Reference-quality 2D GI, no noise | 1.85 ms @512², 7.67 ms @1024², RTX 3080 Laptop | Paper, May 2025 | Paper |
| Split RC | Sparse hashed 3D probes + ray splitting — [arXiv](https://arxiv.org/abs/2607.20384) | 3D diffuse GI | 8.6–11.5 ms laptop GPU — [Pith](https://pith.science/paper/2607.20384) | Paper, Jul 2026 | Paper |
| Love and Deepspace | Surfels + 3D RC — [Arm](https://newsroom.arm.com/blog/gdc-festival-of-gaming-2026-mobile-gaming-trends) | Mobile real-time GI, dynamic | "smooth on mobile" (no numbers) | Shipped | Proprietary |
| Path of Exile 2 | Screen-space RC — [Inara](https://inara.cz/morps/article/31103) | Real-time GI | None public | Shipped | Proprietary |
| INSIDE | Hand-placed decals (bounce/AO/shadow), no GI — [Playdead](https://blog.playdead.com/articles/the_lighting_of_inside/lighting_of_inside.html) | Painterly authored light | N/A | Shipped 2016 | Proprietary |
| Ori 2 | ~30k hand-painted light-direction maps — [TheGamer](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/) | Every texture reacts to lights | N/A | Shipped 2020 | Proprietary |

### Inferences
- Combining the two axes the owner cares about: **quality ceiling** = Holographic RC (2D-reference-indistinguishable,
  deterministic); **production/mobile proof** = RC-family in Love and Deepspace (3D) and PoE2 (screen-space);
  **artistic bar** = authored per-asset light channels (Ori 2, INSIDE). None of these is Godot-derived.
- "Many lights" scalability is a natural property of RC/probe GI (cost independent of light count, emissive pixels
  are lights) — a point stated for HRC as "constant cost for a given scene size" ([arXiv 2505.02041](https://arxiv.org/abs/2505.02041)),
  whereas the shadow-per-light middleware scales with light count (inference from their design; none publishes scaling curves).

### Gaps
- No mobile device measurement for any 2D candidate — PH2D must produce the first one.
- Licenses of bevy_magic_light_2d and the Raylib RC repo not verified.
