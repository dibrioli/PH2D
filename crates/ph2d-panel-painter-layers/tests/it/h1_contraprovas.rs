//! O que as contraprovas da fatia H1 acusaram: mutantes da captura que os testes dela deixavam passar, porque com a
//! lista parada o deslocamento é zero. Aqui a lista anda, o cartão solta-se e voa.

use super::h1_os_cartoes::Mesa;
use ph2d_editor_core::aquecimento::{PASSO_DE_QUADRO_S, com_passo_fixo};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::grip_of;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::{FALLBACK_BRUSH, PainterLayersPanel};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_tool_painter::ids as p;

/// O Stroke é nativo desde a H7, e o título do Jitter desde a H9: estes testes medem a lista. A captura só resta na vista
/// Layers, que tem um cartão só, e por isso nada anda por cima dela.
const CAPTURADO: (ph2d_a11y::NodeId, ph2d_a11y::NodeId) = (
    p::PAINTER_BRUSH_STROKE_SECTION,
    core_ids::PAINTER_BRUSH_STROKE_METHOD,
);

/// A lista anda: no MESMO quadro, os alvos vão com o cabeçalho do cartão.
#[test]
fn the_targets_move_with_the_list_in_the_same_frame() {
    let (sec, id) = CAPTURADO;
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let g0 = mesa.rect(grip_of(sec)).expect("a pega");
    let r0 = mesa.rect(id).expect("o controlo");
    let d0 = (r0.x - g0.x, r0.y - g0.y);
    let cima = mesa
        .rect(grip_of(p::PAINTER_CARD_BASICOS))
        .expect("o de cima");
    let ev = mesa
        .host
        .click_at(cima.x + cima.w * 0.5, cima.y + cima.h * 0.5);
    let _ = mesa.aplica(ev);
    let (mut ultimo, mut moveu) = (g0.y, 0);
    for _ in 0..30 {
        let janela = mesa.janela;
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        let (rects, _) = com_passo_fixo(PASSO_DE_QUADRO_S, || {
            h.paint_with_translations::<PainterLayersPanel>(st, janela)
        });
        mesa.rects = rects;
        let g = mesa.rect(grip_of(sec)).expect("a pega");
        let r = mesa.rect(id).expect("o controlo");
        assert!(
            (r.x - g.x - d0.0).abs() < 0.5 && (r.y - g.y - d0.1).abs() < 0.5,
            "o alvo não foi com o cartão: {r:?} × {g:?}"
        );
        let dy = g.y - ultimo;
        ultimo = g.y;
        if dy.abs() > 0.5 {
            moveu += 1;
        }
    }
    assert!(moveu > 0, "a lista não andou: a régua não mediu nada");
}

/// O Reset de uma secção mora só no cartão dela: um bloco sem Reset não herda o do vizinho, nem o do quadro anterior.
#[test]
fn a_section_reset_lives_in_one_card_only() {
    let mut b = FALLBACK_BRUSH;
    b.pigment_offered = true;
    let mesa = Mesa::nova(true, b, false);
    for reset in [
        p::PAINTER_BRUSH_TILING_RESET,
        p::PAINTER_BRUSH_SYMMETRY_RESET,
        p::PAINTER_BRUSH_STROKE_RESET,
        p::PAINTER_SHAPE_RESET,
    ] {
        let n = mesa.rects.iter().filter(|(i, _)| *i == reset).count();
        assert_eq!(n, 1, "{reset:?} registado {n} vezes");
    }
}

