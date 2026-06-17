# HANDOFF — Painter W2.T2.1 sidebar (Day-7 fechado pós-smoke)

**Data:** 2026-05-28
**Coord-A saindo:** Coord+Implementador da sessão T1.9 → T2.1.
**Próxima sessão:** **PRIMEIRO PASSO = AUDITORIA ADVERSARIAL ≥4 LENTES** (vide §3 abaixo). Sem auditoria, não avance.

---

## §0. Mandato §0 do plano (não-negociável)

[`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md) §0 — **padrão-ouro absoluto, sem gambiarras**. Toda task fecha com ≥2 auditorias paralelas → findings remediadas → re-audit erro-zero. Lente diversity per `feedback_audit_lens_diversity`: rotacionar canon S/T/U/V/W e variantes (sem reusar lentes da sessão anterior).

**Memory feedback que GOVERNA esta sessão:**
- [`feedback_perfection_no_deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md) — deferral aceitável proibido
- [`feedback_audit_lens_diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md) — rotacionar lentes
- [`feedback_destructive_reset_collision_2026_05_28`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_destructive_reset_collision_2026_05_28.md) — stage com `git add -- <paths>` após bloco foundational
- [`feedback_parallel_agent_collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_parallel_agent_collision.md) — `git commit -- <paths>` atômico, nunca `git add -A`

---

## §1. Estado entregue nesta sessão

### Commits locais Painter T1.9 → T2.1 (8 commits, todos verdes)

| Commit    | Escopo                                                                              | LOC          |
|-----------|-------------------------------------------------------------------------------------|--------------|
| `231d6cc` | T1.9 base — wire `StrokeHistory` + `StrokeJournal` em `PainterTool`                | +1096 / -11  |
| `a6acccd` | T1.9 audit 4-lente (S/T/U/V — 50 findings, 40 in-code + 10 W11 carry-overs)        | +755 / -24   |
| `1485471` | T1.9 fix-up — TILT import + `JournalError` Clone (-> bool return)                  | +9 / -7      |
| `1529794` | W2.T2.1 Day-3 skeleton — `ph2d-panel-painter-sidebar` drop-crate                   | +283 / -0    |
| `28b4a27` | W2.T2.1 Day-7 — functional sliders end-to-end (5 wire steps)                       | +349 / -80   |
| `689e39f` | T2.1 follow-up — register em `paint_hero_screen` z_order fallback                  | +1 / -0      |
| `4d71324` | T2.1 chrome canon + opacity stroke wire (4 bugs Enio smoke)                        | +85 / -13    |
| `c55a9c2` | T2.1 propagate `insp_rect` deltas pra `layout.painter_sidebar` (drag/resize/click) | +5 / -0      |
| `73e1413` | T2.1 `paint_panel_corner_dot_bl` no canto BL (visualizador resize)                 | +7 / -2      |

### Smoke Enio aprovou (parcial)

- ✓ Stroke aparece (T1.9 wire — W1 fechado em `project_painter_w1_complete_2026_05_28`)
- ✓ Sidebar visível em right-dock takeover
- ✓ Size slider muda brush size real-time (chip mostra px)
- ✓ Opacity slider muda alpha do stroke (commit `4d71324`)
- ✓ Drag handle no header
- ✓ Resize BR + BL
- ✓ Click no painel não vaza
- ✓ Corner dot BL pintado (commit `73e1413`)

### Pendente (T2.2..T2.7 — vide [plan §5](Painter_projeto/15_plano_de_implementacao.md#5-w2--vertical-mvp-sidebar--undo--classic-color))

- **T2.2** Undo/redo wire via `StrokeHistory::undo()` + replay engine OR snapshot per stroke. Sidebar buttons `PAINTER_SIDEBAR_UNDO_BUTTON` / `REDO_BUTTON` já registrados em store + populate; faltam paint + handler real (atualmente apply_ui_edit::Undo só pop da history, sem re-render canvas).
- **T2.3** Color picker wire — Enio confirmou "Já temos belo color picker" (BlenderColorPicker em editor-core). Wire = thumb na topbar Painter → popover BlenderColorPicker → primary color update.
- **T2.4** Modifier square — eyedropper-while-held (default ship W2; configurável em W10).
- **T2.5** Commit-to-sprite via `on_deactivate` — R3-LE-4 carry-over (request_commit() não wirado).
- **T2.6** A11y nodes para sliders + color thumb + modifier square (gate `hr12_widgets_a11y`).
- **T2.7** Smoke W2 + audit final.

