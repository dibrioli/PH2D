//! **A TINTA dos campos do vetor** — ler e escrever uma `Paint` (cor chapada, gradientes, multiponto,
//! padrão) campo a campo. Irmão de [`super`] (`canais_campos.rs`) pelo tecto de 700 linhas, cortado
//! pelo assunto: o caminho e o traço lá, o que uma tinta tem por dentro aqui.

use super::*;

pub(super) fn ler_paint(p: &Paint, off: u16, j: usize) -> Option<f32> {
    if off == c::OFF_TIPO {
        return Some(tipo_tinta(p));
    }
    match p {
        Paint::Solid(cor) if (c::OFF_SOLIDA..c::OFF_SOLIDA + 4).contains(&off) => {
            Some(rgba(*cor)[usize::from(off - c::OFF_SOLIDA)])
        }
        Paint::Linear { stops, start, end } => match off {
            c::OFF_N_PARAGENS => Some(f32::from(n16(stops.len()))),
            o if (c::OFF_LINEAR..c::OFF_LINEAR + 4).contains(&o) => {
                Some(f(
                    [start[0], start[1], end[0], end[1]][usize::from(o - c::OFF_LINEAR)]
                ))
            }
            o => ler_paragem(stops, o, j),
        },
        Paint::Radial {
            stops,
            center,
            radius,
        } => match off {
            c::OFF_N_PARAGENS => Some(f32::from(n16(stops.len()))),
            o if (c::OFF_RADIAL..c::OFF_RADIAL + 3).contains(&o) => {
                Some(f(
                    [center[0], center[1], *radius][usize::from(o - c::OFF_RADIAL)]
                ))
            }
            o => ler_paragem(stops, o, j),
        },
        Paint::MultiPoint { points } => match off {
            c::OFF_N_PONTOS => Some(f32::from(n16(points.len()))),
            o if (c::OFF_PONTO..c::OFF_PONTO + 8).contains(&o) => {
                let pt = points.get(j)?;
                let [r, g, b, a] = rgba(pt.color);
                Some(
                    [
                        f(pt.pos[0]),
                        f(pt.pos[1]),
                        r,
                        g,
                        b,
                        a,
                        f(pt.influence),
                        f(pt.jitter),
                    ][usize::from(o - c::OFF_PONTO)],
                )
            }
            _ => None,
        },
        Paint::Pattern(pt) => ler_padrao(pt, off),
        Paint::Solid(_) => None,
    }
}

fn ler_paragem(stops: &[GradientStop], off: u16, j: usize) -> Option<f32> {
    if !(c::OFF_PARAGEM..c::OFF_PARAGEM + 5).contains(&off) {
        return None;
    }
    let s = stops.get(j)?;
    let [r, g, b, a] = rgba(s.color);
    Some([f(s.offset), r, g, b, a][usize::from(off - c::OFF_PARAGEM)])
}

pub(super) fn ler_padrao(pt: &ph2d_vec_scene::PatternFill, off: u16) -> Option<f32> {
    match off {
        o if (c::OFF_PADRAO..c::OFF_PADRAO + 8).contains(&o) => Some(
            [
                f(pt.size[0]),
                f(pt.size[1]),
                f(pt.gap[0]),
                f(pt.gap[1]),
                f(pt.origin[0]),
                f(pt.origin[1]),
                f(pt.angle),
                pt.alpha,
            ][usize::from(o - c::OFF_PADRAO)],
        ),
        o if o == c::OFF_PADRAO_MODOS => Some(pos(&LADRILHOS, &pt.kind)),
        o if o == c::OFF_PADRAO_MODOS + 1 => Some(pos(&MODOS, &pt.mode)),
        _ => None,
    }
}

/// O tipo da tinta mudou: converte guardando a cor principal (as paragens, pontos e posições vêm
/// das curvas do mesmo quadro, escritas a seguir — [`prioridade`]).
fn converter(t: &mut Paint, tipo: usize) {
    let cor = t.primary_color();
    let paragens = || vec![GradientStop::new(0.0, cor), GradientStop::new(1.0, cor)];
    *t = match tipo {
        0 => Paint::Solid(cor),
        1 => Paint::Linear {
            stops: paragens(),
            start: [0.0, 0.0],
            end: [1.0, 0.0],
        },
        2 => Paint::Radial {
            stops: paragens(),
            center: [0.0, 0.0],
            radius: 1.0,
        },
        3 => Paint::MultiPoint {
            points: vec![ph2d_vec_scene::GradientPoint::new([0.0, 0.0], cor, 1.0)],
        },
        // O padrão traz uma fonte (imagem ou forma) que a curva não cria: fica o que estava.
        _ => return,
    };
}

