//! A SILHUETA de um texel do dab: o peso `w` (falloff × Shape × película × cápsula), o maior sub-dab e a cor da
//! cápsula sob uma rampa do Shape. Fase 1 de [`super::stamp_band`].

use super::{DabCtx, Invariantes};
use crate::dab::cor_capsula::CorCapsula;

pub(in crate::dab) struct Silhueta {
    pub(in crate::dab) w: f32,
    /// O maior sub-dab da cápsula (a orla em píxeis e o Wash param nele); `0` sem cápsula.
    pub(in crate::dab) maior_sub: f32,
    /// A cor composta sobre os sub-dabs ([`crate::dab::cor_capsula`]); `None` ⇒ lida no texel.
    pub(in crate::dab) cor_cap: Option<CorCapsula>,
}

#[inline(always)]
pub(in crate::dab) fn em(
    ctx: &DabCtx,
    inv: &Invariantes,
    px: i64,
    py: i64,
    dx: f32,
    dy: f32,
) -> Silhueta {
    // `w` is the DEFORMED point and `t` its length — byte-identical to `falloff_t`, which is
    // exactly `apply` then the same `sqrt`. The vector itself is what the LUT's expansion
    // needs (the metric is euclidean in that space), so it is kept instead of thrown away.
    let wv = ctx
        .footprint
        .apply([dx * ctx.inv_radius, dy * ctx.inv_radius]);
    let t = (wv[0] * wv[0] + wv[1] * wv[1]).sqrt();
    // A ponta RETA (`dab_quina`, J3 A-4): a norma do `falloff_t`, o quadrado no referencial — sem quina
    // o `sqrt` de cima, ao bit (`basicos::a_ponta_reta_do_digital_pinta_um_rectangulo`).
    let t = if ctx.footprint.quina() > 0.0 {
        ctx.footprint
            .falloff_t(dx * ctx.inv_radius, dy * ctx.inv_radius)
    } else {
        t
    };
    // The FILM ([`crate::height::film_coverage`]): a brush laying body lays no pigment where it
    // lays no body. Applied to the SILHOUETTE — before the Grain, before the dynamics — so every
    // funnel below (grain, Accumulate-OFF cap, ramps) inherits the cut with no arithmetic.
    // With Smooth Edges ([`crate::height_film::FilmAa`], BUGS #16) the film at a rim texel is
    // its fractional AREA coverage. A capsule composes its sub-dabs at [`Invariantes::c_cap`].
    let c_cap = inv.c_cap;
    let mut maior_sub = 0.0f32;
    let mut cor_cap = inv.cor_cap.then(|| CorCapsula::new(c_cap));
    let w = match (ctx.capsula, &ctx.film_aa) {
        (Some(cap), _) => {
            let uniao = crate::dab::silhouette_capsula_com(
                ctx.spec,
                inv.queda,
                ctx.shape,
                ctx.footprint,
                cap,
                ctx.lei,
                c_cap,
                px,
                py,
                ctx.center,
                ctx.radius,
                |s, r, x| {
                    maior_sub = maior_sub.max(x);
                    let Some(cc) = cor_cap.as_mut() else { return };
                    cc.mais(x, || {
                        let centro = [ctx.center[0] + s.off[0], ctx.center[1] + s.off[1]];
                        let forma = ctx.shape_cor.map(|sh| {
                            let base = sh.basis.sub(s.rotor, s.arco);
                            let sh = crate::dab::ShapeInput { basis: &base, ..sh };
                            crate::dab::shape_value_at(ctx.spec, sh, px, py, centro, r)
                        });
                        crate::ramp_alpha::ramp_texel(
                            ctx.ramp.as_ref(),
                            x,
                            forma,
                            None,
                            0.0,
                            0.0,
                            &crate::texture::GrainLaw::NEUTRA,
                        )
                    });
                },
            );
            if inv.orla_area { maior_sub } else { uniao }
        }
        (None, Some(aa)) => aa.film_at_planned(
            ctx.lut.as_ref(),
            t,
            wv,
            [dx, dy],
            || crate::dab::silhouette_at_com(ctx.spec, inv.queda, ctx.shape, t, px, py, ctx.center, ctx.radius),
            |ox, oy| {
                inv.queda.peso(
                    ctx.spec,
                    ctx.footprint
                        .falloff_t((dx + ox) * ctx.inv_radius, (dy + oy) * ctx.inv_radius),
                )
            },
        ),
        (None, None) => crate::height::film_coverage(
            ctx.spec.deposits_height(),
            crate::dab::silhouette_at_com(ctx.spec, inv.queda, ctx.shape, t, px, py, ctx.center, ctx.radius),
        ),
    };
    Silhueta {
        w,
        maior_sub,
        cor_cap,
    }
}
