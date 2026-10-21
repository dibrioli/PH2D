//! **Os componentes VIVOS da entidade nos canais** — o perfil de largura (`VecStrokeProfile`), o
//! padrão ao longo do caminho (`VecPatternPath` + `VecPatternRotation`), o texto no caminho
//! (`VecTextPath`), o Blend vivo (`VecBlend::steps`) e o Envelope vivo (`VecEnvelope`).
//!
//! Cada um é cozido A CADA QUADRO pelo seu `*_live::recook` a partir do componente: escrever o
//! componente é re-cozer. Um componente ausente não tem campo — salvo o perfil, cuja AUSÊNCIA é o
//! neutro (`profile_live::arm`): o «existe» dele é o próprio interruptor. As fontes, o spine
//! autorado, os filhos e o vínculo (`path`) são estrutura, não canal.
//!
//! ⚠️ O `Morph` vivo NÃO mora aqui: o `t` dele já é o `PropKind::Morph` de sempre, e um segundo
//! canal para o mesmo número seriam dois donos.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{
    Entity, EnvelopeKind, EnvelopeWarp, FieldKey, SimWorld, VecBlend, VecEnvelope, VecPatternPath,
    VecPatternRotation, VecStrokeProfile, VecTextPath,
};
use ph2d_vec_scene::{WidthStop, WidthStops};

fn k(campo: u16, j: usize) -> FieldKey {
    FieldKey::new(VETOR, campo, 0, u16::try_from(j).unwrap_or(u16::MAX))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o canal é f32; o documento guarda f64"
)]
fn f(x: f64) -> f32 {
    x as f32
}

/// O índice discreto `v`, arredondado e preso a `0..=max`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "arredondado e preso antes da conversão"
)]
fn indice(v: f32, max: usize) -> usize {
    (v.round().max(0.0) as usize).min(max)
}

const PERFIL_FIM: u16 = c::PERFIL_MULT + 1;
const PADRAO_FIM: u16 = c::PADRAO_CAMINHO_ROTACAO + 1;
const TEXTO_FIM: u16 = c::TEXTO_CAMINHO_ESPELHO + 1;
const ENVELOPE_FIM: u16 = c::ENVELOPE_CURVA + 1;
const KINDS: [EnvelopeKind; 3] = [
    EnvelopeKind::Perspective,
    EnvelopeKind::Mesh,
    EnvelopeKind::Pins,
];

/// O campo `kk` é de um componente vivo daqui?
#[must_use]
pub fn e_vivo(kk: FieldKey) -> bool {
    kk.familia == VETOR
        && kk.i == 0
        && [
            c::PERFIL_EXISTE..PERFIL_FIM,
            c::PADRAO_CAMINHO_INICIO..PADRAO_FIM,
            c::TEXTO_CAMINHO_INICIO..TEXTO_FIM,
            c::BLEND_PASSOS..c::BLEND_PASSOS + 1,
            c::ENVELOPE_CANTO_X..ENVELOPE_FIM,
        ]
        .iter()
        .any(|r| r.contains(&kk.campo))
}

/// Os campos vivos da entidade. `caminho` = ela tem um caminho (só ele pode ganhar um perfil).
pub fn campos(sim: &SimWorld, ent: Entity, caminho: bool, out: &mut Vec<FieldKey>) {
    let w = sim.world();
    let perfil = w.get::<VecStrokeProfile>(ent);
    if caminho || perfil.is_some() {
        out.push(k(c::PERFIL_EXISTE, 0));
    }
    if let Some(p) = perfil {
        out.push(k(c::PERFIL_N, 0));
        for j in 0..p.stops.as_slice().len() {
            out.extend([k(c::PERFIL_POS, j), k(c::PERFIL_MULT, j)]);
        }
    }
    if w.get::<VecPatternPath>(ent).is_some() {
        out.extend((c::PADRAO_CAMINHO_INICIO..PADRAO_FIM).map(|campo| k(campo, 0)));
    }
    if w.get::<VecTextPath>(ent).is_some() {
        out.extend((c::TEXTO_CAMINHO_INICIO..TEXTO_FIM).map(|campo| k(campo, 0)));
    }
    if w.get::<VecBlend>(ent).is_some() {
        out.push(k(c::BLEND_PASSOS, 0));
    }
    if let Some(env) = w.get::<VecEnvelope>(ent) {
        for j in 0..4 {
            out.extend([k(c::ENVELOPE_CANTO_X, j), k(c::ENVELOPE_CANTO_Y, j)]);
        }
        for j in 0..8 {
            out.extend([k(c::ENVELOPE_ALCA_X, j), k(c::ENVELOPE_ALCA_Y, j)]);
        }
        out.extend(
            [
                c::ENVELOPE_TIPO,
                c::ENVELOPE_WARP,
                c::ENVELOPE_N_PINOS,
                c::ENVELOPE_CURVA,
            ]
            .map(|campo| k(campo, 0)),
        );
        for j in 0..env.pins.len() {
            out.extend((c::ENVELOPE_PINO..c::ENVELOPE_CURVA).map(|campo| k(campo, j)));
        }
    }
}

