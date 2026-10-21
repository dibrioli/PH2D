//! ⭐⭐ **A família OBJETO nos canais** (plano 35 §3.0) — o skew do `Transform` e a aparência da
//! sprite que o Inspector edita: [`publicar`] o que o documento vale em `CanaisLidos`, [`aplicar`]
//! o que a curva mandou em `CanaisEscritos` (com o eco), e declarar a escrita ao
//! [`PreviewDrive`]: o que a curva escreve a tocar é pré-visualização, o undo e o save fotografam o
//! autorado (o skew vai no `SolverPose` da pose; a cor no `SpriteCor`; o self tint e a silhueta no
//! `TweenTint` — o mesmo facto, outro motor).
//!
//! As escritas seguem a fronteira do commit do Inspector: o skew pelo `Transform::clamp_skew`, a
//! opacidade e as cores presas a `0..=1` (o domínio do selector), a emissão pelo
//! `SpriteEmissive::clamped` e **zero REMOVE o componente** (a ausência é o que o render lê como
//! «não emite», byte-idêntico); os cantos ausentes são branco.

use ph2d_ecs::canal::familia::OBJETO;
use ph2d_ecs::canal_objeto as o;
use ph2d_ecs::{
    CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, SpriteCornerTint, SpriteEmissive,
    Transform, World,
};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_render::Sprite;

const fn k(campo: u16, j: u16) -> FieldKey {
    FieldKey::new(OBJETO, campo, 0, j)
}

/// Os campos que a entidade tem: o skew de todo objecto com `Transform`, a aparência de toda
/// sprite.
#[must_use]
pub fn campos(w: &World, e: Entity) -> Vec<FieldKey> {
    let mut out = Vec::new();
    if w.get::<Transform>(e).is_some() {
        out.extend([k(o::SKEW_X, 0), k(o::SKEW_Y, 0)]);
    }
    if w.get::<Sprite>(e).is_some() {
        out.extend([k(o::EMISSAO, 0), k(o::OPACIDADE, 0), k(o::SILHUETA, 0)]);
        out.extend((0..3).map(|c| k(o::TINTA + c, 0)));
        out.extend((0..4).map(|c| k(o::AUTO_TINTA + c, 0)));
        out.extend((0..o::N_CANTOS).flat_map(|j| (0..4).map(move |c| k(o::CANTOS + c, j))));
    }
    out
}

fn canto(w: &World, e: Entity) -> SpriteCornerTint {
    w.get::<SpriteCornerTint>(e)
        .copied()
        .unwrap_or(SpriteCornerTint::IDENTITY)
}