pub(super) fn escrever_paint(t: &mut Paint, off: u16, j: usize, v: f32) {
    let d = f64::from(v);
    if off == c::OFF_TIPO {
        let tipo = indice(v, 5);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "0..=4"
        )]
        let agora = tipo_tinta(t) as usize;
        if tipo != agora {
            converter(t, tipo);
        }
        return;
    }
    match t {
        Paint::Solid(cor) => {
            if (c::OFF_SOLIDA..c::OFF_SOLIDA + 4).contains(&off) {
                set_rgba(cor, off - c::OFF_SOLIDA, v);
            }
        }
        Paint::Linear { stops, start, end } => match off {
            o if (c::OFF_LINEAR..c::OFF_LINEAR + 4).contains(&o) => match o - c::OFF_LINEAR {
                0 => start[0] = d,
                1 => start[1] = d,
                2 => end[0] = d,
                _ => end[1] = d,
            },
            o => escrever_paragem(stops, o, j, v),
        },
        Paint::Radial {
            stops,
            center,
            radius,
        } => match off {
            o if (c::OFF_RADIAL..c::OFF_RADIAL + 3).contains(&o) => match o - c::OFF_RADIAL {
                0 => center[0] = d,
                1 => center[1] = d,
                _ => *radius = d.max(0.0),
            },
            o => escrever_paragem(stops, o, j, v),
        },
        Paint::MultiPoint { points } => {
            if off == c::OFF_N_PONTOS {
                redimensionar(points, indice(v, usize::from(u16::MAX)).max(1));
            } else if (c::OFF_PONTO..c::OFF_PONTO + 8).contains(&off)
                && let Some(pt) = points.get_mut(j)
            {
                match off - c::OFF_PONTO {
                    0 => pt.pos[0] = d,
                    1 => pt.pos[1] = d,
                    o @ 2..=5 => set_rgba(&mut pt.color, o - 2, v),
                    6 => pt.influence = d,
                    _ => pt.jitter = d,
                }
            }
        }
        Paint::Pattern(pt) => escrever_padrao(pt, off, v),
    }
}

fn redimensionar<T: Clone>(xs: &mut Vec<T>, n: usize) {
    if let Some(ultimo) = xs.last().cloned() {
        xs.resize(n, ultimo);
    }
}

fn escrever_paragem(stops: &mut Vec<GradientStop>, off: u16, j: usize, v: f32) {
    if off == c::OFF_N_PARAGENS {
        redimensionar(stops, indice(v, usize::from(u16::MAX)).max(2));
        return;
    }
    if !(c::OFF_PARAGEM..c::OFF_PARAGEM + 5).contains(&off) {
        return;
    }
    let Some(s) = stops.get_mut(j) else { return };
    match off - c::OFF_PARAGEM {
        0 => s.offset = f64::from(v).clamp(0.0, 1.0),
        o => set_rgba(&mut s.color, o - 1, v),
    }
}

pub(super) fn escrever_padrao(pt: &mut ph2d_vec_scene::PatternFill, off: u16, v: f32) {
    let d = f64::from(v);
    match off {
        o if (c::OFF_PADRAO..c::OFF_PADRAO + 8).contains(&o) => match o - c::OFF_PADRAO {
            0 => pt.size[0] = d,
            1 => pt.size[1] = d,
            2 => pt.gap[0] = d,
            3 => pt.gap[1] = d,
            4 => pt.origin[0] = d,
            5 => pt.origin[1] = d,
            6 => pt.angle = d,
            _ => pt.alpha = v.clamp(0.0, 1.0),
        },
        o if o == c::OFF_PADRAO_MODOS => pt.kind = LADRILHOS[indice(v, LADRILHOS.len())],
        o if o == c::OFF_PADRAO_MODOS + 1 => pt.mode = MODOS[indice(v, MODOS.len())],
        _ => {}
    }
}
