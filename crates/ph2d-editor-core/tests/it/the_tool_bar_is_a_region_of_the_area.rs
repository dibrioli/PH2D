//! ⭐⭐ **A FILA DE FERRAMENTAS é uma REGIÃO da área, e a geometria dela tem UMA porta.**
//!
//! Enio, 2026-08-30: *«ainda temos os botões da lateral»*. O trilho vertical saiu; os chips voltam
//! deitados por cima da área de desenho, no modelo do Godot.
//!
//! # As duas leis que este ficheiro mede
//!
//! 1. **Região, não camada** (D5): a fila sai da **área**, entre as colunas — e a régua começa por
//!    baixo dela. As duas não se tapam porque não partilham coordenada. ⛔ O trilho antigo ancorava
//!    em `x = 0` e cobria **86,8 %** da régua da esquerda; uma fila que atravessasse o ecrã faria o
//!    mesmo às colunas.
//! 2. **Uma porta para a geometria**: [`entry_rects`] responde *«onde cai cada entrada?»* nos dois
//!    eixos, e é ela que o pintor e o registo de hit perguntam.
//!
//! ⛔⛔ **A segunda existe porque a resposta estava escrita TRÊS vezes** — o pintor, o hit do
//! trilho e o hit do flyout, cada um com o seu `let mut y`. O comentário do segundo dizia *«Hit-rects
//! MUST mirror exactly what `paint_tool_rail` paints»*, que é a confissão do defeito: **um espelho
//! não é uma lei**. Um pintor horizontal com um hit vertical compilaria e passaria a suíte inteira,
//! e o sintoma seria *«os botões não pegam»* sem um único gate vermelho.

use ph2d_editor_core::screens::hero::tool_bar;
use ph2d_editor_core::screens::layout::{
    CenterSplit, ChromeBands, DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout,
};
use ph2d_editor_core::widget::{RailAxis, RailButtonSize, ToolRail, entry_rects};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ruler};
use ph2d_text::TextSystem;

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(NodeId(1))
}

fn overlap_area(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

fn layout_with_bar() -> HeroLayout {
    HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        false,
        ChromeBands {
            rail_w: 0.0,
            top_bar_h: 0.0,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::default(), 1),
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    )
}

/// ⭐ **A fila e a área de desenho não partilham um pixel** — e a régua, que nasce na borda da
/// área, também não a alcança.
#[test]
fn the_bar_and_the_drawing_area_are_siblings_not_layers() {
    let l = layout_with_bar();
    assert!(l.tool_bar.h > 0.0, "a fila tem de ter faixa");
    assert_eq!(
        overlap_area(l.tool_bar, l.draw_area),
        0.0,
        "a fila entra na área de desenho: {:?} contra {:?}",
        l.tool_bar,
        l.draw_area
    );
    for (name, band) in [
        ("topo", ruler::top_band(l.draw_area)),
        ("esquerda", ruler::left_band(l.draw_area)),
    ] {
        assert_eq!(
            overlap_area(l.tool_bar, band),
            0.0,
            "a fila tapa a régua do {name} — o defeito do trilho antigo, deitado"
        );
    }
}

/// ⛔ **E ela vive ENTRE as colunas** — sai da área, não da janela.
///
/// Uma barra de ferramentas à largura do ecrã passaria por cima da Hierarquia e do Inspector, que
/// é exactamente o modelo `x = 0` que a wave das réguas desmontou.
#[test]
fn the_bar_lives_between_the_columns_never_across_the_window() {
    let l = layout_with_bar();
    for (name, col) in [("hierarchy", l.hierarchy), ("inspector", l.inspector)] {
        assert_eq!(
            overlap_area(l.tool_bar, col),
            0.0,
            "a fila entra na coluna {name}"
        );
    }
    assert!(
        l.tool_bar.w < l.viewport.w,
        "a fila tem a largura da janela — ela devia ser a da ÁREA"
    );
}

