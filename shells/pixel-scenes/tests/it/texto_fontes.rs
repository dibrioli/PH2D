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
use ph2d_pixel_text::{BUILTIN, SYSTEM_MIN_PX, builtin_line_height};

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
fn the_font_list_starts_with_the_pixel_fonts() {
    let todas = ph2d_app_pixel::texto::fontes();
    let fabrica: Vec<&str> = BUILTIN.iter().map(|b| b.0).collect();
    assert_eq!(&todas[..BUILTIN.len()], fabrica.as_slice());
    let pixel = ph2d_pixel_text::carimbo::pixel_popular_count(&todas);
    assert!(pixel >= BUILTIN.len() && pixel <= todas.len());
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
fn picking_a_system_font_never_leaves_the_size_below_the_legible_minimum() {
    let (mut app, sim, _cam, bits) = born();
    let todas = ph2d_app_pixel::texto::fontes();
    if todas.len() <= BUILTIN.len() {
        return;
    }
    com(&mut app, &sim, bits, 0, 1);
    com(&mut app, &sim, bits, BUILTIN.len(), 1);
    assert_eq!(app.options.text_px, SYSTEM_MIN_PX);
    com(&mut app, &sim, bits, BUILTIN.len(), SYSTEM_MIN_PX + 5);
    assert_eq!(
        app.options.text_px,
        SYSTEM_MIN_PX + 5,
        "acima do mínimo, livre"
    );
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

/// P8c: a prévia de cada fonte de pixel na linha da lista (`ROW_H_PX`) sai na maior escala INTEIRA
/// que cabe — nunca abaixo de metade da linha (o unscii-8 saía a 1× com a fração 0,7 de antes).
#[test]
fn every_pixel_font_preview_fills_the_list_row_at_an_integer_scale() {
    let (mut app, sim, _cam, bits) = born();
    app.tool = Tool::Text;
    for b in BUILTIN {
        font_preview::want(b.0);
    }
    quadro(&mut app, &sim, bits);
    let linha = ph2d_tokens::ROW_H_PX;
    let mut tabela = String::new();
    let mut ok = true;
    for b in BUILTIN {
        font_preview::with_preview(b.0, |p| {
            let p = p.unwrap_or_else(|| panic!("{}: sem pré-visualização", b.0));
            let s = p.scale_in(linha);
            #[expect(clippy::cast_precision_loss, reason = "altura de uma máscara de nome")]
            let h = p.h as f32;
            tabela.push_str(&format!("{} h={} s={s} → {}\n", b.0, p.h, s * h));
            ok &= s == s.floor() && s * h <= linha && (s + 1.0) * h > linha && s * h * 2.0 > linha;
        });
    }
    assert!(ok, "linha {linha}:\n{tabela}");
}
