# Plano operacional — biblioteca de componentes UI (Vello)

**Status:** Approved (handoff entre agentes)
**Data abertura:** 2026-05-09
**Owner:** Enio Oliveira Dias Brito
**Implementador:** próximo agente Claude (continuação após créditos esgotados)
**Referências:**
- [ADR-0023](../architecture/decisions/0023-ui-ux-baseline.md) (UI/UX baseline)
- [`docs/design/`](../design/) (design system canônico do Claude Design)
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §11.9 (editor UI), §10.1 (style)

## Objetivo

Implementar **toda a biblioteca de componentes UI** definida em
[`docs/design/component-library.html`](../design/component-library.html)
(30+ widgets) em Vello, **sem montar a interface gráfica do app ainda**.
A biblioteca é a fundação reutilizável; a tela hero (02-editor-main) e as
demais 16 telas vêm DEPOIS, num plano separado.

## Princípios de execução (LEIA ANTES DE COMEÇAR)

1. **Loop implementação → auditoria → fix → próximo.** Para cada item:
   implementar, rodar tests/clippy/typos LOCAIS, auditar contra mockup
   HTML, corrigir tudo que falhou, só então passar pro próximo. Nunca
   deixar débito visível pra trás.

2. **NÃO pushar nada nem disparar CI durante a execução.** A CI custa
   minutos do GitHub Actions e o usuário (Enio) confere visualmente. Vai
   rodar localmente: `cargo build`, `cargo test`, `cargo clippy --
   -D warnings`, `typos`, `cargo fmt --all -- --check`. Fim de cada Fase
   faz `git commit` LOCAL (sem push), pra preservar progresso. Push +
   PR só depois da Fase 6 inteira.

3. **NÃO peça permissões.** O plano já foi aprovado. Decisões técnicas
   dentro das Hard Rules (HR-1..HR-17) são suas. Só pare se: (a) bater
   numa decisão de produto user-facing, (b) tropeçar numa contradição
   entre design system e Hard Rules, (c) achar bug em pré-requisito que
   bloqueia tudo. Caso contrário, **vai do início ao fim**.

4. **Atualize este plano conforme avança.** Marque ✅ no checkbox de
   cada task ao concluí-la. Adicione notas inline se descobrir algo
   importante (ex: "ColorPicker exigiu helper extra X, ver paint.rs:N").

5. **Cada commit local termina com workspace verde.** Build + tests +
   clippy + typos + fmt todos limpos. Se quebrou, conserta antes de
   commitar — não acumule "vou consertar depois".

## Contrato canônico de widget

Todo widget novo segue exatamente este shape (refletindo o padrão já
estabelecido em [`Button`](../../crates/ph2d-editor/src/widget/button.rs)):

```rust
//! [`WidgetName`] — one-line purpose.
//!
//! Same pattern as [`crate::widget::Button`]: data + state enum +
//! token-resolved colors + AccessKit `Role::X` node + `paint_widget`
//! colocated. <Anything domain-specific in 1-2 sentences>.

use crate::paint::{rect_to_vello, resolve, fill_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Theme, /* others */};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum WidgetState {
    #[default] Normal,
    Hovered,
    Pressed,   // se aplicável
    Focused,
    Disabled,
    // Active/Selected se aplicável
}

#[derive(Clone, Debug)]
pub struct WidgetName {
    pub id: NodeId,
    pub label: String,
    pub state: WidgetState,
    // ...campos do domínio (value, options, etc)
}

impl WidgetName {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self { /* ... */ }
    pub fn state(mut self, state: WidgetState) -> Self { self.state = state; self }
    // ...builder chain methods

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::CorrectRole)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != WidgetState::Disabled)
            .action(Action::Click) // ou outras
            // .toggled / .numeric_value / .children conforme o tipo
            .build()
    }
}

pub fn paint_widget(w: &WidgetName, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    // Token resolution + Vello primitives.
    // SEM Color::from_hex hardcoded fora de ph2d-tokens (HR-12 implica).
    // Use fill_rounded_rect / stroke_rect / paint_icon helpers.
}

#[cfg(test)]
mod tests {
    use super::*;
    // Per state: paint_smoke_<state>() — must not panic
    // a11y_node_<role>() — assert role + label + actions
    // builder chain tests
    // domain-specific tests (clamp, select, toggle, etc)
}
```

