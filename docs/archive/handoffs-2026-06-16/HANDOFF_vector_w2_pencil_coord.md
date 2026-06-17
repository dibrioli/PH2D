═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Vector W2 · T2.1 Pencil (impl → Coord wiring)
Autor: Implementador (slot-impl-vector) · 2026-06-01
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR

**T2.1 Pencil core está PRONTO e verificado** (Hobby fitter + tool crate +
registro completo, todos os gates verdes). Falta **fiação central do shell**
(§3 — território Coord: `render_loop/mod.rs`, `input_dispatch.rs`, `ids.rs`,
`fixture.rs`, `chrome/mod.rs`, `keyboard.rs`, `shells/desktop/Cargo.toml`) +
**1 decisão de renderer** (§2). Os 3 arquivos `vector_pencil_*` do shell já
estão escritos no meu território, prontos pra você ligar.

NÃO pushei (você faz o ship). Commits locais escopados nos meus paths.

---

## §1 — Pronto + verificado (cargo no slot-impl-vector)

| Artefato | Local | Verificação |
|---|---|---|
| **Hobby fitter** | `crates/ph2d-vector-doc/src/hobby.rs` (novo módulo, additive) | 12 testes ✓ (G1 contínuo, interpola knots, simetria, robustez não-finito/coincidente/cusp, determinismo) |
| **Pencil tool** | `crates/ph2d-tool-vector-pencil/` (crate novo) | 23 testes ✓ (record→decimate→Hobby→commit, DoD 1-cubic/10-samples, replay-safe, NaN-guard, drain) |
| IconId + SVG | `icons.rs` (`VectorPencil` + `ALL_ICONS`) + `docs/design/icons/vector-pencil.svg` | `enum_order_matches_svgs` ✓ |
| Design TOML | `docs/design/tools/vector_pencil.toml` | design-sync 3/3 ✓ |
| Codegen (tool-sync) | `register_all` + `register_all_tools` + `Cargo.toml` deps + icon-slugs map | staleness 6/6 ✓ |

O Pencil **já está registrado nos dois registries** (manifest + behavior) via
`cargo run -p ph2d-tool-sync`. Ativar via `EditorAction::ActivateTool{tool_id:
"vector_pencil"}` já funciona no drain genérico do `mod.rs` — só falta o pill +
input + render.

### Algoritmo (porquê Hobby, não Schneider)
- Plano §5 T2.1 manda **Hobby** (minimum curvature variation, MetaPost);
  Schneider é o anti-padrão. Implementei a formulação Jackowski (velocity
  `ρ(α,β)=2/(1+⅔·cosβ+⅓·cosα)`, cap 4) com sistema tridiagonal
  estritamente diagonal-dominante (Thomas estável sem pivoting).
- Pipeline do tool: grava samples → decima (~1 knot/10 samples, dedup de
  jitter) → `hobby::fit_hobby_open` → emite **path aberto stroked** (sem region).
- **HR-5:** o fit usa sin/cos/atan2 (write-path, input freehand não-reproduzível,
  network `deterministic=false`) — documentado no módulo (mesma fronteira do
  `cubic_fit`, mas oposta por design).

---

## §2 — DECISÃO: renderer de path aberto (stroke) — território Coord

**Achado:** `ph2d_vector::draw_vector_network` é **fill-only** — itera regions,
pula `fill==None`, e **não faz stroke de segmentos**. Um path aberto do Pencil
(segmentos, sem region) **não renderiza nada** hoje. Isso toca o `ph2d-vector`
(API frozen, §3 do handoff impl) = sua decisão.

**Regra limpa que proponho (e já implementei interina no bridge):** *fazer
stroke de todo segmento cujo `style_ref` resolve a um `StrokeStyle` na
`StyleTable`.* Pencil põe `style_ref` nos segmentos; Pen não (a region carrega
o fill) → **zero double-draw**, a lista committed pode misturar Pen+Pencil.

- **Opção A (padrão-ouro, recomendada eventualmente):** você adiciona um
  stroke-pass ao `draw_vector_network` em `ph2d-vector` com essa regra. Aí
  **deleta** o loop interino `stroke_styled_segments` do
  `vector_pencil_bridge.rs`. Limpa, prepara W2 "Stroke+Fill básico" + W5 GPU
  stroke expansion.
- **Opção B (destrava o smoke Day-4 já):** mantém o stroke interino no bridge
  (já escrito, território meu) por W2; canonical pass vem depois. É uma deleção
  trivial migrar pra A.

Recomendo **B agora** (smoke desbloqueado sem mexer no frozen) + **A** como
follow-up de fechamento W2. Sua chamada.

---

## §3 — Fiação central necessária (Coord) — espelhe a wiring T1.7 do Pen

Os 3 arquivos novos já estão no meu território, prontos. Faltam os edits
centrais (compartilhados / frozen — por isso não toquei, per isolamento §0.2):

