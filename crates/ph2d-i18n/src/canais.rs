//! **Os rótulos dos CANAIS de família** — `canal.*` (plano 35: *«o auto key grava absolutamente
//! tudo»*). O vocabulário de cada família (`ph2d_ecs::canal_vetor`, …) diz a chave; aqui mora a
//! palavra que o dope-sheet e o losango mostram. Inglês por cânone, como o resto do app.

/// A tradução de uma chave de canal, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.timeline.prop.field" => "Property",
        "canal.vetor.forma" => "Shape",
        "canal.vetor.regra" => "Fill Rule",
        "canal.vetor.preenchimento" => "Fill",
        "canal.vetor.traco_cor" => "Stroke Color",
        "canal.vetor.traco_largura" => "Stroke Width",
        "canal.vetor.traco_ponta" => "Stroke Cap",
        "canal.vetor.traco_juncao" => "Stroke Join",
        "canal.vetor.traco_alinhamento" => "Stroke Align",
        "canal.vetor.tracejado" => "Dash",
        "canal.vetor.marca_inicio" => "Start Marker",
        "canal.vetor.marca_fim" => "End Marker",
        "canal.vetor.escala_inicio" => "Start Marker Size",
        "canal.vetor.escala_fim" => "End Marker Size",
        "canal.vetor.marca_redonda" => "Marker Roundness",
        "canal.vetor.mistura" => "Blend",
        "canal.vetor.pilha_ligada" => "Layer On",
        "canal.vetor.pilha_opacidade" => "Layer Opacity",
        "canal.vetor.pilha_mistura" => "Layer Blend",
        "canal.vetor.pilha_deslocamento" => "Layer Offset",
        "canal.vetor.pilha_dilatacao" => "Layer Spread",
        "canal.vetor.pilha_dilatacao_juncao" => "Layer Spread Join",
        "canal.vetor.efeito_ligado" => "Effect On",
        "canal.vetor.efeito" => "Effect",
        "canal.vetor.tamanho" => "Size",
        "canal.vetor.forma_param" => "Shape Settings",
        "canal.vetor.texto_tamanho" => "Font Size",
        "canal.vetor.texto_peso" => "Font Weight",
        "canal.vetor.texto_entrelinha" => "Line Height",
        "canal.vetor.texto_tracking" => "Tracking",
        "canal.vetor.texto_alinhamento" => "Text Align",
        "canal.vetor.texto_quebra" => "Wrap Width",
        "canal.vetor.texto_eixo" => "Font Axis",
        _ => return None,
    })
}
