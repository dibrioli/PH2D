> Pesquisa web de 2026-10-07 (subagente, WebSearch/WebFetch). Cada afirmação traz a fonte; o que não se confirmou está marcado. ⚠️ Fonte é LEITURA de documentação e de fóruns — nenhum destes apps foi CORRIDO (CLAUDE.md §0.9). O Graphite é Apache-2.0 e já está em `../referencias/graphite` (via `fetch-referencias.sh`).

# UI/UX de editores vetoriais: Figma (UI3 + Draw), Sketch, Penpot, Graphite

Pesquisa web feita em 07/10/2026. Onde a única fonte é de terceiros ou um wiki gerado automaticamente, isso vem indicado. O que não consegui confirmar está marcado "(não confirmado)".

---

## A) FIGMA: UI3 (2024) e Figma Draw (2025)

**Layout (UI3)**
- Três regiões separadas do canvas: o **painel de navegação à esquerda** (arquivo, páginas, layers, assets), a **toolbar na base** e o **painel de propriedades à direita** ([Help: Design/Prototype/Code no painel](https://help.figma.com/hc/en-us/articles/360039832014-Design-Prototype-and-view-Code-in-the-Properties-Panel)).
- A toolbar desceu para a base para libertar o topo e dar uma "roomier feel". Também serve de estrutura padrão comum a todos os produtos da Figma ([Figma blog: Behind UI3](https://www.figma.com/blog/behind-our-redesign-ui3/)).
- Os painéis podem ser redimensionados e recolhidos. Variables, assets e find/replace passaram para a barra de navegação ([Help: Navigating UI3](https://help.figma.com/hc/articles/23954856027159), via resumo de busca, porque a página dá 404 ao fetch).
- **Minimize UI** (`Shift+\`) recolhe os painéis e deixa a toolbar. No modo minimizado, os painéis reaparecem quando há seleção e somem ao desselecionar. **Hide all UI** = `⌘\` / `Ctrl+\` ([Figma blog: Making the move to UI3](https://www.figma.com/blog/making-the-move-to-ui3-a-guide-to-figmas-next-chapter/)).
- **Actions menu** = `⌘K` / `Ctrl+K`. Reúne assets, plugins, widgets, IA e comandos sem atalho. O antigo "create link" do `⌘K` mudou para `⇧⌘U`, o que gerou queixas de memória muscular ([Help: Actions menu](https://help.figma.com/hc/en-us/articles/23570416033943-Use-quick-actions), [forum](https://forum.figma.com/t/in-ui3-return-to-cmd-k-shortcut-for-adding-links/78622)).
- Cerca de 200 ícones redesenhados e **property labels opcionais**: ligam-se para aprender e desligam-se depois ([Behind UI3](https://www.figma.com/blog/behind-our-redesign-ui3/), [Making the move](https://www.figma.com/blog/making-the-move-to-ui3-a-guide-to-figmas-next-chapter/)).

**Painel de propriedades contextual**
- O painel tem as abas Design e Prototype. Os controles listados oficialmente são: alignment/rotation/position, frame size, corner radius, constraints, layout guides, component properties, instance, auto layout, blend modes, text, fill, stroke, effects e export ([Help](https://help.figma.com/hc/en-us/articles/360039832014-Design-Prototype-and-view-Code-in-the-Properties-Panel)). As secções **Position / Layout / Appearance** aparecem nas discussões do UI3 ([forum: constraints hidden](https://forum.figma.com/t/ui3-feedback-constraints-settings-are-hidden/77418)). A ordem exata Position → Layout → Appearance → Fill → Stroke → Effects → Export está **(não confirmada em doc oficial)**.
- Duas decisões de reorganização:
  - os controles de componente/variante passaram a ficar **acima** dos atributos visuais;
  - Auto Layout, W e H foram **unificados numa secção Layout**, separados de X/Y. Uma inversão da ordem de posição chegou a ser testada e foi recusada porque "disrupted muscle memory too much" ([Behind UI3](https://www.figma.com/blog/behind-our-redesign-ui3/)).
- **Selection actions**: ao lado do nome da seleção aparecem multi-edit, select matching layers, mask e "…". As **booleanas saíram da toolbar do topo para um canto do painel**. Os usuários acham que esses ícones aparecem de forma inconsistente conforme o tipo de seleção, e o suporte admitiu que não há documentação do porquê ([forum: selection actions](https://forum.figma.com/suggest-a-feature-11/properties-panel-selection-actions-quite-confusing-41667), [forum: booleans moved](https://forum.figma.com/t/boolean-operations-moved-to-properties-is-disrupting-workflow/90350)).
- O eyedropper passou a detectar estilos e variáveis, e não só hex ([Making the move](https://www.figma.com/blog/making-the-move-to-ui3-a-guide-to-figmas-next-chapter/)).

**Vector edit mode (barra secundária contextual)**
- Entra-se selecionando um ou mais vetores e premindo `Enter`. Aparece uma **toolbar secundária** com: Move (V), Pen (P), Variable width, Shape builder, Cut (X), Bend, Eraser (Shift+E), Lasso (Q) e Paint (Shift+B, que preenche regiões fechadas da vector network) ([Help: Edit vector layers](https://help.figma.com/hc/en-us/articles/360039957634)).
- Mirroring de handles: None / Mirror angle / Mirror angle and length. Com vários pontos selecionados aparece uma **bounding box** para escalar/rodar o conjunto; `Shift` escala proporcionalmente e `Alt` a partir do centro (mesma fonte).
- **Shape builder** é uma ferramenta do vector edit mode:
  - arrastar sobre regiões funde-as;
  - clicar numa região extrai-a;
  - `Alt`+clique subtrai.
  - É **destrutivo**, ao contrário das booleanas, que são não-destrutivas ([Help: Shape builder](https://help.figma.com/hc/en-us/articles/31616004109847)).
- Booleanas **sem atalho padrão** (Mask e Flatten `⌘E` têm). A Figma redirecionou o pedido para "atalhos customizáveis" ([forum](https://forum.figma.com/suggest-a-feature-11/shortcuts-for-boolean-operations-36191)).
- Alinhamento: `Alt+A/D/W/S/H/V`. A fonte é de terceiros ([skillademia](https://www.skillademia.com/shortcuts/figma-shortcuts/)), consistente com o uso comum.

**Figma Draw (Config, maio 2025)**
- É um **modo dentro do mesmo ficheiro**: o botão "Draw" na toolbar troca o modo, e o toggle volta para "Design". Em Draw ficam indisponíveis o auto layout e o prototyping ([Help: Figma Draw](https://help.figma.com/hc/en-us/articles/31440394517143)).
- O que muda em Draw:
  - a toolbar põe Pen, Brush e Pencil em destaque e remove funções "menos relevantes";
  - o painel direito ganha **sliders e previews maiores** e dá acesso a Transforms;
  - as layers mostram **thumbnails grandes**, e duplo-clique na thumbnail faz zoom à layer.
  
  Fontes: [Figma blog](https://www.figma.com/blog/introducing-figma-draw/), [Help](https://help.figma.com/hc/en-us/articles/31440394517143).
- Ferramentas novas:
  - brushes, dynamic stroke (wiggle) e variable width;
  - text on path, pattern fill, noise/texture, progressive blur;
  - shape builder, lasso, multi-edit;
  - rotation origin (`Opt+R`) e repeat transforms (efeito caleidoscópio).
  
  Melhorias de vetor: "better boolean operations and outline stroke engine", Shift+clique adiciona nó e ⌘+clique salta entre vetores ([Figma blog](https://www.figma.com/blog/introducing-figma-draw/)).
- Reação dos usuários:
  - Elogio: "draw, design & dev mode all on one toolbar is a game changer".
  - Queixas: falta eraser, falta curvature tool, não há join de paths, shape builder sem orientação de uso, mais pontos que no Illustrator, **toolbar de Draw presa no canto** ("let me move the Draw toolbar"), troca de modo "clunky", janela de brushes sem scrollbar (mau para tablet), e a maior queixa de todas: sem iPad.
  - A Figma respondeu que o variable width já foi lançado e que há mais ferramentas a caminho.
  
  Fonte: [forum Config 2025 Draw](https://forum.figma.com/share-your-feedback-26/config-2025-figma-draw-is-here-let-s-hear-what-you-think-40305).

**Reação ao UI3 e o que foi revertido**
- **Painéis flutuantes revertidos**: no beta "slowed people down" ("Speed is a feature"). A versão final tem painéis **acoplados mas redimensionáveis**, e o flutuante ficou só em Minimize UI, FigJam e no grid do Slides ([Figma 2024 recap](https://www.figma.com/blog/figma-2024-we-shipped-it-you-shaped-it/)). As queixas foram canvas menor, o gap entre painel e viewport a distrair, e painéis que pareciam flutuantes mas não se moviam ([uxdesign.cc, via busca, porque o fetch deu 403](https://uxdesign.cc/figma-scrapes-ui3s-floating-panels-44303803c1ad)).
- **Constraints escondidas** atrás de um botão minúsculo em Position. Em ago/2024 a Figma lançou um toggle para mostrá-las inline ([forum](https://forum.figma.com/suggest-a-feature-11/launched-toggle-constraints-28001/index2.html)).
- **Scroll horizontal nos painéis** chegou em dez/2024 ("a long-time ask") ([recap](https://www.figma.com/blog/figma-2024-we-shipped-it-you-shaped-it/)).
- Ficaram sem reversão:
  - X/Y separados de W/H, com secções que mudam de posição conforme a seleção;
  - props de componente empurram Position/Size para baixo ("panel keeps jumping");
  - booleanas no canto do painel.
  
  Fontes: [forum gripes](https://forum.figma.com/archive-21/ui3-properties-panel-gripes-8217), [forum](https://forum.figma.com/suggest-a-feature-11/some-ui-changes-on-figma-ui3-10622), [booleans](https://forum.figma.com/t/boolean-operations-moved-to-properties-is-disrupting-workflow/90350).
- A toolbar flutuante da base "please leave my canvas free" ([forum](https://forum.figma.com/ask-the-community-7/new-figma-floating-toolbar-remove-11989)).
- Iterações recusadas internamente: uma "super-minimal UI que só aparecia em hover" ("too unstable") e uma grade de auto layout condensada ("moved around too much") ([Behind UI3](https://www.figma.com/blog/behind-our-redesign-ui3/)).
- O UI3 virou geral para todos a 10/10/2024. O UI2 foi removido depois, por volta de maio/2025 ([forum](https://forum.figma.com/t/i-cant-go-back-to-ui2-from-ui3/78661)).

---

## B) SKETCH (macOS)

**Layout**
- Toolbar no topo, layer list à esquerda, canvas e **Inspector à direita**. Há ainda o Components View (Symbols, Text/Layer Styles, Color Variables) e um switch Canvas↔Components na toolbar ([Sketch docs: toolbar](https://www.sketch.com/docs/designing/the-interface/the-toolbar)).
- O logo do Sketch na toolbar abre a **Command Bar**, e o botão "+" ao lado insere layers. A Command Bar é "keyboard-first", aprende **localmente** os comandos mais usados e põe-nos no topo, e insere/substitui componentes (`Y` insere Symbol, `⌥⌘R` substitui). Veio na versão Athens, 27/05/2025 ([Sketch blog: Athens](https://www.sketch.com/blog/command-bar-athens/)).
- **Insert Window** (`C`): grelha de previews grandes de Symbols, Styles, Color Variables e Templates, com sidebar por biblioteca, busca e arrastar para o canvas ([Sketch docs](https://sketch.com/docs/designing/the-interface/the-insert-window)). Se ainda existe depois de Athens/Copenhagen está **(não confirmado)**.

**Inspector**
- Ordem das secções: Preview & Prototyping → **Alignment** (no topo, aparece com 1+ layers) → Layer (posição, rotação, flip) → Layout (tamanho e comportamento) → **Style** (layer styles, opacity, corners, Fills, Borders, Shadows, Effects) → **Selection colors** (agrupadas por tipo e ordenadas por frequência) → Export. O Inspector "hides and shows these controls based on what you have selected or what you're doing" ([Sketch docs: Inspector](https://www.sketch.com/docs/designing/the-interface/the-inspector/)).
- **Copenhagen (2025.3, nov/2025)** reescreveu o Inspector:
  - painéis **flutuantes** para cor e variáveis de cor;
  - campos numéricos que fazem **scrub ao arrastar sem foco**;
  - funções alternativas com `Option`;
  - layer list com **focus mode** (só a seleção e o seu contexto).
  
  Fonte: [Sketch changelog Copenhagen](https://www.sketch.com/changelog/copenhagen/).

**Toolbar contextual: a polémica**
- Em Copenhagen a toolbar passou a ser **contextual** ("adapts based on your selection and tool choice") e **deixou de ser customizável**. A justificação foi que uma toolbar fixa não serve fluxos com "modos" e que é preciso poupar espaço num ecrã de 14″. Um funcionário disse que não vão torná-la customizável "anytime soon, perhaps ever" por razões de arquitetura, e sugeriu a Command Bar no lugar ([forum Sketch, via busca](https://forum.sketch.com/t/copenhagen-update-not-able-to-customize-toolbar/5911)).
- Reações: "I don't have time to relearn my design tool every few months"; quebrou "muscle memory since Sketch 3"; procurar ícones numa "ever changing toolbar"; controles não nativos (Troughton-Smith); falta do botão Share. Houve elogios ao uso contido de Liquid Glass e à leveza da app ([mjtsai](https://mjtsai.com/blog/2025/11/20/sketch-copenhagen/)).

**Edição vetorial**
- Duplo-clique ou `Return` entra no modo de edição. `⌘Return` edita todas as shapes de um grupo ([Sketch docs: vector editing](https://go.sketch.com/docs/designing/vector-editing.md)).
- **Tipos de ponto com atalhos 1–4**: Straight / Mirrored / Independent (Disconnected) / Asymmetric (mirror angle). O Inspector mostra o X/Y do ponto e um **raio por ponto** nos Straight. Clicar no segmento adiciona ponto e `Shift`+clique adiciona no meio. Backspace num handle reta a curva. Há Scissors. `⌘⌥O` abre/fecha o path. A ferramenta Vector é `V` (mesma fonte).
- Booleanas (Union, Subtract, Intersect, Difference) na toolbar, no menu Layer e na Command Bar ([Sketch docs: booleans](https://go.sketch.com/docs/designing/shapes/boolean-operations/)). São não-destrutivas por tradição do Sketch **(não confirmado nesta fonte)**.

---

## C) PENPOT (open source, 2.x)

**Layout (2.0, 2024)**
- A toolbar foi para o **topo do canvas** e é **recolhível**. Tema claro unificado, navegação só por teclado, ícones e tipografia revistos ([Penpot community: sneak peek](https://community.penpot.app/t/heres-a-sneak-peek-of-penpots-ui-redesign/4749), [itsfoss](https://news.itsfoss.com/penpot-2-release)).
- **Esquerda**: Pages, Layers, Assets e Design tokens. **Direita**: abas **Design / Prototype / Inspect**. Inspect dá medidas e CSS/SVG/HTML ([Help: interface tour](https://help.penpot.app/user-guide/the-interface/)).
- Na **base**: paleta de cores e paleta tipográfica com acesso rápido às bibliotecas. Clique aplica fill e **Alt+clique aplica stroke** ([Help: color/stroke](https://help.penpot.app/user-guide/designing/color-stroke/)).
- A sidebar ajusta-se ao contexto: "when editing a grid, the sidebar dynamically adjusts" ([community](https://community.penpot.app/t/heres-a-sneak-peek-of-penpots-ui-redesign/4749)).
- O Design panel tem propriedades fixas (tamanho, posição) e opcionais que se adicionam (stroke, shadow, blur) ([Help](https://help.penpot.app/user-guide/the-interface/)).
- Focus mode (`F`) isola a seleção ([Help: workspace basics](https://help.penpot.app/user-guide/designing/workspace-basics/)).

**Fill / stroke**
- Múltiplos fills e múltiplos strokes. Teclas 0–9 dão opacidade de 10–90 %.
- Cada stroke tem posição center/outside/inside, estilo solid/dotted/dashed/mixed (com campos de dash/gap) e caps com setas, triângulo, quadrado, círculo ou losango.
- O color picker tem RGB, harmony wheel, HSV, eyedropper e tokens.

Fonte: [Help: color/stroke](https://help.penpot.app/user-guide/designing/color-stroke/).

**Paths / nós**
- `P` desenha path (clique = canto, arrastar = curva). Duplo-clique ou `Enter` entra na edição de nós, `Esc` sai. Dois tipos de nó: curve e corner ([Help: layers](https://help.penpot.app/user-guide/designing/layers/)).
- Atalhos de nós: add `Shift+`, delete `Del`, make curve `C`, make corner `X`, join `J`, merge `Ctrl+J`, move nodes `M`, separate `K` ([Help: shortcuts](https://help.penpot.app/user-guide/first-steps/shortcuts/)).
- Aparece uma toolbar de nós ao editar ([community, via busca](https://community.penpot.app/t/a-ux-issue-when-drawing-a-line/10154)). Onde fica exatamente está **(não confirmado)**.

**Booleanas e alinhamento**
- Union, difference, intersection, exclusion e flatten. São **não-destrutivas** (as shapes ficam agrupadas e editáveis) ([Help: layers](https://help.penpot.app/user-guide/designing/layers/)).
- Atalhos: `Ctrl+Alt+U/I/E/D` ([Help: shortcuts](https://help.penpot.app/user-guide/first-steps/shortcuts/)).
- Alinhar e distribuir ficam **no topo do Design panel** ([Help: layers](https://help.penpot.app/user-guide/designing/layers/)). Atalhos: `Alt+A/H/D` (esquerda/centro/direita) e `Alt+W/V/S` (topo/meio/base).
- Atalhos de UI: mostrar/esconder UI `\`, layers `Alt+L`, assets `Alt+I`, paleta `Alt+P`, ajuda `?` ([Help: shortcuts](https://help.penpot.app/user-guide/first-steps/shortcuts/)).

**Motor (relevante para nós)**
- O Penpot está a trocar o render SVG/DOM por um **renderer em Rust → WASM com Skia** (rust-skia), com tiles, culling de viewport e cache de tiles. O estado passa ao Rust serializado em binário ([repo render-wasm](https://ark.sudovanilla.org/Penpot/penpot/src/branch/develop/render-wasm/README.md)).
- Saiu como "**WebGL rendering (beta)**" opcional na 2.16, desligado por omissão. A 2.17 continua a endurecê-lo ([Penpot 2.16](https://community.penpot.app/t/penpot-2-16-release-alegria-geral/10626), [alternativeto 07/2026](https://alternativeto.net/news/2026/7/design-tool-penpot-brings-background-blur-better-webgl-rendering-and-design-token-upgrades/)).

---

## D) GRAPHITE (Rust, node-based)

**Layout** ([Graphite: interface](https://graphite.art/learn/interface/), [Document panel](https://graphite.art/learn/interface/document-panel/), [Menu bar](https://graphite.art/learn/interface/menu-bar/))
- **Title bar**: menus File / Edit / Layer / Select / View / Window / Help.
- **Workspace** de painéis acoplados com calhas arrastáveis e abas por área. Os painéis são **Document** (centro), **Properties** e **Layers** (à direita) e **Data** (inspeção crua dos dados que saem do nó selecionado; está no menu Window).
- **Status bar** na base com **input hints**: que teclas e gestos do rato valem no contexto atual, em notação `+` e `/`.
- O Document panel tem três partes:
  - **Control bar** no topo. À esquerda ficam as **opções da ferramenta ativa** (ex.: o Select escolhe se o clique apanha a layer mais funda ou a mais rasa). À direita ficam os controles fixos do viewport: overlays, snapping, grid retangular/isométrica, render modes (Normal / Outline / …), zoom e o **toggle do Node Graph**.
  - **Tool shelf** vertical à esquerda, em três grupos de cor: General (cinza), Vector (azul) e Raster (laranja). Por baixo, as working colors primária/secundária (trocar/repor).
  - **Viewport** com réguas, scrollbars e overlays.
- Ordem das ferramentas no código: General = Select, Artboard, Navigate, Eyedropper, Fill, Gradient. Vector = Path, Pen, Freehand, Spline, Line, Rectangle, Ellipse, Shape, Text. Raster = Brush. Heal, Clone, Patch e Relight estão comentadas ([source utility_types.rs](https://raw.githubusercontent.com/GraphiteEditor/Graphite/master/editor/src/messages/tool/utility_types.rs)).
- Atalhos: V, A (Path), P (Pen), N (Freehand), M, E, Y, L, Z (Navigate). Vêm de um wiki gerado a partir do repo ([mintlify.wiki](https://mintlify.wiki/GraphiteEditor/Graphite/tools/overview)), por isso são fonte secundária.
- O painel Layers tem na sua control bar blend mode, opacidade, "novo grupo" e apagar ([mintlify.wiki](https://mintlify.wiki/GraphiteEditor/Graphite/interface/overview), secundário).

**Como funde o vetor procedural com a UI tradicional (o ponto mais relevante para nós)**
- O node graph é um **overlay sobre o viewport** (`Ctrl+Space`), não um painel separado. É "a more detailed view of what the Layers and Properties panels show".
- As **layers** são nós grandes, com thumbnail, empilhados de baixo para cima (composição). Os **nós** formam cadeias de processamento ligadas por fios.

  Fonte: [Graphite docs, via busca](https://graphite.rs/learn/interface/document-panel).
- "**Layer node chains**" encapsulam a sequência linear de nós dentro da layer de destino ([Q3 2024](https://graphite.art/blog/graphite-progress-report-q3-2024/)).
- Edições WYSIWYG e o grafo espelham-se: "underlying operations mirroring between both views" ([digitalproduction](https://digitalproduction.com/2025/07/14/graphite-open-source-vector-design-tool-with-a-twist/)).
- O **Properties panel** é dinâmico: mostra os parâmetros de cada nó da layer ou do nó selecionado. Ganhou **secções de nó fixáveis (pin)**, um botão de inserir nó e renomear layer ali mesmo ([Q4 2024](https://graphite.art/blog/graphite-progress-report-q4-2024/)).
- O **Boolean Operation node** é não-destrutivo: converte uma pilha de shape layers numa forma ([Q3 2024](https://graphite.art/blog/graphite-progress-report-q3-2024/)). No menu Layer: Union, Subtract Front, Subtract Back, Intersect, Difference ([Menu bar](https://graphite.art/learn/interface/menu-bar/)).
- O **Path node** permite edição manual de pontos **num estágio do pipeline**, mesmo depois de uma forma procedural como uma estrela. O resultado é editar à mão sem perder os parâmetros ([Q3 2024](https://graphite.art/blog/graphite-progress-report-q3-2024/)).
- Alinhamento em Layer > Align (6 opções). Também Flip, Turn ±90° e Grab/Rotate/Scale **G/R/S com entrada numérica** ao estilo Blender ([Menu bar](https://graphite.art/learn/interface/menu-bar/)).
- Ferramenta Path:
  - `Space`+arrastar handle move a âncora junto;
  - `Tab` troca para o handle oposto;
  - `Shift+Ctrl` dá passos de 15° e trava os ângulos;
  - duplo-clique alterna o ponto entre canto e suave (colinear).
  
  Pen com `Shift` liga endpoints de layers diferentes ([Q4 2024](https://graphite.art/blog/graphite-progress-report-q4-2024/)). Segurar `Alt` mostra a distância entre o selecionado e o hovered ([Q3 2024](https://graphite.art/blog/graphite-progress-report-q3-2024/)).
- Desktop: o build com Tauri foi abandonado ("insurmountable technical incompatibility") ([LWN, 26/12/2025](https://lwn.net/Articles/1051242)). Os release candidates (RC2 a 06/01/2026) correm o viewport nativo em wgpu, com o CEF só na UI externa ([itsfoss, 02/2026](https://itsfoss.com/graphite-graphics-editor/)). Os pincéis pequenos ainda ficam com lag. Um Timeline panel com keyframes em qualquer parâmetro de nó está planeado ([Year in review 2024](https://graphite.art/blog/year-in-review-2024-highlights-and-a-peek-at-2025/)).

---

## Padrões transferíveis

1. **Quatro regiões com papéis fixos**: tool shelf (esquerda) = verbos, barra de opções da ferramenta (topo do canvas) = como o verbo atua, painel de propriedades (direita) = substantivos (a seleção), status bar (base) = o que as teclas fazem agora (Graphite).
2. **Barra de opções da ferramenta à esquerda e controles do viewport fixos à direita, na mesma faixa** (control bar do Graphite). Snapping, grid, render mode e zoom nunca mudam de lugar.
3. **Status bar com input hints vivos** (Graphite): ensina atalhos sem tutorial e combina com o perfil de dono que aprende pelo smoke.
4. **Edição de pontos como sub-modo com barra secundária própria** (Figma vector edit: Move, Pen, Bend, Lasso, Paint, Cut, Shape builder), com entrada por `Enter`/duplo-clique e saída por `Esc` (Figma, Sketch e Penpot).
5. **Tipos de ponto com atalhos numéricos 1–4** (Sketch) mais o par **C/X curva↔canto e J join** (Penpot). Duplo-clique no nó alterna canto↔suave (Graphite).
6. **Seleção de vários pontos ganha bounding box** para escalar/rodar o conjunto (Figma).
7. **Raio de canto por ponto** no painel quando o ponto está selecionado (Sketch).
8. **Booleanas não-destrutivas como grupo ou nó** (Penpot, Graphite). O "Shape builder" destrutivo fica como ferramenta separada dentro do modo de pontos (Figma). No nosso caso a booleana pode ser um nó.
9. **Nó "Path" que se aplica a meio do pipeline** (Graphite): editar à mão o resultado de um gerador procedural sem perder os parâmetros. É o padrão central para um editor node-based.
10. **Painel de propriedades derivado da cadeia de nós da layer**, uma secção por nó, com **pin** de secções e um botão de inserir nó (Graphite). As layers são a vista simplificada do mesmo grafo, e o grafo é um overlay alternável (`Ctrl+Space`).
11. **Alinhamento no topo do painel de propriedades**, só visível com seleção (Sketch, Penpot), e com atalhos `Alt+A/D/W/S/H/V` (Figma/Penpot).
12. **Múltiplos fills e strokes empilhados**, cada um com olho, opacidade e posição inside/center/outside, mais caps e setas (Penpot).
13. **Selection colors**: todas as cores da seleção agrupadas por frequência, editáveis de uma vez (Sketch).
14. **Paleta na base: clique = fill, Alt+clique = stroke** (Penpot). Teclas 0–9 dão opacidade (Penpot).
15. **Campos numéricos com scrub ao arrastar, sem focar** (Sketch Copenhagen). Picker de cor em painel flutuante, não inline.
16. **Paleta de comandos única** (`⌘K` na Figma, Command Bar no Sketch) que aprende os comandos mais usados localmente (Sketch) e cobre o que não tem atalho.
17. **Ocultar a UI em dois níveis**: minimizar painéis (`Shift+\`) e esconder tudo (`⌘\`) (Figma). No Penpot é `\`.
18. **Modo "Draw" como troca da toolbar e do painel, não de documento** (Figma Draw): sliders e previews maiores, thumbnails grandes nas layers, duplo-clique na thumbnail faz zoom.
19. **Property labels opcionais**: ligam-se para aprender e desligam-se depois (Figma).
20. **Grupos de ferramentas com cor por família** (Graphite: geral, vetor, raster), o que encaixa na nossa divisão por família.

## Armadilhas (o que os usuários odiaram)

- **Painéis "flutuantes" que não se movem**: menos canvas, gap distrativo, mais lentos. A Figma reverteu-os ([recap](https://www.figma.com/blog/figma-2024-we-shipped-it-you-shaped-it/)).
- **Toolbar flutuante a tapar o canvas** e impossível de mover. A toolbar de Draw "presa no canto" foi um dos pedidos explícitos ([forum](https://forum.figma.com/ask-the-community-7/new-figma-floating-toolbar-remove-11989), [Draw](https://forum.figma.com/share-your-feedback-26/config-2025-figma-draw-is-here-let-s-hear-what-you-think-40305)).
- **Mover controles frequentes para cantos minúsculos**: booleanas e constraints no UI3. A Figma teve de lançar um toggle para as constraints.
- **Partir grupos que o usuário lê como um só** (X/Y longe de W/H) e **secções que mudam de posição conforme a seleção** ("the panel keeps jumping").
- **Ícones contextuais que aparecem de forma inconsistente** sem regra explicável (selection actions da Figma).
- **Toolbar contextual sem customização**: perde-se memória muscular e caça-se ícones numa barra que muda (Sketch Copenhagen). Contextual só funciona se as posições forem estáveis.
- **Reatribuir atalhos consagrados** (`⌘K` do link virou Actions) e **falta de atalho para booleanas**.
- **Redesenhos frequentes**: "I don't have time to relearn my design tool every few months".
- **Modos com funções desligadas** sem explicação (Draw tira auto layout e prototyping) e **troca de modo "clunky"**.
- **Ferramentas destrutivas ao lado das não-destrutivas sem aviso** (shape builder) e **sem guia de uso**.
- **Pen que gera pontos a mais** e a falta de join/eraser/curvature num modo de desenho.
- **UI só em hover** ("super-minimal"): instável, recusada pela própria Figma.
- **Lag em traços pequenos** (Graphite, raster): a percepção de velocidade é feature ("Speed is a feature").

**Não confirmado / limites desta pesquisa:**
- a ordem exata das secções do painel da Figma no UI3;
- se o Insert Window do Sketch sobrevive depois de Athens;
- onde fica a toolbar de nós do Penpot;
- os atalhos de ferramenta do Graphite (só fonte secundária gerada do repo).

Várias páginas do help da Figma e do uxdesign.cc deram 404/403 ao fetch. Nesses casos usei o resumo do motor de busca e indiquei-o ao lado.
