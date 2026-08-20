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
        "panel.model3d.empty" => "No model in the scene yet.",
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
