//! **Os PAIS deste frame**, resolvidos para intervalos da pilha de z.
//!
//! A metade que a shell possui: ela sabe a ÁRVORE (o parentesco e o `ph2d_ecs::VecFrame` moram em
//! entidades) e o renderer sabe DESENHAR, então o que atravessa a fronteira é um par de ids por pai
//! — `ph2d_vec_scene::VecParentSpan`, dentro do `VecViewState` que já leva *"o que a árvore diz
//! sobre os paths"*.
//!
//! # A LEI: o filho desenha SOBRE o pai
//!
//! É a lei do Godot, e a mesma do Figma (o preenchimento de uma moldura é o FUNDO do card) e do
//! Illustrator (um grupo não cobre o próprio conteúdo). Enio, 2026-08-04: *"o filho é desenhado por
//! trás do pai e fica sobreposto e não visível … em game engines como Godot os filhos são
//! renderizados acima dos pais"*.
//!
//! ⚠️ **A lei é imposta AQUI e não no `z_order`.** A pilha de z é o DFS invertido, então ela põe o
//! pai NA FRENTE dos filhos; a cura óbvia — pôr o contêiner no fundo da própria sub-árvore — já foi
//! **tentada e reprovada**: é ele o ÚLTIMO membro dela que emparelha o `push_clip` da abertura com
//! o `pop_layer` da vez dele, e sem isso o `pop` fecha a camada de outra pessoa e some com arte
//! alheia. Então o que se antecipa é o **DESENHO** do pai, não o lugar dele na pilha.
//!
//! # Isto já existia — só que para MOLDURAS
//!
//! A mesma queixa chegou em 2026-08-02 (*"os filhos estão ficando atrás do pai, logo não podem ser
//! vistos a menos que reduza a opacidade"*) e foi fechada apenas para quem tinha `VecFrame`. A
//! premissa de então — *"invisível para um grupo (sem geometria), fatal para uma moldura"* — está
//! **errada**: um pai comum tem geometria como qualquer outro caminho, e ele cobria os filhos
//! exactamente do mesmo jeito. A condição some; o `clip` continua a ser pergunta de moldura.
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
use ph2d_vec_scene::{VecParentSpan, VecPathId};

/// Os intervalos que a árvore dita, **de fora para dentro**.
///
/// A ordem sai de graça e é load-bearing: o laço anda o DFS de cima para baixo, e no DFS um pai
/// aninhado vem depois do que o contém. Dois pais aninhados podem abrir no MESMO path (o
/// descendente mais ao fundo é comum), e a camada de clip é uma pilha — abrir a de dentro primeiro
/// fecharia na ordem errada.
///
/// Um pai sem descendente VETORIAL não produz intervalo: não há o que antecipar (nem o que
/// recortar), e um intervalo vazio faria a forma abrir e fechar em cima de si mesma.
#[must_use]
pub(crate) fn parent_spans(sim: &SimWorld, snap: &HierarchySnapshot) -> Vec<VecParentSpan> {
    let w = sim.world();
    let mut out = Vec::new();
    for (i, e) in snap.entries.iter().enumerate() {
        let Some(parent_path) = e.vec_path else {
            continue;
        };
        // ⚠️ **TODO pai com conteúdo tem intervalo** — e o `clip` só decide se ele também abre
        // camada. O intervalo é, antes de mais nada, *onde o desenho dele é antecipado*: a pilha
        // de z põe o pai NA FRENTE dos filhos (ela é o DFS invertido), então sem intervalo ele
        // pinta por cima do próprio conteúdo. Enquanto isto era gateado em `VecFrame`, qualquer
        // forma com uma forma filha fazia exactamente isso.
        let clip = w
            .get::<VecFrame>(Entity::from_bits(e.entity))
            .is_some_and(|f| f.clip);
        if let Some(first) = bottom_descendant(snap, i, e.depth) {
            out.push(VecParentSpan {
                parent: parent_path,
                first,
                clip,
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