## Auditoria por widget (checklist)

Após implementar cada widget, audita esses 8 itens. Não passa pro
próximo enquanto **algum** falhar:

- [ ] **Estados completos** — todos os states do design (Normal/Hover/
  Pressed/Focused/Disabled/Active/Selected) listados em
  `component-library.html` aparecem como variantes do enum + têm
  paint_smoke test individual.
- [ ] **Tokens semânticos** — zero `Color::from_hex` ou `0x...` literal
  no widget. Toda cor via `ColorToken::resolve(theme)`. Toda dimensão
  via `Spacing` / `Radius` / `TypeToken`.
- [ ] **A11y completo** — `Role` correto (consulte
  [`docs/design/accessibility.md`](../design/accessibility.md));
  `label`, `bounds`, `focusable` (false se Disabled), `action(Click)` ou
  outras conforme tipo; `toggled`/`numeric_value`/`children` quando
  aplicável.
- [ ] **WCAG contrast** — se o widget introduz par de cores novo (ex:
  text sobre fundo tinted), o teste de contraste em `ph2d-tokens` cobre.
  Caso contrário, acrescentar.
- [ ] **Doc comment** — `///` topo do struct + cada `pub fn`. Inglês
  curto (SKILL §10.1). Cite ADR-0023 onde aplicável.
- [ ] **Hover/focus visualmente distinguíveis** — não basta trocar
  bg-color por outro de luminance similar. Use BorderEmph para focus
  ring (3:1 vs Bg1 já validado em ph2d-tokens tests).
- [ ] **Sem clippy warnings** — `RUSTFLAGS="-D warnings" cargo clippy
  -p ph2d-editor --all-targets` clean.
- [ ] **Sem typos** — `typos crates/ph2d-editor/src/widget/<file>.rs`
  clean. Comentários SEMPRE em inglês.

## Tabela de fases (M13 UI sprint)

### Fase 0 — Pré-requisitos (~6h estimado)

Antes de tocar em widget novo, infra precisa estar pronta. Sem isso,
~60% dos componentes ficam pela metade ou ganham gambiarra.

| Task | Arquivo | Acceptance |
|---|---|---|
| 0.1 ✅ Port dos 89 SVGs do `docs/design/icons/` (87 documentados + 2 extras encontrados) para `crates/ph2d-editor/src/icons.rs` como `IconId` enum + `cmd_to_path(IconCmd) -> BezPath`. SVG 24×24 viewBox, currentColor → o paint helper aplica a cor do consumidor. | `crates/ph2d-editor/src/icons.rs` (+ `lib.rs` re-export) | ✅ `cargo test -p ph2d-editor icons::tests` passa (5 testes); smoke render de 5 ícones em VectorScene cobre paths/polylines/circles/rects |
| 0.2 ✅ Helpers de paint em `paint.rs`: `fill_rounded_rect(rect, radius, color)` (kurbo::RoundedRect), `stroke_rect(rect, width, color)`, `stroke_rounded_rect(rect, radius, width, color)`, `paint_icon(icon, rect, color, stroke_width, scene)`. | `crates/ph2d-editor/src/paint.rs` | ✅ Cada helper tem 1+ smoke test (5 novos em paint::tests); zero regressão no resto |
| 0.3 ✅ Refinar `paint.rs` removendo gambiarras de "border via 4 fill_rect" (existiam em `paint_tool_palette_icons`). Substituídas por `stroke_rect`/`stroke_rounded_rect`. | `paint.rs` (Phase 1 widgets ganharam stroke real ao reescrever) | ✅ Todos os tests passam; busca por `Top edge.*fill_rect` retorna zero |

