//! **A cena do smoke «o losango em todo campo»** (F24c, plano 35 §5) — `PH2D_AUTOKEY_SMOKE=5` monta-a
//! para o dono mexer à mão; `=6` mexe sozinha e a sonda mede os números da mensagem.
//!
//! A estrela livre da `=3`, agora com o preenchimento em DEGRADÊ linear (laranja → roxo), uma
//! segunda entrada na pilha de pinturas (um traço amarelo, ABERTA no painel), o efeito Contour
//! (3 anéis até ao branco), o Inspector à vista, o Auto Key ligado e o relógio no quadro 12. O
//! `=6`, no quadro 12: a opacidade da entrada a 40 %, o Contour a 6 anéis, o degradê na vertical e a
//! estrela rodada; no quadro 24: a entrada de volta a 100 %, o Contour a 3, a rotação a zero; depois
//! pára no quadro 18 e diz o que vê. O `=7` é o `=6` com o Inspector à frente no fim (a foto dos
//! losangos do Transform).

use ph2d_core::Playhead;
use ph2d_ecs::{Entity, SimWorld, Transform, VecContour};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::paint_stack::{PaintEntry, PaintKind};
use ph2d_vec_scene::{
    GradientStop, Opacity, Paint, Rgba8, ShapeKind, StrokeSpec, VecPath, VecScene,
};

use crate::autokey_smoke::avanca as go;
use crate::state::VecState;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const LARANJA: Rgba8 = Rgba8::new(255, 140, 0, 255);
const ROXO: Rgba8 = Rgba8::new(130, 40, 200, 255);
const AZUL: Rgba8 = Rgba8::new(40, 90, 230, 255);
const AMARELO: Rgba8 = Rgba8::new(250, 210, 40, 255);
/// A meia largura do degradê (a estrela tem raio 0,9).
const MEIA: f64 = 0.9;
/// A rotação do quadro 12, em radianos.
const GIRO: f32 = 0.3;

#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    nivel: u8,
    // `0` = o dono mexe · `1` = mexe sozinha · `2` = mexe sozinha e acaba com o Inspector à frente.
    modo: u8,
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
            None
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, [0.0, 0.0], 1.8);
            go(3);
            None
        }
        3 => {
            let (&id, &bits) = vec.entities.iter().next()?;
            crate::shape_live::drop_shape_params(sim, &vec.entities, &[id]);
            let p = scene.path_mut(id)?;
            p.fill = Some(Paint::Linear {
                stops: vec![
                    GradientStop::new(0.0, LARANJA),
                    GradientStop::new(1.0, ROXO),
                ],
                start: [-MEIA, 0.0],
                end: [MEIA, 0.0],
            });
            p.stroke = Some(StrokeSpec::new(AZUL, 0.04));
            p.paints = vec![PaintEntry::new(PaintKind::Stroke(StrokeSpec::new(
                AMARELO, 0.1,
            )))];
            crate::contour_live::arm(sim, &vec.entities, &[id], 1.8);
            if let Some(mut ct) = sim
                .world_mut()
                .get_mut::<VecContour>(Entity::from_bits(bits))
            {
                ct.steps = 3;
            }
            if ph2d_panel_vector::state::open_layer_index() != Some(0) {
                ph2d_panel_vector::state::toggle_open_layer(0);
            }
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            tl.flags.auto_key = true;
            ph.pause();
            ph.seek(QUADRO_12);
            go(4);
            Some(format!(
                "[autokey-smoke] estrela {id} (entidade {bits:#x}): degradê laranja→roxo, traço azul, \
                 entrada 1 da pilha = traço amarelo (aberta), Contour 3 anéis; quadro 12; Auto Key LIGADO"
            ))
        }
        s @ 4..=80 if modo > 0 => {
            go(s + 1);
            let (&id, &bits) = vec.entities.iter().next()?;
            let e = Entity::from_bits(bits);
            match s {
                20 => {
                    scene.path_mut(id)?.paints.first_mut()?.opacity = Opacity::new(0.4);
                    Some("[autokey-smoke] q12: entrada 1 ← 40 %".to_string())
                }
                25 => {
                    sim.world_mut().get_mut::<VecContour>(e)?.steps = 6;
                    Some("[autokey-smoke] q12: Contour ← 6 anéis".to_string())
                }
                30 => {
                    if let Some(Paint::Linear { start, end, .. }) = &mut scene.path_mut(id)?.fill {
                        *start = [0.0, -MEIA];
                        *end = [0.0, MEIA];
                    }
                    Some("[autokey-smoke] q12: degradê ← vertical".to_string())
                }
                35 => {
                    sim.world_mut().get_mut::<Transform>(e)?.rotation += GIRO;
                    Some(format!(
                        "[autokey-smoke] q12: rotação ← +{GIRO} · {}",
                        trilhas(tl, bits)
                    ))
                }
                40 => {
                    ph.seek(QUADRO_24);
                    None
                }
                50 => {
                    scene.path_mut(id)?.paints.first_mut()?.opacity = Opacity::new(1.0);
                    sim.world_mut().get_mut::<VecContour>(e)?.steps = 3;
                    sim.world_mut().get_mut::<Transform>(e)?.rotation = 0.0;
                    Some(format!(
                        "[autokey-smoke] q24: entrada 1 ← 100 %, Contour ← 3, rotação ← 0 · {}",
                        trilhas(tl, bits)
                    ))
                }
                55 => {
                    ph.seek(QUADRO_18);
                    None
                }
                65 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · {}",
                    medir(scene.path(id)?, sim, e),
                    trilhas(tl, bits)
                )),
                // A aba da frente é o topo da ordem z — o que o clique na aba escreve.
                70 if modo == 2 => {
                    hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
                    None
                }
                80 => Some(format!("[autokey-smoke] fim · {}", trilhas(tl, bits))),
                _ => None,
            }
        }
        _ => None,
    }
}

/// O que a tela desenha: a opacidade da entrada, os anéis, o degradê e a rotação.
fn medir(p: &VecPath, sim: &SimWorld, e: Entity) -> String {
    let op = p.paints.first().map_or(-1.0, |x| x.opacity.get());
    let aneis = sim.world().get::<VecContour>(e).map_or(0, |c| c.steps);
    let fim = match &p.fill {
        Some(Paint::Linear { end, .. }) => format!("({:.2}, {:.2})", end[0], end[1]),
        outro => format!("{outro:?}"),
    };
    let rot = sim.world().get::<Transform>(e).map_or(0.0, |t| t.rotation);
    format!("entrada 1 {op:.3} · anéis {aneis} · fim do degradê {fim} · rotação {rot:.4}")
}

/// Quantas trilhas o objecto tem, por grupo, e quantas chaves cada uma.
pub(crate) fn trilhas(tl: &TimelineState, bits: u64) -> String {
    let mut por_grupo: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for b in tl.doc.bindings().iter().filter(|b| b.entity == bits) {
        let nome = match b.prop {
            PropKind::Field(k) => {
                let r = k.info().map_or("?", |i| i.rotulo);
                format!("{}[{}]", r.trim_start_matches("canal.vetor."), k.i)
            }
            p => format!("{p:?}"),
        };
        let n = tl
            .doc
            .active_clip()
            .track(b.target)
            .map_or(0, |t| t.keys().len());
        let e = por_grupo.entry(nome).or_default();
        e.0 += 1;
        e.1 = e.1.max(n);
    }
    let partes: Vec<String> = por_grupo
        .iter()
        .map(|(g, (n, ch))| format!("{g} {n}×{ch}ch"))
        .collect();
    format!("trilhas: {}", partes.join(" · "))
}
