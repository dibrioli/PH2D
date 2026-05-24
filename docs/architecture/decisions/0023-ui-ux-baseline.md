# ADR-0023: UI/UX baseline — designer-first, Procreate-inspired, WCAG 2.2 AA

**Status:** Accepted
**Data:** 2026-05-09
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Origem:** decisão pré-M12; Enio definiu audiência, baseline visual e princípios; LLM consolidou postura a11y e tokens.
**Ratificação:** 2026-05-09 — Enio aceitou todas as decisões + adicionou §5 (Painéis flutuantes) como primitive central.

## Contexto

Pós-M11 o engine tem renderização vetorial (Vello) + layout de texto (parley) operacionais. M12 começaria a construir o editor sem ter fixado:

1. **Quem é o usuário primário** (game dev humano, LLM via MCP, designer não-coder, modder)
2. **Qual o vocabulário visual** (Unity-style 3-pane, Figma-style toolbar, app-style canvas)
3. **Qual a postura multi-modal** (mouse-first com touch como afterthought, ou paridade real)
4. **Qual o nível de acessibilidade** (best-effort, WCAG AA, WCAG AAA)

Sem essas decisões, M12 viraria um rascunho que precisa ser refeito assim que o "design certo" emergisse — re-trabalhar fundamentos custa muito mais do que escolhê-los antes de escrever uma linha.

O plano operacional original M12 dizia *"3 panels (scene tree, inspector, viewport) via taffy + Vello"*. Esta ADR substitui essa formulação por uma arquitetura de quatro zonas Procreate-inspired.

## Decisão

### 1. Audiência primária

**Designer não-coder fazendo cenas visualmente.** Implica:

- Editor visual é **a porta de entrada**, código é a porta de saída avançada (não o contrário, como Unity ou Godot).
- Drag-and-drop é cidadão de primeira classe; teclas de atalho são aceleradores, não pré-requisito.
- WYSIWYG: a viewport do editor é a viewport do jogo. Sem "play mode" que muda render path.
- Snap, grid, guides são padrão (não opt-in).
- Undo/redo é generoso (≥ 250 estados, igual Procreate).
- Asset library (sprites, tiles, sons) é tão visível quanto a scene tree.

**Audiências secundárias** (não ignoradas, não otimizadas):
- Game dev humano usando keyboard+mouse (tem todos os atalhos)
- LLM via MCP (M9 já entrega catálogo de tools; UI não bloqueia LLM)
- Modder/player (cenário M13+)

### 2. Visual baseline — Procreate-inspired canvas-first

**Tema padrão:** Dark Mode charcoal "unobtrusive", mantém foco na arte do canvas. Tokens semânticos abaixo.

**Tema alternativo:** Light Mode mais contrastante para ambientes muito iluminados (sol direto, etc). Procreate usa o mesmo padrão.

**Filosofia central — Canvas-First:**
- UI ocupa apenas as **margens**; o **centro é 100% canvas**, sem obstrução.
- **Modo Zen** acionável a qualquer momento: 4-finger tap (touch), `Tab` (keyboard), Q (mouse, configurável). Esconde toda a UI deixando só um indicador discreto no canto.
- **UI translúcida nas bordas** (alpha ~0.85 em panels, ~0.95 em modals).
- **Notificações flutuantes não-modais** no topo do canvas (ex.: *"Undo: Brush Stroke"*, *"Saved 2 s ago"*) — informam sem interromper.

**Inspirações honestas:** Procreate (canvas-first + multi-modal), Figma (toolbar discoverability), Blender 2.8+ (radial menus + workspace profiles), AccessKit demos (a11y patterns). **Não copiamos pixel-by-pixel** — capturamos princípios.

### 3. Arquitetura de layout — Triângulo de Zonas

**Quatro zonas fixas:**

