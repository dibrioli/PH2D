═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · W3 UI-plumbing (panel interativo + dock + bridge)
Autor: Implementador Painter anterior (sessão 2026-05-31/06-01) · você roda em CONTEXTO SEPARADO
Plano: docs/Painter_projeto/15_plano_de_implementacao.md §6 · design: 02_layers.md
Regras: CLAUDE.md (§0 inegociáveis) + DIRETRIZ.md (§1.4 isolamento, §3.D, §5 UI canônica, §6.6 velocidade)
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ O BACKEND da W3 (layers) está PRONTO, TESTADO e AUDITADO (5 lentes, ║
║ todos CRITICAL/HIGH/MED fechados). Falta a UI-PLUMBING que conecta  ║
║ o painel ao backend — é a TUA tarefa. Nada disso é visível in-app   ║
║ ainda; o teu bloco é o que torna smoke-able.                        ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§0 — SANITY CHECK (rode primeiro)
───────────────────────────────────────────────────────────────────
  git log --oneline -12   # HEAD deve conter 84ae5d7 (audit remediation)
  source scripts/slot-env.sh impl-painter  # OU bash scripts/slot-seed.sh impl-painter
    # prefixe TODO cargo com o CARGO_TARGET_DIR impresso (env não persiste no Bash tool)
  CARGO_TARGET_DIR=<slot> cargo nextest run -p ph2d-tool-painter -p ph2d-painter-brush -p ph2d-panel-painter-layers
    # esperado: ~315 verdes
  ⚠️ Working tree TEM WIP do Coordenador (ph2d-render KTX2). NADA disso é teu.
    Você NÃO pusha (Coord faz ship 1× no fim). Commits locais escopados.

───────────────────────────────────────────────────────────────────
SUA PASTA (edite SÓ aqui — DIRETRIZ §1.4)
───────────────────────────────────────────────────────────────────
  crates/ph2d-panel-painter-layers/   (o painel — paint/event/populate/ids/state)
  crates/ph2d-tool-painter/           (handle_panel_event + os métodos de layer)
  (mesmo módulo se a task exigir: ph2d-painter-brush/)
  crates/ph2d-editor-core/src/ids.rs  (SÓ pra adicionar o helper de id por-row — aditivo,
    como os `hier_*_companion`; é a única exceção foundational, padrão estabelecido)
NÃO TOQUE (foundational/Coord — PARE e reporte): ph2d-render, ph2d-painter-stroke
  (savefile congelado v2 ADR-0046-amд-1), shells/ além do bridge publish abaixo,
  contratos congelados.

───────────────────────────────────────────────────────────────────
O QUE JÁ ESTÁ PRONTO (NÃO refaça — confirme no git)
───────────────────────────────────────────────────────────────────
- T3.3 blend modes (`ph2d-painter-brush/src/blend.rs`): `BlendMode` 22 modos +
  `apply(mode,dst,src)` (W3C, linear-sRGB) + `name()`/`to_u8()`/`from_u8()`/`MAX_BLEND_MODES`.
- T3.1 LayerStack (`ph2d-tool-painter/src/layers.rs`): `LayerStack`/`LayerId`(u64,
  aliasado `RtLayerId` na tool)/`LayerKind`/`Layer` + ops (cap 8 níveis, cap 999).
- T3.2 compositor CPU (`ph2d-tool-painter/src/compositor.rs`): `composite`/`composite_region`.
- T3.4 scaffold + **read-only rows** (`ph2d-panel-painter-layers`): pinta cada layer
  (eye/thumb/name/"Blend NN%") via `state::current_layers()`. Painel DORMENTE
  (`painter_bridge.rs`: visibility=false) até o dock landar.
- **Integração tool↔LayerStack** (Opção A ratificada): `PainterTool` possui
  `layers`+`images`; `current_preview` fast-path N=1 byte-idêntico | composite.
- **Mecanismo multi-layer**: add/select com swap de buffer.
- **Compositor GPU** (Coord, `ph2d-render`, paridade ≤1B): API em
  `HANDOFF_painter_w3_block2_done.md` (`LayerCompositor`/`LayerOp`/`flatten_layer_ops`).
- **Persistência v2** (Coord, ADR-0046-amд-1): formato congelado pronto; a PONTE
  runtime↔savefile é follow-up (vide §FOLLOW-UPS).

