═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · W3 fechado, abrir W4
Autor: Implementador Painter (sessão 2026-06-01/02) · você roda em CONTEXTO SEPARADO
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ W3 (multi-layer) está FEATURE-COMPLETO: 22 blends, máscara, clipping,║
║ alpha-lock, reference, group, DRAG (reorder + into-group). Engine +  ║
║ UI prontos e testados. Tua missão: FECHAR W3 (smoke+audit T3.9) +     ║
║ MULTI-SELEÇÃO (aberto) e abrir W4 (Adjustment Layers — HSB primeiro).║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§0 — ESTADO ATUAL (confirme no git; NÃO refaça)
───────────────────────────────────────────────────────────────────
W0-W2 ratificados. **W3 SHIPADO pelo Coord** (T3.5-T3.8, CI VERDE nas 3 plataformas:
github.com/dibrioli/PH2D/actions/runs/26816195448). Em cima disso, **3 commits LOCAIS
meus (ainda NÃO pushados — `git log origin/main..HEAD`):**
  - `75e89ed` modifier toolbar (Mask/Clip/Lock/Ref — 2ª barra agindo na camada ATIVA).
  - `2ba5774` mask row (sub-row indentada selecionável pra re-editar a máscara).
  - `b8b3b0c` mask brush → preto ao entrar na máscara (senão masking parecia morto).
O shell COMPILA (o agente Vector terminou o scaffold dele). **Pode smokar.**

Auditoria adversarial 6-lentes do W3 = **ZERO CRITICAL** (relatório:
`HANDOFF_painter_w3_audit2_coord_items.md`, memória `project-painter-w3-audit2-perf`).
Perf: dirty-rect + B.1 upload-parcial GPU + B.5 publish gateado em `layers_revision()`.

───────────────────────────────────────────────────────────────────
§1 — ROTEADOR (leia SÓ o que tua tarefa exige — NÃO leia tudo inteiro)
───────────────────────────────────────────────────────────────────
  - `CLAUDE.md` (§0 inegociáveis · §1 roteador · §6 contratos congelados).
  - **O PLANO:** `docs/Painter_projeto/15_plano_de_implementacao.md` — §6=W3 (fechando),
    **§7=W4 Adjustment Layers** (teu próximo grande), §8+=W5+. É o teu mapa.
  - `docs/Painter_projeto/02_layers.md` — §2.7 mask, §2.8 clipping, §2.9 reference,
    §2.10 alpha-lock, §2.11 compositor, §2.3 layer menu (13 ops).
  - ADRs Painter `0043..0053`; W4 usa **ADR-0045** (Adjustment contract).
  - DIRETRIZ §2 (triagem) · §3.D (modificar feature) · §5 (UI canônica + Widget
    Gallery §5.2/§5.3 = **fonte da verdade de UI**) · §6 (velocidade) · §7 (git).
  - Memória: `MEMORY.md` (índice) + `project-painter-w3-audit2-perf-2026-06-01`
    (estado DETALHADO desta wave: T3.5-T3.8, lições, commits).

───────────────────────────────────────────────────────────────────
§2 — TEU TRABALHO (ordem sugerida)
───────────────────────────────────────────────────────────────────
1. **T3.9 — fechar W3:** smoke do batch (shell verde agora) + auditoria adversarial
   multi-lente do W3 inteiro (rotacione lentes, ≥2 paralelas). Feche achados in-pasta
   NA SESSÃO ([[feedback-perfection-no-deferrals]]).
2. **MULTI-SELEÇÃO (ABERTO — a fundação do Coord já existe):** o Coord landou TODO o
   dispatch (`HANDOFF_painter_w3_multiselect_drag_coord.md`): `WidgetStore::cmd_held()/
   shift_held()` + drag. **Falta o consumidor IN-PASTA:**
     - `PainterTool`: um `SelectionSet` (Vec/BTreeSet<LayerId>) + `select_single/_additive/
       _range` + `group_selected` (cria grupo + move TODOS os selecionados).
     - no `event.rs::apply_event`, ao receber `Click(row_id)`: ler
       `host.store().cmd_held()` (aditivo) / `shift_held()` (range) → rotear pro select.
     - painel: highlight de TODAS as rows selecionadas (hoje só `active`) + caminho-2
       (selection-mode/checkbox) opcional. Trocar o "Group" do header de `group_active`
       (envolve só a ativa, interim) → `group_selected`.
3. **Mask Invert/Apply UI:** `LayerStack::set_mask_inverted` + um bake destrutivo
   (Apply mask → multiplica alpha do pai, remove a máscara) — falta a UI (toggle Invert
   na row da máscara + ação Apply). Engine do invert JÁ existe (compositor usa `1-v`).
4. **W4 — Adjustment Layers** (plano §7, ADR-0045): HSB primeiro; `AdjustmentKind` enum
   (cap ≤32, congelado) + recomposição não-destrutiva no compositor. Caminho B (scaffold)
   + C (compositor foundational = Coord). Triagem (DIRETRIZ §2) é teu 1º output.

OPCIONAL/polish: o **menu por-layer completo (§2.3, 13 ops)** — eu fiz a barra de
modificadores agindo na ATIVA como v1 pragmático; o menu por-row é o design completo.