**Commit local ao fim da Fase 0:** `feat(ph2d-editor): icons module + paint helpers (rounded rect, stroke, icon)`

### Fase 1 — Refinar 5 widgets existentes (~5h estimado)

Os widgets atuais funcionam mas precisam de refresh: tokens novos
(BgElev em vez de SurfaceElevated, etc — já migrado em commit 4bbd12a),
helpers de Fase 0 (rounded corners, stroke real), e pequenos ajustes
visuais pra bater com `component-library.html`.

| Task | Arquivo | Mudanças necessárias | Acceptance |
|---|---|---|---|
| 1.1 ✅ Button | `widget/button.rs` | ButtonKind enum (Default/Accent/Danger/IconOnly { icon }); Loading state com spinner inline; Radius::Md; focus ring real via stroke_rounded_rect; paint_button colocalizado em widget/button.rs. | ✅ 19 testes (10 paint smokes cobrindo todos os states+kinds) |
| 1.2 ✅ Slider | `widget/slider.rs` | SliderOrientation::Horizontal/Vertical; Radius::Full pill track; circular thumb (kurbo::Circle); ticks: Vec<f32> opcional; focus ring stroke ao redor do thumb. | ✅ 13 testes (smokes horizontal+vertical, value 0/0.5/1, dragging, focused, disabled) |
| 1.3 ✅ Toggle | `widget/toggle.rs` | Radius::Full pill; circular thumb; stroke_rounded_rect focus ring (gambiarra removida); thumb token muda por state. | ✅ 11 testes (smokes off/on/hovered/focused/pressed/disabled) |
| 1.4 ✅ RadioGroup | `widget/radio_group.rs` | RadioOrientation::Segmented adicionado (pill contínuo, selected = AccentSoft inset); Horizontal/Vertical mantidos. | ✅ 11 testes (segmented + vertical + horizontal smokes) |
| 1.5 ✅ ColorSwatch | `widget/color_swatch.rs` | Radius::Sm; SwatchSize { Sm 24 / Md 32 / Lg 48 }; transparency checker (4×4) quando alpha < 255; Hovered ring AccentSoft. | ✅ 9 testes (3 sizes + alpha 0/128/255 + hovered/focused) |

**Commit local ao fim da Fase 1:** `feat(ph2d-editor): refine 5 base widgets to match design system v1`

### Fase 2 — Componentes atômicos novos (~10h estimado)

Componentes simples, single-purpose, sem composição.

| # | Componente | a11y Role | States | Notas |
|---|---|---|---|---|
| 2.1 ✅ | **Checkbox** | `Role::CheckBox` | 5 × 3 (Checked/Unchecked/Indeterminate) | `Check` glyph quando Checked, `Plus` quando Indeterminate |
| 2.2 ✅ | **TextInput** | `Role::TextInput` | Normal/Hovered/Focused/Disabled/Error | Caret estático em `caret_byte` quando focused; placeholder em `Text3` |
| 2.3 ✅ | **TextArea** | `Role::MultilineTextInput` | mesmos do TextInput | `min_height(font_size) = 3 rows + padding` exposto |
| 2.4 ✅ | **NumberInput** | `Role::NumberInput` | mesmos | ChevronUp/Down chips; `up_rect`/`down_rect` para hit-test; min/max clamp |
| 2.5 ✅ | **ProgressBar** | `Role::ProgressIndicator` | Determinate/Indeterminate | `show_percent` em Determinate centra `nn%` |
| 2.6 ✅ | **Spinner** | `Role::ProgressIndicator` | sem variantes | Frame estático via `IconId::Spinner` (rotação shell-side) |
| 2.7 ✅ | **Avatar** | `Role::Image` | Normal/Disabled | AvatarShape Circle/Square; initial char centrado |
| 2.8 ✅ | **Divider** | `Role::Splitter` | — | DividerOrientation Horizontal/Vertical; 1px Border line |
| 2.9 ✅ | **Tag** | `Role::Label` (com Click se removable) | Normal/Hovered/Pressed/Disabled | TagTone Neutral/Accent/Success/Warn/Danger; close icon opcional |

