//! **P8b: o dropdown de fontes do Text pelo caminho real** — a lista começa pelas de pixel, escolher
//! uma fonte SALTA o tamanho para o válido dela (de pixel: múltiplo da linha; do sistema: nunca abaixo
//! do mínimo legível medido), o campo do tamanho de uma fonte de pixel anda um múltiplo por gesto, e o
//! painel recebe o nome de cada fonte desenhado nela.

use ph2d_app_pixel::{PixelApp, Tool, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_editor_core::pixel_panel::font_preview;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_text::{BUILTIN, builtin_line_height};

use crate::comum::born;

fn quadro(app: &mut PixelApp, sim: &SimWorld, bits: u64) {
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    panel::frame(app, sim.world(), &mut hero, &mut ToastQueue::new());
}

fn com(app: &mut PixelApp, sim: &SimWorld, bits: u64, font: usize, px: u32) {
    let mut o = app.options;
    o.text_font = u16::try_from(font).expect("índice");
    o.text_px = px;
    app.pede(PixelRequest::Options(Box::new(o)));
    quadro(app, sim, bits);
}

/// No arranque o tamanho do Text já é um tamanho VÁLIDO da fonte de omissão (nunca o `1` cru).
#[test]
fn the_text_size_starts_valid_for_the_default_font() {
    let o = PixelApp::default().options;
    let fonte = usize::from(o.text_font);
    assert_eq!(o.text_px, ph2d_pixel_text::valid_px(fonte, o.text_px, 0));
    assert_ne!(o.text_px, 1);
    assert_eq!(Some(o.text_px), builtin_line_height(fonte), "1× da linha");
}

#[test]
fn the_font_list_is_the_factory_list_and_nothing_else() {
    let todas = ph2d_app_pixel::texto::fontes();
    let fabrica: Vec<&str> = BUILTIN.iter().map(|b| b.0).collect();
    assert_eq!(todas, fabrica, "dono, 05/11: só as 12");
}

#[test]
fn picking_a_pixel_font_snaps_the_size_to_a_multiple_of_its_line() {
    let (mut app, sim, _cam, bits) = born();
    for (i, (nome, ..)) in BUILTIN.iter().enumerate() {
        let lh = builtin_line_height(i).expect("linha nativa");
        // Vem de outra fonte com um tamanho que não é múltiplo desta.
        let outra = (i + 1) % BUILTIN.len();
        com(&mut app, &sim, bits, outra, 1);
        com(&mut app, &sim, bits, i, lh * 2 + 1);
        assert_eq!(app.options.text_px, lh * 2, "{}: o mais perto", nome);
        // O campo sobe um: salta ao múltiplo seguinte; desce um: ao anterior; nunca abaixo de 1×.
        com(&mut app, &sim, bits, i, lh * 2 + 1);
        assert_eq!(app.options.text_px, lh * 3, "{}: sobe", nome);
        com(&mut app, &sim, bits, i, lh * 3 - 1);
        assert_eq!(app.options.text_px, lh * 2, "{}: desce", nome);
        com(&mut app, &sim, bits, i, 1);
        assert_eq!(app.options.text_px, lh, "{}: 1× no mínimo", nome);
    }
}

#[test]
fn the_dropdown_gets_each_font_name_drawn_in_that_font() {
    let (mut app, sim, _cam, bits) = born();
    app.tool = Tool::Text;
    for b in BUILTIN {
        font_preview::want(b.0);
    }
    quadro(&mut app, &sim, bits);
    for (i, b) in BUILTIN.iter().enumerate() {
        let lh = builtin_line_height(i).expect("linha");
        font_preview::with_preview(b.0, |p| {
            let p = p.unwrap_or_else(|| panic!("{}: sem pré-visualização", b.0));
            // Tamanho NATIVO (1×): a tinta cabe na altura da linha.
            assert!(p.w > 0 && p.h > 0 && p.h <= lh, "{}: {}×{}", b.0, p.w, p.h);
            assert!(p.cover.contains(&u8::MAX), "{}: tinta", b.0);
        });
    }
}

/// A prévia de cada fonte na linha da lista sai numa escala INTEIRA, com a folga em cima e em baixo e
/// com alturas próximas: as 12 lêem-se do mesmo tamanho (dono, 05/11).
#[test]
fn every_pixel_font_preview_fills_the_list_row_at_an_integer_scale() {
    let (mut app, sim, _cam, bits) = born();
    app.tool = Tool::Text;
    for b in BUILTIN {
        font_preview::want(b.0);
    }
    quadro(&mut app, &sim, bits);
    let linha = 28.0; // LITERAL-PX-OK: a linha da lista em cartões, medida na foto
    let livre = linha - 2.0 * ph2d_tokens::Spacing::Xs.px();
    let mut tabela = String::new();
    let mut ok = true;
    for b in BUILTIN {
        font_preview::with_preview(b.0, |p| {
            let p = p.unwrap_or_else(|| panic!("{}: sem pré-visualização", b.0));
            let s = p.scale_in(linha);
            #[expect(clippy::cast_precision_loss, reason = "altura de uma máscara de nome")]
            let h = p.h as f32;
            tabela.push_str(&format!("{} h={} s={s} → {}\n", b.0, p.h, s * h));
            ok &= s == s.floor() && s * h <= livre && (10.0..=livre).contains(&(s * h));
        });
    }
    assert!(ok, "linha {linha}:\n{tabela}");
}
