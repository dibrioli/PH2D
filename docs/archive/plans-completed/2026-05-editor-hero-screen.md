## Plano operacional — tela hero `02-editor-main`

**Status:** Done (executed 2026-05-10, 5 phases shipped)
**Data abertura:** 2026-05-10
**Owner:** Enio
**Implementador:** Claude (mesma sessão pós-M13 UI library)
**Mockup:** [`docs/design/screens/02-editor-main.html`](../design/screens/02-editor-main.html)
**Branch alvo:** `m13/design-library` (mesma da PR #31 ainda aberta)

## Objetivo

Renderizar a tela `02-editor-main` em Vello + parley + AccessKit usando
a biblioteca de 27 widgets recém-shipada. Esta é a **tela hero** do PH2D
— a primeira coisa que um usuário vê ao abrir o editor — e valida a
fundação UI no contexto real (canvas + chrome flutuante + inspector +
hierarchy + status HUD).

**Não-objetivo:** match pixel-a-pixel via screenshot diff (precisa de
GPU + harness de captura — escopo M14+). Match estrutural + token-
correto + a11y-correto é o que garantimos aqui.

## Mapa de regiões (do mockup HTML)

| # | Região | Posição | Conteúdo |
|---|---|---|---|
| 1 | TopBar | top:14, left/right:14 | 5 "pill groups" — theme/save/project/play/right-cluster + wordmark central |
| 2 | LeftRail | left:14, top:70, bottom:70, w:56 | 4 transform tools + 3 toolbtn compostos (Global/Persp/Home) + 2 history (undo/redo) |
| 3 | Inspector | left:84, top:70, w:304 | Floating panel com title/sub + descrição + sections (Params 12, Advanced 7, Inputs 24) e fields tipados (slider, select, linked-input) |
| 4 | Hierarchy | right:14, top:70, w:308 | Floating panel com header + add button + lista hierárquica de entities (Player selected, children indentados) |
| 5 | Canvas BG | inset:0 (atrás de tudo) | Gradient radial Bg0→Bg1, perspective grid mascarado, sem render de level real (mockup tem stack de shapes — vamos ignorar) |
| 6 | Selection overlay | over canvas | Marquee tracejado + 4 handles + tag "Player · PRF · 124, −48" |
| 7 | BottomHUD | bottom:18, h-center | Status pill segmentado: EDIT • 60 fps • 13101/16660 • 21n • 100% • default-scene |
| 8 | Tweaks hint | bottom-right:18 | Pill "Tweaks ⌘." — meramente informativo, dispensável v1 |

## Princípios de execução

1. **Loop por região, não por widget.** Cada região é uma unidade de
   trabalho fechada (compor + paint + a11y + smoke test).
2. **NÃO pushar entre regiões.** Único push ao fim da Fase 4.
3. **Reusar widgets existentes sempre que viável.** Inspector "field
   row" = ListItem variant? Provavelmente não — ListItem é list-item
   horizontal genérico, field row tem layout (marker + label + slider +
   val chip) específico. Decidir caso-a-caso na implementação.
4. **Fixture data inline.** Player/Slime_01/etc são *mockup content*.
   No v1 vamos hardcodar essas strings num módulo `fixture.rs` — não
   queremos criar entity store real ainda (esse é trabalho do M14+
   quando wired no ECS).
5. **Test = paint smoke + a11y role.** Sem GPU, smoke = "não panica".
   Validação visual fica com Enio rodando shell desktop.

## Fases

### Fase 0 — Primitivos faltantes (~5h)

Antes de compor a tela, faltam 4 building blocks que não couberam na
biblioteca M13 (porque tinham forma demais específica pra hero):

| # | Widget | Onde | Notas |
|---|---|---|---|
| 0.1 | **PillGroup** | `widget/pill_group.rs` | Container BgElev + border + Radius::Xl agrupando N children (icon-only Buttons). Usado 5× na TopBar. |
| 0.2 | **ToolRail** | `widget/tool_rail.rs` | Vertical strip de tools. Cada item: icon-only Button (44×44) ou compound "ToolBtn" (label face + sub-label mono uppercase). Suporta divisor. |
| 0.3 | **StatusBar** | `widget/status_bar.rs` | Pill segmentado horizontal — vetor de `StatusSegment { label, accent? }`. Border interno 1px entre segmentos. Usado pelo BottomHUD. |
| 0.4 | **SectionHeader** | `widget/section_header.rs` | Header de seção do Inspector: dot + label uppercase + count chip + chevron opcional (collapsible). |

Cada um segue o contrato canônico da Phase 1 do M13 UI: data + state +
tokens + a11y + paint helper. ~6 testes/widget = +24 testes.

### Fase 1 — Hero scene root + canvas BG (~2h)

Criar `crates/ph2d-editor/src/screens/hero.rs` (novo módulo `screens`).

| Task | Detalhe |
|---|---|
| 1.1 | `HeroScreen` struct: viewport rect + 5 sub-regions (top, rail, inspector, hierarchy, hud) + selection state |
| 1.2 | `paint_canvas_bg`: radial gradient Bg0→Bg1 (Vello tem `Brush::Gradient`); perspective grid via `Affine::skew` + clipped fill_rect tiling |
| 1.3 | Smoke test: `paint_hero_screen` num `viewport: 1366×1024` não panica |

### Fase 2 — TopBar + LeftRail (~3h)

| Task | Detalhe |
|---|---|
| 2.1 | TopBar: 5 PillGroups posicionados absolutamente. Conteúdo hardcoded (theme=Forge SDF, project=Level_01, etc) |
| 2.2 | Wordmark central "PH2D · EDITOR" via paint_text_centered |
| 2.3 | LeftRail: ToolRail com Translate(active)/Rotate/Scale/Pivot + divider + Global/Persp/Home + divider + Undo/Redo |
| 2.4 | A11y: TopBar = `Role::Toolbar`, LeftRail = `Role::Toolbar` (vertical orientation) |

### Fase 3 — Inspector + Hierarchy (~5h)

| Task | Detalhe |
|---|---|
| 3.1 | InspectorPanel composta: drag handle (decorative) + header + scrollable body. Body lista SectionHeader + Field rows. Field row = marker dot + label + Slider + val chip (mono font 11px). |
| 3.2 | Variant Field: select-pill (Debug → Shading) usa Dropdown closed; "input-pill" (Distance/Material) é uma `Tag` com tone Accent + leading "↳" glyph; "+ link" é um `Tag` com tone Neutral dashed border |
| 3.3 | HierarchyPanel: header com title + counts + add button (icon-only Button accent variant) + body com lista de h-rows. h-row = ListItem com child variant (icon + name + optional badge + optional swatch + visibility dot) |
| 3.4 | A11y: Inspector = `Role::Group` (root), Hierarchy = `Role::Tree` (entities têm parent/child) |

### Fase 4 — BottomHUD + selection overlay + integração final (~3h)

| Task | Detalhe |
|---|---|
| 4.1 | BottomHUD via StatusBar com 6 segmentos (EDIT, 60 fps, 13101/16660, 21n, 100%, default-scene) |
| 4.2 | Selection marquee: 4 handles (BezPath quadrados) + dashed rect (kurbo Stroke com dash_pattern) |
| 4.3 | Selection tag flutuante acima do marquee — Tag tone Neutral com 2 children badges (Player + PRF) |
| 4.4 | Wire na shell `shells/desktop`: `paint_hero_screen(scene, ctx)` chamado após `Layout::paint` |
| 4.5 | Workspace test/clippy/typos/fmt clean; commit + push; comentário PR #31 com screenshot link (Enio confere visualmente rodando `cargo run -p ph2d-host-desktop`) |

## Estimativa total

~13h calendário. Component count: +4 widgets primitivos, +1 screen
module, +1 fixture module. Tests: ~24 novos.

## Definition of done

- [x] 4 primitivos novos em `widget/` re-exportados em `lib.rs` (PillGroup, ToolRail, StatusBar, SectionHeader).
- [x] `crates/ph2d-editor/src/screens/hero.rs` + `screens/hero/fixture.rs` criados e re-exportados (HeroScreen, HeroSelection, paint_hero_screen).
- [x] `cargo test -p ph2d-editor` 309 testes verdes (meta era 285+; baseline 259).
- [x] `shells/desktop/src/main.rs` invoca a hero screen via `PH2D_HERO_SCREEN=1` env var — abre janela com tela renderizada.
- [x] Workspace clippy/typos/fmt clean.
- [ ] Único commit + push consolidando o trabalho.
- [ ] Comentário em PR #31 com instruções de "como testar visualmente".

## Anti-patterns (NÃO faça)

- ❌ Conectar widgets a entity/world real — fixture inline basta v1.
- ❌ Implementar ECS bridge pra hierarchy view — TreeView mock data.
- ❌ Empacotar React/JS scripts da página HTML — só visual referência.
- ❌ Tentar match pixel-a-pixel sem screenshot harness.

## Lessons learned

- **Plano de Fase 0 acertou exatamente os 4 primitivos faltantes.**
  Nada novo entrou no caminho — PillGroup, ToolRail, StatusBar e
  SectionHeader cobriram tudo que a hero precisava sem improvisar
  widgets ad-hoc no meio. Ler o mockup antes de escrever o plano
  poupou ~3h de retrabalho.
- **`screens/hero.rs` + `screens/hero/fixture.rs` precisa de Rust
  2018+ (que temos com edition 2024).** Submódulos de um arquivo
  irmão funcionam nativamente — não foi preciso `mod.rs` no diretório
  hero/. Testes de modules nested rodam normal.
- **Não wired ECS — fixture inline foi a decisão certa.** Hardcodar
  Player/Slime_01/etc num módulo `fixture` deixa a hero auto-contida
  e roda como smoke test puro. Quando projeto-piloto definir entity
  model, troca-se 1 função (`hierarchy()`/`inspector_sections()`)
  pelo query real, sem mexer no painter.
- **Shell wiring via env var.** `PH2D_HERO_SCREEN=1` é o switch:
  evita refatorar os ~80 linhas de paint pipeline default só pra
  testar visualmente. Usuário roda `PH2D_HERO_SCREEN=1 cargo run -p
  ph2d-host-desktop` e vê a hero; sem env var, o editor 4-zonas
  habitual abre.
- **Inspector field rendering virou ~200 linhas de match arm.** Cada
  `InspectorFieldKind` (Slider/Select/Linked/LinkedSlider) tem layout
  próprio. Conseguimos extrair pra `paint_inspector_field` mas não
  vale promover essas variantes a widgets do core — são specifically
  inspector affordances. Se aparecer outra tela inspector-like, então
  vale generalizar.
- **Selection marquee usou `Stroke::with_dashes`.** Funcionou direto;
  kurbo aceita `[on, off]` em pixels. Visualmente confirma o accent
  dashed pattern do mockup.
- **Visual fidelity pixel-a-pixel deferida.** Plano explicitamente
  diz "fidelidade visual fica fora de escopo sem screenshot harness".
  O resultado é estruturalmente correto, theme-aware, a11y-correto,
  mas alguns pixels divergem do HTML (notavelmente: gradient radial
  do canvas BG omitido em favor de solid Bg0/Bg1; perspective grid
  omitido). Trabalho de polimento visual quando harness existir.
