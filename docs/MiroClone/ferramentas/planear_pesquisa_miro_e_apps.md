# Planear — mapa mental e modelos: Miro · FigJam · Whimsical · XMind · MindNode (só documentação oficial)

- **Data:** 2026-10-08
- **Método:** leitura de documentação OFICIAL apenas; nenhum código-fonte de app lido.
  - **Miro (help):** os 652 artigos `en-us` do help center baixados pela API Zendesk
    (`https://help.miro.com/api/v2/help_center/en-us/articles.json?per_page=100&page=1..7`), convertidos
    em texto e varridos por regex (`mind ?map|mindmap|template|shortcut`). O artigo central é
    «Mind map» (360017730753, editado em 2026-09-15). Os GIFs desse artigo (17 anexos) foram
    decompostos em quadros (`magick -coalesce`) e OLHADOS: o que vem só de um quadro está marcado
    **[GIF oficial]** — é imagem do help, não texto.
  - **Miro (developers):** Web SDK `MindmapNode` (experimental), REST «Create mind map node»
    (experimental; o OpenAPI vem embutido na página readme.io) e a página «Mind map (Experimental)».
  - **Miro (modelos):** páginas `miro.com/templates/<slug>/` dos modelos de autoria Miro (perfil
    «Miro»); o conteúdo de cada modelo só aparece na IMAGEM de pré-visualização (ctfassets) — descrita
    a olho, marcada **[imagem oficial]**. Os números de notas/colunas são contagem nessa imagem.
  - **Comunidade** (community.miro.com) marcada **[comunidade]**; «Mironeer» = funcionário Miro;
    «Volunteer Community Moderator» = NÃO funcionário.
  - **Outros apps:** help.figma.com (API Zendesk), whimsical.com/learn, xmind.com/user-guide,
    mindnode.com/article. Excalidraw não repetido (só Ctrl+seta, já registado).
- **Leitura da coluna «lei»:** «silente» = a documentação não diz; nada foi adivinhado. Onde uma lei é
  INFERÊNCIA (a partir do texto ou de uma imagem), está escrito «inferência».

---

## 1. Miro — mapa mental

### 1.0 Dois mapas mentais?

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 0 | O help distingue um mapa «antigo» e um «novo»? | **silente** — o artigo único descreve UM mapa mental (com auto layout, tipos de nó, recolher). Não há «classic»/«new» no help nem na busca da comunidade | — | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 0 | Cronologia documentada (o que separa o «antigo» do actual) | **recolher/expandir** lançado em out/2022 (Product News, conta Miro); **interruptor de auto layout** anunciado em fev/2023; antes disso **[comunidade, 2021]** o auto layout NÃO se desligava | «You can now view high level concepts or zoom in on critical details of your mind map by collapsing and expanding branches» (31/10/2022) · «With auto-layout, you can easily create professional-looking mind maps with a single click. Toggle on to organize and lay out your mind map» (02/02/2023) · «till now there is no option to do this with a MindMap you created in miro» | https://community.miro.com/product-news-31/3-most-voted-community-wish-collapse-and-expand-mind-map-branches-now-live-10379 · https://community.miro.com/product-news-31/unlock-the-full-potential-of-your-ideas-with-auto-layout-11943 · https://community.miro.com/ask-the-community-45/how-do-i-switch-off-auto-layout-4316 |
| 0 | API experimental (2023) | só horizontal e só auto layout; vertical, cotovelo/curva e posição livre «em avaliação» (o produto já os tem — o SDK ficou atrás) | «Mind map theme orientation: Horizontal orientation, from left to right» · «Positioning: Auto-layout positioning» · futuro: «connector shape (elbowed/curved)… Addition of the vertical orientation… Free positioning of nodes» | https://developers.miro.com/docs/mind-maps |

