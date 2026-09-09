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
- ⛔ **Um serviço opcional lido de forma OBRIGATÓRIA vira requisito** — `with_registry_ref` (que faz
  `panic!`) num caminho que corre dentro de `HeroScreen::new` matou **12 testes de chrome** que nada
  têm com painéis; a variante `_opt` existe exactamente para isso. *Quem paga é quem nunca pediu o
  serviço* (`line/UIUX`, 2026-08-30, entrega 21)
- [[feedback_a_clamp_before_a_range_test_deletes_the_test]] — `clamp` antes de `contains` apaga o `if`; teste o valor CRU
- ⛔⛔ **Uma chave de cache que hasheia a SPEC mas não o CÓDIGO GERADO À VOLTA dela colide quando a
  spec é PARTILHADA.** Ao dar às reduções um canal de WGSL por-nó (`wgsl_shared`, 08/09), a chave
  do pipeline de `map` continuava a ser `(value, name, column, dim, op, present, params,
  antecessoras)` — e a `pivot::CENTROID_CX` é **literalmente a mesma `ReduceSpec`** em vários nós.
  O primeiro a declarar um canal emprestaria o módulo dele a todos os outros: um pipeline com
  funções que a expressão do vizinho não pede, ou **sem as que ela pede**. ⇒ *tudo o que entra no
  texto do módulo entra na chave*, e uma constante partilhada entre crates é exactamente onde isso
  morde. (O cache do KERNEL estava ileso: a chave dele já é o `NodeTypeId`.)
- ⭐ **Uma recusa cuja razão é «o substrato não tem onde pôr isto» mede o substrato, não a feature
  — e um ponto de extensão APPEND-ONLY costuma custar menos que a recusa.** O
  `motion.bend ▸ direction` ficou fora do dispositivo por o módulo de uma redução não ter onde pôr
  uma função auxiliar (o polinómio HR-5 teria de ser escrito uma 2.ª vez dentro da string). O
  canal novo é **um método de trait com default vazio**: nenhum implementador muda, todo módulo
  que não o declare sai byte a byte igual, e a recusa desaparece. ⚠️ Ao reconferir uma nota
  dessas, vá **ao gerador** ver o que ele de facto cola — não deduza do doc-comment.