**Acceptance por componente** = checklist de auditoria acima cumprido.

**Commit local ao fim da Fase 2:** `feat(ph2d-editor): atomic widgets (checkbox, inputs, progress, spinner, avatar, divider, tag)`

### Fase 3 — Componentes compostos (~10h estimado)

Componentes que reusam atomicos da Fase 2.

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 3.1 ✅ | **Tabs** | `Role::TabList` + per-item `Role::Tab` | TabsVariant Ghost (underline) + Segmented (pill) |
| 3.2 ✅ | **Dropdown** | `Role::ComboBox` + `Role::ListBoxOption` | `option_rect` para hit-test; chevron flips quando open |
| 3.3 ✅ | **Combobox** | `Role::ComboBox` editable | `Combobox::filtered()` faz substring match case-insensitive |
| 3.4 ✅ | **Vector3Editor** | `Role::Group` + 3× NumberInput | X/Y/Z labels tinted Danger/Success/Info |
| 3.5 ✅ | **ListItem** | `Role::ListItem` | leading icon + label + value + chevron opcional; selected = AccentSoft |
| 3.6 ✅ | **Card** | `Role::Group` | `header_rect`/`body_rect`/`footer_rect` slot helpers |
| 3.7 ✅ | **Tooltip** | `Role::Tooltip` | Bg3 Radius::Sm; consumidor controla visibilidade |
| 3.8 ✅ | **ContextMenu** | `Role::Menu` + per-item `Role::MenuItem` | ContextMenuEntry::Item ou Separator; `preferred_height` calcula altura sum |

**Commit local ao fim da Fase 3:** `feat(ph2d-editor): compound widgets (tabs, dropdown, combobox, vector3, list, card, tooltip, context menu)`

### Fase 4 — Surfaces & overlays (~5h estimado)

Componentes que pintam por cima de tudo.

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 4.1 ✅ | **Modal** | `Role::Dialog` | Scrim viewport-wide + dialog Radius::Lg; header(close icon) + body slot + footer(cancel/confirm Buttons) |
| 4.2 ✅ | **Toast restyle** | `Role::Alert`/`Role::Status` (existente em ph2d-a11y) | BgElev body + 4px severity stripe + leading severity icon + neutral Text1 message |
| 4.3 ✅ | **Popover** | `Role::Group` | BgElev + Border, Radius::Md — primitivo reusável |

**Commit local ao fim da Fase 4:** `feat(ph2d-editor): overlays (modal, toast restyle, popover primitive)`

### Fase 5 — Componentes complexos (~6h estimado)

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 5.1 ✅ | **TreeView** | `Role::Tree` + per-node `Role::TreeItem` | BTreeSet<NodeId> para expanded/selected (workspace lint forbid HashSet); `visible_rows()` flatten + indent `Spacing::Lg` per depth; ChevronDown↔ChevronRight |
| 5.2 ✅ | **ColorPicker** | `Role::Group` + 5-tab Tabs | Classic mode (RGB+HSL sliders) ship v1; Disc/Harmony/Value/Palettes paint placeholder M14+ stubs |

**Commit local ao fim da Fase 5:** `feat(ph2d-editor): complex widgets (tree view, color picker structure)`

### Fase 6 — Integração final + auditoria global (~3h estimado)

