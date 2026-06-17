═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · WoS não cabe no §2.5 — o solver é o gargalo (multigrid)
Autor: Coordenador (jornada 2026-06-06) · responde HANDOFF_vector_w7_poisson_cpu_impl.md
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
Construí + provei o teu GPU Walk-on-Spheres (`8ab00c2`) E o caminho JBU low-res
(`49f7680`). Veredito medido no Metal: **o WoS está ~20-100× fora do budget §2.5, e
JBU NÃO fecha** — porque o gargalo é o **SOLVER**, não a resolução. O caminho real
pro §2.5 é **trocar o WoS por um multigrid Poisson** (que a própria matriz §2.5
nomeia como alternativa). A infra JBU que montei fica **pronta pra qualquer solver**.

Isto é teu (o solver vive em `crates/ph2d-vector-fill/src/diffusion_gpu.rs`, tua crate).

## §1 — O que está PROVADO (não mexer, é referência)
1. **WoS GPU = correto.** Dispatch wgpu do teu `diffusion.wgsl` + parity vs
   `walk_on_spheres_field`: mean |Δ| = 0.0101 @ 256 spp (cai 1/√spp = ruído MC de
   cos/sin ULP, não viés). Gate `gpu_wos_matches_cpu_reference`.
2. **JBU = mecanismo funciona + faithful.** `JbuPipeline` (denoise 3×3 bilateral +
   bilinear, teu `BILATERAL_UPSAMPLE_WGSL`, 1 submit). `gpu_jbu_approximates_full_res`:
   solve em 0.25× reproduz o full-res na região smooth (mean |Δ| = 0.033; a banda fina
   NA curva amacia ~1/scale px — esperado).
Tudo em `crates/ph2d-vector-fill/tests/diffusion_gpu_parity.rs` (#[ignore], dev-test).

## §2 — A MEDIÇÃO (por que JBU não basta) — `vector_diffusion_curve_jbu_budget`
| Config (Metal, ~70 GB/s, --release) | medido | budget |
|---|---|---|
| Lite 270p @16spp **×16 steps** + JBU→1080p | 10.4 ms | 3 ms ✗ |
| Lite ×64 steps | 19.7 ms | 3 ms ✗ |
| Standard 540p @32spp ×64 steps | 97.6 ms | 4 ms ✗ |
| upsample-only →1080p | ~3 ms (≈0.5ms GPU puro + sync submit+poll) | — |

JBU corta 16× os pixels (0.25×), mas mesmo no Lite com só 16 max_steps **o solve WoS
sozinho é ~7 ms** vs 3 ms. A vazão (~11 Gstep/s, e MENOR em baixa-res por subutilização)
× budget não fecha. O upsample é barato (memory-bound); o solve é o problema.

## §3 — O CAMINHO (teu): multigrid Poisson
Diffusion curves (Orzan et al. 2008) = solução de **∇²I = 0** (Laplace; +termo Poisson
opcional p/ gradiente) com as curvas como **condições de contorno Dirichlet** (as cores
dos lados). O solver real-time canônico é um **V-cycle multigrid** (Jacobi/Gauss-Seidel
smoothing + restrict/prolong na pirâmide) — **O(N)**, ~100× mais rápido que WoS Monte-
Carlo p/ a mesma qualidade, e DETERMINÍSTICO (sem ruído MC → sem precisar de denoise).
Tu já tens a rasterização dos segmentos (`pack_curves`/`GpuSegment`); falta:
1. rasterizar cor-da-curva + máscara-de-constraint num grid (o boundary Dirichlet);
2. o V-cycle (smooth → restrict → recurse → prolong → smooth) em compute;
3. (opcional) solve em baixa-res + reusar minha JBU upsample, OU solve full-res direto
   (multigrid costuma ser barato o bastante p/ full-res).

## §4 — A INFRA PRONTA (reusa, não reinvente)
Em `tests/diffusion_gpu_parity.rs`:
- `WosPipeline.solve(...) -> (compute_ms, wgpu::Buffer)` — devolve o field on-GPU (sem
  readback forçado). Espelha pro teu multigrid: solver → buffer.
- `JbuPipeline` + `solve_jbu(...)` — solve-em-baixa-res → upsample→display, on-GPU.
  Funciona com QUALQUER solver que produza um `vec4<f32>` field buffer.
- `readback_field` / `make_buffer` — helpers.
- Os gates `gpu_jbu_approximates_full_res` (qualidade) + `vector_diffusion_curve_jbu_budget`
  (budget por tier) JÁ medem o end-to-end — aponta teu multigrid neles e vê se fecha.
- O gate `vector_diffusion_curve_tier_budget` afirma um piso de throughput (não o budget,
  que seria falso-verde) — quando o multigrid couber, vira um budget-assert real.

## §5 — DECISÃO PENDENTE (do Enio, não-bloqueante)
Se multigrid for grande demais p/ agora, a alternativa é **revisar os budgets ADR-0060
§2.5** pro medido-alcançável (aceitar WoS como caminho high-quality/offline). O Enio
está ciente; ele decide multigrid-agora vs revisar-budget vs aceitar. Reportei a
tabela. Sem push (eu shipo). Refs: `docs/architecture/decisions/0060-*` §2.5,
commits `8ab00c2`/`49f7680`, memória `project_wos_diffusion_over_budget_2026_06_06`.
═══════════════════════════════════════════════════════════════════
