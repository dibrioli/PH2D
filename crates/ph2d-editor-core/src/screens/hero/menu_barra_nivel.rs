//! ⭐ **Um nível da cascata de um menu da BARRA** (onda 8): o cartão da largura da maquete
//! (`chrome.menu-w`), linhas de `chrome.menu-row-h`, o risco curto entre grupos, e cada linha pintada
//! por [`super::menu_barra_linha::pinta`]. As mesmas linhas, o mesmo índice de acerto e a mesma marca
//! de «corrente» do pintor de sempre ([`super::context_menu_overlay`]); só a cara muda.

use super::context_menu_overlay::{anchor_y, clamp_to_viewport, pad_y, paint_theme_swatch};
use super::menu_barra_linha::{self as linha, Glifo, Linha};
use super::menu_row_mark::{MenuRow, contributed_row_is_current, id_is_currently_selected};
use super::menu_tons::tons_do_menu;
use crate::interaction::{ContextMenuRequest, HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::{Color, VectorScene};

/// O nível a pintar: o pedido, a linha dele com o submenu aberto, e o que um módulo lhe soma.
pub(super) struct Nivel<'a> {
    pub req: ContextMenuRequest,
    pub mae: Option<NodeId>,
    pub contrib: &'a [crate::widget::ToolRailEntry],
}

/// O que diz que uma linha é a escolha corrente.
type Estado<'a> = (
    &'a crate::project::ProjectSettings,
    &'a crate::motion::UiMotion,
    &'a crate::luz_edits::PreferenciaDaLuz,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn pinta(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (project, motion, luz): Estado<'_>,
    viewport: Rect,
    rows: &[MenuRow<'_>],
    n: Nivel<'_>,
) {
    let total_h = rows.iter().map(MenuRow::h_na_barra).sum::<f32>() + pad_y() * 2.0;
    let w = linha::largura(store, n.req.kind);
    let rect = clamp_to_viewport(
        n.req.x,
        anchor_y(store, n.req, total_h),
        w,
        total_h,
        viewport,
    );
    let superficie = Superficie::de(theme);
    superficie.moldura(scene, rect);
    let t = tons_do_menu(theme);
    if t.fundo != superficie.fundo() {
        // O fundo que o tema escolheu para o texto ler (ver `menu_tons`), dentro do filete.
        let dentro = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, rect.h - 2.0);
        let raio = (crate::paint::frame_radius(theme, Radius::Md.px()) - 1.0).max(0.0);
        fill_rounded_rect(scene, dentro, raio, t.fundo);
    }
    let row_x = rect.x + Spacing::Xs.px();
    let row_w = rect.w - Spacing::Xs.px() * 2.0;
    let mut y = rect.y + pad_y();
    for row in rows {
        let h = row.h_na_barra();
        let r = Rect::new(row_x, y, row_w, h);
        y += h;
        let MenuRow::Item(id, label, swatch) = row else {
            // O risco curto (maquete `.c15sep{margin:4px 6px}`): fora do índice de acerto.
            let inset = Spacing::Sm.px();
            fill_rounded_rect(
                scene,
                Rect::new(r.x + inset, r.y + (h - 1.0) * 0.5, r.w - inset * 2.0, 1.0),
                0.0,
                t.risco,
            );
            continue;
        };
        hit_index.register(*id, r);
        let corrente = id_is_currently_selected(*id, theme, store, project, motion, luz)
            || contributed_row_is_current(*id, n.contrib, store);
        let tema = super::theme_menu::tema_da_linha(*id, theme);
        let icone = linha::icone_da_linha(store, *id, n.contrib);
        let glifo = match (tema, swatch, icone) {
            (Some(tm), ..) => Glifo::Tema(tm),
            (None, Some(c), _) => Glifo::Amostra(*c),
            (None, None, Some(i)) => Glifo::Icone(i),
            _ if corrente => Glifo::Escolha,
            _ => Glifo::Nada,
        };
        let coluna = crate::widget::command_palette::shortcut_column(label);
        let (rotulo, atalho) = crate::widget::command_palette::split_shortcut(&coluna);
        let l = Linha {
            r,
            rotulo,
            atalho,
            glifo,
            abre_submenu: super::menu_cascade::opens_submenu(*id),
            ligada: corrente && !matches!(glifo, Glifo::Escolha),
            acesa: Some(*id) == store.hot_id() || Some(*id) == n.mae,
            cinzenta: store.menu_row_is_unavailable(*id),
        };
        linha::pinta(scene, text_system, &t, l, |scene, caixa| match glifo {
            Glifo::Tema(tm) => paint_theme_swatch(scene, caixa, tm, theme),
            Glifo::Amostra(c) => amostra(scene, caixa, c, theme),
            _ => {}
        });
    }
}

/// A amostra de uma cor escolhida pelo utilizador, com o filete que a deixa ver sobre qualquer fundo.
fn amostra(scene: &mut VectorScene, caixa: Rect, c: [u8; 4], theme: Theme) {
    let raio = crate::paint::frame_radius(theme, Radius::Xs.px());
    fill_rounded_rect(
        scene,
        caixa,
        raio,
        Color::from_rgba8(c[0], c[1], c[2], c[3]), // LITERAL-COLOR-OK: user-color — a amostra mostra a cor escolhida, não um token
    );
    // FRAME-RAW-OK: a AMOSTRA de uma cor do utilizador; o tema não sabe a cor, o filete é que a separa.
    stroke_rounded_rect(scene, caixa, raio, 1.0, resolve(ColorToken::Border, theme));
}
