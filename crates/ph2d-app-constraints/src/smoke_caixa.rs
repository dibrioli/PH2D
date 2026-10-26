//! **A cena 7 — «Caixa com tampa»** (`PH2D_CONSTRAINT_SMOKE=7`, `docs/BlenderConstraints/04_W4_cad.md` §3.7): poucas
//! regras, cada uma à vista. O CORPO é um retângulo (dois lados horizontais, dois verticais, o canto de baixo à
//! esquerda preso, a largura e a altura cotadas); a TAMPA é um traço preso na dobradiça (coincidente com o canto de
//! cima à esquerda), do comprimento do topo (igual) e aberta por um ÂNGULO animado ([`ANGULO`]). Abre no modo Edit
//! com a tool Regras na mão: os selos à vista, e o Play abre e fecha a tampa.

use std::sync::atomic::{AtomicU8, Ordering};

use bevy_ecs::prelude::Resource;
use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{Easing, EasingFamily, EasingMode, Interp, PropKind, TimelineState};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, RegraId, SegCad};
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex, VertId};

use crate::chaves::Escrita;

/// A largura e a altura do corpo (unidades do mundo).
pub const LARGURA: f64 = 2.0;
pub const ALTURA: f64 = 1.2;
/// As chaves do ângulo da tampa: `(quadro, graus)` — entreaberta, aberta, entreaberta.
pub const ANGULO: [(u32, f32); 3] = [(0, 10.0), (24, 110.0), (48, 10.0)];

/// A cena 7 foi pedida e o documento ainda não a tem.
#[derive(Resource)]
pub(crate) struct Semear;

/// O que se semeou.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Caixa {
    pub corpo: VecPathId,
    pub tampa: VecPathId,
    pub angulo: RegraId,
    /// A lei do ângulo (a chave do canal dela).
    pub lei: Lei,
}

/// Põe a caixa, a tampa e as dez regras no documento.
pub fn semear(cena: &mut VecScene) -> Caixa {
    let (x0, y0) = (-LARGURA / 2.0, -ALTURA / 2.0);
    let corpo = cena.push_path(VecPath {
        verts: [
            [x0, y0],
            [x0 + LARGURA, y0],
            [x0 + LARGURA, y0 + ALTURA],
            [x0, y0 + ALTURA],
        ]
        .into_iter()
        .map(VecVertex::corner)
        .collect(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(196, 150, 96, 255))),
        ..VecPath::default()
    });
    let a = f64::from(ANGULO[0].1).to_radians();
    let dobradica = [x0, y0 + ALTURA];
    let tampa = cena.push_path(VecPath {
        verts: vec![
            VecVertex::corner(dobradica),
            VecVertex::corner([
                dobradica[0] + LARGURA * a.cos(),
                dobradica[1] + LARGURA * a.sin(),
            ]),
        ],
        closed: false,
        stroke: Some(StrokeSpec::new(Rgba8::new(120, 80, 40, 255), 0.06)),
        ..VecPath::default()
    });
    let c = |i: u32| PontoCad::Vert(corpo, VertId(i));
    let t = |i: u32| PontoCad::Vert(tampa, VertId(i));
    let topo = SegCad(c(4), c(3));
    let lado_da_tampa = SegCad(t(1), t(2));
    let regras = [
        Lei::Horizontal(SegCad(c(1), c(2))),
        Lei::Vertical(SegCad(c(2), c(3))),
        Lei::Horizontal(SegCad(c(3), c(4))),
        Lei::Vertical(SegCad(c(4), c(1))),
        Lei::Fixo(c(1), [x0, y0]),
        Lei::Distancia(c(1), c(2), LARGURA),
        Lei::Distancia(c(2), c(3), ALTURA),
        Lei::Coincidente(t(1), c(4)),
        Lei::Igual(lado_da_tampa, topo),
        Lei::Angulo(topo, lado_da_tampa, a),
    ];
    cena.regras_mut()
        .extend(regras.into_iter().map(RegraCad::nova));
    cena.normaliza_ids();
    let (angulo, lei) = cena
        .regras()
        .last()
        .map_or((RegraId::NOVO, Lei::Angulo(topo, lado_da_tampa, a)), |r| {
            (r.id, r.lei)
        });
    Caixa {
        corpo,
        tampa,
        angulo,
        lei,
    }
}

/// As chaves do ângulo ([`ANGULO`]) da regra `regra` na entidade `e` (a do corpo, o dono da regra).
#[must_use]
pub fn chaves_do_angulo(e: u64, lei: &Lei, regra: RegraId, fps: f64) -> Vec<Escrita> {
    let Some(k) = ph2d_vec_entities::medida::chave_da(lei, regra) else {
        return Vec::new();
    };
    ANGULO
        .iter()
        .map(|&(q, v)| Escrita::Chave {
            e,
            p: PropKind::Field(k),
            t: f64::from(q) / fps,
            v,
            interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        })
        .collect()
}

/// Semeia o documento se a cena 7 foi pedida (uma vez).
pub(crate) fn semear_se_pedida(sim: &mut SimWorld, cena: &mut VecScene) {
    if sim.world_mut().remove_resource::<Semear>().is_some() {
        let c = semear(cena);
        sim.world_mut().insert_resource(c);
    }
}

/// 0 = por fazer · 1 = o Edit pedido · 2 = feita.
static FASE: AtomicU8 = AtomicU8::new(0);

/// Com `PH2D_CONSTRAINT_SMOKE=7` e a entidade do corpo nascida: escolhe o corpo e pede o Edit; no quadro seguinte
/// (o vetor já na mão) a tool Regras, as chaves do ângulo (um passo de undo), a timeline à vista e o Play em laço.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(7) {
        return None;
    }
    let c = *sim.world().get_resource::<Caixa>()?;
    let w = sim.world_mut();
    let mut q = w.query::<(bevy_ecs::entity::Entity, &ph2d_ecs::VecPathRef)>();
    let e = q.iter(w).find(|(_, p)| p.0 == c.corpo).map(|(e, _)| e)?;
    match FASE.load(Ordering::Relaxed) {
        0 => {
            FASE.store(1, Ordering::Relaxed);
            hero.gizmo.replace_selection(Some(e.to_bits()));
            hero.bus.push(EditorAction::ObjectMode(ModeRequest::Enter(
                ObjectMode::Edit,
            )));
            None
        }
        1 => {
            FASE.store(2, Ordering::Relaxed);
            hero.bus.push(EditorAction::ToolPanelEvent(
                ph2d_editor_core::tool::PanelEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_RULES),
            ));
            let fps = tl.doc.fps_display;
            crate::chaves::gravar(
                tl,
                &chaves_do_angulo(e.to_bits(), &c.lei, c.angulo, fps),
                false,
                0.0,
                true,
            );
            hero.set_panel_visible("timeline", true);
            let fim = f64::from(ANGULO[ANGULO.len() - 1].0) / fps;
            for keys in [true, false] {
                tl.doc.set_active_loop_for(keys, Some((0.0, fim)));
            }
            ph.set_loop(0.0, fim);
            ph.seek_frame(0, fps);
            ph.play();
            Some(
                "[constraint-smoke] =7 a caixa com tampa: Edit + Regras, a tampa a abrir"
                    .to_owned(),
            )
        }
        _ => None,
    }
}