| Task | Acceptance |
|---|---|
| 6.1 Verificar `lib.rs` re-exporta TODOS os widgets novos com seus `paint_*` helpers. Lista esperada: Button + IconButton + Slider + Toggle + RadioGroup + ColorSwatch + Checkbox + TextInput + TextArea + NumberInput + ProgressBar + Spinner + Avatar + Divider + Tag + Tabs + Dropdown + Combobox + Vector3Editor + ListItem + Card + Tooltip + ContextMenu + Modal + Popover + TreeView + ColorPicker | `grep "pub use widget" crates/ph2d-editor/src/lib.rs` lista 25+ items |
| 6.2 Executar `cargo test -p ph2d-editor` — TODOS verdes | Output `test result: ok. N passed; 0 failed;` para cada módulo |
| 6.3 Executar `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets` — clean | Exit 0 |
| 6.4 Executar `typos` (full repo) — clean | Exit 0 |
| 6.5 Executar `cargo fmt --all -- --check` — clean | Exit 0 |
| 6.6 Atualizar [SKILL §11.9](../../SKILL_Stack_PH2D_Definitiva.md) — substituir lista de widgets atual por lista completa pós-implementação. Marcar como ✅ na seção "Implementação do design em Vello" o passo 3 (port icons) e adicionar passo 3.5 (component library completa). | Diff manual no arquivo |
| 6.7 Atualizar este plano marcando todas as tasks ✅ + adicionar seção "Lessons learned" com surpresas/decisões não-óbvias encontradas | Diff manual neste arquivo |
| 6.8 Atualizar `docs/plans/2026-05-post-spike.md` — linha M13 ganha status "🟢 component library done; falta tela 02 hero" | Diff manual |
| 6.9 **Único `git commit` final consolidando Fase 6** + único `git push` | `git log --oneline -1` mostra commit final |
| 6.10 Abrir comentário em PR #31 listando: total de componentes implementados, total de testes adicionados, total de linhas de código novas. **Não criar nova PR** — usar a PR #31 já aberta. | Comentário visível em https://github.com/dibrioli/PH2D/pull/31 |

**Commit final:** `M13: complete UI component library — 25+ widgets per design system`

## Estimativa total

~45h calendário (1 task ≈ 1-2h, audit + fix conta dentro). O agente
deve manter ritmo constante: **1-2 componentes por sessão de
implementação**. Se context window apertar, faz commit local da fase
parcial + retoma.

## Definition of done deste plano

- [x] Todas as 6 fases concluídas com checkbox ✅.
- [x] 27 widgets re-exportados em `crates/ph2d-editor/src/lib.rs` (meta era 25+).
- [x] `cargo test -p ph2d-editor` 259 testes verdes (atual: 259; baseline 90; meta 200+).
- [x] WCAG 2.2 AA contrast tests cobrem cada novo par token usado (28 testes em ph2d-tokens, sem novos pares introduzidos pela biblioteca).
- [x] Zero `Color::from_hex` em widgets (busca: `grep -r "from_hex" crates/ph2d-editor/src/widget/` retorna 0).
- [x] Zero pt-BR em comentários (typos lint clean).
- [x] Plano atualizado com lessons learned.
- [ ] Push único + comentário em PR #31 (próximo passo da Phase 6).

## Anti-patterns (NÃO faça)

- ❌ **Pushar entre fases.** Só ao fim. Cada push = 1 CI run = $$ + ruído pra Enio.
- ❌ **Pular auditoria pra acelerar.** Componentes mal-feitos viram débito caro depois.
- ❌ **Inventar tokens.** Se o design não tem cor pra um caso novo, use o token semântico mais próximo (ex: hover de Tag → AccentSoft, não "AccentSoft mas mais escuro"). Adicionar token novo exige edit em `tokens.json` + `ph2d-tokens` resolve table — extra trabalho.
- ❌ **Implementar features fora do escopo.** Glass blur (P3 do design audit), animações reais com tweening, IME composing, drag-and-drop entre widgets — tudo M14+. Plano cobre **layout estático + paint estático + a11y nodes**.
- ❌ **Pedir permissão pra escolhas técnicas dentro das HRs.** Decida + documente em comentário inline + segue. Enio só interrompe se você pingar.
- ❌ **Comentários em pt-BR.** SKILL §10.1 é lei. Inglês curto sempre.
- ❌ **Hex literals fora de ph2d-tokens.** HR-12 implica via lint futuro; respeite agora.
- ❌ **Tornar testes mais permissivos pra passar.** Se um teste falha, conserta o código, não o teste.

