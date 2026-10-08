# Guias, snap e alinhamento — Inkscape · Affinity Designer 2 · Illustrator · Godot 4 (+ Canva, Lucid)

- **Data:** 2026-10-08
- **Método:** leitura de documentação OFICIAL apenas; nenhum código-fonte de app lido.
  - **Inkscape (1.4.4 instalado, `pacman -Qi inkscape` → `1.4.4-6.1`, `inkscape --version` →
    `Inkscape 1.4.4 (dcaf3e7d9e, 2026-05-05)`):** notas de versão oficiais 1.2/1.3/1.4 lidas em wikitext
    cru (`https://wiki.inkscape.org/wiki/index.php?title=Release_notes/<v>&action=raw`); os tutoriais
    oficiais que vêm com o app (`/usr/share/inkscape/tutorials/tutorial-basic.svg`,
    `tutorial-tips.svg`, `tutorial-advanced.svg`, os de *Ajuda → Tutoriais*); o manual de referência de
    Tavmjong Bah (http://tavmjong.free.fr/INKSCAPE/MANUAL/html/, ligado a partir de inkscape.org, escrito
    para a 0.48/0.92 — **marcado [manual 0.92]**: pode estar desatualizado na 1.x). As preferências
    padrão vieram de `~/.config/inkscape/preferences.xml` e do esqueleto de preferências embutido no
    binário instalado (não há `preferences-skeleton` em `/usr/share/inkscape`). Rótulos visíveis da UI
    (popover de snap `/usr/share/inkscape/ui/toolbar-snap.ui`, textos da barra de estado e do diálogo de
    Preferências extraídos por `strings` do binário) estão marcados **[artefacto instalado]**: são texto
    que o utilizador lê na tela, não código. `inkscape.org/doc/keys.html` e `inkscape.org/release/*`
    respondem 403 a fetch automático e NÃO foram lidos.
  - **Affinity Designer 2:** `https://affinity.help/designer2/en-US.lproj/pages/…` (Snapping, Dynamic
    guides, Rotating and shearing, Transforming, Distributing, Curve/Construction snapping, Guides,
    Artboards, Constraints, Move Tool). As teclas de modificador nessas páginas são desenhadas por CSS
    (`<span class="key SHIFT-TAG">`); aparecem aqui como `[SHIFT]`, `[ALT]`, `[CMD]`, `[CTRL]`. As páginas
    trazem a variante macOS e Windows do mesmo parágrafo.
  - **Illustrator:** `https://helpx.adobe.com/illustrator/desktop/…` (Smart Guides options, Work with
    Smart Guides, perpendicular, distance guides, transform, rotate, selection, keyboard shortcuts) —
    páginas «Last updated» Oct 2025 – May 2026.
  - **Godot 4:** `https://docs.godotengine.org/en/stable/tutorials/2d/introduction_to_2d.html`
    («Godot Engine (stable)», 4.x) + a referência `EditorSettings`.
  - **Canva / Lucid:** um artigo oficial de cada (guias; definições do documento).
- **Leitura da coluna «lei»:** «silente» = a documentação não diz; nada foi adivinhado. Onde uma lei é
  INFERÊNCIA a partir do texto, está escrito «inferência».

---

## 1. Inkscape (1.2+)

