# Prompt para Claude Design — PH2D Editor UI

Cole este arquivo inteiro como prompt inicial no Claude Design. Ele contém todo o contexto, constraints, referências e critérios de aceitação que você precisa para entregar uma UI completa que o Claude implementador vai construir em Vello.

---

## 1. Quem você está atendendo

Estou desenvolvendo o **PH2D — Power House Game Engine**, motor de jogos 2D em Rust ainda em early-stage. Sou o único decisor de produto/arquitetura; um agente Claude (Sonnet/Opus) é o único developer e implementa tudo. Você (Claude Design) entra como o **único designer** do projeto.

Sua entrega vai ser consumida por outro agente Claude (o implementador Vello). Esse agente NÃO consegue ler arquivos `.fig` do Figma diretamente. Os formatos que ele consome bem são:
- **HTML + CSS** (renderiza em browser; ele compara pixel-a-pixel com o que constrói em Vello)
- **JSON** (para design tokens; ele faz codegen para Rust)
- **SVG** (para ícones; pode converter para Vello paths)
- **PNG** (para mockups estáticos de validação visual)
- **Markdown** (para specs de interação, animação, acessibilidade)

Não use formatos proprietários ou ferramentas que não exportem para os acima.

---

## 2. O produto

PH2D é um **editor visual de jogos 2D**, não um app de pintura. Mas a linguagem visual e UX que admiramos é a do **Procreate (iPad)**: canvas-first, UI flutuante, gestos canônicos, minimalismo refinado, dois top toolbars + sidebar lateral.

Diferente do Procreate, o PH2D precisa lidar com:
- Cenas com hierarquia de entidades (game objects)
- Componentes (Transform, Sprite, Collider, RigidBody, Script, etc.)
- Asset library (sprites, prefabs, scripts Luau, audio, materials)
- Inspector de propriedades editáveis
- Modo Play/Pause/Step (rodar o jogo dentro do editor)
- Hot-reload de scripts
- Console de debug

Plataformas alvo: **Mac, iPad, Win, Linux** (mesma UI em todas — cross-platform via Rust+Vello). iPad é o cliente premium (touch + Pencil); Mac é o dev-driver (mouse+keyboard).

Usuário típico: **game dev solo ou pequeno time**, criando jogos 2D estilo Hollow Knight / Celeste / Stardew Valley. Familiarizado com Unity/Godot/Aseprite mas frustrado com complexidade ou com workflow não-touch.

---

## 3. Anti-objetivo (importante)

**Não copie Procreate.** Procreate é app de pintura — `Brush/Smudge/Eraser` não fazem sentido como tools primárias do PH2D. **Adote a gramática visual** (canvas-first, top dual toolbars, sidebar lateral, gestos, modo Zen, Modify button) mas **substitua o conteúdo** por conceitos de game editor (entidades, prefabs, scripts, scenes).

**Não adicione features de gameplay.** Você é designer de UI/UX, não de mecânica. Não invente sistemas de combate, dialogue tree, etc. Foque em editor.

---

## 4. Reference dump — UI/UX do Procreate (estudo prévio)

Seguinte síntese vem de estudo do Handbook oficial + reviews recentes. Use como **base do vocabulário visual**, mas adapte para o contexto game editor:

### Filosofia
- **Canvas-first 100%** — viewport ocupa tela inteira; UI flutua
- **Minimalismo + progressive disclosure** — features aninhadas; superfície limpa
- **Gesture-first** — todo botão tem gesto alternativo
- **Single-touch companion** — Pencil desenha, mão livre opera UI lateral
- **Modo Zen (4-finger tap)** — esconde toda chrome, só canvas

### Layout canônico
- **Top-left toolbar** (5 botões horizontais): Gallery (back), Actions (wrench), Adjustments (wand), Selection (lasso S), Transform (arrow). Hit-target ≥ 44pt.
- **Top-right toolbar** (5 botões horizontais): Brush, Smudge, Eraser, Layers, Color (única colorida — swatch ativo).
- **Sidebar lateral vertical** (default left, configurável right via Prefs):
  - Brush size slider vertical (top)
  - **Modify button** quadrado central (preto idle, cyan ativo) — entre os 2 sliders
  - Brush opacity slider vertical (bottom espelhado)
  - Undo arrow + Redo arrow (acima e abaixo do conjunto)
  - Sidebar inteira é **arrastável verticalmente** (drag a partir do Modify)
