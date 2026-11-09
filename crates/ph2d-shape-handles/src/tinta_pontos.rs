//! **A tinta dos pontos do «Outline»:** o anel, cada ponto no seu estado (normal · sob o ponteiro · selecionado),
//! o «+» fantasma onde um clique na aresta acrescenta e o alvo da fusão a meio do arrasto.

use crate::pontos::{self as p, Ponto};
use bevy_ecs::world::World;
use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

const PONTO_PX: f64 = 3.0; // LITERAL-PX-OK: raio do ponto
const SOB_PX: f64 = 5.0; // LITERAL-PX-OK: raio do ponto sob o ponteiro
const SELECIONADO_PX: f64 = 6.5; // LITERAL-PX-OK: raio do ponto selecionado
const FUSAO_PX: f64 = 9.0; // LITERAL-PX-OK: o anel do alvo da fusão
const MAIS_PX: f64 = 6.0; // LITERAL-PX-OK: raio do «+» fantasma
const MAIS_BRACO_PX: f64 = 3.0; // LITERAL-PX-OK: meio braço do «+»
const TRACO_PX: f64 = 1.5; // LITERAL-PX-OK: traço dos pontos

fn bola(s: &mut ph2d_vector::Scene, c: (f32, f32), r: f64, miolo: Color, borda: Color) {
    let c = Circle::new((f64::from(c.0), f64::from(c.1)), r);
    s.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(miolo),
        None,
        &c,
    );
    let t = Stroke::new(TRACO_PX);
    s.stroke(&t, Affine::IDENTITY, &Brush::Solid(borda), None, &c);
}

/// O estado de cada coisa sob o ponteiro, para a tinta e para os testes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Estados {
    pub sob: Option<Ponto>,
    pub mais: Option<(f32, f32)>,
    pub fusao: Option<Ponto>,
}

/// O que o ponteiro em `s` promete: o ponto sob ele, ou o «+» na aresta, ou (a meio do arrasto) o alvo da fusão.
#[must_use]
pub fn estados(world: &World, ecra: &[Vec<(f32, f32)>], s: (f32, f32)) -> Estados {
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
            ..Estados::default()
        };
    }
    let mais = p::aresta_em(ecra, s).map(|(pt, t)| {
        let r = &ecra[pt.anel];
        let (a, b) = (r[pt.i], r[(pt.i + 1) % r.len()]);
        (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1))
    });
    Estados {
        mais,
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
    let (cor, fundo) = (
        resolve(ColorToken::Selection, theme),
        resolve(ColorToken::Bg0, theme),
    );
    let (realce, perigo, claro) = (
        resolve(ColorToken::Accent, theme),
        resolve(ColorToken::Danger, theme),
        resolve(ColorToken::Text1, theme),
    );
    let ecra = p::no_ecra(&pose, &m, &o, cam, win);
    let est = estados(world, &ecra, ponteiro);
    let sel = world
        .get_resource::<p::PontosNoCanvas>()
        .map(|x| x.selecionados.clone())
        .unwrap_or_default();
    let s = scene.inner_mut();
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
        s.stroke(traco, Affine::IDENTITY, &Brush::Solid(cor), None, &path);
    }
    for (anel, r) in ecra.iter().enumerate() {
        for (i, &c) in r.iter().enumerate() {
            let pt = Ponto { anel, i };
            if sel.binary_search(&pt).is_ok() {
                bola(s, c, SELECIONADO_PX, cor, claro);
            } else if est.sob == Some(pt) {
                bola(s, c, SOB_PX, fundo, realce);
            } else {
                bola(s, c, PONTO_PX, fundo, cor);
            }
        }
    }
    if let Some(f) = est.fusao
        && let Some(&c) = ecra.get(f.anel).and_then(|r| r.get(f.i))
    {
        let anel = Circle::new((f64::from(c.0), f64::from(c.1)), FUSAO_PX);
        let t = Stroke::new(TRACO_PX * 2.0);
        s.stroke(&t, Affine::IDENTITY, &Brush::Solid(perigo), None, &anel);
    }
    if let Some(c) = est.mais {
        bola(s, c, MAIS_PX, fundo, realce);
        let (x, y) = (f64::from(c.0), f64::from(c.1));
        let mut mais = BezPath::new();
        mais.move_to((x - MAIS_BRACO_PX, y));
        mais.line_to((x + MAIS_BRACO_PX, y));
        mais.move_to((x, y - MAIS_BRACO_PX));
        mais.line_to((x, y + MAIS_BRACO_PX));
        let t = Stroke::new(TRACO_PX);
        s.stroke(&t, Affine::IDENTITY, &Brush::Solid(realce), None, &mais);
    }
}
