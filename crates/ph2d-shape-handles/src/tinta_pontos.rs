//! **A tinta dos pontos do «Outline»:** o anel, cada ponto no seu estado (normal · sob o ponteiro · selecionado),
//! o «+» fantasma onde um clique na aresta acrescenta e o alvo da fusão a meio do arrasto.

use crate::pontos::{self as p, Ponto};
use bevy_ecs::world::World;
use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

const PONTO_PX: f64 = 3.5; // LITERAL-PX-OK: raio do ponto
const SOB_PX: f64 = 5.0; // LITERAL-PX-OK: raio do ponto sob o ponteiro
const SELECIONADO_PX: f64 = 6.0; // LITERAL-PX-OK: raio do ponto selecionado
const MIOLO_PX: f64 = 2.25; // LITERAL-PX-OK: o anel claro dentro do selecionado
const HALO_PX: f64 = 1.75; // LITERAL-PX-OK: o halo escuro que faz ler sobre qualquer arte
const BRILHO_PX: f64 = 4.5; // LITERAL-PX-OK: o brilho à volta do ponto sob o ponteiro
const FUSAO_PX: f64 = 9.5; // LITERAL-PX-OK: o anel do alvo da fusão
const FUSAO_FORA_PX: f64 = 13.5; // LITERAL-PX-OK: o segundo anel do alvo da fusão
const MAIS_PX: f64 = 6.5; // LITERAL-PX-OK: raio do «+» fantasma
const MAIS_BRACO_PX: f64 = 3.25; // LITERAL-PX-OK: meio braço do «+» e do «−»
const TRACO_PX: f64 = 1.5; // LITERAL-PX-OK: traço dos pontos
const ARESTA_PX: f64 = 3.0; // LITERAL-PX-OK: a aresta sob o ponteiro
const TRACEJADO_PX: [f64; 2] = [4.0, 3.0]; // LITERAL-PX-OK: o fio até ao alvo da fusão
const BRILHO_ALFA: f32 = 0.28; // LITERAL-PX-OK: opacidade do brilho
const HALO_ALFA: f32 = 0.7; // LITERAL-PX-OK: opacidade do halo
const FANTASMA_ALFA: f32 = 0.85; // LITERAL-PX-OK: o miolo do «+» fantasma

fn disco(s: &mut ph2d_vector::Scene, c: (f32, f32), r: f64, cor: Color) {
    let c = Circle::new((f64::from(c.0), f64::from(c.1)), r);
    s.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(cor),
        None,
        &c,
    );
}

fn anel(s: &mut ph2d_vector::Scene, c: (f32, f32), r: f64, w: f64, cor: Color) {
    let c = Circle::new((f64::from(c.0), f64::from(c.1)), r);
    s.stroke(
        &Stroke::new(w),
        Affine::IDENTITY,
        &Brush::Solid(cor),
        None,
        &c,
    );
}

/// Um ponto: halo escuro, miolo e borda nítida.
fn bola(
    s: &mut ph2d_vector::Scene,
    c: (f32, f32),
    r: f64,
    miolo: Color,
    borda: Color,
    halo: Color,
) {
    disco(s, c, r + HALO_PX, halo);
    disco(s, c, r, miolo);
    anel(s, c, r - TRACO_PX / 2.0, TRACO_PX, borda);
}

fn barra(s: &mut ph2d_vector::Scene, c: (f32, f32), vertical: bool, cor: Color) {
    let (x, y) = (f64::from(c.0), f64::from(c.1));
    let mut b = BezPath::new();
    if vertical {
        b.move_to((x, y - MAIS_BRACO_PX));
        b.line_to((x, y + MAIS_BRACO_PX));
    } else {
        b.move_to((x - MAIS_BRACO_PX, y));
        b.line_to((x + MAIS_BRACO_PX, y));
    }
    let t = Stroke::new(TRACO_PX).with_caps(ph2d_vector::Cap::Round);
    s.stroke(&t, Affine::IDENTITY, &Brush::Solid(cor), None, &b);
}

fn linha(s: &mut ph2d_vector::Scene, a: (f32, f32), b: (f32, f32), t: &Stroke, cor: Color) {
    let mut p = BezPath::new();
    p.move_to((f64::from(a.0), f64::from(a.1)));
    p.line_to((f64::from(b.0), f64::from(b.1)));
    s.stroke(t, Affine::IDENTITY, &Brush::Solid(cor), None, &p);
}

/// O estado de cada coisa sob o ponteiro, para a tinta e para os testes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Estados {
    pub sob: Option<Ponto>,
    pub mais: Option<(f32, f32)>,
    pub fusao: Option<Ponto>,
    /// A aresta onde o «+» acrescenta (`i` → `i + 1`): realçada.
    pub aresta: Option<Ponto>,
    /// `Alt` sobre um ponto: o clique tira-o (pinta em perigo, com «−»).
    pub tira: Option<Ponto>,
}

/// O que o ponteiro em `s` promete, com as teclas do último gesto ([`crate::TeclasNoCanvas`]).
#[must_use]
pub fn estados(world: &World, ecra: &[Vec<(f32, f32)>], s: (f32, f32)) -> Estados {
    let teclas = world
        .get_resource::<crate::TeclasNoCanvas>()
        .map(|t| t.0)
        .unwrap_or_default();
    estados_com(world, ecra, s, teclas)
}

