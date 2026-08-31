//! ⭐⭐⭐ **A FILA DE FERRAMENTAS NÃO CRESCE** — o que não cabe vai para o `⋯`.
//!
//! > Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir
//! > perdendo espaço.»*
//!
//! # ⛔⛔ O defeito, com número
//!
//! A faixa resolvia o transbordo **crescendo**: `54 → 108 px` no iPad 11 e no iPad mini, no
//! instante em que o pincel entrava em mãos (10 entradas em repouso, **18** com o Painter). Isso é
//! `−3,3` pontos de área de desenho, **permanentes**, justamente quando o ecrã faz falta
//! (`docs/UI_New_and_Simple/medicoes/06_o_orcamento_de_ecra_em_tablet.md`).
//!
//! ⚠️ **A terceira saída continua fora:** encolher o chip mente sobre o preset de tamanho que o
//! artista escolheu.
//!
//! # ⚠️ E o gate carrega num PIXEL
//!
//! Um chip dentro do menu de transbordo pode estar **morto sob o dedo** — é a família que este
//! repo já pagou nos quatro chips do vetor. Um `apply_event(Click(id))` sintético passaria com ele
//! morto: salta exactamente a metade que falha (o `HitIndex` resolver o ponto).

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{HeroScreen, tool_bar};
use ph2d_editor_core::widget::RailButtonSize;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

/// Os alvos onde a faixa dobrava. ⚠️ O 12,9" está aqui como **controlo**: nele ela nunca dobrou, e
/// um gate que só medisse os pequenos não distinguiria «curado» de «nunca aconteceu».
const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

fn hero(w: f32, h: f32) -> (HeroScreen, Rect) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.view.legacy_chrome = false;
    (hero, Rect::new(0.0, 0.0, w, h))
}

fn paint(h: &mut HeroScreen, vp: Rect) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..2 {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, vp, &mut scene, &mut text);
    }
}

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn click_at(h: &mut HeroScreen, r: Rect) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, x, y),
        pointer(PointerKind::Up, x, y),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

/// ⭐⭐⭐ **A faixa é UMA linha em todo tablet, com o pincel em mãos ou sem ele.**
#[test]
fn the_tool_bar_is_one_line_on_every_tablet() {
    for (name, w, h) in TABLETS {
        for painter in [false, true] {
            let (hero, _) = hero(w, h);
            let area_w = w - 612.0; // as duas colunas, abertas
            let (fits, over) = tool_bar::bar_split(&hero.store, painter, false, area_w);
            let lines = ph2d_editor_core::widget::horizontal_lines(
                &fits,
                area_w - 16.0,
                RailButtonSize::Small,
            );
            println!(
                "{name:11} pincel={painter:5} cabem={:2} transbordam={:2} linhas={lines}",
                fits.entries.len(),
                over.len()
            );
            assert_eq!(
                lines, 1,
                "{name} pincel={painter}: a fila precisa de {lines} linhas — ela voltou a crescer"
            );
        }
    }
}

/// ⭐⭐ **E o que transborda vai TODO para o `⋯`** — nada se perde pelo caminho.
#[test]
fn nothing_is_dropped_between_the_bar_and_the_dots() {
    for (name, w, h) in TABLETS {
        for painter in [false, true] {
            let (hero, _) = hero(w, h);
            let area_w = w - 612.0;
            let full = tool_bar::bar_rail(&hero.store, painter, false);
            let (fits, over) = tool_bar::bar_split(&hero.store, painter, false, area_w);
            let ids_of = |r: &[ph2d_editor_core::widget::ToolRailEntry]| {
                r.iter()
                    .filter_map(ph2d_editor_core::widget::ToolRailEntry::node_id)
                    .collect::<Vec<_>>()
            };
            let mut seen = ids_of(&fits.entries);
            let dots = seen
                .iter()
                .position(|i| *i == ph2d_editor_core::ids::TOOL_BAR_OVERFLOW);
            if let Some(i) = dots {
                seen.remove(i);
            }
            seen.extend(ids_of(&over));
            for id in ids_of(&full.entries) {
                assert!(
                    seen.contains(&id),
                    "{name} pincel={painter}: um chip desapareceu entre a fila e o `⋯`"
                );
            }
            assert_eq!(
                dots.is_some(),
                !over.is_empty(),
                "{name} pincel={painter}: o `⋯` e o transbordo discordam — ou um chip morto, ou \
                 verbos inalcançáveis"
            );
        }
    }
}

/// ⭐⭐⭐ **Carregar no `⋯` abre o resto, e carregar num deles serve o verbo e FECHA.**
#[test]
fn the_dots_open_the_rest_and_a_pick_closes_them() {
    // O iPad mini com o pincel: é onde o transbordo de facto acontece.
    let (mut h, vp) = hero(1133.0, 744.0);
    // ⚠️ **O Painter em mãos é a condição do transbordo** — e ele lê-se do `image_edit`, que é o
    // que a shell espelha (`rail_shows_painter_tools`). Sem isto o gate mede a fila de repouso, que
    // cabe em toda parte.
    h.image_edit.mode_on = true;
    h.image_edit.active_tool_id = Some("painter");
    paint(&mut h, vp);
    let dots = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::TOOL_BAR_OVERFLOW)
        .expect("controlo: o `⋯` não foi registado — não há transbordo neste alvo, e o gate mede o vazio");

    click_at(&mut h, dots);
    assert_eq!(
        h.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ToolBarOverflow),
        "carregar no `⋯` não abriu o transbordo"
    );

    // …e o corpo dele regista os chips que sobraram.
    paint(&mut h, vp);
    let over = h.store.tool_overflow().to_vec();
    let first = over
        .iter()
        .find_map(ph2d_editor_core::widget::ToolRailEntry::node_id)
        .expect("controlo: o transbordo publicado está vazio");
    let chip = h
        .hit_index
        .rect_for(first)
        .expect("um chip do transbordo não foi registado — ele está MORTO sob o dedo");

    click_at(&mut h, chip);
    assert!(
        h.store.context_menu().is_none(),
        "escolher um chip do transbordo não fechou o menu"
    );
}
