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
//! # O ALIAS: um token pode SEGUIR outro (plano UI/UX W4b)
//!
//! Um slot autorado guarda uma cor **ou** o nome de outro token ([`TokenValue`]). Isto é o que
//! separa uma tabela de cores de um **sistema** de design: `border` deixa de ser um valor que por
//! acaso combina com `surface` e passa a ser *o mesmo que* `surface` — re-vestir um re-veste o
//! outro, em todos os modos, sem o artista repetir a escolha.
//!
//! ⚠️ **A cadeia é seguida na LEITURA, nunca achatada na escrita.** Achatar (guardar a cor
//! resolvida no momento em que o alias é criado) é o que torna o alias uma cópia: mudar o alvo
//! depois deixaria de alcançar quem o segue, e o vínculo que o artista autorou desapareceria sem
//! nada dizer. É a mesma lei do `cooked()` do vetor — *o documento guarda o AUTORADO, o mundo
//! consome o DERIVADO*.
//!
//! ⚠️ **E o CICLO é recusado na PORTA, não sobrevivido na leitura.** `a → b → a` não tem valor
//! nenhum a devolver, então a única pergunta é *onde* isso é impedido: recusar na escrita torna a
//! tabela acíclica **por construção** e mantém a leitura — que corre por 44 widgets em todo frame —
//! sem trabalho defensivo. A caminhada ainda tem um teto (a casa dos pombos, [`max_alias_hops`]),
//! porque uma tabela pode chegar de um ARQUIVO; as duas camadas têm gate cada.
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

/// **O que um token autorado VALE** — as duas espécies de resposta à MESMA pergunta.
///
/// Um slot guarda uma cor literal **ou** o nome de outro token. Não são duas features: são duas
/// respostas a *"que cor tem este token, neste modo?"*, e por isso moram no mesmo slot e passam
/// pela mesma porta de escrita. Um segundo mapa lateral de aliases seria a segunda tabela que
/// discorda da primeira no dia em que alguém escreve nos dois.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenValue {
    /// Uma cor escolhida no picker.
    Literal(Color),
    /// **Este token SEGUE aquele**, no mesmo modo.
    ///
    /// ⚠️ *No mesmo modo* é o que mantém a re-vestida viva: se `border` segue `surface`, então em
    /// cada um dos quatro modos ele vale o `surface` **daquele** modo. Um alias que atravessasse
    /// modos faria trocar de tema deixar de mudar aquele token.
    Alias(ColorToken),
}

/// Um valor autorado: **que token, em que modo, valendo o quê**.
///
/// ⚠️ A chave é o par `(modo, token)` e não só o token: os quatro modos são quatro respostas à
/// mesma pergunta, e um override que valesse para todos faria trocar de tema deixar de re-vestir —
/// que é a feature que o modo tem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorOverride {
    pub theme: Theme,
    pub token: ColorToken,
    pub value: TokenValue,
}

/// **Onde a cadeia de aliases TERMINA** — a resposta que o [`crate::ColorToken::resolve`] consome.
///
/// Duas terminações, e a diferença importa: uma cadeia pode acabar numa cor que alguém escolheu,
/// ou num token que **ninguém autorou**, e nesse caso o valor é o de FÁBRICA daquele token — que
/// esta crate não vai buscar aqui, porque a tabela gerada é do `color.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authored {
    /// A cadeia termina numa cor literal.
    Colour(Color),
    /// A cadeia termina num token SEM valor autorado ⇒ vale a fábrica **dele**.
    Factory(ColorToken),
}

/// A escrita foi RECUSADA porque fecharia um laço.
///
/// ⚠️ Ela carrega os três tokens porque a mensagem que o artista lê tem de dizer *o que* ele pediu
/// **e** *onde* o laço fecha — "não deu" sem o porquê é a recusa silenciosa que este repo trata
/// como pior que o defeito.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasCycle {
    /// O token que ia passar a seguir alguém.
    pub token: ColorToken,
    /// O alvo pedido.
    pub target: ColorToken,
    /// Onde a cadeia se fecha sobre si mesma.
    pub at: ColorToken,
}

/// **Quantos saltos uma cadeia honesta pode ter** — e o número não foi escolhido, é o princípio da
/// casa dos pombos: só existem `ColorToken::ALL.len()` slots por modo, então uma caminhada que dê
/// mais saltos do que isso **visitou o mesmo token duas vezes**, e isso É um ciclo.
///
/// ⚠️ Ele não é um teto de recurso e nem se aproxima de nada: as cadeias reais têm 1 ou 2 saltos
/// (o laço termina no primeiro slot não-alias). Ele existe para que uma tabela corrompida por fora
/// não trave o app, e tem gate próprio — a porta de escrita já recusa ciclos, então esta é a
/// segunda camada, e camadas em série querem um gate cada.
const fn max_alias_hops() -> usize {
    ColorToken::ALL.len()
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

/// O que o SLOT `(modo, token)` diz — **sem seguir a cadeia**.
///
/// É a pergunta do PAINEL (*"esta linha está autorada, e como?"*) e da PERSISTÊNCIA. Quem quer a
/// cor efetiva usa [`resolved_override`], que é a outra pergunta: **o que este slot DIZ** e **onde
/// a cadeia TERMINA** não são a mesma coisa, e colapsá-las tiraria do painel a única informação
/// que ele precisa para desenhar `→ surface` na linha.
#[must_use]
pub fn color_override(theme: Theme, token: ColorToken) -> Option<TokenValue> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| slot(&o.borrow(), theme, token))
}