| Zona | Categoria | Conteúdo (M12+) |
|---|---|---|
| **Top-right** | Criar (toca o canvas) | Tile painter, Sprite placer, Asset insertion, Brush picker, Layer, Color |
| **Top-left** | Editar / gerir (modifica o que existe) | Gallery (projetos), Actions (file/save/export), Scene tree, Adjustments, Selections, Transform |
| **Sidebar lateral** | Modular (toque frequente, polegar-distance) | Brush size, Opacity, Snap, Undo/Redo, Eyedropper, **Modify button** (Shift virtual) |
| **Center** | Viewport | 100% canvas; sem obstrução. Fullscreen via 4-finger tap |

**Sidebar:**
- **Móvel verticalmente** (drag no Modify button) — ajustável à altura do polegar
- **Espelhável esquerda↔direita** (canhotos)
- Sempre visível em modo normal; some em modo zen

**Princípio que rege a triangulação:** *frequência × natureza* da ação determina a posição. Top-right toca o canvas; top-left modifica o que existe; sidebar modula valores em uso constante.

### 4. Multi-modal input — paridade real

**Princípio:** toda ação tem **≥ 2 caminhos de entrada**, sem hierarquia entre eles. Touch ≠ second-class.

**Carinho especial:** Touch + caneta (Apple Pencil + Wacom). Hover gestures, pressure curves, tilt response, squeeze (Pencil Pro), ExpressKey (Wacom).

**Tabela canônica de gestos** (subconjunto Procreate adaptado):

| Gesto Touch | Função | Mouse/Keyboard | Wacom |
|---|---|---|---|
| 1-finger drag | Pintar / mover seleção | LMB drag | Pen stroke |
| 2-finger pinch | Zoom | Ctrl+scroll / Alt+RMB drag | Touch ring / atalho |
| 2-finger pinch-twist | Rotacionar canvas | R+drag / Alt+Shift+MMB | Rotate gesture |
| 2-finger drag | Pan | MMB drag / Space+LMB | Pen+modifier |
| 2-finger tap | Undo | Ctrl+Z | ExpressKey |
| 3-finger tap | Redo | Ctrl+Shift+Z | ExpressKey |
| 3-finger scrub | Limpar layer | Ctrl+Del | ExpressKey |
| 3-finger swipe down | Menu copy/paste | Ctrl+Shift+C/V | Atalho |
| **4-finger tap** | **Full screen / Zen** | **Tab / F11** | **ExpressKey** |
| Quick-pinch | Fit to screen | F / numpad . | Atalho |
| Draw-and-hold | QuickShape (snap) | Shift durante o stroke | Shift |
| Hover (Pencil) | Pré-visualizar pincel | Mouse-over | Hover quando suportado |

**Customização total:**
- Editor visual de mapeamento dentro do editor (não só arquivo de config)
- Toggle on/off por shortcut (desligar atalhos raramente usados)
- Múltiplos triggers por função (ex.: undo por gesto + atalho + ExpressKey)
- **Detecção de conflito** com warning visual ao atribuir
- **Profiles** por contexto (ex.: "Tile Editing", "Animation", "Physics Setup", "Lighting")

### 5. Painéis flutuantes + arrastáveis (Tool Drawers)

**Padrão central** observado em Procreate (selection panel, brush settings, layer ops): **panels contextuais que aparecem flutuando sobre o canvas quando uma ferramenta exige opções**, são livremente movíveis pelo usuário, e somem quando outra ferramenta é ativada. **Usaremos muito essa primitive em M12+ para todas as edições contextuais.**

**Distinto de §6 (QuickMenu radial):**
- **Radial:** invocação por gesto, transient, 6 slots, fecha após escolha
- **Tool drawer (este):** persistente enquanto ferramenta ativa, multi-row, navegável

**Anatomia padrão:**

