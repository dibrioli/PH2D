//! **A cena do smoke «o menu da linha de GRUPO»** (F27, bloco D) — `PH2D_AUTOKEY_SMOKE=31`. Um
//! retângulo com a cor do preenchimento animada vermelho → azul em `0..1 s` (os 4 canais r, g, b, a:
//! UMA linha «Fill» na dope-sheet) e a opacidade `40 → 100 %` (uma faixa sozinha, fora do grupo). O
//! relógio pára a meio. O dono clica com o botão direito no NOME da linha «Fill»: Delete Track apaga a
//! cor inteira e deixa a opacidade; Post → Loop faz a cor repetir depois de `1 s` e a opacidade não.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{ShapeKind, VecScene};

use crate::autokey_smoke::avanca as go;
use crate::state::VecState;

const MEIO: f64 = 0.5;
/// Vermelho e azul, `0..=255` (r, g, b, a).
const DE: [f32; 4] = [220.0, 40.0, 40.0, 255.0];
const PARA: [f32; 4] = [40.0, 80.0, 230.0, 255.0];

fn cor(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(
        VETOR,
        v::TINTA_PREENCH + v::OFF_SOLIDA + c,
        0,
        0,
    ))
}

fn key(tl: &mut TimelineState, bits: u64, prop: PropKind, t: f64, x: f32) {
    tl.doc.upsert_key(
        bits,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(x),
        Interp::Linear,
    );
}

pub(crate) fn step(
    nivel: u8,
    vec: &mut VecState,
    scene: &mut VecScene,
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
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [0.0, 0.0], 0.6);
            go(3);
            None
        }
        3 => {
            let (&id, &bits) = vec.entities.iter().next()?;
            for c in 0..4 {
                key(tl, bits, cor(c), 0.0, DE[c as usize]);
                key(tl, bits, cor(c), 1.0, PARA[c as usize]);
            }
            key(tl, bits, PropKind::Opacity, 0.0, 0.4);
            key(tl, bits, PropKind::Opacity, 1.0, 1.0);
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            ph.pause();
            ph.seek(MEIO);
            go(4);
            let faixas = tl
                .doc
                .bindings()
                .iter()
                .filter(|b| b.entity == bits)
                .count();
            Some(format!(
                "[autokey-smoke] =31 forma {id} (entidade {bits:#x}): Fill vermelho→azul (4 canais) e \
                 Opacity 40→100 % em 0..1 s · {faixas} faixas (tem de ser 5) · relógio parado em {MEIO} s"
            ))
        }
        _ => None,
    }
}
