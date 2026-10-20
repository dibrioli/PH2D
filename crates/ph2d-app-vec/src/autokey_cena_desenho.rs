//! **A cena do smoke «o Auto Key grava o DESENHO»** (plano 35 §7) — `PH2D_AUTOKEY_SMOKE=3` monta-a
//! para o dono mexer à mão; `=4` mexe sozinha e a sonda mede os números da mensagem.
//!
//! Uma estrela vetorial LIVRE (a forma paramétrica vira caminho, para os pontos serem do artista),
//! laranja, traço azul tracejado, seleccionada, Auto Key ligado, o relógio parado no quadro 12. O
//! `=4`, no quadro 12: preenchimento verde, traço ×3, um canto arredondado, um ponto arrastado; no
//! quadro 24: preenchimento roxo, traço de volta; depois para no quadro 18 e diz o que vê.

use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{DashPattern, Paint, Rgba8, ShapeKind, StrokeSpec, VecPath, VecScene};

use crate::autokey_cena::avanca as go;
use crate::state::VecState;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const LARANJA: Rgba8 = Rgba8::new(255, 140, 0, 255);
const VERDE: Rgba8 = Rgba8::new(0, 200, 90, 255);
const ROXO: Rgba8 = Rgba8::new(130, 40, 200, 255);
const AZUL: Rgba8 = Rgba8::new(40, 90, 230, 255);

#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    nivel: u8,
    sozinha: bool,
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
            p.fill = Some(Paint::Solid(LARANJA));
            let mut traco = StrokeSpec::new(AZUL, 0.04);
            traco.dash = Some(DashPattern::single(0.12, 0.06));
            p.stroke = Some(traco);
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            tl.flags.auto_key = true;
            ph.pause();
            ph.seek(QUADRO_12);
            go(4);
            Some(format!(
                "[autokey-smoke] estrela {id} (entidade {bits:#x}) livre: laranja, traço azul 0,04 \
                 tracejado; relógio parado no quadro 12; Auto Key LIGADO"
            ))
        }
        s @ 4..=80 if sozinha => {
            go(s + 1);
            let (&id, &bits) = vec.entities.iter().next()?;
            let p = scene.path_mut(id)?;
            match s {
                20 => {
                    p.fill = Some(Paint::Solid(VERDE));
                    Some("[autokey-smoke] q12: preenchimento ← verde".to_string())
                }
                25 => {
                    p.stroke.as_mut()?.width = 0.14;
                    Some("[autokey-smoke] q12: traço ← 0,14".to_string())
                }
                30 => {
                    p.verts.get_mut(2)?.corner_radius = 0.15;
                    Some("[autokey-smoke] q12: canto do ponto 2 ← 0,15".to_string())
                }
                35 => {
                    let v = p.verts.get_mut(0)?;
                    for xy in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                        xy[0] += 0.4;
                    }
                    Some(format!(
                        "[autokey-smoke] q12: ponto 0 → +0,4 · {}",
                        trilhas(tl, bits)
                    ))
                }
                40 => {
                    ph.seek(QUADRO_24);
                    None
                }
                50 => {
                    p.fill = Some(Paint::Solid(ROXO));
                    p.stroke.as_mut()?.width = 0.04;
                    Some(format!(
                        "[autokey-smoke] q24: roxo, traço 0,04 · {}",
                        trilhas(tl, bits)
                    ))
                }
                55 => {
                    ph.seek(QUADRO_18);
                    None
                }
                65 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · {}",
                    medir(p),
                    trilhas(tl, bits)
                )),
                80 => Some(format!("[autokey-smoke] fim · {}", trilhas(tl, bits))),
                _ => None,
            }
        }
        _ => None,
    }
}

/// O que a tela desenha: a cor, o traço, o canto e o ponto 0.
fn medir(p: &VecPath) -> String {
    let cor = match &p.fill {
        Some(Paint::Solid(c)) => format!("({}, {}, {})", c.r, c.g, c.b),
        outro => format!("{outro:?}"),
    };
    let w = p.stroke.as_ref().map_or(0.0, |s| s.width);
    let canto = p.verts.get(2).map_or(0.0, |v| v.corner_radius);
    let x0 = p.verts.first().map_or(0.0, |v| v.anchor[0]);
    format!("cor {cor} · traço {w:.4} · canto {canto:.4} · ponto0.x {x0:.4}")
}

/// Quantas trilhas de campo o objecto tem, por grupo, e quantas chaves cada uma.
fn trilhas(tl: &TimelineState, bits: u64) -> String {
    let mut por_grupo: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
    for b in tl.doc.bindings().iter().filter(|b| b.entity == bits) {
        let PropKind::Field(k) = b.prop else { continue };
        let nome = k.info().map_or("?", |i| i.rotulo);
        let n = tl
            .doc
            .active_clip()
            .track(b.target)
            .map_or(0, |t| t.keys().len());
        let e = por_grupo
            .entry(nome.trim_start_matches("canal.vetor."))
            .or_default();
        e.0 += 1;
        e.1 = e.1.max(n);
    }
    let partes: Vec<String> = por_grupo
        .iter()
        .map(|(g, (n, ch))| format!("{g} {n}×{ch}ch"))
        .collect();
    format!("trilhas: {}", partes.join(" · "))
}
