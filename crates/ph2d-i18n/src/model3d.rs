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
        "panel.model3d.mod.mirror" => "Mirror",
        "panel.model3d.mod.array" => "Array",
        // ⭐ Os nomes das linhas de número dos modificadores. ⚠️ Um modificador pode ter VÁRIOS
        // (a matriz tem dois) e pode não ter nenhum (o espelho).
        "field.mod.thickness" => "Thickness",
        "field.mod.distance" => "Distance",
        "field.mod.count" => "Copies",
        "field.mod.spacing" => "Spacing",
        // Ações sobre o objeto escolhido.
        "panel.model3d.act.duplicate" => "Duplicate",
        "panel.model3d.act.delete" => "Delete",
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
        // O rodapé: o custo do último quadro, que é o que diz se a peça ainda é interativa.
        "panel.model3d.trace_cost" => "Trace",
        "panel.model3d.nodes" => "Nodes",
        _ => return None,
    })
}
