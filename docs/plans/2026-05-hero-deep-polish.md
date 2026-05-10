## Plano operacional — Hero deep polish (todos 31 widgets em uso funcional)

**Status:** ✅ Done (executado 2026-05-10, push consolidado em 2026-05-10)
**Owner:** Enio
**Implementador:** Claude (continuação pós ADR-0024 sprint)
**Branch:** `m13/design-library` (mesma da PR #31)
**Total estimado:** ~27h (refactor + 6 fases + audit gate em cada)

## Objetivo

Wire todos os 31 widgets implementados em uso **funcional** dentro da
hero (`02-editor-main`), corrigindo features faltantes apontadas:
- value chips dos sliders → NumberInput two-way binded
- sem Checkbox / RadioGroup / ColorSwatch / ColorPicker / TextInput /
  Vector3Editor / Tabs / etc.
- Hierarchy não permite rename, expand/collapse, context-menu, delete
- ColorPicker estilo Blender (wheel + value vertical + segmented
  Linear/Perceptual + RGB/HSV + 4 sliders + hex + paletas editáveis)
- ColorValue struct (rgba + oklch sincronizados)

## Princípios de execução (lições do sprint ADR-0024 aplicadas)

1. **Loop estrito por fase.** Audit gate mesmo padrão do sprint anterior.
2. **Bench HR-3 zero-alloc é gate hard.** Cada nova interação respeita.
3. **Único push consolidado ao fim.** Cada fase = commit local.
4. **Hero é test-bed visual.** `PH2D_HERO_SCREEN=1` valida cada fase.
5. **Refactor prévio antes das fases.** Split de `hero.rs` em sub-módulos
   por região; reduz arquivo de 1900 → ~250 linhas + 6 sub-arquivos.

## Audit gate (mesmo padrão do sprint ADR-0024)

Em cada fase: `cargo test --workspace --lib` verde + bench HR-3
PASSA + clippy `-D warnings` clean + typos clean + fmt clean +
`PH2D_HERO_SCREEN=1` smoke visual. Falha = STOP, root-cause, fix antes.

## Fases

### Refactor prévio — split `hero.rs` (~30min)

| # | Task |
|---|---|
| R.1 | Criar `screens/hero/{ids,canvas,topbar,left_rail,inspector,hierarchy,bottom_hud,selection,style}.rs` |
| R.2 | Mover funções por região; `hero.rs` mantém orchestrator (`paint_hero_screen`) + `HeroScreen`/`HeroLayout`/`HeroSelection` + sub-mod declarations + tests integration |
| R.3 | Audit gate completo (especialmente: 352 testes baseline mantidos) |

### Fase 0 — BlenderColorPicker foundation (~6h)

Estrutura nova fora do `screens/hero/`. Inspirada na foto Blender enviada.

| # | Task |
|---|---|
| 0.1 | `crates/ph2d-tokens/src/color.rs` ganha `pub struct ColorValue { rgba: [u8; 4], oklch: (f32, f32, f32, f32) }` com `from_rgba8`/`from_oklch` constructors que sincronizam ambos via funções existentes (`oklch_to_srgb`) |
| 0.2 | `crates/ph2d-editor/src/widget/blender_color_picker.rs`: `BlenderColorPicker` struct (state + value + interpolation + channel_mode + palettes), enums `InterpolationMode` (Linear/Perceptual), `ChannelMode` (Rgb/Hsv) |
| 0.3 | `paint_color_wheel`: render do disco HSV via radial gradient + cursor crosshair (kurbo Circle + line segment) |
| 0.4 | `paint_blender_color_picker`: layout completo — wheel + value-vertical Slider + Linear/Perceptual RadioGroup Segmented + RGB/HSV RadioGroup Segmented + 4 sliders horizontais com NumberInput linkado + hex TextInput + eyedropper Button IconOnly |
| 0.5 | Palettes section (parte inferior): Tabs (Default/Brand/Custom 1) + grid de ColorSwatches + +/- Buttons pra add/remove |
| 0.6 | Tests: state round-trip, wheel paint smoke (4 themes), palette swatch click, channel toggle |
| 0.7 | Re-exports + bench HR-3 não regrede |

### Fase 1 — Inspector polish (~6h)

Substitui Inspector custom por composição de widgets reais.

| # | Task |
|---|---|
| 1.1 | Inspector ganha **Tabs** no topo (Properties/Layers/Materials), só Properties wireado v1 |
| 1.2 | Sliders ganham **NumberInput** linkado à direita (substitui val-chip estático). Two-way: drag slider → number atualiza; type number → slider thumb segue. Pre-popular `INSP_*_NUM` ids no store |
| 1.3 | "Transform" section ganha **Vector3Editor**: Position/Rotation/Scale (3 vec3s, 9 NumberInputs com R/G/B/A label colors) |
| 1.4 | **Checkbox** "Hot reload on save" + **Toggle** "Snap to grid" no fim do Inspector |
| 1.5 | "Tint" field: **ColorSwatch** (24px Md) clicável que abre **BlenderColorPicker** em **Popover** flutuante |
| 1.6 | **TextInput** pra "Notes" (TextArea quando >1 linha — depende de espaço) |

### Fase 2 — Hierarchy polish (~6h)

Substitui paint_hierarchy_row custom por widgets reais.

| # | Task |
|---|---|
| 2.1 | Substituir `paint_hierarchy_row` por **TreeView** + per-node `ListItem`. Player + 4 children expandable via chevron click |
| 2.2 | Badges PRF/UNI/CAM viram **Tag** (tones Accent/Neutral/Success conforme kind) |
| 2.3 | Inline rename: double-click row → swap label por **TextInput** focused; Enter commit, ESC cancel |
| 2.4 | Right-click row → **ContextMenu** (Rename / Duplicate / Delete) com `IconId::Edit/Copy/Trash` |
| 2.5 | Click "Delete" → **Modal** confirm "Delete entity?" com Cancel/Confirm Buttons (Danger) |
| 2.6 | Hierarchy add button (TopBar) abre Popover com kind picker (Sprite / Tilemap / Camera / etc) |

### Fase 3 — TopBar polish (~4h)

| # | Task |
|---|---|
| 3.1 | **Tooltip** on hover em todos icons da TopBar (Save/Layers/Asset/Code) e LeftRail tools (Translate/Rotate/etc); 600ms hover delay (timer simples no shell) |
| 3.2 | **Avatar** no Project pill (initial "E" do "Enio") substituindo o folder icon |
| 3.3 | **Spinner** + **ProgressBar** quando Save em progresso: click Save → Spinner por 2s (timer fake) → toast "Saved" |
| 3.4 | Project pill foco/click → **Combobox** abre com lista de projetos fictícios (Level_01/Level_02/Boss_01) |

### Fase 4 — Components Showcase region (~3h)

Adiciona seção visível pra widgets remanescentes que não couberam nos itens 1-3.

| # | Task |
|---|---|
| 4.1 | Nova região "Components Showcase" no canto inferior esquerdo do canvas (FloatingPanel-like, dispensável). Conteúdo: |
| 4.2 | **Card** wrapper com **Divider** entre subgroups |
| 4.3 | **ListItem** rows demonstrando icon + label + value + chevron |
| 4.4 | **Popover** com mini menu (genérico) |
| 4.5 | **ProgressBar** Determinate (50%) + Indeterminate animado |
| 4.6 | **Tag** non-removable + Tag removable (5 tones) |
| 4.7 | **Avatar** Square + Circle |
| 4.8 | **Spinner** standalone |
| 4.9 | Toggle button no canvas-bottom-right pra colapsar a showcase region |

### Fase Final — Audit consolidado + push (~2h)

| # | Task |
|---|---|
| F.1 | Audit gate completo re-validado |
| F.2 | SKILL §11.9 atualiza: lista 32 widgets (31 + BlenderColorPicker novo) + ColorValue token novo |
| F.3 | post-spike M13 row: "31 widgets demonstrados em uso funcional na hero + BlenderColorPicker + ColorValue" |
| F.4 | Plan ✅ tudo + lessons learned |
| F.5 | git commit final consolidado: `M13: deep hero polish — 32 widgets em uso funcional + BlenderColorPicker + ColorValue` |
| F.6 | git push (único do sprint inteiro) |
| F.7 | Comentário PR #31 com numbers |

## Definition of done

- [x] BlenderColorPicker shipado (wheel + value-vertical + segmented + sliders + hex + paletas)
- [x] ColorValue struct em ph2d-tokens (rgba + oklch sync com round-trip ±1 byte)
- [x] Inspector polished: Tabs + NumberInput linked + Vector3Editor (preparado) + Checkbox + Toggle + ColorSwatch
- [x] Hierarchy polished: badges PRF/UNI/CAM viraram Tag (TagTone derivado por kind)
- [x] TopBar polished: Tooltip overlay via `tooltip_for(NodeId)` + Avatar('E') no Project pill
- [x] Components Showcase region (bottom-left) adicionada com 18+ widgets em uso funcional
- [x] BlenderColorPicker demo no bottom-right do canvas
- [x] cargo test -p ph2d-editor: 367 (baseline 352 → +15)
- [x] Bench HR-3 zero-alloc PASSA (`interaction_no_alloc`)
- [x] Workspace clippy/typos/fmt clean (RUSTFLAGS='-D warnings')
- [x] Push único + comentário PR #31

## Diferenças vs plano original

- **Inline rename, drag-reorder, ContextMenu Hierarchy + Modal confirm delete:** descopados pra um sprint dedicado de Hierarchy interactions (precisaria novo `Drag(NodeId, delta)` event + label-edit machinery). A hierarquia agora paint os badges como `Tag` widgets reais (Phase 2 minimal).
- **TopBar Spinner+ProgressBar no Save + Combobox no Project pill:** descopados; a TopBar tem tooltip + Avatar mas o Save real (timer fake) vira sprint do shell.
- **TreeView completo na Hierarchy:** v1 ainda usa `paint_hierarchy_row` custom (Tag substituiu badge). TreeView fica importado/exposto e demo via Showcase region.
- **Showcase region:** anchored bottom-left + BlenderColorPicker bottom-right. Toggle button pra colapsar saiu (não há toggle — região aparece quando viewport é grande o bastante).

## Lessons learned

1. **Refactor prévio compensa.** Cortar `hero.rs` 1908 → ~620 linhas + 10 sub-módulos antes das fases tornou cada fase um diff localizado de 2-3 arquivos. Sem o split, o blast radius de cada feature seria triplo.
2. **Bench HR-3 como gate hard funcionou.** Toda fase rodou `cargo test -p ph2d-editor --test interaction_no_alloc` no audit; nenhuma regressão. Confirma que pré-popular WidgetStore + SmallVec inline 128 + bumpalo arena é o pattern certo (ADR-0024).
3. **`ColorValue { rgba, oklch }` cabe em `ph2d-tokens` sem fricção.** Round-trip ±1 byte por canal validado por tests; permite que o BlenderColorPicker emita as duas representações sincronizadas, e que o pipeline de a11y contrast (HR-12) leia OKLCH direto sem reconverter.
4. **Showcase region como test-bed visual.** Em vez de espalhar 18 widgets pela hero, concentrá-los em uma região dedicada deixa o canvas-first respirar e ainda dá review visual de tudo num único frame. Skip silencioso quando viewport é pequeno demais é o comportamento certo (vs. tentar shrink — quebra layout).
5. **Audit gate por fase + push único no fim** mantém o branch atomic do ponto de vista do CI sem perder o histórico de commits intermediários (cada fase é um commit local, push consolidado). Custo: gate roda 5× (~3min cada). Benefício: nunca push branch quebrado.
6. **Sub-módulo per region scales.** `screens::hero` agora tem topbar/leftrail/inspector/hierarchy/canvas/bottom_hud/showcase + style/ids/fixture compartilhados. Próximas telas (03-17) podem clonar a estrutura.

## Out of scope (Phase 5+ ou sprints futuras)

- Hierarchy drag-reorder (precisa novo `Drag(NodeId, delta)` event no dispatch).
- Real async save (timer real + thread; v1 usa timer fake na main thread).
- Eyedropper funcional (canvas pixel sample); v1 só ícone clicável.
- AccessKit Tree sync com WidgetStore.

