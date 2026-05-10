## Plano operacional — Editor input pipeline (ADR-0024 implementação)

**Status:** Approved (Enio aprovou ADR-0024 + plano em 2026-05-10)
**Owner:** Enio
**Implementador:** Claude (continuação pós-M13 hero)
**Branch:** `m13/design-library` (mesma da PR #31)
**ADR:** [0024-editor-input-and-widget-state.md](../architecture/decisions/0024-editor-input-and-widget-state.md)
**Total estimado:** ~17h (5 fases + audit-loop em cada uma)

## Princípios de execução

1. **Loop estrito por fase: implementar → auditar → corrigir tudo → próxima fase.** Audit é gate, não sugestão.
2. **Bench HR-3 zero-alloc é gate hard.** Falha em qualquer fase = STOP, root-cause, fix antes de qualquer próxima ação. Não acumular dívida de alocação.
3. **Único push consolidado ao fim de tudo.** Cada fase = commit local; push só na Fase Final.
4. **Tela hero é o test-bed visual.** `PH2D_HERO_SCREEN=1 cargo run -p ph2d-host-desktop` valida cada fase manualmente (Enio confere).
5. **Não pular audit pra "fechar fase mais rápido".** Lição da Phase 1 do M13 (stray Mac alias entrou por commit sem `git status` cuidadoso) — não repetir.

## Audit gate — checklist usado em CADA fase

Sem exceção, ao fim de cada fase, validar todos os 5 grupos. Falha em qualquer item dentro do gate **bloqueia avanço para a próxima fase**.

### Build & test gate
- [ ] `cargo build -p ph2d-editor` verde
- [ ] `cargo build -p ph2d-host-desktop` verde
- [ ] `cargo test -p ph2d-editor --lib` verde (≥ baseline da fase anterior)
- [ ] Testes novos da fase atual passam (round-trip de evento sintético — Down→Move→Up gera Click/ValueChanged correto)

### Quality gate
- [ ] `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets` clean
- [ ] `typos` clean (allowlist em `.typos.toml`)
- [ ] `cargo fmt --all -- --check` clean

### Performance gate (HR-3 + frame budget)
- [ ] Bench `interaction_dispatch_no_alloc` PASSA (0 allocs em 30-widget fixture × 10 frames de eventos)
- [ ] Benches existentes sem regressão > 5% (baseline em `target/criterion/`)

### Architectural gate
- [ ] HR-12: a11y `NodeId` no store espelha AccessKit `Tree` (mesmo id, sem duplicar identidade)
- [ ] HR-7: feature flag `editor` corta o `interaction` module em release-game build (`cargo build -p ph2d-host-desktop --no-default-features --features release-game` não inclui símbolos do interaction)
- [ ] HR-15: strings literais em código UI marcadas como dívida (i18n via Fluent não wired ainda; sem regredir)
- [ ] SKILL §10.1: comentários do código novo em inglês curto

### Visual gate (manual, Enio confere)
- [ ] `PH2D_HERO_SCREEN=1 cargo run -p ph2d-host-desktop` abre janela e renderiza hero
- [ ] Widgets shipados até essa fase respondem a hover/click/drag (visualmente perceptível)

## Fase 0 — Infra zero-alloc + bench HR-3 (~5h)

Foundation pass. Estabelecer infraestrutura que as fases A-D herdam. **Sem widget wire ainda** — só validar que a infra base honra HR-3 desde o primeiro frame.

| # | Task | Onde |
|---|---|---|
| 0.1 | Adicionar deps `slotmap`, `smallvec`, `bumpalo` em `crates/ph2d-editor/Cargo.toml` (verificar versões workspace; reusar se já existirem em outras crates) | `Cargo.toml` |
| 0.2 | Criar `crates/ph2d-editor/src/interaction/{mod,state,hit,dispatch}.rs` skeleton (estruturas vazias, sem lógica) | new files |
| 0.3 | `WidgetStore` com `SlotMap<NodeId, InteractiveState>` + métodos `register/get/get_mut/hot_id/active_id/focus_id`. Capacidade fixa ao construir. **Não expor `insert` público** — só `register` chamado na construção. | `state.rs` |
| 0.4 | `HitIndex` com `SmallVec<[(NodeId, Rect); 128]>` + `clear_for_frame/register/hit(x, y) -> Option<NodeId>` (back-to-front first-match) | `hit.rs` |
| 0.5 | `dispatch_pointer<'frame>(store, hit_index, event, &'frame Bump) -> &'frame [WidgetEvent]` — corpo apenas atualiza `hot_id` (resto stub `unimplemented!`); valida que assinatura compila e arena flui | `dispatch.rs` |
| 0.6 | Criar `tests/budget/no_alloc_hot_path.rs` (ou estender se existir) com caso `interaction_dispatch_no_alloc`: fixture de 30 widgets pré-registrados, 10 frames × 5 PointerEvents, validar 0 allocs via `dhat-rs` | `tests/budget/` |
| 0.7 | Re-export `interaction` module em `lib.rs` (`pub mod interaction; pub use interaction::{WidgetStore, HitIndex, WidgetEvent, dispatch_pointer, dispatch_key};`) | `lib.rs` |

### Audit Fase 0
- Audit gate completo (acima)
- **Específico:** bench HR-3 PASSA na primeira execução. Falha aqui = revisar arquitetura ANTES de seguir pra Fase A. Não tentar workaround.

### Commit local Fase 0
`feat(ph2d-editor): interaction module skeleton + HR-3 bench (Phase 0, ADR-0024)`

## Fase A — Button + Toggle wired (~3h)

Mais simples: hover/press/focus/click, sem drag.

| # | Task |
|---|---|
| A.1 | `dispatch_pointer` lógica completa: `Move` → atualiza `hot_id` via hit-test; `Down` em hit (focusable) → `active_id = Some(id)` + state→Pressed; `Up` dentro do mesmo `active_id` → emit `Click` + state→Hovered/Normal; `Up` fora → cancel (revert state, clear `active_id`) |
| A.2 | `dispatch_key`: Tab/Shift-Tab move `focus_id` na ordem de registro (não-focusable pulados); Enter/Space em `focus_id` simula Click (emit event direto) |
| A.3 | `Button` lê state via getter helper `store.button_state(id) -> ButtonState`; paint signature: `paint_button(&Button, rect, scene, text, theme, &WidgetStore)` (last param novo) |
| A.4 | `Toggle` idem; emite `WidgetEvent::Toggled(id)` em Click; estado `on` flips no store |
| A.5 | `HeroScreen` ganha campos `store: WidgetStore`, `hit_index: HitIndex`, `arena: Bump`. Pre-popula no `new()`: TopBar buttons + LeftRail tools + Hierarchy add button. Métodos `handle_pointer(PointerEvent)` e `handle_key(KeyEvent)` para a shell pumping |
| A.6 | Shell desktop: dentro do branch `PH2D_HERO_SCREEN=1`, fwd `on_pointer` → `hero.handle_pointer`, `on_key` → `hero.handle_key`. Shell mantém `Bump` próprio resetado por frame |
| A.7 | Testes novos em `interaction/tests.rs`: `button_click_round_trip` (Down→Up emite Click), `toggle_emits_toggled`, `tab_navigation_skips_disabled`, `enter_on_focused_button_clicks` |

### Audit Fase A
- Audit gate completo
- **Manual visual:** PH2D_HERO_SCREEN=1, hover sobre TopBar Save → cor muda; click → state Pressed visível; release → volta Normal

### Commit local
`feat(ph2d-editor): wire Button + Toggle to interaction store (Phase A)`

## Fase B — Slider/RadioGroup/Checkbox + drag (~4h)

Drag introduz `active_id` que persiste através de Move events.

| # | Task |
|---|---|
| B.1 | `Slider` horizontal: `Down` em rect → `active_id`; `Move` while active → calcula valor pela posição relativa, atualiza `store.slider_value(id)`, emit `ValueChanged(id)`; `Up` → release `active_id`, state→Hovered/Normal |
| B.2 | `Slider` vertical: idem, eixo Y |
| B.3 | `Slider` ticks: se `ticks` não-vazio, novo valor snaps ao tick mais próximo |
| B.4 | `RadioGroup`: per-option hit-test via `option_rect`; click muda `selected`, emit `ValueChanged` |
| B.5 | `RadioGroup` Segmented variant: idem (mesmo per-option hit-test, paint diferente) |
| B.6 | `Checkbox`: click cycle Unchecked↔Checked, emit `Toggled` (Indeterminate só programático, nunca via click) |
| B.7 | Hero wire: Inspector sliders (Move Speed/Jump Height/Friction/Damping/Cam Yaw) registrados no store; arrastar atualiza `display` text em tempo real |
| B.8 | Testes novos: `slider_drag_emits_value_changed_per_move`, `slider_release_clears_active_id`, `slider_ticks_snap_to_nearest`, `radio_click_changes_selection`, `checkbox_click_cycles_unchecked_checked` |

### Audit Fase B
- Audit gate completo
- **Manual visual:** arrastar slider Move Speed na hero → val display "160" muda em tempo real (live preview)

### Commit local
`feat(ph2d-editor): wire Slider drag + RadioGroup + Checkbox (Phase B)`

## Fase C — TextInput/NumberInput/Combobox + focus chain real (~5h)

Keyboard input mandatory aqui — character input + edit operations + focus traversal completo.

| # | Task |
|---|---|
| C.1 | Shell desktop: capturar character input (`winit::WindowEvent::KeyboardInput` text payload + IME commit) e fwd via novo `dispatch_text_input(store, ch, arena)` |
| C.2 | `TextInput`: `Down` dentro do rect → `focus_id = Some(id)` + caret position calculada por hit-test interno (font metrics via `parley`); character input → insert char no `store.text(id)` + caret advance; `Backspace` → delete prev char; Arrow Left/Right → move caret |
| C.3 | `TextInput` `Enter` → emit `TextChanged(id)` (caller relê `store.text(id)`); ESC → blur (clear `focus_id` se for este id) |
| C.4 | `NumberInput`: `up_rect`/`down_rect` click → increment/decrement; arrow up/down em `focus_id` → idem; `Enter` → emit `ValueChanged` |
| C.5 | `Combobox`: text input pra `query`; Down arrow abre + foca primeiro option; Enter seleciona; ESC fecha |
| C.6 | `Dropdown`: click chip → toggle `open`; quando `open`, hit-test entra nos `option_rect`; click option → seleciona, fecha; ESC fecha |
| C.7 | Hero wire: Inspector "Debug" select (Dropdown) torna interativo |
| C.8 | Testes novos: `text_input_char_insert_advances_caret`, `text_input_backspace_at_caret`, `number_input_increment_clamps_max`, `combobox_query_filters_options`, `dropdown_open_close_via_click`, `escape_blurs_focused_text_input` |

### Audit Fase C
- Audit gate completo
- **Manual visual:** Tab navega entre Inspector fields (Move Speed → Jump Height → Friction → ... → Debug); clicar em TextInput hipotético → caret aparece; digitar funciona

### Commit local
`feat(ph2d-editor): wire TextInput + NumberInput + Combobox + focus chain (Phase C)`

## Fase D — TreeView/ContextMenu/ColorPicker/Modal/Tabs + final wiring (~3h)

Composite widgets.

| # | Task |
|---|---|
| D.1 | `TreeView`: click chevron → toggle `expanded.insert/remove(node_id)`; click row → `selected.insert(node_id)` (clear se `!multi_select`) |
| D.2 | `ContextMenu`: click `MenuItem` → emit `Click(menu_item_id)`; ESC ou click fora → fecha (state externo) |
| D.3 | `Tabs`: click tab → `selected = idx`, emit `ValueChanged(tabs_id)` |
| D.4 | `ColorPicker`: tabs já é widget Tabs (D.3); Classic mode RGB sliders já são Sliders (B.1) — só precisa wire propagação `slider value → cp.rgba` quando pertencente ao mesmo `ColorPicker` |
| D.5 | `Modal`: ESC dismiss; click cancel button (já wired via Phase A) → emit `Click(cancel_id)`; idem confirm |
| D.6 | `Tooltip`/`Popover`/`Spinner`/`Avatar`/`Divider`/`StatusBar`: passive (sem input wire — consumer controla visibilidade/state) |
| D.7 | Hero wire: Hierarchy TreeView interativo (click row → muda `hero.selection` → Inspector title atualiza pra entity nova) |
| D.8 | Testes novos: `tree_chevron_click_toggles_expand`, `tree_row_click_selects`, `context_menu_item_click_emits_click`, `tabs_click_changes_selected`, `modal_escape_dismisses` |

### Audit Fase D
- Audit gate completo
- **Manual visual:** Hierarchy click em entity X → Inspector title muda; expand/collapse de tree node funciona; ColorPicker tab switch funciona

### Commit local
`feat(ph2d-editor): wire TreeView + ContextMenu + ColorPicker + Modal + Tabs (Phase D)`

## Fase Final — Audit consolidado + push (~2h)

| # | Task |
|---|---|
| F.1 | Audit gate completo re-validado (todas as 5 categorias × todos os widgets das 4 fases) |
| F.2 | SKILL §11.9: adicionar bullet "interaction module (HR-3 compliant via SlotMap+SmallVec+bumpalo)" na lista do editor |
| F.3 | SKILL "Implementação do design em Vello": adicionar passo "5. ✅ Input pipeline ADR-0024" |
| F.4 | `docs/plans/2026-05-post-spike.md`: M13 row mantém 🟢, atualizar nota: "31 widgets renderizam + 25 widgets interativos via WidgetStore" |
| F.5 | Atualizar este plano: ✅ em todas tasks + lessons learned |
| F.6 | Atualizar `docs/HANDOFF_M13_UI.md` (or HANDOFF.md) — flagar interaction como dependência pra próxima geração de telas |
| F.7 | git commit final: `M13: editor input pipeline (ADR-0024, 25 widgets interactive, HR-3 zero-alloc)` |
| F.8 | git push (único do sprint inteiro) |
| F.9 | Comentário em PR #31 com numbers: LOC, tests added (target +50), widgets wired (25), bench result (0 allocs/frame) |

## Definition of done

- [ ] `crates/ph2d-editor/src/interaction/` shipado com SlotMap+SmallVec+bumpalo
- [ ] Bench HR-3 `interaction_dispatch_no_alloc` PASSA com 0 allocs (verificado em CI)
- [ ] **25 widgets interativos:** Button, Toggle, Slider, RadioGroup, Checkbox, Tag (removable), Tabs, Dropdown, Combobox, ListItem, TreeView, ContextMenu, Modal, ColorPicker (tabs+sliders), TextInput, TextArea, NumberInput, Vector3Editor, Card (header click se title presente), PillGroup (children buttons), ToolRail (entries), SectionHeader (collapsible), StatusBar (passive — só visual feedback de hover opcional). Passive (sem input): Spinner, Avatar, Divider, Tooltip, Popover, ProgressBar, ColorSwatch, SwatchSize.
- [ ] Keyboard nav: Tab/Shift-Tab/Enter/Space/Esc + character input + Arrow keys funcionam
- [ ] Hero screen interativo via `PH2D_HERO_SCREEN=1`
- [ ] `cargo test -p ph2d-editor` ≥ 360 testes verdes (309 baseline + ~50 novos)
- [ ] Workspace clippy/typos/fmt clean
- [ ] Push único + comentário PR #31

## Anti-patterns (NÃO faça)

- ❌ **Pular audit pra "fechar fase mais rápido".** Audit é gate hard.
- ❌ **Aceitar warning clippy "vou consertar depois".** Conserta antes do commit local da fase.
- ❌ **Bench HR-3 falhar e seguir implementando.** STOP imediato. Root-cause + fix antes de mais nada.
- ❌ **Quebrar testes existentes (309 baseline) sem update conjunto.** Se refatoração precisa quebrar teste antigo, atualiza no mesmo commit com justificativa no commit message.
- ❌ **Pushar entre fases.** Único push na Fase Final. Cada fase = commit local.
- ❌ **Implementar input handling especulativo.** Hover/Press/Focus/Drag/Click são canônicos v1; multi-select Cmd-click, double-click, right-click context menu, multi-touch gestures, IME composing — fora de escopo, M14+.
- ❌ **Vazar interaction pra release-game build.** Verificar com `cargo build --no-default-features --features release-game` no audit final.
- ❌ **Comentários pt-BR em código** (SKILL §10.1).
- ❌ **`String` payload em WidgetEvent** (caller relê do store; ver ADR-0024 plano HR-3 mitigation 4).

## Lessons learned

_(preencho ao fim de tudo.)_
