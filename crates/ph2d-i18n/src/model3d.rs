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
        // As formas que se acrescentam. ⚠️ São AÇÕES, não modos: o rótulo diz a forma, e o gesto
        // cria uma. Um "Add Box" seria a palavra "Add" repetida quatro vezes num painel estreito.
        "panel.model3d.add.box" => "+ Box",
        "panel.model3d.add.sphere" => "+ Sphere",
        "panel.model3d.add.cylinder" => "+ Cylinder",
        "panel.model3d.add.torus" => "+ Torus",
        // ⭐ A ESCULTURA. ⚠️ As reticências são a convenção de "isto abre um diálogo" — as outras
        // quatro criam na hora, esta pergunta qual arquivo, e o rótulo tem de dizer a diferença
        // antes do clique.
        "panel.model3d.add.sculpt" => "+ Sculpt…",
        // ⚠️ **Sem reticências**, ao contrário da irmã acima: aquela abre um diálogo, esta não
        // pergunta nada — traz a escultura que já está na cena. A convenção do "…" é o que diz a
        // diferença antes do clique, e é a mesma lição que o rótulo do `+ Sculpt…` registou.
        "panel.model3d.add.sculpt_scene" => "+ Sculpt from scene",
        // As booleanas. ⚠️ "Subtract" e não "Difference": a palavra do documento descreve a
        // operação, e a do botão descreve o que o artista quer FAZER.
        "panel.model3d.op.union" => "Union",
        "panel.model3d.op.subtract" => "Subtract",
        "panel.model3d.op.intersect" => "Intersect",
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
        // ⭐ A porta de SAÍDA, por resolução. ⚠️ Os rótulos dizem o NÍVEL e não o número de
        // triângulos: o número depende da peça, e prometê-lo no botão seria uma promessa que só o
        // resultado pode fazer — é o toast que o diz, depois de sair.
        "panel.model3d.export.draft" => "Export Draft",
        "panel.model3d.export.fine" => "Export Fine",
        "panel.model3d.export.max" => "Export Max",
        // ⭐ Os nomes das linhas de número dos modificadores. ⚠️ Um modificador pode ter VÁRIOS
        // (a matriz tem dois) e pode não ter nenhum (o espelho).
        "field.mod.thickness" => "Thickness",
        "field.mod.distance" => "Distance",
        "field.mod.count" => "Copies",
        "field.mod.spacing" => "Spacing",
        "field.mod.slope" => "Slope",
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
        // ⚠️ "Fillet" e não "Round": é a palavra que um modelador usa, e é a promessa do módulo
        // dita pelo nome dela.
        "field.dim.round" => "Fillet",
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
        "panel.model3d.add.extrude" => "+ Extrude",
        "panel.model3d.add.revolve" => "+ Revolve",
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