/// Quanto vale `kk` — `None` se o componente (ou a paragem, o pino) não existe.
#[must_use]
pub fn ler(sim: &SimWorld, ent: Entity, kk: FieldKey) -> Option<f32> {
    let w = sim.world();
    let j = usize::from(kk.j);
    match kk.campo {
        c::PERFIL_EXISTE => Some(f32::from(u8::from(
            w.get::<VecStrokeProfile>(ent).is_some(),
        ))),
        c::PERFIL_N..PERFIL_FIM => {
            let s = w.get::<VecStrokeProfile>(ent)?.stops.as_slice();
            Some(match kk.campo {
                c::PERFIL_N => f32::from(u16::try_from(s.len()).unwrap_or(u16::MAX)),
                c::PERFIL_POS => f(s.get(j)?.pos),
                _ => f(s.get(j)?.mult),
            })
        }
        c::PADRAO_CAMINHO_INICIO..PADRAO_FIM => {
            let p = w.get::<VecPatternPath>(ent)?;
            Some(match kk.campo {
                c::PADRAO_CAMINHO_INICIO => p.start_offset,
                c::PADRAO_CAMINHO_FIM => p.end_offset,
                c::PADRAO_CAMINHO_ESPACO => p.spacing,
                c::PADRAO_CAMINHO_DESLOCA => p.offset,
                c::PADRAO_CAMINHO_ESPELHO => f32::from(u8::from(p.flip)),
                _ => w.get::<VecPatternRotation>(ent).map_or(0.0, |r| r.0),
            })
        }
        c::TEXTO_CAMINHO_INICIO..TEXTO_FIM => {
            let t = w.get::<VecTextPath>(ent)?;
            Some(match kk.campo {
                c::TEXTO_CAMINHO_INICIO => t.start_offset,
                _ => f32::from(u8::from(t.flip)),
            })
        }
        c::BLEND_PASSOS => w.get::<VecBlend>(ent).map(|b| {
            #[expect(clippy::cast_precision_loss, reason = "passos ≤ MAX_BLEND_STEPS")]
            let n = b.steps as f32;
            n
        }),
        c::ENVELOPE_CANTO_X..ENVELOPE_FIM => ler_envelope(w.get::<VecEnvelope>(ent)?, kk.campo, j),
        _ => None,
    }
}

fn ler_envelope(env: &VecEnvelope, campo: u16, j: usize) -> Option<f32> {
    let eixo = usize::from(matches!(campo, c::ENVELOPE_CANTO_Y | c::ENVELOPE_ALCA_Y));
    Some(match campo {
        c::ENVELOPE_CANTO_X | c::ENVELOPE_CANTO_Y => f(env.corners.get(j)?[eixo]),
        c::ENVELOPE_ALCA_X | c::ENVELOPE_ALCA_Y => f(env.edges.get(j / 2)?[j % 2][eixo]),
        c::ENVELOPE_TIPO => {
            #[expect(clippy::cast_precision_loss, reason = "três tipos")]
            let t = KINDS.iter().position(|x| *x == env.kind).unwrap_or(0) as f32;
            t
        }
        c::ENVELOPE_WARP => {
            let k = env
                .warp
                .and_then(|w| EnvelopeWarp::ALL.iter().position(|x| *x == w))
                .map_or(0, |k| k + 1);
            f32::from(u16::try_from(k).unwrap_or(0))
        }
        c::ENVELOPE_N_PINOS => f32::from(u16::try_from(env.pins.len()).unwrap_or(u16::MAX)),
        c::ENVELOPE_CURVA => f(env.bend),
        pino => {
            let m = usize::from(pino - c::ENVELOPE_PINO);
            f(env.pins.get(j)?[m / 2][m % 2])
        }
    })
}

/// O perfil neutro que ligar o «existe» arma (duas paragens a `1×`); as paragens da curva
/// aplicam-se a seguir (a ordem estrutura-primeiro de `canais_entidade::prioridade`).
fn perfil_neutro() -> VecStrokeProfile {
    VecStrokeProfile {
        stops: WidthStops::new(vec![
            WidthStop {
                pos: 0.0,
                mult: 1.0,
            },
            WidthStop {
                pos: 1.0,
                mult: 1.0,
            },
        ]),
    }
}

