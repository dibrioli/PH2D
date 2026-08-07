//! **A CHAVE decide de que família é a entrada** — a porta única do roteamento (plano UI/UX
//! W4c.5).
//!
//! # Por que ela existe, e por que aqui
//!
//! Duas coisas de fora chegam com um par `(chave, valor)` e precisam de saber que token é aquele:
//! o **arquivo de projeto** (`project_tokens::install`, postcard) e o **import DTCG**
//! (`ph2d-tokens-dtcg`). A lei é a mesma para os dois — *a chave diz a família, e o valor tem de
//! caber nela* —, e o `project_tokens.rs` já a tinha escrito, com o aviso de que o DTCG teria de a
//! juntar de novo.
//!
//! Uma segunda cópia não daria a mesma resposta por muito tempo: a próxima família de tokens
//! entraria numa e faltaria na outra, e o modo de falha seria um valor que entra por um caminho e
//! **cai em silêncio** pelo outro.
//!
//! ⚠️ **Ela mora na `ph2d-tokens` porque é aqui que vivem as duas inversas** ([`ColorToken::from_key`]
//! e [`NumToken::from_key`]) — e porque a pergunta não tem dependência nenhuma: é um `match` sobre
//! duas listas geradas. Pô-la na shell obrigaria a crate do DTCG a depender da shell, que é a
//! aresta ao contrário.
//!
//! # O que ela NÃO decide
//!
//! *Se o valor deve mesmo ser autorado.* Um arquivo de projeto só carrega o que o artista tocou,
//! mas um export DTCG traz a **tabela inteira** — e reimportá-lo autoraria os ~80 tokens de uma
//! vez. Essa segunda pergunta (*isto difere da fábrica?*) é do consumidor, e a resposta dela é o
//! [`ColorToken::factory`] / [`NumToken::factory_px`].
//!
//! E *se o valor é admissível*: um laço de alias, um comprimento negativo, uma fórmula que não
//! parseia. Quem decide isso são as portas de escrita ([`crate::overrides::set_color_overrides`],
//! [`crate::num_overrides::set_num_overrides`]) — um 2º validador aqui seria a segunda resposta a
//! *"este valor serve?"*, e a que diverge no dia em que a linguagem ganhar um operador.

use crate::color::{Color, ColorToken};
use crate::num::NumToken;
use crate::num_overrides::{NumOverride, NumValue};
use crate::overrides::{ColorOverride, TokenValue};
use crate::theme::Theme;

/// **O que uma entrada de fora diz que o token vale** — a união das duas famílias.
///
/// ⚠️ Ela é *emprestada* (`&str` para as duas espécies de texto) de propósito: o roteamento corre
/// sobre listas inteiras vindas de um arquivo, e clonar uma chave por entrada para a devolver logo
/// a seguir seria uma alocação por token para responder a uma pergunta que não guarda nada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthoredValue<'a> {
    /// Uma cor literal — só a família de COR a aceita.
    Colour(Color),
    /// Um comprimento em px — só a família NUMÉRICA o aceita.
    Px(f32),
    /// **A CHAVE do token seguido.** A família sai da do token que SEGUE, e o alvo tem de ser da
    /// mesma: um alias atravessa famílias *dentro* da numérica (px é px) e nunca entre cor e px.
    Alias(&'a str),
    /// Uma FÓRMULA, como texto — só a família NUMÉRICA a aceita.
    Formula(&'a str),
}

/// Uma entrada já emparelhada com o token dela.
#[derive(Clone, Debug, PartialEq)]
pub enum Routed {
    /// Um override de cor, pronto para a porta de escrita.
    Colour(ColorOverride),
    /// Um override numérico, pronto para a porta de escrita.
    Num(NumOverride),
}

/// **Que token é este, e o valor cabe nele?** — `None` quando a entrada não tem dono.
///
/// Três razões para uma entrada cair, e todas são *"nenhuma porta deste app emite isto"*:
///
/// 1. **A chave não existe** no design system — a tabela de fábrica é a autoridade sobre quais
///    tokens existem, e um arquivo pode vir de uma versão que tinha outros.
/// 2. **O valor é da outra família** (uma cor sob `spacing.md`, um número sob `accent`). Dobrá-lo
///    para o que der faria a re-vestida inventar-se sozinha.
/// 3. **O alvo de um alias não existe** — um elo pendurado no vazio não tem valor a devolver.
///
/// ⚠️ Quem chama **conta** o que caiu e diz: uma tabela que encolhe em silêncio lê-se como *"eu
/// nunca autorei isto"*, e o artista procuraria a cor onde ela não está.
#[must_use]
pub fn route(theme: Theme, key: &str, value: AuthoredValue<'_>) -> Option<Routed> {
    // ⚠️ A COR primeiro, e a ordem é inócua **porque as duas famílias são disjuntas nas chaves** —
    // há gate a afirmá-lo (`no_key_is_claimed_by_both_families`). Sem ele esta linha escolheria um
    // dono, e a escolha seria silenciosa.
    if let Some(token) = ColorToken::from_key(key) {
        let v = match value {
            AuthoredValue::Colour(c) => TokenValue::Literal(c),
            AuthoredValue::Alias(target) => TokenValue::Alias(ColorToken::from_key(target)?),
            AuthoredValue::Px(_) | AuthoredValue::Formula(_) => return None,
        };
        return Some(Routed::Colour(ColorOverride {
            theme,
            token,
            value: v,
        }));
    }
    let token = NumToken::from_key(key)?;
    let v = match value {
        AuthoredValue::Px(px) => NumValue::Literal(px),
        AuthoredValue::Alias(target) => NumValue::Alias(NumToken::from_key(target)?),
        AuthoredValue::Formula(src) => NumValue::Expr(src.to_string()),
        AuthoredValue::Colour(_) => return None,
    };
    Some(Routed::Num(NumOverride {
        theme,
        token,
        value: v,
    }))
}

/// **O valor de FÁBRICA deste token, se ele existir** — a pergunta que separa *uma escolha do
/// artista* de *a fábrica escrita por extenso*.
///
/// ⚠️ Só faz sentido para um valor LITERAL: um alias e uma fórmula são estruturais — o artista
/// autorou o *vínculo*, e o número que ele por acaso dá hoje não o desfaz. É por isso que o
/// retorno é a união das duas famílias e não um `f32`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Factory {
    /// A cor de fábrica deste token, neste modo.
    Colour(Color),
    /// O comprimento de fábrica deste token (a escala de fábrica não tem modo).
    Px(f32),
}

/// A fábrica da chave, ou `None` se o design system não a conhece.
#[must_use]
pub fn factory(theme: Theme, key: &str) -> Option<Factory> {
    if let Some(t) = ColorToken::from_key(key) {
        return Some(Factory::Colour(t.factory(theme)));
    }
    NumToken::from_key(key).map(|t| Factory::Px(t.factory_px()))
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
