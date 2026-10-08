# Mobile-proven real-time GI and dynamic lighting (shipped or vendor-measured), and how it transfers to 2D

> Research date: 2026-10-08. Every number carries device + resolution when the source gives them; when it
> does not, that is said. Labels used: **[MEASURED]** = a number from a timing/counter run with a stated
> setup; **[VENDOR CLAIM]** = a relative or marketing figure without a reproducible setup; **[ABSTRACT]** =
> what a talk abstract asserts, not verified; **[CODE]** = read directly from source code at the cited
> version. Radiance Cascades theory, JFA, Godot and Unity URP 2D are covered in sibling notes and are not
> repeated here.

## 1. Mobile GI in shipped titles and vendor demos (2022–2026)

### Takeaway
The only *fully dynamic* GI publicly described as shipping on phones in 2026 is Love and Deepspace (Infold
Games): surfels + 3D Radiance Cascades with software ray tracing, presented with Arm at GDC 2026. No frame
times have been published (abstract-only). Every other large mobile title found (Genshin, Roco Kingdom /
Tencent MagicDawn) relies on **precomputed** transfer (PRT, lightmaps, probe volumes) plus real-time direct
light, and the engine GI systems (Lumen mobile, Unity APV) are either experimental or need per-vertex
fallbacks on phones.

### Cited Findings
**Love and Deepspace (Infold Games, GDC Festival of Gaming, 9–13 Mar 2026, "presented by Arm")**
- Session "The Evolution of Real-Time Global Illumination in 'Love and Deepspace'", speaker Tianlong Ruan
  (Infold Games). — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336) (page returned 403 to direct fetch; content via search index) · [Arm newsroom, GDC 2026 preview](https://newsroom.arm.com/blog/gdc-festival-of-gaming-2026-mobile-gaming-trends)
- **[ABSTRACT]** GI built on **surfels + 3D radiance cascades**; runs on mobile; handles moving objects and
  moving light sources; claims performance "comparable to mainstream PC approaches". Motivation: user-generated
  content (players edit scenes and lighting live), which static baking cannot serve. — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336)
- **[ABSTRACT]** Pipeline details listed as takeaways: surfels are **precomputed per asset at import time**
  (no runtime/scene-edit baking); **software ray tracing against surfels**; fixing gaps between surfels;
  **3D radiance cascades used to accelerate surfel ray tracing**; a **surfel BVH + hash cache** for hit points. — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336)
- Arm's post-GDC recap (16 Mar 2026) calls it a popular talk showing "surfel-based lighting and radiance
  cascades" for mobile, with **no numbers, no device named**. — [Arm newsroom, GDC 2026 takeaways](https://newsroom.arm.com/blog/takeaways-from-gdc-festival-of-gaming-2026)
- No slides, recording or ms figures were found publicly as of 2026-10-08.

**Genshin Impact (miHoYo/HoYoverse)**
- 2021 Unity Japan Dojo deck by Zhenzhong Yi (Head of Genshin console dev): two customized pipelines (mobile
  for Android/iOS, console for PC/console); **mobile is the primary development platform**; lighting
  computed in real time with time-of-day and dynamic weather; stylized PBR, "not completely energy
  conserving". — [Unity Dojo slides (docswell)](https://docswell.com/s/UnityJapan/KWRPQ5-210617-unity-dojo20211mihoyozhenzhongyi)
