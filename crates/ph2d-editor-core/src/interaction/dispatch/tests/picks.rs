//! ⭐ **O contador de ESCOLHAS de cor do seletor** (plano 32,
//! `docs/UI_New_and_Simple/vetor_nova_ui/32_plano_F21_o_misto_completo.md`): escolher uma cor
//! (roda, R/G/B, amostra) conta; mexer na alfa e re-semear o seletor não contam.

use super::*;
use crate::interaction::BlenderHitKind;
use crate::widget::{ChannelMode, InterpolationMode};
use ph2d_tokens::ColorValue;

const PICKER: NodeId = NodeId(100);

fn loja(alvos: &[(NodeId, BlenderHitKind)]) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(8);
    store.register(
        PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(200, 30, 30, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 1.0,
            harmony: crate::widget::Harmony::None,
            picks: 0,
        },
    );
    let mut hits = HitIndex::new();
    for (k, (id, kind)) in alvos.iter().enumerate() {
        store.register(
            *id,
            InteractiveState::BlenderHit {
                parent: PICKER,
                kind: *kind,
            },
        );
        hits.register(*id, Rect::new(0.0, 200.0 * k as f32, 100.0, 100.0));
    }
    (store, hits)
}

fn clica(store: &mut WidgetStore, hits: &HitIndex, k: usize) {
    let arena = Bump::new();
    let y = 200.0 * k as f32 + 50.0;
    let _ = dispatch_pointer(store, hits, pointer(PointerKind::Down, 50.0, y), &arena);
    let _ = dispatch_pointer(store, hits, pointer(PointerKind::Up, 50.0, y), &arena);
}

#[test]
fn escolher_uma_cor_conta_e_mexer_na_alfa_nao() {
    let (mut store, hits) = loja(&[
        (NodeId(101), BlenderHitKind::Wheel),
        (NodeId(102), BlenderHitKind::ChannelSlider(0)),
        (NodeId(103), BlenderHitKind::ChannelSlider(3)),
    ]);
    assert_eq!(store.colour_picks(PICKER), 0);
    clica(&mut store, &hits, 0);
    assert_eq!(store.colour_picks(PICKER), 1, "a roda é uma escolha de cor");
    clica(&mut store, &hits, 1);
    assert_eq!(store.colour_picks(PICKER), 2, "o R é uma escolha de cor");
    clica(&mut store, &hits, 2);
    assert_eq!(
        store.colour_picks(PICKER),
        2,
        "a alfa NÃO é uma escolha de cor"
    );
    // Re-semear o seletor (a ponte, ao mudar de seleção) não é o artista a escolher.
    store.set_blender_value(PICKER, ColorValue::from_rgba8(1, 2, 3, 255));
    assert_eq!(store.colour_picks(PICKER), 2);
    assert_eq!(
        store.colour_picks(NodeId(101)),
        0,
        "quem não é seletor responde 0"
    );
}