### 1.1 Criar

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 1 | Ferramenta | «Mind map» na Creation toolbar; clicar no quadro cria o **nó-pai** (raiz) | «Select Mind map and click anywhere on the board to add the parent node - the starting point of your mind map.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 1 | Atalho da ferramenta | **silente** — o artigo «Shortcuts and hotkeys» não tem entrada de mind map (só «2 always opens the templates modal») | — | https://help.miro.com/hc/en-us/articles/360017731033-Shortcuts-and-hotkeys |
| 1 | Com que forma/texto nasce a raiz | texto: **silente**. **[GIF oficial]** nasce **pílula** de contorno roxo, vazia, com o placeholder «Type something», já em edição, e **quatro «+»** (cima, baixo, esquerda, direita) | «The plus signs on each side of it will prompt where you can start branches and create child nodes. You can create a vertical or a horizontal mind map.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map (anexo `starting_a_mindmap.gif`) |
| 1 | O «+» escolhe a orientação | **inferência** do texto + **[GIF oficial]**: o «+» da direita cria filho à direita e o mapa fica horizontal; os de cima/baixo dão o vertical | «Creating a parent node and starting a horizontal mind map» (legenda) | idem |
| 1 | Pela IA | «Generate mind map» a partir da raiz preenchida; resultado «around your root node», com keep/retry/discard | «Your mind map will be placed on the board around your root node. You can optionally keep, retry, or discard your mind map.» | idem |

### 1.2 Teclas

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 2 | Tab | cria **filho** | «Tab to create a new child node» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 2 | Enter | cria **irmão** e entra logo em edição. **Antes ou depois do actual: silente.** Só em edição? **silente** | «Return to create a sibling node and start editing it right away» | idem |
| 2 | Shift+Enter | quebra de linha DENTRO do nó (não cria nó) | «Shift + Return to break lines inside a node» | idem |
| 2 | Ctrl/Cmd+Enter | nova linha que aceita lista (dentro do nó) | «create a new line with Cmd + Enter (for Mac) or Ctrl + Enter (for Windows)» | idem |
| 2 | Setas | navegam entre nós; **qual seta vai para pai/filho/irmão: silente** | «Arrows to move between nodes» | idem |
| 2 | Delete | apaga o nó **com todos os filhos** (o ramo inteiro; os filhos NÃO sobem) | «Delete to delete the whole node» · «The node will be deleted along with all the child nodes.» | idem |
| 2 | Backspace | **silente** (só «Delete» é citado) | — | idem |
| 2 | Esc | sai da edição | «Escape to exit the editing mode» | idem |
| 2 | Ctrl+Z | **silente** no artigo (desfazer genérico do quadro) | — | idem |
| 2 | Ctrl/Cmd+C/V | duplica um nó | «Ctrl + C and Ctrl + V … to duplicate a node» | idem |
| 2 | Ctrl/Cmd (segurar) | ao arrastar, **move sem trocar de pai** | «Ctrl (on Windows)/Cmd (on Mac) (hold) to stop auto-reassign» · «Hold down Ctrl … if you'd like to move the node instead of reassigning it.» | idem |
| 2 | (comunidade) | **[comunidade, moderador voluntário]** confirma Enter = irmão, Tab = filho | «Enter/Return will create a new sibling node and Tab will create a new child node.» | https://community.miro.com/ask-the-community-45/shortcut-key-to-add-node-to-mind-map-17155 |

