---
name: project-brush-along-path-satellite-not-node
description: "W8 brush bridge = crate satélite que lê contratos congelados, NÃO um graph-node raster (que exigiria Domain::Raster foundational p/ 1 consumidor dead-end)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 00710036-c40d-4ddb-8905-3f77e41f6f0c
---

O **brush bridge** do W8 (stampar brush ao longo de um path vetorial — a metade raster deferida pelo impl) foi feito como **crate satélite isolada `ph2d-brush-along-path`** (`f97e477`), NÃO como graph-node.

**Por quê (decisão durável):** um graph-node raster seria um *dead-end* — nenhum outro node consome raster, então exigiria surface FOUNDATIONAL nova só pra ele: `Domain::Raster` + `RASTER_PORT` + glue `ph2d-raster-graph` (espelho do `ph2d-vector-graph`) + consumo no renderer + ADR. Over-engineering pra **1 consumidor**. O `CookValue` só tem `{Empty, Instances, Opaque}` — sem variante raster; um raster *poderia* andar no `Opaque` (como `VectorNetwork`), mas não há precedente nem 2º consumidor que justifique.

**How to apply:** quando uma feature cross-module precisaria de carrier/domain foundational novo pra UM consumidor, prefira uma **crate satélite** que só LÊ os contratos congelados que ela ponte (aqui: `VectorNetwork` + `Stamp` 96B/ADR-0044) e não edita nenhuma crate alheia — zero mudança de roster/cap, isolamento intacto. Defira o foundational até ≥2 consumidores. O core devolve o tipo bruto (`Vec<Stamp>`) que serve qualquer produtização depois (op do Painter via `StampPipeline`/`apply_stamps`, OU um node raster no futuro). Gotcha: a satélite NÃO pode ter prefixo `ph2d-node-` (node-sync auto-descobre + gera `::register` inexistente → quebra registry-init) — vide [[feedback_node_sync_glob_prefix_gotcha]]. Relacionado: [[feedback_convention_vs_inertia]] [[project_vector_node_opaque_carrier]] [[project_tool_isolation_freeze_2026_05_22]].
