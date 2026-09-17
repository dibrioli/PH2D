//! **O QUE A FAMÍLIA app.vec DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-vec` mostra (do VETOR (os selos da booleana, importar SVG)), na forma `app.vec.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): as
//! CENAS (de smoke e de demonstração), o diagnóstico de consola, os formatos de ficheiro, os nomes
//! de COLUNA (que um nó a jusante lê pelo nome) e os nomes por omissão de objecto — identidade
//! durável, não vocabulário.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.vec.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.vec.bool_shape.bse" => "BSE",
        "app.vec.bool_shape.exc" => "EXC",
        "app.vec.bool_shape.int" => "INT",
        "app.vec.bool_shape.sub" => "SUB",
        "app.vec.bool_shape.uni" => "UNI",
        "app.vec.bool_shape.rcp" => "RCP",
        "app.vec.svg_import.no_drawable_shape" => "no drawable shape",
        "app.vec.svg_import.read" => "read: {e}",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
