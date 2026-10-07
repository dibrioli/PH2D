> Pesquisa web de 2026-10-07 (subagente, WebSearch/WebFetch), aberta a pedido do Enio: *«apps para iPad costumam ser bons nisso»*. Cada afirmação traz a fonte; o que não se confirmou está marcado. ⚠️ LEITURA de documentação, nenhum app CORRIDO (CLAUDE.md §0.9).

# UI/UX de apps vetoriais e de desenho no iPad: o que copiar para um editor desktop + iPad

**Ressalvas sobre as fontes.** As páginas do `helpx.adobe.com` (Illustrator e Fresco) devolveram 403 quando tentei abri-las. As afirmações da Adobe vêm do trecho que a busca mostra ou de `adobe.com/learn`, e cada uma está marcada. Algumas páginas de review vieram truncadas e também ficaram marcadas. O que não pude conferir diz **(não confirmado)**.

---

## 1. Affinity Designer 2 para iPad, e o Affinity unificado da Canva

**Situação em 07/10/2026**
- O Affinity unificado (Designer, Photo e Publisher num app só, gratuito) saiu em 30/10/2025 só para Mac e Windows. Para o iPad ficou anunciado "2026" ([9to5Mac](https://9to5mac.com/2025/10/30/affinity-is-now-an-all-in-one-free-app-with-native-canva-integration/), [AppleInsider](https://appleinsider.com/articles/25/10/30/affinitys-entire-suite-goes-completely-free-on-mac-with-new-all-in-one-app)).
- Um blog diz que a versão de iPad chega em **novembro de 2026**, com base num post da Affinity no Instagram. O próprio texto admite que "Affinity has not yet revealed every detail of the iPad version" ([Weronika Salach](https://www.weronikasalach.com/blog/affinity-v3-is-finally-coming-to-ipad-this-november)). **A versão unificada para iPad ainda não foi lançada, e a sua UI é desconhecida (não confirmado).**
- Os apps V2 de iPad continuam a existir e ficaram gratuitos em outubro de 2025 ([AppleInsider](https://appleinsider.com/articles/25/10/02/all-affinity-apps-are-now-free-for-the-ipad---for-now)). A última versão é a 2.6.4, de 23/09/2025, com nota 4,7 e selo Editors' Choice ([App Store](https://apps.apple.com/us/app/affinity-designer-2-for-ipad/id1616833418)).

**Regiões da tela (V2 iPad)** ([Interface Visual Reference](https://affinity.help/designer2ipad/en-US.lproj/pages/Workspace/interface.html))
- **Topo:** botão Home, menu de **Personas** (Designer, Pixel e Export, cada uma um espaço de trabalho diferente), menu Document (redimensionar, exportar, artboards e o interruptor do Command Controller) e menu Edit (área de transferência, agrupar, expandir traço, modo de preenchimento).
- **Esquerda:** a toolbar de ferramentas.
- **Direita:** a coluna de **Studio**, uma faixa de ícones (Layers, Color, Navigator…). Tocar num ícone abre o painel. A faixa "is scrollable to accommodate all panel icons", com a exceção do iPad Pro 12,9" ([busca/affinity.help](https://affinity.help/designer2ipad/en-US.lproj/pages/Workspace/interface.html)).
- **Base:** a **Context Toolbar** fica "below the canvas" e mostra as opções da ferramenta ativa. Ela rola quando não cabe, menos no 12,9" ([contextBar](https://affinity.help/publisher2ipad/en-US.lproj/pages/Workspace/contextBar.html)).
- Também na base ficam os botões de Zoom, Preview e Snapping.
- **Helper:** "touch and hold to display tooltips on the interface".

**Como a ferramenta ativa muda a UI**
- A Context Toolbar troca a cada ferramenta e também conforme a seleção e a Persona ([contextBar](https://affinity.help/publisher2ipad/en-US.lproj/pages/Workspace/contextBar.html)).
- Os painéis Layers e Brushes têm um **Compact Mode**. No Brushes, por exemplo, mostra só a miniatura e a largura do pincel, e a ajuda diz que serve para iPads menores ([busca/affinity.help](https://affinity.help/designer2ipad/English.lproj/pages/Panels/brushesPanel.html)).

**Command Controller (os modificadores na tela)** ([wheel.html](https://affinity.help/publisher2ipad/en-US.lproj/pages/Workspace/wheel.html))
- É um botão central com modificadores em volta: ⌘, ⌥ e ⇧ segundo a ajuda. A App Store fala em "command, control, option, shift", e o número exato de teclas **não está confirmado**.
- **Três estados:**
  - Inativo.
  - **Held:** vale enquanto se arrasta o botão central sobre o modificador.
  - **Locked:** fica ligado até repetir o gesto. A ajuda compara a diferença a Shift contra Caps Lock.
- **Em repouso** mostra só o botão central e os modificadores travados, como lembrete. **Acordado** mostra todos.
- Para mover: "long-press… until the controller pulses" e arrastar.
- Um toque esconde-o temporariamente.
- Com o retângulo, um modificador faz quadrado, outro desenha a partir do centro, e os dois juntos combinam os efeitos.

**Quick Menu** ([quickMenu](https://affinity.help/designer2ipad/en-US.lproj/pages/Workspace/quickMenu.html))
- Abre com **três dedos a deslizar para baixo**, ou com duplo toque no centro do Command Controller.
- Em cima tem uma linha fixa: Duplicate, Cut, Copy, Paste, Paste Style e Delete.
- Em baixo tem **nove botões grandes que mudam com o contexto** (por exemplo Select All sem seleção, e Group ou Convert to Curves com seleção).
- Para trocar um botão: pressão longa e escolher da lista.
- Para reordenar: pressão longa até os botões "jiggle", depois arrastar.

**Gestos**
- Toque com 2 dedos desfaz e com 3 dedos refaz ([9to5Mac](https://9to5mac.com/2018/07/12/affinity-designer-for-ipad-tips-tricks-video/)).
- **Um dedo da outra mão pousado no canvas faz de modificador.** Desenhar a elipse e segurar o indicador da outra mão trava a proporção e dá um círculo ([logosbynick](https://logosbynick.com/perfect-circle-with-affinity-designer-for-the-ipad/)).
- Segurar com 2 dedos redimensiona a partir do centro ([9to5Mac](https://9to5mac.com/2018/07/12/affinity-designer-for-ipad-tips-tricks-video/)).
- O nome "Hold-down gestures" **não aparece como termo oficial (não confirmado)**. O mecanismo existe com o nome de "touch gestures" ([gestures](https://affinity.help/designeripad/en-US.lproj/pages/GetStarted/gestures.html)).
- Com teclado, uma pressão longa num atalho de ferramenta troca de ferramenta só enquanto se segura ("temporary tool use") ([shortcuts](https://affinity.help/designer2ipad/en-US.lproj/pages/Workspace/shortcuts.html)).
- Na V1, os modificadores na tela eram botões dentro da context toolbar ("On-screen modifier keys", nas preferências) ([onscreenModifiers](https://affinity.help/designeripad/en-US.lproj/pages/Workspace/onscreenModifiers.html)).

**O que os usuários dizem**
- Elogio: alternar entre caneta e dedos é "very smooth and quick".
- Crítica: no 13" com "more space" os alvos de toque ficam "fairly small".
- Crítica: "the top toolbar is hampered by Apple's 3 dot window dropdown widget", e alguém sugere não pôr controles no centro do topo nem no centro da base ([MPU forum](https://talk.macpowerusers.com/t/affinity-publisher-2-for-ipad-mini-review/31514)).

---

## 2. Linearity Curve (antigo Vectornator), iPad

**Regiões da tela** ([interface](https://linearity.io/academy/curve/ipad/user-guide/workspace/interface))
- **Toolbar vertical à esquerda**, com o **Color Widget** dentro dela.
- **Action Bar no topo**, personalizável ("Customize Toolbar": arrasta-se a ferramenta para a barra).
- **Inspector à direita**, com três abas:
  - **Style:** "context-aware", em secções que recolhem (Location, Shape, Arrange, Appearance, Fill, Stroke, Shadow, Path).
  - **Layers/Artboards.**
  - **Library.**
- Tocar na aba esconde ou mostra o Inspector.
- Na atualização Liquid Glass (05/09/2025), o Inspector passou a ter **duas colunas** e "adapting dynamically to window size" ([blog](https://www.linearity.io/blog/linearity-just-got-a-new-look-liquid-glass/)).

**Esconder o chrome**
- O **Magic Canvas** faz "the Curve Interface move out of the way when you are drawing or moving objects".
- **Quick Actions** é uma barra flutuante **logo abaixo do objeto selecionado** e pode ser desligada ([canvas settings](https://www.linearity.io/es/academy/curve/ipad/user-guide/settings/canvas/)).

**Modificadores sem teclado: dedos extra, não um botão** ([tools-gestures](https://www.linearity.io/academy/curve/ipad/user-guide/shortcuts-and-gestures/tools-gestures))
- Com a Pen:
  - +1 dedo no canvas: o nó anterior vira "disconnected".
  - +2 dedos: snap de 45° nas duas alças.
  - +3 dedos: snap de 45° numa alça só (igual ao ⌥ no Mac).
  - Duplo toque no nó final fecha o path.
- Ao desenhar ou redimensionar, +1 dedo trava a proporção.

**Gestos de canvas** ([canvas-gestures](https://linearity.io/es/academy/curve/ipad/user-guide/shortcuts-and-gestures/canvas-gestures))
- 2 dedos fazem pan, pinça e rotação.
- Toque com 2 dedos desfaz e com 3 refaz.

**O que os usuários dizem**
- Elogiam as dicas flutuantes com vídeo em cada ferramenta e o "Help Mode" ([Paperlike](https://paperlike.com/blogs/paperlikers-insights/linearity-curve-review)).
- Há um pedido no fórum para pôr o Inspector à esquerda, pensando em canhotos. O fórum não abriu, por isso o conteúdo **não está confirmado**; só o título apareceu na busca: [forum.linearity.io/t/…/1938](https://forum.linearity.io/t/can-we-move-the-inspector-to-the-left-side-of-the-screen/1938).

---

## 3. Adobe Illustrator para iPad

**Regiões da tela**
- **Toolbar à esquerda.** "long press or double tap to see any nested tools" ([Adobe Learn](https://www.adobe.com/learn/illustrator/web/compare-desktop-vs-ipad)).
- **Direita:** os painéis Properties e Layers, e o **Taskbar**, que é **exclusivo do iPad** e contextual.
  - Tem opacidade, duplicar, apagar e arranjar.
  - Com seleção, ganha alinhar, espelhar, combinar e editar paths ([features tour](https://www.adobe.com/learn/illustrator/web/features-tour)).
  - Shape Builder e Pathfinder **não destrutivo com prévia ao vivo** ([compare](https://www.adobe.com/learn/illustrator/web/compare-desktop-vs-ipad)).
- **Topo:** barra de navegação com Undo, Redo, Publish/Export, Help, Settings e View ([busca/helpx](https://helpx.adobe.com/illustrator/ipad/get-started/overview-and-setup/workspace-on-iPad.html)).
- **Barra "Common Actions"** quando há objeto selecionado. A Pencil mostra o controle de suavização antes de desenhar ([compare](https://www.adobe.com/learn/illustrator/web/compare-desktop-vs-ipad)).
- Ao tocar numa caixa de texto aparece um menu flutuante com tamanho e espaçamento ([9to5Mac](https://9to5mac.com/2020/10/20/adobe-illustrator-ipad-review/)).

**Touch Shortcut (o círculo no canto inferior esquerdo)**
- **Primário:** tocar e segurar o centro.
- **Secundário:** "drag… to the ring that appears around the circle" ([compare](https://www.adobe.com/learn/illustrator/web/compare-desktop-vs-ipad)).
- Na versão de outubro de 2022 passou a **alternar e travar** os modificadores primário e secundário, e o widget **lembra onde foi posto** de uma sessão para a outra ([busca/helpx](https://helpx.adobe.com/illustrator/using/whats-new/mobile-2023.html)).
- O que faz: Shift (eixo, 45°, proporção), seleção múltipla, e na ferramenta de ponto converte suave em canto ([busca: iMore/helpx](https://www.imore.com/adobe-illustrator-ipad-review)).

**Repeat** ([busca/helpx](https://helpx.adobe.com/illustrator/ipad/work-with-colors-and-patterns/repeat-patterns.html))
- **Radial**, **Grid** e **Mirror**, todos no menu Repeat do Taskbar, com **alças na tela** (espaçamento, contagem, início e fim).
- Duplo toque numa cópia edita o original.
- Estreou no iPad antes do desktop ([9to5Mac](https://9to5mac.com/2020/10/20/adobe-illustrator-ipad-review/)).

**O que os usuários dizem**
- Reclamam de não conseguir rolar o canvas com o trackpad, de não ter um jeito fácil de deselecionar quando o zoom está alto, e de crashes no iPadOS 26.4 ([community](https://community.adobe.com/questions-660/the-illustrator-app-keeps-crashing-on-ipados-26-4-2-1559219/index2.html), [deselect](https://community.adobe.com/t5/illustrator-on-the-ipad/illustrator-on-ipad-shortcut-to-deselect/m-p/11992100)).

---

## 4. Adobe Fresco

- **Touch Shortcut** no canto inferior esquerdo ([busca/helpx](https://helpx.adobe.com/fresco/using/getting-started-with-user-interface.html)):
  - O primário transforma o pincel em **borracha** temporária.
  - O secundário, nos pincéis vetoriais, faz o **vector trimmer**: um risco sobre o segmento corta-o no cruzamento seguinte.
  - **Duplo toque trava** o atalho, e o centro fica azul.
  - Help → Touch Shortcut abre um **mapa** que mostra o que cada ferramenta faz no primário e no secundário.
- **Taskbar:** concentra o trabalho com camadas.
- **Versão 7.3 para iPad (maio de 2026):** toolbar e taskbar mudaram de lugar, "the canvas now takes center stage, with your most-used tools right at the top" ([busca/helpx what's new](https://helpx.adobe.com/ca/fresco/desktop/introduction/whats-new.html)).
- **Queixa:** a mudança para o topo "disrupt[s] established muscle memory", e há relatos de "UI clutter and navigation friction". Isto vem de um agregador de reviews cuja página deu 403, por isso **não está confirmado** ([marlvel](https://marlvel.ai/apps/adobe-fresco-draw-paint-app/reviews)).

---

## 5. Procreate e Procreate Dreams

**Regiões da tela** ([interface](https://help.procreate.com/procreate/handbook/interface-gestures/interface))
- **Topo esquerdo, editar:** Gallery, Actions, Adjustments, Selection, Transform.
- **Topo direito, pintar:** Paint, Smudge, Erase, Layers, Color.
- **Sidebar** na borda (à esquerda por omissão; a opção "Right-hand interface" passa-a para a direita) ([prefs](https://help.procreate.com/procreate/handbook/actions/actions-preferences)). Tem:
  - **slider vertical de tamanho**;
  - o **botão Modify** quadrado (conta-gotas por omissão e reprogramável);
  - **slider vertical de opacidade**;
  - Undo e Redo (até 250 passos).
- Fora isso, não há chrome nenhum.

**Gestos** ([gestures](https://help.procreate.com/procreate/handbook/interface-gestures/gestures))
- Toque com 2 dedos desfaz, com 3 refaz, com **4 entra em tela cheia**.
- 3 dedos a deslizar para baixo abrem Copy/Paste. Esfregar com 3 dedos limpa a camada.
- "Quick pinch" encaixa o desenho na tela. Pinça e rotação mexem na vista.
- **Nas camadas:** pinça junta camadas, deslizar para a direita adiciona à seleção, toque com 2 dedos ajusta a opacidade, 2 dedos para a direita ligam o alpha lock.
- **Hover** (Apple Pencil): pinça muda o tamanho, deslizar muda a opacidade.

**Gesture Controls** ([prefs](https://help.procreate.com/procreate/handbook/actions/actions-preferences))
- Cada função (Eyedropper, QuickShape, QuickMenu, tela cheia, limpar…) recebe o gatilho que o usuário escolher.
- Conflito avisado na hora: "You cannot assign the same shortcut to two different features… a yellow warning icon will flash".

**QuickMenu** ([quickmenu](https://help.procreate.com/procreate/handbook/interface-gestures/quickmenu))
- **Menu radial de 6 botões**, aberto por um gesto configurável (também pelo *squeeze* do Pencil Pro).
- Duas formas de usar: tocar (o menu fica aberto) ou **tocar, arrastar na direção do botão e soltar**.
- Para trocar um botão: pressão longa e "Set Action", com 59 ações disponíveis.
- **Perfis** ilimitados.

**QuickShape** ([quickshape](https://help.procreate.com/procreate/handbook/guides/quickshape))
- Desenhar e **manter o dedo parado** faz o traço virar linha, arco, poligonal, elipse, triângulo ou quadrilátero.
- Um **2.º dedo** torna-o regular: quadrado, círculo, triângulo equilátero.
- Sem levantar o dedo, **arrastar escala e gira**; com o 2.º dedo a rotação anda em degraus de 15°.
- O botão "Edit Shape" mostra nós editáveis.

**Críticas**
- A opção "Right-hand interface" confunde canhotos, que procuram a palavra "Left".
- Gestos inconsistentes entre ecrãs, e o autosave sem nenhum aviso ([Pratt IxD](https://ixd.prattsi.org/2025/09/design-critique-procreate-ipad-app/)).

**Procreate Dreams**
- **Stage** na metade de cima e **Timeline** na metade de baixo, com os modos Compose, Perform e Keyframe ([interface](https://help.procreate.com/dreams/handbook/interface-and-gestures/interface)).
- **Perform:** os gestos em tempo real gravam keyframes.
- **Flipbook:** companheiro móvel que mostra três quadros (anterior, atual e seguinte), passados com o dedo ([busca/handbook](https://help.procreate.com/dreams/handbook/1.0/draw-and-paint/flipbook)).
- Na Timeline: riscar com a caneta seleciona faixas, pinça dá zoom, e 3 dedos a deslizar expandem ou reduzem a vista.
- Críticas: faltam gestos do Procreate, a curva de aprendizagem é "gesture-heavy" e não há tutoriais integrados. Isto vem do resumo da busca, porque a página veio truncada ([TechRadar](https://www.techradar.com/computing/software/procreate-dreams-isnt-quite-the-revolution-i-wanted-for-animation-but-im-holding-out-hope)).

---

## 6. Concepts (vetor em canvas infinito)

**Tool Wheel** ([workspace](https://concepts.app/manual/workspace))
- Três anéis:
  - **externo:** 8 ferramentas configuráveis;
  - **interno:** tamanho, opacidade e suavização, com 4 presets rápidos;
  - **centro:** a cor.
- Undo e Redo ficam ao lado da roda.
- **Arrastar a roda até à borda transforma-a numa barra** (no iOS).
- Três modos de interface: normal, compacto e **oculto** (toque no ícone para alternar). Os elementos podem ser arrastados para outra posição.

**Status bar no topo:** Gallery, Precision (grid, snap, medidas, guias de forma), Layers, Objects, Import/Export, Settings.

**Seleção** ([selection](https://concepts.app/manual/selection))
- **Tocar e segurar em qualquer lugar abre a seleção.** O dedo também pode ser configurado para sempre selecionar enquanto a caneta desenha.
- Popup acima da seleção: Clipboard, Duplicate, Group, Lock, Delete, Flip H/V.
- Interruptores de transformação: Rotate on/off, Scale, Stretch ou Off.
- Com algo selecionado, **tocar noutra ferramenta da roda muda o tipo de pincel dos traços selecionados**, e tocar no centro muda a cor deles ([busca/concepts](https://concepts.app/tutorials/select-edit-notes-drawings-designs)). **A mesma roda serve para desenhar e para editar.**

**Outras ferramentas**
- **Nudge:** empurra o traço, ou puxa-o tocando nele, "like a piece of string" ([nudge](https://concepts.app/en/tutorials/nudge-tool/)).
- **Forma por pausa:** desenhar de 1 a 4 traços e **segurar no fim** reconhece a forma ([basics](https://concepts.app/en/learn-the-basics-of-concepts/)).

**Lição de produto** ([10 anos](https://concepts.app/en/stories/designing-concepts-10-years-of-creative-development/))
- Quando lançaram a roda, "Half of our users fell in love… The other half panicked and requested the classic toolbar back". A equipe adiou o lançamento e passou a oferecer as duas.

---

## 7. Outros com UX marcante

- **Graphic for iPad (Autodesign)** ([gestures](https://www.graphic.com/docs-ipad/gestures)):
  - arrastar e segurar um 2.º dedo equivale a **Shift**;
  - arrastar e segurar **dois dedos** equivale a **Shift+Option**;
  - tocar e segurar abre o menu de contexto.
- **Infinite Painter** ([gestures](https://docs.infinitestudio.art/painter/studio/gestures/)):
  - 3 dedos para cima e para baixo mudam o tamanho; para os lados, a opacidade;
  - 4 dedos escondem a UI, e as "mini-toolbars" da ferramenta ficam visíveis;
  - **Touch-Alt:** um dedo na tela enquanto a caneta desenha muda a função da ferramenta (restringir forma, snap direcional);
  - a toolbar é personalizada arrastando ícones até uma "station" no topo ([Creative Bloq](https://www.creativebloq.com/advice/infinite-painter)).
- **Sketchbook** ([puck](https://help.sketchbook.com/docs/using-the-brush-puck)):
  - **Double Puck** flutuante: a metade de cima muda o tamanho a arrastar na horizontal e a opacidade na vertical; a metade de baixo muda a saturação e o brilho da cor;
  - **Lagoon** com marking menus radiais de 8 itens, que funcionam com um "flick" sem esperar o menu aparecer ([busca](https://design.tutsplus.com/vi/tutorials/the-beginners-guide-to-sketchbook-pro--cms-30592)).
- **Amadine:** painéis presos ou soltos. Não encontrei nenhuma UX marcante ([amadine.com](https://amadine.com/useful-articles/what-you-need-to-know-about-amadine-for-ipad-iphone)).
- **Vectorworks:** é CAD e não tem relevância aqui (não pesquisei).

---

## 8. Nomad Sculpt

**Regiões da tela** ([getting started](https://nomadsculpt.com/manual/gettingstarted))
- **Topo:** à esquerda, cena e objeto; à direita, menus da ferramenta.
- **Barra esquerda:** **sliders verticais de raio e intensidade**, o botão **Sub** (comportamento oposto, com rótulo que muda por ferramenta) e atalhos de simetria, máscara, smooth, gizmo e pintura.
- **Barra direita:** as ferramentas, numa área que rola.
- **Base:** undo/redo, histórico, solo, x-ray, grid e wireframe. Pressão longa abre mais opções.

**Personalização** ([interface](https://nomadsculpt.com/manual/interface))
- Espelhar a barra do topo, as laterais e a de baixo.
- Escala da UI, largura de painel, tamanho de fonte.
- Painéis transparentes com blur.
- **Toolbars flutuantes criadas pelo usuário**, com qualquer função ligada a botão, tecla, stylus ou gamepad.

**Gestos:** arrastar no vazio orbita, 2 dedos fazem pan e zoom, toque com 2 dedos desfaz.

**Elogio:** "What Procreate did for… 2D painting… Nomad does for 3D sculpting" ([illustrarch](https://illustrarch.com/articles/design-softwares/93842-nomad-sculpt-review.html)).

---

## Padrões transferíveis para um app desktop + iPad

1. **Ferramentas numa coluna estreita à esquerda** (Affinity, Illustrator, Linearity), com variantes por **pressão longa ou duplo toque** no ícone.
2. **Coluna de ícones à direita que abre painéis** (Studio do Affinity). Recolhida, é só uma faixa; no iPad mini ela rola, no 12,9" cabe inteira.
3. **Uma barra contextual numa borda só, que rola** (base no Affinity, topo no Fresco 7.3). Mostra as opções da ferramenta ou da seleção e nunca exige abrir um painel para o caso comum.
4. **Barra flutuante colada à seleção**, abaixo ou acima do objeto: Quick Actions do Linearity, Common Actions do Illustrator, popup do Concepts. Leva duplicar, apagar, agrupar, espelhar e trancar.
5. **Modificadores na tela num widget flutuante movível que lembra a posição**: Command Controller ou Touch Shortcut, com estados **segurar / travar**. Travado, fica visível em repouso como lembrete.
6. **O mesmo efeito por um dedo extra no canvas**, onde a mão está: +1 dedo = Shift (proporção ou eixo), +2 = Shift+Alt ou snap de 45° (Graphic, Linearity, Affinity). No desktop é a tecla.
7. **Mapa dos modificadores por ferramenta** no menu de ajuda (Touch Shortcut Map do Fresco) e uma **dica de rodapé** mostrando o que o primário e o secundário fazem com a ferramenta atual.
8. **Sidebar fina com 2 sliders verticais** (tamanho e opacidade, ou raio e intensidade) **e um botão modificador** (Modify, Sub) na borda da mão livre, com lado trocável (Procreate, Nomad). Num editor vetorial: largura do traço e opacidade, mais um botão "alt" (por exemplo, alternar entre nó de canto e nó suave).
9. **Menu radial invocado por gesto**, com toque-arrastar-soltar, 6 a 9 slots personalizáveis e perfis (QuickMenu do Procreate, Quick Menu do Affinity). No desktop: tecla mais cursor, ou botão do meio. **Ocupa o canvas sob o dedo e não tem região fixa.**
10. **Desenhar e segurar = forma perfeita**: elipse, retângulo, polígono e linha. +1 dedo torna-a regular, arrastar sem soltar escala e gira, e a forma sai com nós editáveis (QuickShape do Procreate, reconhecimento do Concepts). **Canvas, sem chrome.**
11. **Gestos universais de canvas:** toque com 2 dedos desfaz, com 3 refaz, com 4 alterna o modo foco, quick-pinch encaixa no ecrã. São consistentes entre todos os apps pesquisados.
12. **Modo foco** que esconde tudo menos uma mini-barra da ferramenta ativa (Infinite Painter, Concepts com o modo oculto).
13. **A UI sai do caminho durante o traço ou o arraste e volta ao soltar** (Magic Canvas do Linearity). Painéis e barras perdem opacidade ou recolhem.
14. **Editar a seleção pela mesma paleta que cria**: tocar numa cor ou num pincel com algo selecionado aplica a mudança à seleção (Concepts). Assim não é preciso um painel separado para a "cor do traço".
15. **Puck de arrasto em 2 eixos** (Double Puck do Sketchbook): horizontal = largura, vertical = opacidade, e uma segunda metade para a cor. Um widget pequeno flutuante, ou dentro da sidebar.
16. **Tocar e segurar no vazio = laço de seleção** (Concepts) e **tocar e segurar num objeto = menu de contexto** (Graphic). É o equivalente ao clique direito do desktop.
17. **Repetições com alças na tela** (radial, grade, espelho do Illustrator), editáveis pelo original. Entram pela barra contextual da seleção e ajustam-se no canvas.
18. **Ícones do Studio com modo compacto** (Layers e Brushes do Affinity) para o iPad mini: a mesma informação numa linha de miniaturas.
19. **Inspector que adapta as colunas à largura da janela** (2 colunas no Linearity em 2025). Uma coluna no 11" e no mini, duas no 12,9" e no desktop.
20. **Tocar e segurar num controle = dica** (Helper do Affinity, dicas com vídeo do Linearity). Substitui o hover que o dedo não tem; no desktop é o hover.
21. **Toolbars flutuantes criadas pelo usuário** (Nomad), arrastando qualquer comando da busca. Arrastar um ícone para uma "station" (Infinite Painter) ou para a Action Bar (Linearity).
22. **Lado da mão espelhável** para sidebar, barras laterais e barra de baixo (Nomad, Procreate), com o rótulo **"Canhoto"** e não "Right-hand".
23. **Atribuir gestos a funções numa tela de configuração com aviso de conflito ao vivo** (Gesture Controls do Procreate).
24. **Timeline num painel de base redimensionável**, com riscar para selecionar, pinça para zoom e 3 dedos para expandir (Procreate Dreams), e um **Flipbook flutuante** para desenhar quadro a quadro.

## Armadilhas

- **Controles no centro do topo e no centro da base** ficam por baixo do widget "⋯" de janelas do iPadOS e do indicador de Home ([MPU](https://talk.macpowerusers.com/t/affinity-publisher-2-for-ipad-mini-review/31514)).
- **"Mais espaço" encolhe os alvos de toque** abaixo do usável, até no 13". A densidade tem de seguir o tamanho físico do alvo, não a resolução ([MPU](https://talk.macpowerusers.com/t/affinity-publisher-2-for-ipad-mini-review/31514)).
- **Impor um único idioma de UI divide a base de usuários**: metade odiou a roda do Concepts, e foi preciso oferecer roda e barra ([Concepts](https://concepts.app/en/stories/designing-concepts-10-years-of-creative-development/)).
- **Mover barras numa atualização quebra a memória muscular** (Fresco 7.3, **não confirmado**). Uma mudança de layout deve vir com a opção de voltar ao antigo.
- **Gestos invisíveis não se descobrem.** O Dreams é criticado por ser "gesture-heavy" sem tutorial, e o Procreate não dá nenhum retorno visível do autosave ([TechRadar](https://www.techradar.com/computing/software/procreate-dreams-isnt-quite-the-revolution-i-wanted-for-animation-but-im-holding-out-hope), [Pratt](https://ixd.prattsi.org/2025/09/design-critique-procreate-ipad-app/)). Cada gesto precisa de um caminho alternativo visível (botão ou menu) e de uma folha de ajuda.
- **O mesmo gesto com resultados diferentes em ecrãs diferentes** quebra o modelo mental (o deslizar para a esquerda do Procreate) ([Pratt](https://ixd.prattsi.org/2025/09/design-critique-procreate-ipad-app/)).
- **Rótulos de lateralidade invertidos**: "Right-hand interface" para canhotos ([Pratt](https://ixd.prattsi.org/2025/09/design-critique-procreate-ipad-app/)).
- **Gestos que mudam de comportamento entre versões**: a pressão longa num nó deixou de apagar no Illustrator para iPad ([community](https://community.adobe.com/t5/illustrator-on-the-ipad-discussions/illustrator-for-ipad-using-direct-select-tool-long-press-an-anchor-point-doesn-t-delete-it-anymore/m-p/13322711/highlight/true)).
- **Trackpad e rato esquecidos no iPad**: não rolar o canvas com 2 dedos no trackpad foi chamado "a huge issue" ([community](https://community.adobe.com/t5/illustrator-discussions/trouble-with-illustrator-on-ipad/m-p/14214774)). No iPad os três modos de entrada (dedo, caneta e trackpad ou teclado) têm de funcionar.
- **Deselecionar com zoom alto** quando não há área vazia na tela ([community](https://community.adobe.com/t5/illustrator-on-the-ipad/illustrator-on-ipad-shortcut-to-deselect/m-p/11992100)). É preciso um gesto ou botão explícito para isso, por exemplo na barra da seleção.
- **Modificador na tela "segurado" ocupa a mão livre** e não deixa usar gestos ao mesmo tempo. Por isso todos acrescentaram a **trava** (Affinity com Held/Locked, Illustrator a partir de 2022, Fresco com duplo toque). Estado travado sem indicação visível em repouso é um erro garantido. O Affinity resolve mostrando os travados quando o widget está recolhido.
- **Atalhos em conflito silencioso**: copie o aviso amarelo do Procreate.
- **Não conte com o Affinity unificado para iPad como referência**: ainda não saiu, e a UI dele no iPad é desconhecida (não confirmado).
