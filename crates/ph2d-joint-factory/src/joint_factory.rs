//! **As peças de fábrica de um gabarito** (plano `docs/JointEditor/01_plano.md` §W4, «Make the parts»):
//! objetos reais, nomeados sem colidir, cada um com a forma de colisão filha (ADR-0199) e uma aparência
//! simples. A porta única: a cena 123 e o botão da aba fazem as peças por aqui.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_joint_templates::{Guide, Link, Role, Template};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody, nasce_com_forma};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// A cor de uma peça que se move, de uma peça fixa (o poste, o trilho) e de um disco (a roda).
pub const PART: [f32; 4] = [0.95, 0.62, 0.25, 1.0];
pub const SCENERY: [f32; 4] = [0.62, 0.62, 0.68, 1.0];
pub const TYRE: [f32; 4] = [0.22, 0.22, 0.26, 1.0];

/// **O tecto de peças numa fila** (as tábuas da ponte): o RECURSO é o quadro INTEIRO — o tique da física de UMA
/// ponte mais o desenho cabem em 16,7 ms com folga; senão o passo fixo deve tiques e paga até 8 por quadro (a
/// espiral: o app pára). Tique medido 16/11 (perfil smoke, esta estação, trechos de `SEGMENT_LINKS` elos):
/// 64 tábuas 3,6 ms · **128 7,5** · 192 11,8 · 256 15,8 · 512 32,4. No app em Play (Xvfb, quadro-base 32 ms):
/// **128 → quadro 34–37 ms** (a base) · 256 → **105 ms, ~10 fps, simulação com pico 96 ms** (o travamento do
/// smoke do dono 16/11). Tabela inteira: `docs/JointEditor/02_gabaritos_medidos.md`.
pub const MAX_ROW_PARTS: u16 = 128;

/// Quantas peças o papel `i` leva: `counts[i]` se maior que zero (até [`MAX_ROW_PARTS`]), senão as da fábrica.
#[must_use]
pub fn count_of(role: &Role, counts: &[u16], i: usize) -> u16 {
    let factory = role.many.map_or(1, |m| m.count);
    match (role.many, counts.get(i).copied()) {
        (Some(_), Some(n)) if n > 0 => n.min(MAX_ROW_PARTS),
        _ => factory,
    }
}

/// **Quanto o papel `i` anda** com `counts[i]` peças por papel: um papel preso à ÚLTIMA de uma fila (o poste
/// da direita da ponte) anda com ela — com mais tábuas que as da fábrica, afasta-se o passo de cada uma a mais.
#[must_use]
pub fn shift(t: &Template, counts: &[u16], i: usize) -> [f32; 2] {
    let mut d = [0.0, 0.0];
    for r in t
        .joints
        .iter()
        .filter(|r| r.link == Link::ToLast && usize::from(r.a) == i)
    {
        let b = usize::from(r.b);
        if let Some((row, m)) = t.roles.get(b).and_then(|rb| rb.many.map(|m| (rb, m))) {
            let extra = f32::from(count_of(row, counts, b)) - f32::from(m.count);
            d = [d[0] + m.step[0] * extra, d[1] + m.step[1] * extra];
        }
    }
    d
}

/// **O centro da `n`-ésima peça do papel `i`**, com o `(0, 0)` do desenho em `origin` (metros).
/// A ponte já não depende da ordem da conta: a emenda das correntes é mola amortecida (`chains.rs`,
/// gate `the_bridge_catches_a_dropped_crate_without_throwing_it`).
#[must_use]
pub fn centre_at(t: &Template, counts: &[u16], i: usize, n: u16, origin: [f32; 2]) -> [f32; 2] {
    let Some(role) = t.roles.get(i) else {
        return origin;
    };
    let (step, sh, k) = (
        role.many.map_or([0.0, 0.0], |m| m.step),
        shift(t, counts, i),
        f32::from(n),
    );
    [
        origin[0] + role.at[0] + sh[0] + step[0] * k,
        origin[1] + role.at[1] + sh[1] + step[1] * k,
    ]
}

/// O centro no desenho do gabarito (`y = 0` = o chão).
#[must_use]
pub fn centre(t: &Template, counts: &[u16], i: usize, n: u16) -> [f32; 2] {
    centre_at(t, counts, i, n, [0.0, 0.0])
}

