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
#[derive(Clone, Debug, PartialEq)]
pub enum NumValue {
    /// Um número em px, digitado pelo artista.
    Literal(f32),
    /// **Este token SEGUE aquele**, no mesmo modo.
    Alias(NumToken),
    /// **Uma FÓRMULA** — `{spacing.md} * 2` (plano UI/UX W4c.3).
    ///
    /// ⚠️ Guardada como **TEXTO**, e não como uma árvore parseada, por duas razões que apontam
    /// para o mesmo lado: é o texto que o arquivo grava (uma árvore teria de ser serializada, e o
    /// formato passaria a depender da forma do IR), e é o texto que o artista reabre e edita. É a
    /// mesma decisão que a `motion.expression` tomou.
    ///
    /// ⚠️ E é ela que tira o `Copy` deste enum. A alternativa — internar a fórmula num índice —
    /// poria uma segunda tabela ao lado da que já existe, para poupar clones num caminho que corre
    /// **uma vez por quadro sobre 21 slots**.
    Expr(String),
}

/// Um valor autorado: **que token, em que modo, valendo o quê**.
#[derive(Clone, Debug, PartialEq)]
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
///
/// ⚠️ Sem `Copy`: a frase de uma fórmula recusada é uma `String`, e ela vem do parser — dobrá-la num
/// código de erro sem texto poria o artista a adivinhar QUAL caractere o motor não entendeu.
#[derive(Clone, Debug, PartialEq)]
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
    /// A fórmula não foi admitida. Traz a frase pronta para o toast.
    ///
    /// ⚠️ Um braço PRÓPRIO em vez de dobrar em `NotALength`: *"isto não parseia"* e *"isto não é um
    /// comprimento"* mandam o artista a lugares diferentes, e uma mensagem só mandaria a metade
    /// deles ao lugar errado.
    BadFormula(String),
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

/// *Existe ALGUMA coisa autorada, em qualquer modo?*
///
/// ⚠️ É a bandeira que torna a camada gratuita: com ela em `false` toda leitura sai por um `if` e
/// devolve a fábrica. Ela é `pub` para o [`crate::num_runtime`], que precisa de saber se vale a
/// pena encher a tabela — e **não** é a mesma pergunta que o `is_filled` de lá: *"há autoria"* é
/// sobre o GRAFO, *"a tabela vale"* é sobre a PROJEÇÃO dele.
#[must_use]
pub fn any_authored() -> bool {
    ANY.with(std::cell::Cell::get)
}

/// O slot cru, sem o guard do `ANY` — o helper interno que as caminhadas usam.
fn slot(list: &[NumOverride], theme: Theme, token: NumToken) -> Option<NumValue> {
    list.iter()
        .find(|e| e.theme == theme && e.token == token)
        .map(|e| e.value.clone())
}

/// **Onde a cadeia deste token termina** — a pergunta que o `px(theme)` faz.
///
/// `None` = nada autorado neste slot (o caminho comum, e ele custa **uma leitura de bool**).
#[must_use]
pub fn resolved_num_override(theme: Theme, token: NumToken) -> Option<AuthoredNum> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| resolve_at(&o.borrow(), theme, token, 0))
}

/// A resolução de UM slot, **recursiva**, porque uma fórmula tem N dependências.
///
/// ⚠️ Ela era um laço (`while` sobre a corrente de aliases) até a math chegar. Um alias tem um
/// sucessor e cabe num passeio; uma expressão tem N, e a resolução dela é uma ÁRVORE. Manter o
/// laço obrigaria a uma pilha explícita para poupar recursão num grafo de **21 nós** que a porta
/// de escrita garante acíclico.
///
/// ⚠️ **`depth` é a rede da tabela CORROMPIDA**, não do uso normal: a porta recusa laços, então o
/// único caminho até aqui é um arquivo editado à mão. Estourar cai na fábrica em vez de girar.
fn resolve_at(
    list: &[NumOverride],
    theme: Theme,
    token: NumToken,
    depth: usize,
) -> Option<AuthoredNum> {
    if depth > max_alias_hops() {
        return None;
    }
    match slot(list, theme, token) {
        None => None,
        Some(NumValue::Literal(v)) => Some(AuthoredNum::Px(v)),
        // ⚠️ A pergunta *"o slot seguinte está autorado?"* é feita ANTES de recursar, e a ordem é
        // load-bearing: um `unwrap_or(Factory(next))` colapsaria DOIS fatos diferentes no mesmo
        // `None` — *a cadeia terminou num slot de fábrica* (que vale a fábrica DELE) e *a rede de
        // profundidade estourou* (que tem de valer a fábrica de QUEM PERGUNTOU). O gate da tabela
        // corrompida nasceu vermelho exactamente aí.
        Some(NumValue::Alias(next)) => {
            if slot(list, theme, next).is_none() {
                return Some(AuthoredNum::Factory(next));
            }
            resolve_at(list, theme, next, depth + 1)
        }
        // ⚠️ A fórmula é avaliada com os valores EFETIVOS das dependências — que podem ser, elas
        // próprias, fórmulas. E o resultado **é conferido**: uma divisão por zero, ou uma cadeia que
        // mudou por baixo desde que a porta a admitiu, cai na FÁBRICA em vez de publicar um NaN.
        // Um comprimento inventado seria indistinguível de um autorado.
        Some(NumValue::Expr(src)) => {
            let v = crate::num_expr::eval(&src, &|t| effective_px(list, theme, t, depth + 1))?;
            is_a_length(v).then_some(AuthoredNum::Px(v))
        }
    }
}