fn solta_o_capturado(mesa: &mut Mesa) {
    let g = mesa.rect(grip_of(CAPTURADO.0)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=20 {
        let _ = mesa.ponteiro(PointerKind::Move, x - (i * 30) as f32, y0 + 40.0);
        mesa.quadro();
    }
    let _ = mesa.ponteiro(PointerKind::Up, x - 600.0, y0 + 40.0);
    mesa.assenta();
    assert!(
        mesa.host
            .store()
            .is_loose_in(CAPTURADO.0, ph2d_editor_core::ids::PAINTER_LAYERS_PANEL),
        "o cartão não se soltou"
    );
}

/// Um cartão solto não leva a coluna dos outros: os capturados que ficam na lista não saem do sítio.
#[test]
fn a_loose_card_leaves_the_others_in_the_column() {
    let outro = p::PAINTER_BRUSH_FALLOFF;
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let antes = mesa.rect(outro).expect("o outro");
    solta_o_capturado(&mut mesa);
    let depois = mesa.rect(outro).expect("o outro");
    assert!(
        (depois.x - antes.x).abs() < 1.0 && (depois.w - antes.w).abs() < 1.0,
        "{antes:?} → {depois:?}"
    );
}

/// A lista de um corpo capturado abre debaixo da caixa onde quer que o cartão esteja: um solto, no solto.
#[test]
fn a_loose_captured_list_opens_under_its_box() {
    let id = CAPTURADO.1;
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    solta_o_capturado(&mut mesa);
    let chip = mesa.rect(id).expect("a caixa");
    let antes = mesa.ids();
    let _ = mesa.clica(id);
    mesa.assenta();
    let novos: Vec<Rect> = mesa
        .ids()
        .difference(&antes)
        .filter_map(|n| mesa.rect(*n))
        .collect();
    assert!(!novos.is_empty(), "a lista não abriu");
    for r in novos {
        assert!((r.x - chip.x).abs() < 3.0, "abriu longe: {chip:?} → {r:?}");
    }
}

/// A tinta do cartão chega à peça capturada: com um tema só naquele cartão, as caixas dela usam-no. Medido em 04/11:
/// Máscara deu 0 porque só tinha botões planos e amostras, sem caixa de cartão (legítimo; nativa desde a H8). O que resta
/// capturado com caixas são os parâmetros de um ajuste.
#[test]
fn the_captured_body_wears_the_card_ink() {
    let sec = p::PAINTER_CARD_CAMADAS;
    let mut pilha = ph2d_tool_painter::LayerStack::new();
    let _ = pilha.add_raster("Base", 8, 8).expect("a camada nasce");
    let _ = pilha
        .add_adjustment(ph2d_tool_painter::AdjustmentKind::Bloom)
        .expect("o ajuste nasce");
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, true);
    ph2d_panel_painter_layers::state::set_current_layers(Some(pilha));
    let tema = Theme::Workshop;
    mesa.host.store_mut().set_section_theme(sec, Some(tema));
    mesa.assenta();
    let tinta = CardInk::of_card(Some(tema), Theme::default())
        .color(ColorToken::CardField)
        .premultiply()
        .to_rgba8()
        .to_u32();
    let (h, st, j) = (&mut mesa.host, &mut mesa.st, mesa.janela);
    let n = h
        .paint_draw_data::<PainterLayersPanel>(st, j)
        .into_iter()
        .filter(|c| *c == tinta)
        .count();
    ph2d_panel_painter_layers::state::set_current_layers(None);
    assert!(n >= 2, "as caixas da peça não levam a tinta: {n}");
}

/// Sob uma barra de rolagem, o corpo capturado acompanha a largura real da coluna.
#[test]
fn the_captured_body_keeps_the_column_of_the_native_ones() {
    let (sec, id) = CAPTURADO;
    // A metade direita da linha: a borda dela é a da coluna.
    let nativo = p::PAINTER_BRUSH_BLEND_ON_EMPTY;
    let alta = Mesa::nova(true, FALLBACK_BRUSH, false);
    let borda = |r: Rect| r.x + r.w;
    let folga =
        borda(alta.rect(nativo).expect("o nativo")) - borda(alta.rect(id).expect("o capturado"));
    let janela = Rect::new(0.0, 0.0, 1600.0, 320.0);
    let mut mesa = Mesa::na_janela(true, FALLBACK_BRUSH, false, janela);
    mesa.so_abertos(&[p::PAINTER_CARD_BASICOS, sec]);
    let lista = mesa.host.store().panel_rect(core_ids::PAINTER_LAYERS_PANEL);
    let n = mesa.rect(nativo).expect("o nativo");
    let lista = lista.expect("a lista");
    // Rola até ao capturado, e a borda direita dele é a da coluna.
    for _ in 0..4 {
        let ev = mesa
            .host
            .wheel_at(lista.x + lista.w * 0.5, lista.y + 100.0, -200.0);
        let _ = mesa.aplica(ev);
        mesa.assenta();
        if mesa.rect(id).is_some() {
            break;
        }
    }
    let c = mesa.rect(id).expect("o capturado à vista");
    assert!(
        (borda(n) - borda(c) - folga).abs() < 1.0,
        "colunas diferentes: capturado {c:?} × nativo {n:?}"
    );
}

