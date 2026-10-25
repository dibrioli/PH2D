//! **Quem veste o resultado de uma booleana** (F29) — irmão do `lib.rs` pelo tecto de LOC.

use crate::BoolOp;
use ph2d_vec_scene::VecPath;

/// ⭐⭐⭐ **QUEM VESTE O RESULTADO** — o operando de CIMA que não corta e que tem preenchimento
/// visível; sem nenhum visível, o de cima que não corta.
///
/// - ⛔ **O cortador não doa** (verbo `Subtract`): o resultado é o que sobra do de trás, e o
///   Illustrator dá ao *Minus Front* o estilo do de trás. A base não tem verbo e conta como quem
///   não corta.
/// - ⛔ **Um transparente não doa** (report do dono 25/10, F29: *«operação booleana onde um objeto
///   é transparente fica com preenchimento bugado até o usuário mudar o fill»*): o de cima sem
///   preenchimento vestia o resultado de nada.
/// - O doador dá fill **e** stroke: o resultado veste o visual de UM objecto.
pub(crate) fn doador<'a>(base: &'a VecPath, folds: &[(&'a VecPath, BoolOp)]) -> &'a VecPath {
    let mut quem = folds
        .iter()
        .rev()
        .filter(|(_, op)| *op != BoolOp::Subtract)
        .map(|(p, _)| *p)
        .chain(std::iter::once(base));
    let topo = quem.clone().next().unwrap_or(base);
    quem.find(|p| p.fill.as_ref().is_some_and(visivel))
        .unwrap_or(topo)
}

/// A tinta pinta alguma coisa? (alfa 0 em todos os pontos = transparente.)
fn visivel(paint: &ph2d_vec_scene::Paint) -> bool {
    use ph2d_vec_scene::Paint;
    match paint {
        Paint::Solid(c) => c.a > 0,
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            stops.iter().any(|s| s.color.a > 0)
        }
        Paint::MultiPoint { points } => points.iter().any(|p| p.color.a > 0),
        Paint::Pattern(_) => true,
    }
}
