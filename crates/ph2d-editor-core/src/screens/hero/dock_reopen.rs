//! ⭐⭐⭐ **A ALÇA que traz de volta uma coluna fechada** — e ela é PINTADA, ao contrário da costura.
//!
//! Fechar as duas colunas devolve **89 a 92 %** do ecrã em qualquer dos três tablets alvo — mais do
//! que todas as faixas de chrome somadas valem (`medicoes/06`). Desde 2026-09-07 isso é **um
//! gesto**: arrastar a borda para dentro, para além do mínimo, fecha a coluna.
//!
//! ⛔⛔ **Mas um gesto que fecha sem caminho de volta é uma armadilha**, e num tablet é uma
//! armadilha sem saída: a costura de redimensionar não é pintada — ela vive do **cursor**, e num
//! ecrã de toque não há cursor. O que a torna descobrível é a borda visível da coluna; fechada a
//! coluna, essa borda desaparece.
//!
//! ⇒ esta faixa ocupa exactamente onde a borda estava, é pintada, e um toque nela reabre. *A mão
//! volta a puxar de onde empurrou.*
//!
//! ⚠️ **Ela não é chrome permanente:** só existe enquanto a coluna está fechada, e nesse estado ela
//! substitui uma coluna de 304–308 px por uma de 6. O saldo é a medição inteira do `medicoes/06`.

use super::HeroScreen;
use crate::paint::{fill_rounded_rect, resolve};
use crate::screens::layout::{DockSide, HeroLayout};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// A largura do chevron desenhado dentro da faixa.
const ARROW_W: f32 = 2.0; // LITERAL-PX-OK: traço do chevron da alça, dentro de uma faixa de 6

/// **Pinta as alças das colunas fechadas** — nenhuma, uma, ou as duas.
///
/// ⚠️ **Sem estado e sem hover.** A faixa diz *«há aqui uma coluna fechada»* e mais nada: um realce
/// sob o dedo exigiria um id no `WidgetStore` e um estado que só existe enquanto a coluna não
/// existe. O gesto de reabrir vive na shell, pela porta `dock_reopen_at`.
pub fn paint_dock_reopen(layout: &HeroLayout, scene: &mut VectorScene, theme: Theme) {
    for side in [DockSide::Left, DockSide::Right] {
        let r = layout.dock_reopen(side);
        if r.w <= 0.0 || r.h <= 0.0 {
            continue;
        }
        // A faixa: o tom de uma superfície elevada, para se ler contra a área de desenho.
        fill_rounded_rect(scene, r, 0.0, resolve(ColorToken::BgElev, theme));
        // F5: a FAIXA de ícones pinta os ícones dela (`paint_strip_icons`), não a marca.
        if r.w > crate::screens::dock_seam::DOCK_SEAM_PX {
            continue;
        }
        // ⭐ Uma marca ao meio, na altura de uma linha — o bastante para dizer «puxa-me», sem
        //   pedir um ícone (que exigiria medir texto num sítio que não tem `TextSystem`).
        let mark_h = ph2d_tokens::ROW_H_PX;
        let mark = Rect::new(
            r.x + (r.w - ARROW_W) * 0.5,
            r.y + (r.h - mark_h) * 0.5,
            ARROW_W,
            mark_h,
        );
        fill_rounded_rect(
            scene,
            mark,
            ARROW_W * 0.5,
            resolve(ColorToken::Text3, theme),
        );
    }
}

/// ⭐ F5: **a FAIXA de ícones do ③ recolhido** (iPad 11" e mini — mais estreito que a janela de
/// referência): um ícone por painel que reabrir devolve, numa coluna FLUTUANTE na borda da área do
/// lado da coluna — como a ① (0 px de layout; a alça de 6 px continua). ⚠️ Reservá-la custava o piso
/// medido do ecrã com as colunas fechadas (88,7 % → 86,0 % no iPad 11, o gate do `medicoes/06`).
///
/// Cada ícone é a ABA do painel (`slot_tabs::tab_node_id`): o clique reabre a coluna e traz o painel
/// à frente pela porta das abas ([`reopen_for`]).
#[must_use]
pub fn strip_icons(
    hero: &HeroScreen,
    layout: &HeroLayout,
) -> Vec<(NodeId, crate::icons::IconId, crate::panel::TextKey, Rect)> {
    let handle = layout.dock_reopen(DockSide::Right);
    if !layout.compact() || handle.w <= 0.0 {
        return Vec::new();
    }
    let names = super::dock_columns::reopens(hero, DockSide::Right);
    let mut found = Vec::new();
    crate::panel::with_registry_opt(|reg| {
        for m in reg.panels().iter().map(|p| &p.manifest) {
            if names.contains(&m.id) {
                found.push((
                    super::slot_tabs::tab_node_id(m.panel_node_id),
                    m.icon,
                    m.title,
                ));
            }
        }
    });
    let area = layout.draw_area;
    let right = handle.x > area.x + area.w * 0.5;
    let rects = super::float_bar::column_rects(area, &[found.len()]);
    found
        .into_iter()
        .zip(rects)
        .map(|((id, icon, title), r)| {
            let r = if right {
                super::float_bar::mirrored(area, r)
            } else {
                r
            };
            (id, icon, title, r)
        })
        .collect()
}