| Elemento | Função |
|---|---|
| Header com drag handle | Permite arrastar; pode incluir botões pin / collapse |
| Tab row / segmented (opcional) | Mode picker (ex.: Procreate Selection: *Automatic / Freehand / Rectangle / Ellipse*) |
| Action grid | Botões com icon + label (ex.: *Add / Remove / Invert / Copy & Paste / Feather / Save & Load / Color Fill / Clear*) |

**Comportamento:**
- **Drag livre** pelo header — usuário decide onde fica
- **Snap soft** quando solto perto de bordas/cantos (bottom-center é o default Procreate)
- **Auto-dismiss** quando ferramenta muda OU quando 4-finger tap (modo Zen)
- **Translúcido** (alpha igual sidebar, ~0.85) — não obstrui canvas mais do que precisa
- **Não-modal** — clicar fora não fecha; é parte do workspace persistente
- **Memória de posição por ferramenta** — quando você reativa a Selection tool, o panel volta onde você o deixou
- **Collapse** via double-tap header → vira chip-icon que reabre com tap

**Interação multi-modal (per §4):**
- Touch: drag o header
- Mouse: drag o header (cursor vira "move" no hover)
- Pencil/Wacom: pen drag o header

**Casos de uso na engine (M12+):**
- **Selection** — Add / Remove / Invert / Copy / Feather / Color Fill / Clear / Save & Load (igual Procreate na imagem de referência da ratificação)
- **Brush settings** — size / flow / jitter / scatter além do que cabe na sidebar
- **Layer ops** — blend mode picker, opacity, mask, clipping
- **Transform** — move / rotate / scale / skew + numeric inputs
- **Tile painter** — brush shape, random rotation, snap precision, layer target
- **Animation** — timeline scrubber + keyframe ops
- **Physics** — body type, mass, friction, restitution sliders quando entity tem rigid body
- **Curve / path tool** — node ops, smoothing, simplify
- **Color picker** — HSL/RGB/HEX inputs + recent + saved palettes

**Por que NÃO docks fixos tradicionais (Unity-style):**
- Dock fixa rouba pixel real do canvas o tempo todo, mesmo quando ferramenta está inativa
- User-positioning permite adapting a workflow + handedness (canhoto coloca no lado oposto)
- Touch-first ergonomic: usuário aproxima o painel da mão dominante; dock fixo força braço cruzar tela

**Primitive arquitetural (M12 PR):** `ph2d_editor::FloatingPanel { header, tabs?, body, anchor, position, tool_id }` — single primitive reusável; toda tool option vira instância dela. State (posição + collapsed?) é serializado por `tool_id` no save de workspace.

### 6. QuickMenu radial

**Menu radial customizável invocado por:**
- **Touch:** hold (configurável: long-press 500 ms padrão) ou squeeze do Apple Pencil Pro
- **Mouse:** Q hold + drag, ou RMB hold
- **Wacom:** ExpressKey configurável

**Características:**
- **6 botões** (limite cognitivo Miller-friendly)
- **Touch-drag (flick gesture):** após aprendido, o menu nem precisa ser visto — memória muscular direcional
- **Profiles ilimitados** (sketching, coloring, tile-paint, animate, physics, lighting…)
- Ações disponíveis pra mapear: 50+ comandos do editor

### 7. Sliders com precisão progressiva

**Cada slider numérico expõe controle fino sem widget extra:**

- **Tablet/Pencil:** drag horizontal no slider; **afastar perpendicularmente** aumenta precisão (mais distância = passos menores). O eixo perpendicular vira o controle de resolução.
- **Mouse:** Shift+drag = modo fino; OU drag vertical perpendicular ao slider (mesmo princípio).
- **Wacom:** funciona nativo se o mapeamento expor pressão como modificador.

Sem popup numérico, sem botão "modo fino". O comportamento é o widget.

### 8. Modify button (Shift virtual)

Sidebar tem um botão **Modify** que age como tecla Shift virtual:
- Hold + tap em qualquer lugar = eyedropper (default)
- Reprogramável para qualquer ação
- **No tablet:** crítico — substitui modifier keys que não existem
- **No desktop:** mantém visível em modo touch; modifier keys reais cobrem o caso normal

