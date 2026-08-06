//! **A ESCALA VIRA AUTORÁVEL** — a camada de override sobre os tokens numéricos (plano UI/UX
//! W4c.1), irmã da [`crate::overrides`] e no molde exacto dela.
//!
//! # O que é IGUAL ao de cor, e por isso não se re-litiga aqui
//!
//! A chave é o par `(modo, token)`; a escrita tem **uma porta**; a cadeia de aliases é seguida na
//! **leitura** e nunca achatada na escrita; o ciclo é recusado **na porta**, não sobrevivido na
//! leitura; vazio ⇒ o valor de fábrica, bit a bit. O *porquê* de cada uma dessas decisões está
//! escrito na [`crate::overrides`] e vale verbatim — repeti-lo aqui daria dois textos a envelhecer
//! em direcções diferentes.
//!
//! # ⚠️ O que é DIFERENTE, e é uma coisa só
//!
//! **A fábrica de um número não tem modo.** `docs/design/tokens.json` guarda `spacing.*` no topo,
//! fora de `themes`, então os quatro modos partilham a MESMA escala de fábrica — enquanto cada um
//! tem a sua tabela de cor.
//!
//! A chave continua a ser `(modo, token)` mesmo assim, e a razão é o modelo que este plano segue:
//! num sistema de variáveis (Figma), **um modo é uma coluna de valores**, e um espaçamento pode
//! fazer parte da identidade de um modo tanto quanto uma cor. A consequência honesta, que fica
//! nomeada em vez de descoberta: **autorar `spacing.md` no Forge não o move no Workshop** — e um
//! token que ninguém autorou vale o mesmo nos quatro, porque a fábrica é uma só.
//!
//! # Um alias atravessa FAMÍLIAS, e isso é a unidade a falar
//!
//! `radius.md → spacing.md` é legal porque os dois são **px**: a família é *o que se mede em
//! pixels* ([`crate::num::NumToken`]), então um alias dentro dela nunca troca de unidade. É também
//! o que faz o ciclo ser uma pergunta só — um grafo, não três.

use std::cell::RefCell;

use crate::num::NumToken;
use crate::theme::Theme;

/// **O que um token numérico autorado VALE** — as duas espécies de resposta à mesma pergunta.
///
/// ⚠️ Um `enum`, e não um `px: f32` com um `alias: Option<NumToken>` ao lado: os dois campos seriam
/// mutuamente exclusivos e nada no tipo o diria, então um slot poderia carregar os dois e a leitura
/// teria de escolher um vencedor que ninguém especificou. A representação apaga o caso.
///
/// ⚠️ **A terceira espécie — `Expr` — é a W4c.3**, e ela entra AQUI (mais um variant), nunca num
/// mapa ao lado: math é outra resposta à mesma pergunta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumValue {
    /// Um número em px, digitado pelo artista.
    Literal(f32),
    /// **Este token SEGUE aquele**, no mesmo modo.
    Alias(NumToken),
}

/// Um valor autorado: **que token, em que modo, valendo o quê**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumOverride {
    pub theme: Theme,
    pub token: NumToken,
    pub value: NumValue,
}

/// **Onde a cadeia de aliases TERMINA** — a resposta que o [`crate::num::NumToken::px`] consome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthoredNum {
    /// A cadeia termina num número escolhido.
    Px(f32),
    /// A cadeia termina num token SEM valor autorado ⇒ vale a fábrica **dele**.
    Factory(NumToken),
}

/// A escrita foi RECUSADA — e a recusa **diz porquê**, porque um gesto que não acontece sem nada na
/// tela é indistinguível de um botão quebrado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumRefusal {
    /// O elo pedido fecharia um laço, e é **aqui** que ele fecha.
    Cycle {
        token: NumToken,
        target: NumToken,
        at: NumToken,
    },
    /// O número não é um comprimento.
    ///
    /// ⚠️ Não-finito **e negativo** caem no mesmo braço porque falham pela mesma razão: os três
    /// membros desta família são **comprimentos**, e nem um `NaN` nem um `-4` descrevem um. Não há
    /// máximo aqui, e a ausência é deliberada — ver o cabeçalho do [`crate::num`].
    NotALength(f32),
}