/// F5: pinta a faixa (fundo, ícones, alvos e dicas com o NOME do painel).
pub fn paint_strip_icons(hero: &mut HeroScreen, layout: &HeroLayout, scene: &mut VectorScene) {
    let icons = strip_icons(hero, layout);
    if icons.is_empty() {
        return;
    }
    let theme = hero.theme;
    super::float_bar::paint_panel(scene, icons.iter().map(|i| i.3), theme);
    for (id, icon, title, r) in icons {
        let fg = super::float_bar::paint_button(
            scene,
            &mut hero.hit_index,
            &hero.store,
            theme,
            super::float_bar::Btn {
                id,
                rect: r,
                on: false,
                tint: ColorToken::Text2,
            },
        );
        crate::paint::paint_icon(
            scene,
            icon,
            super::float_bar::inner(r),
            fg,
            ph2d_tokens::StrokeToken::Default.px(),
        );
        hero.store.set_tooltip(id, title.tr().to_owned());
    }
}

/// F5: a aba de um painel cuja coluna está RECOLHIDA (a faixa) reabre a coluna antes de o trazer à
/// frente. `true` se reabriu.
pub fn reopen_for(hero: &mut HeroScreen, panel_node: NodeId) -> bool {
    for side in [DockSide::Left, DockSide::Right] {
        if !super::dock_columns::is_closed(hero, side) {
            continue;
        }
        let mut hit = false;
        let names = super::dock_columns::reopens(hero, side);
        crate::panel::with_registry_opt(|reg| {
            hit = reg
                .panels()
                .iter()
                .any(|p| p.manifest.panel_node_id == panel_node && names.contains(&p.manifest.id));
        });
        if hit {
            return super::dock_columns::open(hero, side);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::layout::{
        CenterSplit, ChromeBands, DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W,
    };

    fn layout(docks: DockSides) -> HeroLayout {
        HeroLayout::for_viewport_bands(
            Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            docks,
        )
    }

    /// ⭐ **Com as duas colunas abertas, a alça não existe** — e não pinta nada.
    #[test]
    fn an_open_column_has_no_handle() {
        let l = layout(DockSides::BOTH);
        assert_eq!(l.dock_reopen(DockSide::Left).w, 0.0);
        assert_eq!(l.dock_reopen(DockSide::Right).w, 0.0);
        let mut scene = VectorScene::new();
        paint_dock_reopen(&l, &mut scene, Theme::Dark);
        assert_eq!(
            scene.inner().encoding().n_path_segments,
            0,
            "a alca pintou com as duas colunas abertas: ela seria chrome permanente"
        );
    }

    /// ⭐⭐ **Fechada, ela existe, pinta, e ocupa a borda EXTERIOR.**
    #[test]
    fn a_closed_column_gets_a_painted_handle_on_its_outer_edge() {
        let l = layout(DockSides {
            left: false,
            right: true,
        });
        let h = l.dock_reopen(DockSide::Left);
        let (col, _) = l.side_columns();
        assert!(h.w > 0.0, "a coluna fechada nao ofereceu alca");
        assert_eq!(
            h.x, col.x,
            "a alca da esquerda tem de tomar a borda EXTERIOR"
        );
        assert_eq!(
            l.dock_reopen(DockSide::Right).w,
            0.0,
            "a direita esta' aberta"
        );

        let mut scene = VectorScene::new();
        paint_dock_reopen(&l, &mut scene, Theme::Dark);
        assert!(
            scene.inner().encoding().n_path_segments > 0,
            "a alca nao pintou: num tablet ela seria invisivel, e o caminho de volta some"
        );
    }

    /// ⛔ **A alça e a costura nunca respondem ao mesmo tempo** — uma exige a coluna aberta, a
    /// outra fechada. Se as duas respondessem, um toque seria ambíguo.
    #[test]
    fn the_handle_and_the_seam_are_never_both_offered() {
        for docks in [
            DockSides::BOTH,
            DockSides {
                left: false,
                right: true,
            },
            DockSides {
                left: true,
                right: false,
            },
            DockSides::NONE,
        ] {
            let l = layout(docks);
            for side in [DockSide::Left, DockSide::Right] {
                let seam = l.dock_seam(side).w > 0.0;
                let handle = l.dock_reopen(side).w > 0.0;
                assert!(
                    !(seam && handle),
                    "{side:?} com {docks:?}: a costura e a alca foram oferecidas as duas"
                );
            }
        }
    }
}
