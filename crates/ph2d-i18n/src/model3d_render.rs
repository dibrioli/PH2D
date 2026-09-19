//! ⭐⭐⭐ **O vocabulário da APRESENTAÇÃO da cena 3D** — o olhar, a exposição e a camada de ESTILO.
//!
//! # Porque é um ficheiro irmão
//!
//! O [`super::model3d`] nomeia o **DOCUMENTO**: as formas, as operações, os modificadores, os
//! números do material. Isto nomeia **como a cena é APRESENTADA** — a gestão de cor
//! (`docs/Render3d/05`) e a direcção de arte por cima dela (`docs/Render3d/03`, a `W8`). São dois
//! assuntos com ritmos diferentes: aquele cresce quando nasce uma forma, este quando nasce uma lei
//! de render.
//!
//! ⛔ O corte foi forçado pelo tecto de `700` linhas — *corte por responsabilidade, nunca uma
//! entrada no `FILE_OVERAGE_OK`* (`CLAUDE.md` §5.0).

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⭐⭐⭐ **A CAMADA DE ESTILO** (`docs/Render3d/03`, a `W8`) — os botões para mentir de
        // propósito por cima de um pipeline honesto.
        //
        // ⚠️ **"Style" e não "Look"**: o *Look* deste app já tem dono (a vista da gestão de cor,
        // `panel.model3d.look.*`), e duas linhas com o mesmo nome a fazer coisas diferentes é a
        // forma mais barata de um artista escrever no controlo errado.
        "panel.model3d.section.style" => "Style",
        // ⚠️ **"Rim Light" é o nome da INDÚSTRIA** — é assim que ele se chama no Blender, no Unreal
        // e em todo tutorial de iluminação. *Um nome inventado obriga o artista a aprender duas
        // vezes.*
        "panel.model3d.style.rim_color" => "Rim Color",
        "panel.model3d.style.rim_strength" => "Rim Strength",
        // ⛔⛔ **ERA "Rim Width", e o rótulo prometia o CONTRÁRIO do que o número faz** (auditoria
        // de 2026-09-19, §10.10): a lei é `(1 − |N·V|)^w`, logo **subir «Width» ESTREITA** o contorno
        // — `w = 1` acende `50 %` da silhueta, `w = 3` acende `79 %`, `w = 64` acende `99 %`.
        //
        // ⚠️ A nota que o justificava dizia *«o artista lê a LARGURA da banda, e o expoente é como a
        // lei a produz»* — o que seria verdade se o valor fosse **remapeado**, e ele não é: o número
        // da fileira **é** o expoente. ⇒ *ou o valor se inverte, ou o rótulo diz o que ele é.*
        //
        // ⭐ **Fica o rótulo**, e a razão é o preço: remapear cria duas representações do mesmo
        // valor (o que a fileira mostra e o que a lei corre), que é a forma de defeito que esta casa
        // já paga noutros sítios. *Falloff* é o vocabulário da indústria para um expoente de Fresnel,
        // e a dica ao lado diz a direcção por extenso.
        "panel.model3d.style.rim_width" => "Rim Falloff",
        // ⚠️ **"Edge"/"Cavity" e não "Convex"/"Concave"** — os dois primeiros são o que um artista
        // de texturas diz (*edge wear*, *cavity map*); os segundos são a geometria por baixo.
        "panel.model3d.style.convex" => "Edge Tint",
        "panel.model3d.style.concave" => "Cavity Tint",
        // ⭐⭐⭐ **DUAS nitidezes e não uma** (auditoria de 2026-09-19): numa peça real os filetes
        // leem `H·R ≈ 11`–`34` e as covas `≈ −3`–`−5`, e **nenhum limiar partilhado serve os dois**.
        // ⚠️ Os nomes seguem os das TINTAS que cada um governa (*Edge* · *Cavity*), e não a palavra
        // «curvatura» — o artista já escolheu a cor numa fileira chamada `Edge Tint`.
        "panel.model3d.style.edge_sharpness" => "Edge Sharpness",
        "panel.model3d.style.cavity_sharpness" => "Cavity Sharpness",
        // ⭐⭐⭐ **A alavanca do report *«bordas muito duras sem ajustes finos»*** — a distância a
        // que a curvatura é lida.
        //
        // ⚠️ **O nome é «Softness» e não «Radius», e a direcção é load-bearing:** subir suaviza. É a
        // lição do `Rim Width`, que tem o nome ao contrário do que o número faz — *um rótulo que
        // promete o oposto do knob é um defeito que nenhum gate de fiação apanha.*
        "panel.model3d.style.softness" => "Curvature Softness",
        // ⚠️ **"Shadows"/"Highlights" é o vocabulário da GRADE DE COR**, o mesmo das rodas de um
        // colorista.
        "panel.model3d.style.shadow_tint" => "Shadow Tint",
        "panel.model3d.style.highlight_tint" => "Highlight Tint",
        // ⚠️ "Zone Pivot" e não "Threshold": não há limiar nenhum — a repartição é uma razão suave,
        // e este número é onde ela vale meio a meio.
        "panel.model3d.style.pivot" => "Zone Pivot",
        "panel.model3d.style.saturation" => "Indirect Saturation",
        // ⭐⭐⭐ **AS FRASES DOS BALÕES** (`<chave>.tip`) — report do dono, 2026-09-19: *«Zone pivot
        // não sei para que serve»*.
        //
        // ⚠️ **Um rótulo nomeia; uma frase EXPLICA**, e nenhuma superfície deste painel a tinha. Cada
        // uma diz **o que o botão faz** e, onde há direcção, **para que lado** — que é exactamente o
        // que o `Rim Falloff` acima custou a esta secção.
        "panel.model3d.style.rim_color.tip" => {
            "The colour of the light that grazes the silhouette. Needs Rim Strength above zero."
        }
        "panel.model3d.style.rim_strength.tip" => {
            "How much light is added along the silhouette. It lifts the piece off the background."
        }
        "panel.model3d.style.rim_width.tip" => {
            "How fast the rim fades inwards. Higher values TIGHTEN it to a thinner line."
        }
        "panel.model3d.style.convex.tip" => {
            "The colour that edges and fillets take — the piece tints itself by its own shape, with \
             no painted map."
        }
        "panel.model3d.style.concave.tip" => {
            "The colour that hollows and creases take. It is the other half of Edge Tint."
        }
        "panel.model3d.style.edge_sharpness.tip" => {
            "Which edges count as edges: features tighter than this fraction of the piece take the \
             full tint. Higher tints more of the piece."
        }
        "panel.model3d.style.cavity_sharpness.tip" => {
            "The same, for hollows — a separate knob because hollows are usually far larger than \
             fillets, and one threshold cannot serve both."
        }
        // ⭐ A frase da queixa nº 1, e ela nomeia a alavanca em vez do mecanismo.
        "panel.model3d.style.softness.tip" => {
            "How far away the shape is read to decide what is an edge and what is a hollow. Raise \
             it to soften hard-edged tint bands; too far and small hollows stop being read."
        }
        "panel.model3d.style.shadow_tint.tip" => {
            "The colour the dark parts of the image are pulled towards — the shadow half of a film \
             colour grade."
        }
        "panel.model3d.style.highlight_tint.tip" => {
            "The colour the bright parts are pulled towards. Cool shadows with warm highlights is \
             half of a stylised look."
        }
        // ⭐⭐⭐ A frase da queixa nº 2, à letra.
        "panel.model3d.style.pivot.tip" => {
            "Where the boundary between shadow and highlight sits. It does nothing until Shadow \
             Tint and Highlight Tint are different colours."
        }
        "panel.model3d.style.saturation.tip" => {
            "How much colour the light bounced between parts of the piece carries. One is the \
             honest amount; above that the bounce is exaggerated on purpose."
        }
        "panel.model3d.add.light" => "Light",
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
        // ⭐ **A LARGURA da tigela** (W139) — para a mesma profundidade, uma esfera maior cava um
        // prato raso e larguíssimo e uma pequena cava um poço. ⚠️ "Crater" e não "Bite radius": o
        // nome diz a COISA que o artista vê, não o número que a fórmula usa.
        "field.dim.crater" => "Crater",
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
        "field.dim.cell" => "Cell",
        // ⚠️ **"Bulge" e não "Fillet"**: ele arredonda a PAREDE inteira, não só o aro — chamar-lhe
        // filete prometeria o controlo por aresta que as outras formas têm, que é outra coisa.
        "field.dim.bulge" => "Bulge",
        // ⭐⭐ **AS DO NÓ DE TORO** (W134). ⚠️ **"Radius" e "Thickness" são as MESMAS palavras do
        // toro simples**, e de propósito: um nó é uma corda que anda na superfície de um toro, e
        // reusar o vocabulário é o que torna as duas formas legíveis lado a lado. A corda tem
        // palavra própria porque é a terceira medida, que o toro não tem.
        "field.dim.cord" => "Cord",
        // ⚠️ **A LETRA DA LITERATURA E a palavra simples, no mesmo rótulo** — a mesma decisão dos
        // `n1`/`n2` de Gielis, com a metade que falta lá: `p` e `q` são como todo tutorial os chama,
        // e "Winds"/"Loops" é o que eles fazem para quem nunca leu nenhum.
        "field.dim.knot_p" => "P · Winds",
        "field.dim.knot_q" => "Q · Loops",
        "field.dim.thread_depth" => "Thread Depth",
        "field.dim.thread_flank" => "Flank Angle",
        "field.dim.thread_starts" => "Starts",
        "field.dim.thread_hands" => "Hands",
        // ⚠️ **"Top" e "Side" e não "XY" e "Z"**: o artista escolhe pelo que VÊ, e o eixo de cima
        // desta casa é o `Y` — chamar-lhes pelos eixos obrigaria a decifrar qual é qual.
        "field.dim.exponent_top" => "Top Exponent",
        "field.dim.exponent_side" => "Side Exponent",
        // ⚠️ **Os nomes da LITERATURA, e não uns amigáveis** — é a mesma decisão do alfabeto do
        // L-System: `m`, `n1`, `n2` e `n3` são como toda publicação e todo tutorial de Gielis os
        // chamam, e rebaptizá-los tornaria o conhecimento de fora inaplicável aqui.
        "field.dim.top_symmetry" => "Top Symmetry",
        "field.dim.top_n1" => "Top N1",
        "field.dim.top_n2" => "Top N2",
        "field.dim.top_n3" => "Top N3",
        "field.dim.side_symmetry" => "Side Symmetry",
        "field.dim.side_n1" => "Side N1",
        "field.dim.side_n2" => "Side N2",
        "field.dim.side_n3" => "Side N3",
        // ⚠️ **Um vértice por LINHA, com o eixo no nome** — o painel é uma coluna de números, e
        // «A» sozinho não diz qual dos dois é.
        // ⭐⭐ **AS LINHAS DO POLÍGONO** (W132) — a contagem e os `2N` números dos vértices.
        //
        // ⚠️ **Escritas por extenso, uma a uma, e a tabela tem o tamanho do `MAX_POLYGON_VERTICES`.**
        // Uma chave montada em tempo de execução não tem onde viver (o `tr` devolve `&'static str`),
        // e a rota de chave desconhecida pinta o identificador cru **e vaza uma string por quadro**.
        "field.dim.vertices" => "Vertices",
        "field.dim.v1x" => "V1 · X",
        "field.dim.v1y" => "V1 · Y",
        "field.dim.v2x" => "V2 · X",
        "field.dim.v2y" => "V2 · Y",
        "field.dim.v3x" => "V3 · X",
        "field.dim.v3y" => "V3 · Y",
        "field.dim.v4x" => "V4 · X",
        "field.dim.v4y" => "V4 · Y",
        "field.dim.v5x" => "V5 · X",
        "field.dim.v5y" => "V5 · Y",
        "field.dim.v6x" => "V6 · X",
        "field.dim.v6y" => "V6 · Y",
        "field.dim.v7x" => "V7 · X",
        "field.dim.v7y" => "V7 · Y",
        "field.dim.v8x" => "V8 · X",
        "field.dim.v8y" => "V8 · Y",
        "field.dim.v9x" => "V9 · X",
        "field.dim.v9y" => "V9 · Y",
        "field.dim.v10x" => "V10 · X",
        "field.dim.v10y" => "V10 · Y",
        "field.dim.v11x" => "V11 · X",
        "field.dim.v11y" => "V11 · Y",
        "field.dim.v12x" => "V12 · X",
        "field.dim.v12y" => "V12 · Y",
        "field.dim.v13x" => "V13 · X",
        "field.dim.v13y" => "V13 · Y",
        "field.dim.v14x" => "V14 · X",
        "field.dim.v14y" => "V14 · Y",
        "field.dim.v15x" => "V15 · X",
        "field.dim.v15y" => "V15 · Y",
        "field.dim.v16x" => "V16 · X",
        "field.dim.v16y" => "V16 · Y",
        "field.dim.v17x" => "V17 · X",
        "field.dim.v17y" => "V17 · Y",
        "field.dim.v18x" => "V18 · X",
        "field.dim.v18y" => "V18 · Y",
        "field.dim.v19x" => "V19 · X",
        "field.dim.v19y" => "V19 · Y",
        "field.dim.v20x" => "V20 · X",
        "field.dim.v20y" => "V20 · Y",
        "field.dim.v21x" => "V21 · X",
        "field.dim.v21y" => "V21 · Y",
        "field.dim.v22x" => "V22 · X",
        "field.dim.v22y" => "V22 · Y",
        "field.dim.v23x" => "V23 · X",
        "field.dim.v23y" => "V23 · Y",
        "field.dim.v24x" => "V24 · X",
        "field.dim.v24y" => "V24 · Y",
        "field.dim.v25x" => "V25 · X",
        "field.dim.v25y" => "V25 · Y",
        "field.dim.v26x" => "V26 · X",
        "field.dim.v26y" => "V26 · Y",
        "field.dim.v27x" => "V27 · X",
        "field.dim.v27y" => "V27 · Y",
        "field.dim.ax" => "A · X",
        "field.dim.ay" => "A · Y",
        "field.dim.bx" => "B · X",
        "field.dim.by" => "B · Y",
        "field.dim.cx" => "C · X",
        "field.dim.cy" => "C · Y",
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
        // ⭐ **A segunda família de aresta da estrela** (W143/W144) — as quinas do CONTORNO (a
        // ponta e o vale), com tecto próprio `2,03x` mais longo que o do chanfro das faces.
        "field.dim.corner_chamfer" => "Corner Chamfer",
        "field.dim.round" => "Fillet",
        // ⭐⭐⭐ **O RAIO DA JUNÇÃO** (W98) — como esta forma se encontra com o resultado das
        // anteriores. ⚠️ **Palavra própria, e não "Fillet" outra vez:** desde o verbo por forma, uma
        // caixa arredondada que corta com aresta viva mostra os **dois** números ao mesmo tempo, e
        // dois rótulos iguais na mesma coluna são dois controles que o artista não sabe separar.
        //
        // ⚠️ E o **grupo** usa esta mesma chave, de propósito: o raio dele é o raio de junção
        // **padrão**, o que as formas caladas usam. *Uma grandeza, uma palavra.*
        "field.dim.joint" => "Joint",
        // ⭐⭐⭐ **OS NÚMEROS DO MATERIAL** (`docs/Render3d/05`), na ordem do `Param::Material`.
        //
        // ⚠️ **"Base Color R/G/B" e não "R/G/B"**: a coluna já tem trios (posição, rotação), e três
        // letras soltas debaixo deles leem-se como um quarto eixo. A primeira palavra é o que diz de
        // que grandeza é o trio — a mesma lei que o `field.dim.seam_width` abaixo paga.
        //
        // ⛔⛔ **A DÍVIDA DAS TRÊS LINHAS ESTÁ PAGA, e o painel já não as mostra** (Enio,
        // 2026-09-14: *«em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*). Os
        // três canais dobram-se numa **amostra** que abre o selector de cor da casa — ver
        // `ph2d_panel_model3d::ParamRow::swatch`.
        //
        // ⚠️ **E as três chaves FICAM, o que não é folga:** elas rotulam `Param::Material(0..2)`,
        // que continua a ser a **porta de escrita** da cor — a dobra é da apresentação, não da
        // fonte. *Apagar a etiqueta de um número porque um painel deixou de o pintar é deixar o
        // número sem nome no dia em que outra vista o mostrar.*
        // ⭐⭐⭐ **A NOTA DE UMA ESCRITA QUE ESPALHA** (14/09) — ver
        // `ph2d_panel_model3d::ParamRow::subject`. Um pedido de material alcança a selecção
        // inteira, e o controlo mostra o valor de UMA forma: sem esta nota o artista lê «estou a
        // pintar esta» e pinta cinco.
        //
        // ⚠️ **Três chaves e não uma frase pronta**: a nota tem um NÚMERO no meio, e quem o sabe é
        // o shell — ele compõe `«Material applies to: 3 shapes»` das peças traduzidas, como a
        // fileira do verbo já compõe o nome da forma.
        "panel.model3d.material_applies_to" => "Material applies to",
        "panel.model3d.shapes" => "shapes",
        // ⚠️ **«they differ» e não «mixed»**: o aviso só aparece quando o valor mostrado é FALSO
        // sobre as outras formas, e o artista tem de perceber *o que* está errado no que vê — não
        // um rótulo de estado.
        "panel.model3d.material_mixed" => "they differ; the swatch shows the first",
        "field.dim.base_r" => "Base Color R",
        "field.dim.base_g" => "Base Color G",
        "field.dim.base_b" => "Base Color B",
        // ⭐⭐⭐ **O RÓTULO DA AMOSTRA** — a linha é a cor inteira, e não um canal dela.
        "field.dim.base_color" => "Base Color",
        // ⚠️ **"Roughness" e não "Gloss"**: é a palavra do OpenPBR, e é ela que o artista encontra
        // no Blender, no Substance e no Houdini. *Um sinónimo local obriga a traduzir de cabeça.*
        // ⚠️ **"Roughness" e não "Specular Roughness"**: é a rugosidade que o artista pensa quando
        // pensa em rugosidade, e a do verniz nomeia-se por oposição a esta. *O nome longo fica para
        // quem precisa de se distinguir.*
        "field.dim.roughness" => "Roughness",
        "field.dim.metalness" => "Metalness",
        // ⭐⭐ **AS CINCO ÚLTIMAS ENTRADAS DO OpenPBR** (`docs/Render3d/05` §22).
        //
        // ⚠️ **"Base Weight" e não "Opacity"**: ele não torna a peça transparente — tira a camada
        // difusa e deixa o realce. *Um nome emprestado de outra grandeza é o defeito mais caro desta
        // tabela.*
        "field.dim.base_weight" => "Base Weight",
        // ⚠️ **"Diffuse Roughness" sem o "Base"**: a coluna do rótulo tem ~14 caracteres, e a palavra
        // que distingue é a do meio. É a mesma poda do `Coat IOR`.
        "field.dim.base_diffuse_roughness" => "Diffuse Roughness",
        "field.dim.specular_weight" => "Specular Weight",
        "field.dim.specular_color" => "Specular Color",
        "field.dim.specular_r" => "Specular Color R",
        "field.dim.specular_g" => "Specular Color G",
        "field.dim.specular_b" => "Specular Color B",
        "field.dim.specular_ior" => "IOR",
        // ⭐⭐⭐ **O BRILHO PRÓPRIO** (`docs/Render3d/05` §20). ⚠️ **"Emission" e não "Glow"**: é a
        // palavra do OpenPBR e a que o Blender, o Substance e o Houdini escrevem — *um sinónimo
        // local obriga a traduzir de cabeça*, que é a mesma razão escrita no `roughness` acima.
        "field.dim.emission" => "Emission",
        // ⚠️ **A linha é a cor inteira** (a amostra), e as três dos canais só existem para a porta
        // de escrita — elas nunca são pintadas, pela mesma dobra da cor base.
        "field.dim.emission_color" => "Emission Color",
        // ⭐⭐⭐ A LUZ como objecto 3D (ordem do dono, 14/09) — `docs/Render3d/05`.
        "field.dim.light_intensity" => "Intensity",
        "field.dim.light_color" => "Color",
        // ⚠️ Os dois canais seguidores têm chave porque a tabela os enumera; a linha deles é
        // **dobrada** na amostra e nunca é pintada (ver `scene_panel::param_rows`).
        "field.dim.light_color_g" => "Color G",
        "field.dim.light_color_b" => "Color B",
        "field.dim.emission_r" => "Emission Color R",
        "field.dim.emission_g" => "Emission Color G",
        "field.dim.emission_b" => "Emission Color B",
        "field.dim.subsurface_weight" => "Subsurface",
        "field.dim.subsurface_color" => "Subsurface Color",
        "field.dim.subsurface_scale" => "Subsurface Radius Scale",
        "field.dim.thin_walled_no" => "Solid",
        "field.dim.thin_walled_yes" => "Thin Walled",
        "field.dim.subsurface_r" => "Subsurface Color R",
        "field.dim.subsurface_g" => "Subsurface Color G",
        "field.dim.subsurface_b" => "Subsurface Color B",
        "field.dim.subsurface_radius" => "Subsurface Radius",
        "field.dim.subsurface_scale_r" => "Subsurface Radius Scale R",
        "field.dim.subsurface_scale_g" => "Subsurface Radius Scale G",
        "field.dim.subsurface_scale_b" => "Subsurface Radius Scale B",
        "field.dim.subsurface_anisotropy" => "Subsurface Anisotropy",
        "field.dim.thin_walled" => "Thin Walled",
        // ⭐⭐⭐ **O VERNIZ** (`docs/Render3d/05` §21). ⚠️ **"Coat" e não "Clear Coat"**: é o nome do
        // OpenPBR e o que o Blender escreve — e a segunda palavra passou a ser errada no dia em que
        // o verniz ganhou cor (um verniz âmbar não é *clear*).
        "field.dim.coat" => "Coat",
        "field.dim.coat_roughness" => "Coat Roughness",
        "field.dim.coat_color" => "Coat Color",
        "field.dim.coat_r" => "Coat Color R",
        "field.dim.coat_g" => "Coat Color G",
        "field.dim.coat_b" => "Coat Color B",
        // ⚠️ **"Coat IOR" com a sigla INTEIRA**: *«Index of Refraction»* por extenso não cabe na
        // coluna do rótulo, e a sigla é a que aparece em todos os programas do ofício.
        "field.dim.coat_ior" => "Coat IOR",
        // ⚠️ **"Darkening" e não "Tint"**: ele não tinge — ele escurece a base por baixo do verniz,
        // que é um efeito físico de reflexão interna. *A tinta é a linha de cima.*
        "field.dim.coat_darkening" => "Coat Darkening",
        // ⭐⭐ **O SEGUNDO NÚMERO de uma junta** (W145). ⚠️ **"Seam Width" e não "Width"**: a
        // coluna já tem larguras da FORMA, e duas palavras iguais para grandezas de sujeitos
        // diferentes é o defeito que o `field.dim.joint` acima existe para não repetir.
        "field.dim.seam_width" => "Seam Width",
        // ⚠️ **"Chamfer Bias" nomeia o CARÁCTER a que pertence**, porque a linha aparece debaixo do
        // chip "Chamfer" e só ali: sem a primeira palavra, "Bias" num painel de modelação lê-se como
        // um deslocamento do campo.
        "field.dim.bevel_bias" => "Chamfer Bias",
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
        // ⭐⭐⭐ O SOMBREAMENTO (`docs/Render3d/05`) — o 2.º pulldown da área.
        //
        // ⚠️ **O rótulo diz o GRUPO e a face diz o ESTADO** (a mesma lei do `area.view`): o chip
        // fechado lê `Matcap` ou `Render`, e o que ele abre é o sombreamento inteiro — o modo, a vista
        // e a exposição.
        "panel.model3d.area.shading" => "Shading",
        // A luz do OLHO (a fotografia de sempre) contra a luz da CENA (material, lâmpadas, céu).
        "panel.model3d.shading.matcap" => "Matcap",
        "panel.model3d.shading.render" => "Render",
        // As vistas — os nomes do Blender para as mesmas transformações.
        "panel.model3d.look.standard" => "Standard",
        "panel.model3d.look.neutral" => "Neutral",
        // A exposição em stops: cada um dobra (ou divide ao meio) a luz da cena.
        "panel.model3d.exposure.minus2" => "Exposure \u{2212}2",
        "panel.model3d.exposure.minus1" => "Exposure \u{2212}1",
        "panel.model3d.exposure.zero" => "Exposure 0",
        "panel.model3d.exposure.plus1" => "Exposure +1",
        "panel.model3d.exposure.plus2" => "Exposure +2",
        _ => return None,
    })
}
