---
name: project-wos-diffusion-over-budget-2026-06-06
description: "Vector W7 GPU Walk-on-Spheres é CORRETO mas ~20-100× fora do budget ADR-0060 §2.5; JBU low-res solve é o caminho; o gate afirma piso de throughput, não o budget"
metadata: 
  node_type: memory
  type: project
  originSessionId: 00710036-c40d-4ddb-8905-3f77e41f6f0c
---

**Vector W7 GPU diffusion-curve solver (Walk-on-Spheres).** Construí o pipeline wgpu do `diffusion.wgsl` do impl + parity vs `walk_on_spheres_field` (commit `8ab00c2`, `crates/ph2d-vector-fill/tests/diffusion_gpu_parity.rs`). Parity **passou** (mean |Δ|=0.0101 @ 256 spp; cai ~1/√spp = ruído MC de cos/sin ULP, não viés). MAS o bench provou que o WoS naive de 64-steps **não cabe** nos budgets de ADR-0060 §2.5: sustenta ~11 G-walk-steps/s no Metal → todo tier OVER (Heavy 512²@64spp ≈ 99ms vs 5ms; em 1080p real ~8× pior; até Lite 128²@16spp = 3.04ms vs 3ms).

**Why (não-óbvio):** o budget §2.5 foi aspiracional (setado antes de medir). WoS Monte-Carlo full-res é caro. O `BILATERAL_UPSAMPLE_WGSL` que o impl shippou (solve baixa-res + bilateral-upsample) é o caminho canônico pra caber — mas é o "render-embed" que o Enio adiou (escolheu a fatia contida solve+parity+bench).

**How to apply:** o gate `vector_diffusion_curve_tier_budget` NÃO afirma o budget (seria falso-verde) — afirma um **piso de throughput** (≥4 Gstep/s, regression guard real) e reporta cada tier vs budget pra manter o gap visível.

**JBU explorado (commit `49f7680`, 2026-06-06):** construí o caminho JBU low-res (WoS solve em solve_scale → `JbuPipeline` denoise+bilinear via `BILATERAL_UPSAMPLE_WGSL`, encadeado on-GPU). Qualidade PROVADA (`gpu_jbu_approximates_full_res`: solve 0.25× reproduz o full-res na região smooth, mean 0.033). MAS **JBU NÃO fecha o §2.5**: o gargalo é o **SOLVER**, não a resolução — mesmo no Lite (270p@16spp×16steps) o solve WoS sozinho é ~7ms vs budget 3ms; o upsample 1080p é ~0.5ms GPU puro (medido ~3ms com sync submit+poll que produção batcheia). Caminho real pra §2.5 = **solver mais rápido (multigrid Poisson** que a matriz §2.5 nomeia) ou revisar budget. A infra JBU fica pronta pra qualquer solver. Relacionado: [[project_vector_node_opaque_carrier]] [[project_node_effect_pure_for_renderer_consumed]] [[feedback_no_industrial_claims_without_verification]].