/// O ponto sob ele (com `Alt`, o que sai), ou o «+» na aresta, ou (a meio do arrasto) o alvo da fusão.
#[must_use]
pub fn estados_com(
    world: &World,
    ecra: &[Vec<(f32, f32)>],
    s: (f32, f32),
    teclas: crate::Teclas,
) -> Estados {
    let arrasto = world
        .get_resource::<p::PontosNoCanvas>()
        .and_then(|x| x.arrasto.clone());
    if let Some(a) = arrasto {
        let fusao = a
            .armado
            .then(|| p::alvo_da_fusao(ecra, a.ponto, s))
            .flatten();
        return Estados {
            sob: Some(a.ponto),
            fusao,
            ..Estados::default()
        };
    }
    if let Some(pt) = p::ponto_em(ecra, s) {
        return Estados {
            sob: Some(pt),
            tira: teclas.alt.then_some(pt),
            ..Estados::default()
        };
    }
    let Some((pt, t)) = p::aresta_em(ecra, s) else {
        return Estados::default();
    };
    let r = &ecra[pt.anel];
    let (a, b) = (r[pt.i], r[(pt.i + 1) % r.len()]);
    Estados {
        mais: Some((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1))),
        aresta: Some(pt),
        ..Estados::default()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn desenhar(
    world: &World,
    alvo: u64,
    cam: &Camera2d,
    win: WindowSize,
    ponteiro: (f32, f32),
    theme: Theme,
    traco: &Stroke,
    scene: &mut VectorScene,
) {
    let Some((pose, m, o)) = p::do_alvo(world, alvo) else {
        return;
    };
    let (cor, fundo, claro) = (
        resolve(ColorToken::Selection, theme),
        resolve(ColorToken::Bg0, theme),
        resolve(ColorToken::Text1, theme),
    );
    let (realce, perigo) = (
        resolve(ColorToken::Accent, theme),
        resolve(ColorToken::Danger, theme),
    );
    let halo = fundo.with_alpha(HALO_ALFA);
    let ecra = p::no_ecra(&pose, &m, &o, cam, win);
    let est = estados(world, &ecra, ponteiro);
    let sel = world
        .get_resource::<p::PontosNoCanvas>()
        .map(|x| x.selecionados.clone())
        .unwrap_or_default();
    let s = scene.inner_mut();
    // O contorno com halo: lê-se sobre arte clara e escura.
    let mut largo = traco.clone();
    largo.width += 2.0 * HALO_PX;
    for r in &ecra {
        let mut path = BezPath::new();
        for (i, &(x, y)) in r.iter().enumerate() {
            let q = Point::new(f64::from(x), f64::from(y));
            if i == 0 {
                path.move_to(q);
            } else {
                path.line_to(q);
            }
        }
        path.close_path();
        s.stroke(&largo, Affine::IDENTITY, &Brush::Solid(halo), None, &path);
        s.stroke(traco, Affine::IDENTITY, &Brush::Solid(cor), None, &path);
    }
    if let Some(a) = est.aresta
        && let Some(r) = ecra.get(a.anel)
    {
        let (x, y) = (r[a.i], r[(a.i + 1) % r.len()]);
        let t = Stroke::new(ARESTA_PX + 2.0 * HALO_PX).with_caps(ph2d_vector::Cap::Round);
        linha(s, x, y, &t, halo);
        linha(
            s,
            x,
            y,
            &Stroke::new(ARESTA_PX).with_caps(ph2d_vector::Cap::Round),
            realce,
        );
    }
    let arrastado = est.fusao.and(est.sob);
    if let (Some(f), Some(d)) = (est.fusao, arrastado)
        && let (Some(&c), Some(&o)) = (
            ecra.get(f.anel).and_then(|r| r.get(f.i)),
            ecra.get(d.anel).and_then(|r| r.get(d.i)),
        )
    {
        let t = Stroke::new(TRACO_PX).with_dashes(0.0, TRACEJADO_PX);
        linha(s, o, c, &t, perigo);
    }
    for (anel_i, r) in ecra.iter().enumerate() {
        for (i, &c) in r.iter().enumerate() {
            let pt = Ponto { anel: anel_i, i };
            let sob = est.sob == Some(pt);
            if est.tira == Some(pt) {
                disco(s, c, SOB_PX + BRILHO_PX, perigo.with_alpha(BRILHO_ALFA));
                bola(s, c, SOB_PX, perigo, perigo, halo);
                barra(s, c, false, claro);
            } else if sel.binary_search(&pt).is_ok() {
                if sob {
                    disco(
                        s,
                        c,
                        SELECIONADO_PX + BRILHO_PX,
                        realce.with_alpha(BRILHO_ALFA),
                    );
                }
                bola(s, c, SELECIONADO_PX, realce, realce, halo);
                anel(s, c, MIOLO_PX, TRACO_PX, claro);
            } else if sob {
                disco(s, c, SOB_PX + BRILHO_PX, realce.with_alpha(BRILHO_ALFA));
                bola(s, c, SOB_PX, claro, realce, halo);
            } else {
                bola(s, c, PONTO_PX, claro, realce, halo);
            }
        }
    }
    if let Some(f) = est.fusao
        && let Some(&c) = ecra.get(f.anel).and_then(|r| r.get(f.i))
    {
        anel(s, c, FUSAO_PX, TRACO_PX * 2.0 + 2.0 * HALO_PX, halo);
        anel(s, c, FUSAO_PX, TRACO_PX * 2.0, perigo);
        anel(
            s,
            c,
            FUSAO_FORA_PX,
            TRACO_PX,
            perigo.with_alpha(BRILHO_ALFA * 2.0),
        );
    }
    if let Some(c) = est.mais {
        bola(s, c, MAIS_PX, fundo.with_alpha(FANTASMA_ALFA), realce, halo);
        barra(s, c, false, realce);
        barra(s, c, true, realce);
    }
}