/// Quanto um token VALE agora, para alimentar uma fórmula — autorado, ou a fábrica dele.
fn effective_px(list: &[NumOverride], theme: Theme, token: NumToken, depth: usize) -> f32 {
    match resolve_at(list, theme, token, depth) {
        Some(AuthoredNum::Px(v)) => v,
        Some(AuthoredNum::Factory(t)) => t.factory_px(),
        None => token.factory_px(),
    }
}

/// *Fazer `token` seguir `target` fecharia um laço?* — devolve **onde** ele fecha.
///
/// ⚠️ A caminhada é a do [`crate::alias_walk`], partilhada com a camada de cor: a lei do ciclo é
/// sobre o GRAFO e não sabe nada sobre o que um slot vale.
fn closes_a_loop(
    list: &[NumOverride],
    theme: Theme,
    token: NumToken,
    targets: &[NumToken],
) -> Option<NumToken> {
    crate::alias_walk::closes_a_loop(token, targets, |cur| match slot(list, theme, cur) {
        Some(NumValue::Alias(next)) => vec![next],
        // ⚠️ Uma fórmula que não parseia não tem dependências que se possam seguir — e ela nunca
        // chega aqui pela porta (a admissão vem antes). Este braço existe para a tabela que veio
        // de um arquivo: sem dependências, aquele ramo termina, que é o conservador certo.
        Some(NumValue::Expr(src)) => crate::num_expr::deps_of(&src).unwrap_or_default(),
        _ => Vec::new(),
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
                if let Some(at) = closes_a_loop(&list, theme, token, &[target]) {
                    return Err(NumRefusal::Cycle { token, target, at });
                }
            }
            // ⚠️ A admissão de uma fórmula é feita AQUI, e é o que torna o resto da camada simples:
            // se ela parseia, todo nome dela é um token deste sistema, ela não fecha laço e o
            // resultado É um comprimento, então a LEITURA nunca precisa de ter opinião sobre erro.
            Some(NumValue::Expr(ref src)) => {
                let deps = crate::num_expr::deps_of(src).map_err(NumRefusal::BadFormula)?;
                if let Some(at) = closes_a_loop(&list, theme, token, &deps) {
                    // O `target` da mensagem é onde a busca reencontrou o token de partida — para
                    // uma fórmula não há um alvo único, e apontar o primeiro nome escrito mandaria
                    // o artista ao lugar errado quando o laço passa pelo segundo.
                    return Err(NumRefusal::Cycle {
                        token,
                        target: at,
                        at,
                    });
                }
                // ⚠️ E o VALOR é conferido com a tabela **como ela ficaria**: uma fórmula avaliada
                // contra a tabela de hoje pode dar um comprimento e a de amanhã não. O que a porta
                // pode prometer é o instante da escrita; o resto cai na fábrica, por desenho.
                let src = src.clone();
                let v = crate::num_expr::eval(&src, &|t| effective_px(&list, theme, t, 0))
                    .ok_or_else(|| {
                        NumRefusal::BadFormula("that formula could not be evaluated".into())
                    })?;
                if !is_a_length(v) {
                    return Err(NumRefusal::NotALength(v));
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
            NumValue::Alias(target) => closes_a_loop(&kept, e.theme, e.token, &[target]).is_none(),
            NumValue::Literal(v) => is_a_length(v),
            // ⚠️ Uma fórmula que não parseia é descartada AQUI, e não deixada para a leitura cair
            // na fábrica: o painel mostraria a linha como autorada e o artista veria o valor de
            // fábrica ao lado dela — *autorado* e *inerte* ao mesmo tempo.
            NumValue::Expr(ref src) => crate::num_expr::deps_of(src)
                .is_ok_and(|deps| closes_a_loop(&kept, e.theme, e.token, &deps).is_none()),
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