### 1.3 Layout

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 3 | Direcções | **horizontal** (filhos à esquerda e/ou à direita da raiz) e **vertical** (em cima e/ou em baixo). Nada de «só para baixo» nem 4 lados | SDK: «When the layout is `horizontal`, `start` represents the **left** and `end` represents the **right** side… When the layout is `vertical`, `start` represents the **top** and `end` represents the **bottom**» · **[comunidade, moderador voluntário]** «the native mind map object just allows child nodes to be created on either side but not on all four sides» | https://developers.miro.com/docs/mindmap_mindmapnode · https://community.miro.com/ask-the-community-45/how-to-have-the-nodes-for-the-mindmap-on-all-4-sides-not-just-two-7516 |
| 3 | Onde se escolhe | no estilo do mapa («change the orientation»); o `layout` só se define na RAIZ. **[GIF oficial]** botão na barra do nó-raiz com 2 orientações + 2 linhas (curva, cotovelo) | «You can choose the style for your mind map (the nodes can either use curved or straight lines) and change the orientation.» · SDK: «Layout can **only** be set for the root mind map node.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map (`changing_mind_map.gif`) · https://developers.miro.com/docs/mindmap_mindmapnode |
| 3 | Trocar de lado (horiz.) | **[comunidade, moderador voluntário]** mudar horizontal↔vertical existe; passar nós da esquerda para a direita só à mão | «While you can change the orientation from horizontal to vertical placement, there is no way to swap nodes from the left to the right side of the root node other than doing it manually.» | https://community.miro.com/ask-the-community-45/switch-orientation-of-mindmap-10315 |
| 3 | Auto layout | **ligado por omissão**; liga/desliga **no nó-PAI** (menu de contexto). **[GIF oficial]** o interruptor «Auto» aparece na barra do nó-raiz | «all child nodes will automatically align. This is because auto layout is turned on by default.To turn auto layout on or off, click the parent node, and in the context menu toggle auto layout.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 3 | Com auto layout desligado | comando pontual «Layout nodes» no «…» do pai | «click the parent node, then click the three dots (...) in the context menu and select Layout nodes.» | idem |
| 3 | Escopo (pai vs mapa) | texto diz «parent node»; **se o interruptor é por sub-árvore ou do mapa inteiro: silente**. **[GIF oficial]** só se vê na raiz | (citação acima) | idem |
| 3 | Arrastar com auto layout | arrastar = **troca de pai** (reassign); Ctrl/Cmd segura = só move. **[GIF oficial]** soltar sobre outro nó insere-o entre os filhos desse nó, na posição do largar | «To reassign a child node to another parent node, simply drag the node.» | idem (`image1.gif`, `reassign_nodes.gif`) |
| 3 | Inserir nó entre dois | **não há comando**: o próprio help ensina o contorno (reatribuir, criar, reatribuir de volta) | «Temporarily reassign one of the nodes to another one, create a new node, reassign the node back.» | idem |
| 3 | Espaçamentos (números) | **silente**. **Inferência** medida na captura oficial «Miro Mind Map» a 100 %: passo vertical entre irmãos-folha ≈ **54 px**; distância horizontal entre o texto do pai e o do filho ≈ **110–120 px** | — | https://miro.com/templates/mind-map-basic/ (imagem `Mind_Map_View-web.png`) |
| 3 | Alinhamento dos filhos | **silente** no texto. **Inferência** da mesma captura: cada pai fica **centrado verticalmente na altura total das suas sub-árvores** (a raiz em y=480 é o meio de 183…778 do lado esquerdo; «Use cases» y=345 é o meio de 183…507) | — | idem |
| 3 | Tamanho | arrastar o ponto branco escala o MAPA inteiro; a fonte da raiz escala todo o mapa | «You can also change your mind map's size by selecting it and dragging the white dot.» · «changing the text size in the parent node will affect the whole mind map.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 3 | Não roda / não agrupa / não tranca | lista oficial | «certain objects (Kanban, mind map, tables) can't be rotated.» · «Mind map nodes can't be locked and grouped.» | https://help.miro.com/hc/en-us/articles/360017572314-Switch-type · artigo Mind map |
| 3 | Selecção por laço | mapa mental só entra se ≥ 90 % coberto | «excluding USM, Kanban, mind map, and tables - to select these objects, cover at least 90% by the selection field» | https://help.miro.com/hc/en-us/articles/360017730953-Working-with-objects |

