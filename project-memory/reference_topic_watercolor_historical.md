---
name: reference-topic-watercolor-historical
description: "Aquarela/wash/fluid REMOVIDOS (ADR-0096) + satélites aposentados — índice histórico"
metadata:
  node_type: memory
  type: reference
---

Memórias da era aquarela/wash (módulo deletado por ADR-0096; backups em `backups/`) e de decisões
supersedidas — mantidas como arquivos, fora do índice principal:

- [[reference_watercolor_state_of_art]] — Curtis 1997 + capilar; K–M multi-pigmento
- [[project_rebecca_watercolor_cleanroom]] — derivada do Rebelle; fingering é arquitetura
- [[project_watercolor_v2_gpu_first_refactor]] — pintura lenta é submit/copy-bound
- [[project_wash_undo_event_driven_rebuild]] — write parcial: atualize AMBOS gêmeos
- [[project_wash_pigment_color_mixbox_residual]] — c=unmix + residual r=rgb−mix(c)
- [[project_wash_gpu_resident_reimpl]] — portar física B1-B9; zero fallback CPU
- [[project_wash_curtis_gd_migration_2026_06_15]] — divergiram por NÃO implementar Curtis
- [[project_aquarela_paper_ramp_broken]] — REVERTIDA; não reconstrua sem pedir
- [[project_painter_fluid_4k_perf_architecture]] — hot loop CPU + readback é o gargalo
- [[project_painter_rendering_modes_research]] — design NÃO implementado (era watercolor)
- [[project_painter_w3_audit2_perf_2026_06_01]] — audit-2 W3 (tracker velho; partial GPU upload)
- [[project_wos_diffusion_over_budget_2026_06_06]] — vector W7 (sistema retirado por ADR-0108)
- [[project_vector_node_opaque_carrier]] — nodes de vector retirados (ADR-0108)
- [[project_brush_along_path_satellite_not_node]] — crates removidas (ADR-0099); lição na DIRETRIZ §2
- [[project_ktx2_phase1_done_phase2_aborted_2026_05_26]] — decisão registrada em CLAUDE.md §5
- [[project_imageio_avif_pathc_2026_05_28]] — registrada em CLAUDE.md §5 (precisa meson/nasm/cmake)
- [[project_panel_loc_gate_parser_masked_debt]] — resolvido; resto absorvido pelo gate atual
- [[project_brush_audit_2026_06_18]] — auditoria multiagente do brush engine (2026-06-18): 2 HIGH corrigidos; os claims de paridade CPU↔GPU MENTIAM (latentes não-gateados)
- [[project_painting_removed_layers_effects_kept]] — toda a pintura/brush engine foi DELETADA (ADR-0099, 2026-06-20); sobra o host de Layers+Efeitos
- HISTÓRICO: 3D (saiu do PH2D, ADR-0179; código em `b1a6f9b07`) — [campo implícito/SDF (24)](reference_topic_implicit_field_laws.md) · [quad remesh (20)](reference_topic_quad_remesh_rulers.md) · [MagicaCSG](reference_magicacsg_installed_under_wine.md) · [viés NDC](feedback_a_depth_bias_constant_in_ndc_is_not_constant_in_the_world.md) · [BSDF×albedo](feedback_a_per_texel_albedo_is_the_base_color_never_a_factor_at_the_end.md)