───────────────────────────────────────────────────────────────────
§3 — PADRÕES + ARMADILHAS (reuse; JÁ QUEIMARAM nesta sessão)
───────────────────────────────────────────────────────────────────
  - **Botão fixo novo EXIGE register em `populate.rs`** do painel — pintar + hit_index
    NÃO basta, o dispatcher dropa o click em silêncio ([[feedback-panel-populate-register]]).
    (Delete/Duplicate não funcionavam por isso.) E forward em `event.rs` + route em
    `handle_panel_event`.
  - **Máscara é owner-attached** (NÃO está no z-order/`root`): selecionável mas NÃO
    arrastável — `LayerStack::is_mask` filtra do row-set publicado + guard no
    `handle_layer_reparent`. `collect_subtree` coleta a máscara no remove.
  - **Mask brush = PRETO por default** ao entrar (máscara começa BRANCA = tudo visível;
    cor default é laranja CLARO → grayed mal esconde). `sync_mask_brush_color` salva/
    restaura a cor real. `effective_active_color` força chroma 0 ao pintar máscara.
  - **DRAG (T3.8):** `WidgetEvent::PainterLayerReparent { dragged, drop: PainterLayerDrop }`
    (Coord) → shell `painter_bridge::apply_layer_reparent` (downcast, arquivo allowlisted)
    → `PainterTool::handle_layer_reparent`. O painel publica
    `store.set_painter_layer_row_ids(...)` por frame.
  - **Popover canônico:** espelhe o blend dropdown (`blend.rs::paint_blend_popover` +
    `PENDING_BLEND_DD` em `state.rs` — deferred paint no fim, on top). Pra QUALQUER menu/popover.
  - **TRÊS drenos de preview compõem** (`take_preview_arc`/`current_preview`/`run_full`).
    Mexeu no composite → cheque os 3. `is_trivial_stack` já guarda mask/clipping.
  - **B.5 publish gate:** o bridge só republica o LayerStack quando `layers_revision()`
    muda (bump em `invalidate_composite` + `set_source`; stroke NÃO bumpa).
  - **Color:** straight no canvas; blend em LINEAR; premul byte-space (convenção
    project-wide, preview≡Apply). Não grave linear em buffer sRGB.
  - **base = bottom-of-root = a sprite** (sem blend; reorder/delete travam ela no fundo).
  - **Q16.16:** posição fora da janela (|v|≥32768) é DROPADA (`f32_to_q1616_checked`), não
    clampada (B.4).
  - UI strings em INGLÊS ([[feedback-app-ui-english-only]]).

───────────────────────────────────────────────────────────────────
§4 — TUA PASTA + GATES (isolamento DIRETRIZ §1.4)
───────────────────────────────────────────────────────────────────
EDITE: `crates/ph2d-tool-painter/` · `crates/ph2d-painter-brush/` ·
`crates/ph2d-panel-painter-layers/` (paint.rs orquestrador + **paint_rows.rs** rows) ·
`crates/ph2d-panel-painter-sidebar/` · `crates/ph2d-editor-core/src/ids.rs` (SÓ aditivo) ·
`shells/desktop/src/render_loop/painter_bridge.rs` (publish + preview + a rota de drag).
NÃO TOQUE (PARE e reporte ao Coord): `ph2d-render`, `ph2d-painter-stroke` (savefile
congelado), `shells/` além do bridge, contratos congelados (§6 CLAUDE.md), o dispatch
de `editor-core/interaction/` (foundational).
GATES (batched no fim do bloco — `cargo check` ESCONDE eles):
  cargo test -p ph2d-editor-core --test no_magic_numeric --test no_literal_color \
    --test hr12_widgets_a11y --test architecture_panel_loc_cap --test node_id_collisions
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
  cargo clippy -p <tuas crates> --all-targets --no-deps -- -D warnings
**panel_loc_cap:** por-fn 200 E por-arquivo 600 → `paint.rs` JÁ foi dividido em
`paint_rows.rs`; se estourar de novo, divida em mais um sibling. **Bug do parser:**
apóstrofo/aspas em comentário `//` infla a contagem ([[project-panel-loc-gate-parser-masked-debt]]).

───────────────────────────────────────────────────────────────────
§5 — COORD-SCOPE (NÃO faça; coordene)
───────────────────────────────────────────────────────────────────
  - **CHROME_IDS:** adicionar os 4 ids novos (`PAINTER_LAYERS_MASK/CLIP/ALPHA_LOCK/
    REFERENCE`) ao `tests/node_id_collisions.rs` (espelho do batch anterior — passam,
    só não-cobertos). Follow-up Coord pendente.
  - **GPU LayerCompositor** (real-time many-layers) — Coord, sequenciado. O CPU
    composite + dirty-rect + B.5 deixam W3 aceitável (não bloqueia W4).
  - **Persistência host** (save/load do doc Painter) — não existe; bloqueado em W12
    Reproject. Quando existir: tu expões `LayerStack::from_nodes` + setter de `next_id`.
  - W4 compositor foundational (se tocar o caminho real-time) — Coord (caminho C).

───────────────────────────────────────────────────────────────────
§6 — GIT + MULTI-AGENTE (Coord + agente Vector ativos no MESMO working tree!)
───────────────────────────────────────────────────────────────────
Você NÃO pusha (Coord faz ship 1× no fim). **CUIDADO COM COLISÃO:**
  - `git status` antes de stage; `git add -- <SÓ teus paths>`; `git commit --no-verify
    -F <msgfile> -- <paths>` (aspas em -m quebram com parênteses → use -F arquivo).
  - WIP alheio (Vector: `ph2d-tool-vector-*`, `ph2d-vector*`, shell; docs; .vscode)
    NÃO comite. Commite SÓ tuas 4-5 crates.
  - **`index.lock`:** outro agente commitando → espere clarear (loop `[ -f .git/index.lock ]`
    + sleep) antes de stage. O shell/clippy do workspace pode ficar VERMELHO transitório
    pelo WIP do Vector — teus crates compilam isolados (`-p <crate> --no-deps`); não é teu.
  - Slot CoW pra velocidade: `bash scripts/slot-seed.sh <slot>` → prefixe `CARGO_TARGET_DIR`.
═══════════════════════════════════════════════════════════════════
