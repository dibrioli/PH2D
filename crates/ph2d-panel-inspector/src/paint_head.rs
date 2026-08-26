//! **O CABEÇALHO do Inspector** — título, subtítulo, botão de fechar e o divisor.
//!
//! ⚠️ **Irmão de [`super::paint`] por CAP de FUNÇÃO.** O `paint_inspector` é o orquestrador das
//! doze seções e vive numa tolerância que **só desce**; isto não é orquestração — é chrome de
//! painel, e não olha para seção nenhuma. Saiu quando a §11 Animation nasceu (2026-08-23).
//!
//! ⚠️ **O subtítulo é o TAMANHO da sprite na unidade do projeto**, e ele é derivado do snapshot
//! na hora: guardá-lo formatado faria a troca de unidade (m ↔ px) precisar de um segundo sítio
//! para se lembrar.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::state::{current_display_unit, current_inspector_sprite, current_pixels_per_meter};

/// Pinta o cabeçalho e devolve o `y` onde o CORPO começa (`content_top`).
pub(crate) fn paint_panel_head(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &ph2d_editor_core::interaction::WidgetStore,
) -> f32 {
    // Canonical panel title — reserve right slot for the X close
    // button (UI canon post-2026-05-24: every panel except Hierarchy
    // carries close X; vide `docs/UI_Padrao/components/panel_chrome.md`).
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    // ⚠️ **A reserva é a do PAR** (close + add), e não a do close sozinho: o título tem de parar
    // antes dos DOIS, senão ele passa por baixo do `+`. É a mesma reserva que a Hierarquia usa.
    let title_size = paint_panel_title(
        rect,
        "Inspector",
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_ADD_RESERVE,
        scene,
        text_system,
        theme,
    );
    // X close → toggles inspector visibility (apply_event handler).
    ph2d_editor_core::widget::panel_chrome::paint_panel_close_button(
        rect,
        ids::INSP_CLOSE,
        hit_index,
        scene,
        theme,
    );
    // ⭐ **O `+` — a UMA porta de anexar um componente** (ADR-0166 / F3). Ele fica à esquerda do
    // X, com a mesma geometria e o mesmo look de ícone-fantasma do `Add` da Hierarquia (é o
    // precedente literal desta casa para um par close+add num cabeçalho).
    let add_size = 30.0_f32; // LITERAL-PX-OK: quadrado do botão de cabeçalho — o mesmo da Hierarquia
    let add_rect = Rect::new(
        // ⚠️ Deslocado do X pela largura dele: os dois partilham a reserva do par.
        rect.x + rect.w - PANEL_HEAD_PAD - add_size - Spacing::Xl.px(),
        title_y - 2.0,
        add_size,
        add_size,
    );
    hit_index.register(ids::INSP_ADD_COMPONENT, add_rect);
    let add_btn = ph2d_editor_core::widget::Button::new(ids::INSP_ADD_COMPONENT, "Add Component")
        .icon_only(ph2d_editor_core::icons::IconId::Add)
        .visual(store.button_visual(ids::INSP_ADD_COMPONENT));
    ph2d_editor_core::widget::paint_button(&add_btn, add_rect, scene, text_system, theme);

    let sprite_for_header = current_inspector_sprite();
    let subtitle_owned;
    let subtitle: &str = match sprite_for_header.as_ref() {
        Some(info) => {
            let unit = current_display_unit();
            let ppm = current_pixels_per_meter();
            let w = unit.from_meters(info.world_size[0], ppm);
            let h = unit.from_meters(info.world_size[1], ppm);
            subtitle_owned = match unit {
                ph2d_editor_core::project::DisplayUnit::Meters => {
                    format!("{:.3} × {:.3} {}", w, h, unit.suffix())
                }
                ph2d_editor_core::project::DisplayUnit::Pixels => {
                    format!("{:.0} × {:.0} {}", w, h, unit.suffix())
                }
            };
            subtitle_owned.as_str()
        }
        None => "",
    };
    paint_text(
        text_system,
        scene,
        subtitle,
        rect.x + PANEL_HEAD_PAD,
        title_y + title_size + Spacing::Xs.px(),
        TypeToken::Sm.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    let div_y = title_y + TypeToken::Md.px() + TypeToken::Sm.px() + Spacing::Xl.px();
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    div_y + Spacing::Sm.px()
}
