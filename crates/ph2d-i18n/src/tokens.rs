//! ⭐⭐ **AS PALAVRAS DO DESIGN SYSTEM** — a 7.ª fatia da fronteira dos motores.
//!
//! ⛔⛔ **O painel dos tokens e a galeria de widgets estão a ZERO e a palavra crua está noutro
//! sítio:** a [`ph2d-tokens`] é *design-data puro, zero runtime deps* (está escrito no `Cargo.toml`
//! dela) e **não é varrida por censo nenhum** — a régua do repo cobre os painéis, as `ph2d-app-*`,
//! a `ph2d-editor-core`, a `ph2d-param-editors` e a shell. ⇒ os **nomes dos oito temas** da barra
//! do topo, os **quatro desenhos de slider** com as quatro linhas de ajuda deles e as duas recusas
//! de fórmula viviam ali, invisíveis.
//!
//! Quem os achou foi o **idioma de teste**, na 2.ª fotografia do dono (*«quase todas as palavras do
//! design Tokens»* · *«Widget lab inteiro»*).
//!
//! # ⚠️ Quatro destes nomes são PRÓPRIOS, e entram na mesma
//!
//! *Forge*, *Workshop*, *Sunstone* e *Blueprint* são nomes de tema; *Dark*, *Gray*, *Light* e
//! *Black (OLED)* são descrições. Entram todos: **um nome próprio na TABELA é uma decisão que um
//! tradutor pode tomar; um nome próprio no CÓDIGO é uma decisão que ninguém pode tomar.**
//!
//! ⚠️ **A chave do nome NÃO é o [`id`](ph2d_tokens::Theme::id) do tema** — aquele vai para o
//! ficheiro de preferências e não pode mudar de idioma nenhum.
//!
//! # ⭐ As duas RECUSAS, e porque elas são chaves e não frases
//!
//! A `ph2d-tokens` não pode resolver uma frase (zero deps) e a recusa atravessa a fronteira como
//! `String`. Ela tem **duas espécies**: as daqui, que são chaves com o prefixo
//! `ph2d_tokens::num_expr::CHAVE_DE_RECUSA`, e a do motor de fórmulas, que é **dinâmica** (nomeia
//! o identificador que não foi entendido). A shell separa-as pelo prefixo — *um contrato, com
//! gate, e não uma heurística sobre texto de motor*.

/// A tradução de uma chave `tokens.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "design.refusal.bad_formula" => "That formula could not be evaluated",
        "design.refusal.no_formula_host" => "Formulas are not available in this build",
        "design.slider.bar" => "Bar",
        "design.slider.bar.blurb" => {
            "the fill is the whole background \u{b7} reads at a glance \u{b7} competes with the number"
        }
        "design.slider.ghost" => "Ghost",
        "design.slider.ghost.blurb" => {
            "flattest of all \u{b7} vanishes in a long list \u{b7} barely reads as draggable"
        }
        "design.slider.inset" => "Inset",
        "design.slider.inset.blurb" => {
            "a capsule in a groove \u{b7} clearly a control \u{b7} spends height on framing"
        }
        "design.slider.underline" => "Underline",
        "design.slider.underline.blurb" => {
            "2 px fill at the bottom \u{b7} cleanest text \u{b7} quietest at a glance"
        }
        "design.theme.blueprint" => "Blueprint",
        "design.theme.dark" => "Dark",
        "design.theme.forge" => "Forge",
        "design.theme.gray" => "Gray",
        "design.theme.light" => "Light",
        "design.theme.oled" => "Black (OLED)",
        "design.theme.sunstone" => "Sunstone",
        "design.theme.workshop" => "Workshop",
        _ => return None,
    })
}
