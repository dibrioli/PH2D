//! **Os intervalos de RECORTE deste frame**, resolvidos contra a pilha de z que se vai desenhar.
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `frame_spans.rs`.

use ph2d_ecs::scene::HierarchySnapshot;
use ph2d_ecs::{Entity, SimWorld, VecClipContent};
use ph2d_vec_scene::{VecClipSpan, VecPathId};

/// Os intervalos que a árvore dita, **de fora para dentro** e **bem-aninhados**.
///
/// `order` é a pilha final (fundo → topo), a mesma que o `reorder_to` acabou de escrever.
///
/// Uma moldura sem descendente VETORIAL depois dela não produz intervalo: não há o que recortar, e
/// um intervalo vazio faria a camada abrir e fechar em cima de si mesma.
#[must_use]
pub fn clip_spans(
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
        // ⚠️ **A pergunta é o RECORTE, não a moldura** (2026-08-21): qualquer forma fechada com
        // `VecClipContent` abre intervalo, seja ela contêiner ou não. Este sítio nunca precisou
        // saber o que a silhueta é — o renderer já recortava por um caminho arbitrário, com a
        // regra de preenchimento dele.
        if w.get::<VecClipContent>(Entity::from_bits(e.entity))
            .is_none()
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
#[path = "frame_spans_tests.rs"]
mod tests;