| # | pergunta | lei | citação literal | URL / fonte |
|---|---|---|---|---|
| 1 | Rodado: AABB ou arestas rodadas? | **Ambos, como categorias SEPARADAS**: «Bounding boxes» (cantos, arestas, meios, centro da caixa) e «Nodes»/«Paths» (nós e caminhos reais — num retângulo rodado, os cantos rodados são nós). Uma não snapa na outra. A caixa é visual (com traço) ou geométrica (só nós), por preferência | «Note that snapping of bounding boxes is independent of snapping of nodes (i.e., a node cannot be directly snapped to a point on a bounding box and vice versa). Both bounding boxes and nodes, however, can be snapped to the page boundary, Guide Lines and Grids.» **[manual 0.92]** | http://tavmjong.free.fr/INKSCAPE/MANUAL/html/Snapping.html |
| 1 | (modo simples 1.2) | o popover simples tem só caixas + caminhos + alinhamento | «Simple: Only 3 options: snap bounding boxes and paths, activate / deactivate the new alignment snapping).» | https://wiki.inkscape.org/wiki/Release_notes/1.2 |
| 1 | visual × geométrica | duas definições de caixa | «The Visual bounding box includes the stroke width if the stroke is visible. […] The Geometric bounding box mode uses only the nodes to determine the bounding box.» **[manual 0.92]** | http://tavmjong.free.fr/INKSCAPE/MANUAL/html/Transforms.html |
| 1 | AABB é alinhada aos eixos? | **silente** na letra; o texto só descreve a caixa como retângulo de 8 alças e as guias de alinhamento como horizontais/verticais (inferência: caixa nos eixos do documento) | — | — |
| 1 | categorias do popover | **[artefacto instalado]** opções: Bounding boxes · Edges · Corners · Edge midpoints · Centers · Nodes · Paths · Path intersections · Cusp nodes · Smooth nodes · Line midpoints · Object midpoints · Object rotation centers · Perpendicular lines · Tangential lines · Alignment · Same distances · Nodes in same path · Page borders · Page margins · Grids · Guide lines · Text baselines and anchors · Clips · Masks | rótulos do popover | `/usr/share/inkscape/ui/toolbar-snap.ui` |
| 2 | Guias no ângulo de um objeto rodado | **Sim, por comando**: *Object → Object to Guides* (`Shift+G`) põe guias ao longo das ARESTAS de um retângulo rodado (com a preferência ligada) e ao longo de cada segmento reto de um caminho | «Rectangles and Boxes: Guide Lines are drawn along edges, even when rotated, if the Conversion to guides uses edges instead of bounding box box is checked […] Otherwise the bounding box is used. Paths: A Guide Line is drawn along each straight line segment.» **[manual 0.92]** | http://tavmjong.free.fr/INKSCAPE/MANUAL/html/Snapping.html |
| 2 | guias angulares | uma guia é âncora + ângulo; pode ser rodada à mão com passo do snap de rotação | «Guide Lines […] are defined by an x - y anchor (origin point) through which the line passes and an angle.» · «Ctrl + Shift + Left Mouse Drag : Rotate Guide Line with constraint. Angle snaps to multiples of the Rotation snap angle , (15 degrees, by default).» **[manual 0.92]** | idem |
| 2 | guias temporárias de alinhamento | **só horizontais ou verticais** (nunca no ângulo do objeto) | «Inkscape will display horizontal or vertical temporary guide lines that indicate when the selected object can be aligned relative to another object on the canvas. It connects the points of the objects that are in alignment.» | https://wiki.inkscape.org/wiki/Release_notes/1.2 |
| 2 | perpendicular/tangente | existem, movidos para o popover na 1.2 | «The options for snapping perpendicularly and tangentially to paths or guide lines have been moved from the document preferences to the snap popover» | idem |
| 2 | desenhar guias (1.4) | nova opção: a forma desenhada vira guias | «A new option has been added to automatically convert the currently drawn shape to guides, allowing users to 'draw' guides instead of dragging them out of the rulers.» | https://wiki.inkscape.org/wiki/Release_notes/1.4 |
| 3 | Passo do snap de rotação | **Ctrl = passos de 15°**; teclas `[` `]` também 15°, `Ctrl+[ ]` 90° | «Rotate the object by dragging the corner handles. (Press Ctrl to restrict rotation to 15 degree steps. […])» · «Default rotates are by 15 degrees; with Ctrl , you rotate by 90 degrees.» | `/usr/share/inkscape/tutorials/tutorial-basic.svg` (Ajuda → Tutoriais → Basic) |
| 3 | preferência | **`rotationsnapsperpi = 12`** (12 por π = 15°); rótulo na UI «Rotation snaps every:» | `<group id="rotationsnapsperpi" value="12"/>` | `~/.config/inkscape/preferences.xml` (linha 732) e o esqueleto padrão embutido em `/usr/lib/inkscape/libinkscape_base.so.1.4.4.0` **[artefacto instalado]** |
| 3 | snap ao ângulo de OUTROS objetos | **silente** (nada além do passo fixo e das guias perpendicular/tangente) | — | — |
| 3 | como o ângulo aparece | na barra de estado, em graus com 2 casas | «<b>Rotate</b>: %0.2f°; with <b>%s</b> to snap angle» **[artefacto instalado]** (texto da barra de estado) | `libinkscape_base.so.1.4.4.0` |
| 4 | Contentores (grupos/camadas/páginas) como alvo | **grupos/camadas: silente** para snap. Páginas: bordas, margens e sangrias são alvos (1.3) | «Page margins and bleeds are available as snapping targets, and can be de-/activated with the 'page borders' snap control.» | https://wiki.inkscape.org/wiki/Release_notes/1.3 |
| 5 | Dentro de pai rodado: eixos locais? | **silente** | — | — |
| 6 | Espaçamento igual ao arrastar | **Sim** — «Same distances» (distribuição, 1.2), considera vários objetos próximos; rótulo de distância opcional | «With distribution snapping (option Same distances), multiple objects close by are taken into account, making it possible to align objects in a grid with very little effort.» · «Alignment and Distribution snapping guide lines can display the distance(s) between objects as a little label. This can be enabled from Edit → Preferences → Snapping: Show snap distance in case of alignment or distribution snap.» | https://wiki.inkscape.org/wiki/Release_notes/1.2 |
| 6 | restrição depois do snap | a guia prende frouxamente a direção | «Once a guide shows up, the movement of the selection is loosely constrained in the direction of the guide.» | idem |
| 7 | Modificador que suspende | **[manual 0.92]** Shift durante o arrasto. **[artefacto instalado]** a barra de estado da 1.4 diz «with <b>%s</b> to disable snapping» (tecla preenchida em tempo de execução). Global: `%` | «Shift + Left Mouse Drag : Temporarily disable snapping to Grids or Guide Lines .» · «To activate / deactivate snapping globally, click on the snap symbol in the top right corner or press %.» | Transforms.html (manual) · Release_notes/1.2 |
| 7 | Tolerância | antes: «Snap Distance» por documento; **1.2 removeu as tolerâncias por tipo de alvo**; valor padrão atual: **silente** | «The options to configure snap tolerances per snap target type that previously could be set in the document preferences have been removed to simplify snapping for users.» | Release_notes/1.2 |
| 7 | limitar alvos | «Only snap the node closest to pointer»; atraso; peso; e (1.4) grade/guias podem ser alvo ÚNICO. Padrões: `snapclosestonly=0`, `snapdelay=0`, `snapweight=0.5`, `snapindicator=1` | «Only snap the node closest to pointer : If this option is enabled, only one node is used as a snapping point […] Useful if an object has many snapping points. Note: if an object has too many nodes, and this option is disabled, a “convex hull” is used for snapping.» · «Delay (in ms): […] Useful if there are many snapping targets» **[manual 0.92]** · «More options for finetuning snapping precedence between grid and guidelines vs. objects and alignment/distribution […] These allow you to set grids and / or guidelines as the only available snap target» | Snapping.html (manual) · Release_notes/1.4 · `preferences.xml` |
| 7 | limite de desempenho | **[artefacto instalado]** mensagem de aviso: há um tecto de 200 caminhos-alvo | «Warning: limit of 200 snap target paths reached, some will be ignored» | `libinkscape_base.so.1.4.4.0` |

