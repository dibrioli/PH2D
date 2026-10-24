//! **Onde uma gravação posta no mundo cai nos texels de uma sprite** (W18, `docs/Macros/16_W18_plano.md`
//! §2.7): o encaixe vivo de uma macro aplicada por cima — o Painter toca por ele, a família MACROS
//! confere-o. Vive na FOLHA porque são duas famílias a partilhá-lo (HOWTO §1.2).
//!
//! ```text
//! px gravado ─(centrado na origem do objecto, 1 px = 1/ppm, y para cima)→ local do objecto
//!            ─(o mundo dele)→ mundo ─(o inverso do mundo da sprite)→ local da sprite
//!            ─(a lei do picking: anchor, size, v para baixo)→ texel do alvo
//! ```
//! `ppm` = os texels por unidade local da sprite-alvo (`W / size.x`): uma gravação encaixada na pose do
//! alvo cai texel a texel, sem esticar.

use ph2d_ecs::{Entity, GlobalTransform, World};

/// Afim 2×3 `[a, b, c, d, e, f]`: `x' = a·x + b·y + c`, `y' = d·x + e·y + f`.
pub type Afim = [f64; 6];

fn compoe(p: Afim, q: Afim) -> Afim {
    // p ∘ q
    [
        p[0] * q[0] + p[1] * q[3],
        p[0] * q[1] + p[1] * q[4],
        p[0] * q[2] + p[1] * q[5] + p[2],
        p[3] * q[0] + p[4] * q[3],
        p[3] * q[1] + p[4] * q[4],
        p[3] * q[2] + p[4] * q[5] + p[5],
    ]
}

fn inverso(p: Afim) -> Option<Afim> {
    let det = p[0] * p[4] - p[1] * p[3];
    if det.abs() < 1e-12 || !det.is_finite() {
        return None;
    }
    let (a, b, d, e) = (p[4] / det, -p[1] / det, -p[3] / det, p[0] / det);
    Some([a, b, -(a * p[2] + b * p[5]), d, e, -(d * p[2] + e * p[5])])
}

/// O mundo de `e` como afim (a MESMA base do `GlobalTransform` que o render e o picking usam).
fn mundo(world: &World, e: Entity) -> Option<Afim> {
    let t = ph2d_ecs::world_transform(world, e)?;
    let [a, b, c, d, tx, ty] = GlobalTransform::from_transform(t).affine().map(f64::from);
    Some([a, c, tx, b, d, ty])
}

/// **O afim px-gravado → texel do alvo** do objecto `objeto` sobre a sprite `alvo`, cuja tela tem
/// `texels` e a gravação `gravado`. `None` = falta uma pose, a sprite, ou uma base degenerada.
#[must_use]
pub fn gravacao_no_alvo(
    world: &World,
    objeto: Entity,
    alvo: Entity,
    texels: (u32, u32),
    gravado: (u32, u32),
) -> Option<Afim> {
    let s = world.get::<ph2d_render::Sprite>(alvo)?;
    let (sw, sh) = (f64::from(s.size[0]), f64::from(s.size[1]));
    let (w, h) = (f64::from(texels.0), f64::from(texels.1));
    if sw <= 0.0 || sh <= 0.0 || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (ax, ay) = (f64::from(s.anchor[0]), f64::from(s.anchor[1]));
    let ppm = w / sw;
    let (w0, h0) = (f64::from(gravado.0), f64::from(gravado.1));
    let gravacao = [
        1.0 / ppm,
        0.0,
        -w0 / (2.0 * ppm),
        0.0,
        -1.0 / ppm,
        h0 / (2.0 * ppm),
    ];
    let texel = [
        w / sw,
        0.0,
        (0.5 - ax / sw) * w,
        0.0,
        -h / sh,
        (0.5 + ay / sh) * h,
    ];
    let m = mundo(world, objeto)?;
    let t = inverso(mundo(world, alvo)?)?;
    let afim = compoe(texel, compoe(t, compoe(m, gravacao)));
    afim.iter().all(|x| x.is_finite()).then_some(afim)
}
