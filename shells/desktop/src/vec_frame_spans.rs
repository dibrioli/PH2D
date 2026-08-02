//! **As molduras deste frame**, resolvidas para intervalos da pilha de z.
//!
//! A metade da moldura que a shell possui: ela sabe a ÁRVORE (o `ph2d_ecs::VecFrame` mora numa
//! entidade) e o renderer sabe DESENHAR, então o que atravessa a fronteira é um par de ids por
//! moldura — `ph2d_vec_scene::VecClipSpan`, dentro do `VecViewState` que já leva *"o que a árvore
//! diz sobre os paths"*.
//!
//! # Por que o snapshot da hierarquia, e não uma varredura da cena
//!
//! É a MESMA estrutura que `vec_zorder::z_order` consome para produzir a pilha, no MESMO ponto do
//! frame (depois do `sync`, depois do `reorder_to`). Derivar o intervalo de outra fonte seria uma
//! segunda resposta a *"em que ordem estas formas estão?"*, e a resposta divergente apareceria
//! como recorte deslocado por uma forma — o tipo de defeito que ninguém liga à ordem de z.
//!
//! # A inversão que decide tudo
//!
//! ⚠️ O DFS lista **pai antes dos filhos** e a pilha de z é o **inverso** dele
//! (`z_order`: `entries … .rev()`). Logo o descendente que aparece por ÚLTIMO na sub-árvore do DFS
//! é o que desenha por PRIMEIRO — é ele o `first` do intervalo. Ler isto ao contrário produz um
//! recorte que abre no lugar errado e some com quase toda a arte da moldura, então há gate.

use ph2d_ecs::scene::HierarchySnapshot;
use ph2d_ecs::{Entity, SimWorld, VecFrame};
use ph2d_vec_scene::{VecClipSpan, VecPathId};

/// Os intervalos de recorte que a árvore dita, **de fora para dentro**.
///
/// A ordem sai de graça e é load-bearing: o laço anda o DFS de cima para baixo, e no DFS uma
/// moldura aninhada vem depois da que a contém. Duas molduras aninhadas podem abrir no MESMO path
/// (o descendente mais ao fundo é comum), e a camada de clip é uma pilha — abrir a de dentro
/// primeiro fecharia na ordem errada.
///
/// Uma moldura sem descendente VETORIAL não produz intervalo: não há o que recortar, e um
/// intervalo vazio faria a moldura abrir e fechar em cima de si mesma.
#[must_use]
pub(crate) fn clip_spans(sim: &SimWorld, snap: &HierarchySnapshot) -> Vec<VecClipSpan> {
    let w = sim.world();
    let mut out = Vec::new();
    for (i, e) in snap.entries.iter().enumerate() {
        let Some(frame_path) = e.vec_path else {
            continue;
        };
        let entity = Entity::from_bits(e.entity);
        // Só molduras que RECORTAM. Uma moldura com `clip` desligado continua sendo uma moldura
        // (dona de um tamanho autorado); ela só não abre camada.
        if !w.get::<VecFrame>(entity).is_some_and(|f| f.clip) {
            continue;
        }
        if let Some(first) = bottom_descendant(snap, i, e.depth) {
            out.push(VecClipSpan {
                frame: frame_path,
                first,
            });
        }
    }
    out
}

/// O descendente de `entries[i]` que desenha por PRIMEIRO (o mais ao fundo em z) — ou seja, o
/// ÚLTIMO com geometria vetorial dentro da sub-árvore, na ordem do DFS.
fn bottom_descendant(snap: &HierarchySnapshot, i: usize, depth: u8) -> Option<VecPathId> {
    let mut found = None;
    for e in &snap.entries[i + 1..] {
        // A sub-árvore acaba na primeira entrada que não é mais funda que a raiz dela.
        if e.depth <= depth {
            break;
        }
        if let Some(p) = e.vec_path {
            found = Some(p);
        }
    }
    found
}

#[cfg(test)]
#[path = "vec_frame_spans_tests.rs"]
mod tests;
