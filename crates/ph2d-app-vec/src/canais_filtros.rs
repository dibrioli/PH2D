//! **Os FILTROS da forma nos canais** (`VecFilter::ops`, `canal_vetor::FILTRO_*`) — o leitor e o
//! escritor puros da pilha de FX raster. `i` = o degrau; o olho (`FILTRO_LIGADO`) é um grupo, o
//! resto do degrau outro. Cada degrau publica só os campos que o TIPO dele usa (a tabela
//! `FxKindSpec`, a mesma que decide as linhas do painel): um knob morto não vira canal.
//! Criar, remover e reordenar degraus é estrutura (plano 35 §8), não canal.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{FieldKey, FxOp, VecFilter};

const FIM: u16 = c::FILTRO_PARAGEM + 5;

/// O campo `kk` é de um filtro?
#[must_use]
pub fn e_de_filtro(kk: FieldKey) -> bool {
    kk.familia == VETOR && (c::FILTRO_TIPO..FIM).contains(&kk.campo)
}

fn n16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// Os campos (sem `j`) que um degrau deste tipo tem.
fn usa(op: &FxOp) -> Vec<u16> {
    let s = FxOp::spec(op.kind);
    let mut out = vec![c::FILTRO_TIPO, c::FILTRO_LIGADO, c::FILTRO_OPACIDADE];
    let mut se = |tem: bool, campos: &[u16]| {
        if tem {
            out.extend_from_slice(campos);
        }
    };
    se(s.radius_label.is_some(), &[c::FILTRO_RAIO]);
    se(
        s.offset_labels.is_some(),
        &[c::FILTRO_DESLOCA_X, c::FILTRO_DESLOCA_Y],
    );
    se(
        s.color_label.is_some(),
        &[
            c::FILTRO_COR,
            c::FILTRO_COR + 1,
            c::FILTRO_COR + 2,
            c::FILTRO_COR + 3,
        ],
    );
    se(
        s.color_b_label.is_some(),
        &[
            c::FILTRO_COR_B,
            c::FILTRO_COR_B + 1,
            c::FILTRO_COR_B + 2,
            c::FILTRO_COR_B + 3,
        ],
    );
    se(!s.modes.is_empty(), &[c::FILTRO_MODO]);
    se(s.takes_blend, &[c::FILTRO_MISTURA]);
    se(
        s.noise_labels.is_some(),
        &[c::FILTRO_ESCALA, c::FILTRO_DETALHE, c::FILTRO_SEMENTE],
    );
    se(s.grow_label.is_some(), &[c::FILTRO_CRESCE]);
    se(
        s.adjust_labels.is_some(),
        &[c::FILTRO_MATIZ, c::FILTRO_SATURACAO, c::FILTRO_BRILHO],
    );
    se(s.takes_ramp, &[c::FILTRO_N_PARAGENS]);
    out
}

/// Os campos que a pilha tem agora.
#[must_use]
pub fn campos(f: &VecFilter) -> Vec<FieldKey> {
    let mut out = Vec::new();
    for (i, op) in f.ops.iter().enumerate() {
        let i = n16(i);
        out.extend(
            usa(op)
                .into_iter()
                .map(|campo| FieldKey::new(VETOR, campo, i, 0)),
        );
        if FxOp::spec(op.kind).takes_ramp {
            for j in 0..n16(paragens(op)) {
                out.extend((c::FILTRO_PARAGEM..FIM).map(|campo| FieldKey::new(VETOR, campo, i, j)));
            }
        }
    }
    out
}

fn paragens(op: &FxOp) -> usize {
    usize::from(op.stop_count).min(FxOp::MAX_GRADIENT_STOPS)
}

