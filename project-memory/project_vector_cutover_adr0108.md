---
name: project-vector-cutover-adr0108
description: Vector module repositioned (ADR-0108) + cutover done — new ph2d-vec-* engine + ph2d-tool-vector, old 30 crates retired; durable gotchas
metadata:
  type: project
---

O **Vector Module foi reposicionado** ([ADR-0108], plano `docs/Vector Module/18_plano_reposicionamento_rive_native.md`, Enio 2026-07-05/06): abandonou a ambição estratosférica (17_plano, W1..W20) por um módulo **GPU-first, editor-first, referenciado no runtime MIT do Rive** (reimplemento nativo kurbo/Vello, NÃO vendoriza rive-rs). Confie no repo, não nas notas antigas de "W1/W2".

**Motor novo = 4 crates KEEP:** `ph2d-vec-scene` (modelo de documento puro; path = âncora+2 handles estilo Rive `CubicVertex`; sem vello/kurbo/color — fica do lado certo do gate deferido `vello_kurbo_only`), `ph2d-vec-edit` (`PenTool` unificado desenho+edição-de-ponto + `History` por snapshot + `PenStyle`), `ph2d-vec-render` (BezPath→Vello), `ph2d-vec-boolean` (edit-time, linesweeper+kurbo).

**Cutover FECHADO 2026-07-06 (Fase R):** graduado à tool real **`ph2d-tool-vector`** (pill cluster `vector_tools`, `IconId::Vector`, painel Style = Width slider + paletas Stroke/Fill). **Arquitetura: document ≠ tool** — a cena (`AppGfx.vec_scene`) + pen/history vivem no SHELL; a tool carrega só config de estilo; `render_loop::vector_bridge` faz downcast (allowlistado) pra sincronizar estilo→pen + recolorir seleção. **Sistema antigo RETIRADO** (30 crates: 5 `ph2d-tool-vector-*` + 2 `ph2d-panel-vector-*` + 16 `ph2d-node-vector-*` + 7 libs graph/kurbo/sdf/fill/llm/llm-client/fanout-audit) + backend MCP `vector.*` (LLM4SVG) + chrome morto (LLM-prompt/API-key/point-type) + integração ECS vector-scene. **Rig+skinning (LBS port do Rive, MIT) = deferido pro FIM**, após o módulo de desenho completo.

**Gotchas duráveis do cutover (economizam horas):**
- **Icon codegen ordena por SLUG (`file_stem`, sem `.svg`), não por filename.** Então `"vector" < "vector-pen"` (prefixo). O drama `'-'`(0x2D)`<'.'`(0x2E) só morde se ordenar filenames — aqui NÃO. `build.rs` em `ph2d-editor-core` + gate `enum_order_matches_svgs`.
- **Gate `architecture_vector_contract_surface` FICA** — vive em `ph2d-vector-doc` (fundacional), escaneia SÓ `ph2d-vector-doc`+`-traits`; retirar nodes/tools **não o quebra**. As 4 fundacionais `ph2d-vector{,-doc,-traits,-font}` sempre ficam (o editor inteiro pinta via `ph2d-vector`).
- **Painter reusa `IconId::VectorPen/Pencil/Shape`** como glyphs compartilhados (Shapes flyout: Curve/FreeHand/Shapes) — NÃO são das tools deletadas; **não deletar** esses IconIds nem seus SVGs. Só `VectorDirect/VectorSelect` saíram.
- **Membership é glob (`crates/*`)** → deletar dirs basta. MAS todo `Cargo.toml` com dep numa crate deletada quebra o carregamento do workspace inteiro → **remova as deps ANTES** (senão nem os sync binaries rodam). Deps geradas de registry-init ficam em marker-regions `# <ph2d-*-sync:begin/end>` (blanke-as e re-rode o sync); há regiões HAND-MAINTAINED fora dos markers (ex.: contador de painéis em `panel-registry-init`).
- **4 sync binaries:** `cargo run -p ph2d-{tool,node,panel,chrome}-sync` regeneram registries + `chrome/mod.rs` (markers). Rode após deletar/adicionar crates ou toggles.
- **Teardown guiado pelo COMPILADOR** (deletar definições → `cargo check` enumera cada ref pendente, file:line limpos) bate grep (saída de grep vem ofuscada/ruidosa neste ambiente).
- **`ColorSwatch` em painel de TOOL só emite `Click`** pelo caminho do shell (`input_handlers.rs`); NÃO cruza pro sistema de picker OKLCH do chrome (dois sistemas de interação distintos). Picker real numa tool-panel exige plumbing cross-system novo → por ora a tool usa **paleta preset** (RadioGroup, `SelectOption`). Picker OKLCH = follow-up.
- Downcast de tool concreta no shell exige o arquivo estar no allowlist de `architecture_no_downcast_to_concrete_tool_in_shell` (novo `render_loop/vector_bridge.rs` entrou lá).

Ver também [[project-node-centric-decision-2026-05-21]] (norte de nós), [[feedback-new-tool-icon-needs-iconid]], [[feedback-fanout-registry-init-friction]].