/// **Quantos saltos uma cadeia honesta pode ter** — a casa dos pombos sobre esta família.
const fn max_alias_hops() -> usize {
    NumToken::ALL.len()
}

thread_local! {
    /// A lista de valores autorados. **Esparsa**: só o que difere da fábrica viaja.
    static OVERRIDES: RefCell<Vec<NumOverride>> = const { RefCell::new(Vec::new()) };
    /// *Existe algum override numérico?* — a única pergunta que o caminho comum paga.
    ///
    /// ⚠️ **Um flag PRÓPRIO, separado do de cor**: um flag partilhado faria abrir o picker de uma
    /// cor pôr a escala inteira no caminho lento, e vice-versa.
    static ANY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O que o SLOT `(modo, token)` diz — **sem seguir a cadeia**.
///
/// É a pergunta do PAINEL (*"esta linha está autorada, e como?"*) e da PERSISTÊNCIA.
#[must_use]
pub fn num_override(theme: Theme, token: NumToken) -> Option<NumValue> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| slot(&o.borrow(), theme, token))
}

/// O slot cru, sem o guard do `ANY` — o helper interno que as caminhadas usam.
fn slot(list: &[NumOverride], theme: Theme, token: NumToken) -> Option<NumValue> {
    list.iter()
        .find(|e| e.theme == theme && e.token == token)
        .map(|e| e.value)
}

/// **Onde a cadeia deste token termina** — a pergunta que o `px(theme)` faz.
///
/// `None` = nada autorado neste slot (o caminho comum, e ele custa **uma leitura de bool**).
#[must_use]
pub fn resolved_num_override(theme: Theme, token: NumToken) -> Option<AuthoredNum> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| {
        let list = o.borrow();
        let mut cur = token;
        for _ in 0..max_alias_hops() {
            match slot(&list, theme, cur) {
                None => {
                    return if cur == token {
                        None
                    } else {
                        Some(AuthoredNum::Factory(cur))
                    };
                }
                Some(NumValue::Literal(v)) => return Some(AuthoredNum::Px(v)),
                Some(NumValue::Alias(next)) => cur = next,
            }
        }
        // Excedeu a casa dos pombos ⇒ há um laço. A porta de escrita não deixa um nascer, então
        // isto só é alcançável por uma tabela corrompida fora daqui: cai na fábrica em vez de girar.
        None
    })
}

/// *Fazer `token` seguir `target` fecharia um laço?* — devolve **onde** ele fecha.
///
/// ⚠️ A caminhada é a do [`crate::alias_walk`], partilhada com a camada de cor: a lei do ciclo é
/// sobre o GRAFO e não sabe nada sobre o que um slot vale.
fn closes_a_loop(
    list: &[NumOverride],
    theme: Theme,
    token: NumToken,
    target: NumToken,
) -> Option<NumToken> {
    crate::alias_walk::closes_a_loop(token, target, max_alias_hops(), |cur| {
        match slot(list, theme, cur) {
            Some(NumValue::Alias(next)) => Some(next),
            _ => None,
        }
    })
}

