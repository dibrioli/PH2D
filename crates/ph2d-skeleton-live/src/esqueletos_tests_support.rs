//! ⚠️ **A fixtura das CADEIAS, num sítio só** — ela atravessa dois módulos de teste (a topologia e
//! a recusa do bind), e uma cópia por ficheiro divergiria no primeiro ajuste.

use ph2d_ecs::{Entity, SimWorld};

/// Monta `n` cadeias independentes de três ossos cada, afastadas umas das outras.
pub(crate) fn cadeias(sim: &mut SimWorld, n: usize) -> Vec<Entity> {
    let mut raizes = Vec::new();
    for c in 0..n {
        let base = 10.0 * c as f64;
        let mut pai = None;
        for k in 0..3 {
            let x = base + f64::from(k);
            let osso =
                crate::bone::create(sim, pai, [x, 0.0], [x + 1.0, 0.0]).expect("o osso nasce");
            let e = Entity::from_bits(osso);
            if k == 0 {
                raizes.push(e);
            }
            pai = Some(e);
        }
    }
    raizes
}
