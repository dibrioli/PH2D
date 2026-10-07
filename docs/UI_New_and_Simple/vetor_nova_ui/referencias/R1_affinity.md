> Pesquisa web de 2026-10-07 (subagente, WebSearch/WebFetch) — ⭐ **o estilo preferido do Enio** (*«gosto mais do estilo Affinity que do Inkscape»*). Manual oficial (affinity.studio/help, affinity.help) como fonte primária. ⚠️ LEITURA, nenhum app CORRIDO (CLAUDE.md §0.9): o Affinity é Windows/macOS, sem build Linux; correr sob Wine é possível se uma LEI (não a UI) precisar de oráculo.

# UI/UX do Affinity (Designer 2 e o "Affinity by Canva") para edição vetorial: relatório

**Sobre as fontes.** A maior parte vem do manual oficial. A central de ajuda nova ([affinity.studio/help](https://www.affinity.studio/help/workspace-interface/)) descreve a versão atual do app novo, ou seja, a atualização de setembro de 2026 ([release notes](https://www.affinity.studio/help/release-notes/)). O manual do Designer 2 continua em affinity.help. Não consegui ler o Reddit, porque o site bloqueia o acesso automático. No lugar dele usei o fórum oficial, o Hacker News, o Trustpilot e reviews publicadas.

---

## 0. Nomes e datas (confirmados)

- **Affinity Designer 1.0** saiu em 1/out/2014. O **Designer 2.0** saiu em **nov/2022** e trouxe "shape builder and knife tools, vector warping, and an x-ray view" ([Wikipedia](https://en.wikipedia.org/wiki/Affinity_Designer)). A última linha da V2 foi a **2.6** ([manual D2, "New in V2.6"](https://affinity.help/designer2/en-US.lproj/pages/Introduction/keyFeatures.html)).
- A **Canva comprou a Serif em mar/2024** ([Wikipedia](https://en.wikipedia.org/wiki/Affinity_(software))).
- O **"Affinity" (também chamado "Affinity by Canva", versão 3.0)** foi lançado em **30/out/2025**, no keynote "Creative Freedom". Funde Designer, Photo e Publisher num app só e é gratuito ([Wikipedia](https://en.wikipedia.org/wiki/Affinity_(software)), [tbreak](https://tbreak.com/affinity-free-all-in-one-design-app-uae/)). O manual diz "Affinity by Canva user (launched October 2025)" ([ajuda](https://www.affinity.studio/help/info-for-v1-and-v2-users/)).
- O app é grátis. Só as funções "Canva AI" exigem o plano Canva Premium/Pro ([Wikipedia](https://en.wikipedia.org/wiki/Affinity_(software))). É obrigatório ter conta Canva ([mjtsai](https://mjtsai.com/blog/2025/10/31/combined-affinity-studio-now-free/)).
- **"Personas" passaram a se chamar "Studios"** ("Formerly Personas, Studios are workspaces…", [ajuda: novidades](https://www.affinity.studio/help/introduction-new-features/)).
- Depois vieram a **3.1** em 16/mar/2026, com tema claro e cor de destaque ([scrapbookcampus](https://scrapbookcampus.com/2026/03/affinity-3-1-new-features/), [alternativeto](https://alternativeto.net/news/2026/3/affinity-s-latest-update-brings-light-mode-pixel-to-vector-conversion-and-layer-blending)), e a **atualização de setembro de 2026** ([release notes](https://www.affinity.studio/help/release-notes/)).

## 1. Layout da janela

### 1.1 Mapa geral

| Área | Designer 2 | Affinity (2025+) |
|---|---|---|
| Troca de modo | "Persona Toolbar" (Designer/Pixel/Export). O ícone ativo fica com cores saturadas ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/interface.html)) | "(B) Studios" dentro do Toolbar ([ajuda](https://www.affinity.studio/help/workspace-interface/)) |
| Toolbar (topo) | Comandos frequentes. Muda com a Persona e é personalizável ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/interface.html)) | Studios, "Operations and design aids" e Export ([ajuda](https://www.affinity.studio/help/workspace-interface/)) |
| Context Toolbar | Faixa **fixa** abaixo do Toolbar ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/contextBar.html)) | **Flutuante** por omissão: "appears undocked at the top center of the document view" ([ajuda](https://www.affinity.studio/help/workspace-context-bar/)) |
| Tools panel | À esquerda, com os seletores de cor por baixo ([D2](https://affinity.help/designer2/en-US.lproj/pages/Panels/toolsPanel.html)) | Igual, e o conteúdo muda com o Studio ([ajuda](https://www.affinity.studio/help/panels-tools-panel/)) |
| Studio (painéis) | "Right Studio" e Left Studio opcional ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/customizingWorkspace.html)) | "Right panel group". Os grupos podem ficar à esquerda ou à direita ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)) |
| Barra de status | "Highlights available keyboard modifiers for the selected tool" ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/interface.html)) | "Displays tool hints and available keyboard modifiers for the selected tool" ([ajuda](https://www.affinity.studio/help/workspace-interface/)) |

### 1.2 Toolbar do topo (app novo, Studio Vector)

Fonte: [ajuda](https://www.affinity.studio/help/workspace-toolbar/).

- **View Mode** Vector/Pixel. Clique direito abre opções avançadas, incluindo wireframe.
- **Operations** (só no Vector): Add, Subtract, Intersect, Xor, Divide.
- **Arrange** (Back, Back One, Forward One, Front) e **Transform** (Flip H/V e rotação de 90°).
- **Alignment**: um pop-up com "Align to", que pode ser a seleção, o spread, a margem ou o primeiro/último objeto selecionado.
- **Insertion**: inserir atrás da seleção, no topo da camada ou dentro da seleção (recorte).
- **Snapping** (liga/desliga, desligado por omissão) e um pop-up de opções.
- No Designer 2 havia também **Defaults** ("Synchronize defaults from selection" e "Revert defaults") e **Force Pixel Alignment** ([D2 toolbar](https://affinity.help/designer2/en-US.lproj/pages/Workspace/toolbar.html)).
- **Personalização**: View > Customize Toolbar, arrastando itens para dentro ou para fora da barra, com rótulo "Icon Only" ou "Icon and Text". Reset é feito arrastando o "default set" de volta para a barra ([ajuda](https://www.affinity.studio/help/workspace-customizing-toolbar/)).
- Na 3.1 dá para mover as informações do documento (tamanho, zoom) para a aba do documento e liberar espaço no Toolbar ([scrapbookcampus 3.1](https://scrapbookcampus.com/2026/03/affinity-3-1-new-features/)).

### 1.3 Context Toolbar

Comportamento geral:
- O conteúdo depende da ferramenta, do tipo de seleção e do Studio. Exemplo do manual: a Node Tool mostra opções diferentes para uma forma geométrica e para uma curva ([ajuda](https://www.affinity.studio/help/workspace-context-bar/)).
- No app novo pode ser encaixada pelo menu (View > Context Toolbar > Dock) ou arrastada até posições âncora: "top center, bottom center or any corner of the document view" ([ajuda](https://www.affinity.studio/help/workspace-context-bar/)).
- A ajuda descreve um "Cleaner look by hiding more advanced tool options" ([novidades](https://www.affinity.studio/help/introduction-new-features/)). Na prática, as opções avançadas ficam num botão **"Settings"** dentro da barra (veja as páginas de cada ferramenta abaixo).

**Move Tool (V)** ([ajuda](https://www.affinity.studio/help/tools-tools-move/)):
- Sem seleção: Auto-select (Default/Objects/Groups), Document Setup, App Settings, Transform Objects Separately e Settings (Transform Origin, Hide Selection while Dragging, Show Alignment Handles).
- Com objeto selecionado: **Fill e Stroke como amostras de cor que abrem um pop-up**, campo de largura do traço, um botão "Edit stroke settings" (estilo, juntas, pontas, setas), Lock Children, Group/Ungroup e Convert to Curves.
- Com imagem selecionada: Replace, Edit, Develop, Liquify etc.

**Node Tool (A)** ([ajuda](https://www.affinity.studio/help/tools-tools-node/), [D2](https://affinity.help/designer2/en-US.lproj/pages/Tools/tools_node.html)):
- Fill, Stroke e largura do traço.
- **Convert**: Sharp, Smooth ou Smart.
- **Action**: Split Curve, Break, Close, Smooth Curve, Join, Reverse.
- **Transform**: Transform Mode (caixa em volta dos nós selecionados), Transform Origin, Alignment Handles, Selection Box From Curves.
- **Snap**: alinhar a nós das curvas selecionadas, encaixar na geometria delas, encaixar vários nós arrastados juntos, alinhar alças pelo snapping global, e "construction snapping".
- **View**: mostrar a orientação da curva (segmento final em vermelho).

**Pen Tool (P)** ([ajuda](https://www.affinity.studio/help/tools-tools-pen/)):
- Mask/Selection (converter a forma), Fill, Stroke e largura.
- **Modos Pen, Smart, Polygon e Line**.
- Conversão Sharp/Smooth/Smart e as mesmas Actions da Node Tool.
- Use line style e Use fill.
- Em Settings: as opções de snapping da Node Tool, "Preserve selection when creating new curves", "Add new curve to selected curves object" e "Rubber band mode" (pré-visualiza o próximo segmento).

**Ferramentas de forma, exemplo Rectangle (M)** ([ajuda](https://www.affinity.studio/help/tools-tools-rectangle/)):
- Fill, Stroke, largura, Presets, Single radius, Absolute sizes.
- **Corner**: Rounded, Straight, Concave ou Cutout, com percentagem.
- Convert to Curves, Transform Objects Separately, Keep selected e Settings.
- Na tela aparecem **alças vermelhas** para editar o raio. Duplo-clique numa alça repõe o valor padrão.

**Artistic Text (T)** ([ajuda](https://www.affinity.studio/help/tools-tools-art-text/)): família, estilo, eixos de fonte variável, tamanho (com slider em pop-up e botões +/−), Change Case, B/I/U/S, sobrescrito/subscrito, cor do texto e do realce, alinhamento e justificação, entrelinha, listas, e atalhos para os painéis Typography, Character e Paragraph e para os estilos de caractere e de parágrafo.

### 1.4 Tools panel (esquerda)

- **Flyouts**: um triângulo no canto inferior direito do ícone indica um grupo de ferramentas. O flyout abre com clique nesse canto ou com clique longo ([ajuda](https://www.affinity.studio/help/panels-tools-panel/)).
- **Sub Tools** (app novo): uma janela flutuante opcional com as ferramentas aparentadas. Por exemplo, ao escolher Rectangle aparecem as outras formas. Pode ser desligada em View > Tools > Sub Tools ([ajuda](https://www.affinity.studio/help/panels-tools-panel/), [Lenscraft](https://lenscraft.co.uk/photography-blog/affinity-3-for-photographers/)).
- **Rich Tool Tips**: a dica pode mostrar a descrição completa da ferramenta ou só o nome ([ajuda](https://www.affinity.studio/help/panels-tools-panel/)).
- **Customize Tools**: arrastar ferramentas de um diálogo com busca, remover arrastando para o documento, reordenar, e **Number of Columns** ([ajuda](https://www.affinity.studio/help/workspace-customizing-tools-panel/)). Uma review fala em "up to eight columns" ([Amateur Photographer](https://amateurphotographer.com/review/affinity-by-canva-review/)). No D2, com duas ou mais colunas aparece também o seletor "sem cor" ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/customizingToolsPanel.html)).
- **Seletores de cor** Fill/Stroke por baixo das ferramentas, com uma seta para trocar ([D2](https://affinity.help/designer2/en-US.lproj/pages/Panels/toolsPanel.html)).
- O painel pode ficar encaixado ou flutuante (View > Tools > Dock) ([ajuda](https://www.affinity.studio/help/workspace-customizing-tools-panel/)).
- **Ordem exata das ferramentas: não confirmada.** O manual lista as ferramentas e os atalhos (§6), mas não a ordem.

### 1.5 Painéis do Studio (vetor)

- **Layers** ([ajuda](https://www.affinity.studio/help/panels-layers-panel/)): opacidade, blend mode, cadeado, visibilidade, "Edit within current container" e etiquetas de cor. Clique direito oferece Hide Others e Show Others. Há opções de tamanho das miniaturas e de auto-scroll até a camada selecionada.
- **Color** ([ajuda](https://www.affinity.studio/help/panels-clr-panel/)):
  - Roda HSL, sliders RGB/HSL/CMYK/LAB, Hex, conta-gotas, cores recentes, opacidade/ruído, acordes de cor.
  - **X** alterna entre os seletores, **Shift+X** troca as cores, **D** volta a preto/branco.
  - **Alt ao escolher cor mantém a opacidade e o ruído** do objeto.
- **Swatches** ([ajuda](https://www.affinity.studio/help/panels-swatches-panel/)): paletas da aplicação, do documento e do sistema, cores globais, spot e overprint, busca, e criação de paleta a partir do documento.
- **Stroke** ([ajuda](https://www.affinity.studio/help/panels-stroke-panel/)): estilos None/Solid/Dash/Path Brush, largura (aceita expressões), cap, join, miter, alinhamento dentro/centro/fora, traço atrás do preenchimento, "Scale with Object", setas e perfil de pressão, e tracejado com "Balanced Dash Pattern".
- **Transform** ([ajuda](https://www.affinity.studio/help/panels-transform-panel/)): seletor de âncora de 9 pontos, X/Y/W/H com cadeado de proporção, R e S (S vira L, comprimento, quando se seleciona uma linha reta), Scale Override e expressões. **Com a Node Tool, o painel opera sobre os nós selecionados.**
- **Appearance** ([ajuda](https://www.affinity.studio/help/panels-appearance-panel/)): **vários preenchimentos e traços** por objeto. O item ativo tem um ponto branco e é ele que os painéis Stroke e Color editam. Cada item tem o seu blend mode e a ordem muda com arrastar e soltar.
- Também existem Symbols (com Sync liga/desliga) ([ajuda](https://www.affinity.studio/help/panels-symbols-panel/)), Constraints ([ajuda](https://www.affinity.studio/help/panels-constraints-panel/)), Assets ([ajuda](https://www.affinity.studio/help/panels-assets-panel/)), Styles ([ajuda](https://www.affinity.studio/help/panels-styles-panel/)) e Navigator com pontos de vista guardados ([ajuda](https://www.affinity.studio/help/panels-navigator-panel/)).
- **Manejo dos painéis** ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)): duplo-clique no rótulo recolhe o grupo, arrastar o rótulo mostra um destaque de onde vai encaixar, **Alt+arrastar impede o encaixe**, os painéis da esquerda e da direita ligam-se separadamente em cada Studio, e há **presets de painéis** (Window > Panels > Add Preset).
- **Painel "Properties": não existe.** A lista de painéis da ajuda tem cerca de 90 entradas e nenhuma se chama Properties ([ajuda: índice](https://www.affinity.studio/help/workspace-interface/)). Esse papel é cumprido pela Context Toolbar.

### 1.6 Campos numéricos

Fonte: [ajuda](https://www.affinity.studio/help/workspace-field-input/).
- Arrastar para a esquerda ou para a direita **sobre o próprio campo** muda o valor (em Opacity e X/Y/W/H).
- Unidades são convertidas: escrever "9 cm" num campo em mm dá 90 mm.
- Roda do rato muda 1 unidade; com Shift muda 10; com Ctrl/⌥ muda 0,1.
- Os campos aceitam expressões ([Stroke](https://www.affinity.studio/help/panels-stroke-panel/)).

## 2. Personas (Designer 2) e Studios (app novo)

### Designer 2

- Três Personas: **Designer** (vetor), **Pixel** (raster dentro do mesmo documento) e **Export** (slices) ([D2](https://affinity.help/designer2/en-US.lproj/pages/Introduction/about_Personas.html)).
- Ao trocar de Persona carrega-se "a unique workspace layout containing the tools and panels… This may also change the menus available" ([D2](https://affinity.help/designer2/en-US.lproj/pages/Introduction/switchingPersonas.html)).
- Os arranjos de painéis são guardados "per Persona". Os **Studio presets** valem para o app inteiro (Window > Studio > Add Preset) ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/customizingWorkspace.html)).

### App novo

- Um **Studio** é um "workspace that groups tools and panels specific to a design discipline… They show in Affinity as Toolbar icons" ([ajuda](https://www.affinity.studio/help/workspace-about-studios/)).
- Studios padrão: Vector, Pixel, Layout e Canva AI. Escondidos por omissão: Slice, Retouching, Color Grading, Typography, Compositing, Astrophotography e Scripting. Liquify, Develop e Tone Mapping abrem a partir de menus ou da barra contextual ([ajuda](https://www.affinity.studio/help/workspace-about-studios/)).
- **O que muda num Studio**: as ferramentas, a arrumação do Toolbar, a posição da Context Toolbar e a visibilidade, agrupamento e posição dos painéis. Tudo é **guardado automaticamente** ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)).
- Pelo clique direito no ícone do Studio há três níveis de reposição: **Update** (gravar o estado atual como padrão), **Revert** (voltar ao padrão gravado) e **Reset to Master** (voltar ao padrão de fábrica) ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)).
- O **Studio Manager** permite criar (clonando outro ou do zero), definir nome, ícone, **cor** e texto da dica, escolher **"fill context"** (Fill/Stroke como no vetor, ou Foreground/Background como no pixel), esconder, reordenar e **exportar/importar `.afstudio`**. Cada Studio novo recebe uma tecla F automática ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)).
- O rótulo pode ser só ícone, ícone e texto, ou **Auto**: o nome só aparece no Studio ativo ([ajuda](https://www.affinity.studio/help/workspace-customizing-studios/)).
- **Trocar de Studio**: botão no Toolbar ou teclas F. No Mac são F1 Vector, F2 Pixel, F3 Layout, F4 Canva AI. No Windows são F2 Vector, F3 Pixel, F4 Layout ([ajuda: atalhos](https://www.affinity.studio/help/workspace-shortcuts-workspace/)).
- **O documento é o mesmo**: o que muda é só o conjunto de ferramentas. Um usuário no HN: "Personas are now studios and as far as I can tell features from all apps are now integrated into one" ([HN floo](https://news.ycombinator.com/item?id=45764848)).

## 3. Detalhes de interação muito elogiados

**Tipos de nó** ([ajuda](https://www.affinity.studio/help/tools-tools-node/), [editar curvas](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/)):
- **Sharp** é uma cúspide. **Smooth** é um Bézier com alças colineares. **Smart** é "a best-fitting curve", com as alças calculadas automaticamente.
- Para converter: botões na Context Toolbar, menu de contexto do nó, Alt+clique (vira Sharp) ou **duplo-clique no nó para alternar sharp/smooth**.

**Edição direta na Node Tool** ([atalhos Vector](https://www.affinity.studio/help/workspace-shortcuts-vector-studio/), [editar curvas](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/)):
- **Arrastar o próprio segmento** muda a curvatura.
- Alt+clique num segmento endireita-o. Ctrl+clique apaga o segmento.
- `[` e `]` passam ao nó anterior/seguinte.
- Backspace apaga o nó mantendo a geometria. Alt+Backspace apaga e reajusta a curva ("Fit to Curve").
- Seleção de vários nós por marquee, Shift ou **laço livre ou poligonal com Alt**. Esses nós podem ser alinhados e distribuídos ([ajuda](https://www.affinity.studio/help/curves-shapes-select-align-nodes/)).
- Duplo-clique numa curva com a Move Tool entra na Node Tool. Duplo-clique fora volta ([ajuda](https://www.affinity.studio/help/tools-tools-move/)).
- Ao arrastar um nó sobre o nó de outra curva, o cursor muda para dois quadrados e a curva **copia as alças** desse nó ([ajuda](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/)).

**Editar enquanto desenha** ([ajuda](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/), [atalhos](https://www.affinity.studio/help/workspace-shortcuts-vector-studio/)):
- Segurar Ctrl/⌘ com a Pen ativa dá a Node Tool temporária.
- Espaço+arrastar reposiciona o último nó.
- **O cursor da Pen indica o resultado do próximo clique**: asterisco = curva nova, + = novo nó, inclinado = canto, círculo = fechar a forma.
- **Ferramentas temporárias**: segurar a tecla da ferramenta enquanto usa e soltar volta à anterior ([atalhos](https://www.affinity.studio/help/workspace-shortcuts-vector-studio/)).

**Snapping** ([ajuda](https://www.affinity.studio/help/design-aids-snapping/)):
- **Presets**: Page layouts, Object creation, Curve drawing, UI design, Pixel work.
- Screen tolerance. Encaixes de página (grid, guias, spread, margens e os pontos médios) e de objeto (bounding box e pontos médios, **gaps and sizes**, key points, geometria e interseções).
- **Alt suspende o snapping** temporariamente. Cada camada pode ter "Exclude From Snapping".
- **Smart guides com cor**: vermelho = alinhamento horizontal, verde = vertical, amarelo = ponto-chave ou geometria. Mostram **a distância em rótulos** ([ajuda](https://www.affinity.studio/help/design-aids-dynamic-guides/)).
- **Snapping candidates** ([ajuda](https://www.affinity.studio/help/design-aids-snapping/)):
  - Os alvos de encaixe são limitados a uma **lista** dos objetos criados, selecionados ou **sob o cursor** mais recentes, com um Maximum.
  - Alternativas: só as camadas imediatas, camadas e filhos, ou todas.
  - Os candidatos mostram um **"purple halo"**.
  - Na V1 este diálogo chamava-se "Snapping Manager". Na V2 e na V3 são as "Snapping options" no Toolbar.

**Snapping de curvas** ([construção](https://www.affinity.studio/help/design-aids-construction-snapping/), [curvas](https://www.affinity.studio/help/design-aids-curve-snapping/)):
- A alça encaixa em linha com o nó vizinho, a 90°, em ângulo espelhado, paralela à alça vizinha, ou num "logical triangle".
- Shift iguala o comprimento das alças, e o encaixe aparece marcado com traços duplos.
- **Curve energy** (setembro de 2026): pentes de curvatura G2. A tecla **crase (`) segurada** mostra-os temporariamente ([ajuda](https://www.affinity.studio/help/design-aids-curve-energy/)).

**Menu de contexto** (clique direito num nó): Sharp/Smooth/Smart, Split Curve After Node, Break, Close, Reverse, Delete Node e Fit To Curve Delete Node ([ajuda](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/)). Com a Move Tool, Ctrl+clique direito lista os objetos aninhados sob o cursor para escolher um ([ajuda](https://www.affinity.studio/help/object-control-select/)).

**"Quick Menu"**: existe **só no iPad**: "gain instant access to clipboard options and nine customizable shortcut operations with a simple three-finger swipe". No iPad há ainda o **Command Controller**, um menu radial de modificadores ([App Store](https://apps.apple.com/us/app/affinity-designer-2-for-ipad/id1616833418)). No desktop não encontrei nada equivalente além do menu de contexto e do Feature Finder.

**Cor sem abrir painel**: as amostras Fill e Stroke da Context Toolbar abrem um pop-up de cor, e as do Tools panel servem de cores ativas ([Move Tool](https://www.affinity.studio/help/tools-tools-move/)).

**Fill Tool (G), gradiente na tela** ([ajuda](https://www.affinity.studio/help/tools-tools-gradient/)):
- Na Context Toolbar: Context (Fill ou Stroke), Type (Linear, Elliptical, Radial, Conical, **Diffusion**, Mesh, Bitmap, Solid), Color em pop-up, Rotate 90°, Reverse, "Anchor to document" e Pigment blend.
- Shift restringe o ângulo a múltiplos de 45°.
- **Como se adicionam ou removem paragens arrastando no gradiente: não confirmado.**

**Corner Tool (C)** ([ajuda](https://www.affinity.studio/help/tools-tools-corner/)): arrastar a partir de um nó desenha um anel com o raio. Tipos Rounded, Straight, Concave e Cutout, com "Bake Appearance". Desde setembro de 2026 dá para **arredondar diretamente com a Node Tool** ([editar curvas](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/), [release notes](https://www.affinity.studio/help/release-notes/)).

**Shape Builder (S)** ([ajuda](https://www.affinity.studio/help/tools-tools-shape-builder/)):
- Três ações: Add, Delete e Create (Create mantém os originais).
- A seleção das áreas candidatas pode ser feita por linha livre, linha reta ou marquee.
- Clean up de geometria e "Use style from first selected area".
- Teclas do numpad: **+ adicionar, − apagar, * criar** ([atalhos](https://www.affinity.studio/help/workspace-shortcuts-editing/)).

**Knife (K)** ([ajuda](https://www.affinity.studio/help/tools-tools-knife/)): corte livre ou em linha reta, com estabilizador Rope ou Window.

**Vector Warp** ([ajuda](https://www.affinity.studio/help/object-control-warp/)):
- Vector > New Warp Group. O warp é **não destrutivo**: o grupo deforma os filhos e eles continuam editáveis.
- Presets Mesh, Quad, Perspective e formas (Arc, Bulge, Flag…).
- Edita-se com a Node Tool. "Mute Mesh" desliga o efeito temporariamente.

**Outros detalhes:**
- **Contour (O)** faz offset do contorno ([ajuda](https://www.affinity.studio/help/tools-tools-contour/)).
- **Point Transform (F)** mostra Δ de posição, escala e rotação no painel Transform ([ajuda](https://www.affinity.studio/help/tools-tools-point-transform/)).
- **Ciclo da caixa de seleção** com a tecla `.`: Base box, Regular bounds ou Planar ([ajuda](https://www.affinity.studio/help/object-control-select/)).
- **Marquee que toca vs. que contém**: configurável, ou Ctrl durante o arraste ([ajuda](https://www.affinity.studio/help/object-control-select/)).
- **Teclas numéricas definem a opacidade**: 4 = 40 %, 43 = 43 % ([atalhos](https://www.affinity.studio/help/workspace-shortcuts-editing/)).
- `[` e `]` mudam a largura do traço ([ajuda](https://www.affinity.studio/help/curves-shapes-edit-lines-and-shapes/)).

**Separated mode e janela única:**
- O **Separated mode** existia na **V1** (Mac): sem moldura, com painéis e documentos soltos ([V1](https://affinity.help/designer/en-US.lproj/pages/Workspace/workspaceModes.html)).
- **Saiu na V2** e foi trocado por documentos destacáveis ("tearable") ([D2](https://affinity.help/designer2/en-US.lproj/pages/Workspace/workspaceModes.html)). No fórum: "they no longer have the separated view, but instead gave the option to 'float' individual documents" ([fde101, fórum](https://forum.affinity.serif.com/index.php?/topic/170297-i-dont-like-the-new-ui-design/)).
- A V3 mantém o mesmo modelo ([ajuda](https://www.affinity.studio/help/workspace-appdoc-windows/)).

**Esconder a UI** ([atalhos](https://www.affinity.studio/help/workspace-shortcuts-workspace/)): **Tab** esconde e mostra a UI. Ctrl/⌘+Shift+H mostra e esconde os painéis. Ctrl+Alt+T / ⌥⌘T mostra e esconde o Toolbar.

**Assistant** ([ajuda](https://www.affinity.studio/help/desktop-settings-assistant/)): decide o que fazer em ações ambíguas (por exemplo, pintar sobre um vetor), avisa com um alerta e grava a sequência como **um único passo de undo**.

**Feature Finder** (setembro de 2026) ([ajuda](https://www.affinity.studio/help/workspace-feature-finder/)):
- É uma paleta de comandos (Ctrl+Alt+Shift+F) que procura pelo caminho de menu completo, em qualquer ordem de palavras.
- Ao escolher um resultado, o elemento correspondente **pisca na tela**, mesmo que o painel esteja recolhido ou noutro monitor.
- O manual sugere procurar com os termos de outros apps.

**Aparência** ([ajuda](https://www.affinity.studio/help/workspace-ui-appearance/), [settings UI](https://www.affinity.studio/help/desktop-settings-user-interface/)): tons de cinza do pasteboard e da UI, contraste do texto, cor de destaque, tema escuro ou claro (3.1+), tamanho das alças das ferramentas e atraso da dica. **"Affinity does not support Whole UI Scaling"**.

## 4. O app unificado: o que mudou e como foi recebido

### O que mudou (fontes oficiais)

Segundo as [novidades](https://www.affinity.studio/help/introduction-new-features/):
- Studios no lugar das Personas, Studios personalizados e importação/exportação deles.
- "Restyled impactful monochromatic iconography".
- "Floating context toolbars can be repositioned or pinned to workspace areas".
- "Cleaner look by hiding more advanced tool options".
- "Unified menus and shortcuts".

Fora das notas oficiais:
- Sub-tools flutuantes, **ícones maiores** e **dicas grandes e explicativas** ([Lenscraft](https://lenscraft.co.uk/photography-blog/affinity-3-for-photographers/)).
- Filtros e ajustes do Photo foram para um novo menu **Pixel**, que tem mais níveis ([Lenscraft](https://lenscraft.co.uk/photography-blog/affinity-3-for-photographers/)).

**Painel "Properties" e "quick actions": não encontrados.** A personalização é feita por Customize Toolbar, Customize Tools e Studio Manager.

### O que os usuários elogiam

- "The UI is clean and modern… the lack of bloat and toggle-able workspaces" e "Everything is customizable… drag and drop buttons and windows just about anywhere" ([Explorers Design, 12/nov/2025](https://www.explorersdesign.com/affinity-impressions/)).
- "being able to float/dock the contextual bar" ([alternativeto](https://alternativeto.net/news/2025/10/affinity-design-platform-is-now-a-free-all-in-one-app-for-photo-vector-and-layout-design)).
- "It's a very elegant implementation… easily switch between all three modalities with a click" ([HN Slow_Hand](https://news.ycombinator.com/item?id=45765848)).
- Dá para esconder o Studio de IA: "You can choose which studios have a visible toggle" ([HN gazook89](https://news.ycombinator.com/item?id=45764669)).

### O que os usuários criticam

- **Ícones monocromáticos**: "a retrograde step as colour icons are nicer to work with" ([Amateur Photographer](https://amateurphotographer.com/review/affinity-by-canva-review/)).
- **A interface parece simplificada demais**: "I don't like the new UI. It feels dumbed down" ([HN internetter](https://news.ycombinator.com/item?id=45762728)). Outro: "not a fan of the new icons… feels a bit less responsive than the v2" ([HN yoz-y](https://news.ycombinator.com/item?id=45765763)).
- **Fusão dos três apps**: "jumble together the vector, bitmap and publishing apps" ([HN rcarmo](https://news.ycombinator.com/item?id=45762874)). Também se perdeu a possibilidade de usar o Photo em tema escuro e o Publisher em claro ao mesmo tempo ([mjtsai](https://mjtsai.com/blog/2025/10/31/combined-affinity-studio-now-free/)).
- **Pixel ou vetor?** "hard to tell when (and why) Affinity Studio is treating certain elements… as pixels and vectors" ([Explorers Design](https://www.explorersdesign.com/affinity-impressions/)).
- **Opiniões divididas**: "confusing", "cluttered" e "overwhelming" contra "cleaner and more efficient" ([Lenscraft](https://lenscraft.co.uk/photography-blog/affinity-3-for-photographers/)).
- **Barra flutuante e sub-tools a sobrepor-se a painéis**, descrito como "a very hardy bug" ([CreoFora, 3/nov/2025](https://creofora.com/forum/index.php?thread%2F154-docking-of-tool-panels%2F=)).
- **Legibilidade e falta de escala da UI em 4K**: "awful in terms of usability and readability" (Ciaran, 15/dez/2025, [Trustpilot](https://ca.trustpilot.com/review/www.affinity.studio)). O próprio manual confirma que não há escala da UI inteira ([ajuda](https://www.affinity.studio/help/workspace-ui-appearance/)).
- **Pedidos que vêm da V2**: personalizar a Context Toolbar, porque "my most used settings are in the overflow", ou pelo menos torná-la rolável ([Gripsholm Lion, fórum 2024](https://forum.affinity.serif.com/index.php?/topic/198031-customiziing-context-toolbar/)). E poder destacá-la: "I can't seem to move the Context Toolbar" ([fórum](https://forum.affinity.serif.com/index.php?/topic/205659-how-do-i-move-the-context-toolbar/)). A V3 resolveu a segunda parte (ela agora flutua), mas **personalizar o conteúdo dela continua sem confirmação**.
- **A estética plana da V2 já tinha sido criticada**: "The flat design feels like a multiplatform wobbly app" (Zaxonov, 17 likes) e "the new UI with one window is a step back" ([fórum nov/2022](https://forum.affinity.serif.com/index.php?/topic/170297-i-dont-like-the-new-ui-design/)).

## 5. Por que preferem o Affinity ao Inkscape (UX)

- **Painéis compactos**: o Inkscape tem campos numéricos "immense with these large minus and plus buttons", enquanto o Affinity cabe "a lot of numeric inputs in a very small space". O painel Fill & Stroke do Inkscape tem "tabs stacked on top of tabs" e esconde a coluna lateral quando ela fica estreita demais. No Affinity, Fill/Stroke alterna com "a tiny little button" ([Virtual Curiosities](https://www.virtualcuriosities.com/articles/3460/comparison-affinity-designer-vs-inkscape)).
- **Ferramentas dedicadas no lugar de efeitos de caminho**: o Corner Tool e o Stroke Width Tool são ferramentas de verdade, enquanto no Inkscape são "Live Path Effects" ([Virtual Curiosities](https://www.virtualcuriosities.com/articles/3460/comparison-affinity-designer-vs-inkscape)).
- **Estabilidade e polimento**: "Inkscape is a total mess to use" ([HN](https://news.ycombinator.com/item?id=45762726)). "Inkscape and GIMP have always felt pretty rough around the edges" ([HN](https://news.ycombinator.com/item?id=45764707)). "v2… still beats GIMP/Inkscape… in at least UX" ([HN](https://news.ycombinator.com/item?id=45764168)). "the most crashy software I've used" ([HN](https://news.ycombinator.com/item?id=45763820)). Houve contestação no mesmo fio: "I use Inkscape every day on macOS and it runs just fine" ([HN](https://news.ycombinator.com/item?id=45763698)).
- **Comunidade Glowforge**: "I abandoned Inkscape for AD 4 years ago and have never looked back" e "working with fonts in AD feels like a radically better experience" ([Glowforge](https://community.glowforge.com/t/inkscape-or-affinity-designer/101170)).
- **Resumo de review**: "sleek and modern interface… more user-friendly than Inkscape", contra o Inkscape "a bit cluttered" (resumo de busca das comparações [LogosByNick](https://logosbynick.com/inkscape-vs-affinity-designer/), que bloqueou a leitura direta). A [stackshare](https://stackshare.io/stackups/inkscape-vs-affinity-designer) diz que o Inkscape é "functional but dated — non-remappable keys, clunky toolbars".

## 6. Atalhos das ferramentas vetoriais (app novo, Studio Vector)

Fonte: [atalhos Vector](https://www.affinity.studio/help/workspace-shortcuts-vector-studio/).

- **V** Move · **A** Node · **P** Pen · **N** Pencil · **B** Path/Vector Brush · **G** Fill (no D2 chamava-se Gradient Tool) · **Y** Transparency · **C** Corner · **O** Contour · **K** Knife · **S** Shape Builder · **F** Point Transform · **L** Stroke Width · **W** Blend · **R** Vector Flood Fill · **I** Color Picker · **H** View · **Z** Zoom
- **U** percorre Rectangle, Ellipse e Rounded Rectangle. A página do Rectangle diz **M** nos Studios Vector e Layout e U no Pixel ([ajuda](https://www.affinity.studio/help/tools-tools-rectangle/)), por isso há uma inconsistência no próprio manual. **T** alterna entre Artistic e Frame Text.
- Opção "Use Shift modifier to cycle tools", para evitar alternar sem querer entre ferramentas que partilham a tecla.

Edição geral ([atalhos](https://www.affinity.studio/help/workspace-shortcuts-editing/)):
- Ctrl+J duplica. Ctrl+Enter converte em curvas. Ctrl+G / Ctrl+Shift+G agrupa e desagrupa.
- Alt+clique seleciona o objeto que está por baixo.
- Shift+arrastar restringe o movimento. Shift na rotação usa passos de 15°.
- Ctrl+arrastar redimensiona a partir do centro.
- `;` liga e desliga o snapping. Ctrl+Y alterna wireframe/X-ray. Tab esconde a UI.
- Ctrl+Shift+V cola só o estilo ("Paste Style").

---

## Padrões transferíveis (o que copiar)

1. **Uma barra contextual única** cujo conteúdo depende da ferramenta e do tipo de seleção (por exemplo, forma paramétrica ou curva). As amostras **Fill e Stroke abrem um pop-up de cor ali mesmo**, ao lado da largura do traço e de um botão "editar traço".
2. **Opções avançadas num botão "Settings"** dentro da barra contextual. A barra fica curta e as opções de snapping de nós, por exemplo, continuam a um clique.
3. **Barra contextual com posições âncora** (topo, base e cantos da área de desenho), que também se pode encaixar. Isto resolve o pedido de anos dos usuários sem criar uma barra solta em qualquer lugar.
4. **Workspaces por modo com troca por tecla F**. Cada workspace guarda sozinho o seu arranjo e tem três níveis de reposição (gravar como padrão, voltar ao padrão, voltar à fábrica). Para o nosso app, o equivalente é o modo Object/Edit/Pose.
5. **Workspaces exportáveis** (um ficheiro), com **cor e ícone** próprios para se saber em que modo se está.
6. **Painel de ferramentas** com flyouts (triângulo no canto ou clique longo), uma **barra de sub-ferramentas** opcional para a família ativa, 1 a N colunas, um diálogo de personalização com busca, e **dicas ricas** (nome e frase do que a ferramenta faz).
7. **Barra de status com a dica da ferramenta e dos modificadores disponíveis** ("Alt: canto, Shift: 45°…"), atualizada pela ferramenta e pelo estado em que o cursor está.
8. **Cursores que dizem o resultado do próximo clique** (novo nó, fechar a forma, continuar a curva, canto).
9. **Nós Sharp, Smooth e Smart**. O Smart calcula as alças sozinho. Duplo-clique no nó alterna o tipo, Alt+clique faz canto, Alt+clique num segmento endireita, e arrastar o segmento muda a curvatura.
10. **Ferramenta temporária**: segurar Ctrl/⌘ na Pen dá a Node Tool. Segurar a tecla de qualquer ferramenta e soltar volta à anterior. Duplo-clique alterna entre mover e editar nós.
11. **Snapping com feedback visível**: guias coloridas (vermelho = horizontal, verde = vertical, amarelo = ponto), **rótulo com a distância**, setas para espaços iguais, e **lista de candidatos** limitada aos objetos recentes ou sob o cursor, destacados com um halo. Alt suspende o encaixe. Presets de snapping por tipo de trabalho.
12. **Snapping de construção para alças** (colinear, 90°, espelhada, paralela) e Shift para igualar os comprimentos, com marcas visuais. Opcionalmente, pentes de curvatura numa tecla segurada.
13. **Campos numéricos** em que se arrasta sobre o rótulo para mudar o valor, com roda do rato (Shift = ×10, Ctrl = ×0,1), expressões e unidades convertidas. **Sem botões +/−**.
14. **Painel Transform** com âncora de 9 pontos, que opera sobre os nós quando se está a editar nós. Uma linha reta mostra o comprimento (L) no lugar do cisalhamento.
15. **Vários preenchimentos e traços por objeto** (Appearance), com o item ativo marcado. Os painéis Color e Stroke editam esse item.
16. **Tab esconde toda a UI.** Atalhos separados para os painéis e para o Toolbar.
17. **Uma paleta de comandos que mostra onde o comando mora**, fazendo piscar o botão ou o painel na tela. Ensina a interface em vez de só executar.
18. **Comandos frequentes no topo**: booleanas, alinhar (com "alinhar a" seleção, página, primeiro ou último), e **onde nasce o objeto novo** (atrás, no topo, dentro).
19. **Warp e cantos não destrutivos**: o efeito fica no grupo ou no nó e há um "Bake" explícito.

## Armadilhas (o que os usuários odeiam)

1. **Ícones monocromáticos e iguais entre si**: a mudança da V3 foi chamada de retrocesso. Use forma bem distinta e, se possível, uma cor por família.
2. **Opção usada todos os dias escondida no overflow** da barra contextual, sem forma de reordenar ou rolar. Garanta que a barra tenha prioridade definida e rolagem.
3. **Barra flutuante a tapar o desenho ou a sobrepor-se a painéis**. A posição âncora tem de evitar colisões e encaixar sempre.
4. **Interface sem escala própria**: ilegível em 4K. Nós temos escala da interface e ela tem de ser mantida.
5. **Modo ambíguo**: o usuário não sabe se o objeto é pixel ou vetor, ou em que Studio está. O modo tem de estar sempre visível.
6. **Fundir modos demais num só lugar** sem deixar personalizar cada um. Os usuários perderam "um tema por app".
7. **Mexer na posição de comandos conhecidos** (ex.: o novo menu Pixel com mais níveis) obriga a reaprender. Mudanças de layout pedem um caminho de migração, e uma paleta de comandos que aceita os nomes antigos ajuda nisso.
8. **Visual plano com controles pouco visíveis** (crítica de 2022 à V2): botões e sliders precisam de contraste e de área de clique evidente.
9. **Painéis com abas sobre abas e controles +/− enormes** (a crítica ao Inkscape). Densidade sem desperdício vertical.
10. **Rótulos que mudam de significado entre modos** (Fill/Stroke vira Foreground/Background). O Affinity torna isto explícito como "fill context" do Studio. Se fizermos o mesmo, o rótulo tem de mudar junto.
11. **Personalização que não se consegue guardar ou que some**: há queixas de Studio personalizado perdido (fórum, V2.5.5) e de "you can't save a custom studio workplace" (Trustpilot, resumo de busca). Gravação automática com reposição fácil é obrigatória.

**O que ficou por confirmar:** a ordem exata do painel de ferramentas; como se adicionam ou removem paragens de gradiente na tela; se a Context Toolbar da V3 permite escolher os itens; a existência de "quick actions" ou de um painel Properties (não encontrados, provavelmente não existem); e as reações no Reddit, a que não tive acesso.