/// **A porta ÚNICA de escrita** — `None` devolve o token à fábrica.
///
/// ⚠️ Escrever *o número de fábrica* como override **não é** o mesmo que soltar: o arquivo passaria
/// a carregar um valor que só por acaso coincide, e re-editar o `tokens.json` deixaria de alcançar
/// aquele token em silêncio. Soltar é `None`, e é o que o botão *Reset* faz.
///
/// # Erros
///
/// - [`NumRefusal::Cycle`] para um alias que fecharia um laço, dizendo **onde**.
/// - [`NumRefusal::NotALength`] para um literal que não é um comprimento.
pub fn set_num_override(
    theme: Theme,
    token: NumToken,
    value: Option<NumValue>,
) -> Result<(), NumRefusal> {
    OVERRIDES.with(|o| {
        let mut list = o.borrow_mut();
        match value {
            Some(NumValue::Literal(v)) if !is_a_length(v) => {
                return Err(NumRefusal::NotALength(v));
            }
            Some(NumValue::Alias(target)) => {
                if let Some(at) = closes_a_loop(&list, theme, token, target) {
                    return Err(NumRefusal::Cycle { token, target, at });
                }
            }
            _ => {}
        }
        let at = list
            .iter()
            .position(|e| e.theme == theme && e.token == token);
        match (at, value) {
            (Some(i), Some(v)) => list[i].value = v,
            (Some(i), None) => {
                list.remove(i);
            }
            (None, Some(v)) => list.push(NumOverride {
                theme,
                token,
                value: v,
            }),
            (None, None) => {}
        }
        ANY.with(|a| a.set(!list.is_empty()));
        Ok(())
    })
}

/// *Este número descreve um comprimento?* — a pergunta que a porta faz, **uma vez**.
///
/// ⚠️ `>= 0.0` é falso para `NaN`, então o não-finito já cairia aqui; a chamada a `is_finite` fica
/// porque `+inf` é `>= 0.0` e **passaria** — e um infinito num layout não é um valor grande, é um
/// `NaN` a acontecer na primeira subtracção.
#[must_use]
pub fn is_a_length(v: f32) -> bool {
    v.is_finite() && v >= 0.0
}

/// Todos os valores autorados — o que a persistência guarda.
///
/// ⚠️ **Ordenado por `(modo, chave do token)`**: o arquivo é comparado byte a byte por gates e por
/// quem investiga um diff, e uma lista cuja ordem depende da ordem dos cliques faria dois
/// documentos logicamente iguais parecerem diferentes.
#[must_use]
pub fn num_overrides() -> Vec<NumOverride> {
    let mut out = OVERRIDES.with(|o| o.borrow().clone());
    out.sort_by(|a, b| (a.theme as u8, a.token.key()).cmp(&(b.theme as u8, b.token.key())));
    out
}

/// Instala a lista inteira (o load de projeto). Substitui o que houver.
///
/// Devolve **quantas entradas foram descartadas** — por fecharem um laço, ou por não serem um
/// comprimento. ⚠️ As duas alternativas ao descarte são piores: recusar o arquivo inteiro joga fora
/// uma re-vestida por causa de duas linhas, e aceitar põe numa tabela um estado que a porta de
/// escrita promete não ter. O que cai é **DITO** pelo chamador.
pub fn set_num_overrides(list: Vec<NumOverride>) -> usize {
    let mut kept: Vec<NumOverride> = Vec::with_capacity(list.len());
    let mut dropped = 0usize;
    for e in list {
        let ok = match e.value {
            // ⚠️ Contra o que JÁ foi aceite, não contra a lista de entrada: é isso que torna o
            // resultado acíclico por construção, seja qual for a ordem em que os laços aparecem.
            NumValue::Alias(target) => closes_a_loop(&kept, e.theme, e.token, target).is_none(),
            NumValue::Literal(v) => is_a_length(v),
        };
        if ok {
            kept.push(e);
        } else {
            dropped += 1;
        }
    }
    OVERRIDES.with(|o| {
        let mut cur = o.borrow_mut();
        *cur = kept;
        ANY.with(|a| a.set(!cur.is_empty()));
    });
    dropped
}

/// Devolve **toda** a escala à fábrica.
pub fn clear_num_overrides() {
    // Uma lista vazia não tem nada a descartar — o zero é aritmética, não confiança.
    let _ = set_num_overrides(Vec::new());
}

/// Quantos tokens numéricos deste modo estão autorados — o readout que o painel mostra.
#[must_use]
pub fn num_overridden_count(theme: Theme) -> usize {
    if !ANY.with(std::cell::Cell::get) {
        return 0;
    }
    OVERRIDES.with(|o| o.borrow().iter().filter(|e| e.theme == theme).count())
}

#[cfg(test)]
#[path = "num_overrides_tests.rs"]
mod tests;