### 1.4 Lado em que nasce o filho da raiz (modo dos dois lados)

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 4 | Pelo teclado (Tab na raiz) | **silente** (alterna? equilibra? — nada dito) | — | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 4 | Pelo «+» | o lado é o do «+» clicado (há um em cada lado) | «The plus signs on each side of it will prompt where you can start branches» | idem |
| 4 | Pela API | o lado é decidido pela **posição inicial** do filho em relação à raiz (esquerda→esquerda, direita→direita); `direction` só se define nos filhos directos da raiz | «If the initial position of the child node is to the left of the root mind map node, the child node is attached to the left of the root node.» · «`direction` property can only be set for immediate children of the root node.» | https://developers.miro.com/docs/mindmap_mindmapnode |
| 4 | Equilíbrio automático | **não documentado em parte nenhuma**; a captura oficial tem **3 à esquerda e 2 à direita** (inferência: não equilibra por contagem) | — | https://miro.com/templates/mind-map-basic/ |

### 1.5 Recolher / expandir

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 5 | Como | pairar sobre o ramo e clicar no **«−»** | «To collapse, hover over the branch and click the minus ( - ) button. Any comments attached to that branch will also be hidden.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 5 | O que se vê recolhido | um **número** = nós + comentários escondidos; clicar no número expande. **[GIF oficial]** é uma pastilha na cor do ramo | «After collapsing a branch, you'll see a number that indicates how many nodes and comments are hidden. To expand the branch again, simply click the number.» | idem (`Expand_and_collapse.gif`) |
| 5 | Persiste? | **silente** (inferência: é estado do objecto, mas nada o diz) | — | idem |
| 5 | O mapa encolhe ao recolher? | **[comunidade, 2022]** NÃO: o resto fica onde estava; o contorno é «Layout nodes» na raiz | «Right now, if one collapses a node, the overall mind map stay at it is, it does not shrink to a more synthesized view» | https://community.miro.com/ideas/mind-map-align-child-nodes-at-the-root-level-automatically-when-collapsing-a-node-10666 |
| 5 | Atalho | **silente** | — | — |

### 1.6 Estilo

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 6 | Cor por ramo | existe cor de RAMO (o nó com forma é «filled with the same color as the branch»); botão **Randomize** colore. **[GIF oficial]** Randomize dá uma cor a cada ramo de 1.º nível; sem ele, o mapa nasce todo roxo | «if you want your mind map to look colorful and bright, use Randomize tool.» · «The style will be applied and the node will be filled with the same color as the branch» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map (`changing_mind_map.gif`) |
| 6 | Forma dos nós | 4 tipos: **None** (só texto), **Pill**, **Rounded rectangle**, **Rectangle**; cor e opacidade de preenchimento por nó. **[GIF oficial]** o filho «None» é texto com a linha do ramo a passar por BAIXO dele (sublinhado) | «There are four visual styles of node: None - no shape is applied · Pill - an oval pill-shape · Rounded rectangle … · Rectangle» | idem · REST enum `pill, rectangle, rounded_rectangle, none`: https://developers.miro.com/reference/create-mindmap-nodes-experimental |
| 6 | Tipo de linha | **curva** ou **recta/cotovelo** | «the nodes can either use curved or straight lines» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 6 | Espessura que afina com a profundidade | **silente** (nas capturas oficiais todos os ramos parecem da mesma espessura — inferência) | — | — |
| 6 | Fonte | não se troca a família; tamanho só pela raiz (escala tudo); REST: 10–288 dp, omissão 14 | «Currently, it isn't possible to change the font.» · «you can only change the size of the parent node» · REST: «Default: `14`.» (`minimum":10`, `maximum":288`) | artigo Mind map · REST acima |