- **Floating panels** (Color picker, Layers, Brush Library, Adjustments, Actions): dark blur translucent, rounded ~12pt, abrem ancorados próximos ao botão que invocou; tap fora fecha; só um por vez.

### Sistema visual (Procreate canônico)
- **Theme dark default**, light optional (Prefs)
- Background painéis: cinza muito escuro com vibrancy iOS (não preto puro). Provável `#1C1C1E`–`#2C2C2E` (system gray)
- **Accent cyan-blue** (estado ativo). Hex visualmente próximo de `#0AB4FF` mas não confirmado oficialmente — você decide o exato
- Tipografia: SF Pro (sistema iOS); UI usa pesos 400/500/600
- Iconografia: line-art monocromático, stroke ~1.5pt, sem fill exceto color swatch
- Border-radius painéis: ~12pt
- Animation: 200-300ms, spring iOS, sem motion gratuito

### Tools (formato)
| Tool Procreate | Glyph | Comportamento |
|---|---|---|
| Brush | Pincel | Modal — tap segundo abre Brush Library |
| Smudge | Dedo | Modal — usa brush como borrador |
| Eraser | Borracha | Modal — usa brush como apagador |
| Selection | Lasso S | Modal — substitui sidebar inferior por subtoolbar (Auto/Freehand/Rect/Ellipse) |
| Transform | Seta 4-pontas | Modal — substitui sidebar inferior por subtoolbar (Freeform/Uniform/Distort/Warp) + Interpolation |

### Gestures canônicos
- Pinch 2-finger — zoom + rotate
- 2-finger tap — undo (não-customizável)
- 3-finger tap — redo (não-customizável)
- 3-finger swipe down — Cut/Copy/Paste menu
- 3-finger scrub — Clear active layer
- 4-finger tap — Toggle Full Screen (Zen)
- Tap+hold canvas — Eyedropper
- Drag color swatch sobre área — ColorDrop (flood fill); hold para Threshold
- Draw + hold — QuickShape (snap polígono)
- Draw line + hold — QuickLine (linha reta perfeita)
- Customizable — QuickMenu radial 6 botões

### Color picker (signature)
5 abas: **Disc** (HSV ring + saturation disc), **Classic** (square + HSB sliders), **Harmony** (5 algoritmos: Complementary/Split/Analogous/Triadic/Tetradic), **Value** (sliders precisos + hex), **Palettes** (custom collections). History (10 cores recentes) acima.

### Layers panel anatomy
- Row: thumbnail 44pt + nome + N (blend abreviation) + visibility checkbox
- Right-swipe revela: Lock / Duplicate / Delete
- Left-swipe: add to multi-selection
- Tap em layer já ativa: menu (Rename, Select alpha, Copy, Fill, Clear, Alpha Lock, Mask, Clipping Mask, Reference, Merge Down, Combine Down, Flatten)
- 26 blend modes (Normal/Multiply/Screen/Overlay/...)
- Groups collapsible
- Background layer especial (não deletável, cor editável)

### Acessibilidade
- VoiceOver ativo desabilita drawing — Procreate aceita esse trade-off
- Dynamic Type ≥66% mostra popup com nome+ícone ampliado em tap-and-hold
- Hit-target mínimo 44×44pt (iOS HIG)

### Críticas conhecidas (mitigar)
- Sliders verticais e Modify button confundem novatos (sem labels)
- Search ausente em Galleries grandes
- Curva de aprendizado é íngreme intencionalmente (knowledge-in-the-head)

→ **PH2D deve mitigar** com tooltips, first-run tour, search desde dia 1.

---

## 5. Como adaptar Procreate para PH2D (game editor)

