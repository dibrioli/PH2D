//! ⭐ **A camada pelo NOME fora do cartão** (dono 07/11, fecho da GEF02): a face clássica (`PH2D_CARDS=0`) e o
//! Ray Sensor diziam só o número. A face clássica passa à MESMA lista suspensa do cartão (os mesmos ids
//! `INSP_PHYS_LAYER`, a mesma edição; os oito chips «0»–«7» não chegavam às camadas 8–31); o Ray Sensor fica com
//! o campo de número (arrastar, escrever, Auto Key) e ganha ao lado o nome, no contorno da cor da camada.

use super::physics_colide::lista_da_camada;
use crate::camadas_fisica as camadas;
use crate::ids;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{frame_radius, paint_layer_outline, paint_text_elided, resolve};
use ph2d_editor_core::widget::paint_dropdown_chip;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A linha «Layer» da face clássica: rótulo, lista suspensa com o nome e o contorno da cor, e o losango.
#[allow(clippy::too_many_arguments)]
pub(super) fn linha_classica(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    camada: u8,
) -> f32 {
    let rotulo = ph2d_i18n::tr("component.field.collider.4");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        rotulo,
        seccao,
    );
    hit_index.register(ids::INSP_PHYS_LAYER_PICK, linha.control);
    let aberta = matches!(
        store.get(ids::INSP_PHYS_LAYER_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = lista_da_camada(camada)
        .open(aberta)
        .visual(store.dropdown_visual(ids::INSP_PHYS_LAYER_PICK));
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    let raio = frame_radius(theme, Radius::Sm.px());
    paint_layer_outline(
        scene,
        theme,
        ids::INSP_PHYS_LAYER_PICK,
        usize::from(camada),
        linha.control,
        raio,
        true,
    );
    if aberta {
        crate::state_popovers::set_pending_phys_layer_dd(Some((camada, linha.control)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        scene,
        theme,
        linha.dot,
        &ids::INSP_PHYS_LAYER,
    ) {
        hit_index.register(alvo, linha.dot);
    }
    linha.fim(ph2d_tokens::row_pitch_px())
}

/// Quanto do controlo do Ray Sensor (`controlo_w`) o nome da camada leva à direita: o nome mais um
/// respiro, sem deixar a caixa do número abaixo do piso dela.
#[must_use]
pub(super) fn largura_do_nome(text_system: &mut TextSystem, controlo_w: f32, camada: usize) -> f32 {
    let texto = camadas::rotulo(camada);
    let pede = text_system.prefix_width(&texto, TypeToken::Sm.px()).ceil() + Spacing::Sm.px() * 2.0;
    pede.min((controlo_w - ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX).max(0.0))
}

/// O nome da camada do Ray Sensor em `caixa`, no contorno da cor da camada (o mesmo idioma do seletor Layer).
/// O contorno regista-se pelo id do campo (`INSP_RAY_LAYER`): é a caixa do NOME dele.
pub(super) fn nome_do_raio(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    caixa: Rect,
    j: usize,
) {
    let texto = camadas::rotulo(j);
    let fonte = TypeToken::Sm.px();
    let folga = Spacing::Sm.px();
    paint_text_elided(
        text_system,
        scene,
        &texto,
        caixa.x + folga,
        caixa.y + ph2d_editor_core::paint::folga_do_texto(caixa.h, fonte),
        fonte,
        (caixa.w - folga * 2.0).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
    let raio = frame_radius(theme, Radius::Sm.px());
    paint_layer_outline(scene, theme, ids::INSP_RAY_LAYER, j, caixa, raio, true);
}