A API da tool que você vai acionar (PRONTA + testada):
  painter.layers() -> &LayerStack                     // fonte do snapshot do painel
  painter.set_layer_visible(RtLayerId, bool)
  painter.set_layer_opacity(RtLayerId, f32)
  painter.set_layer_blend_mode(RtLayerId, BlendMode)
  painter.add_raster_layer(name) -> Option<RtLayerId> // cria+ativa no topo (swap de buffer)
  painter.select_layer(RtLayerId)                      // troca ativa (swap de buffer)
  ph2d_panel_painter_layers::set_current_layers(Option<LayerStack>) // bridge publica

───────────────────────────────────────────────────────────────────
SUA TAREFA — 3 peças (DoD: criar 2 layers, pintar em cada, ver compor, toggle visibility)
───────────────────────────────────────────────────────────────────
**(1) Widgets interativos por-row no painel** (`ph2d-panel-painter-layers`):
  - Ids dinâmicos por-row: adicione em `editor-core/src/ids.rs` um helper
    `pub fn painter_layer_widget_id(layer_id: u64, kind: PainterLayerWidget) -> NodeId`
    = `hash_node_id(&format!("painter_layer.{}.{}", kind.tag(), layer_id))`
    (FNV via `ph2d_tool_registry::hash_node_id`; é runtime mas o painel não é hot path —
    format! por-row é aceitável, igual o sidebar formata "NN px"). + um const fixo
    `PAINTER_LAYERS_ADD`. kinds: Row(select) · Visibility · Opacity · Blend.
    (Alternativa: padrão companion-bit dos `hier_*_companion` — mas hash é mais simples
    e collision-safe; o tool decodifica iterando layers × kinds e casando o id.)
  - No `paint.rs`: registre o hit de cada widget por-row (eye toggle, opacity slider,
    blend dropdown/chip) + um botão "+Layer" (`PAINTER_LAYERS_ADD`). Reuse os widgets
    canônicos do Widget Gallery (paint_toggle/paint_slider/paint_button/blend popover).
    Os rows hoje são read-only — troque os indicadores por widgets clicáveis.
  - `populate.rs`: registre os widgets por-row (espelha o sidebar; [[feedback-panel-populate-register]]
    — sem register, o click é dropado em silêncio).
  - `event.rs`: classifique cada WidgetEvent → `EditorAction::ToolPanelEvent(PanelEvent::…)`
    (Toggle/SetValue/SelectOption/Click) — canal genérico TG-B, mirror do sidebar.
**(2) `handle_panel_event` na tool** (`ph2d-tool-painter/src/tool.rs`):
  - Hoje roteia os sliders do sidebar por id fixo. ADICIONE: decode do id por-row
    (itere `self.layers` × kinds, compute `painter_layer_widget_id`, case) → chame
    set_layer_visible/opacity/blend_mode/select_layer; `PAINTER_LAYERS_ADD` → add_raster_layer.
**(3) Dock toggle C + bridge publish** (`shells/desktop/src/render_loop/painter_bridge.rs`):
  - Enio escolheu **C = toggle**: um botão alterna brush-settings ⇄ layers no mesmo slot.
    Sugestão mínima: thread-local de modo (`painter_dock_shows_layers: bool`) + um botão
    no header de cada painel (sidebar mostra "Layers", layers mostra "Brush"). Visibility:
    sidebar = active && !show_layers; layers = active && show_layers.
  - Bridge publish: troque `set_current_layers(None)` → `Some(painter.layers().clone())`
    quando painter ativo (o accessor `layers()` já existe). E flipe a visibility (hoje
    `insert("painter_layers", false)` — vide o comentário SCAFFOLD lá).
  - GPU real-time: opcional agora (o CPU `current_preview` já compõe); quando quiser perf,
    use a API do Bloco 2 do Coord (HANDOFF_painter_w3_block2_done.md).