### Manter (gramática visual)
- Canvas-first 100%
- Two top toolbars + sidebar lateral vertical
- Modify button central como cycle programável
- Floating panels com glass blur + rounded corners
- Sliders verticais
- Gestures canônicos (pinch/2-finger/3-finger/4-finger taps + ColorDrop drag + QuickMenu radial)
- Modo Zen
- Right-handed/left-handed flip

### Substituir (paint → game editor)
| Procreate | PH2D |
|---|---|
| Brush/Smudge/Eraser | Place (entity brush) / Pan-Select / Erase (delete on hover) |
| Brush Library | **Asset Library** — dual-pane: categorias (Sprites/Prefabs/Scripts/Audio/Materials) + grid thumbs |
| Layers panel | **Hierarchy panel** — entity tree, mesma anatomia (thumb + nome + visibility + lock + collapse para groups). Blend modes → render layers / sorting layers. |
| Color picker | **Inspector** — propriedades do entity selecionado, 5 abas: Transform / Physics / Render / Script / Custom. Color picker continua existindo como popover quando edita campo de cor. |
| Adjustments | **Scene effects / Post-processing** — tone-mapping, bloom, ambient color |
| Actions menu | **Project menu** — Add (asset/scene/script/entity), Canvas (scene settings/world bounds/grid), Build (export builds), Video (record gameplay), Prefs, Help |
| Selection sub-modes | **Pick modes** — Auto (by tag), Freehand lasso entities, Rectangle marquee, Ellipse radius |
| Transform sub-modes | **Gizmo modes** — Translate/Rotate/Scale/Skew |
| Brush Studio modal | **Component editor** — modal full-window com 14+ categorias (Transform, Sprite, Collider, RigidBody, Script, AnimationState, ParticleSystem, Light, Audio, etc.) |
| ColorDrop drag | **Drag prefab to scene** — drag asset thumb sobre canvas = instancia entity |
| QuickShape draw+hold | **Place-and-hold** — colocar entity e segurar = snap-align (grid/pixel/parent) |
| QuickMenu radial | **Editor commands** — 6 slots customizáveis: Run, Pause, Step, Reset, Hot-reload, Export |

### Novidades exclusivas PH2D (não tem em Procreate)
- **Bottom HUD pill** (compacta, lower-left) — fps / zoom level / scene name / cost meter (estilo sdf3d-studio reference)
- **Play / Pause / Step controls** no top-right
- **Console** (toggleable bottom panel) — log + Luau errors + assertions
- **Search global** (`Cmd+P` style) — buscar entidades, assets, scripts
- **History panel explícito** (não confiar só em 2-finger tap undo)

---

## 6. Constraints técnicos da implementação

A entrega final será construída em Vello (renderer GPU vetorial Linebender) por outro agente. Saiba o que Vello consegue:

✅ **Vello consegue:**
- Rects, gradients (linear/radial/conic), fills, strokes
- Bezier paths arbitrárias (use SVG)
- Blur (Gaussian)
- Transformações afins (translate/rotate/scale)
- Texto via parley (system fonts; custom .ttf/.otf possível)
- Color blending PREMULTIPLIED_ALPHA
- Cores em sRGB e linear
- Border radius (via path)

⚠️ **Vello tem limitações:**
- Não tem componentes de UI nativo — você desenha, eu construo a partir do zero
- Mesh gradients complexas (multi-color irregular) podem exigir trabalho
- Drop shadows reais (com blur) são mais caros — preferir borders + offsets
- Sem widget primitivo "checkbox" — eu construo do seu mockup

✅ **Stack confirmado:**
- Rust + wgpu 28 + Vello 0.8
- parley 0.6 (texto)
- AccessKit 0.24 (a11y cross-platform)
- ph2d-tokens (color/typo/spacing tokens — você pode redefinir)

❌ **Restrições legais:**
- Tudo que você usar como ASSET (fonte, ícone, imagem) precisa ser MIT/Apache/CC0/Public Domain
- Sem licenças GPL/LGPL/AGPL
- Sem royalty (PH2D será comercial proprietário)

---

## 7. Acessibilidade (mandatório)

