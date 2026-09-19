//! ⭐⭐⭐ **A FRASE QUE FALTA — um BALÃO por fileira, quando a tabela de textos tiver uma.**
//!
//! # ⛔⛔ O report que a obrigou
//!
//! Enio, 2026-09-19, sobre a camada de estilo: *«Zone pivot não sei para que serve»*. ⚠️ E ele tinha
//! razão duas vezes: no estado em que o painel abre aquele botão é **matematicamente um no-op** (as
//! duas tintas nascem brancas ⇒ o parêntesis é exactamente zero), e mesmo armado o nome não diz o
//! que ele faz. *Um controlo cujo nome não chega para o usar é um controlo que o artista não usa.*
//!
//! # ⭐⭐ O mecanismo já alcançava este painel — faltavam duas linhas
//!
//! Medido na auditoria: um `Move` real sobre o slider põe o `hot_id` correcto e o
//! `WidgetStore::set_tooltip` aceita a dica; o passe de hover (`paint_hover_tooltip`) corre **depois
//! de todos os painéis**, logo ele já a pintaria. As chamadas a `set_tooltip` nesta crate eram
//! **`0`**.
//!
//! # ⚠️⚠️ A CONVENÇÃO, e porque ela não é uma tabela
//!
//! A dica de uma fileira é a chave dela mais o sufixo [`SUFIXO`]: `panel.model3d.style.pivot` ⇒
//! **`panel.model3d.style.pivot.tip`**. ⛔ Uma tabela `key → tip` neste ficheiro seria a segunda
//! lista ao lado da que o produtor das fileiras já mantém, e a que envelhece na primeira fileira
//! nova — exactamente o defeito que o `CHIP_FAMILIES` do [`crate::populate`] curou para os botões.
//!
//! ⭐ **A chave ausente é o estado NORMAL, não um erro:** a esmagadora maioria das fileiras não
//! precisa de explicação, e uma dica que repete o rótulo é ruído que ensina o artista a ignorar os
//! balões — exactamente quando um deles passar a ser o que importa.
//!
//! # ⛔⛔⛔ Porque a pergunta *«esta chave existe?»* precisa de um MEMO
//!
//! A `ph2d-i18n` responde a uma chave desconhecida **devolvendo a própria chave**, e fá-lo com um
//! `Box::leak` (`leak_key`): *«só dispara em gralhas do programador, logo a fuga por gralha é
//! aceitável»*. Isso é verdade para uma gralha e **falso** para um sondador: perguntar *«tens dica
//! para esta fileira?»* a cada fileira, a cada quadro, vazaria uma `String` **por quadro** — e o
//! gate `every_vertex_row_has_a_label` do lado do documento já nomeia essa fuga por escrito.
//!
//! ⇒ a porta honesta, sem tocar na crate de textos, é **perguntar UMA vez por chave distinta**. O
//! memo é `thread_local` e imutável na prática: a tabela de textos é `const` e uma chave nunca muda
//! de resposta em runtime.
//!
//! ⚠️ **O preço, nomeado:** cada chave **sem** dica deixa uma cópia dela própria na heap, **uma vez
//! por processo** (`~40 B` para os nomes deste painel). ⛔ Não cresce com os quadros, não cresce com
//! as linhas — só com o número de nomes distintos que o painel chega a pintar. *É a diferença entre
//! uma fuga limitada e uma fuga por quadro, e é ela que torna esta porta aceitável.*
//!
//! ⚠️ **E quando a `ph2d-i18n` migrar para Fluent** (o doc do `tr` promete-o), esta porta passa a ser
//! o sítio a mudar: um `try_tr` que devolva `Option` apaga o memo **e** a fuga.

use ph2d_editor_core::interaction::WidgetStore;
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::state::ParamRow;

/// O sufixo que transforma a chave de uma fileira na chave da dica dela.
pub(crate) const SUFIXO: &str = ".tip";

thread_local! {
    /// ⚠️ A resposta por chave, perguntada uma vez. Ver o cabeçalho.
    ///
    /// ⛔ **`BTreeMap` e nunca `HashMap`** — é a espinha do determinismo deste repo, e há lint
    /// estrutural a impô-lo (`CLAUDE.md` §5). ⚠️ Aqui a ordem nem é observável, e é exactamente por
    /// isso que a lei é do TIPO e não do uso: *uma excepção que depende de quem a lê é uma excepção
    /// que a próxima leitura não tem.*
    static MEMO: RefCell<BTreeMap<&'static str, Option<&'static str>>> =
        const { RefCell::new(BTreeMap::new()) };
}

#[cfg(test)]
thread_local! {
    /// ⭐ **Quantas vezes a pergunta chegou de facto à `ph2d-i18n`** — o instrumento que prova que a
    /// fuga é por CHAVE e não por quadro. Ver `dica_tests::a_pergunta_chega_uma_vez_por_chave`.
    static PERGUNTAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Quantas vezes o memo falhou e a pergunta foi ao `tr` — só em teste.
#[cfg(test)]
pub(crate) fn perguntas_feitas() -> usize {
    PERGUNTAS.with(std::cell::Cell::get)
}

/// ⭐⭐⭐ **A dica de uma fileira, ou `None`** — e a chave ausente não custa nada depois da 1.ª vez.
#[must_use]
pub(crate) fn dica_da_fileira(key: &'static str) -> Option<&'static str> {
    MEMO.with(|m| {
        if let Some(r) = m.borrow().get(key) {
            return *r;
        }
        #[cfg(test)]
        PERGUNTAS.with(|p| p.set(p.get() + 1));
        let chave = format!("{key}{SUFIXO}");
        let texto = ph2d_i18n::tr(&chave);
        // ⚠️ **O contrato do `tr` é o round-trip**: chave desconhecida devolve a própria chave. É a
        // única resposta que a crate de textos dá hoje, e compará-la é o que evita inventar uma
        // segunda tabela deste lado.
        let r = (texto != chave).then_some(texto);
        m.borrow_mut().insert(key, r);
        r
    })
}

/// ⭐⭐ **Pendura a dica desta fileira em cada widget que ela registou.**
///
/// ⚠️ **São VÁRIOS ids por fileira, e a razão é o `hot_id`:** numa linha viva o rato pode estar sobre
/// o trilho **ou** sobre o campo, e o passe de hover pergunta pelo id que está quente. Pendurar só
/// num deles daria uma dica que aparece em metade da linha — *pior do que nenhuma, porque o artista
/// conclui que a explicação não existe*.
///
/// ⛔ **Uma fileira TRAVADA não regista nada** (é a lei do `paint_fact`), logo não recebe balão: não
/// há widget quente onde ele possa nascer. ⚠️ A razão da trava já é pintada **ao lado** dela, que é
/// a superfície que o dono escolheu em 2026-09-18.
pub(crate) fn pendura(store: &mut WidgetStore, row: &ParamRow, ids: &[ph2d_a11y::NodeId]) {
    let Some(texto) = dica_da_fileira(row.key) else {
        return;
    };
    for id in ids {
        store.set_tooltip(*id, texto);
    }
}

#[cfg(test)]
#[path = "dica_tests.rs"]
mod tests;