- miHoYo stated ray tracing would undermine the art style (no RT planned). — [OpenCritic news](https://opencritic.com/news/2490/ray-tracing-will-not-come-to-genshin-impact)
- No primary source found on Genshin's mobile indirect-light method; Honkai: Star Rail and Zenless Zone Zero:
  no primary technical lighting source found.

**Roco Kingdom (Tencent MoreFun Studios) + MagicDawn, GDC 2026 (12 Mar 2026)**
- **[ABSTRACT]** "Illuminating Roco's Legacy: Scalable Global Illumination from Theory to Practice": ~64 km²
  cross-platform open world; GI built on **Precomputed Radiance Transfer + Cascade Lighting Volumes**,
  time-of-day indirect light for dynamic objects, foliage and terrain, within mobile limits on package size,
  memory, bandwidth and compute. — [GDC schedule](https://schedule.gdconf.com/session/illuminating-rocos-legacy-scalable-global-illumination-from-theory-to-practice/915519) · [GDC Vault](https://gdcvault.com/play/1035733/Illuminating-Roco-s-Legacy-Scalable)
- **[VENDOR CLAIM]** MagicDawn baked lighting raises frame rate ">1.5×" vs traditional real-time lighting in
  the same Unity scene (interview; no device/resolution). — [Inven Global interview](https://www.invenglobal.com/articles/20316/comprehensive-interview-tencent-unveils-3-core-ai-technologies-at-gdc-2026-integrated-into-the-production-pipeline) · [Tencent press release](https://press.levelinfinite.com/en/Tencent-Games-Debuts-MagicDawn-at-GDC-2026-Showcasing-AI-Driven-Global)
- Speaker lists differ between the pre-event and post-event releases (Sheng Feng + Kaming Chan vs Feng Sheng
  + Zhenyuan Zhang). — same sources.

**Sky: Children of the Light (thatgamecompany)**
- A GDC 2026 talk titled "Dynamic Environment Lighting in Sky: Children of the Light" (speaker listed as
  Oliver Castaneda) exists, but the only index page found is a thin auto-summary (403 on fetch); no technique
  or numbers could be extracted. — [gamedev.net listing](https://gamedev.net/news/994-dynamic-environment-lighting-in-sky-children-of-the-light-gdcrewind-gdc2026/)
- Earlier Sky talks (GDC 2020 "Art of Sky") are art-direction, not rendering. — [GDC Vault](https://gdcvault.com/play/1026903/Art-of-Sky-Children-of)

**Arm demos: Steel Arms (2023–2024) and Lumilings / MegaLights (2026)**
- Steel Arms: UE 5.3, Lumen with hardware RT on **Immortalis-G715**, GDC 2024 talk "Steel Arms: Bringing
  Lumen to Immortalis". Arm's learning path gives only qualitative tuning (small Max Trace Distance lowers cost,
  higher Lumen Scene Detail raises it); **no ms/fps published**. — [GDC 2024 schedule](https://schedule.gdconf.com/session/steel-arms-bringing-lumen-to-immortalis-presented-by-arm/903171) · [Arm learning path: HWRT Lumen](https://learn.arm.com/learning-paths/mobile-graphics-and-gaming/best-practices-for-hwrt-lumen-performance/1-ray-tracing/)
- **[VENDOR CLAIM]** Immortalis-G715 launch (Jun 2022): HW RT ">300 %" faster than software RT on Mali-G710,
  using 4 % of shader-core area. Reviewers noted no benchmarks/fps counters at launch. — [CNX Software](https://www.cnx-software.com/2022/06/29/arm-immortalis-g715-gpu-supports-hardware-based-ray-tracing/)
- MegaLights on Mali (Arm blog, Owen Wu, 10 Jun 2026) — see §3 for numbers.

**Unreal Lumen on mobile (status 2026)**
- Epic docs: Lumen on mobile is **experimental since UE 5.4**, **Android only**, via the desktop renderer on
  Vulkan SM5 profiles (Xclipse 9xx, Adreno 7xx, Mali G7xx); **Lumen does not work on iOS/iPadOS/tvOS**. No
  5.6/5.7 change to that status was found. — [Epic: Lumen GI on mobile](https://dev.epicgames.com/documentation/unreal-engine/using-lumen-global-illumination-on-mobile-in-unreal-engine)
- Snapdragon Studios showed Lumen + RT soft shadows on Adreno (Snapdragon 8 Gen 2) at Unreal Fest 2023; listing
  has no timings. — [Unreal forums: Unreal Fest 2023 talk](https://forums.unrealengine.com/t/1787617)

**Unity Adaptive Probe Volumes (APV) on mobile**
- Unity 2023.2: APV received runtime-performance work "especially for URP on mobile". — [Unity 2023.2 Tech Stream](https://unity.com/blog/engine-platform/2023-2-tech-stream-now-available)
- Per-pixel APV sampling "may be unsuitable for a range of mobile devices"; a **per-vertex** quality mode was
  added; roadmap (2 Apr 2025) still lists "performance on lower-end platforms may still require further
  optimizations". **No Android ms figures published.** — [Unity roadmap: APV per-vertex](https://activation.unity3d.com/roadmap/1972-adaptive-probe-volumes-apv-per-vertex-quality)
- Fantasy Kingdom demo uses APV + GPU Resident Drawer + STP on mobile (cost of APV not isolated). — [Unity Fantasy Kingdom](https://unity.com/en/demos/fantasy-kingdom)

**Research outlook (not mobile-proven)**
- "Split Radiance Cascades: Real-Time GI via Sparse Radiance Probes" (Freeman, Sannikov; arXiv 22 Jul 2026):
  3D diffuse GI with a sparse hashmap of world-space probes; abstract mentions **no mobile evaluation**. — [arXiv 2607.20384](https://arxiv.org/abs/2607.20384)

### Inferences
- The pattern that *ships* on phones is **"precompute the expensive geometric part offline, evaluate lighting
  live"**: Love and Deepspace precomputes surfels per asset; Roco precomputes PRT; APV precomputes probes.
  Only Love and Deepspace keeps lights and objects fully dynamic, and it does so by tracing against a cheap
  proxy (surfels) accelerated by radiance cascades — not against triangles.
- For PH2D (2D): the analogous "per-asset precompute" is a per-sprite occluder/SDF/silhouette proxy built at
  import; the live part is 2D Radiance Cascades over a screen-space or world-space field. The Love and Deepspace
  result is the strongest public signal that Radiance Cascades are viable on a phone budget, but it is an
  abstract, not a measurement.
- DDGI on mobile: no shipped title or vendor measurement found (see Gaps).

### Gaps
- Love and Deepspace: no ms, device, resolution or cascade configuration published (GDC Vault recording may
  have them; not accessible here).
- Sky, Honkai: Star Rail, Zenless Zone Zero: no primary rendering-tech source found on their mobile lighting.
- DDGI (NVIDIA RTXGI-style irradiance probes) on mobile: no shipped title or vendor-measured mobile cost found.
- Arm "Immortalis ray tracing" demos: no per-pass timings found.

## 2. Hardware ray tracing on phones, and ray query in wgpu 29

### Takeaway
HW RT exists on Apple A17 Pro+/M3+, Immortalis-G715+, Adreno 740+ and Xclipse, but **no vendor has published a
per-pass ms cost for ray-query shadows/occlusion on a phone**; the only hard mobile ray data are Arm counter
readings (rays started). wgpu 29.0.4 exposes `Features::EXPERIMENTAL_RAY_QUERY` on **Vulkan** (needs
`VK_KHR_ray_query` + acceleration structures) and on **Metal** (iOS 18+/macOS 15+, device `supportsRaytracing`
&& `supportsRaytracingFromRender`), still flagged experimental.

### Cited Findings
**wgpu (read from changelog and the locally vendored crates `wgpu-hal-29.0.4`, `naga-29.0.4`)**
- Initial acceleration structure + ray query support landed in wgpu 23.0.0 (2024-10-25). — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)
- wgpu 27.0.0 (2025-10-01): `EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE` merged into
  `EXPERIMENTAL_RAY_QUERY`. — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)
- wgpu 28.0.0 (2025-12-17): WGSL now requires `enable wgpu_ray_query;` (and `wgpu_ray_query_vertex_return`). — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)
- wgpu 29.0.0 (2026-03-18): **"Implements ray-tracing acceleration structures for metal backend"** (#8071);
  DXR in wgpu-hal since 24.0. — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)
- **[CODE]** `wgpu-hal-29.0.4/src/metal/adapter.rs`: `supports_raytracing` is true only if
  `available!(macos = 15.0, ios = 18.0, …)` **and** `device.supportsRaytracing()` **and**
  `device.supportsRaytracingFromRender()`; then `features.set(F::EXPERIMENTAL_RAY_QUERY, self.supports_raytracing)`.
  Vulkan sets `EXPERIMENTAL_RAY_QUERY` when acceleration-structure + ray-query extensions are present
  (`VK_KHR_acceleration_structure`, `VK_KHR_ray_query`, `VK_KHR_deferred_host_operations`,
  `VK_KHR_buffer_device_address`). — [wgpu-hal source (trunk)](https://github.com/gfx-rs/wgpu/tree/trunk/wgpu-hal/src)
- **[CODE]** `naga-29.0.4/src/back/msl/writer.rs` has `const RAY_QUERY_MODERN_SUPPORT: bool = false; //TODO`
  (MSL ray query emitted via the intersector path in 29.x). — [naga source](https://github.com/gfx-rs/wgpu/tree/trunk/naga/src/back/msl)
- wgpu 30.0.0 (2026-07-01) changes: Metal switched from `intersector` to `intersection_query` so AABBs and
  non-opaque triangles are handled (#9304); guards against invalid ray-query calls on Metal (#9442); Metal
  combined buffer+acceleration-structure limit set to 29 per stage (#9709). Latest release v30.0.1 (2026-08-21).
  — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)
- Unreleased trunk (after 30.0.1): ray-tracing *pipelines* (`EXPERIMENTAL_RAY_TRACING_PIPELINES`) progressing,
  SPIR-V only. — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)

**Phone hardware and measured/claimed costs**
- **[VENDOR CLAIM]** A17 Pro (Sep 2023): HW RT "4× faster" than software RT on A16; no primary timing found;
  3DMark Solar Bay on iPhone questioned as to whether it uses RT. — [Macworld A17 Pro](https://www.macworld.com/article/2069288/a17-pro-benchmarks-geekbench-cpu-gpu-performance.html) · [MacRumors thread](https://spy.macrumors.com/threads/apple-silicon-and-ray-tracing-performance.2408769/)
- **[MEASURED, third-party]** Android Authority 2023 RT benchmarks (reflections/volumetrics, not shadows):
  Adreno 740 close to Immortalis-G715, the latter ~10 % ahead on average (fps, no per-pass ms). — [Android Authority](https://www.androidauthority.com/ray-tracing-benchmarks-snapdragon-8-gen-2-vs-dimensity-9200-vs-exynos-2200-3355127/)
- **Shipped title with RT on Adreno:** War Thunder Mobile — ray-traced shadows (penumbras, translucency,
  self-shadowing), Snapdragon 8 Gen 2+ only; no timings. — [Qualcomm OnQ, May 2023](https://www.qualcomm.com/news/onq/2023/05/hardware-accelerated-ray-tracing-improves-lighting-effects-in-mobile-gaming)
- **[VENDOR CLAIM]** Qualcomm: "Ray Tracing Position Fetch" −40 % bandwidth, +20 % performance (workload not
  stated). — [Snapdragon Elite Gaming](https://www.qualcomm.com/products/mobile/snapdragon/smartphones/snapdragon-elite-gaming)
- **[MEASURED, counters]** Arm MegaLights on Mali-G1 (12-core): **ray queries are the default** path; full RT
  pipeline only needed for masked-material evaluation; Started Rays 6.94 M (2 spp) / 13.1 M (4 spp) / 49.9 M
  (16 spp) per captured frame; resolution not stated. — [Arm blog: MegaLights on mobile, 10 Jun 2026](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/lighting-at-scale-bringing-hundreds-of-dynamic-lights-to-mobile-with-unreal-megalights)

### Inferences
- For 2D occlusion ("is light L visible from pixel P"), a ray query against a TLAS of sprite-occluder quads/
  AABBs is technically expressible in wgpu 29 on Vulkan Android RT GPUs and iOS 18+ Apple GPUs with RT
  (A17 Pro/M3 and later). But: (a) it is `EXPERIMENTAL_*`; (b) AABB/non-opaque hits on Metal needed the 30.0
  intersection_query change; (c) **no device without HW RT** (most phones, all WebGPU) gets it, so a non-RT
  path with identical answers is mandatory anyway (CLAUDE.md §0 rule 0: fallback must give the same answer).
- In 2D, a screen-space SDF/JFA march or Radiance Cascades over a 2D occupancy texture already answers the same
  query at bandwidth cost proportional to screen pixels; ray query only wins when occluders are few and the
  screen is large, which no source has measured on mobile. Treat HW ray query as an optional accelerator,
  never the baseline.

### Gaps
- No public ms cost for a ray-query shadow/occlusion pass on any phone GPU (Apple, Mali/Immortalis, Adreno).
- Whether `supportsRaytracingFromRender` is true on A17 Pro / A18 / A19 iPhones under wgpu was not verified
  on device.
- Adreno 830 / Immortalis-G925 / A19 RT measurements: none found.

## 3. Many lights on mobile (clustered/froxel, MegaLights)

### Takeaway
Google Filament (Apache-2.0, mobile-first) does clustered forward with **froxels built on the CPU**, capped at
**255 lights per view** by design (UBO ≥16 KiB on GLES 3.0, `uint8` light count). Arm's 2026 MegaLights port
shows the stochastic route: ~190+ shadowed point lights on Mali-G1 at ~41 fps vs ~8 fps deferred (vendor
measurement, resolution not given).

### Cited Findings
**Filament froxel lighting [CODE], files last changed 2026-08-11**
- `CONFIG_MAX_LIGHT_COUNT = 255`: "limited by UBO size, ES3.0 only guarantees 16 KiB … record buffer data
  type (uint8_t) … light count in a uint8_t". — [Filament EngineEnums.h](https://github.com/google/filament/blob/main/libs/filabridge/include/private/filament/EngineEnums.h)
- Froxel grid: `FROXEL_SLICE_COUNT = 16` depth slices; default **4096 froxels** "fits in a 16 KiB buffer, the
  minimum guaranteed in GLES 3.x and Vulkan 1.1"; record buffer 16384 entries; X/Y tile size derived from
  viewport and rounded to multiples of 8. Per-frame memory table in the header: 4096 froxels → 512 KiB;
  8192 → 768 KiB; 65536 → 6.0 MiB. — [Filament Froxelizer.h](https://github.com/google/filament/blob/main/filament/src/Froxelizer.h) · [Froxelizer.cpp](https://github.com/google/filament/blob/main/filament/src/Froxelizer.cpp)
- Froxelization runs on the **CPU via Filament's JobSystem** (thread-sharded `FroxelThreadData`), output
  uploaded as UBOs — no compute-shader dependency, which keeps it working on GLES 3.0. — [Froxelizer.cpp](https://github.com/google/filament/blob/main/filament/src/Froxelizer.cpp)
- Docs: clustered forward renderer; "hundreds of lights under certain conditions"; prefer spot lights with
  tight cones over point lights; keep shadow casters few. No mobile ms figures published. — [Filament lighting docs (mirror)](https://www.mintlify.com/google/filament/rendering/lighting)

**Unreal MegaLights on Mali (Arm blog, Owen Wu, 10 Jun 2026)**
- **[MEASURED, vendor]** Mali-G1, 12-core, Android, Vulkan SM5 + HW RT, UE 5.7: scene with ~195 point lights
  + 1 directional, all shadowed: **MegaLights ≈41 fps (~24 ms) vs traditional deferred ≈8 fps (~125 ms)**;
  ~5×. Resolution not stated. Counters: Started Rays 2.35 M vs 57.0 M (~24× fewer); Compute Active 85 M vs
  544 M (~6×). — [Arm blog](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/lighting-at-scale-bringing-hundreds-of-dynamic-lights-to-mobile-with-unreal-megalights)
- Method: per **8×8 screen tile**, importance-sample a few lights with temporal feedback; only those get a
  visibility ray; 1–4 rays/pixel typical (`NumSamplesPerPixel` 2/4/16); downsample factor 2 cuts importance
  cost ~¼ at more noise; ray count = W·H·N / F²; temporal accumulation instead of a dedicated denoiser;
  importance cost grows ~linearly with lights overlapping a tile. — same source
- Caveats from the same post: MegaLights is beta, mobile SM5 experimental, no official mobile support;
  Android forward path default caps 8 lights per cell (had to raise to 32); fp16 overflow fix needed. — same source

### Inferences
- 2D mapping of froxels: drop the Z slices → a **2D tile grid** (e.g. 16×16 or 32×32 px tiles) with a per-tile
  light list; CPU binning à la Filament works for hundreds of lights and avoids compute on weak GPUs; the
  255-light / uint8 / 16 KiB limits are GLES 3.0 limits, not wgpu limits (wgpu storage buffers lift them).
- 2D mapping of MegaLights: per tile, stochastically pick K lights by importance, cast one 2D occlusion
  march (SDF/RC) per chosen light per pixel, accumulate temporally. This bounds cost by pixels × K, not by
  light count — exactly the property a "many torches" 2D scene needs.

### Gaps
- No published Filament ms numbers for N lights on a named phone.
- MegaLights test resolution not stated; Mali-G1 is a 2025 flagship IP, retail-device identity not named.
- Arm "clustered shading on Mali" article: not located in this pass.

## 4. Temporal techniques and upscalers on mobile (cost numbers)

### Takeaway
Temporal amortization is now standard on mobile: open-source temporal upscalers with **published mobile ms**
exist — Snapdragon GSR 2 at **~0.9–1.1 ms** (fragment 2-pass, Snapdragon 8 Gen 3, output 1260×2800) — and Arm
ASR (MIT, FSR2-derived) with vendor fps gains on Immortalis-G720. MetalFX has no published A-series cost.

### Cited Findings
**Snapdragon GSR 2 (BSD-3-Clause, v2.0.0 2024-10-21)**
- **[MEASURED, vendor]** Snapdragon 8 Gen 3, max GPU frequency, output 1260×2800:

  | Scale | Input | 2-pass FS | 2-pass CS | 3-pass CS |
  |---|---|---|---|---|
  | 2.0× | 630×1400 | 0.905 ms | 1.801 ms | 2.015 ms |
  | 1.7× | 740×1648 | 1.024 ms | 1.910 ms | 2.199 ms |
  | 1.5× | 840×1866 | 1.107 ms | 1.998 ms | 2.397 ms |

  — [snapdragon-gsr sgsr/v2 README](https://github.com/SnapdragonStudios/snapdragon-gsr/tree/main/sgsr/v2) · [Qualcomm blog, Oct 2024](https://www.qualcomm.com/developer/blog/2024/10/introducing-snapdragon-game-super-resolution-2)
- Note the fragment-shader variant is ~2× cheaper than compute on Adreno. — same source
- License BSD 3-clause. — [snapdragon-gsr repo](https://github.com/SnapdragonGameStudios/snapdragon-gsr)

**Arm Accuracy Super Resolution (ASR)**
- MIT license; derived from AMD FidelityFX FSR 2 v2.2.2. — [arm/accuracy-super-resolution](https://github.com/arm/accuracy-super-resolution) · [Arm blog intro](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/introducing-arm-accuracy-super-resolution)
- **[VENDOR CLAIM]** +53 % fps at 2× upscaling on Immortalis-G720 at 2800×1260. — [Arm blog intro](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/introducing-arm-accuracy-super-resolution)
- Vulkanised 2025 (Sergio Alapont, Arm): baseline target 2400×1080 on Immortalis-G720, measured in Streamline;
  presets Quality ("ultra-optimized FSR2, roughly 100 % IQ match") / Balanced / Performance; comparison at
  2× (1200×540 → 2400×1080); internal benchmark at target 1600×720, 3 runs × 900 frames; **ms values are in
  chart images, not extractable**; "advise to run on late 2022 devices (i.e. Immortalis-G715) and onward",
  spatial upscaler fallback for older; Mali subgroup = 16 threads. — [Vulkanised 2025 slides (PDF)](https://vulkan.org/user/pages/09.events/vulkanised-2025/T24-Sergio-Alapont-Arm-ArmASR.pdf)
- **[VENDOR CLAIM, via press]** up to 1.5× FSR 2 performance, 20–40 % fps increase. — [Jon Peddie](https://www.jonpeddie.com/news/arm-enters-the-rt-scaling-race/)

**MetalFX (Apple)**
- No Apple-published per-frame ms on A-series. A developer forum report (2023, M1 Pro, 1904×1370 → 2856×2055):
  temporal MetalFX added ~10 ms to a ~0.6 ms frame (single anecdote). — [Apple Developer Forums 728545](https://developer.apple.com/forums/thread/728545)
- Genshin on iPhone 15 Pro Max: 58.7 fps with MetalFX vs 59.4 fps without (game-level, not isolated). — [Wccftech](https://wccftech.com/iphone-15-pro-max-running-genshin-impact-metalfx-enabled-results/amp/)
- WWDC 2025 (Metal 4): MetalFX denoising upscaler + frame interpolation announced; no A-series costs given. — [WWDC25 session 211](https://developer.apple.com/videos/play/wwdc2025/211/)

**Neural frame generation (Arm, 2026)**
- Arm Neural Frame Rate Upscaling (NFRU) in Early Access; Neural Graphics Development Kit (Vulkan ML extension,
  SDK, UE plugins); Sumo Digital session on frame-gen coexisting with RT lighting; no numbers. — [Arm GDC 2026 takeaways](https://newsroom.arm.com/blog/takeaways-from-gdc-festival-of-gaming-2026)

### Inferences
- A temporal upscaler costing ~1 ms at 1260×2800 means **rendering GI at ¼ pixel count is worth it** whenever
  the GI pass costs more than ~1.3 ms at native res.
- 2D mapping: GI in 2D is low-frequency; computing the radiance/GI buffer at ½ or ¼ resolution and
  upsampling (bilateral/depth-free) is cheaper than a full temporal upscaler; temporal accumulation of the GI
  buffer itself (as MegaLights does) amortizes cost across frames. Because Arm ASR (MIT) and SGSR 2 (BSD-3)
  are permissive and shader-only, they can be ported to WGSL if a full-frame upscaler is ever needed.

### Gaps
- Arm ASR per-preset ms on Immortalis-G720 (in chart images only).
- MetalFX temporal cost on A17 Pro/A18/A19: no measurement found.

## 5. Sustained performance / thermal budgets

### Takeaway
The only explicit, sourced budget found is Unity's: use ~**65 % of the frame** (≈10.8 ms at 60 Hz) to leave
thermal headroom. Arm documents the mechanism (ADPF thermal headroom, adaptive quality) but no Arm-authored ms
budget was found.

### Cited Findings
- 60 fps = 16.66 ms; Unity's rule of thumb keeps ~65 % → **~10.83 ms** per frame, rest left idle "to combat
  device thermal issues over extended play"; 60 fps drains battery ~2× faster than 30 fps; most mobile games
  target 30. — [Unity: best practices for profiling game performance](https://unity.com/how-to/best-practices-for-profiling-game-performance)
- Arm: Android Dynamic Performance Framework — Thermal State monitor and Thermal Headroom API to lower shadow,
  reflection, texture quality as throttling approaches. — [Arm: getting started with ADPF](https://community.arm.com/arm-community-blogs/b/graphics-gaming-and-vr-blog/posts/getting-started-with-adpf) · [Arm: adaptive performance](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/how-to-use-adaptive-performance)
- Memory traffic: LPDDR4 ≈ 100 pJ/byte (Arm figure cited by Unity) — bandwidth is a first-order power/heat
  cost. — [Unity profiling guide](https://unity.com/how-to/best-practices-for-profiling-game-performance)
- Arm ASR material frames upscaling as the lever against thermal throttling (power usage, average temperature,
  performance shown on a slide, values in images). — [Vulkanised 2025 slides](https://vulkan.org/user/pages/09.events/vulkanised-2025/T24-Sergio-Alapont-Arm-ArmASR.pdf)

### Inferences
- A defensible PH2D mobile target: total GPU ≤ ~10–11 ms at 60 Hz sustained, of which a lighting/GI budget of
  roughly 2–4 ms (inference, not sourced) leaves room for sprites, post and the upscaler; measure
  sustained (≥10 min) not cold-start, and wire the thermal-headroom signal to GI resolution/cascade count.
- Bandwidth (not ALU) is the likely limiter for 2D GI on tile-based mobile GPUs: cascades/SDF textures should
  be small formats (R16F/RG16F, RGBA8 where possible) and passes merged to avoid round-trips to DRAM.

### Gaps
- No Arm, Qualcomm or Apple document found stating an explicit sustained ms budget (e.g. "8–10 ms GPU").
- No studio post-mortem with measured sustained-vs-peak GPU clocks found in this pass.

## 6. Translation table: technique → 2D lighting (one line each)

### Takeaway
Every mobile-proven technique reduces to one of three ideas — precompute geometry proxies, bound per-pixel
light work by tiles/importance, amortize over time/resolution — and each has a direct 2D analogue cheaper than
its 3D original.

### Cited Findings
(Sources are those cited in §§1–5; this section is mapping, see Inferences.)
- Surfels + 3D RC (Love and Deepspace) — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336)
- PRT + cascade lighting volumes (Roco) — [GDC schedule](https://schedule.gdconf.com/session/illuminating-rocos-legacy-scalable-global-illumination-from-theory-to-practice/915519)
- APV per-vertex — [Unity roadmap](https://activation.unity3d.com/roadmap/1972-adaptive-probe-volumes-apv-per-vertex-quality)
- Froxels — [Filament Froxelizer.h](https://github.com/google/filament/blob/main/filament/src/Froxelizer.h)
- MegaLights — [Arm blog](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/lighting-at-scale-bringing-hundreds-of-dynamic-lights-to-mobile-with-unreal-megalights)
- SGSR 2 / Arm ASR — [SGSR 2 README](https://github.com/SnapdragonStudios/snapdragon-gsr/tree/main/sgsr/v2) · [Arm ASR repo](https://github.com/arm/accuracy-super-resolution)
- Ray query — [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)

### Inferences
- **Surfels + RC (Love and Deepspace)** → per-sprite precomputed occluder/emitter proxy (SDF or silhouette)
  baked at import; 2D Radiance Cascades in screen/world space trace against it — the 2D version drops one
  dimension of both probes and directions.
- **PRT / cascade lighting volumes (Roco)** → for static 2D backgrounds, bake per-texel transfer (e.g. a few SH/
  angular coefficients of occlusion) so time-of-day/sky colour changes relight without re-tracing.
- **APV / irradiance probes, per-vertex fallback** → a coarse 2D world-space probe grid sampled per sprite
  vertex (or per sprite centre) as the low-end tier; per-pixel cascade sampling as the high tier.
- **Filament froxels** → 2D tile light lists (no Z slices), CPU-binned for hundreds of point/spot lights;
  wgpu storage buffers remove the 255-light GLES limit.
- **MegaLights stochastic lights** → per-tile importance sampling of K lights + one 2D occlusion march each +
  temporal accumulation; cost ∝ pixels × K, independent of light count.
- **HW ray query** → optional accelerator for hard 2D shadows against a TLAS of occluder AABBs on RT-capable
  phones; must have a bit-for-bit-equivalent SDF/RC fallback.
- **Temporal upscalers (SGSR 2 ~1 ms, ASR)** → render the GI/radiance buffer at ½–¼ res and upsample; port
  SGSR 2 (BSD-3) or ASR (MIT) to WGSL only if a full-frame upscaler is needed.
- **Thermal headroom (ADPF)** → drive cascade count / GI resolution / light K from a headroom signal, holding
  ~10.8 ms total at 60 Hz.

### Gaps
- None of these mappings has a published 2D mobile measurement; PH2D would have to produce the first numbers
  on its own devices.