/// Quanto vale `kk` — `None` se o degrau (ou a paragem) não existe.
#[must_use]
pub fn ler(f: &VecFilter, kk: FieldKey) -> Option<f32> {
    if !e_de_filtro(kk) {
        return None;
    }
    let op = f.ops.get(usize::from(kk.i))?;
    let j = usize::from(kk.j);
    if (c::FILTRO_PARAGEM..FIM).contains(&kk.campo) {
        if j >= paragens(op) {
            return None;
        }
        let m = usize::from(kk.campo - c::FILTRO_PARAGEM);
        return Some(if m == 0 {
            op.stop_pos[j]
        } else {
            op.stops[j][m - 1]
        });
    }
    if kk.j != 0 {
        return None;
    }
    Some(match kk.campo {
        c::FILTRO_TIPO => f32::from(op.kind),
        c::FILTRO_LIGADO => f32::from(u8::from(op.enabled)),
        c::FILTRO_RAIO => op.radius,
        c::FILTRO_DESLOCA_X => op.offset[0],
        c::FILTRO_DESLOCA_Y => op.offset[1],
        x @ c::FILTRO_COR..c::FILTRO_COR_B => op.color[usize::from(x - c::FILTRO_COR)],
        x @ c::FILTRO_COR_B..c::FILTRO_OPACIDADE => op.color_b[usize::from(x - c::FILTRO_COR_B)],
        c::FILTRO_OPACIDADE => op.opacity,
        c::FILTRO_MODO => f32::from(op.mode),
        c::FILTRO_MISTURA => f32::from(op.blend),
        c::FILTRO_ESCALA => op.scale,
        c::FILTRO_DETALHE => f32::from(op.detail),
        c::FILTRO_SEMENTE => f32::from(op.seed),
        c::FILTRO_CRESCE => op.grow,
        c::FILTRO_MATIZ => op.hue,
        c::FILTRO_SATURACAO => op.sat,
        c::FILTRO_BRILHO => op.bright,
        _ => f32::from(op.stop_count),
    })
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "arredondado e preso ao domínio antes da conversão"
)]
fn byte(v: f32, min: u8, max: u8) -> u8 {
    v.round().clamp(f32::from(min), f32::from(max)) as u8
}

fn ultimo(n: usize) -> u8 {
    u8::try_from(n.saturating_sub(1)).unwrap_or(u8::MAX)
}

/// Aplica `v` a `kk` com o domínio do painel: as cores, a opacidade e as posições em `0..=1`,
/// os discretos arredondados ao índice válido. Um degrau ou uma paragem ausente fica ausente.
pub fn escrever(f: &mut VecFilter, kk: FieldKey, v: f32) {
    if !e_de_filtro(kk) {
        return;
    }
    let Some(op) = f.ops.get_mut(usize::from(kk.i)) else {
        return;
    };
    let j = usize::from(kk.j);
    if (c::FILTRO_PARAGEM..FIM).contains(&kk.campo) {
        if j < paragens(op) {
            match usize::from(kk.campo - c::FILTRO_PARAGEM) {
                0 => op.stop_pos[j] = v.clamp(0.0, 1.0),
                m => op.stops[j][m - 1] = v.clamp(0.0, 1.0),
            }
        }
        return;
    }
    if kk.j != 0 {
        return;
    }
    match kk.campo {
        c::FILTRO_TIPO => {
            op.kind = byte(v, 0, ultimo(FxOp::KINDS));
            if FxOp::spec(op.kind).modes.is_empty() {
                op.mode = 0;
            }
        }
        c::FILTRO_LIGADO => op.enabled = v >= 0.5,
        c::FILTRO_RAIO => op.radius = v.max(0.0),
        c::FILTRO_DESLOCA_X => op.offset[0] = v,
        c::FILTRO_DESLOCA_Y => op.offset[1] = v,
        x @ c::FILTRO_COR..c::FILTRO_COR_B => {
            op.color[usize::from(x - c::FILTRO_COR)] = v.clamp(0.0, 1.0);
        }
        x @ c::FILTRO_COR_B..c::FILTRO_OPACIDADE => {
            op.color_b[usize::from(x - c::FILTRO_COR_B)] = v.clamp(0.0, 1.0);
        }
        c::FILTRO_OPACIDADE => op.opacity = v.clamp(0.0, 1.0),
        c::FILTRO_MODO => {
            op.mode = byte(v, 0, ultimo(FxOp::spec(op.kind).modes.len()));
        }
        c::FILTRO_MISTURA => op.blend = byte(v, 0, FxOp::BLEND_KINDS - 1),
        c::FILTRO_ESCALA => op.scale = v.max(0.0),
        c::FILTRO_DETALHE => op.detail = byte(v, 1, FxOp::MAX_DETAIL),
        c::FILTRO_SEMENTE => op.seed = byte(v, 0, u8::MAX),
        c::FILTRO_CRESCE => op.grow = v,
        c::FILTRO_MATIZ => op.hue = v,
        c::FILTRO_SATURACAO => op.sat = v.clamp(-1.0, 1.0),
        c::FILTRO_BRILHO => op.bright = v.clamp(-1.0, 1.0),
        _ => op.stop_count = byte(v, 0, ultimo(FxOp::MAX_GRADIENT_STOPS + 1)),
    }
}

#[cfg(test)]
#[path = "canais_filtros_tests.rs"]
mod tests;
