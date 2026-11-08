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
    let aberta = matches!(
        store.get(ids::INSP_PHYS_LAYER_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = lista_da_camada(camada)
        .open(aberta)
        .visual(store.dropdown_visual(ids::INSP_PHYS_LAYER_PICK));
    let (control, dot, fim) = if chip_cabe_ao_lado(text_system, &dd, x, w, y, seccao) {
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
        (
            linha.control,
            linha.dot,
            linha.fim(ph2d_tokens::row_pitch_px()),
        )
    } else {
        nome_por_cima(scene, text_system, theme, x, w, y, rotulo)
    };
    hit_index.register(ids::INSP_PHYS_LAYER_PICK, control);
    paint_dropdown_chip(&dd, control, scene, text_system, theme);
    let raio = frame_radius(theme, Radius::Sm.px());
    paint_layer_outline(
        scene,
        theme,
        ids::INSP_PHYS_LAYER_PICK,
        usize::from(camada),
        control,
        raio,
        true,
    );
    if aberta {
        crate::state_popovers::set_pending_phys_layer_dd(Some((camada, control)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        scene,
        theme,
        dot,
        &ids::INSP_PHYS_LAYER,
    ) {
        hit_index.register(alvo, dot);
    }
    fim
}

/// O chip da opção MAIS LARGA da lista (na fonte do chip, pela inversa do orçamento) cabe na caixa ao
/// lado do nome? Medido pela lista, nunca pela escolhida: ao trocar de camada a linha não salta.
fn chip_cabe_ao_lado(
    text_system: &mut TextSystem,
    dd: &ph2d_editor_core::widget::Dropdown<u8>,
    x: f32,
    w: f32,
    y: f32,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> bool {
    let texto = dd
        .options
        .iter()
        .map(|o| text_system.prefix_width(&o.label, TypeToken::Base.px()))
        .fold(0.0_f32, f32::max);
    let caixa = ph2d_editor_core::property_row::caixa_do_controlo(x, w, y, seccao);
    ph2d_editor_core::widget::dropdown_chip_width_for(texto, ROW_H_PX) <= caixa.w
}

/// ⭐ O degrau estreito: o chip não cabe ao lado sem encolher a coluna partilhada do corpo ⇒ o nome
/// por cima e o chip a toda a largura por baixo — o mesmo idioma da [`paint_choice_row`] quando ela
/// refluiria (e o que os oito chips «0»–«7» de antes faziam). Devolve (caixa do chip, ponto, fim).
///
/// [`paint_choice_row`]: ph2d_editor_core::property_row::paint_choice_row
fn nome_por_cima(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
) -> (Rect, Rect, f32) {
    let fonte = TypeToken::Sm.px();
    ph2d_editor_core::paint::paint_text(
        text_system,
        scene,
        rotulo,
        x,
        y,
        fonte,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let (_, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, fonte);
    let gy = y + fonte + ph2d_editor_core::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX;
    let chip = Rect::new(x, gy, w, ROW_H_PX);
    (chip, dot, gy + ROW_H_PX + ph2d_tokens::control_gap_px())
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