### 1.7 Mover, converter, colar

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 7 | Mover ramo para outro pai | arrastar (reassign); Ctrl/Cmd = mover sem reatribuir; no tablet também | «Can I move/reassign a node? Yes, simply drag the node.» · «Can I reassign mind map nodes on tablet? Yes, simply drag the node to another parent node.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map · https://help.miro.com/hc/en-us/articles/360017731633-Tablet-app |
| 7 | Mover o mapa inteiro | arrastar a raiz | «Select the parent node and move it.» | artigo Mind map |
| 7 | Nota ↔ nó | **não**: notas/textos/cartões não viram nós (só copiar o texto). Nó → nota: **silente** | «Can I convert sticky notes, text blocks, cards, and other widgets into mind map nodes? No, it isn't currently possible.» | idem |
| 7 | Colar lista indentada vira mapa | **silente** para texto. Colar um mapa do **Mural** vira mapa Miro (raiz + filhos), ordem pode mudar | «Mind maps from Mural copy-paste as Miro Mind Maps, including the root node, each child node, and their text.» · «the node order may change when pasted into Miro.» | https://help.miro.com/hc/en-us/articles/22957521683986 |
| 7 | Vários pais | não; contorno = linha de conexão | «As a workaround, you can use connection lines to link a child node to several parent nodes.» | artigo Mind map |
| 7 | Imagem no nó | não (backlog) | «there is no option to attach an image to a node, we have it in the backlog.» | idem |

### 1.8 Limites

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 8 | Número de nós | **sem limite** publicado | «There is no limit to the number of nodes (parent or child) that you can build.» | https://help.miro.com/hc/en-us/articles/360017730753-Mind-map |
| 8 | API | REST cria sem `position` → (0,0) do quadro; filho reposicionado em relação ao pai (`relativeTo: parent_center`) | «If the position is not provided in the request, nodes default to coordinates x=0, y=0, effectively placing them at the center of the board.» | https://developers.miro.com/reference/create-mindmap-nodes-experimental |

---

## 2. Miro — modelos

### 2.1 Mecânica de inserção

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| M1 | Onde abre a galeria | ícone **Templates** da toolbar; **tecla 2**; também dentro das ferramentas frame/nota/tabela/mapa mental | «To add a template to your existing board… choose the Templates () iconon the toolbar.» · «For example, 2 always opens the templates modal.» · «Templates are built into frames, sticky notes, tables, and mind maps» | https://help.miro.com/hc/en-us/articles/360017572134-Templates · https://help.miro.com/hc/en-us/articles/360017731033-Shortcuts-and-hotkeys |
| M2 | Galeria | busca (inclui custom), categorias, etiquetas, destaques, pré-visualização; **cheio de exemplo OU vazio** | «Use search to easily find a template…» · «Browse top use cases using categories, topic tags, and featured templates.» · «Decide if you'd like to start with pre-filled example content (available on some templates) or a blank template.» | Templates |
| M3 | Onde nasce no quadro | **help silente** (centro da vista? cursor? — nada). **[comunidade, 2022, conta Nick_Miro; a página mostra o selo Mironeer]** nasce **por cima** (ordem z) do que já existe | «in Miro all objects (except frames ) are created in this way. The "older" an object the more "behind" it is.» | https://community.miro.com/ask-the-community-45/new-template-placement-on-the-board-9395 |
| M4 | Agrupado? frame com o nome? | **desagrupado** ao inserir; «frame com o nome do modelo» **silente** (o help sugere ao utilizador criar um frame à volta) | «When the template is added, the elements are automatically ungrouped… If you'd like to move more than one object at once, you can group them or create a frame around them.» | Templates |
| M5 | Fica seleccionado? | **silente** no help. **[comunidade]** no GIF do funcionário «The entire template is already selected» (não diz se é automático) | (citação) | thread 9395 |
| M6 | Escala | **silente** | — | — |
| M7 | Limpar o conteúdo de exemplo | «Clear content» / Ctrl+Delete mantém cores e tamanhos | «Alternatively, use the shortcut Cmd + Delete (Mac) or Ctrl + Delete (Windows).» | Templates |

### 2.2 Conteúdo dos 6 modelos (autoria «Miro»; descrição a partir da IMAGEM oficial)