- **WCAG 2.2 Level AA** em TODAS combinações texto-fundo nos themes
- **Hit targets** mínimos:
  - Touch: 44×44pt (iOS HIG)
  - Mouse: 24×24pt
  - Reportar AMBOS no spec
- **Focus rings** visíveis (BorderEmphasis ≥ 3:1 contrast)
- **AccessKit roles** declarados em cada widget — sugestão por widget (Button/Slider/Switch/RadioButton/RadioGroup/Group/Window/Live/etc.)
- **Live regions** para toasts (Polite/Assertive baseado em severity)
- **Dynamic Type** equivalent — escala a fonte do sistema
- **Color contrast** anotado em cada par foreground/background

---

## 8. Themes — entregar 4 modos

Já temos referência forte do **sdf3d-studio** (app sister no /Volumes/MAC_EXTERNO/PROJETOS/Referencias/). Adote a mesma estrutura de 4 themes:

1. **Forge SDF** (default) — dark com magenta accent OKLCH(0.74 0.16 340)
2. **Procreate** — dark com cyan accent OKLCH(0.78 0.14 205) (próximo do canônico Procreate)
3. **Sunstone** — light warm com orange accent OKLCH(0.70 0.22 55)
4. **Blueprint** — light CAD com blue accent OKLCH(0.62 0.20 250) — usa sidebar layout em vez de floating

Use **OKLCH** (não HSL/HEX) — perceptualmente uniforme entre themes.

---

## 9. Deliverables (o que entregar)

Crie a pasta `docs/design/` no repositório PH2D com estes arquivos:

### 9.1 `tokens.json` — design tokens machine-readable
```json
{
  "themes": {
    "forge-sdf": {
      "color": {
        "bg-0": "oklch(0.16 0.012 290)",
        "bg-1": "...",
        "accent": "...",
        ...
      },
      "spacing": { "xxs": "2px", "xs": "4px", ... },
      "radius": { "sm": "8px", ... },
      "shadow": { "sm": "...", "md": "...", "lg": "..." },
      "row-h": "26px",
      "icon-btn-size": "36px",
      "section-gap": "14px",
      "panel-layout": "floating"
    },
    "procreate": { ... },
    "sunstone": { ... },
    "blueprint": { ... }
  },
  "typography": {
    "font-sans": "Inter, -apple-system, ...",
    "font-mono": "JetBrains Mono, ui-monospace, ...",
    "size": { "xs": "11px", "sm": "13px", "base": "14px", "md": "16px", "lg": "20px", "xl": "28px" },
    "weight": { "regular": 400, "medium": 500, "semibold": 600, "bold": 700 }
  }
}
```

### 9.2 `component-library.html` — visual catalog
Single self-contained HTML file. Para cada componente:
- Mostrar TODOS os estados (Normal, Hover, Pressed, Focused, Disabled, Active, Selected)
- Mostrar variantes (primary/secondary/ghost/danger/success se aplicável)
- Mostrar nos 4 themes (toggle button no canto)
- Inclua label do nome do estado abaixo de cada exemplo

Componentes mínimos:
- IconButton (circular 36-40pt + tooltip)
- TextButton (ghost/primary/danger)
- Slider (horizontal + vertical, com input de expressão `2*pi/3`)
- Toggle/Switch
- Checkbox
- RadioGroup (segmented + traditional vertical list)
- ColorSwatch + Color picker modal (5 tabs Procreate-style)
- Dropdown / Select
- Combobox (search + select)
- TextInput (single-line)
- TextArea (multi-line)
- NumberInput (with stepper buttons)
- Vector3Editor (3 sliders inline X/Y/Z)
- Toast (4 severities)
- Tooltip
- ContextMenu (right-click)
- Modal / Dialog
- FloatingPanel (glass + grab pill + close + drag handle)
- Tabs (top + segmented)
- TreeView (hierarchy panel — entity tree)
- ListItem (asset row, layer row)
- ProgressBar
- Spinner
- Avatar / IconBadge
- Divider (h + v)
- Card (asset thumbnail card)

### 9.3 `screens/` — telas completas
Uma pasta com 1 HTML por tela (full viewport mockup). Mínimo:

