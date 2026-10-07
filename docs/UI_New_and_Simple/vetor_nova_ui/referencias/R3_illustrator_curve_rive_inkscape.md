> Pesquisa web de 2026-10-07 (subagente, WebSearch/WebFetch). Cada afirmação traz a fonte; o que não se confirmou está marcado. ⚠️ Fonte é LEITURA de documentação e de fóruns — nenhum destes apps foi CORRIDO (CLAUDE.md §0.9); para UI isso basta, para LEI (geometria, comportamento) não.

# Pesquisa de UI/UX: editores vetoriais 2D (Illustrator, Linearity, Rive, Inkscape e outros)

Fontes verificadas em 07/10/2026. Duas limitações. As páginas helpx.adobe.com e inkscape.org deram 403 ao serem abertas, por isso as afirmações sobre elas vêm do resumo do buscador ou de páginas espelho, e uso a URL que o buscador devolveu. Não fiz capturas de ecrã. Onde não houve fonte, marquei "(não confirmado)".

---

## A) Adobe Illustrator (2024–2026)

**Regiões da janela**
- O workspace tem: Application Bar, **Control panel** (barra horizontal no topo), **Toolbar** (à esquerda), **Properties panel** (à direita), **Contextual Task Bar** (flutua sobre o canvas), Status bar e Help Bar. [helpx workspace](https://helpx.adobe.com/lv/illustrator/using/workspace-basics.html)
- O **Control panel** é uma linha única no topo. Mostra as opções do objeto, da ferramenta ou do fluxo ativo. [helpx control panel](https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/control-panel-overview.html)
- O **Properties panel** mostra os controlos mais importantes da seleção ou da ferramenta ativa:
  - Sem seleção, mostra o nível do documento: artboard, réguas, grelha e preferências.
  - Com seleção, mostra secções como Transform e Appearance, e um bloco de **Quick Actions** (por exemplo Group, Recolor, Pathfinder, Align). [helpx properties](https://helpx.adobe.com/in/illustrator/using/properties-panel.html) · [Adobe Learn](https://www.adobe.com/learn/illustrator/web/basics-essentials-workspace-illustrator)

**Toolbar Basic vs Advanced e a gaveta "Edit Toolbar"**
- Há três configurações: *Getting Started*, *Basic* (o dia a dia) e *Advanced* (todas as ferramentas). Troca-se em Window > Toolbars. [octet.design](https://octet.design/journal/illustrator-tools)
- O botão "…" no fim da toolbar abre a gaveta **All Tools**. Arrasta-se cada ferramenta entre a gaveta e a barra para personalizar. [community](https://community.adobe.com/questions-652/organising-toolbar-786617)

**Contextual Task Bar (CTB)**
- É uma barra flutuante que aparece perto da seleção com as "próximas ações mais relevantes". [anúncio Adobe](https://community.adobe.com/announcements-651/contextual-task-bar-in-illustrator-2025-quick-actions-when-you-need-them-1546580)
- O que mostra em cada caso [helpx CTB](https://helpx.adobe.com/illustrator/using/contextual-task-bar.html):
  - **Path com Selection tool:** Gen Shape Fill, Fill e Stroke.
  - **Path com Direct Selection:** Simplify, Smooth, remover pontos, ligar extremidades, cortar no ponto, converter ponto em canto ou suave.
  - **Texto:** família, estilo, tamanho e fill.
  - **Canvas vazio:** Draw (Pencil), Generate Vectors, Place Image.
- Desde a v29 pode ser **fixada (pin)** numa posição, que se mantém ao trocar de seleção ou de documento. A posição volta ao normal pelo menu "More". Liga e desliga em Window > Contextual Task Bar. [anúncio](https://community.adobe.com/announcements-651/contextual-task-bar-in-illustrator-2025-quick-actions-when-you-need-them-1546580) · [community v29](https://community.adobe.com/questions-652/illustrator-max-2025-v29-0-align-object-to-path-image-trace-text-to-vector-graphic-and-more-812615)

**Ferramentas de destaque**
- **Shape Builder (Shift+M):** arrastar sobre as regiões funde-as. Com Alt/Option entra no modo apagar regiões ou arestas. Shift+arrastar faz um laço retangular. Tem *gap detection* de 3, 6 ou 12 pt, ou valor livre. [helpx shape builder](https://helpx.adobe.com/illustrator/using/building-new-shapes-using-shape.html)
- **Live Corners:** com a Direct Selection aparecem "widgets" (pontos) em cada canto. Arrastar muda o raio, Alt+clique alterna entre round, inverted e chamfer, e duplo-clique abre o valor numérico. [Adobe Learn](https://adobe.com/learn/illustrator/web/live-corners-in-illustrator) · [CreativeLive](https://site.creativelive.com/class/infographic-elements-in-illustrator-jason-hoppe/lessons/how-to-use-corner-widgets)
- **Width tool (Shift+W):** arrastar a partir do traço cria *width points*. Um width point **não é um anchor point**: não muda a geometria, só a aparência. Duplo-clique edita os dois lados separadamente. O perfil resultante aparece no Control e no Stroke panel. [Peachpit](https://www.peachpit.com/articles/article.aspx?p=2979069)
- **Curvature (Shift+~):** a curva sai suave por omissão. Alt+clique ou duplo-clique cria ou converte em canto. [Algonquin](https://cg.algonquindesign.ca/information/drawing-tools)
- **Intertwine:** em Object > Intertwine > Make, clica-se ou faz-se um laço sobre o cruzamento e o trecho de trás passa para a frente (efeito de entrelaçado). [illustratorhow](https://illustratorhow.com/intertwine-tool/)
- **Retype:** identifica a fonte em imagens ou em texto convertido em contornos e torna-o editável. **Mockup:** aplica arte vetorial sobre a foto de um produto, adaptando-a às curvas. [helpx 2024](https://helpx.adobe.com/illustrator/using/whats-new/2024.html) · [Fast Company](https://www.fastcompany.com/91161756/adobe-illustrator-mockups-tool)
- **Generativo (só como nota):** Text to Vector Graphic, Generative Shape Fill e Text to Pattern. [community v29](https://community.adobe.com/questions-652/illustrator-max-2025-v29-0-align-object-to-path-image-trace-text-to-vector-graphic-and-more-812615)
- **Novidades v29–v30:** Objects on Path, Dimension tool, snapping tangente e perpendicular, artboards bloqueáveis e coloridas, novo font browser, Turntable (beta). [community v30](https://community.adobe.com/questions-652/illustrator-2026-v30-0-font-browser-color-snapping-artboards-enhancements-turntable-and-more-1546612)

**Fill/stroke, nós, booleanas e alinhamento**
- Fill e stroke editam-se no widget da toolbar: X alterna o ativo, Shift+X troca, D repõe o padrão. [hongkiat](https://www.hongkiat.com/blog/adobe-illustrator-keyboard-shortcuts/)
- Nós: Direct Selection (A), com as ações de ponto na CTB.
- Booleanas: Pathfinder no Properties, mais o Shape Builder.
- Alinhamento: Align no Properties e no Control. O modo "key object" não ficou confirmado nesta pesquisa.
- Outros atalhos: V, A, P, Ctrl+J juntar, Ctrl+8 compound path, Ctrl+Y modo outline. [hongkiat](https://www.hongkiat.com/blog/adobe-illustrator-keyboard-shortcuts/)

**O que os usuários amam e odeiam**
- Elogio: o Properties panel mostra o essencial (tamanho e orientação da artboard) sem obrigar a abrir sub-painéis. [VectorStyler forum](https://www.vectorstyler.com/forum/post/24126)
- Crítica ao Properties panel [community](https://community.adobe.com/t5/illustrator-discussions/the-new-properties-panel-is-a-disappointment/m-p/9497627):
  - "doesn't make other panels obsolete": caps de traço e distribuição continuam noutros painéis.
  - Mostra propriedades sem nada selecionado, "you're actually not modifying anything".
  - Um expert acha o Control panel "much more convenient and efficient".
- Crítica à CTB: "incredibly annoying", aparece "IN THE CENTER of the screen" e reaparece em cada documento novo. A solução aceita foi desligá-la. [community](https://community.adobe.com/questions-652/how-to-deactivate-disable-turn-off-contextual-task-bar-in-illustrator-805537) · [outro fio](https://community.adobe.com/t5/illustrator-discussions/i-hate-this-contextual-task-bar-in-v27-9-on-illustrator-amp-i-want-my-properties-back-across-the-top/m-p/14085860/highlight/true)

---

## B) Linearity Curve (ex-Vectornator) e Linearity Move

**Regiões (Mac)**
- **Toolbar à esquerda** com seleção, edição, desenho, borracha e texto. [Academy Mac](https://linearity.io/academy/curve/mac/user-guide/workspace/interface)
- **Barra no topo**, personalizável com clique-direito ou em View > Customize Toolbar. No iPad chama-se **Action Bar**: fica no topo ao centro, tem undo, redo, zoom, copy e paste, e é **sensível ao contexto**, ou seja, os botões mudam com o tipo de objeto selecionado. [Academy Mac](https://linearity.io/academy/curve/mac/user-guide/workspace/interface) · [iPad interface](https://linearity.io/academy/curve/ipad/user-guide/workspace/interface)
- **Inspector à direita.** As antigas abas Style, Arrange e Path foram **fundidas numa única aba Style**. Layers e Import continuam separadas. [vectornator 4.11](https://vectornator.io/blog/4-11-0-interview)
- Secções da aba Style, em blocos recolhíveis:
  - Location: posição, rotação, tamanho, flip.
  - Shape: suavização e raio de canto.
  - Arrange: ordem, align, distribute, máscara, repetir ação, grupo.
  - Appearance: blend, opacidade, blur.
  - Fill, Stroke e Shadow, cada um com o seu interruptor.
  - **Path:** booleanas, edição de nós, texto em caminho.
  - As secções específicas do tipo de objeto aparecem logo abaixo de Location.
  - [Academy Mac](https://linearity.io/academy/curve/mac/user-guide/workspace/interface)
- Desde junho de 2025 o inspector **esconde controlos irrelevantes** para a seleção atual. Os menus de clique-direito ganharam grupo, alinhar e duplicar, iguais no canvas e no painel de layers. Ctrl/Cmd+D repete a última duplicação. [blog Linearity 18/06/2025](https://www.linearity.io/blog/linearity-curve-improvements-you-ll-actually-notice/)

**Ferramentas**
- Atalhos [Academy shortcuts](https://linearity.io/academy/curve/mac/user-guide/shortcuts-and-gestures/tools-shortcuts):
  - V Selection, A Node, P Pen, N Pencil, B Brush, M Shape Builder, C Scissors, E Eraser, I Eyedropper.
  - Formas: R, O, L.
  - ⇧⌘P volta à ferramenta anterior.
- **Pen:** o nó assimétrico é o padrão, ⌥ faz um nó "disconnected" e ⇧ prende a 45°. Termina-se com duplo-clique ou com o botão "Finish Path" do inspector. [Academy drawing tools](https://linearity.io/academy/curve/mac/user-guide/vector-editing/drawing-tools)
- **Brush:** traço de largura variável, com perfil editável num gráfico.
- **Node tool:** permite dar um raio de canto diferente a cada canto. [Academy drawing tools](https://linearity.io/academy/curve/mac/user-guide/vector-editing/drawing-tools)
- **Shape Builder:**
  - No modo Merge, um contorno azul grosso mostra o que vai fundir.
  - No modo Erase, um toque apaga uma interseção e arrastar apaga várias.
  - [Academy shaping tools](https://www.linearity.io/academy/curve/mac/user-guide/vector-editing/shaping-tools/)
- **Auto Trace:** o menu aparece **na barra de Quick Actions** assim que se seleciona uma imagem. Tem quatro modos (Sketch, Illustration, Photo, Basic Shapes) com sliders de detalhe e de cor. [Academy Auto Trace](https://www.linearity.io/academy/curve/mac/user-guide/images/auto-trace)

**Como simplifica em relação ao Illustrator**
- Uma só aba de estilo em vez de dezenas de painéis.
- Uma barra de ações que muda com a seleção.
- Um único Shape Builder com dois modos.
- Clique-direito com o mesmo conteúdo no canvas e nas layers.
- O marketing fala em "learning curve significativamente menor". É fonte do próprio fabricante, logo parcial. [linearity guide](https://linearity.io/guide/switch-from-illustrator-to-curve/)

**Linearity Move (animação)**
- A timeline tem duas vistas: **Project view**, com as cenas e as transições entre elas, e **Scene view**, que se abre com duplo-clique numa cena. [Academy Move timeline](https://www.linearity.io/academy/move/mac/user-guide/interface/timeline/)
- Os keyframes vêm **sempre em pares** (início e fim) e arrastam-se juntos.
- Há um toggle **Animate/Design no canto superior esquerdo da timeline**:
  - Em Animate, editar no canvas cria keyframes.
  - Em Design, a edição vale para todas as cenas.
- O "Auto Animate" foi descontinuado. [Academy Auto Animate](https://linearity.io/academy/move/mac/user-guide/animate-mode/auto-animate/)

---

## C) Rive (o mais relevante para nós)

**Regiões**
- **Toolbar no topo**, por esta ordem [toolbar](https://rive.app/docs/editor/interface-overview/toolbar):
  - Editor menu: histórico, export, share, render.
  - Ferramentas: select, translate, rotate e scale; artboards e components; formas e pen; bones; State Machine player.
  - Invite e Publish.
  - Toggle de pré-visualização de data binding.
  - **Mode Toggle (Tab).**
- **Sidebar à esquerda:** Hierarchy (árvore com pais e filhos e ordem de render), assets, data, animações e o AI Agent. [overview](https://rive.app/docs/editor/interface-overview/overview)
- **Inspector à direita**, totalmente contextual. Sem seleção mostra as propriedades do ficheiro, incluindo a interpolação padrão. [inspector](https://rive.app/community/doc/inspector/docasBwsL61r)
- **Align/distribute no topo do Inspector** quando há objetos selecionados. [inspector (busca)](https://rive.app/community/doc/inspector/docasBwsL61r)
- **Timeline em baixo**, que **só aparece em Animate mode**. Com uma state machine selecionada, a timeline dá lugar a um **grafo**. [overview](https://rive.app/docs/editor/interface-overview/overview)

**Design vs Animate**
- Troca-se com **Tab**. [design-vs-animate](https://rive.app/docs/editor/fundamentals/design-vs-animate-mode)
- Design é para estrutura: criar e apagar objetos, fills, importar, e montar o rig (bones e constraints).
- Animate é para timelines, state machines, keys e listeners. A UI ganha a timeline, a lista de animações e o painel de interpolação à direita da timeline. [animate overview](https://rive.app/docs/editor/animate-mode/animate-mode-overview)
- Aviso da própria Rive: em Animate com uma timeline ativa, mover, redimensionar ou recolorir "may be keyed automatically" (keys criadas sem querer). [design-vs-animate](https://rive.app/docs/editor/fundamentals/design-vs-animate-mode)

**Keys no Inspector**
- Ao lado de cada propriedade animável há um botão de key com três estados [keys](https://rive.app/docs/editor/animate-mode/keys):
  - **Contorno cinza:** sem keys.
  - **Contorno azul:** a propriedade está animada.
  - **Preenchido a azul:** há uma key exatamente no playhead.
- Alt+arrastar a borda de um grupo de keys estica-as no tempo. [keys](https://rive.app/docs/editor/animate-mode/keys)
- A easing edita-se no painel de interpolação ou no **Graph Editor**, que substitui a timeline quando ligado. Há também uma *work area* para limitar a reprodução. [interpolation](https://rive.app/docs/editor/animate-mode/interpolation-easing) · [timeline](https://rive.app/docs/editor/animate-mode/timeline)

**Edição de vértices**
- Entra-se com duplo-clique, ou com Enter (duas vezes numa forma paramétrica, uma vez num path). Sai-se com Esc ou com o botão **"Done Editing" no topo do Inspector**. [edit vertices](https://rive.app/docs/editor/fundamentals/edit-vertices)
- No modo vértice o Inspector troca de conteúdo: Open Path, Reverse Direction, Convert Radial Corners, Simplify.
- Há quatro tipos de handle (Straight, Mirrored, Detached, Asymmetric) e Corner Radius por vértice.
- Atalhos de vértice [changelog](https://rive.app/changelog/new-path-and-vertex-shortcuts):
  - Ctrl+clique alterna entre mirrored e straight.
  - Alt+clique num handle separa-o.
  - Alt+clique num vértice, com a Pen, apaga-o.
  - Alt+arrastar duplica.

**Bones, mesh e IK**
- **B** cria bones: cada clique acrescenta um osso à cadeia, e Esc ou V termina. Só o osso raiz tem X e Y. [bones](https://rive.app/docs/editor/manipulating-shapes/bones)
- Bind: no **Inspector > Bind Bones** carrega-se em +, depois Ctrl/Cmd+Shift+clique escolhe os ossos, depois Done. [bones](https://rive.app/docs/editor/manipulating-shapes/bones)
- Os vértices mostram **gráficos de pizza** com o peso de cada osso. Há Edit Weights, Smooth e Auto-Weights.
- **Mesh:** tem um modo de edição com Edit Contour (P). A Mesh Pen cria arestas forçadas. A Weight tool é Shift+B. Pode-se ligar e desligar a visualização de triângulos, pesos e deformação. [meshes](https://rive.app/docs/editor/manipulating-shapes/meshes)
- **IK:** cria-se um grupo com estilo "Target" (G), seleciona-se o último osso e adiciona-se o constraint no Inspector. Propriedades: Bone Count, Invert, Strength e ordem dos constraints. [IK](https://rive.app/docs/editor/constraints/ik-constraint)
- Fill e stroke: Solid, Linear e Radial no Inspector. [fill-and-stroke](https://rive.app/docs/editor/fundamentals/fill-and-stroke)
- **Booleanas: não existem.** Há um pedido no fórum ("tricky if not impossible to add, subtract, and intersect"). Não confirmei se mudou em 2026. [feature request](https://rive.app/community/forums/feature-requests/fsyCzA4A9Yar/boolean-shape-functions/fttMYEgUWXiP)
- Opinião de animadores: a separação entre design e animação é "intuitive after a short amount of time", e o editor é comparado ao "Figma face ao After Effects". [hackdesign](https://www.hackdesign.org/toolkit/rive/) · [3dartist](https://3dartist.substack.com/p/wtf-is-rive)

---

## D) Inkscape 1.3 e 1.4 (contraste)

**Regiões (1.3)**
- Menu no topo; logo abaixo, a **Tool Controls Bar** (opções da ferramenta ativa); toolbox vertical à esquerda.
- **Commands Bar à direita**, com uma seta para os comandos escondidos.
- **Snap** no canto superior direito (ímã, atalho %).
- **Docking de diálogos à direita do canvas.**
- Paleta e status bar em baixo.
- [manual 1.3](https://inkscape-manuals.readthedocs.io/en/1.3/interface.html)

**Snap**
- Na 1.2 a barra de snap foi trocada por um **popover** com dois modos. O Simple tem 3 opções, incluindo o alinhamento; o Advanced tem tudo. As definições passaram a ser globais em vez de gravadas no documento. [release 1.2](https://wiki.inkscape.org/wiki/Release_notes/1.2)
- A 1.3 devolveu, como opção, a barra de snap permanente. [release 1.3](https://wiki.inkscape.org/wiki/Release_notes/1.3)

**Diálogos**
- Na 1.2 cada painel de diálogos ganhou um mini-menu com a lista de diálogos agrupados por propósito. As zonas de docking "piscam" quando se arrasta um diálogo para perto. Align e Arrange foram fundidos. [release 1.2](https://wiki.inkscape.org/wiki/Release_notes/1.2)

**1.3**
- **Shape Builder (X):** clique adiciona (azul), Shift+clique remove (rosa), arrastar une. [release 1.3](https://wiki.inkscape.org/wiki/Release_notes/1.3)
- Layers & Objects ganhou de volta a busca, a opacidade e o blend. Arrastar os ícones de olho e cadeado na vertical aplica-os a vários objetos de uma vez.

**1.4** [release 1.4](https://wiki.inkscape.org/wiki/Release_notes/1.4)
- As toolbars deixaram de enfiar tudo o que não cabe numa única setinha no fim; agora repartem as opções por vários dropdowns. As próprias notas admitem que antes as opções "were hard to discover".
- O Fill & Stroke ganhou ângulo numérico para gradientes.
- Object Attributes e Object Properties fundiram-se num só diálogo.
- Swatches redesenhado, Filter Gallery e Extension Gallery com busca.
- **Handles personalizáveis** por CSS.
- Shape Builder também sobre raster.
- Comportamento novo ao apagar nós.

**1.5 (GTK4)**
- Notas em rascunho na wiki: Fill & Stroke redesenhado com um novo "PaintSwitch" e recolor global, seletores de cor em popover em vez de janela, laço de seleção com cores azul e rosa, documentos em abas. [release 1.5](https://wiki.inkscape.org/wiki/Release_notes/1.5)
- O marco no GitLab tinha data de 29/04/2026. **Não confirmei se já saiu.** [milestone](https://gitlab.com/groups/inkscape/-/milestones/16)

**Por que é visto como menos intuitivo**
- O próprio projeto reconhece "inconsistent behavior and lack of discoverability". [inkscape-devel](https://lists.inkscape.org/hyperkitty/list/inkscape-devel@lists.inkscape.org/message/M2CWEMBH7F3EJ3BGKXATFDGHXAZOVOBV/)
- Fill & Stroke descrito como "legacy… too cluttered", e ambíguo ao editar um passo de gradiente. [inkscape-devel](https://lists.inkscape.org/hyperkitty/list/inkscape-devel@lists.inkscape.org/thread/2OJTEMMCJTABG7GU34EMMSQB6UOATV3N/)
- Itens escondidos no overflow da toolbox perdem o tooltip e o atalho. [gitlab #8875](https://gitlab.com/inkscape/inbox/-/issues/8875) · [#7057](https://gitlab.com/inkscape/inbox/-/issues/7057)
- Reviews recentes falam em "cluttered… less intuitive" e em termos e localização das ferramentas confusos. [Capterra](https://www.capterra.com/p/178240/Inkscape/reviews/)
- Citação antiga (2010), só como histórico: alternar entre path e fill "is a bad copy of Illustrator and doesn't work as intuitive", e as layers "isn't as inviting". [mastersofmedia](https://mastersofmedia.hum.uva.nl/blog/2010/10/31/fail-and-prejudice-using-inkscape-as-an-alternative-to-illustrator/)
- No macOS, diálogos que abrem atrás da janela dão a impressão de travamento. [HN](https://news.ycombinator.com/threads?id=ltlnx)

**Atalhos:** S, N, B, Ctrl+Shift+F (Fill & Stroke), Ctrl++ (união) — (não confirmado nesta pesquisa; são os padrões conhecidos).

---

## E) Bônus

- **Concepts:** vetor com uma **roda de ferramentas radial** personalizável e a ferramenta **Nudge**, que empurra o traço "como um fio". É o exemplo mais lúdico que encontrei. [Concepts](https://concepts.app/en/learn-the-basics-of-concepts/)
- **Boxy SVG:** filosofia declarada de evitar diálogos sobrepostos. Os painéis recolhem com um clique ou uma tecla, e quase tudo (gradientes, paths) edita-se direto no canvas. [Flathub](https://flathub.org/br/apps/com.boxy_svg.BoxySVG)
- **Graphic (Autodesign):** um só canvas, Toolbar, uma **Tool Options bar** própria, Layers à direita e abas de documento. [graphic.com](https://graphic.com/docs/interface)
- **Amadine:** painéis acoplados ou soltos, ferramentas à esquerda e opções à direita. [parkablogs](https://www.parkablogs.com/comment/30885)
- **Vectorpea:** "Photopea do Illustrator" no navegador. Não encontrei nada de UX marcante. [gamefromscratch](https://gamefromscratch.com/vectorpea-beta-launched/)
- **Affinity by Canva (fim de 2025):** gratuito, um só app com "Studios" (Vector, Pixel, Layout) que trocam o conjunto de ferramentas e são personalizáveis. É o paralelo direto do nosso modo por tipo de objeto. [cgpress](https://cgpress.org/archives/canva-reimagines-affinity-studio-and-releases-it-free-to-all-users.html)

---

## Padrões transferíveis

1. **Modo reconfigura as barras, não apenas a ferramenta.** No Rive, a timeline só existe em Animate e dá lugar ao grafo quando se seleciona uma state machine. Para nós: Object, Edit e Pose mudam o que aparece em baixo e à direita.
2. **Tab troca o modo principal** (Rive). Já coincide com o nosso Tab/Ctrl+Tab.
3. **Botão de key com 3 estados ao lado de cada propriedade animável:** sem key, animada, key no playhead (Rive). É barato e ensina sozinho.
4. **"Done Editing" no topo do Inspector** ao entrar em edição de vértices (Rive). Dá uma saída visível e o Inspector passa a mostrar só ações de path.
5. **Barra de ações contextual em posição fixa**, como a Action Bar do Linearity no topo ao centro, e não colada à seleção. Se for flutuante, tem de ser fixável e lembrar a posição (lição da CTB).
6. **Um único Inspector com secções recolhíveis** que esconde o que não se aplica, em vez de abas Style/Arrange/Path (Linearity juntou-as).
7. **Align e distribute no topo do Inspector, só com seleção** (Rive), e não num painel separado.
8. **Interruptor por secção para Fill, Stroke e Shadow** (Linearity): ligar e desligar sem perder os valores.
9. **Toolbar Basic e Advanced com uma gaveta "…"** (Illustrator), para o principiante ter poucas ferramentas e o avançado ter todas.
10. **Shape Builder com dois modos e uma tecla modificadora** (Alt no Illustrator, Shift no Inkscape), com pré-visualização colorida do que vai fundir ou apagar (azul/rosa no Inkscape, contorno grosso no Linearity).
11. **Widgets de canto no próprio canvas** (Live Corners): arrastar muda o raio, Alt+clique alterna o tipo, duplo-clique abre o número.
12. **Width points separados dos anchor points** (Illustrator): a largura variável é aparência, não geometria. Encaixa bem num traço com perfil.
13. **Pesos de bones como pizza em cada vértice, mais Auto-Weights e Smooth** (Rive). Mostra a influência sem pintar tudo.
14. **IK como constraint no Inspector, com Target e Strength de 0 a 100 %** (Rive), em vez de uma ferramenta separada.
15. **Clique-direito igual no canvas e nas layers** (Linearity, 2025).
16. **Inspector sem seleção mostra o documento** (Illustrator, Rive), mas sem sugerir que se está a editar um objeto (ver a crítica nas Armadilhas).
17. **Snap em popover com modo Simple de 3 opções e um Advanced** (Inkscape 1.2), com a barra fixa como opção.
18. **Keyframes em par na timeline, que se arrastam juntos** (Move), útil para um modo simplificado.

## Armadilhas

- **Barra flutuante que tapa a arte e reaparece sozinha** (CTB: "IN THE CENTER of the screen", desligada pela maioria). Se houver uma, tem de ter pin, lembrar a posição e desligar com uma tecla.
- **Inspector que mostra campos editáveis sem seleção** leva o usuário a mexer em valores que não afetam nada. ([community](https://community.adobe.com/t5/illustrator-discussions/the-new-properties-panel-is-a-disappointment/m-p/9497627))
- **Painel "contextual" incompleto** obriga a ir a outro painel (caps, distribuição) e acaba a duplicar a UI em vez de a substituir.
- **Keys criadas sem querer em modo de animação** (aviso do próprio Rive). Animate tem de ser visualmente inconfundível e as edições estruturais devem pedir Design.
- **Overflow escondido numa setinha no fim da barra:** o Inkscape admitiu que era "hard to discover", e os itens lá dentro perdem tooltip e atalho.
- **Muitos diálogos acoplados ao mesmo tempo** (Inkscape), sobretudo um Fill & Stroke "too cluttered" e ambíguo em gradientes.
- **Duas barras contextuais concorrentes** (Control panel, Properties e CTB no Illustrator): o usuário perde o sítio onde as coisas estavam, e um "sumiço" do Control panel deu um fio inteiro de suporte.
- **Ausência de booleanas num editor de animação** (Rive) gera pedidos recorrentes, por isso convém tê-las desde cedo.