/// O scrub do Segments: um quadro a meio, com o pincel ainda antigo, não puxa o número de volta.
#[test]
fn scrubbing_segments_is_not_pulled_back_by_the_seed() {
    let chip = p::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP;
    let mut b = FALLBACK_BRUSH;
    b.symmetry_enabled = true;
    b.symmetry_circular = true;
    let mut mesa = Mesa::nova(true, b, false);
    mesa.so_abertos(&[p::PAINTER_BRUSH_SYMMETRY_SECTION]);
    let antes = mesa.host.store().number_value(chip).expect("semeado");
    let _ = mesa.arrasta_a_pega(chip, 40.0);
    let no_gesto = mesa.host.store().number_value(chip).expect("campo");
    assert!((no_gesto - antes).abs() > 0.5, "não se moveu: {no_gesto}");
    mesa.quadro();
    let a_meio = mesa.host.store().number_value(chip).expect("campo");
    assert!(
        (a_meio - no_gesto).abs() < 1e-6,
        "a semente puxou: {a_meio}"
    );
}

/// O painel publica as alturas da lista, como o de antes publicava as dele.
#[test]
fn the_list_publishes_its_heights() {
    let janela = Rect::new(0.0, 0.0, 1600.0, 500.0);
    let _mesa = Mesa::na_janela(true, FALLBACK_BRUSH, false, janela);
    let (conteudo, visivel) = (
        ph2d_panel_painter_layers::state::last_content_h(),
        ph2d_panel_painter_layers::state::last_visible_h(),
    );
    assert!(
        visivel > 0.0 && conteudo > visivel,
        "{conteudo} / {visivel}"
    );
}

/// A arrastar uma camada, o fantasma com o nome dela pinta-se por cima da lista.
#[test]
fn a_dragged_layer_row_paints_its_ghost() {
    use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
    // A linha que voa é a da lista: a superfície do cartão do Kanban (H9).
    let tinta = CardInk::of_card(None, Theme::default())
        .color(ColorToken::KanbanCardRaised)
        .premultiply()
        .to_rgba8()
        .to_u32();
    let conta = |mesa: &mut Mesa| {
        let (h, st, j) = (&mut mesa.host, &mut mesa.st, mesa.janela);
        h.paint_draw_data::<PainterLayersPanel>(st, j)
            .into_iter()
            .filter(|c| *c == tinta)
            .count()
    };
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, true);
    let mut pilha = ph2d_tool_painter::LayerStack::new();
    let l = pilha.add_raster("One", 8, 8).expect("a camada nasce");
    let _ = pilha.add_raster("Two", 8, 8).expect("a camada nasce");
    ph2d_panel_painter_layers::state::set_current_layers(Some(pilha));
    mesa.assenta();
    let parada = conta(&mut mesa);
    let r = mesa
        .rect(painter_layer_widget_id(l.0, PainterLayerWidget::Row))
        .expect("a linha");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 3.0, y + 12.0);
    let _ = mesa.ponteiro(PointerKind::Move, x + 3.0, y + 19.0);
    let a_voar = conta(&mut mesa);
    ph2d_panel_painter_layers::state::set_current_layers(None);
    assert!(a_voar > parada, "sem fantasma: {parada} → {a_voar}");
}