/// ⭐⭐ **O que o quadro REGISTA é o que a porta diz** — a prova de que pintor e hit são uma lei.
///
/// *Mutação que sangra:* qualquer aritmética própria no registo de hit do `tool_bar`, ou o pintor
/// a deixar de consumir o `entry_rects`.
#[test]
fn the_painted_bar_registers_every_chip_where_the_door_says() {
    let mut h = hero();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, viewport, &mut scene, &mut text);
    let l = h.last_layout.expect("o quadro publicou o layout");
    assert!(
        l.tool_bar.h > 0.0,
        "sem chrome legado a fila tem de existir"
    );

    // ⭐ **A MESMA porta que o pintor usou** — `bar_rail`, não uma lista remontada aqui. Uma
    // segunda montagem seria a tabela paralela que este ficheiro inteiro existe para proibir.
    let rail = tool_bar::bar_rail(&h.store, false, h.image_edit.mode_on);
    let content = tool_bar::content_rect(l.tool_bar);
    let mut seen = 0usize;
    for slot in entry_rects(
        &rail,
        content,
        h.store.rail_button_size(),
        RailAxis::Horizontal,
    ) {
        let Some(id) = slot.id else { continue };
        seen += 1;
        let got = h
            .hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("{id:?}: chip pintado e SEM alvo no índice do quadro"));
        assert!(
            (got.x - slot.rect.x).abs() < 0.5 && (got.y - slot.rect.y).abs() < 0.5,
            "{id:?}: o alvo ({got:?}) não é onde a porta diz ({:?})",
            slot.rect
        );
    }
    // ⚠️ **Contagem DERIVADA da lista, não um número.** Aqui esteve `seen >= 10`, e ele reprovou
    // no dia em que os dois toggles de painel se mudaram para o menu *Ver* — sobre uma lista
    // correcta. Um mínimo escrito à mão mede a versão da lista em que foi escrito.
    let expected = rail
        .entries
        .iter()
        .filter(|e| e.node_id().is_some())
        .count();
    assert_eq!(
        seen, expected,
        "nem todos os chips da lista chegaram ao índice"
    );
}

/// ⭐ **A ADVANCE é a mesma nos dois eixos** — é o que faz a coluna e a fila serem uma lei só.
///
/// ⚠️ A comparação é entre **extensões ao longo do eixo**: o mesmo rail deitado ocupa em `x` o que
/// em pé ocupa em `y`. Se um dia divergirem, o chip da fila e o da coluna deixam de ter o mesmo
/// tamanho e o preset de tamanho passa a mentir num deles.
#[test]
fn the_two_axes_advance_by_the_same_law() {
    let h = hero();
    let rail = ToolRail::new(
        NodeId(203),
        "Editor tools",
        ph2d_editor_core::screens::hero::left_rail::rail_entries(&h.store, false),
    );
    let size = h.store.rail_button_size();
    let origin = Rect::new(100.0, 200.0, 400.0, 400.0);
    let v = entry_rects(&rail, origin, size, RailAxis::Vertical);
    let hz = entry_rects(&rail, origin, size, RailAxis::Horizontal);
    assert_eq!(v.len(), hz.len(), "os dois eixos vêem listas diferentes");
    for (a, b) in v.iter().zip(hz.iter()) {
        assert_eq!(a.id, b.id, "a ordem divergiu entre os eixos");
        assert!(
            ((a.rect.y - origin.y) - (b.rect.x - origin.x)).abs() < 0.001,
            "a entrada {:?} cai em {} na coluna e em {} na fila",
            a.id,
            a.rect.y - origin.y,
            b.rect.x - origin.x
        );
    }
}

