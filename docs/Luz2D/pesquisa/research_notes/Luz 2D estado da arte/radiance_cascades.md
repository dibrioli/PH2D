# Radiance Cascades (RC) for 2D real-time GI — method, variants, implementations, measured cost, mobile viability

Research cut-off: 2026-10-08. Primary sources read in full text: Sannikov RC paper PDF (GitHub, [WIP] jcgt-template draft), Freeman/Sannikov/Margel HRC (arXiv 2505.02041, May 2025), Freeman/Sannikov Split RC (arXiv 2607.20384, Jul 2026). Everything else is blog/README/forum and is flagged as such.

## 1. Sannikov 2023 — the core method (penumbra hypothesis, interval per cascade, merge, cost/memory)

### Takeaway
RC splits the radiance field into distance bands ("intervals"): near bands are stored with fine spatial / coarse angular resolution, far bands with coarse spatial / fine angular resolution, justified by the "penumbra condition". Each cascade is traced independently over its own short interval and then merged top-down; in flatland the whole hierarchy costs at most ~2x cascade 0 (2x-angular scaling) or ~1x cascade 0 per cascade (4x-angular scaling, the variant used in practically every 2D implementation). Cost is independent of the number of lights. Input is any field that answers "radiance + transmittance along a ray segment" — in 2D practice an emissive/occluder texture raymarched through a JFA-built distance field.

