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
        // ⛔ Era "3D Model" e a aba dizia "Model 3D" — uma fonte só desde que o `Panel::TITLE` é a
        // chave (2026-09-17); ganhou a palavra da aba, que é a que o menu *Window* também diz.
        "panel.model3d.title" => "Model 3D",
        "panel.model3d.empty" => "Select an object to edit its dimensions.",
        // ⭐⭐ **O RODAPÉ é uma FRASE, e a ordem das palavras é dela** (2026-09-17): o pintor
        // montava `"{}: {} \u{b7} {} {:.1} ms"` à mão com os dois rótulos traduzidos lá dentro —
        // as palavras vinham da tabela e a SINTAXE não, e há línguas em que a contagem vem à
        // frente do nome. ⚠️ Os dois rótulos continuam a ser chaves próprias e entram aqui como
        // ARGUMENTOS: a frase não os duplica.
        "panel.model3d.footer" => "{nodes}: {count} \u{b7} {cost} {ms} {unit}",
        // ⚠️ O rótulo diz **Radius**, e é um compromisso que o documento honra: quem escolher a
        // mistura orgânica vê um número que entrega 3/4 do que promete (ver `Blend::Organic`), e
        // isso é uma decisão de produto por tomar — não uma etiqueta a corrigir aqui.
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
        // ⚠️ **"Coil" e não "Helix"**: é a palavra que um artista usa para a peça (mola, bobina),
        // e "helix" é a curva matemática.
        "panel.model3d.add.helix" => "Coil",
        "panel.model3d.add.gyroid" => "Gyroid Lattice",
        "panel.model3d.add.rounded_cylinder" => "Rounded Cylinder",
        "panel.model3d.add.superquadric" => "Superquadric",
        "panel.model3d.add.superformula" => "Superformula",
        "panel.model3d.add.triangle" => "Triangle",
        "panel.model3d.add.polygon" => "Polygon",
        // ⚠️ **"Torus Knot" e não "Knot"**: é o nome que toda a literatura e todo o outro programa
        // 3D lhe dá, e um artista que já o viu procura por ele.
        "panel.model3d.add.torus_knot" => "Torus Knot",
        "panel.model3d.add.thread" => "Thread",
        "panel.model3d.add.knurl" => "Knurled Grip",
        "panel.model3d.add.bezier" => "Bezier Curve",
        "panel.model3d.add.parabola" => "Parabola",
        "panel.model3d.add.circle_wave" => "Circle Wave",
        // ⭐⭐ **AS DUAS ÚLTIMAS FORMAS DO CATÁLOGO** (W139).
        //
        // ⚠️ **"Cratered Sphere" e não o nome que a literatura de SDF lhe dá.** A fórmula publicada
        // chama-se `sdDeathStar`, e esse nome é uma marca registada de outra gente — um rótulo de
        // produto não é uma citação bibliográfica. O que fica diz o que a peça É, que é também o
        // que o §0.8 pede: quem nunca a viu sabe o que vai receber.
        "panel.model3d.add.cratered_sphere" => "Cratered Sphere",
        // ⚠️ **"Lens" e não "Vesica Segment"**: a `add.vesica` já existe e é uma CHAPA (a lente 2D
        // puxada em Z), e esta é o sólido de revolução. Dois rótulos que começassem pela mesma
        // palavra fariam a busca da paleta devolver os dois para a mesma intenção — e é justamente
        // a forma que o artista NÃO quer que ele escolheria primeiro.
        "panel.model3d.add.lens" => "Lens",
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
        // ⭐⭐⭐ **AS QUATRO DA W145** (pedido do Enio, 2026-09-09).
        //
        // ⚠️ **"Soft" e não "Smooth"**, pela MESMA razão que fez o `Organic` não ser "Smooth": a
        // palavra já é um verbo de malha no módulo de escultura. E não "Blend", que num app 3D é o
        // nome genérico de toda esta fileira.
        "panel.model3d.character.soft" => "Soft",
        // ⚠️ **"Bead" é a palavra de oficina** — cordão de solda, de cola, de vedante. "Weld"
        // prometeria uma operação de topologia (soldar duas peças numa), que é outra coisa e que
        // este app tem noutro módulo.
        "panel.model3d.character.bead" => "Bead",
        // ⚠️ **"Groove" e não "Panel Line"**: a feição é um canal, e é ela que o rótulo nomeia — o
        // uso mais comum dela é uma linha de painel, mas nomear o uso deixaria o artista sem
        // palavra para os outros.
        "panel.model3d.character.groove" => "Groove",
        "panel.model3d.character.ridge" => "Ridge",
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
        // ⭐⭐⭐ **O MATERIAL** (`docs/Render3d/05`) — o que a forma mede à LUZ.
        "panel.model3d.section.material" => "Material",
        "panel.model3d.section.light" => "Light",
        // ⭐ **E a APRESENTAÇÃO da cena vive no irmão** — o olhar, a exposição e o estilo
        // (`super::model3d_render`). ⚠️ *Um `_ => None` que não delegasse deixaria aquelas chaves a
        // devolver a própria chave ao artista*, que é como um rótulo em falta se lê na tela.
        _ => return super::model3d_render::tr(key),
    })
}