| # | modelo | elementos | citação / fonte | URL |
|---|---|---|---|---|
| T1 | **Start, Stop, Continue** (retrospectiva) | **[imagem oficial]** 4 áreas brancas lado a lado com título por cima — «Start», «Stop», «Continue», «Discussion and action items» (inferência: frames, pelo título fora da caixa); dentro, uma pergunta em texto («What should we start doing?» / «…do differently?» / «…keep doing?»); grelha 3 colunas de notas: **11 azuis/lilás**, **11 vermelhas-rosa**, **11 verdes**; à esquerda um cartão de instruções («Start, Stop, Continue» + texto + botões Private mode, Timer); à direita widgets de votação e pessoas, e «Agile coach» (sidekick) | «Start: Identify new practices… Stop: … Continue: …» | https://miro.com/templates/start-stop-continue-basic/ |
| T1b | **Mad Sad Glad** | **[imagem oficial]** cartão de instruções («Mad, Sad, Glad» + «Conduct a retrospective on understanding the emotions and feelings of the team.» + Private mode, Timer) e **3 colunas altas** brancas; em cada uma um cartão-etiqueta com UMA nota (**amarela** Mad, **vermelha-rosa** Sad, **verde** Glad) e a pergunta «What made us mad?/sad?/glad?» | — | https://miro.com/templates/mad-sad-glad-retrospective/ |
| T2 | **Brainwriting** | **[imagem oficial, capa]** grelha de notas sem frame: linha 1 = «Participant 1…6» (uma cor por participante: amarelo-claro, amarelo, laranja, rosa, vermelho, lilás), linhas seguintes = ideias («Idea #1», «Idea #2 Improvement»…); 6 colunas × 4 linhas visíveis | «Using the brainwriting template, you can boost group participation and stimulate new ideas.» | https://miro.com/templates/brainwriting-basic/ |
| T3 | **Kanban Framework** | **[imagem oficial]** objecto Kanban com **3 colunas** «Backlog | 4», «In progress | 0», «Done | 0» (contador ao lado do título; sublinhado cinza/amarelo/verde-lima), **2 raias** «Team A | 2», «Team B | 2»; cartões «Type something» com barra colorida (amarelo, lilás); «+» por célula | «A typical Kanban board has three columns: backlog, in progress, and done.» | https://miro.com/templates/kanban-framework/ |
| T4 | **Mind Map** | **[imagem oficial]** raiz pílula «Miro Mind Map»; 1.º nível de cada lado (esq.: Use cases, More than a mind map, Availability; dir.: Robust collaborative features, Mind mapping features); nós «None» (texto); ramos **curvos**, **uma cor por ramo de 1.º nível** (verde-água, roxo, magenta, laranja, violeta) | «Branch out with sub-topics, add keywords» | https://miro.com/templates/mind-map-basic/ |
| T5 | **Flowchart** | **[imagem oficial actual]** Start (azul) → Process (amarelo) → Process (amarelo) → **Decision** (losango preto) → «Yes» → 2 Process lilás → End (azul); «No» → Process laranja → Process laranja → volta ao 2.º Process; setas em cotovelo com rótulos Yes/No. O TEXTO da página descreve a versão anterior (pretos arredondados = início/fim, amarelos, losango laranja, rosas) | «The black rectangles with rounded corners represent the starting and ending points… the diamond represents a decision… the arrows between the shapes» | https://miro.com/templates/simple-flowchart/ |
| T6 | **Customer Journey Map** | **[imagem oficial]** zona «Persona» (cartão de instruções + «Customer profile»: imagem, notas Name/About me lilás, Needs/Pain points amarelas, Goals/Scenario verdes) e zona «Journey Map»: tabela **4 estágios** (Stage 1…4) × linhas **Story, Actions, Touchpoints, Emotions, Pain points, Backstage, Opportunities & owners**; Actions com 3 notas por estágio numa cor por estágio (lilás, amarela, verde, laranja), Touchpoints brancas, Emotions um emoji, Pain points azuis, Opportunities cartões contornados | «By mapping out the customer's Story, Actions, Touchpoints, Emotions, and Pain Points for each Stage of their journey» | https://miro.com/templates/customer-journey-map-basic/ |
| — | Tamanho dos modelos | **silente** em todas as páginas (nenhuma publica dimensões) | — | — |

---

## 3. FigJam

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| F1 | Criar | Shapes > Mind maps, clicar no quadro; ou de um texto seleccionado; ou IA | «Click Shapes > Mind maps. Click the board to place you mind map.» | https://help.figma.com/hc/en-us/articles/18917944627095-Create-mindmaps-in-FigJam |
| F2 | Filho / irmão | **Cmd+Return** = filho; **Cmd+Shift+Return** = irmão (NÃO Tab/Enter); «+» na direcção desejada | «Use ⌘CommandReturn to quick create a new child node.» · «Press ⌘Command⇧ShiftReturn to add a sibling node.» | idem |
| F3 | Nós | texto, forma ou nota; arrastar objecto sobre um nó liga-o | «mind maps are composed of nodes, which are the text fields, shapes, or stickies» | idem |
| F4 | Layout / lados / recolher / apagar ramo | **silente** | — | idem |
| F5 | Modelos | inserir pelo seletor; em secções há «Templates» junto ao título; onde nasce: **silente** | «Hover over a template and click Insert template.» | https://help.figma.com/hc/en-us/articles/1500004414082-Find-and-use-FigJam-templates |

## 4. Whimsical

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| W1 | Criar | toolbar ou tecla **M** | «Use the keyboard shortcut M» | https://whimsical.com/learn/get-started/mind-maps |
| W2 | Tab / Enter | Tab = filho, Enter = irmão, **Cmd+Enter = irmão ACIMA**, Option+Enter = novo pai, Shift+Enter = quebra de linha | «Add child Tab · Add sibling Enter · Add sibling above Command + Enter · Add parent Option + Enter · Add line break Shift + Enter» | https://whimsical.com/learn/shortcuts/mac |
| W3 | Recolher | **Cmd+/** | «Collapse/Expand node Command + /» | idem |
| W4 | Layout | orientação e linha curva/recta na toolbar; arrastar reordena; lados/equilíbrio: **silente** | «Curved or straight lines … The orientation of the mind map» · «Drag and drop the nodes to rearrange them» | https://whimsical.com/learn/get-started/mind-maps |
| W5 | Colar lista | **colar uma lista na raiz vira mapa** | «Just copy a list from somewhere else, then select a mind map's starting node and paste with command (or ctrl ) + V» | idem |
| W6 | Delete / setas | **silente** | — | — |

## 5. XMind

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| X1 | Tab / Enter | Tab = subtópico, Enter = irmão; «+» ao lado (filho) e **por baixo** (irmão) | «You can also press Tab to add a subtopic.» · «Click the + button below the topic where the sibling topic will be added.» · «You can also press Enter to add a sibling topic.» | https://xmind.com/user-guide/add-topic-button |
| X2 | Estruturas | Mind Map, Logic Chart, Brace Map, Org Chart, Tree Chart, Tree Table, Timeline, Fishbone, Matrix; misturáveis por ramo | «Multiple structures can coexist in the same mind map» | https://xmind.com/user-guide/structure-new · https://xmind.com/structure |
| X3 | Equilíbrio esq./dir. | **«Auto Balance Map»** opcional, por **ALTURA** de conteúdo (não por contagem) | «Automatically adjusts unbalanced content heights on either side of the mind map for structural balance.» | https://xmind.com/user-guide/map-style-new |
| X4 | Espaçamento | «Compact Map», «Justify Topic Alignment»; números: **silente** | «Compacts the mind map layout to minimize space usage.» | idem |
| X5 | Posição livre | «Branch Free-Positioning» opt-in (só estrutura Mind Map) | «Enables free movement of mind map branches for personalized layouts.» | https://xmind.com/user-guide/advanced-layout-new |
| X6 | Recolher | botão ao pairar; **Ctrl+/**; recolhido mostra o **número** de subtópicos, clicar expande | «If the topic has been folded, the number ofthe sub-topics will be displayed right beside the main topic» | https://xmind.com/user-guide/fold-unfold-subtopic-new |

## 6. MindNode

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| N1 | Tab / Enter | Tab = filho; Enter = irmão; **⌥Enter = irmão ACIMA**; ⇧Enter = novo nó principal | «⇥ New Child Node · ⏎ Create New Sibling Node · ⌥⏎ Create New Sibling Node Above Current Node · ⇧⏎ Create New Main Node» | https://mindnode.com/article/shortcuts-mac |
| N2 | Setas | ← / → movem; **⌘↑/⌘↓ reordenam irmãos**; ⌘←/⌘→ movem o nó | «⌘ ↑ Move Selected Node Above Sibling Node · ⌘↓ Move Selected Node Below Sibling Node» | idem |
| N3 | Apagar | ⌫ apaga o nó **e os filhos**; ⌥⌫ apaga só os seleccionados (filhos ficam) | «⌫ Delete Node/Connection (for nodes, also all child nodes are deleted) · ⌥⌫ Delete Selected Nodes (if child nodes are not selected, they are not deleted)» | idem |
| N4 | Equilíbrio | comando **Balance Children ⌥⌘B** | «⌥⌘B Balance Children» | idem |
| N5 | Recolher | ⌥. dobra/desdobra | «⌥. Fold/Unfold Node» | idem |

---

## 7. Convergência

| lei | apps que a documentam | o que o Miro faz |
|---|---|---|
| **Tab = filho** | 4 (Miro, Whimsical, XMind, MindNode); FigJam diverge (Cmd+Return) | Tab = filho ✔ |
| **Enter = irmão** | 4 (Miro, Whimsical, XMind, MindNode); FigJam diverge (Cmd+Shift+Return) | Enter = irmão, já em edição ✔ |
| **Onde nasce o irmão** | 3 dizem ABAIXO/depois por omissão e dão tecla para ACIMA (Whimsical Cmd+Enter, MindNode ⌥Enter, XMind «+ below») | **silente** |
| **Shift+Enter = quebra de linha** | 2 (Miro, Whimsical); MindNode usa-o para novo nó principal | quebra de linha ✔ |
| **Apagar = ramo inteiro** | 2 (Miro, MindNode — que dá ⌥⌫ para poupar os filhos); restantes silentes | ramo inteiro, filhos não sobem ✔ |
| **Recolher com contador** | 2 (Miro, XMind: número clicável); Whimsical/MindNode têm tecla | «−» ao pairar; número de nós+comentários escondidos; sem tecla documentada |
| **Recolher por tecla** | 3 (Whimsical Cmd+/, XMind Ctrl+/, MindNode ⌥.) | **silente** |
| **Setas navegam** | 2 (Miro, MindNode); semântica pai/filho/irmão nunca explícita | «Arrows to move between nodes» (sem semântica) |
| **Arrastar = trocar de pai** | 3 (Miro, Whimsical «rearrange», FigJam «drag to node») | sim; Ctrl/Cmd segura = só move |
| **Linha curva ou recta** | 2 (Miro, Whimsical) | curva / recta(cotovelo) |
| **Colar lista indentada = mapa** | 1 (Whimsical) | **silente** (só colar mapa do Mural) |
| **Equilíbrio esq./dir.** | DIVERGE: XMind opcional por ALTURA; MindNode por comando; Miro/Whimsical/FigJam silentes | lado = «+» clicado ou posição inicial (API); nada automático documentado |
| **Auto layout desligável** | 2 (Miro no pai; XMind «Branch Free-Positioning» opt-in) | ligado por omissão, desliga no pai, «Layout nodes» pontual |
| **Quatro lados** | 0 | só 2 lados (horizontal ou vertical) [comunidade] |
| **Limite de nós** | 1 (Miro: sem limite) | sem limite |
