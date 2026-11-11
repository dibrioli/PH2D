//! ⭐ **As etiquetas na letra do código escurecem pelo fundo REAL** (10/11): o nome na lista, as
//! entradas do «In this script», a frase da grelha, os títulos e as linhas dos menus da barra e os
//! exemplos da ajuda pintam-se fora do poço do código; o «Crisp + stem darkening» do script decide
//! por quem está DEBAIXO de cada uma (cartão, linha acesa, popover, caixa do exemplo), nunca pelo poço.

use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::widget::Superficie;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::code_text::set_mode;
use ph2d_panel_code::mono_label;
use ph2d_panel_code::tab_bar::menu::MenuItem;
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, TextRendering, Theme};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::{Color, VectorScene};

fn mono_id(ts: &mut TextSystem) -> u64 {
    let mut cache = ph2d_panel_code::LineCache::default();
    let l = cache.get(ts, "a", &[], ph2d_panel_code::code_font_px());
    let run = l.lines().next().and_then(|l| l.runs().next());
    run.expect("moldado").font().data.id()
}

/// O alargamento de cada corrida da mono, pela ordem da pintura.
fn alargamentos(ts: &mut TextSystem, s: &VectorScene) -> Vec<f64> {
    let mono = mono_id(ts);
    let r = &s.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .filter(|g| g.font.data.id() == mono)
        .map(|g| g.font_embolden.amount.xx)
        .collect()
}

fn lum(c: Color) -> f32 {
    c.discard_alpha().relative_luminance()
}

#[test]
fn a_etiqueta_mono_escurece_pelo_fundo_que_recebe() {
    let mut ts = TextSystem::without_system_fonts();
    set_mode(TextRendering::CrispPlusDarkened);
    let tinta = Color::from_rgba8(90, 90, 90, 255);
    let mut uma = |bg: Color| {
        let mut s = VectorScene::new();
        let r = Rect::new(0.0, 0.0, 200.0, 20.0);
        mono_label::paint_on(&mut ts, &mut s, Theme::default(), "alive", tinta, bg, r);
        alargamentos(&mut ts, &s)
    };
    let (escuro, claro) = (
        uma(Color::from_rgba8(20, 20, 20, 255)),
        uma(Color::from_rgba8(230, 230, 230, 255)),
    );
    set_mode(TextRendering::Default);
    assert!(
        escuro.iter().all(|&a| a > 0.0) && !escuro.is_empty(),
        "{escuro:?}"
    );
    assert!(
        claro.iter().all(|&a| a == 0.0) && !claro.is_empty(),
        "{claro:?}"
    );
}

const CMD: ph2d_a11y::NodeId = hash_node_id("t_fundo_cmd");
const SUB: ph2d_a11y::NodeId = hash_node_id("t_fundo_sub");

/// Nos 12 temas: título fechado (cartão da barra), título aberto e linha acesa (acento), linha
/// apagada (popover). Cada um alarga sse a tinta é mais clara que o fundo debaixo dela. O controlo:
/// em algum tema o poço daria a resposta errada (senão o teste não distinguia a cura).
#[test]
fn os_menus_da_barra_escurecem_pelo_fundo_debaixo_de_cada_linha() {
    let mut ts = TextSystem::without_system_fonts();
    force_cards_on_this_thread(Some(true));
    let mut distingue = 0;
    for theme in Theme::ALL {
        set_mode(TextRendering::CrispPlusDarkened);
        let mut bar = TabBar::<u8>::default();
        let items = vec![
            MenuItem::Cmd {
                id: CMD,
                name: "Toggle Peek".into(),
                keys: String::new(),
                live: true,
            },
            MenuItem::Sub {
                id: SUB,
                name: "Text rendering".into(),
                rows: vec![],
            },
        ];
        let crumb = Crumb {
            script: "a.luau".into(),
            func: None,
            line: 1,
        };
        let titles = [(0u8, "File"), (1u8, "View")];
        let r = Rect::new(0.0, 0.0, 900.0, height());
        let mut s = VectorScene::new();
        bar.open = Some(1);
        bar.hover = Some(CMD);
        bar.paint(r, &titles, &crumb, &mut s, &mut ts, &theme);
        bar.paint_popup(&items, &mut s, &mut ts, &theme, (0, 0));
        let got = alargamentos(&mut ts, &s);
        set_mode(TextRendering::Default);
        let c = |t: ColorToken| resolve(t, theme);
        let card = c(ph2d_panel_code::cards::card_tone());
        let pop = Superficie::de(theme).fundo();
        let want = [
            (c(ColorToken::Text1), card),
            (c(ColorToken::AccentFg), c(ColorToken::Accent)),
            (c(ColorToken::AccentFg), c(ColorToken::Accent)),
            (c(ColorToken::Text1), pop),
        ];
        assert_eq!(got.len(), want.len(), "{theme:?}: {got:?}");
        let poco = c(ColorToken::KanbanCard);
        for (i, ((ink, bg), a)) in want.iter().zip(&got).enumerate() {
            assert_eq!(*a > 0.0, lum(*ink) > lum(*bg), "{theme:?} etiqueta {i}");
            distingue += usize::from((lum(*ink) > lum(*bg)) != (lum(*ink) > lum(poco)));
        }
    }
    force_cards_on_this_thread(None);
    assert!(distingue > 0, "nenhum tema distingue o fundo real do poço");
}

/// A linha acesa assenta no acento, a apagada no fundo do popover (o que o escurecimento lê).
#[test]
fn a_linha_acesa_assenta_no_acento_e_a_apagada_no_popover() {
    use ph2d_panel_code::tab_bar::menu::row_bg;
    for theme in Theme::ALL {
        assert_eq!(
            row_bg(true, &theme),
            resolve(ColorToken::Accent, theme),
            "{theme:?}"
        );
        assert_eq!(
            row_bg(false, &theme),
            Superficie::de(theme).fundo(),
            "{theme:?}"
        );
    }
}
