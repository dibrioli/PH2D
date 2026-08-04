//! **A TABELA DE COR VIRA AUTORÁVEL** — a camada de override sobre a tabela gerada
//! (plano UI/UX W6, degrau 1).
//!
//! # Porque ela mora AQUI, e não num parâmetro
//!
//! [`crate::color::ColorToken::resolve`] já é a **porta única** de *"que cor tem este token, neste
//! modo?"* — os 44 widgets do catálogo passam por ela, e é isso que faz um valor editado re-vestir
//! **o app inteiro** sem uma linha de pintura nova. Passar a tabela por parâmetro seria a
//! alternativa "pura", e ela custa reescrever todos esses chamadores para carregar um argumento
//! que 99,9% deles não olham — o oposto do que a porta única compra.
//!
//! ⚠️ **É estado global mutável lido por uma função de aparência pura, e isto está escrito de
//! propósito.** O padrão é o que o repo já usa entre shell e folha (`spectral_state`, os knobs do
//! Wet Paint, os `state_*` dos painéis): a **shell publica, a folha lê**.
//!
//! # `thread_local`, e a razão não é performance
//!
//! Cada teste do Rust corre na própria thread ⇒ um gate que arma um override **não pode** envenenar
//! o vizinho, que é exatamente o modo de falha de um `static` global numa suíte paralela. E a
//! consequência honesta: **quem PINTA tem de ser a mesma thread que PUBLICOU**. No produto é (o
//! frame inteiro corre na thread de UI); um consumidor fora dela leria a tabela de fábrica, e é por
//! isso que isto está nomeado aqui em vez de descoberto num screenshot.
//!
//! # Vazio ⇒ BYTE-IDÊNTICO, e é o que torna a camada barata
//!
//! Sem override nenhum o `resolve` faz **uma leitura de bool** e devolve o valor gerado, na mesma
//! ordem de operações de sempre. Não há segunda tabela, não há cópia da tabela de fábrica, e o
//! gate `design_token_sync` — que re-parseia o JSON com um parser independente — continua a medir
//! **a tabela gerada**, porque é ela que o override cobre e nunca substitui.

use std::cell::RefCell;

use crate::color::{Color, ColorToken};
use crate::theme::Theme;

/// Um valor autorado: **que token, em que modo, com que cor**.
///
/// ⚠️ A chave é o par `(modo, token)` e não só o token: os quatro modos são quatro respostas à
/// mesma pergunta, e um override que valesse para todos faria trocar de tema deixar de re-vestir —
/// que é a feature que o modo tem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorOverride {
    pub theme: Theme,
    pub token: ColorToken,
    pub colour: Color,
}

thread_local! {
    /// A lista de valores autorados. **Esparsa**: só o que difere da fábrica viaja.
    static OVERRIDES: RefCell<Vec<ColorOverride>> = const { RefCell::new(Vec::new()) };
    /// *Existe algum override?* — a pergunta que o caminho comum faz, e a única que ele paga.
    ///
    /// ⚠️ Ela é redundante com `OVERRIDES.is_empty()` **e não é higiene**: chegar ao `Vec` custa
    /// um `RefCell::borrow`, e o caso comum (nenhum override) é todo frame de todo app que nunca
    /// abriu o painel.
    static ANY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// **A cor autorada deste token neste modo**, se houver — a pergunta que o `resolve` faz.
#[must_use]
pub fn color_override(theme: Theme, token: ColorToken) -> Option<Color> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| {
        o.borrow()
            .iter()
            .find(|e| e.theme == theme && e.token == token)
            .map(|e| e.colour)
    })
}

/// **A porta ÚNICA de escrita** — `None` devolve o token à fábrica.
///
/// ⚠️ Escrever *a cor de fábrica* como override **não é** o mesmo que soltar: o arquivo passaria a
/// carregar um valor que só por acaso coincide, e re-editar a tabela de fábrica deixaria de
/// alcançar aquele token em silêncio. Soltar é `None`, e é o que o botão *Reset* faz.
pub fn set_color_override(theme: Theme, token: ColorToken, colour: Option<Color>) {
    OVERRIDES.with(|o| {
        let mut list = o.borrow_mut();
        let at = list
            .iter()
            .position(|e| e.theme == theme && e.token == token);
        match (at, colour) {
            (Some(i), Some(c)) => list[i].colour = c,
            (Some(i), None) => {
                list.remove(i);
            }
            (None, Some(c)) => list.push(ColorOverride {
                theme,
                token,
                colour: c,
            }),
            (None, None) => {}
        }
        ANY.with(|a| a.set(!list.is_empty()));
    });
}

/// Todos os valores autorados — o que a persistência guarda.
///
/// ⚠️ **Ordenado por `(modo, chave do token)`**, e isso não é arrumação: o arquivo é comparado
/// byte a byte por gates e por quem investiga um diff, e uma lista cuja ordem depende da ordem dos
/// cliques faria dois documentos logicamente iguais parecerem diferentes. É a mesma lei que o
/// `VecInstance::set` aplica aos overrides de instância.
#[must_use]
pub fn color_overrides() -> Vec<ColorOverride> {
    let mut out = OVERRIDES.with(|o| o.borrow().clone());
    out.sort_by(|a, b| (a.theme as u8, a.token.key()).cmp(&(b.theme as u8, b.token.key())));
    out
}

/// Instala a lista inteira (o load de projeto). Substitui o que houver.
pub fn set_color_overrides(list: Vec<ColorOverride>) {
    OVERRIDES.with(|o| {
        let mut cur = o.borrow_mut();
        *cur = list;
        ANY.with(|a| a.set(!cur.is_empty()));
    });
}

/// Devolve **toda** a tabela à fábrica.
pub fn clear_color_overrides() {
    set_color_overrides(Vec::new());
}

/// Quantos tokens deste modo estão autorados — o readout que o painel mostra.
#[must_use]
pub fn overridden_count(theme: Theme) -> usize {
    if !ANY.with(std::cell::Cell::get) {
        return 0;
    }
    OVERRIDES.with(|o| o.borrow().iter().filter(|e| e.theme == theme).count())
}

#[cfg(test)]
#[path = "overrides_tests.rs"]
mod tests;