---

## §2. Surface T2.1 disponível pro próximo agente

### `crates/ph2d-tool-painter::PainterTool` (novos pub methods)

- `ui_snapshot() -> PainterUiSnapshot` — projection read-only ADR-0043 §2.3
- `apply_ui_edit(PainterUiEdit)` — single source of truth pra Size/Opacity/SetColor/Undo/Redo/Reset/Toggles (cap 15 variants)
- `handle_panel_event(PanelEvent)` — wired pra `PAINTER_SIDEBAR_*` NodeIds

### `crates/ph2d-panel-painter-sidebar`

- `PainterSidebarPanel` typed Panel impl (ADR-0029 §4.3)
- `set_current_painter_snapshot(Option<PainterUiSnapshot>)` — shell publica per-frame
- 8 NodeIds canon em `editor_core::ids::PAINTER_SIDEBAR_*`
- Chrome completo: surface + corner dots (BR+BL) + title + close button + drag handle + 2 resize handles

### `crates/ph2d-editor-core`

- `screens::layout::HeroLayout.painter_sidebar: Rect` — slot
- `paint_hero_screen` orchestrator inclui `PAINTER_SIDEBAR_PANEL` no z_order fallback
- `insp_rect` propagado em hero.rs:625-655 inclui `layout.painter_sidebar = insp_rect`

### `shells/desktop`

- `Cargo.toml` feature `panel-painter-sidebar` no default
- `painter_bridge.rs` wire: visibility + snapshot publish + inspector hide edge-trigger

---

## §3. AUDITORIA OBRIGATÓRIA — 4 LENTES PARALELAS antes de qualquer T2.X

**INSTRUÇÃO ABSOLUTA AO PRÓXIMO AGENTE:** sua **PRIMEIRA AÇÃO** é lançar 4 audits adversariais em paralelo sobre commits `1529794`..`73e1413` (W2.T2.1 wire completo). Não comece T2.2/2.3/2.4/2.5/2.6 antes.

### Lentes rotacionadas (não reusar S/T/U/V de T1.9)

#### W — Widget canon adherence
Comparar `paint.rs` + `populate.rs` + `event.rs` do Painter sidebar contra padrão BgRemoval/Padding (DIRETRIZ v7.0 §5.2 widget canon). Procurar:
- Slider/chip layout mismatches (BgRemoval usa `paint_slider_with_chip_layout_adaptive` em narrow viewport — Painter usa `paint_slider_with_chip_layout` não-adaptive; possível regressão em iPad portrait)
- `register_slider_chip_pair` helper duplicado em vez de re-usar canon (cross-crate)
- Initial seeds em populate divergentes do snapshot defaults (split-brain inicialização — bug 2026-05-27 padrão)
- Faltam links `link_slider_number_mapped` quando display ≠ storage (size_px é mapped, eu uso identity-mapped; pode causar typing bug)
- HR-15 strings: "Painter" / "Size" / "Opacity" são pt-BR-OK English-permissible mas confirme

#### X — Shell integration coupling
Analisar `painter_bridge.rs` mudanças. Procurar:
- Edge-trigger Inspector hide pode resetar manualmente-escondido Inspector pelo rail (Wave 10 Etapa 4 pattern check)
- Snapshot publish dentro do downcast — se feature `panel-painter-sidebar` off, snapshot never published, panel renders defaults (intentional?)
- Borrow conflicts entre `tools.active_mut()` + `painter.ui_snapshot()` + `painter_preview` cache drain
- Visibility wire stomp/edge-trigger semantics