### Cited Findings
- Paper: "Radiance Cascades: A Novel Approach to Calculating Global Illumination [WIP]", Alexander Sannikov, Grinding Gear Games; never submitted to a journal (uses jcgt template), kept on GitHub; discussion lives in the Graphics Programming Discord RC thread — [Raikiri/RadianceCascadesPaper](https://github.com/Raikiri/RadianceCascadesPaper); PDF: [RadianceCascades.pdf](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf). Public unveiling dated November 2023 by secondary sources — [80.lv](https://80.lv/articles/radiance-cascades-new-approach-to-calculating-global-illumination).
- Abstract: data structure that stores the radiance field "by decomposing it into multiple ranges (from near-field to far-field) and storing them separately"; claims "a constant cost that is independent of scene complexity, number of light sources" — [RC paper PDF](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Penumbra condition** (Sec. 2.1): to resolve a penumbra at distance D from a probe, linear (spatial) probe spacing must scale ∝ D and angular spacing ∝ 1/D; the **penumbra hypothesis** (Sec. 2.2) assumes a discretization satisfying this condition represents any penumbra of angular size > w/d — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Cascade scaling** (eq. 19): cascade i encodes interval [t_i, t_{i+1}] (t_0 = 0, each cascade starts where the previous ends); with probe spacing Δp ∼ 2^i, angular step Δω ∼ 1/2^i and interval length t ∼ 2^i — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Memory, flatland** (eq. 20): with 2x angular per cascade, M_i ∼ M_0/2^i, so total memory of arbitrarily many cascades = 2·M_0. 3D world-space grid: each cascade decreases as (1/2^i)^3·(2^i)^2; 3D surface/screen cascades: each cascade costs the same (memory grows linearly with cascade count) — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Cost** (eq. 21): rays(N) = 2^N·rays(0) while cost(N) = (1 + 1/2 + … + 1/2^N)·cost(0) < 2·cost(0); cascade count bounded by log(P0). Breaks down for "a point light that is just too small", a "penumbra that is too sharp", or badly chosen P0, Q0, t0 (cascade 0 probe count, directions, range) — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Merge**: radiance intervals compose like premultiplied alpha — Merge(⟨r_n,t_n⟩,⟨r_f,t_f⟩) = ⟨r_n + t_n·r_f, t_n·t_f⟩ (transmittance = 1−α) — formalized in [HRC paper, eqs. 7–8](https://arxiv.org/pdf/2505.02041); same concept as RC paper eq. 13/16.
- **Interval extension trick (Radiant 2d)**: a long interval [512,1024] is built as [512,544] then doubled repeatedly by self-extension; only possible on rectilinear probe grids (flatland and 3D world-space), not screen-space with per-probe depth — [RC paper §4.1](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **4x angular scaling in practice**: Radiant 2d actually used 4x angular resolution per cascade instead of 2x, "so all cascades are encoded with the same amount of storage"; the author reports it "seemingly produced higher quality:cost ratio" — [RC paper §4.1](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf). Community explanations recommend 4x branching in 2D because interval count stays equal across cascades — [m4xc.dev, 2024-10-22](https://m4xc.dev/articles/fundamental-rc/).
- **Typical 2D config (jason.today)**: base 4; cascade count = ceil(log_base(diagonal)) + 1 (300x400 canvas → 5, plus one extra as a "naive fix" for light not reaching the far edge); cascade 0 interval = 1 px; interval for cascade i starts at base^(i−1) and has length base^i; probe spacing sqrt(base)^i; rays per probe base^(i+1) (4, 16, 64, …); estimated cost ≈ "16 naive rays" — [jason.today/rc](https://jason.today/rc) (part 2 of a series whose part 1 is dated 2024-07-27: [jason.today/gi](https://jason.today/gi)). The author also uses an unexplained "modifierHack" multiplying intervals by sqrt(base) for base ≥ 16 — [jason.today/rc](https://jason.today/rc).
- **GM Shaders config**: each cascade has 1/4 the probes and 4x the rays (e.g. 16x16→8x8→4x4→2x2 probes with 64/256/1024/4096 rays); interval origin = interval·(1−4^i)/(1−4), length = interval·4^i — [GM Shaders Guest: Radiance Cascades, Xor & Alex (Yaazarai), 2024-04-13](https://mini.gmshaders.com/p/radiance-cascades).
- **Input pipeline in 2D**: emissive/occluder scene → Jump Flood Algorithm (log2(max dim) ping-pong passes) → distance field → sphere-traced rays per interval — [jason.today/gi](https://jason.today/gi); a public WebGPU/shader component uses "six radiance cascades, base 4, geometric intervals, linear RGBA16F", JFA distance field, merged top-down with visibility alpha — [21st.dev/vgpu](https://21st.dev/@vgpu/components/radiance-cascades.md). HRC instead uses DDA (Amanatides–Woo) with analytic per-pixel integration of extinction/emission — [HRC §5](https://arxiv.org/pdf/2505.02041).
- **Light-count independence (measured)**: Radiant 2d lights "2, 10², 100² particles in the same amount of time of about 12ms" (Fig. 13) — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf). HRC: "does not branch based on the scene data, it has constant time for a given size" and "functions independently from the quantity of light sources or the complexity of the geometry" — [HRC](https://arxiv.org/pdf/2505.02041).

### Inferences
- For a 2D engine the practical default is branching 4 (2x spatial per axis, 4x directions), c0 = 4 directions, c0 interval ≈ 1 probe spacing, ~log4(diagonal)+1 cascades (≈ 6–7 at 1080p–4K). With branching 4, each cascade costs the same as c0, so total cost ∝ cascade count (logarithmic in resolution), not the "< 2·c0" of the 2x scheme.
- Emissive sprites "lighting their neighbours" is exactly the case RC handles natively: every emissive pixel is a light, and cost doesn't depend on how many there are.

### Gaps
- The RC paper PDF has no version dates; the exact revision history (22 commits) was not inspected.
- Sannikov's video talks (linked in the HN thread) were not transcribed; no cost numbers were extracted from them.

## 2. Known artifacts and fixes (ringing, bilinear fix, nearest variants, pre-averaging, layout, leaks, temporal)

### Takeaway
Vanilla RC rings (concentric bands around lights) and leaks light at cascade borders. The canonical fix is the **bilinear fix** (Sannikov; written up in Osborne & Sannikov 2024), which casts 4x rays per merge (one toward each bilinear neighbour probe of cascade N+1); cheaper "nearest" variants trade ringing for pixelation/dither. **Pre-averaging** (Yaazarai) cuts merge lookups 16→4 and storage by 75%. **Direction-first** layout was reported as a 2x speed-up and then retracted. Classic RC is deterministic/noise-free and does not need temporal accumulation; temporal is used only in the screen-space variants.

### Cited Findings
- "In its default formulation, it has noticeable artifacts manifesting as rings around light sources, but there are ways to remove these, such as the 'Bilinear Fix' described in Osborne and Sannikov [2024]" — [HRC §2](https://arxiv.org/pdf/2505.02041); "Osborne and Sannikov [2024] introduces the bilinear fix method which removes ringing artifacts in 2D implementations" — [Split RC §2](https://arxiv.org/pdf/2607.20384). Osborne & Sannikov is the astrophysics paper (arXiv 2408.14425, v1 2024-08-26, DexRT code, journal RASTI DOI 10.1093/rasti/rzae062) — [arXiv abs](https://arxiv.org/abs/2408.14425).
- RC "has errors when resolving small penumbras" — [HRC §2](https://arxiv.org/pdf/2505.02041). In an equal-time comparison (512x512, 14 px occluder, 10 px light) optimized classic RC gave RMSE 0.0354 at 1.96 ms vs HRC 0.0184 at 1.85 ms: "towards the right of the scene, the cascade level that picks up the light is increased, which results in not enough spatial resolution to pick up the penumbra" — [HRC Fig. 8](https://arxiv.org/pdf/2505.02041).
- **Fix variants (Yaazarai repo, demos dated 2024-03/04)**: *Bilinear-Fix* "casts 4x rays, reprojecting one to each cN+1 bilinear probe before merging … fixes occlusion since merged rays are now perfectly continuous"; *Nearest-Fix* one ray toward the nearest cN+1 probe → pixelation (2x2 cN probes share one cN+1 probe); *Nearest-Interlaced-Fix* shifts the nearest choice so neighbours don't share → "stylistic dithering artifacts while solving ringing and occlusion"; *Vanilla* = no fixes — [Yaazarai/GMShaders-Radiance-Cascades](https://github.com/Yaazarai/GMShaders-Radiance-Cascades) (Unlicense, based on fad's Shadertoy).
- jason.today: bilinear filtering of the upper cascade texture smooths the probe grid; clamping the merge offset to probe bounds stops edge leaks; ringing on single-dot scenes persists, "still an active area of research", and fixes "often cost about double the frame time" — [jason.today/rc](https://jason.today/rc).
- Ringing comes from "the disconnect between probes in adjacent cascades"; bilinear and **parallax** fixes are mentioned as remedies (out of scope of the article); **direction-first** layout groups same-direction intervals into blocks for locality during merging — [m4xc.dev (Max Coppen), 2024-10-22](https://m4xc.dev/articles/fundamental-rc/).
- **Pre-averaging** (GM Shaders part 2, 2024-07-13, Yaazarai & Xor): merge lookups 16→4, memory "75% reduction" (one pixel stores the average of four rays); with hardware bilinear (`gpu_set_texfilter`) + pre-averaging, merge = one texture sample; single-pass trace+merge shader "under 100 lines" — [GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2). Split RC also uses pre-averaging — [Split RC §6](https://arxiv.org/pdf/2607.20384).
- **Direction-first retraction**: the article first reported direction-first doubling performance, then added an update that it "is not actually faster" — the gain came from a GameMaker performance issue; still easier to debug — [GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2).
- **Light-leak fixes**: give the ray range a small overlap into cascade N+1; clamp interpolated probe positions at direction-first block borders; invert c0 hit alpha (else light volumes render black) — [GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2). Listed residual limitation: "occasional light leaks and non-linear attenuation transitions" — [GM Shaders RC 1](https://mini.gmshaders.com/p/radiance-cascades).
- **Cascade 0 range**: decreasing c0 range makes "very crude and disconnected radiance intervals" connect into a continuous approximation (Fig. 14) — [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf). Split RC Fig. 17 explores length-scaling l ∈ {2,4} and t0 ∈ {0.5…36} at 2048x768, 32 c0 directions, 1 c0 probe/pixel; t0 chosen so aliasing "is barely apparent" — [Split RC](https://arxiv.org/pdf/2607.20384).
- **Temporal**: HRC "does not use stochastic raytracing and therefore is noiseless and does not need any temporal caching" — [HRC abstract](https://arxiv.org/abs/2505.02041v1). Sannikov's screen-space-probe/world-space-interval variant reprojects each cascade from the previous frame for temporal accumulation (30–50 ms/frame on RTX3060, 0.1–0.5 s to converge) — [RC paper §4.5](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Depth-aware upscaling ("Bilinear 3D")**: a later Sannikov method for screen-space RC, adopted in a 2025 Chalmers MSc thesis (Gavras) — [Chalmers ODR](https://odr.chalmers.se/items/3ee9fb4e-1880-46c2-802a-a660e38dc9ee/full).

### Inferences
- For a quality 2D product the bilinear fix is effectively mandatory (ringing around small emissive sprites is very visible); budget ~2x the vanilla cost (jason.today) or ~4x rays in the trace stage (Yaazarai's description). Pre-averaging is a free win and should be default. Direction-first is a layout/debuggability choice, not a proven speed-up.
- The fundamental remaining classic-RC weakness for "emissive sprites with soft shadows" is small lights/sharp penumbras far from the light — the case HRC was designed to fix.

### Gaps
- I could not locate the exact first public description/date of the bilinear fix in the Discord (inaccessible); attribution here follows the two Freeman/Sannikov papers.
- The parallax fix was not found described with a source beyond the m4xc mention.

## 3. Holographic Radiance Cascades (Freeman, Sannikov, Margel — arXiv 2505.02041, 2025-05-04)

### Takeaway
HRC reformulates RC for 2D with probe grids that decrease in resolution in only one direction (per quadrant), removing the redundancy/ray gaps of classic RC, giving every cascade the same accuracy and handling hard-ish shadows. Measured on an RTX 3080 Laptop (CUDA via LuisaCompute, Rust): 0.55 ms @256², 1.85 ms @512², 7.67 ms @1024², 33.9 ms @2048², scene-independent. At equal time to classic RC it halves the penumbra RMSE. Cost: O(X² log X) time and ≈ N·X² memory for the acceleration structure (≈ N times classic RC's).

### Cited Findings
- Authors/affiliations: Rouli Freeman (University of Oxford), Alexander Sannikov (Grinding Gear Games), Adrian Margel; submitted 4 May 2025, cs.GR, 9 pages — [arXiv abs](https://arxiv.org/abs/2505.02041v1).
- Claims: single-shot, scene-agnostic; results "visually indistinguishable from the 2D reference solution at realtime framerates"; builds rays by combining short intervals; improves over RC "by removing redundancies in the radiance field encoding" and handles hard shadows — [arXiv abs](https://arxiv.org/abs/2505.02041v1). Conclusion: "reformulating Radiance Cascades using a grid that decreases in resolution in only one direction to avoid shadow artifacts", plus an acceleration structure not relying on empty-space skipping → consistent performance and volumetrics — [HRC §6](https://arxiv.org/pdf/2505.02041). Split RC summarizes HRC as "anisotropic probe layouts to eliminate ray gaps larger than the base cascade probe spacing, which results in every cascade having the same accuracy" — [Split RC §2](https://arxiv.org/pdf/2607.20384).
- Algorithm: fluence split into four quadrants handled separately (they can reuse memory on square grids); N = ⌈log2 X⌉ levels; T0..T2 traced directly (short rays), T3..TN built from T_{n−1} by merging; "Merge Up" builds T, "Merge Down" builds R_{N−1}..R_0. A +1 offset on R0 lookups prevents bright "crosses" from overlapping diagonals — [HRC §4](https://arxiv.org/pdf/2505.02041).
- **Table 1 (RTX 3080 Laptop GPU, output fluence grid = scene resolution, fp16 storage, no compression)** — [HRC](https://arxiv.org/pdf/2505.02041):

  | Size | Merge Up | Merge Down | Total |
  |---|---|---|---|
  | 256×256 | 0.30 ms | 0.25 ms | 0.55 ms |
  | 512×512 | 1.00 ms | 0.85 ms | 1.85 ms |
  | 1024×1024 | 4.10 ms | 3.57 ms | 7.67 ms |
  | 2048×2048 | 18.1 ms | 15.8 ms | 33.9 ms |

- Comparisons (same GPU): Fig. 1 @1024² — path traced 600 spp 1080 ms (RMSE 0.00962), PT 52 spp 98.6 ms (RMSE 0.0667), HRC 7.67 ms (RMSE 0.00630). Fig. 8 @512² penumbra — HRC 1.85 ms RMSE 0.0184; PT+NEE 10 spp 1.55 ms RMSE 0.00711 (NEE needs scene knowledge; noisy penumbra); PT 600 spp 109 ms RMSE 0.0260; classic RC 1.96 ms RMSE 0.0354. Fig. 9 pinhole @1024² — HRC 7.67 ms RMSE 0.0095 vs PT+NEE 52 spp 30.5 ms RMSE 0.0139. Volumetric fractal scene: PT takes 12x longer (1310 ms for 600 spp) while HRC is unchanged — [HRC §5](https://arxiv.org/pdf/2505.02041).
- **Ray count**: 4(N + 3 + 2^−N) ray computations per output value without acceleration = 52 per base probe at 1024²; with their acceleration structure, 19 rays per base probe, all shorter than 6x the base spacing — [HRC §4.3.3](https://arxiv.org/pdf/2505.02041).
- **Complexity/memory**: O(X² log X) time; fluence storage 2 layers of R (2·X²·V_r); acceleration structure Σ T_n ≈ N·X²·(V_r+V_t) — "the majority of the memory cost", avoidable with a BVH for Trace (→ 2·X²·V_r); radiance & transmittance stored as 3-vectors of fp16 — [HRC §4.3.2, §5](https://arxiv.org/pdf/2505.02041). (pdftotext renders 4^N as "4N"; with N = log2 X, 4^N = X².)
- **Deployment note from the authors**: "in an actual renderer, the output fluence would likely be interpolated from these probes onto a screen 2x or 4x the size (merging with rays traced to the probes to avoid light leaks); this makes the algorithm significantly more feasible to run in combination with other GPU usage" — [HRC §5](https://arxiv.org/pdf/2505.02041).
- **Limitations/artifacts**: blurs a shadow by ~4 px vs reference; checkerboard near shadow edges (reduced by a cross blur, eq. 21); Moiré-like aliasing for small lights because ray segments are fixed; artifacts visible when lights are smaller than 8x the base probe resolution; poor 3D scaling (N⁴ memory for an N³ scene); only 4·(2²+1) = 20 traced outgoing directions per object → blurred specular — [HRC §5–6](https://arxiv.org/pdf/2505.02041). Future work: amortize across frames (cache unchanged world, jitter), compress acceleration structure — [HRC §6](https://arxiv.org/pdf/2505.02041).

### Inferences
- Scaling in Table 1 is ≈ 3.4–4.6x per 4x pixels, consistent with O(X² log X): HRC at full 1080p (~2.07 Mpx, between 1024² and 2048²) would be ~15–17 ms on an RTX 3080 Laptop — too expensive at native res; at 2x-downsampled probes (960×540 ≈ 0.52 Mpx) it lands near ~4 ms on that GPU (interpolated from Table 1, not measured).
- HRC's memory (~N× the scene grid, ~10–11 levels at 1–2k) is a real concern on mobile with shared/limited bandwidth; classic RC with pre-averaging is far lighter.
- HRC is the better quality target for small emissive sprites, but its own "lights < 8x base probe spacing" artifact means sprite-sized lights still need care (probe spacing 1 px at c0, or a direct-light pass for tiny lights).

### Gaps
- No public reference implementation/code release for HRC was found (paper says Rust + LuisaCompute/CUDA). No HRC numbers on any non-NVIDIA, integrated or mobile GPU.
- Venue: arXiv only found; no confirmation of conference/journal acceptance.

## 4. Other 2024–2026 follow-ups (3D, screen-space, surfel, Split RC, theses)

### Takeaway
2025–2026 work moved RC into 3D (Split RC, Surfel RC, Mathis' 3D shadertoy, Love and Deepspace on mobile) and screen-space (PoE2, Chalmers theses). For 2D the state of the art remains classic RC + bilinear fix (cheap) and HRC (accurate). The single concrete mobile evidence of RC in production is Love and Deepspace (3D, surfel + RC, GDC 2026), with no numbers published.

### Cited Findings
- **Split Radiance Cascades** (Freeman & Sannikov, arXiv 2607.20384, submitted 2026-07-22, 13 pp): 3D diffuse GI with world-space probes in a sparse hashmap, "ray splitting" (one full ray from visible surfaces deposited into all cascades weighted by hit distance, reducing raytracing cost by the cascade count), LODs like cascaded shadow maps — [arXiv abs](https://arxiv.org/abs/2607.20384). Measured on RTX 3080 Laptop (Rust/LuisaCompute/CUDA, OptiX): total 8.6 ms Sponza / 11.5 ms San Miguel (init 1.2/1.3, gen rays 0.4/0.6, trace 1.3/4.5, deposit 2.2/1.9, merge 1.0/0.8, shade 2.5/2.4); Fig. 1 single frame 13.2 ms of which 6.1 ms is 1 spp raytracing — [Split RC Table 2](https://arxiv.org/pdf/2607.20384).
- Split RC related-work states RC "delivers state of the art results in 2D, as can be seen in games such as Core Keeper and Kyora" — [Split RC §1](https://arxiv.org/pdf/2607.20384). **Contradiction/unverified**: a May 2026 secondary article describes Core Keeper lighting as a grid/cell light buffer, not RC — [foro3d](https://foro3d.com/en/2026/mayo/iluminacion-dinamica-en-pixel-art-el-caso-tecnico-de-core-keeper.html); I found no developer source either way.
- Three fully-3D RC variants listed: Mathis 2024 ([Shadertoy X3XfRM](https://www.shadertoy.com/view/X3XfRM)), Surfel RC (Max Coppen 2025, [mxcop/src-dgi](https://github.com/mxcop/src-dgi); "interpolation artifacts are noticeable"), and **Love and Deepspace** "utilizes Radiance Cascades in 3D [Ruan 2026] to increase the efficiency of standard irradiance probes for use on mobile devices" — [Split RC §2](https://arxiv.org/pdf/2607.20384).
- **Love and Deepspace (GDC 2026, Tianlong Ruan, Infold/Papergames, Arm-presented, 2026-03-10)**: real-time GI combining surfels (computed at asset import) with 3D Radiance Cascades, claimed to run "smoothly on mobile", dynamic objects and lights, "performance comparable to mainstream PC algorithms"; motivation is UGC lighting; abstract stresses mobile memory/bandwidth constraints — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336); [Arm newsroom](https://newsroom.arm.com/blog/gdc-festival-of-gaming-2026-mobile-gaming-trends). No ms figures public (vendor claim, unverified).
- **Radiance Cascades in astrophysics** (Osborne & Sannikov, arXiv 2408.14425, 2024-08-26): non-LTE radiative transfer, DexRT GPU code; source of the bilinear-fix write-up — [arXiv](https://arxiv.org/abs/2408.14425).
- **Theses**: Chalmers (Gavras, 2025) — RC fundamentals in 2D screen space + Screen-space Probes with World-space Intervals (SPWI), ray tracing instead of marching, "Bilinear 3D" upscaling; no numbers on the record page — [Chalmers ODR](https://odr.chalmers.se/items/3ee9fb4e-1880-46c2-802a-a660e38dc9ee/full). Chalmers thesis on screen-space RC in a 3D game ("As We Descend"): ~2 ms lowest / 6 ms medium / 26 ms high settings, GPU not given in snippet — [Chalmers ODR](https://odr.chalmers.se/items/9e42007a-bb60-440f-ba28-04410822758f/full) (via search snippet; unverified detail). A 2026 DiGRA paper compares RC vs Unity URP SSGI vs realtime lightmaps (FPS/frame time; numbers not retrieved) — [DiGRA PDF](https://dl.digra.org/index.php/dl/article/download/2775/2759/2819). A 2025 BTH MSc thesis also appears in searches — [diva-portal](https://www.diva-portal.org/smash/get/diva2:2007296/FULLTEXT01.pdf) (not read).

### Inferences
- No 2D-specific RC paper at HPG/I3D/SIGGRAPH 2024–2026 was found; the academic line for 2D is HRC (arXiv). The 3D/mobile result (Love and Deepspace) suggests RC-style hierarchies are considered mobile-viable by a shipping studio, but it is surfel-anchored 3D, not per-pixel 2D flatland.

### Gaps
- Love and Deepspace talk slides/recording (GDC Vault) not found; no device/ms numbers.
- Kyora's use of RC was not independently confirmed beyond the Split RC citation.

## 5. Public implementations and every measured number found

### Takeaway
Measured 2D numbers are scarce and all desktop: HRC (RTX 3080 Laptop, Table above), Sannikov's Radiant 2d (~12–30 ms "GTX3060", res unstated), GM Shaders (RTX 3080, 1024² and 1080p at the 120 FPS cap → ≤ 8.33 ms, vsync-limited), a Raylib/C# port (60 FPS @1200×900 on GTX 1660, likely vsync-capped), and an HN anecdote of 0.8 ms @4K on RTX 4090 (screen-space, PoE2-style). **No measurement on any mobile, Apple, Adreno, Mali, Intel iGPU or Steam Deck was found.**

### Cited Findings
- **Radiant 2d (Sannikov)**: "per-pixel global illumination in 2d in a constant time of about 30ms on a GTX3060, depending on required fidelity"; Fig. 13 "2, 10², 100² particles in the same amount of time of about 12ms" — resolution and cascade config not stated; "GTX3060" is not a real SKU (likely RTX 3060; unverified) — [RC paper §4.1](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **Sannikov other demos**: surface-RC direct radiance-field renderer < 0.5 ms on GTX3060; screen-space probes with world-space intervals 30–50 ms/frame on RTX3060 — [RC paper §4.4–4.5](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- **GM Shaders / Yaazarai (GameMaker, 2024-07-13)**: RTX 3080, probe density 0.25 (4 probes per pixel as worded), 4 rays per c0 probe: position-first 1024×1024 → "29.91 / 8.33ms (120 FPS)"; direction-first 1920×1080 → "25.95 / 8.33ms (120 FPS)" (first figure unlabeled; 8.33 ms = the 120 Hz frame cap, so actual GPU cost is ≤ 8.33 ms) — [GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2). Memory: 4 rays/probe at 1024² = a "4K texture"; doubling probe density → "8K texture (2048 x 2048 x 4)" — same source.
- **HN, 2024-10-26** (thread on m4xc's "Fundamentals of Radiance Cascades"): user Negitivefrags — on an RTX 4090 "the GI solution runs in 0.8ms for a 4k scene in 'High' quality", constant regardless of scene; context is PoE2's screen-space RC (anecdote; poster's GGG affiliation not verified here) — [HN 41957008](https://news.ycombinator.com/item?id=41957008).
- **Hybrid46/RadianceCascade2DGlobalIllumination** (C# Raylib port of Youssef Afella's Unity implementation, GLSL 330): "runs at 60 fps" at 1200×900 with cascadeCount = 6 on a GTX 1660; performance "drops linearly with cascadeCount and rayRange"; claims suitability for "desktop & mobile GPUs that support OpenGL 3.3+" but only lists Windows/Linux/macOS (mobile untested) — [GitHub](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination).
- **jason.today**: no ms figures; 1024×1024 demo "may require a modern GPU to run smoothly"; knobs: probe spacing 2, `rcScale`, `pixelRatio`; part 1 "supports desktop and mobile" (no numbers) — [jason.today/rc](https://jason.today/rc), [jason.today/gi](https://jason.today/gi).
- **Other implementations (no numbers found)**: Bevy `solis_2d` 0.1.1 (lommix, 2024-10-12, Bevy 0.14, WGSL, WIP, "aiming to be compatible with all targets") — [lib.rs](https://lib.rs/crates/solis_2d); TypeGPU (Software Mansion, WebGPU) radiance-cascades package for SDF-described 2D scenes, reworked in PR #2617 — [docs](https://docs.swmansion.com/TypeGPU/ecosystem/typegpu-radiance-cascades/), [PR](https://github.com/software-mansion/TypeGPU/pull/2617); vgpu component (6 cascades, base 4, RGBA16F, JFA; MIT) — [21st.dev](https://21st.dev/@vgpu/components/radiance-cascades.md), [vgpu.sh](https://vgpu.sh/examples/radiance-cascades); tmpvar playground — [tmpvar.com](https://tmpvar.com/poc/radiance-cascades/); Shadertoy bilinear example [cslSW4](https://www.shadertoy.com/view/cslSW4) and fad's skybox [mtlBzX](https://www.shadertoy.com/view/mtlBzX); Vulkan 2D — [tlegoc/RadianceCascadesGIVulkan](https://github.com/tlegoc/RadianceCascadesGIVulkan); Unity — [ZY4N demo](https://github.com/ZY4N/Unity-2D-Radiance-Cascade-Demo), [ShidoArima UPM](https://openupm.com/packages/com.shidon.radiance-cascade/); OpenGL — Izendi Radiance_Cascades_V1 (search snippet). Asbjørn Lystrup's sticky-slime sandbox uses RC lighting — [80.lv](https://80.lv/articles/asbjorn-lystrup-s-sticky-slime-simulation-gets-better-lighting/). Xor shared a WebGL 2D RC shader in the Construct community (not viewport-scaled) — [construct.net forum](https://www.construct.net/en/forum/game-development/tools-and-resources-27/global-illumination-ray-183906).
- No Godot 2D RC addon in the Asset Library was found; only a Heriot-Watt masterclass proposal — [HW proposal](https://projects.hw.ac.uk/Proposals/Details/KQY8I33OVEYQ).

### Inferences
- The only rigorous 2D timings are HRC's; everything else is either vsync-capped FPS or unstated resolution. For a "beat Godot on mobile" claim, PH2D will need to produce its own measurements (desktop + phone) — there is no public baseline to cite.

### Gaps
- No mobile/iGPU/Apple/Steam Deck 2D RC measurement exists in any source found (searched explicitly).
- No bevy_radiance_cascades crate with that exact name was found; the one Bevy crate (solis_2d) is WIP with no benchmarks.

## 6. Cost scaling (lights, resolution, cascade count) and memory at 1080p/1440p/4K

### Takeaway
Confirmed: cost is independent of light count and of scene content (measured by Sannikov and HRC). Cost scales with pixel count (≈ linear, HRC ≈ X² log X) and with cascade count (linearly for branching-4; geometrically bounded for branching-2). Memory is small for classic RC with pre-averaging; HRC needs ~N× more for its acceleration structure.

### Cited Findings
- Light-count/scene independence: [RC paper Fig. 13](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf); [HRC §5](https://arxiv.org/pdf/2505.02041).
- Resolution: HRC 0.55 → 1.85 → 7.67 → 33.9 ms for 256² → 2048² — [HRC Table 1](https://arxiv.org/pdf/2505.02041).
- Cascade count: classic 2x-angular total cost < 2·cost(c0) — [RC paper eq. 21](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf); 4x-angular keeps each cascade's interval count equal, so total ∝ cascade count — [m4xc.dev](https://m4xc.dev/articles/fundamental-rc/); empirically "drops linearly with cascadeCount and rayRange" — [Hybrid46 README](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination). Split RC: with K=2 in 2D, total work bounded for arbitrary cascade count (from search snippet of [Split RC](https://arxiv.org/pdf/2607.20384)).
- Memory: flatland total = 2·M0 (2x scheme) — [RC paper eq. 20](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf); pre-averaging −75% — [GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2); HRC ≈ N·X²·(V_r+V_t) — [HRC §4.3.2](https://arxiv.org/pdf/2505.02041).
- jason.today estimate: naive 512 rays/px at 1024² ≈ 1 B texture lookups ≈ 1 s/frame; RC ≈ 100x faster, ≈ cost of 16 naive rays — [jason.today/rc](https://jason.today/rc).

### Inferences
- **Memory estimate (my arithmetic, not from a source)** for classic branching-4 RC, c0 = 1 probe/pixel, 4 directions, RGBA16F (8 B/texel). Each cascade has the same texel count as c0 = 4·W·H; merge needs only 2 cascades live (ping-pong top-down):
  - 1080p (2.07 Mpx): 66 MB per cascade → ~133 MB ping-pong; pre-averaged ~17 MB per cascade (~33 MB pair).
  - 1440p (3.69 Mpx): 118 MB per cascade → ~236 MB; pre-averaged ~30 MB (~59 MB pair).
  - 4K (8.29 Mpx): 265 MB per cascade → ~531 MB; pre-averaged ~66 MB (~133 MB pair).
  - With c0 probe spacing 2 px (common; also HRC's "interpolate onto 2x/4x screen" advice), divide by 4: 1080p pre-averaged pair ≈ 8 MB. Plus JFA/SDF (RG16/RG32 at scene res) and emissive/occluder textures.
- **HRC memory estimate (my arithmetic)** at 1024², N=10, V_r+V_t = 12 B (two fp16 RGB triples): acceleration structure ≈ 10·1 M·12 B ≈ 126 MB (+ the (1+2^−n) terms), fluence 2 layers ≈ 12 MB — far heavier than pre-averaged classic RC.
- Cascade count only grows logarithmically with resolution (≈ 6 at 1080p, 7 at 4K for base 4), so resolution (pixel count, i.e. c0 probe count) dominates.

### Gaps
- No source publishes measured GPU memory footprints at 1080p/1440p/4K; the table above is derived.

## 7. Path of Exile 2 shipping details

### Takeaway
PoE2 ships screen-space (flatland-like, 2^i×1 probe scaling) RC with a heavily optimized screen-space raymarcher; Sannikov reports 3 ms on a GTX 1050 (resolution not stated). PoE2 launched in early access on PC, PS5 and Xbox Series X|S (Dec 2024); on consoles GI is part of the fidelity preset only. GI is the most expensive PC graphics setting per player guides (conflicting magnitudes).

### Cited Findings
- PoE2 GI uses "a hierarchy of screenspace radiance probe cascades that are populated with screenspace raymarching"; raymarches 4 rays simultaneously and stores occlusion in a 128-bit bitmask (≈ 128 binary occlusion rays per cascade texel); Fig. 15 "running in 3ms on GTX1050" for indirect light from a glowing character — [RC paper §4.2](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf).
- PoE2 probes scale as ∼2^i·1 "(same as flatland), can't capture offscreen light sources, but it works 1 to 2 orders of magnitude faster" than world-space-interval screen probes — [RC paper §4.5](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf). RC "is currently used in the game Path of Exile 2 … for screenspace illumination" — [HRC §2](https://arxiv.org/pdf/2505.02041).
- Consoles (Dec 2024 guide): PS5/Xbox Series X performance preset = shadows only at 67% resolution (claims 100–110 FPS); fidelity preset = "Shadow + Global Illumination" at native 4K — [ONE Esports](https://www.oneesports.gg/gaming/path-of-exile-2-console-graphics-settings) (secondary).
- PC: disabling GI reported as ~32% FPS gain ("largest single graphics setting") by one guide, ~17% by another; PCGamesN RTX 4070 117 fps avg @1080p optimized — [iqondigital](https://iqondigital.com/learn/games/optimize-path-of-exile-2-performance), [checkfps](https://checkfps.io/optimizar-fps/path-of-exile-2) (secondary, conflicting, rigs unstated).
- HN anecdote: 0.8 ms @4K "High" on RTX 4090 — [HN](https://news.ycombinator.com/item?id=41957008).

### Inferences
- PoE2's console choice (GI only in the 4K/fidelity preset, dropped for the 100+ FPS preset) shows that even GGG treats RC GI as a premium-tier cost on PS5-class hardware — a caution for phones.

### Gaps
- No GGG-official per-platform GI budget (ms) found. Whether PoE2 has reached 1.0 by 2026-10 was not verified (a Dec 2025 0.4.0 patch is mentioned by a guide). No Digital Foundry-style isolated GI cost found.

## 8. Mobile viability assessment (for a Rust/wgpu 2D engine)

### Takeaway
There is **zero published measurement** of 2D RC (classic or HRC) on any phone, Apple Silicon, Intel iGPU or Steam Deck. The only mobile production evidence is Love and Deepspace's surfel+3D-RC (GDC 2026, no numbers). On paper, classic 2D RC at reduced probe resolution (c0 spacing 2 px, pre-averaged, bilinear fix, fp16) should fit a few-ms mobile budget at ~540p-equivalent probe grids, but this is an extrapolation that PH2D must measure itself.

### Cited Findings
- Explicit searches for iPhone/Android/Steam Deck/M1 RC benchmarks returned nothing — (this research; see searches in §5). Implementations that *claim* mobile suitability do so without measurements: [Hybrid46](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination), [solis_2d](https://lib.rs/crates/solis_2d), [jason.today/gi](https://jason.today/gi) ("supports desktop and mobile").
- Love and Deepspace: RC (3D, surfel-anchored) "runs smoothly on mobile"; mobile constrained by "memory, bandwidth, and hardware performance" — [GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336); Split RC cites it as RC "for use on mobile devices" — [Split RC](https://arxiv.org/pdf/2607.20384).
- HRC authors themselves recommend computing probes at reduced resolution and interpolating to a screen 2x–4x larger for feasibility alongside other GPU work — [HRC §5](https://arxiv.org/pdf/2505.02041).
- Tuning levers documented: probe spacing 2, `rcScale`, `pixelRatio` — [jason.today/rc](https://jason.today/rc); renderScale 0.5–2.0, cascadeCount 4–10, rayRange — [Hybrid46](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination).

### Inferences
- Cost model to use: work ∝ (probe count at c0) × (rays per c0 probe, ×4 with bilinear fix) × (cascade count) × (march steps per interval), plus a JFA (≈ log2(max dim) passes). All terms are independent of light count — the decisive advantage over per-light 2D shadow techniques (Godot 2D uses per-light occluder shadow passes, so cost grows with lights; verify in the Godot research notes).
- Rough extrapolation (speculative): HRC at 512² = 1.85 ms on RTX 3080 Laptop; a high-end phone GPU has roughly an order of magnitude less memory bandwidth and FP throughput (general knowledge, not sourced here), and RC is texture-fetch/bandwidth heavy → expect on the order of 10–20 ms for HRC at 512² on a flagship phone, i.e. HRC is likely too heavy for mobile at useful resolution unless amortized over frames. Classic RC with pre-averaging at 960×540 probes (≈ 0.5 Mpx c0) is plausibly a few ms on flagship phones. **These are not measurements.**
- Mobile-specific design implications: prefer fp16 storage, pre-averaging (−75% memory/bandwidth), half-res c0 + bilateral/edge-aware upsample, update upper (far) cascades at reduced rate (temporal amortization — suggested as future work by HRC §6), skip JFA when occluders are static (cache SDF), and keep the whole chain as compute passes (wgpu on Metal/Vulkan/WebGPU). Tile-based mobile GPUs penalize many full-screen ping-pong passes — fusing trace+merge per cascade into a single pass (as GM Shaders does) matters more on mobile than on desktop.
- The fair "beat Godot on mobile" benchmark must vary the number of emissive sprites (1 → 1000): RC's flat curve vs. per-light shadow cost is where the win will show.

### Gaps
- No device measurements of any kind (Apple A/M-series, Adreno, Mali, Intel iGPU, Steam Deck/RDNA2 APU) for 2D RC — must be measured by PH2D.
- No data on WebGPU-in-browser RC performance on phones.
- Love and Deepspace's actual ms/device numbers are unpublished (GDC Vault not found).
