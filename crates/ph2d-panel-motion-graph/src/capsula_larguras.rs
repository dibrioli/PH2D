//! ⭐⭐⭐ **QUANTO MEDE O NOME DE UM NÓ** — o canal que leva uma medida de TEXTO até à
//! GEOMETRIA, que não tem medidor nenhum.
//!
//! Ordem do dono (2026-09-20): *«aumenta a largura do retângulo conforme o tamanho do nome»*.
//! ⇒ a largura de uma cápsula deixa de ser a do cartão e passa a depender do NOME — e quem
//! precisa dela não é só o pintor: é o **hit-rect** (senão metade da pastilha não responde ao
//! dedo) e é o **pino** (senão ele fica dentro da forma em vez de na borda dela).
//!
//! ⛔⛔ **E uma ESTIMATIVA por contagem de caracteres não serve, com o número:** o pior avanço do
//! catálogo é `0,6730` por caractere (*«ADSR»*, maiúsculas largas) e a média de um nome comprido
//! é `0,497` — uma estimativa generosa o suficiente para nunca CORTAR entrega *«Simulation
//! Zone»* numa pastilha de `321` unidades onde o texto mede `232`, **`38 %` de enchimento**. ⚠️ É
//! a mesma estimativa que a fonte usava e que custou `20 %` de corpo; *aqui ela custaria a
//! pastilha inteira*.
//!
//! ⭐⭐ **Porque um cache serve, e não é uma aposta sobre a ordem do quadro:** o
//! `crate::interact::process` corre **DENTRO** do `paint`, depois desta medição — logo o hit-rect
//! deste quadro e o desenho deste quadro lêem o MESMO número, sem um quadro de atraso. *A
//! armadilha clássica (o `size` de agora contra o rect publicado no quadro anterior) não existe
//! aqui porque não há dois quadros envolvidos.*

use crate::snapshot::GraphViewSnapshot;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    /// `nome → largura em unidades de grafo`, ao corpo [`crate::paint::paint_capsula::CAPSULA_FONTE`].
    ///
    /// ⚠️ **A chave é o NOME e não o id do nó**, porque é do nome que a largura depende: dois nós
    /// do mesmo tipo partilham a medida, e um nó renomeado mede-se outra vez.
    static LARGURAS: RefCell<BTreeMap<String, f32>> = const { RefCell::new(BTreeMap::new()) };
}

/// Acima disto a tabela é esvaziada. ⚠️ Ela cresce com nomes DISTINTOS, e o artista pode
/// renomear um nó quantas vezes quiser numa sessão — *um cache sem tecto é uma fuga de memória
/// com um nome simpático*. O catálogo tem 136 tipos, logo este tecto é ~30× a população natural.
const TECTO: usize = 4096; // LITERAL-PX-OK: entradas de cache, não uma medida de desenho

/// **Mede o que falta** — chamado uma vez por quadro, antes de qualquer geometria.
///
/// ⭐ Só mede o que ainda não está na tabela, logo em regime é uma busca por nó e zero trabalho
/// de texto. *É isto que torna aceitável chamá-lo em todo quadro em vez de só abaixo do limiar:
/// um quadro que ATRAVESSA o limiar precisa da medida já lá, e adivinhar o regime antes do
/// `interact` (que pode mexer no zoom) seria adivinhar.*
pub(crate) fn medir(text_system: &mut ph2d_text::TextSystem, snap: &GraphViewSnapshot) {
    LARGURAS.with_borrow_mut(|m| {
        if m.len() > TECTO {
            m.clear();
        }
        for n in &snap.nodes {
            if m.contains_key(n.display_name.as_str()) {
                continue;
            }
            let w = text_system.prefix_width_weighted(
                &n.display_name,
                crate::paint::paint_capsula::CAPSULA_FONTE,
                ph2d_text::FontWeight::SEMI_BOLD,
            );
            m.insert(n.display_name.clone(), w);
        }
    });
}

/// A largura MEDIDA deste nome, se alguém já a mediu.
pub(crate) fn largura_medida(nome: &str) -> Option<f32> {
    LARGURAS.with_borrow(|m| m.get(nome).copied())
}

/// Esquece tudo — só para os gates, que montam nomes que o produto nunca vê.
#[cfg(test)]
pub(crate) fn esquece() {
    LARGURAS.with_borrow_mut(BTreeMap::clear);
}

/// Prega uma medida — só para os gates, que não têm medidor de texto.
#[cfg(test)]
pub(crate) fn prega(nome: &str, largura: f32) {
    LARGURAS.with_borrow_mut(|m| {
        m.insert(nome.to_string(), largura);
    });
}
