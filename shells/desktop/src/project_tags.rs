//! ⭐⭐ **A ÁRVORE DE TAGS a chegar à SESSÃO** — a porta ÚNICA por onde os bytes do `ProjectState`
//! viram a árvore viva do `AppGfx` (TOP-20 #9, `docs/Components/08_plano_tags.md` §2.2).
//!
//! # ⚠️ Porque é uma porta, e não duas linhas no sítio do restauro
//!
//! Restaurar a árvore e **reapontar a pertença dos gémeos fundidos** são duas metades do MESMO
//! gesto: um documento com dois caminhos iguais e ids diferentes (o Blender distribui dois no
//! ficheiro dele — medido) funde-os no menor id, e os objectos que carregavam o descartado ficam a
//! apontar para uma tag que a árvore já não tem. *A tag deles desapareceria em silêncio.*
//!
//! ⛔ Enquanto as duas metades eram duas linhas soltas dentro do restauro, **apagar a segunda não
//! reprovava nada**: o gate do undo usa os nossos próprios bytes, que nunca têm gémeos. Uma porta
//! com um gate seu é o que torna a metade esquecível impossível de esquecer.

use ph2d_ecs::World;
use ph2d_tags::TagTree;

/// **Os bytes viram árvore, e a pertença acompanha.** Devolve a árvore e quantos objectos foram
/// reapontados (`0` no caso normal — um documento nosso não tem gémeos).
pub(crate) fn apply(bytes: &[u8], world: &mut World) -> (TagTree, usize) {
    let (tree, remap) = ph2d_app_components::tags_doc::restore(bytes);
    let reapontados = ph2d_ecs::tags::remap(world, &remap);
    if reapontados > 0 {
        eprintln!("[proj] tags: {reapontados} objecto(s) reapontados para o gemeo que ficou");
    }
    (tree, reapontados)
}

#[cfg(test)]
#[path = "project_tags_tests.rs"]
mod tests;