| # | Arquivo central | Edit (espelhar o Pen) |
|---|---|---|
| 1 | `shells/desktop/Cargo.toml` | add dep `ph2d-tool-vector-pencil` (mirror `ph2d-tool-vector-pen`) |
| 2 | `shells/desktop/src/render_loop/mod.rs` | `mod vector_pencil_bridge;` + chamar `vector_pencil_bridge::dispatch(...)` depois do pen bridge (mesma lista `committed_vector_pen_paths`); incluir `pencil_has_in_progress_stroke` no warn de destructive-deactivate; reconcile do pill state |
| 3 | `shells/desktop/src/input_dispatch.rs` | `mod vector_pencil_input;` + rotear Primary **Down**→`try_vector_pencil_pointer_down`, Move-pressed→`try_vector_pencil_pointer_drag`, **Up**→`try_vector_pencil_pointer_up` (o Pencil é **drag**, o Pen é click — precisa do stream de move/up no pointer FSM) |
| 4 | `shells/desktop/src/input_dispatch/keyboard.rs` | Esc → `try_vector_pencil_escape` (ao lado do pen) |
| 5 | `crates/ph2d-editor-core/src/ids.rs` | `pub const TOPBAR_VECTOR_PENCIL: WidgetId` (mirror `TOPBAR_VECTOR_PEN`) |
| 6 | `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs` | `mod vector_pencil_toggle;` + chamar `vector_pencil_toggle::apply(...)` na cadeia de dispatch |
| 7 | `crates/ph2d-editor-core/src/screens/hero/fixture.rs` | add `TopBarCluster::single("PENCIL", IconId::VectorPencil)` ao cluster `vector_tools` |

Arquivos meus prontos: `render_loop/vector_pencil_bridge.rs`,
`input_dispatch/vector_pencil_input.rs`,
`chrome/vector_pencil_toggle.rs`. (Não compilam standalone — dependem dos
símbolos centrais acima; é o fluxo invertido normal.)

**Lista committed compartilhada:** reusei `App::committed_vector_pen_paths`
(modelo de cena vetorial unificado Pen+Pencil). Rename opcional pra
`committed_vector_paths` é cleanup seu.

---

## §4 — W2 restante (não iniciado)

T2.1 entregou o **core do Pencil**. Faltam (plano §5): smoke Day-4 do Enio
(depende da fiação §3 + decisão §2), T2.2 Shapes, T2.3 Select/Direct,
T2.4 Color picker (gotcha §4.1 do handoff impl: `ph2d-painter-color::ClassicPicker`
não existe → confirmar widget), T2.5 Undo CRDT (crdt.rs stub → real), T2.6 Audit.

Também pendente de W1 (nota do Enio): carry-overs LOW §3.4, mini-round de
re-audit, smoke do Enio. T1.6 CRDT undo deferido p/ W2 (T2.5).

═══════════════════════════════════════════════════════════════════
RESPOSTA DO COORDENADOR · 2026-06-01 (commit `69b3788`)
═══════════════════════════════════════════════════════════════════

**§2 — DECIDI OPÇÃO A (padrão-ouro) E FIZ.** O `draw_vector_network` agora tem
stroke-pass canônico: stroke de todo segmento cujo `style_ref` resolve a um
`StrokeStyle`. NÃO é violação de contrato congelado — o próprio doc do W1 dizia
"strokes deferred to W2, lands with the Pencil tool". Pen region edges não têm
`style_ref` → pulados (zero double-draw Pen+Pencil). +4 testes em ph2d-vector.
→ **AÇÃO TUA:** **delete o interino `stroke_styled_segments` + a Layer (a)** do
teu `vector_pencil_bridge.rs` (linhas ~89-92 + a fn). Os committed paths agora
renderizam (fill+stroke) pelo `draw_vector_network` que o **pen bridge** já chama
por frame. Mantém só a Layer (b) (overlay in-progress) + o drain do
`take_committed_asset`. Até deletares = double-stroke benigno (idêntico, solid).

**§3 — FIADO PONTA-A-PONTA (`69b3788`). Shell compila.** Os 7 edits:
1. `Cargo.toml` dep ✓ · 2. `mod.rs` mod + dispatch call + warn destrutivo (o pill
reconcile já varre o cluster `vector_tools` genérico → pega a PENCIL sozinho) ·
3. `input_dispatch.rs` mod + roteamento de **DRAG** (Down→pointer_down,
CursorMoved→pointer_drag, Up→pointer_up, off-canvas consume) espelhando o
**Painter** (drag, não o click do Pen) · 4. `keyboard.rs` Esc · 5. `ids.rs`
`TOPBAR_VECTOR_PENCIL` · 6. `chrome/mod.rs` mod + apply · 7. `fixture.rs` pill PENCIL.
Verificado: shell `cargo check` ✓, ph2d-vector 5/5 stroke, editor-core 620+4,
clippy-clean ph2d-vector + shell.

**SMOKE Day-4 PRONTO p/ o Enio:** ativa o pill **PENCIL** no TopBar → arrasta no
canvas → traço freehand suavizado em cubics (Hobby). Esc cancela / limpa cena.

**⚠ Ship-blocker NÃO-teu, NÃO-meu:** o gate `arch_mode_has_reconcile` (editor-core,
cross-crate) está VERMELHO por `ph2d-tool-painter/src/layers.rs::set_blend_mode`
(commit `612cc34` T3.5 do **Painter impl**) — falta um reconcile/invalidate call
ou entry em `BENIGN_SET_MODE`. Flaguei pro Painter impl; não é do Vector.
═══════════════════════════════════════════════════════════════════
