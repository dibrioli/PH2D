//! ⭐⭐ **O comprimento EMPURRA os filhos** (dono 30/10, como o Blender/Spine): quando um osso vai de
//! `l0` a `l1`, cada filho DIRECTO (osso ou qualquer coisa pendurada nele) anda ao longo do eixo do pai
//! — `translation.x *= l1 / l0`, o `y` fica. Um filho na ponta fica na ponta; no meio, no meio. Os
//! netos seguem pela hierarquia. Escritores: a mão e a curva (`ph2d_app_skeleton::comprimento`).

use ph2d_ecs::{Children, Entity, SimWorld, Transform};

/// Tolerância relativa do «está na ponta»: o filho que lá estava fica EXACTAMENTE na ponta nova (sem
/// isto o `f32` deriva quadro a quadro sob uma curva).
const NA_PONTA: f32 = 1e-5;

/// ⭐ **Empurra os filhos directos de `osso`** de `l0` para `l1`. Devolve `(filho, antes, depois)` de
/// quem mudou — a curva declara-os à pré-visualização. `l0 <= 0` (sem eixo de onde medir) ⇒ nada.
pub fn empurra_filhos(
    sim: &mut SimWorld,
    osso: Entity,
    l0: f64,
    l1: f64,
) -> Vec<(Entity, Transform, Transform)> {
    if !(l0 > 0.0) || l0 == l1 {
        return Vec::new();
    }
    let filhos: Vec<Entity> = (sim.world().get::<Children>(osso))
        .map(|f| f.iter().copied().collect())
        .unwrap_or_default();
    #[expect(clippy::cast_possible_truncation, reason = "o Transform é f32")]
    let (r, p0, p1) = ((l1 / l0) as f32, l0 as f32, l1 as f32);
    let mut mudaram = Vec::with_capacity(filhos.len());
    for f in filhos {
        let Some(mut t) = sim.world_mut().get_mut::<Transform>(f) else {
            continue;
        };
        let antes = *t;
        let x = antes.translation.x;
        t.translation.x = if (x - p0).abs() <= NA_PONTA * p0.max(1.0) {
            p1
        } else {
            x * r
        };
        let depois = *t;
        if depois != antes {
            mudaram.push((f, antes, depois));
        }
    }
    mudaram
}
