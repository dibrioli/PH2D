//! ⭐⭐ **OS NOMES DAS MISTURAS** (`ph2d-blend-mode`) — o vocabulário que TRÊS painéis pintam e que
//! régua nenhuma deste repo via.
//!
//! # Porque ele escapou a tudo
//!
//! A `ph2d-blend-mode` é um MOTOR: o censo lexical não a varre e o de porta segue o texto só dentro
//! da mesma crate. E o gate de RUNTIME também não a apanhava inteira — dezasseis destas palavras
//! são texto de chegada de OUTRAS chaves da tabela (`paint_brush.brush_blend.*`,
//! `node.opts.*_blend_labels.*`), logo a ponte reconhecia-as e só `Behind` e `Clear` apareciam
//! soltos. ⚠️ *Uma família migrada pela metade lê-se como uma família limpa com dois literais
//! soltos* — e eram vinte e dois.
//!
//! # ⛔⛔ E a SONDA que os declarou órfãos estava INVÁLIDA
//!
//! A prova de que um rótulo não tem pintor é renomeá-lo e ver a workspace compilar. Numa corrida
//! com **sete** renomeações ao mesmo tempo, a `ph2d-vec-scene` não compilou — e com ela ficaram por
//! verificar **todos os painéis que dela dependem**, que são exactamente os três que pintam isto.
//! O relatório lê-se igual a *«sem chamadores»*.
//!
//! ⇒ *uma sonda de órfão por renomeação não afirma nada sobre uma crate a JUSANTE de outra que não
//! compilou* — e quem a corre tem de ler os ERROS todos, não só os que esperava. Quem me apanhou
//! foi o `cargo check --workspace` a seguir.

/// A tradução de uma chave `blend.mode.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "blend.mode.normal" => "Normal",
        "blend.mode.multiply" => "Multiply",
        "blend.mode.darken" => "Darken",
        "blend.mode.color_burn" => "Color Burn",
        "blend.mode.linear_burn" => "Linear Burn",
        "blend.mode.lighten" => "Lighten",
        "blend.mode.screen" => "Screen",
        "blend.mode.color_dodge" => "Color Dodge",
        "blend.mode.add" => "Add",
        "blend.mode.overlay" => "Overlay",
        "blend.mode.soft_light" => "Soft Light",
        "blend.mode.hard_light" => "Hard Light",
        "blend.mode.vivid_light" => "Vivid Light",
        "blend.mode.linear_light" => "Linear Light",
        "blend.mode.difference" => "Difference",
        "blend.mode.exclusion" => "Exclusion",
        "blend.mode.hue" => "Hue",
        "blend.mode.saturation" => "Saturation",
        "blend.mode.color" => "Color",
        "blend.mode.luminosity" => "Luminosity",
        "blend.mode.behind" => "Behind",
        "blend.mode.clear" => "Clear",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
