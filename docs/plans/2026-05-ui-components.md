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
| 0.1 ✅ Port dos 87 SVGs do `docs/design/icons/` para `crates/ph2d-editor/src/icons/` como módulo `pub mod icons { pub static ICON_NAME: BezPath = ... }` ou `IconId` enum + lookup. Use `kurbo::BezPath::from_svg("M...Z")` quando possível. SVG 24×24 viewBox, currentColor → o paint helper aplica a cor do consumidor. | `crates/ph2d-editor/src/icons.rs` (+ `lib.rs` re-export) | `cargo test -p ph2d-editor icons::tests` passa; pelo menos 1 smoke test que renderiza 5 ícones diferentes em VectorScene |
| 0.2 Helpers de paint em `paint.rs`: `fill_rounded_rect(rect, radius, color)` (kurbo::RoundedRect), `stroke_rect(rect, width, color)`, `stroke_rounded_rect(rect, radius, width, color)`, `paint_icon(icon: IconId, rect: Rect, color: Color, scene)`. | `crates/ph2d-editor/src/paint.rs` | Cada helper tem 1+ smoke test; widgets antigos refatorados pra usar (sem regressão de testes) |
| 0.3 Refinar `paint.rs` removendo gambiarras de "border via 4 fill_rect" (existem em `paint_tool_palette_icons` e em widgets de Fase 1). Substituir por `stroke_rect`/`stroke_rounded_rect`. | `paint.rs` + 5 widgets Fase 1 | Todos os tests existentes passam; busca por `// Top edge\n.*fill_rect` retorna zero resultados |

**Commit local ao fim da Fase 0:** `feat(ph2d-editor): icons module + paint helpers (rounded rect, stroke, icon)`

### Fase 1 — Refinar 5 widgets existentes (~5h estimado)

Os widgets atuais funcionam mas precisam de refresh: tokens novos
(BgElev em vez de SurfaceElevated, etc — já migrado em commit 4bbd12a),
helpers de Fase 0 (rounded corners, stroke real), e pequenos ajustes
visuais pra bater com `component-library.html`.

| Task | Arquivo | Mudanças necessárias | Acceptance |
|---|---|---|---|
| 1.1 Button | `widget/button.rs` | Adicionar variant `IconButton` (icon-only, square 36px). Adicionar variant `Danger` (background = ColorToken::Danger). Bordas arredondadas via `Radius::Md`. Spinner inline para `state = Loading` (novo state). | 8 paint_smoke tests (Normal/Hover/Pressed/Focused/Disabled + accent + danger + icon-only); contraste validado |
| 1.2 Slider | `widget/slider.rs` | Adicionar `orientation: SliderOrientation::Horizontal\|Vertical`. Track agora `Radius::Full` (pill). Thumb circular (kurbo::Circle ou ellipse approx). Tick marks opcionais para snap-to-grid. | Paint tests para horizontal+vertical, com value 0/0.5/1; thumb visualmente centrado |
| 1.3 Toggle | `widget/toggle.rs` | Pill com `Radius::Full`. Thumb circular. Animação de transição (não implementar tween real — apenas posição final correta para `on`/`off`). Remover hack de "re-fill body inside ring" (usar stroke_rect). | Paint tests on/off + focused/disabled; sem flicker visual |
| 1.4 RadioGroup | `widget/radio_group.rs` | Adicionar variant `Segmented` (todas as opções num row contínuo, selected = AccentSoft fill) + `Vertical` já existe. Borders arredondadas. Per-option a11y já correto. | Paint tests segmented/vertical com 3 opções, 1 selecionada |
| 1.5 ColorSwatch | `widget/color_swatch.rs` | Bordas arredondadas (`Radius::Sm`). Adicionar `size: SwatchSize::Sm\|Md\|Lg` (24/32/48 px). Indicador de "transparent checkerboard" quando `rgba.a < 255`. | Paint tests todos os tamanhos + alpha=0/128/255 |

**Commit local ao fim da Fase 1:** `feat(ph2d-editor): refine 5 base widgets to match design system v1`

### Fase 2 — Componentes atômicos novos (~10h estimado)

Componentes simples, single-purpose, sem composição.

| # | Componente | a11y Role | States | Notas |
|---|---|---|---|---|
| 2.1 | **Checkbox** | `Role::CheckBox` | Normal/Hover/Pressed/Focused/Disabled × Checked/Unchecked/Indeterminate (3-state) | Box `Radius::Xs`, checkmark via `paint_icon(IconId::Check)` |
| 2.2 | **TextInput** | `Role::TextInput` | Normal/Hover/Focused/Disabled/Error | Borda `BorderStrong`/Border conforme estado; cursor blink (não implementar — só desenhar cursor estático em pos `caret_pos`); placeholder em `Text3` |
| 2.3 | **TextArea** | `Role::MultilineTextInput` | mesmos do TextInput | Idem TextInput, mas `min_height: 3 * row_h` |
| 2.4 | **NumberInput** | `Role::NumberInput` | Normal/Hover/Focused/Disabled | TextInput + 2 botõezinhos `▲`/`▼` à direita; clamp via `min`/`max` opcionais |
| 2.5 | **ProgressBar** | `Role::ProgressIndicator` | Determinate (`value: 0..=1`) e Indeterminate | Track `Radius::Full` `Bg2`; fill `Accent`; texto `value%` opcional centrado |
| 2.6 | **Spinner** | `Role::ProgressIndicator` | Sempre rotaciona (mas pintamos o frame estático — animação real é shell-side) | Arc 270° via `kurbo::Arc`; cor `Accent` |
| 2.7 | **Avatar** | `Role::Image` | Normal/Disabled | Square ou circle (`Radius::Full`); placeholder com inicial centrada se não tiver imagem |
| 2.8 | **Divider** | `Role::Splitter` (horizontal/vertical) | — (sem state) | 1px line `Border`; orientation enum |
| 2.9 | **Tag** / **Chip** | `Role::Label` (com `action(Click)` se removable) | Normal/Hover/Pressed | Pill `Radius::Full` `Bg2`/`AccentSoft`; X opcional removível |

