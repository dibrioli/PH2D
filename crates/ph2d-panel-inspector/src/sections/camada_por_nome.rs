//! ⭐ **A camada pelo NOME fora do cartão** (dono 07/11, fecho da GEF02): a face clássica (`PH2D_CARDS=0`) e o
//! Ray Sensor diziam só o número. As duas passam à MESMA lista suspensa do cartão do collider (os ids de cada um,
//! a mesma edição; os oito chips «0»–«7» não chegavam às camadas 8–31; dono 08/11: o Ray Sensor com os mesmos
//! campos).

use super::physics_cartao::Tinta;
use super::physics_colide::{DO_COLLIDER, Quem, chip, lista};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A linha «Layer» da face clássica da física: a [`linha_da_camada`] do collider.
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
    let seccao = Seccao::medida(text_system, 1, &[rotulo]);
    let mut t = Tinta {
        scene,
        text: text_system,
        theme,
        hit: hit_index,
        store,
        x,
        w,
    };
    linha_da_camada(&mut t, y, &DO_COLLIDER, camada, Rotulo::from(rotulo), seccao)
}

/// A linha «Layer» de `quem`: rótulo, lista suspensa com o nome e o contorno da cor, e o losango
/// ([`chip`]). Estreita, o nome sobe por cima do chip.
pub(super) fn linha_da_camada(
    t: &mut Tinta<'_>,
    y: f32,
    quem: &'static Quem,
    camada: u8,
    rotulo: Rotulo<'_>,
    seccao: Seccao,
) -> f32 {
    let dd = lista(quem, camada);
    // No cartão o rótulo vai em cima (a linha do cartão); fora dele, estreita, o nome sobe por cima do chip.
    let ao_lado = crate::cards::active() || chip_cabe_ao_lado(t.text, &dd, t.x, t.w, y, seccao);
    let (control, dot, fim) = if ao_lado {
        let linha = ph2d_editor_core::property_row::paint_label_row(
            t.scene, t.text, t.theme, t.x, t.w, y, ROW_H_PX, rotulo.texto, seccao,
        );
        (
            linha.control,
            linha.dot,
            linha.fim(ph2d_tokens::row_pitch_px()),
        )
    } else {
        nome_por_cima(t.scene, t.text, t.theme, t.x, t.w, y, rotulo.texto)
    };
    chip(t, (control, dot), quem, camada, rotulo);
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