───────────────────────────────────────────────────────────────────
ARMADILHAS (decoradas — me custaram caro)
───────────────────────────────────────────────────────────────────
  1. **GATES ESCONDIDOS** ([[feedback-full-gate-periodically]]): `cargo check` + `no_literal_color`
     NÃO pegam `arch_color_space_typed` (no_bare_byte_color) NEM `no_magic_numeric`. A
     auditoria achou 2 RED que eu não rodei. RODE no fechamento:
       cargo test -p ph2d-editor-core --test arch_color_space_typed --test no_magic_numeric \
         --test no_literal_color --test hr12_widgets_a11y --test architecture_panel_loc_cap
       cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
       clippy --all-targets -p <tuas crates> -- -D warnings
     `Vec<u8>`/`&[u8]` de cor em `pub fn` precisa `// COLOR-RAW-OK:` NA LINHA do pub fn
     (single-line; multi-line escapa o parser). `f32` não-design (ex.: *100.0) precisa
     `// LITERAL-PX-OK:` na linha.
  2. **RtLayerId (runtime u64) vs device::LayerId (u32)**: `layer_target` usa device::LayerId;
     o modelo de layer usa RtLayerId. NÃO confunda. Save narrow u64→u32 é a PONTE (follow-up).
  3. **Undo cruza layer switch = corrupção** (auditoria CRITICAL, já mitigado): hoje
     add/select RESETAM o undo (`reset_undo_after_layer_switch`). Logo o usuário perde
     undo ao trocar de layer — aceitável v1. Undo per-layer (transações) é follow-up do
     Coord; NÃO tente per-layer undo sem o ADR dele.
  4. **gamma/sRGB**: qualquer pixel novo respeita linear↔sRGB (compositor decode/encode via
     `ph2d_color::srgb`). Não grave linear num buffer sRGB (bug histórico).
  5. **Agentes paralelos na mesma árvore** = colisão ([[feedback-parallel-agent-collision]]):
     se rodar auditoria multiagêntica, os probes podem deixar resíduo — `git status` + grep
     antes de confiar nos gates.
  6. UI strings em INGLÊS ([[feedback-app-ui-english-only]]).

───────────────────────────────────────────────────────────────────
AUDITORIA W3 — RESULTADO (5 lentes, sessão anterior)
───────────────────────────────────────────────────────────────────
Tudo CRITICAL/HIGH/MEDIUM/LOW fechado em `84ae5d7`. Confirmados CORRETOS por sweep:
blend math (22 modos W3C + HSL), compositor color-space (gamma OK), fast-path exatidão,
buffer-swap round-trip, CPU↔GPU blend parity. FLAGS pro Coord (cross-boundary, NÃO tuas):
  - Semântica exata do cap de profundidade (alinhei runtime a 8 níveis = savefile; confirmar).
  - GPU-readback parity deveria exercer um caso non-separable (Hue/Saturation) midtone.
  - Gate `layers_blend_mode_golden` (SSIM vs Photoshop) ainda ausente — coberto por math-tests.

───────────────────────────────────────────────────────────────────
FOLLOW-UPS (depois do teu bloco — alguns Coord)
───────────────────────────────────────────────────────────────────
  - Undo per-layer transacional (Coord, ADR-0046-amд / same-ring).
  - Ponte de persistência runtime↔savefile v2 (TUA, sobre o formato do Coord;
    contrato em HANDOFF_painter_w3_layerstack_divergence_RATIFIED.md §4: z-order top-first,
    bools→bits modifiers, blend.to_u8(), mask→Box<LayerNode>, narrow id u64→u32, load
    widen + next_id=max+1).
  - `run_full` ainda baka só `canvas_rgba` (N=1); commit/flatten multi-layer = follow-up.
  - Mask (T3.5) + clipping (T3.6): estendem o compositor (pontos de extensão comentados;
    `collect_subtree` tem TODO pra mask subtree). Mask lifecycle desenhado em T3.5.

───────────────────────────────────────────────────────────────────
GIT + COMMITS DESTA JORNADA (locais, não-pushados)
───────────────────────────────────────────────────────────────────
  W2: c43d5d7 swatch · 59555b7 T2.4/T2.6 · 602f32d eyedropper-ícone · cb976b3 eyedropper funcional
  W3: 33fb4eb blend+layers · 1bed40e compositor · 6e17c5a panel-render · efe59b9 scaffold(Coord)
      · 5d91c91 tool-integration · a375479 multi-layer · 84ae5d7 audit-remediation
  Coord W3: 6ba3ed7 GPU compositor · 249735e persistência v2 · 2106e99 ratify divergência
  Regras: `git add -- <só teus paths>`; `git commit --no-verify -m '…' -- <paths>` (ASPAS
  SIMPLES — backtick em -m "…" sofre command-substitution no bash e mangleia a msg).
  `git status` antes de stage; WIP alheio (ph2d-render KTX2, docs, .vscode) NÃO comite.
═══════════════════════════════════════════════════════════════════