1. `01-welcome.html` — splash / project picker
2. `02-editor-main.html` — editor default state (canvas + chrome completo)
3. `03-editor-place-tool.html` — Place tool ativo, prefab sendo posicionado
4. `04-editor-select-tool.html` — entities selecionadas com gizmo
5. `05-asset-browser.html` — Asset Library aberta full
6. `06-hierarchy-panel.html` — Hierarchy panel com 30+ entities, groups, nested
7. `07-inspector.html` — Inspector de entity selecionada com 5 tabs (Transform/Physics/Render/Script/Custom)
8. `08-color-picker.html` — Color picker modal (5 tabs Disc/Classic/Harmony/Value/Palettes)
9. `09-component-editor.html` — Component editor modal (full-window, 14 cats)
10. `10-script-editor.html` — Luau code editor modal
11. `11-console.html` — Bottom console panel expandido (logs, errors)
12. `12-quickmenu.html` — QuickMenu radial 6 slots (overlay no canvas)
13. `13-zen-mode.html` — Modo Zen (chrome todo escondido)
14. `14-play-mode.html` — Play state (tools desabilitadas, HUD muda)
15. `15-build-export.html` — Build/Export modal
16. `16-prefs.html` — Preferences (sub-tabs: General/Theme/Gestures/Shortcuts)
17. `17-search-global.html` — Cmd+P style global search overlay

Cada tela deve:
- Aspect ratio iPad 12.9" landscape (4:3 ≈ 1366×1024)
- E aspect ratio iPad 11" landscape (1194×834)
- E aspect ratio Mac default (16:10 ≈ 1440×900)
- Selecionar UM dos 4 themes (use forge-sdf como default; mostrar 1-2 telas em outros themes pra validar)

### 9.4 `interactions.md` — spec de interação
Para cada widget e flow:
- **Mouse behavior** — click/hover/drag
- **Touch behavior** — tap/long-press/drag/pinch
- **Keyboard shortcuts** — incluir Mac (Cmd) e Win/Linux (Ctrl)
- **Animation** — duration ms + easing (ease-out, spring, cubic-bezier)
- **Edge cases** — empty state, error state, loading state

### 9.5 `gestures.md` — gestos canônicos
Mapping completo dos gestos PH2D (adaptados de Procreate):
- 1-finger drag (pan canvas)
- 2-finger pinch (zoom)
- 2-finger tap (undo)
- 3-finger tap (redo)
- 4-finger tap (Zen)
- Long-press (radial QuickMenu)
- Apple Pencil double-tap (Modify cycle)
- Edge swipe (panel show/hide)
- ...

### 9.6 `icons/` — pasta com SVG
Cada ícone como SVG individual, optimized (no metadata). Stroke 1.5pt, viewBox 24×24. Mínimo:
- Tools: place, pan, select, erase, color, text, layer, asset, camera, grid
- Actions: undo, redo, save, open, build, play, pause, step, reset, settings, help, search
- Navigation: chevron-left/right/up/down, close (×), check (✓), more (···)
- File types: sprite, prefab, script, audio, scene, material
- Status: success, warning, error, info

### 9.7 `animation.md` — motion specs
- Easing curves recomendadas (cubic-bezier values)
- Duration tiers: instant (<100ms), fast (150ms), default (250ms), slow (400ms)
- Stagger patterns (lista anima 1-2-3 com delay)
- Spring physics quando relevante (sidebar drag, panel snap)

### 9.8 `accessibility.md` — a11y compliance
- Contrast ratios calculados (WCAG AA) por par de tokens
- AccessKit role mapping por widget
- Keyboard navigation order (focus traversal)
- Screen reader labels canônicos
- Reduced motion fallbacks

### 9.9 `README.md` — overview
- Sumário do design system
- Como navegar os arquivos
- Convenções (naming, structure)
- Como atualizar / contribuir

---

## 10. Princípios de design (siga estes)

### 10.1 Consistência > criatividade
Não invente novo widget para resolver problema novo se um widget existente serve. Reuse.

### 10.2 Density vs comfort
PH2D é dev tool — deve ser denso (mais info por tela que app casual). Mas em iPad com touch, precisa breathing room. Use density variable conforme device (data-device="mobile|tablet|desktop").

