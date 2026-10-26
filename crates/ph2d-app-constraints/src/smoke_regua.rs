//! **A cena 6 — «Régua de desenho»** (`PH2D_CONSTRAINT_SMOKE=6`, `docs/BlenderConstraints/04_W4_cad.md` §5): um
//! perfil de [`DENTES`] dentes no documento vetorial, H/V em todo lado, os topos e as folgas iguais, o canto de
//! baixo à esquerda e o chão das folgas presos, e a ALTURA do dente do meio animada ([`MEDIDA`]). O gate que toca e arrasta o tempo é
//! `an_animated_dimension_plays_and_scrubs_to_the_same_bit` (as mesmas chaves, [`chaves_da_medida`]).

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::prelude::Resource;
use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_ecs::canal_cad::{MEDIDA as CAMPO, chave};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineState};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, RegraId, SegCad};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertId};

use crate::chaves::Escrita;

/// Quantos dentes; o do meio é o que estica.
pub const DENTES: u32 = 5;
/// A largura de um topo e de uma folga, a altura dos dentes e a da base (unidades do mundo).
pub const LARGURA: f64 = 0.5;
pub const ALTURA: f64 = 0.8;
pub const BASE: f64 = 0.6;
/// As chaves da altura do dente do meio: `(quadro, valor)`.
pub const MEDIDA: [(u32, f32); 3] = [(0, 0.8), (24, 1.6), (48, 0.8)];

/// A cena 6 foi pedida e o documento ainda não a tem (o `encenar` só vê o mundo; o [`crate::quadro`] semeia).
#[derive(Resource)]
pub(crate) struct Semear;

/// O que se semeou: o path da régua e a regra animada (a [`step`] grava as chaves nela).
#[derive(Resource, Clone, Copy, Debug)]
pub struct Regua {
    pub path: VecPathId,
    pub medida: RegraId,
}

/// Os cantos do perfil, pela ordem do contorno: o canto de baixo à esquerda, e por cada dente o topo e (salvo o
/// último) a folga; fecha pela base.
fn cantos() -> Vec<[f64; 2]> {
    let x0 = -(f64::from(2 * DENTES - 1) * LARGURA) / 2.0;
    let mut v = vec![[x0, -BASE]];
    for k in 0..DENTES {
        let xl = x0 + f64::from(2 * k) * LARGURA;
        let xr = xl + LARGURA;
        v.extend([[xl, ALTURA], [xr, ALTURA]]);
        if k + 1 < DENTES {
            v.extend([[xr, 0.0], [xr + LARGURA, 0.0]]);
        }
    }
    v.push([x0 + f64::from(2 * DENTES - 1) * LARGURA, -BASE]);
    v
}

/// Põe a régua e as regras dela no documento; devolve o path e a regra animada.
pub fn semear(cena: &mut VecScene) -> Regua {
    let cantos = cantos();
    let mut p = VecPath {
        verts: cantos.iter().copied().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    };
    p.fill = Some(Paint::Solid(Rgba8::new(214, 168, 92, 255)));
    let id = cena.push_path(p);
    #[expect(clippy::cast_possible_truncation, reason = "20 cantos")]
    let n = cantos.len() as u32;
    let v = |i: u32| PontoCad::Vert(id, VertId(i % n + 1));
    let lado = |i: u32| SegCad(v(i), v(i + 1));
    let mut regras: Vec<RegraCad> = (0..n)
        .map(|i| {
            RegraCad::nova(if i % 2 == 0 {
                Lei::Vertical(lado(i))
            } else {
                Lei::Horizontal(lado(i))
            })
        })
        .collect();
    regras.extend((1..DENTES).map(|k| RegraCad::nova(Lei::Igual(lado(1), lado(4 * k + 1)))));
    regras.extend((1..DENTES - 1).map(|k| RegraCad::nova(Lei::Igual(lado(3), lado(4 * k + 3)))));
    regras.push(RegraCad::nova(Lei::Fixo(v(0), cantos[0])));
    // O chão: as folgas todas ao nível da primeira, e ela presa — o dente estica para CIMA, não afunda o pé.
    regras.push(RegraCad::nova(Lei::Fixo(v(3), cantos[3])));
    regras.extend(
        (1..DENTES - 1).map(|k| RegraCad::nova(Lei::Horizontal(SegCad(v(3), v(4 * k + 3))))),
    );
    // A altura do dente do meio: do pé (a folga à esquerda dele) ao topo.
    let meio = 4 * (DENTES / 2);
    regras.push(RegraCad::nova(Lei::Distancia(v(meio), v(meio + 1), ALTURA)));
    cena.regras_mut().extend(regras);
    cena.normaliza_ids();
    let medida = cena.regras().last().map_or(RegraId::NOVO, |r| r.id);
    Regua { path: id, medida }
}

/// As chaves da medida animada ([`MEDIDA`]) da regra `regra` na entidade `e` (a do path), a acelerar e a travar.
#[must_use]
pub fn chaves_da_medida(e: u64, regra: RegraId, fps: f64) -> Vec<Escrita> {
    MEDIDA
        .iter()
        .map(|&(q, v)| Escrita::Chave {
            e,
            p: PropKind::Field(chave(CAMPO, regra.0)),
            t: f64::from(q) / fps,
            v,
            interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        })
        .collect()
}

/// Semeia o documento se a cena 6 foi pedida (uma vez).
pub(crate) fn semear_se_pedida(sim: &mut SimWorld, cena: &mut VecScene) {
    if sim.world_mut().remove_resource::<Semear>().is_some() {
        let r = semear(cena);
        sim.world_mut().insert_resource(r);
    }
}

static FEITA: AtomicBool = AtomicBool::new(false);

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=6` e a entidade da régua já nascida: grava as chaves (um passo de undo),
/// escolhe a régua, mostra a timeline e toca em laço.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(6) || FEITA.load(Ordering::Relaxed) {
        return None;
    }
    let r = *sim.world().get_resource::<Regua>()?;
    let w = sim.world_mut();
    let mut q = w.query::<(bevy_ecs::entity::Entity, &ph2d_ecs::VecPathRef)>();
    let e = q.iter(w).find(|(_, p)| p.0 == r.path).map(|(e, _)| e)?;
    FEITA.store(true, Ordering::Relaxed);
    let fps = tl.doc.fps_display;
    crate::chaves::gravar(
        tl,
        &chaves_da_medida(e.to_bits(), r.medida, fps),
        false,
        0.0,
        true,
    );
    hero.set_panel_visible("timeline", true);
    hero.gizmo.selection = Some(e.to_bits());
    let fim = f64::from(MEDIDA[MEDIDA.len() - 1].0) / fps;
    for keys in [true, false] {
        tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
    }
    ph.set_loop(0.0, fim);
    ph.seek_frame(0, fps);
    ph.play();
    Some("[constraint-smoke] =6 a altura do dente gravada; a tocar".to_owned())
}