/// **Nenhum chip se sobrepõe ao vizinho, nos dois eixos.** O controlo negativo do laço da porta:
/// um `advance` esquecido daria chips empilhados no mesmo pixel, e todos os outros gates passariam.
#[test]
fn no_two_entries_share_a_pixel_on_either_axis() {
    let h = hero();
    let rail = ToolRail::new(
        NodeId(203),
        "Editor tools",
        ph2d_editor_core::screens::hero::left_rail::rail_entries(&h.store, true),
    );
    let size = h.store.rail_button_size();
    for axis in [RailAxis::Vertical, RailAxis::Horizontal] {
        let slots = entry_rects(&rail, Rect::new(0.0, 0.0, 400.0, 400.0), size, axis);
        for pair in slots.windows(2) {
            assert_eq!(
                overlap_area(pair[0].rect, pair[1].rect),
                0.0,
                "{axis:?}: {:?} e {:?} sobrepõem-se",
                pair[0].id,
                pair[1].id
            );
        }
    }
}

/// ⛔⛔ **NADA se perde: a fila QUEBRA DE LINHA em vez de apagar chips.**
///
/// ⚠️ **O transbordo era MUDO.** A faixa blinda a tinta E o hit (`push_clip`) e o
/// `HitIndex::register` **descarta** um rect totalmente cortado ⇒ um chip a mais não ficava
/// truncado, ficava **inexistente**, sem nada no ecrã a dizê-lo. A auditoria de 2026-08-30 mediu
/// as células: a **1280 px** com o preset *Large* o *Undo* e o *Redo* desapareciam; com as colunas
/// arrastadas ao máximo (`DOCK_W_MAX = 720`) desapareciam **os dezasseis**.
///
/// ⛔ **E o gate que aqui estava media UMA célula** — 1366 px, preset `Small`, colunas por
/// omissão — e passava em todas as outras. *Uma varredura de uma célula não é uma varredura.*
#[test]
fn nothing_is_lost_at_any_width_preset_or_column_size() {
    let h = hero();
    for painter in [false, true] {
        let rail = ToolRail::new(
            NodeId(203),
            "Editor tools",
            ph2d_editor_core::screens::hero::left_rail::rail_entries(&h.store, painter),
        );
        let chips = rail
            .entries
            .iter()
            .filter(|e| e.node_id().is_some())
            .count();
        for w in [1280.0_f32, 1366.0, 1600.0, 1920.0] {
            for size in [
                RailButtonSize::Small,
                RailButtonSize::Medium,
                RailButtonSize::Large,
            ] {
                for dock in [220.0_f32, 308.0, 720.0] {
                    let area_w = w - dock * 2.0;
                    let content_w = (area_w - 8.0).max(0.0);
                    if content_w <= size.chip_px() {
                        continue; // área menor que um chip: a faixa nem se pinta
                    }
                    let lines = ph2d_editor_core::widget::horizontal_lines(&rail, content_w, size);
                    let content = Rect::new(0.0, 0.0, content_w, 1000.0);
                    let slots = entry_rects(&rail, content, size, RailAxis::Horizontal);
                    let painted = slots
                        .iter()
                        .filter(|s| s.id.is_some() && s.rect.x + s.rect.w <= content_w + 0.001)
                        .count();
                    assert_eq!(
                        painted,
                        chips,
                        "painter={painter} w={w} {size:?} dock={dock}: {} de {chips} chips caem \
                         FORA da faixa — a blindagem apaga-os sem aviso ({lines} linhas)",
                        chips - painted
                    );
                }
            }
        }
    }
}

/// **E a faixa cresce com as linhas** — senão a quebra só empurra os chips para fora por baixo.
#[test]
fn the_band_grows_with_the_number_of_lines() {
    let one = tool_bar::tool_bar_h(RailButtonSize::default(), 1);
    let two = tool_bar::tool_bar_h(RailButtonSize::default(), 2);
    assert!(two > one, "duas linhas não pedem mais altura que uma");
    let three = tool_bar::tool_bar_h(RailButtonSize::default(), 3);
    assert!(
        (three - two - (two - one)).abs() < 0.001,
        "o passo entre linhas não é constante: {one} {two} {three}"
    );
}
