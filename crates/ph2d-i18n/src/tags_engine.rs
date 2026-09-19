//! ⭐⭐ **AS CINCO RECUSAS DA ÁRVORE DE TAGS** (`ph2d-tags`) — e a migração que a própria crate
//! tinha PRESCRITO por escrito.
//!
//! # A cura estava no doc, e ela nomeia porque as frases estavam cruas
//!
//! O cabeçalho do `TagError` dizia: *«em inglês e sem `ph2d_i18n::tr`, como as vizinhas: esta crate
//! é uma FOLHA sem dependências de UI, e pô-la a depender do catálogo pelo texto de cinco frases
//! inverteria a pilha. O dia em que a segunda língua entrar, o que muda é quem CHAMA isto, não a
//! assinatura.»*
//!
//! ⇒ mudou quem chama. A folha publica `message_key()` e quem resolve é o
//! `fase_tag_tree_commits` da shell, onde o catálogo já está ao alcance. **A `ph2d-tags` continua
//! sem uma dependência de UI**, que era a razão inteira de elas estarem cruas.
//!
//! ⚠️ *Uma decisão medida com a cura escrita ao lado dela não é dívida enquanto a condição não
//! muda — e a condição aqui era a segunda língua, que o `PH2D_LANG=teste` já é.*
//!
//! # ⛔ Porque nenhuma régua as via
//!
//! A frase atravessa TRÊS crates — a lei (`ph2d-tags`), o instantâneo (`ph2d-editor-core`) e o
//! pintor (`ph2d-panel-tags`) — e nenhum dos dois censos a segue: o lexical não varre a folha, e o
//! de porta segue o texto até um pintor **da mesma crate**. ⚠️ **E o gate de RUNTIME também não**,
//! por outra razão: a linha da recusa só é pintada depois de um gesto REJEITADO, e ele pinta cada
//! painel no estado de OMISSÃO.

/// A tradução de uma chave `tags.error.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "tags.error.empty" => "A tag needs a name.",
        // ⚠️ As aspas curvas e o travessão são os do texto original — a recusa NOMEIA o carácter
        // que a causou, e trocá-lo por `"` faria a frase apontar para outro carácter.
        "tags.error.has_separator" => {
            "A name cannot contain \u{201c}/\u{201d} \u{2014} drag the tag instead."
        }
        "tags.error.collision" => "A tag with this name already exists here.",
        "tags.error.into_own_subtree" => "Cannot move a tag inside itself.",
        "tags.error.missing" => "That tag no longer exists.",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