/// Aplica `v` a `kk`. Um componente ausente fica ausente (o «existe» do perfil é a excepção: é o
/// interruptor dele).
pub fn escrever(sim: &mut SimWorld, ent: Entity, kk: FieldKey, v: f32) {
    if !e_vivo(kk) {
        return;
    }
    let j = usize::from(kk.j);
    let d = f64::from(v);
    let liga = v >= 0.5;
    let Ok(mut em) = sim.world_mut().get_entity_mut(ent) else {
        return;
    };
    match kk.campo {
        c::PERFIL_EXISTE => match (liga, em.contains::<VecStrokeProfile>()) {
            (true, false) => {
                em.insert(perfil_neutro());
            }
            (false, true) => {
                em.remove::<VecStrokeProfile>();
            }
            _ => {}
        },
        c::PERFIL_N..PERFIL_FIM => {
            let Some(mut p) = em.get_mut::<VecStrokeProfile>() else {
                return;
            };
            let mut s = p.stops.as_slice().to_vec();
            match kk.campo {
                c::PERFIL_N => {
                    let ultimo = s.last().copied().unwrap_or(WidthStop {
                        pos: 1.0,
                        mult: 1.0,
                    });
                    s.resize(indice(v, usize::from(u16::MAX)), ultimo);
                }
                c::PERFIL_POS => match s.get_mut(j) {
                    Some(x) => x.pos = d,
                    None => return,
                },
                _ => match s.get_mut(j) {
                    Some(x) => x.mult = d.max(0.0),
                    None => return,
                },
            }
            p.stops = WidthStops::new(s);
        }
        c::PADRAO_CAMINHO_ROTACAO => {
            if em.contains::<VecPatternPath>() {
                if v == 0.0 {
                    em.remove::<VecPatternRotation>();
                } else {
                    em.insert(VecPatternRotation(v));
                }
            }
        }
        c::PADRAO_CAMINHO_INICIO..PADRAO_FIM => {
            let Some(mut p) = em.get_mut::<VecPatternPath>() else {
                return;
            };
            match kk.campo {
                c::PADRAO_CAMINHO_INICIO => p.start_offset = v,
                c::PADRAO_CAMINHO_FIM => p.end_offset = v,
                c::PADRAO_CAMINHO_ESPACO => p.spacing = v,
                c::PADRAO_CAMINHO_DESLOCA => p.offset = v,
                _ => p.flip = liga,
            }
        }
        c::TEXTO_CAMINHO_INICIO..TEXTO_FIM => {
            let Some(mut t) = em.get_mut::<VecTextPath>() else {
                return;
            };
            match kk.campo {
                c::TEXTO_CAMINHO_INICIO => t.start_offset = v,
                _ => t.flip = liga,
            }
        }
        c::BLEND_PASSOS => {
            if let Some(mut b) = em.get_mut::<VecBlend>() {
                let max = ph2d_tool_vector::params::MAX_BLEND_STEPS;
                let n = indice(v, usize::try_from(max).unwrap_or(usize::MAX));
                b.steps = u32::try_from(n).unwrap_or(max).max(1);
            }
        }
        _ => {
            if let Some(mut env) = em.get_mut::<VecEnvelope>() {
                escrever_envelope(&mut env, kk.campo, j, v);
            }
        }
    }
}

fn escrever_envelope(env: &mut VecEnvelope, campo: u16, j: usize, v: f32) {
    let d = f64::from(v);
    let eixo = usize::from(matches!(campo, c::ENVELOPE_CANTO_Y | c::ENVELOPE_ALCA_Y));
    match campo {
        c::ENVELOPE_CANTO_X | c::ENVELOPE_CANTO_Y => {
            if let Some(p) = env.corners.get_mut(j) {
                p[eixo] = d;
            }
        }
        c::ENVELOPE_ALCA_X | c::ENVELOPE_ALCA_Y => {
            if let Some(e) = env.edges.get_mut(j / 2) {
                e[j % 2][eixo] = d;
            }
        }
        c::ENVELOPE_TIPO => env.kind = KINDS[indice(v, KINDS.len() - 1)],
        c::ENVELOPE_WARP => {
            env.warp = indice(v, EnvelopeWarp::ALL.len())
                .checked_sub(1)
                .map(|k| EnvelopeWarp::ALL[k]);
        }
        c::ENVELOPE_N_PINOS => {
            let n = indice(v, usize::from(u16::MAX));
            let ultimo = env.pins.last().copied().unwrap_or_default();
            env.pins.resize(n, ultimo);
        }
        c::ENVELOPE_CURVA => env.bend = d.clamp(-1.0, 1.0),
        pino => {
            let m = usize::from(pino - c::ENVELOPE_PINO);
            if let Some(p) = env.pins.get_mut(j) {
                p[m / 2][m % 2] = d;
            }
        }
    }
}

#[cfg(test)]
#[path = "canais_vivos_tests.rs"]
mod tests;
