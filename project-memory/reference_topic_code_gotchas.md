---
name: reference-topic-code-gotchas
description: Gotchas silenciosos de código do PH2D — IconId · registry-init · node-sync · companion allowlist · inject · pixel center · exact-pin · ISPC · zero-alloc · Arc::from · áudio mudo · OS-green · low-res (13)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:59:25.541Z
---

- [[feedback_new_tool_icon_needs_iconid]] — tool nova exige `IconId` (gate `enum_order_matches_svgs`)
- [[feedback_fanout_registry_init_friction]] — fan-out registry-init: 2 testes à mão
- [[feedback_node_sync_glob_prefix_gotcha]] — node-sync glob prefix: crate de nó ≠ `ph2d-node-`
- [[feedback_hier_companion_dispatch_allowlist]] — hier companion allowlist: 2 sites em `pointer.rs`
- [[feedback_pipeline_inject_dont_cap]] — inject, don't cap
- [[feedback_pixel_center_vs_edge_coord]] — pixel center vs edge: subtraia 0.5
- [[feedback_exact_pin_needs_substring_gate]] — exact-pin exige gate substring
- [[feedback_ispc_cross_process_concurrency]] — ISPC crasha com cargo concorrente entre processos
- [[feedback_zero_alloc_gate_capacity_not_global_counter]] — zero-alloc gate mede CAPACIDADE, não contador global
- [[reference_arc_from_vec_always_copies]] — `Arc::from(Vec)` SEMPRE copia; `collect` TrustedLen não
- [[project_audio_multichannel_silence]] — áudio: meter vivo, sem som = mute do WirePlumber
- [[project_painter_t19_latent_red_macos_2026_05_28]] — claimed-green ≠ seu-OS-green
- [[project_painter_canvas_res_64_not_sim_scale]] — Painter "low-res" = canvas 64px, não escala da sim
