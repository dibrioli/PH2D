//! **Os intervalos de RECORTE deste frame**, resolvidos contra a pilha de z que se vai desenhar.
//!
//! A metade que a shell possui: ela sabe a ÁRVORE (o parentesco e o `ph2d_ecs::VecFrame` moram em
//! entidades) e o renderer sabe DESENHAR, então o que atravessa a fronteira é um par de ids por
//! moldura — `ph2d_vec_scene::VecClipSpan`, dentro do `VecViewState` que já leva *"o que a árvore
//! diz sobre os paths"*.
//!
//! # O que este módulo DEIXOU de produzir (2026-08-04)
//!
//! Ele produzia um intervalo para **todo pai**, recortasse ou não, porque a pilha de z era o DFS
//! **invertido** — o pai desenhava na FRENTE dos filhos e o renderer tinha de **antecipar** o
//! desenho dele. Enio: *"o filho é desenhado por trás do pai … em game engines como Godot os
//! filhos são renderizados acima dos pais"*.
//!
//! ⚠️ A cura não foi um terceiro remendo: foi **inverter a projeção** (`vec_zorder::z_order`). O
//! filho passou a desenhar sobre o pai em TODA rota — incluindo a INSTÂNCIA de componente, que
//! percorre a cena e não tem renderer por trás, e por isso não tinha como herdar a antecipação
//! (*"ao criar a instância, os filhos que no mestre aparecem na frente dos pais vão para trás"*).
//! Com a antecipação morta, o intervalo voltou a ser o que o nome diz: **recorte**, e só de quem
//! recorta.
//!
//! # O intervalo sai da pilha FINAL, e é isso que o Z global exige
//!
//! O Z sobrepõe a árvore, então a sub-árvore de uma moldura **deixou de ser contígua** na pilha:
//! um filho com Z alto sai de dentro do intervalo e não é recortado. Resolver o intervalo contra o
//! DFS o descreveria onde a forma já não está — e um intervalo que aponta para o path errado é um
//! `pop_layer` no lugar errado.

use ph2d_ecs::scene::HierarchySnapshot;
use ph2d_ecs::{Entity, SimWorld, VecFrame};
use ph2d_vec_scene::{VecClipSpan, VecPathId};

/// Os intervalos que a árvore dita, **de fora para dentro** e **bem-aninhados**.
///
/// `order` é a pilha final (fundo → topo), a mesma que o `reorder_to` acabou de escrever.
///
/// Uma moldura sem descendente VETORIAL depois dela não produz intervalo: não há o que recortar, e
/// um intervalo vazio faria a camada abrir e fechar em cima de si mesma.
#[must_use]
pub(crate) fn clip_spans(
    sim: &SimWorld,
    snap: &HierarchySnapshot,
    order: &[VecPathId],
) -> Vec<VecClipSpan> {
    let w = sim.world();
    let mut out = Vec::new();
    for (i, e) in snap.entries.iter().enumerate() {
        let Some(frame_path) = e.vec_path else {
            continue;
        };
        if !w
            .get::<VecFrame>(Entity::from_bits(e.entity))
            .is_some_and(|f| f.clip)
        {
            continue;
        }
        let Some(fi) = order.iter().position(|o| *o == frame_path) else {
            continue;
        };
        let kin: Vec<VecPathId> = descendants(snap, i, e.depth).collect();
        // ⚠️ **O intervalo vai até onde a sub-árvore é CONTÍGUA**, e a primeira coisa que não
        // descende da moldura o fecha. Duas propriedades saem daí, e as duas são load-bearing:
        //
        // 1. **Um estranho nunca é recortado.** O Z pode pousar uma forma alheia entre a moldura
        //    e os filhos dela; um intervalo que fosse até o ÚLTIMO descendente a engoliria, e
        //    arte de outra pessoa sumiria na borda de um card que não é dela.
        // 2. **A lista fica LAMINAR de graça.** A camada de clip é uma pilha, e dois intervalos
        //    que se cruzassem fariam um `pop_layer` fechar a camada de outrem. Uma corrida
        //    contígua de descendentes de uma moldura interna é, por construção, uma corrida
        //    contígua de descendentes da externa — então ou aninha, ou é disjunta.
        //
        // E o preço é exatamente o que *"o Z sobrepõe a hierarquia"* promete: o filho que o Z
        // tirou da corrida **deixa de ser recortado**.
        let mut li = fi;
        while order.get(li + 1).is_some_and(|p| kin.contains(p)) {
            li += 1;
        }
        if li > fi {
            out.push(VecClipSpan {
                frame: frame_path,
                last: order[li],
            });
        }
    }
    out
}

/// Os caminhos vetoriais sob `entries[i]`, na ordem do DFS.
fn descendants(
    snap: &HierarchySnapshot,
    i: usize,
    depth: u8,
) -> impl Iterator<Item = VecPathId> + '_ {
    snap.entries[i + 1..]
        .iter()
        // A sub-árvore acaba na primeira entrada que não é mais funda que a raiz dela.
        .take_while(move |e| e.depth > depth)
        .filter_map(|e| e.vec_path)
}

#[cfg(test)]
#[path = "vec_frame_spans_tests.rs"]
mod tests;
