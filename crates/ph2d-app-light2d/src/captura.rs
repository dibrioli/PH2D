//! **Os sprites da cena para a C0** (W3) — das MESMAS instâncias que o passe de sprite desenha (o
//! `PresentWorld`, casadas pelo `SimRef` com o papel que a coleta deu a cada sprite): o mesmo quad, o
//! mesmo UV, a mesma textura. ⇒ a sombra é a silhueta que o artista vê.
//!
//! Fica com o QUAD (a `CenaDeLuz` já o tem, e o gabarito vê-o): uma malha (a pele deforma o quad), um
//! UV com ladrilho/rolagem, ou uma textura cozida (comprimida). Ditos no handoff W3.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use ph2d_ecs::{PresentWorld, SimRef};
use ph2d_light2d_ecs::{CenaDeLuz, FormaDoSprite, SpriteDeLuz};
use ph2d_light2d_gpu::{Amostragem, CapturaDeSprites, Quem, SpriteNaC0};
use ph2d_render::{RenderInstance, SpriteMesh, SpriteRenderer};

/// O quad de uma instância no mundo: `mundo(s, t) = origem + s·eixo_s + t·eixo_t`, com `(0, 0)` em
/// cima à esquerda (`quad_pos = (s − ½, ½ − t)`, a lei do `QuadVertex::QUAD_STRIP`).
fn quad_no_mundo(i: &RenderInstance) -> ([f64; 2], [f64; 2], [f64; 2]) {
    let b = i.basis.map(f64::from);
    let base = |l: [f64; 2]| [b[0] * l[0] + b[2] * l[1], b[1] * l[0] + b[3] * l[1]];
    let (sx, sy) = (f64::from(i.size[0]), f64::from(i.size[1]));
    let canto = base([
        f64::from(i.anchor[0]) - 0.5 * sx,
        f64::from(i.anchor[1]) + 0.5 * sy,
    ]);
    (
        [
            f64::from(i.world_pos[0]) + canto[0],
            f64::from(i.world_pos[1]) + canto[1],
        ],
        base([sx, 0.0]),
        base([0.0, -sy]),
    )
}

/// Os sprites da cena com textura amostrável, prontos para a C0.
pub(crate) fn sprites_da_cena<'r>(
    cena: &CenaDeLuz,
    present: &mut PresentWorld,
    renderer: &'r SpriteRenderer,
) -> CapturaDeSprites<'r> {
    let mut c = CapturaDeSprites::default();
    if cena.sprites.is_empty() {
        return c;
    }
    let papeis: BTreeMap<Entity, &SpriteDeLuz> =
        cena.sprites.iter().map(|s| (s.entidade, s)).collect();
    let mut por_textura: BTreeMap<u32, usize> = BTreeMap::new();
    let mut q = present
        .world_mut()
        .query::<(&RenderInstance, &SimRef, Option<&SpriteMesh>)>();
    for (inst, sim, malha) in q.iter(present.world()) {
        let Some(papel) = papeis.get(&sim.0) else {
            continue;
        };
        if malha.is_some() || inst.uv_xform != RenderInstance::IDENTITY_UV_XFORM {
            continue;
        }
        let Some(vista) = renderer.vista_da_textura(inst.texture_id) else {
            continue;
        };
        let textura = *por_textura.entry(inst.texture_id).or_insert_with(|| {
            c.texturas.push((vista, Amostragem::Bilinear));
            c.texturas.len() - 1
        });
        let [mut u0, mut v0, mut u1, mut v1] = inst.atlas_uv;
        if inst.flip_uv & RenderInstance::FLIP_X_BIT != 0 {
            std::mem::swap(&mut u0, &mut u1);
        }
        if inst.flip_uv & RenderInstance::FLIP_Y_BIT != 0 {
            std::mem::swap(&mut v0, &mut v1);
        }
        let (origem, eixo_s, eixo_t) = quad_no_mundo(inst);
        c.sprites.push(SpriteNaC0 {
            origem,
            eixo_s,
            eixo_t,
            uv: [u0, v0, u1, v1],
            emissao: papel.emissao,
            opacidade: papel.opacidade,
            premultiplicada: inst.premultiplied > 0.5,
            textura,
            albedo: papel.albedo,
            cor_em_srgb: false,
        });
        c.substituidos.insert(match papel.forma {
            FormaDoSprite::Emissor(i) => Quem::Emissor(i),
            FormaDoSprite::Oclusor(i) => Quem::Oclusor(i),
        });
    }
    c
}