**Acceptance por componente** = checklist de auditoria acima cumprido.

**Commit local ao fim da Fase 2:** `feat(ph2d-editor): atomic widgets (checkbox, inputs, progress, spinner, avatar, divider, tag)`

### Fase 3 — Componentes compostos (~10h estimado)

Componentes que reusam atomicos da Fase 2.

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 3.1 | **Tabs** | `Role::TabList` (parent) + `Role::Tab` per option | Horizontal row; selected = underline `Accent` `BorderEmph` 2px; ghost por default; pode ser segmented variant |
| 3.2 | **Dropdown** / **Select** | `Role::ComboBox` (closed) → `Role::ListBox` quando aberto | Closed = TextInput-like com chevron; aberto = popover com lista de `Role::Option`. Posicionamento `Layer::Overlay` |
| 3.3 | **Combobox** | `Role::ComboBox` com `editable: true` | TextInput + Dropdown filtrável; sugestões aparecem conforme typing |
| 3.4 | **Vector3Editor** | `Role::Group` (parent) + 3× `Role::NumberInput` | 3 NumberInput lado a lado com labels X/Y/Z (cores opcionais danger/success/info) |
| 3.5 | **ListItem** | `Role::ListItem` | Row com `icon? + label + value? + chevron?`; selected = `AccentSoft` fill; `Density` aware (compact/cozy/comfortable) |
| 3.6 | **Card** | `Role::Group` | Surface `Bg2` `Radius::Lg` com header opcional, body, footer; padding `Spacing::Lg` |
| 3.7 | **Tooltip** | `Role::Tooltip` | Pequeno popover acima do hovered widget; `Bg3` `Radius::Sm`; aparece só quando o consumidor pede (sem hover state interno) |
| 3.8 | **ContextMenu** | `Role::Menu` + `Role::MenuItem` per item | Vertical list de ListItems com keyboard shortcuts; aparece em `Layer::Overlay`; suporta separators (Divider) |

**Commit local ao fim da Fase 3:** `feat(ph2d-editor): compound widgets (tabs, dropdown, combobox, vector3, list, card, tooltip, context menu)`

### Fase 4 — Surfaces & overlays (~5h estimado)

Componentes que pintam por cima de tudo.

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 4.1 | **Modal** | `Role::Dialog` | Centered card sobre scrim (`BgScrim`); ESC dismiss; focus trap (não implementar trap real, só marcar a11y); header + body + 2 buttons (cancel/confirm) |
| 4.2 | **Toast styled** (refinar existente) | `Role::Alert` (assertive) ou `Role::Status` (polite) per severity | Restyle do `ToastQueue::paint`: usar `fill_rounded_rect` com `Radius::Md`, severity icon à esquerda (`paint_icon(IconId::Info/Check/Warn/Error)`), shadow `Shadow::Md` |
| 4.3 | **Popover** | `Role::Group` (genérico) | Container reutilizável usado por Dropdown/Tooltip/ContextMenu; `BgElev` `Radius::Md` shadow `Shadow::Lg` |

**Commit local ao fim da Fase 4:** `feat(ph2d-editor): overlays (modal, toast restyle, popover primitive)`

### Fase 5 — Componentes complexos (~6h estimado)

| # | Componente | a11y Role | Notas |
|---|---|---|---|
| 5.1 | **TreeView** | `Role::Tree` + `Role::TreeItem` per node | Indentação por depth × `Spacing::Lg`; expand chevron via `paint_icon(IconId::ChevronRight)` rotacionado quando expandido; selected highlight `AccentSoft`; suporta multi-select (via `selected: HashSet<NodeId>` — mas use `BTreeSet` per HR-5 / ADR-0022 se for em pipeline determinístico; para editor UI runtime, HashSet OK pois não é simulation) |
| 5.2 | **ColorPicker** | `Role::Group` (parent) + `Role::TabList` | 5 abas (Disc / Classic / Harmony / Value / Palettes) — implementar SÓ a estrutura de tabs + 1 modo (Classic com sliders RGB+HSL) por enquanto. Demais modos ficam stub para M14+. Vide screen 08-color-picker.html para referência visual |

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

- [ ] Todas as 6 fases concluídas com checkbox ✅.
- [ ] 25+ widgets re-exportados em `crates/ph2d-editor/src/lib.rs`.
- [ ] `cargo test -p ph2d-editor` ≥ 200 testes verdes (atual: 90; meta:
  90 + 8/widget × 25 widgets = 290).
- [ ] WCAG 2.2 AA contrast tests cobrem cada novo par token usado.
- [ ] Zero `Color::from_hex` em widgets (busca: `grep -r "from_hex" crates/ph2d-editor/src/widget/` retorna 0).
- [ ] Zero pt-BR em comentários (typos lint clean nos novos arquivos).
- [ ] Plano atualizado com lessons learned.
- [ ] Push único + comentário em PR #31.

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

_(Agente preenche aqui ao fim. Surpresas, decisões não-óbvias,
descobertas que valem documentar pra quem for tocar nisso depois.)_