/// Quanto vale `kk` no documento — `None` se a entidade não tem o campo.
#[must_use]
pub fn ler(w: &World, e: Entity, kk: FieldKey) -> Option<f32> {
    if kk.familia != OBJETO || kk.i != 0 {
        return None;
    }
    if let o::SKEW_X | o::SKEW_Y = kk.campo {
        let t = w.get::<Transform>(e)?;
        return (kk.j == 0).then_some(if kk.campo == o::SKEW_X {
            t.skew_x
        } else {
            t.skew_y
        });
    }
    let s = w.get::<Sprite>(e)?;
    let c = componente(kk.campo);
    match kk.campo {
        o::CANTOS..o::CANTOS_FIM => canto(w, e).0.get(usize::from(kk.j)).map(|cor| cor[c]),
        _ if kk.j != 0 => None,
        o::EMISSAO => Some(w.get::<SpriteEmissive>(e).map_or(0.0, |em| em.clamped())),
        o::OPACIDADE => Some(s.opacity),
        o::TINTA..o::TINTA_FIM => Some(s.tint[c]),
        o::AUTO_TINTA..o::AUTO_TINTA_FIM => Some(s.self_tint[c]),
        o::SILHUETA => Some(if s.tint_fill { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// O componente (r 0 · g 1 · b 2 · a 3) de um campo de um bloco de cor.
fn componente(campo: u16) -> usize {
    let base = match campo {
        o::TINTA..o::TINTA_FIM => o::TINTA,
        o::AUTO_TINTA..o::AUTO_TINTA_FIM => o::AUTO_TINTA,
        o::CANTOS..o::CANTOS_FIM => o::CANTOS,
        _ => campo,
    };
    usize::from(campo - base)
}

fn cor(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Escreve `v` em `kk`, pela fronteira do commit do Inspector (ver o cabeçalho). Só escreve o que
/// muda.
pub fn escrever(w: &mut World, e: Entity, kk: FieldKey, v: f32) {
    if ler(w, e, kk).is_none() {
        return;
    }
    if let o::SKEW_X | o::SKEW_Y = kk.campo {
        let v = Transform::clamp_skew(v);
        if let Some(mut t) = w.get_mut::<Transform>(e) {
            let alvo = if kk.campo == o::SKEW_X {
                &mut t.skew_x
            } else {
                &mut t.skew_y
            };
            if *alvo != v {
                *alvo = v;
            }
        }
        return;
    }
    let c = componente(kk.campo);
    match kk.campo {
        o::EMISSAO => {
            let v = SpriteEmissive(v).clamped();
            let ja = w.get::<SpriteEmissive>(e).map(|em| em.0);
            if v <= 0.0 {
                if ja.is_some() {
                    w.entity_mut(e).remove::<SpriteEmissive>();
                }
            } else if ja != Some(v) {
                w.entity_mut(e).insert(SpriteEmissive(v));
            }
        }
        o::CANTOS..o::CANTOS_FIM => {
            let mut ct = canto(w, e);
            let alvo = &mut ct.0[usize::from(kk.j)][c];
            if *alvo == cor(v) {
                return;
            }
            *alvo = cor(v);
            w.entity_mut(e).insert(ct);
        }
        _ => {
            let Some(mut s) = w.get_mut::<Sprite>(e) else {
                return;
            };
            let (alvo, novo) = match kk.campo {
                o::OPACIDADE => (&mut s.opacity, cor(v)),
                o::TINTA..o::TINTA_FIM => (&mut s.tint[c], cor(v)),
                o::AUTO_TINTA..o::AUTO_TINTA_FIM => (&mut s.self_tint[c], cor(v)),
                o::SILHUETA => {
                    let on = v >= 0.5;
                    if s.tint_fill != on {
                        s.tint_fill = on;
                    }
                    return;
                }
                _ => return,
            };
            if *alvo != novo {
                *alvo = novo;
            }
        }
    }
}

/// ⭐ **Publica o que o documento de cada entidade vale AGORA** (através do eco).
///
/// Esquece antes o que cada entidade publicou desta família: um componente tirado (ou a entidade
/// apagada) deixa de ter campos.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = entidades
        .iter()
        .filter_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            let w = sim.world();
            w.get_entity(e).ok()?;
            let v: Vec<_> = campos(w, e)
                .into_iter()
                .filter_map(|kk| Some((kk, ler(w, e, kk)?)))
                .collect();
            Some((b, v))
        })
        .collect();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, OBJETO);
    }
    for (b, valores) in lotes {
        for (kk, v) in valores {
            lidos.publicar(b, kk, v);
        }
    }
}

/// O que a pré-visualização guarda desta entidade, por driver.
fn fotografia(sim: &SimWorld, e: Entity) -> [Option<Driven>; 3] {
    [Driver::SolverPose, Driver::SpriteCor, Driver::TweenTint].map(|d| Driven::read(d, sim, e))
}

/// ⭐ **Aplica ao documento o que a curva mandou neste quadro**, regista o eco e declara a escrita
/// à pré-visualização. Devolve `true` se o documento mudou.
pub fn aplicar(sim: &mut SimWorld, drive: &mut PreviewDrive) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(OBJETO))
    else {
        return false;
    };
    if escritas.is_empty() {
        return false;
    }
    escritas.sort_by_key(|&(e, kk, _)| (e, kk));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for lote in escritas.chunk_by(|a, b| a.0 == b.0) {
        let Some(e) = Entity::try_from_bits(lote[0].0) else {
            continue;
        };
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let antes = fotografia(sim, e);
        for &(b, kk, v) in lote {
            escrever(sim.world_mut(), e, kk, v);
            if let Some(ficou) = ler(sim.world(), e, kk) {
                ecos.push((b, kk, v, ficou));
            }
        }
        let depois = fotografia(sim, e);
        for (a, d) in antes.into_iter().zip(depois) {
            if let (Some(a), Some(d)) = (a, d)
                && a != d
            {
                drive.driven(e, a, d);
                mudou = true;
            }
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, kk, v, f) in ecos {
        lidos.eco(b, kk, v, f);
    }
    mudou
}

#[cfg(test)]
#[path = "canais_objeto_tests.rs"]
mod tests;