#### Y — State machine / persistence
- `apply_ui_edit::ResetSidebar` restaura defaults mas preserva history — testar se size/opacity preservam pós-cancel
- `ToggleSymmetry` cycle Vertical→None hardcoded sem persist (toggle round-trip stuck em Vertical)
- `Undo` pop da history sem re-render → canvas e history DESINCRONIZED até replay engine ship
- `eyedropper_armed` toggle não tem visual feedback no sidebar
- `cached_brush_hash` invalidation mid-session (set_brush trigger)

#### Z — Cross-platform / determinism / regression
- `paint_panel_close_button` register no fim do frame pode shadowar widgets do close se body cresce
- `set_panel_rect` publishing per-frame race com hit barrier order (paint_hero_screen registra rect ANTES de paint chama → first-frame skip)
- `panel_resize_anchor` é shared `INSP_PANEL` parent — Painter resize muta Inspector resize delta cross-tool
- HR-3 alloc hot path: `format!("{size_px:.0} px")` em paint = ~5 string allocs per frame; arch-gate `painter_no_alloc_hot_path` ainda verde?
- HR-5 cross-OS: f32 math em layout cálculo (size_px conversion 0..1 → 1..2048 + format) cross-OS bit-identical?
- T1.9 regression check: T2.1 wire interfere com begin/queue/end_stroke?

### Output esperado

Cada audit retorna findings classificados Sev (CRITICAL/HIGH/MEDIUM/LOW) com:
- Arquivo + linha
- Evidência (quote código)
- Por que é problema
- Fix sugerido

**Próximo passo pós-audit (mandato §0):**
- CRITICAL + HIGH + MEDIUM → in-code remediation (commit follow-up local)
- LOW → `docs/plans/2026-05-wave-11-carry-overs.md` §Painter T2.1 audit

Sem audit completo + remediação, T2.2 não abre. Mandato §0.

---

## §4. Ordem recomendada pós-audit

1. ✅ Audit W/X/Y/Z paralelo + remediation (commit `<hash>` Painter T2.1 audit)
2. → **T2.5** commit-to-sprite (R3-LE-4 carry-over T1.5 — pequeno, alto valor — `request_commit()` keybind Cmd+Enter + tool-switch trigger)
3. → **T2.3** color picker wire (BlenderColorPicker popover na topbar Painter; thumb topo direito vide plan §5)
4. → **T2.4** modifier square (eyedropper-while-held — central da sidebar)
5. → **T2.2** undo/redo replay engine (mais pesado — needs `replay_stroke_det` ship OR snapshot-per-stroke storage)
6. → **T2.6** A11y nodes (HR-12 obrigatório)
7. → **T2.7** smoke + audit final W2

---

## §5. Estado git + working tree

- **9 commits locais não pushados** (T1.9 + T2.1 chain).
- Working tree tem WIP de outros agentes (Sprite Inspector v2, Vector W1.T1.7 R3, asset-cooker, render):
  - `M Cargo.lock`, `M crates/ph2d-asset/`, `M crates/ph2d-editor-core/src/interaction/dispatch/{number_input,tick}.rs`, etc.
  - **NÃO TOQUE** esses paths (vide [`feedback_destructive_reset_collision_2026_05_28`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_destructive_reset_collision_2026_05_28.md)).
- Use SEMPRE `git commit -- <paths>` atômico após `git add -- <paths>`.

---

## §6. Quick start próximo agente

```bash
# 1. Confirmar HEAD
git log --oneline -3
# Esperado: 73e1413 c55a9c2 ... 4d71324 ...

# 2. Validar build limpo (background)
cargo check --workspace --tests &
cargo clippy -p ph2d-tool-painter -p ph2d-panel-painter-sidebar --tests --all-targets -- -D warnings &
wait

# 3. LANÇAR 4 AUDITS EM PARALELO — ver §3
# (não inventar lentes; usar W/X/Y/Z do brief)

# 4. Consolidar findings + remediar in-code
# 5. T2.5 → T2.3 → T2.4 → T2.2 → T2.6 → T2.7
```

---

**Boa sorte. Pad-rão-ouro.**
