//! **A cena do smoke «o menu de misturas e a palavra coincidem»** (F28, [plano](../../../docs/UI_New_and_Simple/F28_plano.md))
//! — `PH2D_AUTOKEY_SMOKE=34`. Um retângulo com a MISTURA chaveada uma vez por linha do menu Blend
//! (as 19 que o Vetor desenha, de 0,2 em 0,2 s, dentro dos 4 s da cena) e uma 20.ª chave pedida como «Vivid Light»
//! (o código 12, que o Vetor não desenha): ela guarda a escolha mais próxima que existe, «Hard Light».
//! O relógio pára em 0,8 s, na 5.ª chave — a linha 4 do menu, que é o código 5 (Lighten): foi a partir
//! daqui que o censo viu linha e código a separarem-se.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{BlendMode, ShapeKind, VecScene};

use crate::autokey_smoke::avanca as go;
use crate::state::VecState;

const PASSO: f64 = 0.2;
const PARADO: f64 = 0.8;
/// Pedido: Vivid Light — o Vello não o exprime.
const FORA_DO_MENU: f32 = 12.0;

const MISTURA: PropKind = PropKind::Field(FieldKey::new(VETOR, v::MISTURA, 0, 0));

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
            let modos: Vec<BlendMode> = ph2d_vec_render::blend::offered().collect();
            let mut t = 0.0;
            for m in &modos {
                key(tl, bits, t, f32::from(m.to_u8()));
                t += PASSO;
            }
            key(tl, bits, t, FORA_DO_MENU);
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            ph.pause();
            ph.seek(PARADO);
            go(4);
            Some(relatorio(tl, bits, &modos))
        }
        _ => None,
    }
}

fn key(tl: &mut TimelineState, bits: u64, t: f64, x: f32) {
    tl.doc.upsert_key(
        bits,
        MISTURA,
        RationalTime::from_seconds(t),
        AnimValue::Float(x),
        Interp::Hold,
    );
}

/// O log que a foto confere: cada chave, a linha do menu que a escreveu e a palavra que ela diz.
fn relatorio(tl: &mut TimelineState, bits: u64, modos: &[BlendMode]) -> String {
    let target = tl.doc.bind(bits, MISTURA);
    let guardados: Vec<f32> = tl
        .doc
        .active_clip()
        .track(target)
        .map(|tr| {
            tr.keys()
                .iter()
                .filter_map(|k| match k.value {
                    AnimValue::Float(x) => Some(x),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let palavra = |x: f32| {
        BlendMode::from_u8(x as u8)
            .label_key()
            .trim_start_matches("blend.mode.")
            .to_owned()
    };
    let iguais = modos
        .iter()
        .zip(&guardados)
        .filter(|(m, x)| f32::from(m.to_u8()) == **x)
        .count();
    let ultimo = guardados.last().copied().unwrap_or(-1.0);
    format!(
        "[autokey-smoke] =34 entidade {bits:#x}: {} chaves de Blend (tem de ser 20) · linhas do menu = \
         código guardado em {iguais}/{} · pedido {FORA_DO_MENU} (vivid_light) guardou {ultimo} ({}) · \
         relógio em {PARADO} s = linha 4 = {}",
        guardados.len(),
        modos.len(),
        palavra(ultimo),
        palavra(guardados.get(4).copied().unwrap_or(0.0)),
    )
}
