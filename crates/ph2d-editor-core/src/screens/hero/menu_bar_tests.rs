//! Gates da barra de menus — e dos outros dois rótulos do chrome cuja caixa é MEDIDA contra o
//! texto (as abas de tarefa e as abas de encaixe).

use super::*;
use crate::screens::hero::{layout_tabs, slot_tabs_face};
use crate::screens::task_layout::TaskLayout;
use ph2d_tokens::{TextRendering, UiFont, UiTextSize, UiTextStyle, UiWeight};

/// Corre `f` uma vez por combinação de estilo (fonte × peso × tamanho) e nitidez, com o estilo
/// publicado como o quadro o publica, e devolve o que cada corrida acusou.
pub(in crate::screens::hero) fn em_todo_estilo(
    mut f: impl FnMut(&mut TextSystem) -> Vec<String>,
) -> Vec<String> {
    let mut acusados = Vec::new();
    for rendering in [
        TextRendering::Default,
        TextRendering::CrispHeavy,
        TextRendering::CrispHeavyPlus,
    ] {
        for font in UiFont::ALL {
            for weight in UiWeight::ALL {
                for size in UiTextSize::ALL {
                    ph2d_text::set_active_text_rendering(rendering);
                    ph2d_text::set_active_text_style(UiTextStyle { font, weight, size });
                    let mut ts = TextSystem::without_system_fonts();
                    for a in f(&mut ts) {
                        acusados.push(format!("{rendering:?} {font:?} {weight:?} {size:?}: {a}"));
                    }
                }
            }
        }
    }
    ph2d_text::set_active_text_rendering(TextRendering::Default);
    ph2d_text::set_active_text_style(UiTextStyle::default());
    acusados
}

fn branco() -> ph2d_vector::Color {
    ph2d_vector::Color::from_rgba8(255, 255, 255, 255) // LITERAL-COLOR-OK: cor de teste
}

/// ⭐⭐ **Nenhum rótulo cuja caixa é medida contra ele sai cortado — em nenhuma fonte, peso,
/// tamanho ou nitidez** (2026-10-01).
///
/// ⛔⛔ O defeito é a volta em vírgula flutuante: a caixa é `texto + 2·recuo`, o pintor devolve
/// `caixa − 2·recuo` de orçamento, e `(t + a) − a` fica **um ULP abaixo** de `t` numa fracção do
/// domínio — a elisão compara `<=`, logo o rótulo medido para caber sai `Wind…`. Só aparece em
/// larguras EXACTAS de fonte, e por isso só apareceu quando a Inter passou a ser desenhada a
/// sério. ⇒ o gate varre **todas** as combinações, e a régua é o que foi PINTADO (o instrumento das
/// elisões), nunca uma conta refeita aqui.
///
/// Três superfícies, a mesma lei:
/// - **a barra de menus** (`Window` -> `Wind…`, foto da Inter) — pela [`pinta_titulo`];
/// - **as abas de tarefa** (`Flip` -> `F…` no Grande) — pela geometria do `layout_tabs` e o
///   pintor que ele usa;
/// - **as abas de encaixe** (`Inspector` -> `Inspect…` na Atkinson) — pela `face`, que é a única
///   porta da geometria de dentro de uma aba.
///
/// *Mutação: cada uma das três curas desfeita ⇒ pelo menos uma combinação corta.*
#[test]
fn nenhum_rotulo_medido_do_chrome_e_cortado() {
    let bar = Rect::new(0.0, 0.0, 1930.0, MENU_BAR_H);
    let mut titulos: Vec<String> = TaskLayout::ALL
        .iter()
        .map(|l| l.spec().title.tr().to_owned())
        .collect();
    titulos.extend(MENUS.iter().map(|(_, k, _)| k.tr().to_owned()));
    // Nomes de painel reais — o da foto primeiro.
    for k in ["panel.inspector.title", "chrome.menu.inspector"] {
        titulos.push(ph2d_i18n::tr(k).to_owned());
    }

    let acusados = em_todo_estilo(|ts| {
        let mut scene = VectorScene::new();
        let (_, medidos) = crate::text_elide::elisao::medindo(|| {
            for (_, title, r) in menu_rects(bar, ts) {
                pinta_titulo(ts, &mut scene, title, r, branco());
            }
            for (l, r) in layout_tabs::tab_rects(bar, 0.0, ts) {
                crate::paint::paint_text_centered(
                    ts,
                    &mut scene,
                    l.spec().title.tr(),
                    r,
                    TypeToken::Sm.px(),
                    branco(),
                );
            }
        });
        let mut fora: Vec<String> = medidos
            .iter()
            .filter(|m| !m.coube())
            .map(|m| format!("barra/tarefa «{}» -> «{}»", m.texto, m.pintado))
            .collect();
        for t in &titulos {
            let w = slot_tabs_face::natural_w(t, ts);
            let face = slot_tabs_face::face(Rect::new(0.0, 0.0, w, MENU_BAR_H), t, ts);
            let mostrado = face.label.map(|(s, _, _)| s).unwrap_or_default();
            if mostrado != *t {
                fora.push(format!("aba de encaixe «{t}» -> «{mostrado}»"));
            }
        }
        fora
    });
    assert!(
        acusados.is_empty(),
        "rótulos medidos e cortados ({}):\n  {}",
        acusados.len(),
        acusados.join("\n  ")
    );
}