## 2. Affinity Designer 2

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 1 | Rodado: AABB ou geometria? | **Ambos, opções separadas**: «Snap to object bounding boxes» e «Snap to object geometry» (vértices: cantos, interseções). Se a caixa de um objeto rodado é a AABB ou a caixa rodada: **silente** | «Snap to object bounding boxes —when checked, objects can be aligned based on its bounding box.» · «Snap to object geometry —when checked, curves can be snapped to vertices and not simply to the bounding box or key points. Vertices are object corners or intersections, such as the points of a star, within a shape's bounding box.» | https://affinity.help/designer2/en-US.lproj/pages/DesignAids/snapping.html |
| 1 | pontos-chave | «key points» = pontos automáticos (ex.: início/fim de canto arredondado) | «Snap to shape key points —when checked, objects can be aligned to key points on shapes, such as the start and end of a rounded corner.» | idem |
| 2 | Guias no ângulo do objeto | **silente para objetos**. Guias dinâmicas: vermelho = horizontal, verde = vertical. Só em CURVAS há «construction snapping» angular (paralelas, 90°, refletido) — para alças de nó | «Red line: Object snaps to target horizontally. Green line: Object snaps to target vertically. Yellow node: Object snaps to shape's key points (often centers) or geometry.» · «Construction snapping lets you snap a node's control handle to useful construction angles relative to the adjacent node and opposing control handle.» | snapping.html · https://affinity.help/designer2/en-US.lproj/pages/DesignAids/constructionSnapping.html |
| 2 | guias dinâmicas | aparecem ao arrastar, para bordas, centros, vértices e elementos da página | «As you drag objects to help you align and resize objects to existing object edges, centers, vertices, and page elements (e.g. page margins). These guides display in red when aligning horizontally and in green when aligning vertically.» | https://affinity.help/designer2/en-US.lproj/pages/DesignAids/dynamicGuides.html |
| 3 | Passo da rotação | **Shift = 15°** (Move Tool); Point Transform Tool: Shift+Cmd = 15° | «The [SHIFT] constrains rotation to 15° increments.» · «[SHIFT][CMD] rotates the object only (in 15° intervals).» | https://affinity.help/designer2/en-US.lproj/pages/ObjectControl/rotateShear.html · …/ObjectControl/transform.html |
| 3 | ângulo de OUTRO objeto | **não por snap**, mas por comando: «Make Same → Rotation» copia a rotação do objeto-chave (Alt-clique) | «To rotate objects to a targeted rotated key object: […] With the [ALT] pressed, click the target rotated object. […] On the Toolbar, click Alignment , then set the Make Same option, choosing Rotation .» | rotateShear.html |
| 3 | origem da rotação snapa | a origem de transformação snapa a caixa, centro, pontos-chave ou geometria | «You can snap the origin to the bounding box, center, key points, or the geometry of other objects (or even the same object).» | rotateShear.html |
| 3 | como o ângulo aparece | **silente** na página de rotação (só «Transform panel … R —Rotation») | «R —Rotation. Rotates the object (layer) by a specified number of degrees in relation to the selected anchor point.» | https://affinity.help/designer2/en-US.lproj/pages/Panels/transformPanel.html |
| 4 | Artboards como alvo | **Sim** — bordas, cantos, meios do artboard atual; outros artboards se forem candidatos | «When snapping is active, dynamic guide lines appear when an object is aligned to the edges, corners, or midpoints of the current artboard. You can likewise snap to other document artboards, if they have been established as snapping candidates.» | https://affinity.help/designer2/en-US.lproj/pages/Artboards/artboards_designAids.html |
| 4 | camadas como fronteira de candidatos | «Immediate layers» limita os candidatos à camada atual, irmãs e pai imediato | «Immediate layers —limits the number of candidates to only the current layer, layers that are siblings or the immediate parent layer.» | snapping.html |
| 4 | artboard pode rodar | sim (Artboard Tool / Transform panel). Snap de um artboard rodado: **silente** | «Artboards can also be rotated and sheared directly on the pasteboard using the Artboard Tool or using precision edits using the Transform panel .» | https://affinity.help/designer2/en-US.lproj/pages/Artboards/artboards_moveResize.html |
| 5 | Pai rodado: eixos locais? | **silente** (Constraints ancoram o filho «in relation its container», só para escala/âncora) | «A child object's scaling and anchoring is always in relation its container.» | https://affinity.help/designer2/en-US.lproj/pages/DesignAids/constraints.html |
| 6 | Espaçamento igual ao arrastar | **Sim** — «Snap to gaps and sizes» (setas mostram gaps e tamanhos iguais); guias dinâmicas com rótulo de distância | «Snap to gaps and sizes —when checked, arrows represent matched gaps between snapping candidates and matched horizontal and/or vertical sizes.» · «Dynamic guides also include labels which report the distance between the snapping objects (measured in the document's set units).» | snapping.html |
| 7 | Modificador que suspende | **Alt** (renderizado `[ALT]`) enquanto posiciona | «Press the [ALT] while you're positioning an object. Snapping won't occur while the [ALT] is depressed.» (tecla por CSS `class="key ALT-TAG"`) | snapping.html |
| 7 | Tolerância | «Screen tolerance» (distância de ecrã); valor padrão **silente** | «Screen tolerance —controls the distance you have to be to an object before snapping occurs.» | snapping.html |
| 7 | limitar alvos + desempenho | **Lista de candidatos com máximo** (FIFO: os novos substituem os antigos); candidato = criado ou pairado; aviso de desempenho com «All layers» | «Candidate List —limits the number of objects which are snapping candidates to the number you set. Creating a new object, or hovering over an existing object, designates it as a snapping candidate in this case. Only the active snapping candidates can be snapped to.» · «Maximum —limits the number of active candidates when Candidate List is selected (see above). If you reach this limit, new candidates replace older candidates in chronological order.» · «If you have a document with a high number of objects or layers and activate the All layers snapping candidate option, you may experience performance issues when moving or resizing objects due to the shear number of snapping candidates.» | snapping.html |
| 7 | excluir objeto | por objeto, no painel Layers | «On the Layers panel, -click the object entry and select Exclude From Snapping .» | snapping.html |

## 3. Adobe Illustrator

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 1 | Rodado: AABB ou geometria? | **pela GEOMETRIA** do objeto (bordas e centro). A caixa de seleção RODA com o objeto (há comando para reorientá-la) | «When moving an object or artboard, use the Smart Guides to align the selected object or artboard to other objects or artboards based on their geometry. The alignment is based on the geometry of objects and artboards. Guides appear as the object approaches the other objects' edge or center point.» | https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/work-with-smart-guides.html |
| 1 | caixa rodada | a caixa acompanha a rotação; *Reset Bounding Box* volta-a aos eixos | «To reorient the bounding box after you rotate it, select Object > Transform > Reset Bounding Box .» | https://helpx.adobe.com/illustrator/desktop/manage-objects/reshape-transform-objects/transform-objects.html |
| 1 | Alignment Guides | centro e bordas de objetos geométricos, artboard e sangria | «Alignment Guides — Displays guides along the center and edges of geometric objects, artboard, and bleeds.» | https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/smart-guides-options.html |
| 2 | Guias em ângulo | **Construction Guides**: ângulos FIXOS escolhidos nas preferências (até 6; ex. 45/90), ao desenhar/transformar com Pen/Shaper. Guia ao ângulo de um objeto rodado: **silente** | «Construction Guides — Displays guides at specified angles as you draw new objects. Construction Guides support up to six angles.» · «use Smart Guides construction guides to position anchor points to specific preset angles, such as 45 or 90 degrees.» | smart-guides-options.html · work-with-smart-guides.html |
| 2 | perpendicular | snap perpendicular só entre caminhos RETOS e só nas pontas | «Snapping is supported only between two linear paths. You cannot snap a line perpendicularly to a curved path.» · «Snapping is limited to line endpoints only.» | https://helpx.adobe.com/illustrator/desktop/measure-and-align/plot-and-measure/snap-to-perpendicular.html |
| 2 | objeto → guia | qualquer vetor pode virar guia (*View › Guides › Make Guides*) — logo uma linha rodada vira guia rodada (inferência) | «You can create ruler guides (straight vertical or horizontal lines) and guide objects (vector objects converted to guides).» | https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/align-graphic-objects-with-guides.html |
| 3 | Passo da rotação | **silente** para a rotação por arrasto nas páginas atuais; Shift restringe MOVIMENTO a 45°. Eixos podem ser rodados por *Constrain Angle* (só citado) | «Constrain movement to 45° angle (except when using Reflect tool) — Hold down Shift» · «General : Configure general application behavior and default settings, such as keyboard increment, constrain angle, and corner radius.» | https://helpx.adobe.com/illustrator/using/default-keyboard-shortcuts.html · https://helpx.adobe.com/illustrator/desktop/get-started/preferences-and-settings/set-app-preferences.html |
| 3 | como o ângulo aparece | Smart Guides «Transform Tools» mostram o valor ao rodar | «Transform Tools — Displays transformation values when scaling, rotating, or shearing objects.» | smart-guides-options.html |
| 3 | ângulo de OUTRO objeto | **silente** | — | — |
| 4 | Artboards como alvo | **Sim**; o próprio artboard também snapa ao ser arrastado | «They help you align, edit, and transform objects or artboards relative to other objects, artboards, or both by snap-aligning and displaying X, Y location, and delta values.» | work-with-smart-guides.html |
| 4 | distância ao artboard pai | Alt/Option mostra distâncias ao artboard | «Invoke Distance Guides — Displays the distance between two objects and the position of an object relative to its parent artboard.» · «Move the cursor over an empty area of the artboard while pressing Alt (Windows) or Option (macOS)» | smart-guides-options.html · https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/use-distance-guides-for-accurate-placement.html |
| 4 | grupos | **silente** | — | — |
| 5 | Pai rodado: eixos locais? | **silente** | — | — |
| 6 | Espaçamento igual | **Sim** — Spacing Guides | «Spacing Guides — Displays guides to help evenly space objects as you move them.» | smart-guides-options.html |
| 7 | Suspender | liga/desliga Smart Guides `Ctrl/Cmd+U`; modificador de suspensão temporária: **silente**. Snap to Grid ou Pixel Preview DESLIGAM as Smart Guides | «Show/ hide smart guides — Ctrl + U — Command + U» · «If you enable Snap to Grid or Pixel Preview in the View menu, you cannot use Smart Guides .» | default-keyboard-shortcuts.html · work-with-smart-guides.html |
| 7 | Tolerância | em **pontos** (unidade do documento, não px de ecrã); padrão **silente** | «Snapping Tolerance — Specifies the distance (in points) within which Smart Guides activate when the pointer comes close to another object.» | smart-guides-options.html |
| 7 | (seleção, não snap) | tolerância de seleção de âncora = 3 px | «Tolerance — Sets how close the pointer must be to an anchor point for selection. The default value is 3 pixels.» | https://helpx.adobe.com/illustrator/desktop/manage-objects/select-objects/object-selection-methods-and-preferences.html |
| 7 | limitar alvos / desempenho | **silente** | — | — |

## 4. Godot 4 (editor 2D)

| # | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| 1 | Rodado: AABB ou arestas rodadas? | **silente** (só «sides», «center», «anchor» do nó) | «Snap to Node Sides: Snaps to the node's sides, such as for the rotation pivot or anchor positioning. Snap to Node Center: Snaps to the node's center […] Snap to Other Nodes: Snaps to other nodes while moving or scaling. Useful to align nodes in the editor.» | https://docs.godotengine.org/en/stable/tutorials/2d/introduction_to_2d.html |
| 2 | Guias em ângulo | **não**: guias só horizontais/verticais, arrastadas da régua | «you can create guides by dragging from the horizontal or vertical ruler towards the viewport. A purple guide will appear» | idem |
| 3 | Passo da rotação | **Rotation Step** configurável + **Rotation Offset**; o doc dá 15 como EXEMPLO (padrão: silente) | «Rotation Offset: Sets the offset to shift rotational snapping. Rotation Step: Defines the snapping degree. E.g., 15 means the node will rotate and snap at multiples of 15 degrees if rotation snap is enabled and the rotate mode is used.» | idem |
| 3 | onde vale | Select Mode NÃO usa snap na rotação/escala; só Rotate Mode | «In this mode, rotation and scaling will not use the snapping options if snapping is enabled.» | idem |
| 3 | ângulo de outro nó / como aparece | **silente** (a régua mostra ângulos aos eixos, não a rotação) | «If you drag diagonally, it will draw a triangle and show the separate distances in terms of x, y, and total distance to the target, including the angles to the axes in degrees.» | idem |
| 4 | Contentor como alvo | **Sim**: «Snap to Parent» (bordas do pai) | «Snap to Parent: Snaps to parent's edges. For example, scaling a child control node while this is enabled will snap to the boundaries of the parent.» | idem |
| 5 | Pai rodado: eixos locais? | **silente**. «Snap Relative» desloca a GRELHA pela posição atual do nó (não pela rotação) | «Snap Relative : Toggles the usage of snapping based on the selected node's current transform values. For example, if the grids are set to 32x32 pixels and if the selected node is located at x: 1, y: 1 , then, enabling this option will temporarily shift the grids by x: 1, y: 1 .» | idem |
| 6 | Espaçamento igual | **silente** | — | — |
| 7 | Modificador | Shift ao arrastar = snap num só eixo; modificador de suspensão: **silente** | «keeping Shift pressed while dragging a node in the Select or Move modes will try to snap the node in a single axis while moving.» | idem |
| 7 | linhas de smart snap | cor configurável; aparecem ao mover | «The smart snapping lines will automatically display when moving 2D nodes if smart snapping is enabled in the Snapping Options menu at the top of the 2D editor viewport.» | https://docs.godotengine.org/en/stable/classes/class_editorsettings.html (`editors/2d/smart_snapping_line_color`) |
| 7 | tolerância / limites | **silente** (nenhuma `EditorSettings` de distância de snap 2D) | — | class_editorsettings.html |

## 5. Extras de quadro branco (só o que o oficial diz)

| app | pergunta | lei | citação literal | URL |
|---|---|---|---|---|
| Lucid (chart/spark) | alvos | snap a objetos, grelha e guias (toggles por documento); guias só horizontais/verticais | «Snap to objects : Toggle on “Snap to objects” to force objects to align with other objects on the workspace.» · «To add a guide, right-click anywhere on the canvas and select Create Horizontal Guide or Create Vertical Guide.» | https://help.lucid.co/hc/en-us/articles/15578781626772-Adjust-document-and-board-settings |
| Lucid | rodados, rotação, espaçamento, modificador, tolerância | **silente** no artigo oficial | — | — |
| Canva | guias | guias predefinidas (12 col., 3 col., 3×3) ou à mão, travadas por padrão; snap/rotação **silentes** no artigo | «By default, preset and custom guides are locked to prevent unintentional edits, but can be unlocked if desired.» | https://www.canva.com/help/adding-and-removing-guides/ |

---

## 6. Síntese por pergunta (o que é comum e o que diverge)

1. **Rodado:** Inkscape e Affinity separam «caixa» de «geometria/nós» como categorias independentes;
   Illustrator alinha pela geometria e a caixa roda com o objeto; Godot silente.
2. **Guias no ângulo:** nenhuma das guias TEMPORÁRIAS documentadas segue o ângulo do objeto (Inkscape,
   Affinity, Godot, Lucid: H/V). Ângulo só por: Inkscape «Object to Guides» (arestas rodadas),
   Illustrator «Make Guides» e Construction Guides de ângulos FIXOS, Affinity construction snapping só
   para alças de curva.
3. **Rotação:** 15° em Inkscape (Ctrl, `rotationsnapsperpi=12`), Affinity (Shift) e Godot (Rotation
   Step configurável, 15 só como exemplo); Illustrator silente na rotação por arrasto. Snap ao ângulo de
   OUTRO objeto: nenhum documenta; Affinity copia a rotação por comando (Make Same).
4. **Contentores:** Godot «Snap to Parent»; Affinity e Illustrator artboards são alvos; Inkscape páginas
   (bordas, margens, sangrias).
5. **Pai rodado / eixos locais:** silente em todos.
6. **Espaçamento igual:** Inkscape «Same distances», Affinity «Snap to gaps and sizes», Illustrator
   «Spacing Guides»; Godot/Lucid/Canva silentes.
7. **Suspender / tolerância / limites:** Affinity Alt; Inkscape Shift (manual 0.92 + barra de estado);
   Illustrator só liga/desliga (`Ctrl+U`); Godot silente. Tolerância: Affinity em ecrã («Screen
   tolerance»), Illustrator em pontos, Inkscape removeu as por-alvo na 1.2. Limites de alvos: Affinity
   lista de candidatos com máximo FIFO + aviso de desempenho; Inkscape «só o nó mais perto do ponteiro»,
   atraso, e o aviso instalado de 200 caminhos-alvo.
