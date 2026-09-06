//! **AS STRINGS DO PAINEL DE MODELAGEM 3D** (ADR-0161) — o irmão de tabela do [`super`].
//!
//! ⚠️ **Um arquivo por painel compra ISOLAMENTO**, que é o que o Modo L pede de um toque
//! foundational (`CLAUDE.md` §0.2): com todas as chaves num `match` só, duas linhas paralelas que
//! acrescentassem uma chave cada colidiriam nas mesmas linhas. É o mesmo corte que o `sculpt3d.rs`
//! já fez, pela mesma razão.
//!
//! ⚠️ **`Option` e não `&str`:** devolver a chave crua aqui seria uma segunda resposta a *"o que
//! fazer com uma chave desconhecida?"* — o `leak_key` do pai responde isso, uma vez.

/// A tradução de uma chave `panel.model3d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.model3d.title" => "3D Model",
        "panel.model3d.empty" => "Select an object to edit its dimensions.",
        // ⚠️ O rótulo diz **Radius**, e é um compromisso que o documento honra: quem escolher a
        // mistura orgânica vê um número que entrega 3/4 do que promete (ver `Blend::Organic`), e
        // isso é uma decisão de produto por tomar — não uma etiqueta a corrigir aqui.
        "panel.model3d.radius" => "Radius",
        // ⭐ Os três verbos do gizmo. ⚠️ **"Size", e não "Scale"** — a escala deste módulo é
        // UNIFORME por decisão medida (ADR-0161 §6: a não-uniforme destrói a propriedade de
        // distância), e "Scale" num app 3D promete três eixos. Um rótulo tem de prometer o que o
        // modelo entrega.
        "panel.model3d.mode.move" => "Move",
        "panel.model3d.mode.rotate" => "Rotate",
        "panel.model3d.mode.scale" => "Size",
        // Em que eixos o gizmo aponta. ⚠️ "Global"/"Local" são as palavras do Blender para a mesma
        // escolha — quem já modela sabe o que elas querem dizer sem experimentar.
        "panel.model3d.frame.global" => "Global",
        "panel.model3d.frame.local" => "Local",
        // ⭐⭐ O RÓTULO do pulldown que este módulo põe na fila de ferramentas (D2, a metade do
        // cabeçalho de área — ver `ph2d_panel_model3d::area_bar`).
        //
        // ⚠️ **O rótulo diz o GRUPO; a face diz o ESTADO** (`Front`, `User`). São perguntas
        // diferentes e por isso são dois textos: um chip cujo rótulo repetisse a face não diria o
        // que ele abre.
        //
        // ⛔ **Não há rótulo de *Gizmo*, e a ausência é a decisão:** os verbos do gizmo vivem nos
        // chips `MOVE`/`ROT`/`SCALE` que o trilho já pinta.
        "panel.model3d.area.view" => "View",
        // ⭐⭐⭐ A PORTA DE CRIAR (W100) — um botão, que abre a paleta de formas.
        //
        // ⚠️ As reticências dizem "isto abre alguma coisa" (a convenção que o `Sculpt…` já usava), e
        // o atalho vai no rótulo pela razão das vistas nomeadas: é a única forma de a tecla ser
        // descoberta por quem não sabe que ela existe.
        "panel.model3d.add.open" => "+ Add shape… (A)",
        // As formas do catálogo. ⚠️ **Sem o "+"** desde a W100: elas deixaram de ser botões de uma
        // fileira e passaram a ser ITENS de uma paleta, onde o "+" seria ruído repetido em 60
        // linhas — o verbo já está no título do modal ("Add Shape").
        "panel.model3d.add.box" => "Box",
        "panel.model3d.add.sphere" => "Sphere",
        "panel.model3d.add.cylinder" => "Cylinder",
        "panel.model3d.add.torus" => "Torus",
        // ⭐⭐ O LOTE DA W101. ⚠️ "Cone" e "Truncated Cone" são a MESMA primitiva com defaults
        // diferentes — o rótulo diz a forma que nasce, não o tipo interno.
        "panel.model3d.add.cone" => "Cone",
        "panel.model3d.add.cone_truncated" => "Truncated Cone",
        "panel.model3d.add.capsule" => "Capsule",
        // ⚠️ **Sem o número de lados no rótulo.** Ele nasce hexagonal e o primeiro controlo do
        // painel são os lados — pôr "Hexagonal Prism" aqui prometeria uma forma fixa, e o artista
        // procuraria "Octagonal Prism" numa lista que nunca o terá.
        "panel.model3d.add.prism" => "Prism",
        // ⭐⭐ O LOTE DA W102. ⚠️ A pirâmide e o tronco são o MESMO prisma com o topo estreitado —
        // o rótulo diz a forma que nasce, não o tipo interno.
        "panel.model3d.add.pyramid" => "Pyramid",
        "panel.model3d.add.pyramid_truncated" => "Truncated Pyramid",
        "panel.model3d.add.wedge" => "Wedge",
        "panel.model3d.add.torus_arc" => "Torus Arc",
        // ⭐⭐ O LOTE DA W103 — o fim da fila do doc 08. ⚠️ **"Star" sem o número de pontas**, pela
        // razão do prisma: ela nasce de 5 e o primeiro controlo do painel são as pontas, então
        // "5-Point Star" prometeria uma forma fixa e mandaria procurar "6-Point Star" numa lista
        // que nunca a terá.
        "panel.model3d.add.star" => "Star",
        "panel.model3d.add.box_frame" => "Box Frame",
        "panel.model3d.add.ellipsoid" => "Ellipsoid",
        "panel.model3d.add.octahedron" => "Octahedron",
        "panel.model3d.add.round_cone" => "Round Cone",
        "panel.model3d.add.cut_sphere" => "Cut Sphere",
        "panel.model3d.add.hollow_dome" => "Hollow Dome",
        "panel.model3d.add.link" => "Chain Link",
        "panel.model3d.add.solid_angle" => "Solid Angle",
        "panel.model3d.add.gear" => "Gear",
        "panel.model3d.add.cross" => "Cross",
        "panel.model3d.add.heart" => "Heart",
        "panel.model3d.add.moon" => "Moon",
        "panel.model3d.add.drop" => "Drop",
        "panel.model3d.add.pie" => "Pie",
        "panel.model3d.add.trapezoid" => "Trapezoid",
        "panel.model3d.add.vesica" => "Vesica",
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **"Diamond" e não "Rhombus"**: é a palavra do catálogo de fluxograma e a que um artista
        // procura na paleta; o nome geométrico fica no documento, onde o leitor é a próxima LLM.
        "panel.model3d.add.arrow" => "Arrow",
        "panel.model3d.add.double_arrow" => "Double Arrow",
        "panel.model3d.add.bent_arrow" => "Bent Arrow",
        "panel.model3d.add.chevron" => "Chevron",
        "panel.model3d.add.rhombus" => "Diamond",
        "panel.model3d.add.circle_segment" => "Circle Segment",
        // ⚠️ **Três portas, uma forma** — ver o construtor de cada uma: o tubo é alto, a anilha é
        // chata e o arco tem sector. *É a porta que o artista procura, não a fórmula.*
        "panel.model3d.add.tube" => "Tube",
        "panel.model3d.add.washer" => "Washer",
        "panel.model3d.add.ring_arc" => "Ring Arc",
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **"Speech Balloon" e não "Speech Rect"**: o rótulo diz o que a forma É para quem a
        // procura, e o nome geométrico fica no documento, onde o leitor é a próxima LLM.
        "panel.model3d.add.speech_rect" => "Speech Balloon",
        "panel.model3d.add.speech_oval" => "Speech Oval",
        // ⭐ **Duas portas, uma forma** — a fieira de bolhas é o que as separa.
        "panel.model3d.add.thought" => "Thought Balloon",
        "panel.model3d.add.cloud" => "Cloud",
        // ⭐ **Os quatro do fluxograma** (W122). ⚠️ Os nomes são os do ANSI/ISO 5807 — quem
        // desenha um fluxograma conhece-os, e inventar («Slanted Box») obrigaria a procurar.
        "panel.model3d.add.parallelogram" => "Parallelogram",
        "panel.model3d.add.delay" => "Delay",
        "panel.model3d.add.display" => "Display",
        "panel.model3d.add.off_page" => "Off-page Connector",
        "panel.model3d.add.spiral" => "Spiral",
        "panel.model3d.add.document" => "Document",
        "panel.model3d.add.bolt" => "Lightning Bolt",
        "panel.model3d.add.shield" => "Shield",
        "panel.model3d.add.tag" => "Tag",
        "panel.model3d.add.check" => "Check Mark",
        "panel.model3d.add.banner" => "Banner",
        "panel.model3d.add.brace" => "Brace",
        // ⭐ A ESCULTURA. ⚠️ As reticências são a convenção de "isto abre um diálogo" — as outras
        // criam na hora, esta pergunta qual arquivo, e o rótulo tem de dizer a diferença antes do
        // clique.
        "panel.model3d.add.sculpt" => "Sculpt…",
        // ⚠️ **Sem reticências**, ao contrário da irmã acima: aquela abre um diálogo, esta não
        // pergunta nada — traz a escultura que já está na cena. A convenção do "…" é o que diz a
        // diferença antes do clique, e é a mesma lição que o rótulo do `Sculpt…` registou.
        "panel.model3d.add.sculpt_scene" => "Sculpt from scene",
        // As booleanas. ⚠️ "Subtract" e não "Difference": a palavra do documento descreve a
        // operação, e a do botão descreve o que o artista quer FAZER.
        "panel.model3d.op.union" => "Union",
        "panel.model3d.op.subtract" => "Subtract",
        "panel.model3d.op.intersect" => "Intersect",
        // ⭐⭐⭐ **O VERBO DA FORMA** — a fileira que diz o que ESTA forma faz ao resultado das
        // anteriores. ⚠️ **Palavras diferentes das da operação acima, e de propósito:** as duas
        // fileiras aparecem juntas com sujeitos diferentes (o grupo · a forma), e repetir "Union"
        // faria as duas lerem-se como a mesma pergunta feita duas vezes. As escolhidas são as do
        // *Shape Mode* do Illustrator, que é o padrão-ouro deste desenho.
        "panel.model3d.verb_of" => "This shape",
        // ⚠️ **`Inherit` é o primeiro**, e é o que torna a escolha reversível: sem ele, pedir um
        // verbo uma vez tirava a forma do padrão do grupo para sempre.
        "panel.model3d.verb.inherit" => "Inherit",
        "panel.model3d.verb.add" => "Add",
        "panel.model3d.verb.cut" => "Cut",
        "panel.model3d.verb.common" => "Common",
        // ⭐⭐ **O MODO DO LAÇO** (W112) — o que o rectângulo de selecção faz ao que apanha.
        //
        // ⚠️ **Dois e não três:** «substituir» não é alcançável por laço neste módulo (um arrasto
        // sem modificador é o Orbit), então um chip para ele seria pintado e morto.
        "panel.model3d.select.title" => "Lasso",
        "panel.model3d.select.add" => "Add",
        "panel.model3d.select.subtract" => "Subtract",
        // ⭐⭐⭐ **O CARÁTER da mistura** (W99) — a FORMA da transição, ao lado do número que diz o
        // tamanho. ⚠️ **Não há um "Sharp" aqui:** a aresta viva é o **raio zero**, e o slider já o
        // exprime — um quarto chip seria uma segunda porta para o mesmo facto, e as duas podiam
        // discordar.
        //
        // ⚠️ **"Fillet" repete o rótulo da linha de número, e é de propósito:** ali ele diz *quanto*,
        // aqui diz *qual forma*. É a mesma palavra para a mesma coisa — o contrário é que confundia.
        "panel.model3d.character.fillet" => "Fillet",
        "panel.model3d.character.chamfer" => "Chamfer",
        // ⚠️ **"Organic" e não "Smooth":** este app já usa "Smooth" para alisar malha no módulo de
        // escultura, e duas coisas diferentes com o mesmo nome no mesmo app é o que faz o artista
        // procurar no sítio errado.
        "panel.model3d.character.organic" => "Organic",
        // ⭐ Os MODIFICADORES. ⚠️ São interruptores: aceso quer dizer que o objeto já tem um.
        // "Hollow" e não "Shell" — o rótulo diz o que se OBTÉM ("oco"), e "Shell" num app 3D é
        // ambíguo com a casca de superfície. "Grow/Shrink" diria os dois sentidos, mas o número faz
        // isso melhor: negativo encolhe, e o rótulo fica com o nome da operação.
        "panel.model3d.mod.shell" => "Hollow",
        "panel.model3d.mod.offset" => "Offset",
        // ⚠️ "Mirror" e "Array" espelham e repetem no **X local** do objeto — quem quer outro eixo
        // roda o objeto, que é a mesma lei do cilindro e do torno. Um seletor de eixo por
        // modificador seria um terceiro vocabulário de orientação no mesmo painel.
        "panel.model3d.mod.mirror" => "Mirror X",
        "panel.model3d.mod.mirror_y" => "Mirror Y",
        "panel.model3d.mod.mirror_z" => "Mirror Z",
        "panel.model3d.mod.array" => "Array",
        // ⚠️ "Radial" gira em torno do **Z** do objeto — o eixo em que um cilindro aponta, que é o
        // eixo de um flange. Cada modificador nomeia o seu, como as primitivas já fazem.
        "panel.model3d.mod.radial" => "Radial",
        // ⚠️ "Taper" e não "Draft": a palavra de moldagem nomeia o PORQUÊ (tirar a peça do molde) e
        // a de modelagem nomeia o QUE ACONTECE (a secção afina). Quem usa isto aqui está a dar
        // forma, não a projetar um molde.
        "panel.model3d.mod.taper" => "Taper",
        // ⚠️ "Twist" e não "Torsion": a palavra do artista é a do gesto (Blender, 3ds Max, Houdini e
        // ZBrush chamam-lhe todos Twist), e a de engenharia nomeia a tensão, não a forma.
        "panel.model3d.mod.twist" => "Twist",
        // ⚠️ "Bend" e não "Curve": *curve* nesta casa já é a curva do editor vetorial e a rampa do
        // falloff do Painter. As quatro referências (Blender, 3ds Max, Houdini, ZBrush) dizem Bend.
        "panel.model3d.mod.bend" => "Bend",
        // ⭐ A porta de SAÍDA, por resolução. ⚠️ Os rótulos dizem o NÍVEL e não o número de
        // triângulos: o número depende da peça, e prometê-lo no botão seria uma promessa que só o
        // resultado pode fazer — é o toast que o diz, depois de sair.
        "panel.model3d.export.draft" => "Export Draft",
        "panel.model3d.export.fine" => "Export Fine",
        "panel.model3d.export.max" => "Export Max",
        // ⭐ Os nomes das linhas de número dos modificadores. ⚠️ Um modificador pode ter VÁRIOS
        // (a matriz tem dois). ⛔ **Nenhum tem zero desde 2026-09-04**: o espelho era o único, e
        // era exactamente por isso que ele acendia sem mudar um pixel.
        "field.mod.thickness" => "Thickness",
        "field.mod.distance" => "Distance",
        "field.mod.count" => "Copies",
        "field.mod.spacing" => "Spacing",
        "field.mod.slope" => "Slope",
        // ⭐ A torção conta-se em VOLTAS por unidade, e não em graus: é a moeda da forma (uma volta
        // é uma volta em qualquer escala), e o grau obrigaria o slider a andar de 0 a 720.
        "field.mod.turns" => "Turns",
        // ⭐ Os cabeçalhos das secções de números. ⚠️ Um modificador nomeia-se pela PRÓPRIA chave
        // (`panel.model3d.mod.*`), então esta lista só precisa dos dois genéricos.
        "panel.model3d.section.shape" => "Shape",
        "panel.model3d.section.modifier" => "Modifier",
        // ⚠️ "From"/"To" e não "Lower"/"Upper": a banda é uma FAIXA ao longo do eixo, e o artista
        // lê-a como um intervalo. (O Blender diz "Limits", o 3ds Max "Upper/Lower Limit" — os dois
        // nomeiam a cerca; aqui nomeia-se o intervalo, que é o que a linha mostra.)
        "field.mod.from" => "From",
        "field.mod.to" => "To",
        // ⚠️ "Falloff" e não "Smooth": a palavra nomeia o que ela FAZ ao fim da banda (o efeito
        // decai), e é a que o Houdini e o Blender usam para a mesma coisa. "Smooth" já é o nome de
        // um verbo de escultura nesta casa.
        "field.mod.falloff" => "Falloff",
        // ⭐⭐⭐ **A DIRECÇÃO em que o modificador age** (Enio, 2026-08-31). ⚠️ **"Axis" e não
        // "Direction"**: as quatro referências (Blender *Deform Axis*, 3ds Max *Bend Axis*, Houdini,
        // ZBrush) dizem eixo, e *direction* nesta casa já é o sentido de um gradiente.
        "field.mod.axis" => "Axis",
        // ⭐⭐⭐ **ONDE o plano do espelho está** (report do Enio, 2026-09-04). ⚠️ **"Plane" e não
        // "Distance"**: "Distance" já é o número do afastamento (`field.mod.distance`) e quer dizer
        // outra coisa — *quanto a superfície anda*. Aqui o número é a **posição** de um plano, e é
        // isso que o Blender ("Mirror Object"), o 3ds Max ("Mirror Axis") e o MoI nomeiam.
        "field.mod.mirror_plane" => "Plane",
        // ⚠️ Uma letra só, e é de propósito: a fileira tem três botões numa linha de painel, e o
        // artista lê X/Y/Z de relance. Os nomes por extenso não cabem e não acrescentam nada.
        "field.axis.x" => "X",
        "field.axis.y" => "Y",
        "field.axis.z" => "Z",
        // ⭐⭐ A junta ENTRE as cópias que uma repetição gera (Enio, 2026-08-30). ⚠️ **"Seam" e não
        // "Joint"**: o painel já diz "Joint" para o raio de junção de uma forma com as IRMÃS dela
        // (`field.dim.joint`), e a mesma palavra em duas fileiras da mesma coluna faria o artista
        // arrastar a errada. Aqui a costura é entre cópias de UMA forma.
        "field.mod.joint_chamfer" => "Seam Chamfer",
        "field.mod.joint_fillet" => "Seam Fillet",
        // Ações sobre o objeto escolhido.
        "panel.model3d.act.duplicate" => "Duplicate",
        "panel.model3d.act.delete" => "Delete",
        // ⚠️ "Isolate" e não "Solo": no idioma da casa o SOLO é do mixer de áudio (uma pista a
        // tocar entre várias), e a palavra do 3D — a que o Blender e o módulo irmão usam — é esta.
        "panel.model3d.act.isolate" => "Isolate",
        // ⭐⭐ **O vínculo ao desenho** (W57). ⚠️ "Unlink"/"Link Drawing" e não "Detach"/"Attach":
        // *detach* neste módulo já é o gesto de **tirar um nó da peça** (`can_detach`), e dois
        // sentidos para a mesma palavra no mesmo painel é onde o artista aprende errado.
        "panel.model3d.act.unlink" => "Unlink",
        "panel.model3d.act.link" => "Link Drawing",
        // ⭐⭐⭐ **RELIGAR a escultura cujo arquivo sumiu** (W76). ⚠️ As reticências prometem o
        // DIÁLOGO, como no `+ Sculpt…` — este verbo vai pedir o arquivo novo, não conserta sozinho.
        // ⚠️ E ele nasceu SEM esta linha: o `tr` de uma chave desconhecida devolve a própria chave
        // (o `leak_key` do irmão), então o botão dizia `panel.model3d.act.relink` na tela e todo
        // gate de alcance ficava verde — eles perguntam se o verbo é OFERECIDO, nunca o que ele
        // DIZ. Quem passou a perguntar é o `every_act_the_row_can_emit_says_something_other_than_
        // its_own_key`, que varre os `const ACT_` do fonte em vez de reescrever a lista.
        "panel.model3d.act.relink" => "Relink Sculpture…",
        // ⭐ Os NOMES das dimensões. ⚠️ Eles vivem aqui e não numa tabela do documento: a
        // `ph2d-field` devolve **chaves**, e quem traduz é a UI (HR-15).
        "field.dim.width" => "Width",
        "field.dim.height" => "Height",
        "field.dim.depth" => "Depth",
        "field.dim.radius" => "Radius",
        "field.dim.thickness" => "Thickness",
        // ⭐ AS DIMENSÕES DA W101. ⚠️ "Bottom"/"Top" e não "R1"/"R2": o artista vê a peça e sabe
        // qual é o fundo; um índice obriga-o a experimentar para descobrir.
        "field.dim.radius_bottom" => "Bottom Radius",
        "field.dim.radius_top" => "Top Radius",
        // ⚠️ **"Length" e não "Height"** para a cápsula: é o comprimento do SEGMENTO, e a peça mede
        // mais do que isso (mais um raio em cada ponta). Chamar-lhe altura prometeria o tamanho
        // total, que o número não é.
        "field.dim.length" => "Length",
        "field.dim.sides" => "Sides",
        "field.dim.points" => "Points",
        // ⚠️ **"Inner Radius" e não "Ratio"**: o documento guarda um raio, e um rótulo de razão
        // prometeria um número entre 0 e 1 que o campo não é.
        "field.dim.radius_inner" => "Inner Radius",
        "field.dim.radius_outer" => "Outer Radius",
        "field.dim.teeth" => "Teeth",
        "field.dim.tooth" => "Tooth Width",
        "field.dim.cut" => "Cut",
        "field.dim.size" => "Size",
        "field.dim.bite" => "Bite",
        "field.dim.offset" => "Offset",
        "field.dim.arm" => "Arm",
        "field.dim.half_width" => "Half Width",
        // ⭐ **"Span" é a envergadura transversal de uma chapa** — a segunda diagonal de um losango,
        // a abertura de um chevron. ⚠️ Não é "Height": nesta família **Height é a espessura em Z**,
        // e a mesma palavra em duas linhas do mesmo painel faria o artista arrastar a errada.
        "field.dim.span" => "Span",
        "field.dim.shaft" => "Shaft Width",
        "field.dim.head_width" => "Head Width",
        "field.dim.head_length" => "Head Length",
        "field.dim.heads" => "Heads",
        "field.dim.tail" => "Tail",
        "field.dim.lobes" => "Lobes",
        "field.dim.point" => "Point",
        // ⚠️ **"Skew" e não "Slant"**: é a palavra que o CAD e o SVG usam para a mesma grandeza, e
        // ela é um COMPRIMENTO (o escorregamento da base de cima), não um ângulo.
        "field.dim.skew" => "Skew",
        // ⭐ As da W123. ⚠️ **"Pitch" é a palavra do parafuso e da mola** — o afastamento por
        // volta —, e "Turns" é uma contagem que aceita meias voltas.
        "field.dim.pitch" => "Pitch",
        "field.dim.turns" => "Turns",
        "field.dim.wave" => "Wave",
        "field.dim.hole" => "Hole",
        "field.dim.notch" => "Notch",
        // ⚠️ Em GRAUS na cabeça do artista, mas o documento guarda radianos — o painel mostra o
        // número cru, e o rótulo não promete unidade nenhuma.
        "field.dim.angle" => "Sweep",
        // ⚠️ "Fillet" e não "Round": é a palavra que um modelador usa, e é a promessa do módulo
        // dita pelo nome dela.
        // ⚠️ **"Fillet" é o arredondamento da forma DELA PRÓPRIA** — as 12 arestas de uma caixa, o
        // aro de um cilindro. Ele existe numa peça de uma forma só.
        // ⭐⭐⭐ **O CHANFRO, e ele vem ANTES do filete na fileira** (Enio, 2026-08-30: *«poderíamos
        // ter os 2, com chamfer antes de fillet para a possibilidade de arredondar as bordas geradas
        // por chamfer»*). A ordem na lista **é** a ordem em que as duas operações acontecem na forma:
        // o corte reto primeiro, o arco por cima do que ele deixou.
        //
        // ⚠️ **A palavra é a mesma do chip de carácter** (`panel.model3d.character.chamfer`), e é de
        // propósito: os dois medem a mesma coisa — o recuo ao longo de cada face —, e um artista que
        // aprendeu a palavra num sítio não pode encontrar outra no seguinte.
        "field.dim.chamfer" => "Chamfer",
        "field.dim.round" => "Fillet",
        // ⭐⭐⭐ **O RAIO DA JUNÇÃO** (W98) — como esta forma se encontra com o resultado das
        // anteriores. ⚠️ **Palavra própria, e não "Fillet" outra vez:** desde o verbo por forma, uma
        // caixa arredondada que corta com aresta viva mostra os **dois** números ao mesmo tempo, e
        // dois rótulos iguais na mesma coluna são dois controles que o artista não sabe separar.
        //
        // ⚠️ E o **grupo** usa esta mesma chave, de propósito: o raio dele é o raio de junção
        // **padrão**, o que as formas caladas usam. *Uma grandeza, uma palavra.*
        "field.dim.joint" => "Joint",
        // ⭐⭐ **A RESOLUÇÃO do contorno vivo** (W55). ⚠️ "Resolution" e não "Quality": o número diz
        // com que finura o **desenho** é convertido na peça, e "Quality" prometeria uma opinião
        // sobre o resultado. Quem modela num CAD conhece a palavra com este sentido exacto.
        //
        // ⚠️ **Sem unidade no rótulo**, ao contrário das dimensões: ela não mede nada da peça — é
        // uma contagem de níveis, e o que ela compra (arestas, custo) é um facto que o rodapé já
        // diz.
        "field.dim.resolution" => "Resolution",
        // ⭐ A POSE. ⚠️ "Position" é LOCAL, como o Inspector da casa mostra o `Transform` — um painel
        // que mostrasse mundo contradiria o número ao lado no dia em que alguém agrupasse.
        "field.dim.pos_x" => "Position X",
        "field.dim.pos_y" => "Position Y",
        "field.dim.pos_z" => "Position Z",
        // ⭐ A ROTAÇÃO, em GRAUS. ⚠️ "Rotation X/Y/Z" é o que o Blender chama aos mesmos três
        // números, na mesma ordem — e a ordem é parte do significado: quem lê "Rotation Y" espera o
        // segundo giro de um XYZ Euler, não um eixo qualquer.
        "field.dim.rot_x" => "Rotation X",
        "field.dim.rot_y" => "Rotation Y",
        "field.dim.rot_z" => "Rotation Z",
        // ⚠️ A escala só aparece numa OPERAÇÃO: numa forma, o tamanho são as dimensões dela.
        "field.dim.scale" => "Scale",
        "panel.model3d.kind.union" => "Union",
        "panel.model3d.kind.intersection" => "Intersect",
        "panel.model3d.kind.difference" => "Subtract",
        "panel.model3d.kind.box" => "Box",
        "panel.model3d.kind.cylinder" => "Cylinder",
        "panel.model3d.kind.extrude" => "Extrude",
        // ⭐⭐ AS FORMAS DE PERFIL (W53) — o desenho do editor vetorial vira peça. É o fluxo do
        // MoI, e o motor delas está construído e medido desde a W3; faltava o botão.
        "panel.model3d.add.extrude" => "Extrude",
        "panel.model3d.add.revolve" => "Revolve",
        // ⭐ As seis VISTAS NOMEADAS (W47). O atalho vai no rótulo: é a única forma de a tecla ser
        // descoberta por quem não sabe que ela existe.
        "panel.model3d.view.front" => "Front (1)",
        "panel.model3d.view.back" => "Back (^1)",
        "panel.model3d.view.right" => "Right (3)",
        "panel.model3d.view.left" => "Left (^3)",
        "panel.model3d.view.top" => "Top (7)",
        "panel.model3d.view.bottom" => "Bottom (^7)",
        // ⭐ O RÓTULO no canto de cada viewport (W90d) — o nome NU, sem o atalho.
        //
        // ⚠️ **Chaves próprias e não as de cima**: o rótulo do botão traz o atalho de propósito (é a
        // única forma de a tecla ser descoberta), e um "(7)" no canto da imagem seria a promessa de
        // um controlo que ali não existe. *A mesma palavra em dois sítios pode ter de dizer coisas
        // diferentes.*
        "viewport.model3d.view.front" => "Front",
        "viewport.model3d.view.back" => "Back",
        "viewport.model3d.view.right" => "Right",
        "viewport.model3d.view.left" => "Left",
        "viewport.model3d.view.top" => "Top",
        "viewport.model3d.view.bottom" => "Bottom",
        // A vista que não é nenhuma das seis — o artista pôs a câmera onde quis.
        "viewport.model3d.view.user" => "User",
        // ⭐ O TÍTULO do menu que o cabeçalho abre (W109) — ele é o nome acessível do popup, não um
        // rótulo pintado: o [`ContextMenu`] usa-o para o `Role::Menu` do AccessKit.
        "viewport.model3d.view.menu" => "View",
        // Os TRÊS gestos de câmera que não são uma vista.
        "panel.model3d.camera.ortho" => "Ortho (5)",
        "panel.model3d.camera.frame" => "Frame (Home)",
        // ⭐ A divisão do canvas. O atalho é o do Blender para a mesma coisa.
        "panel.model3d.camera.quad" => "Quad View (^\u{2325}Q)",
        // ⭐ O estado de VISTA que precisa de se anunciar: só um nó está à vista, e qual.
        // ⚠️ A frase traz o **nome** ao lado — "estás a ver só uma parte" sem dizer qual deixa o
        // artista à procura.
        "panel.model3d.isolated" => "Isolated (Shift+I)",
        // O rodapé: o custo do último quadro, que é o que diz se a peça ainda é interativa.
        "panel.model3d.trace_cost" => "Trace",
        "panel.model3d.nodes" => "Nodes",
        _ => return None,
    })
}