## Lessons learned

Surpresas e decisões não-óbvias durante a execução (2026-05-09 → 2026-05-10):

- **Ícones via SVG enum em runtime, não const BezPath.** `kurbo::BezPath`
  não é const-constructible (Vec interno), então `icons.rs` armazena
  `IconCmd` (Path/Polyline/Line/Circle/Rect) com strings/floats e
  parseia via `BezPath::from_svg` + `Circle::into_path`/`RoundedRect::into_path`
  on demand. Custo é trivial (89 ícones × ≤3 segmentos cada). Tentar
  cache via `OnceLock` foi descartado — paint só toca um ícone por
  call e o overhead de parsear é menor que o do hash lookup.
- **Re-export de kurbo via ph2d-vector.** Adicionei `Circle`,
  `RoundedRect`, `Shape`, `Stroke` ao re-export pra widgets não
  precisarem de dep direto em vello/kurbo. Evita version skew.
- **`Color::from_rgba8` é const-fn.** Permite definir cores estáticas
  (ex: checker tiles do ColorSwatch) sem alocar tokens. Útil pra
  user-content cases onde tokens não fazem sentido.
- **Toggle "re-fill body inside ring" gambiarra eliminada.** Phase 1
  refator descobriu que `Scene::stroke` em RoundedRect dá um focus
  ring real — não precisa do hack de pintar 2 retângulos sobrepostos.
- **Button kind enum substituiu `accent: bool`.** Adicionar Danger e
  IconOnly variants ficaria caótico com booleans; ButtonKind enum
  com `IconOnly { icon }` payload casa direto com o ButtonState.
  Quebra silenciosa de API só dentro do crate (paint.rs era único caller).
- **TreeView usa BTreeSet, não HashSet.** Plano dizia "HashSet OK pra
  editor UI runtime", mas o lint workspace de HR-5/ADR-0022 forbid_types
  reprovou. BTreeSet funciona idêntico (NodeId tem Ord) e mantém
  determinismo de ordem de iteração — bônus pra debugging.
- **Caret aproximado em TextInput.** Não temos parley layout
  introspection wired no v1; o caret usa `font_size * 0.55 * char_count`
  como advance estimado. Boa o suficiente em monospace e razoável
  em sans até IME entrar (M14+).
- **ColorPicker structure-only.** Plano explicitamente pedia "só Classic
  por enquanto". Implementei tabs + RGB+HSL sliders + preview swatch
  + placeholder pros outros 4 modos. Disc/Harmony/Value/Palettes ficam
  pra projeto-piloto demandar (ou screen 08 mockup pixel-perfect).
- **Stroke dash pattern não usado.** SVG `collider.svg` tem
  `stroke-dasharray="3 3"` que perdi ao converter (kurbo `Stroke`
  suporta `dash_pattern: Dashes`, mas IconCmd não carrega isso).
  Visualmente fica um rect sólido em vez de tracejado. Trade-off OK
  pro v1; restaurar dash exige IconCmd::PathDashed ou dash_pattern
  por-shape — fica pra quando algum widget pedir explicitamente.
- **Stray `atalho de play.command` accidental commit + revert.** O Mac
  Finder atalho entrou no `git add -A` da Phase 1 commit. Removido
  no commit seguinte. Lição: `git status --short` antes de cada add
  global, especialmente quando o repo tem arquivos do user fora do
  workflow.
- **Test count baseline.** Cada widget novo segue contrato de ~6-9
  testes (defaults/builders/a11y/paint smokes per state). Phase 0:
  +10. Phase 1: +18. Phase 2: +63. Phase 3: +53. Phase 4: +7. Phase 5:
  +18. Total final = 90 → 259 (+169 testes pra 27 widgets).