### 10.3 Mostrar o que faz
Procreate sofre crítica por afford ambíguo. Sempre incluir tooltip + first-run hint.

### 10.4 Cores como semântica, não decoração
Accent só para active state. Vermelho só para error/destructive. Verde só para success. Não use cor para diferenciar widgets que tem mesmo papel.

### 10.5 Responsivo, não adaptativo
Mesma UI em iPad 11", iPad 12.9", Mac. Layout reflowa elegantemente. Não desenhe versão "iPad Mini" separada.

### 10.6 Touch-first sem prejudicar mouse
Hit targets generosos. Hover states existem para mouse mas não dependem deles para descobrir features.

### 10.7 Respeite o canvas
Canvas é sagrado. UI nunca cobre conteúdo crítico. Painéis floating se afastam quando se aproximam de seleção ativa.

---

## 11. Critérios de aceitação

Vou validar tua entrega rodando o `component-library.html` no browser e checando:

✅ **Todos os 4 themes funcionam** (toggle no canto top-right) sem nenhum elemento quebrado em qualquer um
✅ **Todos os widgets listados em 9.2 presentes** em todos seus estados
✅ **Todas as 17 telas listadas em 9.3 presentes** com aspect ratios corretos
✅ **tokens.json válido** (parseable + completo + 4 themes)
✅ **Contraste WCAG AA** validado (você reporta os ratios)
✅ **icons/** com pelo menos 30 SVGs nomeados (tools + actions + navigation + file types + status)
✅ **interactions.md, gestures.md, animation.md, accessibility.md** preenchidos com substância
✅ **Estilo coeso** — nada solto que pareça outro produto

---

## 12. Liberdades que tens

- **Decida ícones** — não te dou referências; você escolhe glyphs que façam sentido (ou referencia heroicons/lucide style)
- **Decida exatamente os tons** dos themes (use as accents indicadas como ponto de partida; ajuste hue/chroma para acessibilidade AA)
- **Decida tipografia** (recomendo Inter + JetBrains Mono — mas se tiver alternativa MIT/SIL melhor, justifique)
- **Decida border-radius/spacing scale** dentro dos limites razoáveis
- **Decida animation curves**
- **Sugira features adicionais** ao editor se vir gap óbvio (mas marque como "[?]" para eu validar antes)

---

## 13. Restrições explícitas

- **NÃO** use componentes de bibliotecas existentes (Material UI, Chakra, etc.) — design from scratch
- **NÃO** use cores em HEX cru — use OKLCH
- **NÃO** use imagens raster pesadas — só SVG ou rendered HTML
- **NÃO** use JavaScript para funcionalidade real — só para o theme switcher (HTML estático, foco é visual)
- **NÃO** invente terminologia inconsistente — use vocabulário existente onde aplicável (Inspector, Hierarchy, Asset, Scene, Component, Prefab, Build, etc.)
- **NÃO** entregue em pieces — entregue tudo de uma vez em uma única branch/PR
- **NÃO** faça mais do que pedido — escopo está em §9

---

## 14. Workflow esperado

1. Você lê tudo isso (~30 min)
2. Você faz design — pode levar várias horas/dias
3. Você commita TODOS os arquivos de §9 numa única branch + abre PR
4. Eu (cliente) reviso a PR — comentários inline ou pedido de revisão
5. Itera até aprovação
6. Após merge: outro agente Claude (implementador Vello) consome teus arquivos e constrói em Rust

---

## 15. Em caso de dúvida

- **Pequena ambiguidade** — decida e marque "[?]" no spec; eu valido depois
- **Grande ambiguidade arquitetural** — pergunte antes de desenhar (escreva no PR description)
- **Tradeoff entre touch e mouse** — touch ganha (iPad é flagship)
- **Tradeoff entre simples e poderoso** — poderoso ganha (PH2D é dev tool)
- **Tradeoff entre teus gostos e Procreate** — Procreate ganha em gramática visual

---

Vai. Faça o melhor design possível. O sucesso do PH2D depende disso.
