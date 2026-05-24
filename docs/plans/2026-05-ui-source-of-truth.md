# Plano — UI single source of truth: Widget Gallery + Inspector + Hierarchy

**Status:** Aberto (drafted 2026-05-24). Fase 1 (ajustes) **ativa**; Fases 2-3 **deferidas** até Fase 1 fechar.
**Origem:** auditoria 2026-05-24 (DIRETRIZ v7.0 §5.2 expôs que "Widget Gallery é a fonte de verdade" não basta — há canon ambíguo com duas variantes de `number_chip`, e o widget que o Enio quer não existe).
**Decisão do Enio:** ajustar os 3 painéis **antes** de propagar pra docs/agentes, pra evitar reescrever docs duas vezes citando widgets que deixaram de existir.

---

## Norte

Os **3 painéis** abaixo passam a ser a fonte única de verdade da UI. Toda decisão de "como esse widget se comporta?" se resolve abrindo um deles:

| Painel | Crate | Cobre |
|---|---|---|
| **Widget Gallery** | [`ph2d-panel-widget-gallery`](../../crates/ph2d-panel-widget-gallery/) | **Componentes isolados** (todo widget primitive em sua forma canônica) |
| **Inspector** | [`ph2d-panel-inspector`](../../crates/ph2d-panel-inspector/) | **Componentes em contexto de painel docado** (composição: sliders+chips, dropdowns, vector editors, color pickers em painel real) |
| **Hierarchy** | [`ph2d-panel-hierarchy`](../../crates/ph2d-panel-hierarchy/) | **Componentes de navegação** (tree view, DnD reparent, multi-select, contextual menus) |

Todo painel novo (BgRemoval, Padding, Color Equalization, Upscale, futuros image-tools, painter, etc.) **copia literalmente** dos 3 — não inventa, não compacta, não simplifica.

---

## Fase 1 — Ajustes nos 3 painéis (ATIVA)

Sub-fases ainda a planejar com o Enio (cada uma vira sub-doc ou item aqui). A primeira já identificada na auditoria:

### 1.A — Unificar `number_chip` (decisão tomada 2026-05-24: canon ÚNICO)

**Decisão do Enio:** **um único chip numérico no projeto inteiro**, com arrows + click-drag h/v + step-on-click. Vale pra Gallery, Inspector, painéis de tool, BlenderColorPicker, qualquer site futuro. **Sem variantes "compactas"**, sem exceção.

**Origem** (vide análise 2026-05-24 que originou este plano): existiam 2 widgets concorrentes — `paint_number_input_with_buffer` (boxed, arrows, drag, step) vs `paint_number_chip` (pill, sem arrows, drag só). O docstring de [`slider_with_chip.rs`](../../crates/ph2d-editor-core/src/widget/slider_with_chip.rs) desautorizava o boxed pra slider+chip, então agentes copiavam o pill — chip sem arrows e sem afordância visual de step.

**Caminho de execução:**