### 9. Hover gestures

**Para alterar tamanho/opacidade do pincel sem sair do canvas:**

| Modo | Mecanismo |
|---|---|
| Apple Pencil hover | Pinch interno = diminuir, pinch externo = aumentar (size); slide L/R = opacity |
| Wacom hover | Idem quando suportado pelo driver |
| Mouse | Scroll wheel + modificador (`[`/`]` para size, Shift+scroll para opacity) |
| Touch sem hover | Long-press + drag radial |

### 10. Acessibilidade — WCAG 2.2 Level AA + AccessKit

**Standard formal:** **WCAG 2.2 Level AA** — industry definitive (Apple/Microsoft/Adobe/Figma todos garantem AA). AAA é aspiracional, raramente atingível 100 % (contrast 7:1, todo conteúdo com nível de leitura ≤ Lower Secondary, etc).

**Cross-platform abstraction:** **[AccessKit](https://github.com/AccessKit/accesskit)** (Linebender, mesma org de Vello/parley/skrifa). Único projeto Rust sério; integra Mac VoiceOver + Win Narrator + Linux AT-SPI + iPadOS VoiceOver pela mesma trait `Node`.

**Postura derivada da audiência (designer não-coder):**

| Item | Decisão |
|---|---|
| **Multi-touch fallback** | **"Single-Touch Companion"** overlay (igual Procreate): toda ação multi-touch tem botão equivalente single-touch flutuante |
| **Multi-key fallback** | Sticky modifiers (Mac convention) + on-screen Modify button já decidido |
| **Keyboard nav** | **100 % reachable.** Tab/Shift-Tab/Arrows; Escape sempre fecha modal/popover; focus visible sempre (ring ≥ 2 px no token `border-emphasis`) |
| **Reduced motion** | OS setting respeitado: `prefers-reduced-motion` → desliga animações ≥ 200 ms |
| **Live regions** | Announce undo/redo, save, errors via `accesskit::Live` sem roubar foco |
| **Contrast** | AA tokens enforced (4.5:1 texto, 3:1 elementos UI). Lint contra paleta — qualquer cor proposta fora dos tokens bate erro |
| **Text scaling** | Type scale em rem-equivalente; respeitar OS text size override até 200 % sem clipping |
| **Color-blind safe** | Estado nunca codificado SÓ por hue; sempre acompanha shape/icon/position |

**Gate M12:** ainda *"Mac VoiceOver navega editor"*; agora estende para *Windows Narrator + iPadOS VoiceOver no mesmo nível*. Linux AT-SPI fica em best-effort até alguém pedir.

### 11. Hierarquia visual por luminosidade (não por hue)

Estado é codificado por **luminosidade/saturação de uma única hue**, não por hues diferentes (mantém coesão visual + acomoda color-blindness).

**Exemplo Procreate:** layer ativo = azul brilhante (Primary), layer secundário selecionado = azul escuro (Secondary).

**Aplicação na engine:** uma única hue de destaque amostrada das paletas oficiais do Procreate (Dark + Light) com 3 intensidades.

**Valores propostos** (a serem sampled-from-screenshot e validados WCAG AA na PR de implementação dos tokens — não tenho Procreate instalado pra sample exato agora):

| Token | Dark Mode | Light Mode | Notas |
|---|---|---|---|
| `background` | `#161616` (charcoal) | `#F5F5F5` (warm off-white) | Procreate-canvas-feel; nem preto puro nem branco puro |
| `surface` | `#1F1F1F` | `#FFFFFF` | Painéis e sidebar |
| `surface-elevated` | `#2A2A2A` | `#FFFFFF` + shadow | Popovers, tooltips |
| `text-primary` | `#E8E8E8` | `#1A1A1A` | ≥ 4.5:1 vs surface |
| `text-secondary` | `#9A9A9A` | `#5A5A5A` | ≥ 4.5:1 vs surface |
| `accent-primary` | **`#0AB4FF`** (Procreate iconic blue-cyan) | **`#0078D4`** (mesma hue, ajustada para AA em fundo claro) | Active/selected primary |
| `accent-secondary` | `#0786BF` (mesma hue, dimmer) | `#005A9E` | Selected secondary / pressed |
| `accent-tertiary` | `#04566F` (mesma hue, mais dimmer) | `#003D6B` | Hover, focus ring |

**Disclaimer honesto:** o exato HEX do accent Procreate pode variar entre versões e screenshots. A PR de implementação valida com:
1. Sample direto de screenshots oficiais Procreate 5+ (ambos modos)
2. Verificar AA contra os tokens `surface` propostos acima
3. Ajustar ±5 % luminance se necessário sem mudar hue

A direção (cyan-blue Procreate-style, não verde-Inkscape ou laranja-Blender) é o que importa.

### 12. Tokens semânticos

**Color tokens** (light + dark variants, AA contrast garantido):

```
background      — primary canvas behind everything
surface         — panels, sidebar, toolbar
surface-elevated — popovers, tooltips, modals
border          — low-contrast separator
border-emphasis — focused/hovered border, focus ring (≥ 4.5:1)
text-primary    — main copy, ≥ 4.5:1 vs surface
text-secondary  — labels, captions, ≥ 4.5:1 (on AA), ≥ 3:1 minimum on AAA-aspirational
text-disabled   — explicit non-interactive (3:1 ok per WCAG)
accent-primary  — single brand hue, max intensity
accent-secondary — same hue, dimmer
accent-tertiary — same hue, dimmest
success         — semantic green (≥ 3:1 vs surface)
warning         — semantic amber (idem)
error           — semantic red (idem)
info            — semantic blue/cyan (idem; pode coincidir com accent-primary se for azul)
```

Cores literais NÃO são usadas em código de widget — sempre token. Lint enforced no design system crate (futuro `ph2d-tokens`).

**Type scale:**

| Token | Tamanho |
|---|---|
| `font-size-xs` | 11 px (badges, tooltips) |
| `font-size-sm` | 13 px (sidebar labels, secondary text) |
| `font-size-base` | 14 px (default body) |
| `font-size-md` | 16 px (primary text, inputs) |
| `font-size-lg` | 20 px (panel headers) |
| `font-size-xl` | 24 px (page headers) |
| `font-mono` | 13 px (paths, IDs, code) |

Type respeita OS text scaling até 200 %.

**Densidade:** **compact** (Procreate é naturalmente compacto sendo tablet-first). Spacing scale em múltiplos de 4 px (4, 8, 12, 16, 24, 32).

**Iconography:** **mix curado de Blender Icons + Godot Icons + Inkscape Icons**, re-skinned para stroke consistente PH2D.

**Por quê os três:**
- **Godot Icons** (MIT) — primário; estilo monoline mais editor-appropriate, cobre 80 % dos icons que precisamos
- **Blender Icons** (GPL — usar SVGs sob fair-use ou redesenhar com mesma pegada; verificar) — preencher icons de 3D-adjacent / bone / skeleton / animation que Godot é mais fraco
- **Inkscape Icons** (LGPL) — preencher icons de vector-editing (pen tool, node editor, path operations) que Procreate-style canvas pode evocar

**Risco de inconsistência visual:** os três têm estilos distintos. Resolução:
1. **Re-pintar tudo** num único stroke-width (1.5 px @ 16 px / 1.75 px @ 20 px) e geometric grid PH2D
2. **Filled vs outline**: outline default; filled só para "estado ON" (ex: layer visibility eye-on vs eye-off)
3. **Sem cor literal** nos SVGs — todos usam `currentColor`, recebem hue via token CSS-var-equivalent

**Tamanhos canônicos:** 16 px (sidebar, dense) e 20 px (toolbar, comfortable). Tudo em SVG; rasterização runtime via Vello.

**Licensing nota:** Godot icons são MIT (compatível com proprietary). Blender e Inkscape são GPL/LGPL — re-desenhar inspirado-em é mais limpo legalmente do que copiar SVG. PR de tokens documenta cada icon com sua origem ("inspired by Blender icon X" vs "MIT-licensed from Godot project").

**Animation:** mínima. Default easing `ease-out` 150 ms. Respeitar `prefers-reduced-motion` → 0 ms para tudo ≥ 200 ms.

## iPad readiness em M12

**M12 ships desktop only** (`shells/desktop` — Mac primary, Linux/Windows secondary). iPad shell é M14+ por dois motivos:

1. **shells/ipad** precisa UIKit/SwiftUI bridge não-trivial; ph2d-host trait existe mas a implementação é trabalho dedicado
2. **HR-13 memory budget validation** requer hardware iPad Pro M2 físico — projeto ainda não tem

**Mas M12 NÃO bloqueia iPad.** O design já assume:

| Abstração | Status M12 | Implicação iPad |
|---|---|---|
| `ph2d_host::PlatformHost` (HR-1) | desktop impl em uso desde M1 | iPad implementa o mesmo trait; ph2d-editor consome só o trait |
| `ph2d_input::Event` (M8) | gestos multi-touch + hover + pencil já tipados; gilrs adapter no desktop | UIKit gesture recognizers traduzem pra mesmo `Event` enum |
| `ph2d_a11y` (AccessKit, ADR-0023 aqui) | Mac VoiceOver via AccessKit Mac backend | iPadOS VoiceOver via AccessKit iOS backend (best-effort hoje, melhora upstream) |
| `ph2d_render` (Vello/wgpu, M5+M11) | wgpu Metal backend no Mac | wgpu Metal backend no iPad (sem mudança) |
| Tokens semânticos (M12) | mesma paleta | mesma paleta |

**Concretamente em M12:** widgets do editor são desenhados com Vello e respondem a `ph2d_input::Event`. Touch events no iPad chegam via UIKit → `Event::*Pointer*` / `Event::Pencil*`. Os widgets nunca veem se está em mouse, touch ou pencil — só veem `Event`.

**Riscos para a transição iPad:**
- **AccessKit iOS** ainda imaturo — pode requerer fallback parcial até upstream amadurecer
- **Notch / Dynamic Island** safe-area insets — adicionar à HostHandler quando shells/ipad existir
- **Apple Pencil hover** só no iPad Pro M2+ — fallback obrigatório (single-touch companion já decidido)

Conclusão: M12 ships desktop hoje, mas cada decisão arquitetural acima é compatível com o iPad shell que vai chegar em M14+. Não estamos pintando-nos num canto.

## Consequências

### M12 reformulado

Plano original M12 (*"3 panels: scene tree, inspector, viewport"*) é **substituído** pela arquitetura de 4 zonas:

- Top-left, top-right, sidebar, viewport (definição completa em §3 acima)
- ph2d-editor wraps tudo
- ph2d-a11y vira **componente crítico** (era opcional no plano antigo); cada widget implementa `accesskit::Node`
- Adiciona dep nova: **AccessKit** em `ph2d-a11y`
- Adiciona crate novo: **`ph2d-tokens`** (color + type + spacing tokens; substitui qualquer literal em widget code)

### Inputs ampliados

- ph2d-input ganha gesture detection: 2/3/4-finger tap, pinch, pinch-twist, swipes, hover, draw-and-hold (M12+)
- Novo subsistema "input mapping" com profiles + conflict detection (parte de ph2d-input ou crate separado `ph2d-bindings`)

### Crates afetados

- `ph2d-editor` → arquitetura de 4 zonas + Modify button + zen mode + notifications + **`FloatingPanel` primitive (§5)** consumida por toda tool option
- `ph2d-a11y` → AccessKit integration; cada widget exporta Node (incluindo `FloatingPanel`s — drag handle exposta como `accesskit::Role::Window` movable)
- `ph2d-input` → gesture detection (multi-touch, hover, pencil pressure)
- `ph2d-text` → respeitar OS text scaling
- `ph2d-vector` → Single-Touch Companion overlay primitive
- **Novo:** `ph2d-tokens` (M12 mesmo PR ou follow-up)
- **Novo:** `ph2d-bindings` ou módulo dentro de `ph2d-input` (M12+)

### Plano operacional

`docs/archive/plans-completed/2026-05-post-spike.md` M12 row atualizada para refletir o novo escopo + dependência desta ADR.

## Implementação / Rollout

**M12 (este sprint, próxima implementação):**
- AccessKit integrado em ph2d-a11y (skeleton)
- ph2d-tokens criado com color + type + spacing tokens
- ph2d-editor com 4 zonas básicas (placeholder content em cada uma)
- **`FloatingPanel` primitive (§5)** funcional + 1 instância demo (ex.: panel de Selection com tabs + action grid, igual Procreate)
- 1 widget de exemplo (Button) implementando `accesskit::Node` + tokens
- Modo Zen funcional (Tab para alternar)
- Notification toast no topo, não-modal

**M13+ (paralelo, ditado por demanda):**
- Gesture detection completa (2/3/4-finger, pinch-twist, hover)
- Editor visual de input mapping
- QuickMenu radial
- Sliders com precisão progressiva
- Single-Touch Companion overlay completo
- Profiles por contexto

## Não decidido (deferred — fica para PR de implementação)

- **HEX exato dos accents** — direção fixada (Procreate cyan-blue). Sample-from-screenshot + AA validation acontece no PR de tokens; valores propostos em §12 são ponto de partida.
- **Animation library:** custom vs lyon vs nada (apenas easings inline). Provavelmente nada — animações são simples demais para justificar dep.
- **Drag-and-drop infrastructure:** Vello primitive vs custom. Decidir no PR M12.
- **Re-skin de cada icon** — 200+ icons Blender/Godot/Inkscape precisam passar pelo mesmo grid 16/20px + stroke 1.5/1.75. Trabalho mecânico distribuído ao longo de M12-M13.

## Riscos

1. **AccessKit em iPadOS** — suporte ainda jovem; pode requerer fallback parcial. Mitigação: testar early, abrir issue upstream se algo faltar.
2. **Procreate como inspiração ≠ cópia** — manter linguagem visual diferente o suficiente para evitar confusão de marca + risco legal. Mitigação: cores próprias, iconography própria, marca PH2D em todo lugar.
3. **Multi-modal input editor** é complexidade própria — pode estourar M12 e bleed para M13. Mitigação: editor de mapeamento é M13+, não M12.
4. **WCAG AA é o teto inicial** — design pode flertar com AAA em alguns lugares (contrast >7:1) sem se comprometer formalmente. Re-avaliar quando primeiro produto comercial pedir certificação formal.
5. **`ph2d-tokens` cria nova dep para todo crate de UI** — aceitar; é o ponto da decisão.

## Referências

- **WCAG 2.2 (W3C):** https://www.w3.org/TR/WCAG22/
- **AccessKit:** https://github.com/AccessKit/accesskit
- **Procreate Handbook (Touch Gestures):** https://procreate.com/handbook (referência de princípios; não cópia)
- **Apple HIG — Accessibility:** https://developer.apple.com/design/human-interface-guidelines/accessibility
- **Microsoft Inclusive Design Toolkit:** https://www.microsoft.com/design/inclusive/
- ADR-0021 (Sim/Present boundary) — UI vive em PresentWorld
- HR-1 (cross-platform abstraction), HR-12 (a11y como concern de presentation)