/// O slot cru, sem o guard do `ANY` — o helper interno que as caminhadas usam.
fn slot(list: &[ColorOverride], theme: Theme, token: ColorToken) -> Option<TokenValue> {
    list.iter()
        .find(|e| e.theme == theme && e.token == token)
        .map(|e| e.value)
}

/// **Onde a cadeia deste token termina** — a pergunta que o `resolve` faz.
///
/// `None` = nada autorado neste slot (o caminho comum, e ele custa **uma leitura de bool**).
#[must_use]
pub fn resolved_override(theme: Theme, token: ColorToken) -> Option<Authored> {
    if !ANY.with(std::cell::Cell::get) {
        return None;
    }
    OVERRIDES.with(|o| {
        let list = o.borrow();
        let mut cur = token;
        for _ in 0..max_alias_hops() {
            match slot(&list, theme, cur) {
                // Chegámos a um token que ninguém autorou. Se for o próprio token de partida, não
                // há nada autorado; se for outro, é o fim de uma cadeia e vale a fábrica DELE.
                None => {
                    return if cur == token {
                        None
                    } else {
                        Some(Authored::Factory(cur))
                    };
                }
                Some(TokenValue::Literal(c)) => return Some(Authored::Colour(c)),
                Some(TokenValue::Alias(next)) => cur = next,
            }
        }
        // Excedeu a casa dos pombos ⇒ há um laço. As portas de escrita não deixam um nascer, então
        // isto só é alcançável por uma tabela corrompida fora daqui: cai na fábrica em vez de girar.
        None
    })
}

/// *Fazer `token` seguir `target` fecharia um laço?* — devolve **onde** ele fecha.
///
/// ⚠️ **A caminhada mora no [`crate::alias_walk`]**, e não aqui: a pergunta é sobre o GRAFO e não
/// sabe nada sobre cor, então a camada numérica faz exactamente a mesma e uma segunda cópia seria
/// a que esquece o auto-alias no dia em que alguém ajusta esta. O que fica aqui é o que é DESTA
/// família: como se lê um slot, e quantos slots existem.
fn closes_a_loop(
    list: &[ColorOverride],
    theme: Theme,
    token: ColorToken,
    target: ColorToken,
) -> Option<ColorToken> {
    crate::alias_walk::closes_a_loop(token, target, max_alias_hops(), |cur| {
        match slot(list, theme, cur) {
            Some(TokenValue::Alias(next)) => Some(next),
            _ => None,
        }
    })
}

/// **A porta ÚNICA de escrita** — `None` devolve o token à fábrica.
///
/// ⚠️ Escrever *a cor de fábrica* como override **não é** o mesmo que soltar: o arquivo passaria a
/// carregar um valor que só por acaso coincide, e re-editar a tabela de fábrica deixaria de
/// alcançar aquele token em silêncio. Soltar é `None`, e é o que o botão *Reset* faz.
///
/// # Erros
///
/// Recusa um [`TokenValue::Alias`] que fecharia um laço, dizendo **onde** ele fecha. Um literal
/// nunca pode falhar (ele TERMINA uma cadeia; nunca a alonga), e é por isso que os chamadores que
/// só escrevem cor podem afirmar isso no `expect`.
pub fn set_color_override(
    theme: Theme,
    token: ColorToken,
    value: Option<TokenValue>,
) -> Result<(), AliasCycle> {
    OVERRIDES.with(|o| {
        let mut list = o.borrow_mut();
        if let Some(TokenValue::Alias(target)) = value
            && let Some(at) = closes_a_loop(&list, theme, token, target)
        {
            return Err(AliasCycle { token, target, at });
        }
        let at = list
            .iter()
            .position(|e| e.theme == theme && e.token == token);
        match (at, value) {
            (Some(i), Some(v)) => list[i].value = v,
            (Some(i), None) => {
                list.remove(i);
            }
            (None, Some(v)) => list.push(ColorOverride {
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
///
/// Devolve **quantas entradas foram descartadas** por fecharem um laço. ⚠️ Um arquivo pode trazer
/// um ciclo (foi editado à mão, ou vem de um build onde a lista tinha outra ordem), e as duas
/// alternativas ao descarte são piores: **recusar o arquivo inteiro** joga fora uma re-vestida por
/// causa de duas linhas, e **aceitar** põe um laço numa tabela que a porta de escrita promete não
/// ter. O que cai é DITO pelo chamador — uma tabela que encolhe em silêncio lê-se como *"eu nunca
/// autorei isto"*, a mesma lei que o `project_tokens::install` já aplica ao token desconhecido.
pub fn set_color_overrides(list: Vec<ColorOverride>) -> usize {
    let mut kept: Vec<ColorOverride> = Vec::with_capacity(list.len());
    let mut dropped = 0usize;
    for e in list {
        // ⚠️ Contra o que JÁ foi aceite, não contra a lista de entrada: é isso que torna o
        // resultado acíclico por construção, seja qual for a ordem em que os laços aparecem.
        if let TokenValue::Alias(target) = e.value
            && closes_a_loop(&kept, e.theme, e.token, target).is_some()
        {
            dropped += 1;
            continue;
        }
        kept.push(e);
    }
    OVERRIDES.with(|o| {
        let mut cur = o.borrow_mut();
        *cur = kept;
        ANY.with(|a| a.set(!cur.is_empty()));
    });
    dropped
}

/// Devolve **toda** a tabela à fábrica.
pub fn clear_color_overrides() {
    // Uma lista vazia não tem laço a descartar — o zero é aritmética, não confiança.
    let _ = set_color_overrides(Vec::new());
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
