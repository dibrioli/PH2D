# Blender Texture Paint — referência visual da UI (manual oficial)

17 capturas do **Blender Manual** (Texture Paint + Brush), baixadas para servir de
referência de layout/UX ao novo Painter do PH2D. **Não** são telas finais do PH2D —
o PH2D usa seu próprio design system (tokens + Widget Gallery, HR-15). São o **alvo
funcional**: que controles existem e como se agrupam.

## Procedência & licença

| | |
|---|---|
| Origem | `https://projects.blender.org/blender/blender-manual` (Gitea oficial), `manual/images/` |
| Commit | `4164b56d4317f0248a4c24eb21c8ca5f3bfc5f9f` (branch `main`) |
| Baixado em | 2026-06-20 (via LFS batch API) |
| Licença | **CC-BY-SA 4.0** (Blender Manual) — atribuição: Blender Documentation team |

CC-BY-SA é **copyleft de documentação**, não de código — não contamina o código do PH2D.
Mantenha a atribuição se reproduzir as imagens. Pasta **untracked** no git (igual à referência
de código GPL) até o Enio decidir versioná-la.

## Manifesto (nome → o que mostra → uso no plano)

### Texture Paint (modo)
- `texture-paint_introduction_paint-mode.png` — header do modo Paint (seletor de modo). Ref. do **estado "Paint" do tool**.
- `texture-paint_introduction_example.png` — exemplo de pintura aplicada numa textura.
- `texture-paint_tool-settings_brush-settings_popover.png` — **popover de Brush completo**: thumbnail+nome do brush, **Radius (50 px)**, **Strength (0.700)**, **Blend (Mix)**, cada um com toggle de pressão (ícone lápis); **Color Picker** (roda HSV + value), Color Palette, Gradient, Options, Mask. **É o mapa da seção de Brush Settings** (Fase 4).
- `texture-paint_tool-settings_mask_panel.png` — painel de máscara de stencil/textura.
- `texture-paint_tool-settings_texture-slots_panel.png` — slots de textura (alvo da pintura; análogo ao layer-alvo no PH2D).

### Brush (compartilhado com sculpt/paint)
- `brush_introduction_brush-tool.png` — barra de ferramentas de brush (Draw/Soften/Smear/Clone/Fill/Mask).
- `brush_introduction_brush-asset-shelf.png` — prateleira de brushes como assets (presets).
- `brush_stroke_stroke-panel.png` — **painel Stroke**: **Stroke Method = Space**, **Spacing 10%**, **Jitter**, **Input Samples**, **Smooth Stroke (Radius 75px / Factor 0.900)**. **Mapa do motor de stroke** (Fase 2).
- `brush_cursor_panel.png` — painel do cursor de brush (ring/overlay).
- `brush_texture_ui-example.jpg` — brush com textura aplicada.

### Falloff (curva do dab — alvo direto da máscara de dab, Fase 1)
- `brush_falloff_brush-curve.png` — editor de **curva de falloff** custom.
- `brush_falloff_{smooth,sharp,root,sphere,constant,linear}.png` — as **presets de falloff** (perfil radial intensidade×raio). Cada uma é uma função `f(r/R)∈[0,1]` a portar como preset da máscara de dab.

## Mapeamento rápido UI Blender → PH2D (detalhe no plano §02/§03)

| Controle Blender | Onde vive no PH2D |
|---|---|
| Radius, Strength, Blend, pressure toggles | `ph2d-panel-painter-brush` (Fase 4) → `BrushSpec` em `ph2d-painter-brush` |
| Color Picker / Palette | reusa `BlenderColorPicker` widget já existente (SKILL §11.9) |
| Stroke Method / Spacing / Jitter / Smooth | motor de stroke em `ph2d-painter-brush` (Fase 2) |
| Falloff curve/presets | máscara de dab em `ph2d-painter-brush` (Fase 1) |
| Texture slots | layer-alvo no host `ph2d-tool-painter` (já existe) |
