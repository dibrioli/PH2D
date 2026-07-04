---
name: project-painter-w3-block2-persist-ktx2-2026-06-01
description: "Painter W3 Block 2 GPU compositor + persistência v2 layer-stack + KTX2 magenta — fechados, auditados (1 CRITICAL DoS remediado), local"
metadata: 
  node_type: memory
  type: project
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

Coord (2026-06-01) fechou 3 peças foundational do Painter W3 + KTX2, todas LOCAIS (não-pushadas), auditadas por workflow multi-agêntico (33 agentes, 6 lentes adversariais).

**Painter W3 Block 2 — compositor GPU** (`6ba3ed7`): `ph2d-render::LayerCompositor` — 22 modos W3C + grupos via stack-machine por-pixel, **2 entry points** `cs_flat`/`cs_grouped` (flat = sem array de stack → alta ocupação; grouped só quando há grupos). Cache `texture_2d_array` (BTreeMap key→slice + dirty por versão + LRU) + dirty-rect. decode sRGB via **LUT 256** (bit-exata + 20× mais rápida que pow). Paridade bit-a-bit vs `apply_blend` ≤1 byte. API: `LayerOp::{Layer{key:u64,blend_mode:u8,opacity},PushGroup,PopGroup}` + `LayerPixelProvider`. Desacoplado (fala u8 cru, dev-dep só pro gate).

**Persistência v2** (`249735e`, ADR-0046-amд-1): `device::LayerStackEntry::Node(LayerNode)=1` (Reserved=0 frozen) — layer stack sobrevive save/load. Ponte u64↔u32 (stroke records ficam u32). Migração v1→v2 (1 raster default).

**KTX2 W2.T4 fechado** (`385e7e2`): magenta missing-texture placeholder via `mark_missing` (addendum do plano linha 372).

**Lição-perf (durável):** recompose 4K-cheio × 50 layers lê **1.66 GB → é bandwidth-bound**, não shader-bound. Esta dev Mac (8GB, M-base) tem ~70 GB/s → piso ~23ms; o budget de 5ms literal exige GPU ≥330 GB/s. **Não gateie 5ms no full-recompose** — gateie o caminho INTERATIVO (dirty-rect, <5ms real) + escala-linear no full. Probe revelou: remover pow/stack/blend não mexe o full → é memória, não ALU. Spec reconciliada (02_layers §2.12, 08_perf).

**Lição-DoS (durável, CRITICAL achado pela auditoria):** tipo serde recursivo (`Group{children:Vec<LayerNode>}` + `mask:Option<Box>`) com `Deserialize` DERIVADO + postcard (sem depth-limit) → `from_bytes` estoura a pilha num file forjado ANTES de qualquer validação (validação roda DEPOIS = inútil contra isso; SIGABRT incatchável). **Fix:** `Deserialize` hand-written com guarda de profundidade (thread-local + RAII guard) erra acima de um bound; `Serialize` segue derivado (wire/cook-hash intactos). Regra: **todo tipo recursivo serde num boundary de trust precisa de deserialize depth-bounded**, não só validação pós-hoc.

**Auditoria:** 27 achados → 15 confirmados, 12 refutados (verificação adversarial derrubou falsos-positivos). 1 CRITICAL + 6 LOW/MEDIUM remediados (`4368a77`, `834b840`). Relatório: `docs/AUDIT_painter_w3_ktx2_session_2026-06-01.md`. Aceitos-LOW (sem caller de produção ainda): eviction/version GPU tests, region clamp test, magenta bridge test.

Implementador (janela separada) fez Block 1 in-memory (`5d91c91`/`a375479`) + painel read-only (`6e17c5a`). Pendente Enio: dock toggle (C) + ship. Relacionado: [[feedback_perfection_no_deferrals]] [[feedback_audit_lens_diversity]].