1. **Estender [`paint_number_chip`](../../crates/ph2d-editor-core/src/widget/slider_with_chip.rs)** com arrows visíveis (ChevronUp/Down) + cálculo de `up_rect`/`down_rect` reutilizando a lógica de [`number_input.rs:117`](../../crates/ph2d-editor-core/src/widget/number_input.rs#L117) (`stepper_width = clamp(h*0.6, 16, 22)`).
2. **Atualizar `paint_slider_with_chip_layout`** — assinatura inalterada; o chip novo aparece automaticamente. `link_slider_number` DEIXA de auto-marcar `mark_chip_no_stepper` (porque a partir de agora chip TEM stepper).
3. **Atualizar `paint_blender_color_picker`** (canais R/G/B/A + hex) pra usar o chip novo. Se o tamanho atual dos chips de canal não comportar arrows, **alarga-se o layout** — universalidade tem prioridade sobre compacidade.
4. **`paint_number_input_with_buffer` (boxed)** deixa de existir como API distinta. Se o Inspector ainda precisa dele em algum site, vira `paint_number_chip(big_rect)` — ou se a diferença de chrome ainda fizer sentido visualmente, fica como wrapper documentado. Decidido durante a execução, com Enio aprovando o smoke.
5. **`mark_chip_no_stepper`** deixa de ser chamado em qualquer lugar; a API fica deprecada (mantida pra back-compat curta) e removida na próxima Wave.
6. **Inverter o gate** [`architecture_panel_chip_pill_no_stepper`](../../crates/ph2d-editor-core/tests/architecture_panel_chip_pill_no_stepper.rs) → `architecture_no_chip_without_steppers` (falha se algum painel recriar uma pill sem arrows).
7. **Smoke do Enio** em: Widget Gallery, Inspector, BgRemoval, Color Equalization, Padding, Upscale, Equalize Sizes, BlenderColorPicker — todos os sites de chip devem mostrar arrows + permitir drag + step-on-click.

**Quando 1.A fechar:** todo chip do projeto tem o mesmo visual + comportamento; arch-gate ativo previne regressão; seed da Gallery + Inspector mostra a forma única.

### 1.B — Outras divergências a identificar

A varrer painel a painel (BgRemoval, Color Equalization, Padding, Upscale, Equalize Sizes) e comparar contra Gallery/Inspector/Hierarchy. Lista a popular conforme cada uma aparecer no smoke do Enio ou no review.

Candidatos suspeitos (não verificados ainda):
- Dropdowns em painéis de tool: usam exatamente o mesmo padrão do Inspector?
- Botões Apply/Cancel: kind/state/spacing coerentes?
- Section headers nos painéis de tool: mesmo padrão do Inspector?
- Toggles e checkboxes: estados visuais idênticos?

---

## Fase 2 — Doc por componente (DEFERIDA — abre após Fase 1)

Quando Gallery + Inspector + Hierarchy estiverem nivelados, criar `docs/design/components/<componente>.md` (um por widget primitive), cada um descrevendo:

- Nome canônico + path do widget
- Onde aparece nos 3 painéis (link direto pro arquivo + range de linhas)
- Estados visuais (Normal/Hover/Pressed/Focused/Disabled)
- Comportamento de input (click, drag, keyboard, touch)
- Como compor com outros widgets (link + chip, dropdown + label, etc.)
- Tokens consumidos (`ColorToken::X`, `Spacing::Y`)
- A11y (Role, Action, numeric_value)
- Gates ativos que tocam esse componente

Lista provisória de componentes a documentar (≈20 docs):
button, checkbox, color_picker, color_swatch, combobox, divider, dropdown, icon_button, list_item, number_input, panel_chrome, pill_group, popover, progress_bar, radio_group, scrollbar, section_header, slider, slider_with_chip, tabs, tag, text_input, toggle, tree_view, vector3_editor.

---

## Fase 3 — Sweep de docs canônicos (DEFERIDA — abre após Fase 2)

Atualizar todas as referências para apontar **explicitamente** aos 3 painéis (não só ao Gallery) + linkar ao doc-por-componente da Fase 2. Lista de docs a atualizar (varredura de 2026-05-24):

**Vivos (a atualizar):**
- [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](../IntegracaoMultiAgente/DIRETRIZ.md) §5.2 + §5.2.1-5.2.4 + checklist 5.3
- [`CLAUDE.md`](../../CLAUDE.md) Design system
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §10.5 ("UI canônica = Widget Gallery")
- [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) §10.17 + §11.4 (single-Gallery refs)
- [`docs/Painter_projeto/13_referencias.md`](../Painter_projeto/13_referencias.md) §referencias
- [`docs/design/README.md`](../design/README.md) §"source of truth"
- [`crates/ph2d-tool-bgremoval/INTEGRATION.md`](../../crates/ph2d-tool-bgremoval/INTEGRATION.md) §2
- ADRs ativas que mencionem Gallery: [0028](../architecture/decisions/0028-wave-2-codegen-design-canonical.md) + [0029](../architecture/decisions/0029-trait-driven-panel-host.md) — só ADR amendment se mudança material.
- [`docs/IntegracaoMultiAgente/examples-fan-out.md`](../IntegracaoMultiAgente/examples-fan-out.md) — checklist do revisor de tool/panel.
- [`docs/Migracao/2026-05-wave-2-eliminating-all-collisions.md`](../Migracao/2026-05-wave-2-eliminating-all-collisions.md) — só se o conteúdo for canonical-vivo.

**Não tocar (arquivo / histórico):**
- Tudo em `docs/archive/`
- Bug docs (`docs/UI_Bugs/`, `docs/Image Tools Bugs/`) — apenas atualizar a regra estrutural; NÃO reescrever os fatos dos bugs.

---

## Critérios de fechamento por fase

| Fase | Fecha quando | Bloqueia |
|---|---|---|
| **1.A** | Widget canônico decidido + implementado + seed Gallery atualizado + smoke do Enio | 1.B, 2, 3 |
| **1.B** | Cada divergência identificada → resolvida ou opt-out documentado | 2 |
| **2** | Doc por componente publicado, link cruzado com Gallery/Inspector/Hierarchy | 3 |
| **3** | Varredura completa, todos os links ativos apontam pros 3 painéis + componente doc | — |

---

## Anti-padrões a evitar durante a execução

1. **Não toque nos docs antes da Fase 1 fechar** — o canon ainda está mudando; cada doc atualizado prematuramente vira débito de re-trabalho.
2. **Não crie doc-por-componente "preventivamente"** — escreva DEPOIS de ver o widget estável nos 3 painéis. Doc-de-design abstrato vira ficção rapidamente.
3. **Não unifique gratuitamente** — se Gallery e Inspector legitimamente mostram o mesmo widget de formas diferentes (ex: compacto vs expanded), documente o porquê, não force uniformidade visual.
4. **Não confunda "fonte de verdade" com "todo painel é igual"** — painel de tool tem chrome diferente (Apply/Cancel, preview-on-canvas, dock). O canon é dos COMPONENTES, não do layout.
