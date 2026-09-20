//! **ONDE UM NÓ ARRASTADO É LARGADO** — as duas leituras que uma largada pode ter, ordem do dono
//! (2026-09-19):
//!
//! > *«Se arrastar um nó no grafo em cima de outro nó, eles mudam de posição na cadeia. Se
//! > arrastar num nó em cima de uma conexão (linha) […] ele passa a ser conectado naquela linha,
//! > contudo, sem quebrar a cadeia.»*
//!
//! ⚠️⚠️ **As duas perguntas usam ALVOS DIFERENTES de propósito, e isso é medido e não estético:**
//!
//! | leitura | o que se testa | porquê |
//! |---|---|---|
//! | outro NÓ | o **CURSOR** dentro da carta dele | duas cartas sobrepostas d(ão) uma resposta ambígua; o cursor dá uma só, e é onde o artista está a olhar |
//! | um FIO | a **CARTA arrastada** a atravessar o fio | é o gesto do Blender (*«arraste o nó sobre a ligação»*), e a carta é muito maior que o ponto — pedir que o cursor caia sobre a linha de um bezier de 2 px é pedir pontaria |
//!
//! ⛔ **E o nó ganha do fio**, sempre: uma carta largada sobre outra tem quase sempre um fio a
//! passar por baixo, e a leitura mais ESPECÍFICA é a que o artista apontou. *Sem a ordem, o gesto
//! de trocar dois nós vizinhos seria um splice aleatório no fio que os liga.*
//!
//! ⚠️ **Só nós SIMPLES**: um cartão dobrado (`Subgraph`) e um fantasma não entram nem como sujeito
//! nem como alvo — o primeiro não tem uma cadeia própria para trocar (as portas dele são
//! derivadas dos membros) e o segundo **não vive neste nível**.

use super::{View, geom};
use crate::snapshot::{GraphIntent, GraphViewSnapshot, NodeViewKind};

/// O que a largada de um nó quer dizer. `None` (a ausência) é a largada de sempre: mover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Largada {
    /// Sobre outro nó — os dois trocam de lugar na cadeia.
    Troca(u32),
    /// Sobre um fio, nomeado pela ponta de CHEGADA (uma entrada recebe uma fonte só).
    Fio(u32, u16),
}

/// Ver o cabeçalho do módulo. `cursor` é em píxeis de ecrã, como todo o resto deste painel.
pub(super) fn largada(
    snap: &GraphViewSnapshot,
    view: &View,
    arrastado: u32,
    cursor: (f32, f32),
) -> Option<Largada> {
    let eu = snap
        .nodes
        .iter()
        .find(|n| n.id == arrastado && n.kind == NodeViewKind::Node)?;
    // (1) O CURSOR sobre a carta de outro nó — a leitura mais específica.
    if let Some(outro) = snap.nodes.iter().find(|n| {
        n.id != arrastado
            && n.kind == NodeViewKind::Node
            && geom::card_rect(n, view).contains(cursor.0, cursor.1)
    }) {
        return Some(Largada::Troca(outro.id));
    }
    // (2) Um fio a atravessar a CARTA arrastada. ⭐ A travessia é a lei que a faca já usa
    // (`wires_crossed`, um segmento de cada vez) aplicada às quatro ARESTAS da carta — *uma
    // segunda prova de cruzamento escrita aqui divergiria daquela no dia em que o bezier mudasse*.
    let r = geom::card_rect(eu, view);
    let cantos = [
        (r.x, r.y),
        (r.x + r.w, r.y),
        (r.x + r.w, r.y + r.h),
        (r.x, r.y + r.h),
    ];
    let mut candidatos: Vec<(u32, u16)> = Vec::new();
    for i in 0..4 {
        for w in crate::paint::wires_crossed(snap, view, cantos[i], cantos[(i + 1) % 4]) {
            if !candidatos.contains(&w) {
                candidatos.push(w);
            }
        }
    }
    // ⛔ **Os fios do PRÓPRIO nó não contam** — enfiá-lo num fio que já sai dele é um laço, e a
    // carta arrastada está sempre por cima deles.
    candidatos.retain(|&(tn, tp)| {
        tn != arrastado
            && snap
                .edges
                .iter()
                .any(|e| e.to_node == tn && e.to_port == tp && e.from_node != arrastado)
    });
    // ⚠️ **O mais PERTO do centro da carta**, e não o primeiro da lista: com duas linhas a
    // atravessar a mesma carta, *«a primeira»* é uma propriedade da ordem das arestas no
    // documento, que o artista não vê.
    let centro = (r.x + 0.5 * r.w, r.y + 0.5 * r.h);
    candidatos
        .into_iter()
        .filter_map(|(tn, tp)| {
            let e = snap
                .edges
                .iter()
                .find(|e| e.to_node == tn && e.to_port == tp && !e.delayed)?;
            let (p0, p3) = crate::paint::wire_endpoints(snap, e, view)?;
            Some((dist_ao_segmento(centro, p0, p3), Largada::Fio(tn, tp)))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, l)| l)
}

/// A distância de um ponto ao segmento `a→b` — a régua do desempate acima. ⚠️ Ela mede a CORDA
/// do bezier e não a curva: para escolher entre dois fios que já se sabe atravessarem a mesma
/// carta, a diferença é menor do que a largura da carta.
fn dist_ao_segmento(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let len2 = vx * vx + vy * vy;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0)
    };
    (p.0 - (a.0 + t * vx)).hypot(p.1 - (a.1 + t * vy))
}

/// **O PEDIDO que a largada de um nó produz** — a tradução de [`Largada`] em intenção, do lado
/// de quem sabe o que cada uma quer dizer.
///
/// ⛔ **Só é chamada com UM nó arrastado.** Com uma selecção de três largada sobre um quarto nó,
/// *qual* troca com ele não tem resposta — e inventar uma (o primeiro da lista, o mais perto)
/// faria o mesmo gesto dar resultados diferentes conforme a ordem de selecção, que o artista não
/// vê. *Uma ambiguidade resolvida por sorteio é pior do que um gesto que não dispara.*
pub(super) fn pedir(
    snap: &GraphViewSnapshot,
    view: &View,
    arrastado: u32,
    cursor: (f32, f32),
    movido: (f32, f32),
) {
    match largada(snap, view, arrastado, cursor) {
        Some(Largada::Troca(outro)) => super::push_intent(GraphIntent::SwapInChain {
            a: arrastado,
            b: outro,
            back_dx: movido.0,
            back_dy: movido.1,
        }),
        Some(Largada::Fio(to_node, to_port)) => {
            super::push_intent(GraphIntent::SpliceExistingIntoWire {
                node: arrastado,
                to_node,
                to_port,
            });
        }
        None => {}
    }
}

#[cfg(test)]
#[path = "interact_node_drop_tests.rs"]
mod tests;
