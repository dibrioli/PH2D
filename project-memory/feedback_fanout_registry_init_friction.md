---
name: feedback-fanout-registry-init-friction
description: "Em batches fan-out de N tools paralelas, registry-init tem 2 testes hand-maintained (canonical Image Tools cluster order + expected_icon_slug map) que tool-sync NÃO regenera. Estendê-los faz parte do trabalho do Implementador, não só do Coord pre-work."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 81bae6ea-3272-4a9b-9381-2ebdcbbd80f7
---

Em **batches fan-out de N tools paralelas** (ex.: `image_tools_4` 2026-05-23 — 4 sessões adicionando 4 image tools), o `ph2d-tool-sync` regenera os blocos entre marcadores codegen mas **NÃO** atualiza:

1. **`crates/ph2d-tool-registry-init/src/lib.rs::image_tools_cluster_in_canonical_order`** — unit test com `vec![…]` hardcoded da ordem do cluster `image_tools` por `order`. Sem extensão, fail no test.
2. **`crates/ph2d-tool-registry-init/tests/tool_manifest_design_sync.rs::expected_icon_slug`** — fn que mapeia `manifest_id → icon_slug`. Sem entries para novos tools, retorna `<unknown>` e o test `every_registered_manifest_has_matching_design_toml` falha.

**Why:** o Coord pre-work do batch `ccf0cf0` (4 SVGs + 4 IconId + 4 design TOMLs) parou um passo antes — não estendeu esses 2 testes. Cada Implementador rodando sync individualmente tropeça neles. Estender uma vez, no primeiro commit do batch, unblocks todas as outras sessões paralelas (elas só rodam sync e veem registry-init já curado).

**How to apply:** quando você for o **primeiro** Implementador a fechar num batch fan-out de N tools, estenda esses 2 testes para os N tools antecipadamente (não só pro seu). Os outros agentes não vão precisar tocar. Se a tarefa for um tool isolado (não-batch), só extenda pro seu. Editar registry-init nesses casos NÃO viola §1.3 "isolamento" — está fora dos blocos `<ph2d-tool-sync:*>`, é hand-maintained, e o briefing literal só proíbe edit dos blocos codegen'd.

**Bonus — pre-commit fmt em batch paralelo:** o hook roda `cargo fmt --check` workspace-wide, então drift de fmt em arquivos de outros agentes paralelos (que você não pode tocar) bloqueia seu commit. Use `git commit --no-verify` per [[feedback-fast-mode-ship]] — esse é exatamente o caso fast-mode contemplado em DIRETRIZ §7.0. Documente no body do commit por que o --no-verify foi usado.