/// O centro da caixa que envolve todas as peças, no desenho do gabarito (para pô-las no meio da vista).
#[must_use]
pub fn middle(t: &Template, counts: &[u16]) -> [f32; 2] {
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for (i, r) in t.roles.iter().enumerate() {
        let h = r.guide.half();
        for n in 0..count_of(r, counts, i) {
            let c = centre(t, counts, i, n);
            for k in 0..2 {
                lo[k] = lo[k].min(c[k] - h[k]);
                hi[k] = hi[k].max(c[k] + h[k]);
            }
        }
    }
    if lo[0] > hi[0] {
        return [0.0, 0.0];
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

/// Uma peça com a forma de colisão filha e a aparência do mesmo tamanho; atrito 1 (as rodas agarram).
pub fn block(
    sim: &mut SimWorld,
    name: &str,
    guide: Guide,
    rgba: Option<[f32; 4]>,
    at: [f32; 2],
    kind: BodyKind,
) -> Entity {
    let (shape, size, tint) = match guide {
        Guide::Box { half } => (
            ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            [half[0] * 2.0, half[1] * 2.0],
            if kind == BodyKind::Static {
                SCENERY
            } else {
                PART
            },
        ),
        Guide::Disc { radius } => (ColliderShape::Ball { radius }, [radius * 2.0; 2], TYRE),
    };
    let name = ph2d_unique_name::unique_name(sim, name);
    let w = sim.world_mut();
    let e = w
        .spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, size, rgba.unwrap_or(tint)),
            Name::new(name),
            RigidBody { kind },
        ))
        .id();
    let f = nasce_com_forma(w, e, shape);
    if let Some(mut c) = w.get_mut::<Collider>(f) {
        c.friction = 1.0;
    }
    ph2d_ecs::assign_missing_stable_ids(w);
    ph2d_ecs::assign_missing_root_order(w);
    e
}

/// O sufixo que deixa TODOS os nomes da máquina livres de uma vez («», « (1)», « (2)»…): as peças de uma
/// segunda ponte chamam-se todas «… (1)», nunca umas sim e outras não.
fn free_suffix(sim: &mut SimWorld, t: &Template, counts: &[u16], world_roles: bool) -> String {
    let names: Vec<String> = t
        .roles
        .iter()
        .enumerate()
        .filter(|(_, r)| !(r.may_be_world && world_roles))
        .flat_map(|(i, r)| {
            let ri = u16::try_from(i).unwrap_or(u16::MAX);
            (0..count_of(r, counts, i)).map(move |n| t.part_name(ri, n))
        })
        .collect();
    let used: std::collections::BTreeSet<String> = {
        let w = sim.world_mut();
        w.query::<&Name>()
            .iter(w)
            .map(|n| n.as_str().to_owned())
            .collect()
    };
    (0_u32..)
        .map(|k| {
            if k == 0 {
                String::new()
            } else {
                format!(" ({k})")
            }
        })
        .find(|s| names.iter().all(|n| !used.contains(&format!("{n}{s}"))))
        .unwrap_or_default()
}

/// **Faz as peças de fábrica** de `t`, com o (0, 0) do desenho em `origin` (mundo, metros) e `counts[i]`
/// peças nos papéis de várias. Devolve `(papel, peça)` na ordem de [`ph2d_joint_assembly::joint_assembly::assemble`];
/// `None` = o papel fica por conta do cenário (`world_roles` e o papel admite-o).
pub fn make_parts(
    sim: &mut SimWorld,
    t: &Template,
    origin: [f32; 2],
    counts: &[u16],
    world_roles: bool,
) -> Vec<(u16, Option<Entity>)> {
    let suffix = free_suffix(sim, t, counts, world_roles);
    let mut out = Vec::new();
    for (i, role) in t.roles.iter().enumerate() {
        let r = u16::try_from(i).unwrap_or(u16::MAX);
        if role.may_be_world && world_roles {
            out.push((r, None));
            continue;
        }
        let kind = if role.fixed {
            BodyKind::Static
        } else {
            BodyKind::Dynamic
        };
        for n in 0..count_of(role, counts, i) {
            let at = centre_at(t, counts, i, n, origin);
            let name = format!("{}{suffix}", t.part_name(r, n));
            out.push((r, Some(block(sim, &name, role.guide, None, at, kind))));
        }
    }
    out
}
