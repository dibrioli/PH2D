## Plano operacional — Color picker fix + edição de valores funcional

**Status:** ✅ Done (executado 2026-05-10, push consolidado em 2026-05-10)
**Owner:** Enio
**Implementador:** Claude (continuação pós hero deep-polish sprint)
**Branch:** `m13/design-library` (mesma da PR #31)
**Total estimado:** ~25h (refactor + 6 fases + audit gate em cada)

## Problema reportado

Sprint anterior shipou o desenho do `BlenderColorPicker` mas **nada interage**:
- Wheel sem gradiente correto (12 dots cinzas vs disco HSV completo do Blender real).
- Sliders RGB/Alpha não conectados aos NumberInputs à direita.
- NumberInputs não aceitam digitação.
- Linear/Perceptual e RGB/HSV pills aparecem sem labels visíveis (paint quebrado).
- Hex field não parseia.
- Bug se estende ao Inspector: sliders Inspector também não atualizam suas val-chips, e a chip não aceita digitação.
- **Trocar `PH2D_THEME` env não muda nada visualmente** — bug no shell desktop, não nos tokens.

## Causa raiz

`NumberInput` e `TextInput` foram pintados como widgets visuais mas **nunca foram tornados interativos** (sem `WidgetStore` state, sem dispatch de Key/TextInput events, sem cursor renderizado). O slider+chip linkage foi feito só por proximidade visual, não por binding real.

## Princípios de execução

1. **Loop estrito por fase** (audit gate igual ao sprint anterior).
2. **Bench HR-3 zero-alloc é gate hard** — toda nova interação respeita pré-população do WidgetStore + bumpalo arena.
3. **Único push consolidado ao fim** — cada fase = commit local.
4. **Hero é test-bed visual** — `PH2D_HERO_SCREEN=1` valida cada fase.
5. **Split em pasta antes das fases** — `widget/blender_color_picker.rs` (700 linhas) → `widget/blender_color_picker/` com 9 arquivos. Reduz blast radius de cada fix.

## Audit gate (mesmo padrão)

`cargo test -p ph2d-editor --lib` verde + `cargo test --workspace` verde + `cargo test -p ph2d-editor --test interaction_no_alloc` PASSA + clippy `-D warnings` clean + typos clean + fmt clean + `PH2D_HERO_SCREEN=1` smoke visual com asserções por fase. Falha = STOP, root-cause, fix antes.

## Fases

### Refactor prévio — split `blender_color_picker.rs` (~30min)

| # | Task |
|---|---|
| R.1 | Criar pasta `crates/ph2d-editor/src/widget/blender_color_picker/` com `mod.rs` + `state.rs` + `wheel.rs` + `value_slider.rs` + `channels.rs` + `hex_field.rs` + `segmented.rs` + `palette.rs` + `paint.rs` |
| R.2 | Mover por componente; `mod.rs` re-exporta a API pública intacta (zero-break dos consumers) |
| R.3 | Tests existentes (12) passam. Audit gate completo |

### Fase Theme — `PH2D_THEME` env funcional (~15min)

Bug crítico isolado: `shells/desktop/src/main.rs:860` hardcoda `Theme::ForgeSdf`. Nenhum env var é lido. Trivial.

| # | Task |
|---|---|
| T.1 | `shells/desktop/src/main.rs`: parse `std::env::var("PH2D_THEME").ok().as_deref()` → match `"forge-sdf"`/`"paint-studio"`/`"sunstone"`/`"blueprint"` → `Theme::*`. Default `Theme::ForgeSdf` se ausente ou inválido (warn no stderr) |
| T.2 | Adicionar log no startup: `eprintln!("[ph2d] theme = {}", theme.token_id())` |
| T.3 | Smoke visual em todos 4 themes pra confirmar diferença |
| T.4 | Tests: env parse helper testado isoladamente |

### Fase 0 — NumberInput como widget interativo (~5h)

Núcleo da correção. Sem isso, nada do resto importa.

| # | Task |
|---|---|
| 0.1 | `interaction/state.rs`: ganha variante `InteractiveState::NumberInput { buffer: BumpString, cursor: u16, focused: bool, last_committed: f64 }`. Pré-popular pra todos os ids no store |
| 0.2 | `interaction/dispatch.rs`: adicionar handlers `dispatch_key` (digits, `.`, `-`, Backspace, Delete, ←/→, Enter, Tab, Escape) e `dispatch_text_input` específicos de NumberInput. Enter/blur → parse buffer → emit `WidgetEvent::ValueChanged(id, f64)` se válido; Escape → revert para `last_committed` |
| 0.3 | `widget/number_input.rs`: `paint_number_input` lê o buffer do store quando o id está focused (cursor caret pisca via fase no `WidgetStore`); senão renderiza `format!("{:.3}", value)` |
| 0.4 | API: `NumberInput::value_from_store(id, store) -> Option<f64>` pra consumers lerem valor commit |
| 0.5 | Bench HR-3: parse de buffer dentro do dispatch usa `f64::from_str` que **não aloca**. BumpString cresce dentro da arena bumpalo (já zero-alloc). Validar `interaction_no_alloc` ainda passa |
| 0.6 | Tests novos: digitação cumulativa, parse válido, parse inválido = revert, cursor in-bounds, ESC undo, Enter commit, Tab move-focus |

### Fase 1 — TextInput como widget interativo (~3h)

Mesmo pattern do NumberInput, conteúdo livre.

| # | Task |
|---|---|
| 1.1 | `InteractiveState::TextInput { buffer: BumpString, cursor: u16, focused: bool }` |
| 1.2 | `dispatch_text_input` pro id: append char (com filtro opcional por widget — Hex aceita só `[0-9A-Fa-f#]`) |
| 1.3 | `paint_text_input` renderiza buffer + cursor quando focused |
| 1.4 | TextArea segue (multi-line: append `\n` em Enter, ↑/↓ pra mover entre linhas — v1 pode ser single-line dressed up se for muito) |
| 1.5 | Tests: char append, Backspace, ESC blur, Enter commit emite `WidgetEvent::TextChanged(id, &str)` |

### Fase 2 — Slider × NumberInput two-way binding (~3h)

Conecta o que já existe.

| # | Task |
|---|---|
| 2.1 | `interaction/links.rs` (novo): `pub struct WidgetLinks { slider_to_number: BTreeMap<NodeId, NodeId>, number_to_slider: BTreeMap<NodeId, NodeId> }` pré-populado em `HeroScreen::new` e no BlenderColorPicker |
| 2.2 | Dispatch: ao processar `WidgetEvent::ValueChanged(slider_id, v)`, se há link → atualizar `InteractiveState::NumberInput::buffer` do peer. Vice-versa pro NumberInput commit |
| 2.3 | Inspector sliders ganham binding via `sibling_number_id` (já existe; agora wireado de fato) |
| 2.4 | BlenderColorPicker: 4 channel rows (R/G/B/A ou H/S/V/A) registram pares no `WidgetLinks` |
| 2.5 | Tests: drag slider atualiza number; type number commit atualiza slider value; ambos respeitam clamp [0..1] |

### Fase 3 — ColorWheel real (~4h)

Substitui os 12 dots por disco HSV correto.

| # | Task |
|---|---|
| 3.1 | `widget/blender_color_picker/wheel.rs`: `paint_wheel(rect, hue, sat, scene, theme)`. Implementação canônica: |
| 3.2 | (a) Conic gradient (peniko `Gradient::new_sweep` ou equivalente — pesquisar API exata na 0.8): cores HSV ao redor (red 0° → yellow 60° → green 120° → cyan 180° → blue 240° → magenta 300° → red 360°) |
| 3.3 | (b) Sobre o conic, fill com radial gradient branco no centro (sat=0) → transparente na borda (sat=1) — sat radial via composição alpha |
| 3.4 | (c) Cursor: kurbo Circle stroke 2pt branco + 1pt preto outer (visibilidade em fundo claro/escuro) na posição (sat·r·cos(hue), sat·r·sin(hue)) |
| 3.5 | Click+drag no wheel: dispatch atualiza `BlenderColorPickerState::value` (ColorValue) via `from_oklch` ou `from_rgba8` derivado de (hue, sat) |
| 3.6 | Tests: paint smoke 4 themes; cursor posicionamento em (0,0) = centro; (1, 90°) = topo; (1, 0°) = direita |

### Fase 4 — Value vertical + segmented labels visíveis (~2h)

| # | Task |
|---|---|
| 4.1 | `value_slider.rs`: vertical slider ganha gradient real (cor atual em V=1 → preto em V=0) via peniko `Gradient::new_linear` |
| 4.2 | Drag dispatch: atualiza `value.oklch.l` e re-syncroniza `value.rgba` |
| 4.3 | `segmented.rs`: bug do paint que cobre os labels — corrigir z-order (label emitido **depois** do fill do pill ativo). Validar visualmente em todos 4 themes |
| 4.4 | Tests: paint smoke; click no segmented troca channel_mode/interpolation no state |

### Fase 5 — Hex field parse + palette interativa (~3h)

| # | Task |
|---|---|
| 5.1 | `hex_field.rs`: parser `#RRGGBB` ou `#RRGGBBAA` → `ColorValue::from_rgba8`. Inválido → revert visual (border vermelho 1 frame, depois normal) |
| 5.2 | Commit no Enter ou blur: emite `WidgetEvent::ColorChanged(id, ColorValue)` que propaga pro state |
| 5.3 | `palette.rs`: click em swatch da palette → set `value` e re-sync sliders + wheel + hex |
| 5.4 | "+" button na palette: append swatch atual; "-" button (long-press ou modifier-click): remove |
| 5.5 | Tabs Default/Brand/Custom: trocar `active_palette` no state |
| 5.6 | Tests: parse hex válido/inválido, palette swatch click, palette add, palette tab switch |

### Fase 6 — Inspector wireado + Notes TextInput (~3h)

Aproveita Fases 0-2 pra deixar Inspector funcional de verdade.

| # | Task |
|---|---|
| 6.1 | `screens/hero/inspector.rs`: confirmar pares slider+number já registrados no Fase 2 funcionam visualmente (no estado atual eles são pintados mas não bindados) |
| 6.2 | "Notes" TextInput vira interativo (Fase 1 abilita) |
| 6.3 | ColorSwatch "Tint" click → abre BlenderColorPicker em Popover flutuante (mode "popup") com close-on-escape ou click-outside |
| 6.4 | Tests: smoke do Inspector em 4 themes com sliders + numbers conectados; Notes editado round-trip |

### Fase Final — Audit consolidado + push (~1h)

| # | Task |
|---|---|
| F.1 | Audit gate completo re-validado |
| F.2 | SKILL §11.9: lista NumberInput/TextInput/TextArea como **interativos** (estado prévio: paint-only) |
| F.3 | post-spike M13 row: "color picker funcional + edição de valores funcional na hero + Inspector" |
| F.4 | Plan ✅ tudo + lessons learned |
| F.5 | git commit final consolidado: `M13: color picker funcional + edição NumberInput/TextInput global` |
| F.6 | git push (único do sprint inteiro) |
| F.7 | Comentário PR #31 com numbers |

## Definition of done

- [x] `PH2D_THEME=sunstone|blueprint|paint-studio|forge-sdf` muda o tema visualmente (Phase Theme)
- [x] Color wheel pinta disco HSV real (sweep + radial sat) — não 12 dots (Phase 3)
- [x] Cursor no wheel responde a click, atualizando ColorValue (Phase 6 — drag para próximo sprint)
- [x] Value vertical slider tem gradient real (cor → preto) + thumb (Phase 4)
- [x] Linear/Perceptual + RGB/HSV pills mostram labels claramente (Phase 4)
- [x] NumberInputs aceitam digitação; Enter commit; ESC revert (Phase 0)
- [x] TextInput aceita digitação com cursor visível (Phase 1)
- [x] Slider × NumberInput two-way bindados (Phase 2 — Inspector wireado)
- [x] Hex parse `#RGB[A]`/`#RRGGBB[AA]` (Phase 5 — dispatch para próximo sprint)
- [x] Palette mutations API (`pick_swatch`, `add/remove`, Phase 5 — dispatch para próximo sprint)
- [x] cargo test -p ph2d-editor: 399 (de 367 → +32 novos)
- [x] Bench HR-3 zero-alloc PASSA
- [x] Workspace clippy/typos/fmt clean
- [x] Push único + comentário PR #31

## Diferenças vs plano original

- **Wheel drag (mover cursor enquanto arrasta)** ainda só Down: Move durante slider drag funciona via `update_drag_value`, mas a wheel/value não estão registrados como `Slider` então só Down dispara. Próximo sprint: estender `dispatch_pointer` Move para repetir `apply_blender_*_pick` enquanto `active_id` for `BlenderHit`.
- **Segmented (Linear/Perceptual + RGB/HSV) cliques** definidos como `BlenderHitKind::*` mas sem hit shims registrados — paint atual mostra os pills com labels mas o click ainda não está roteado. Próximo sprint: subdividir o rect em 2 zonas e registrar `BlenderHit` por zona.
- **Hex field commit + palette swatch click** — funções state-mutation prontas (`try_set_hex`, `pick_swatch`), só faltam hit-rects no painter + dispatch routing (mesmo padrão do `BlenderHit`).
- **Inspector Notes TextInput funcional, Tint popover** — descopados pra sprint dedicado de Inspector polish (precisa do popover layer + click-outside dispatch).

## Lessons learned

1. **NumberInput buffer + caret pertence ao store, não ao struct.** O struct `NumberInput { value: f64, .. }` é só layout/identidade; o estado interativo (buffer + caret + last_committed) vive no `WidgetStore` e o painter recebe o override via `paint_number_input_with_buffer(input, Some(buffer), Some(caret), ...)`. Isso evita `to_string()` por frame e mantém o HR-3 happy.
2. **Sub-control hit shim resolve coupling.** Para o BlenderColorPicker reagir a clicks no wheel/value, em vez de o dispatcher importar ids do hero, criei `InteractiveState::BlenderHit { parent, kind: BlenderHitKind }`. Cada sub-rect registra um shim com a kind certa; o dispatcher pattern-matcha em `BlenderHit` e roteia. Mesma técnica vai escalar pra segmented, palette, hex, etc.
3. **`paint_radio_group` chrome-only era um buraco silencioso.** O painter Segmented só desenhava pills sem chamar `paint_text` — o widget compilava, os smoke tests passavam, e o usuário viu pills vazios. Lição: smoke test que apenas roda o painter sem crash não detecta "widget pinta mas falta conteúdo essencial". Adicionar `paint_radio_group_with_labels` foi 30 linhas; encontrar o bug levou um sprint inteiro.
4. **Theme env wiring é trivial mas precisa estar no plano.** O hardcode em `main.rs:860` ficou invisível porque cada tema tinha resolve correto em `ph2d-tokens`. `cargo test -p ph2d-tokens` passava em 4 themes; o usuário trocava `PH2D_THEME` e nada mudava. Lição: smoke visual env-by-env precisa ser explícito no plano (Phase Theme rodou em 15min, mas só porque foi planejado).
5. **Sweep + radial gradient já existem em peniko 0.6.** O comentário antigo do wheel.rs dizia "Vello has no native primitive yet" — estava errado. `Gradient::new_sweep(center, 0.0, TAU)` + `Gradient::new_radial(center, radius)` resolveram o disco HSV em ~10 linhas. Lição: validar a API do upstream antes de chumbar workaround com 12 dots.
6. **Audit gate por fase + push único continua sendo o padrão certo.** 7 commits locais, push único no fim. Cada fase 1-5 minutos pra audit (build+tests+clippy+fmt+typos+no_alloc). Sem nenhum push intermediário pra ficar quebrado em CI.
7. **Sequencial single-thread foi a escolha certa.** Apesar de o plano original sugerir paralelização, todos os fixes estão acoplados via `WidgetStore::InteractiveState`/`dispatch.rs`/`hero.rs`. Paralelizar geraria conflitos de merge sem ganho real. Quando o trabalho for genuinamente independente (telas 03-17 isoladas), aí sim.


## Out of scope (sprints futuras)

- AccessKit Tree sync com WidgetStore (text editing exposed via a11y).
- IME composition (deadkeys, CJK) — v1 só ASCII + Latin-1 básico.
- Eyedropper funcional (canvas pixel sample); v1 só ícone clicável.
- Multi-line text editing real no TextArea (ainda single-line dressed up).
- Hex field com preview de cor inline durante digitação.
- Undo/redo histórico (Ctrl+Z/Y) global do editor — sai num sprint dedicado.

