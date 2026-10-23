//! W24 — a barra de estilo, a ÚLTIMA a pintar-se, não tapa nenhum outro chrome do quadro: as gavetas
//! da barra curta, os painéis da navegação e da equipa, a barra curta numa janela minúscula. A
//! verdade é o que o ECRÃ registou para o clique (o `hit_index`), não a lista que a colocação lê.

use super::*;
use crate::screens::hero::board_nav::{self, NavButton};

/// A barra de estilo do último quadro.
fn style_bar() -> Vec<(Item, Rect)> {
    super::super::STYLE_BAR.with(|v| v.borrow().clone())
}

fn ed(t: &mut T) -> &mut ph2d_board_edit::Editor {
    t.hero.documents.live.editor.as_mut().expect("o editor do quadro")
}

fn meets(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Um rectângulo desenhado de `a` para `b` (px de ecrã) e seleccionado por um clique na borda de cima;
/// `false` quando um painel por cima dele o não deixa nascer ou seleccionar.
fn select_box(t: &mut T, a: (f32, f32), b: (f32, f32)) -> bool {
    ed(t).tool = ph2d_board_edit::Tool::Shape(ph2d_board_model::ShapeType::Rectangle);
    t.drag(a, b);
    ed(t).tool = ph2d_board_edit::Tool::Select;
    t.key(board_keys::BoardKey::Escape, NO_MODS, None);
    let edge = ((a.0 + b.0) / 2.0, a.1);
    t.drag(edge, edge);
    ed(t).selection().len() == 1
}

const NO_MODS: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};

/// Cada botão da barra de estilo contra cada outro rectângulo clicável do ecrã e contra `extra`
/// (o que se pinta sem se registar); devolve os encontros.
fn collisions(t: &T, extra: &[Rect]) -> Vec<String> {
    let bar = style_bar();
    let own: Vec<NodeId> = bar.iter().map(|(it, _)| node(*it)).collect();
    let others: Vec<(String, Rect)> = t
        .hero
        .hit_index
        .iter_registrations()
        .filter(|(id, _)| !own.contains(id))
        .map(|(id, r)| (format!("{id:?}"), r))
        .chain(extra.iter().map(|r| ("pintado".to_owned(), *r)))
        .collect();
    let mut out = Vec::new();
    for (it, r) in &bar {
        for (who, o) in &others {
            if meets(*r, *o) {
                out.push(format!("{it:?} {r:?} sobre {who} {o:?}"));
            }
        }
    }
    out
}

/// As caixas postas por onde a barra dela cruza uma gaveta (junto da barra curta, de cima a baixo).
fn spots(t: &T) -> Vec<((f32, f32), (f32, f32))> {
    let c = t.hero.last_canvas;
    let right = toolbar_rects(c)
        .iter()
        .map(|(_, r)| r.x + r.w)
        .fold(0.0, f32::max);
    let mut out = Vec::new();
    for fy in [0.15, 0.3, 0.45, 0.6, 0.75] {
        let y = c.y + c.h * fy;
        out.push(((right + 40.0, y), (right + 140.0, y + 60.0)));
    }
    out
}

/// ⭐ (a) Cada gaveta da barra curta aberta, com uma caixa seleccionada junto dela: a barra de estilo
/// não lhe tapa um botão (antes pintava-se por cima — classe anterior à W23).
#[test]
fn the_style_bar_never_covers_an_open_drawer() {
    let drawers: [(&str, Item); 4] = [
        ("formas", Item::MoreShapes),
        ("notas", Item::Tool(ph2d_board_edit::Tool::Shape(ph2d_board_model::ShapeType::Sticky))),
        ("caneta", super::super::pen::PEN_TOOL),
        ("modelos", super::super::mind::TEMPLATES),
    ];
    let mut bad = Vec::new();
    let mut seen = std::collections::BTreeMap::new();
    for w in [1366.0, 1024.0, 800.0] {
        for (name, opener) in drawers {
            let mut probe = T::new();
            probe.vp_w = w;
            probe.paint();
            for (a, b) in spots(&probe) {
                let mut t = T::new();
                t.vp_w = w;
                t.paint();
                if !select_box(&mut t, a, b) {
                    continue;
                }
                t.click_bar(opener);
                let open = match name {
                    "formas" => t.hero.documents.live.shapes_open,
                    "modelos" => super::super::mind::sync_gallery(&mut t.hero.documents.live),
                    "notas" => super::super::notes::flyout_open(ed(&mut t).tool),
                    _ => super::super::pen::flyout_open(ed(&mut t).tool),
                };
                assert!(open, "{w} {name}: a gaveta não abriu");
                if style_bar().is_empty() {
                    // A ferramenta que abre a gaveta largou a selecção: nada a tapar.
                    continue;
                }
                *seen.entry(name).or_insert(0) += 1;
                bad.extend(collisions(&t, &[]).into_iter().map(|c| format!("{w} {name}: {c}")));
            }
        }
    }
    eprintln!("caixas com a barra pintada, por gaveta: {seen:?}");
    assert_eq!(seen.len(), drawers.len(), "uma gaveta nunca teve a barra ao lado: {seen:?}");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// ⭐ (a) Os painéis da direita (molduras, comentários, Live, minimapa) e os botões do canto abertos,
/// com a caixa seleccionada junto deles: a barra de estilo foge de todos.
#[test]
fn the_style_bar_never_covers_the_right_hand_panels() {
    let mut bad = Vec::new();
    let mut seen = 0;
    for w in [1366.0, 1024.0, 800.0] {
        for fy in [0.08, 0.3, 0.55, 0.8] {
            for fx in [0.55, 0.75, 0.88] {
                let mut t = T::new();
                t.vp_w = w;
                t.paint();
                for b in [NavButton::Frames, NavButton::Comments, NavButton::Minimap] {
                    t.hero.apply_event(WidgetEvent::Click(board_nav::button_node(b)));
                }
                t.paint();
                let c = t.hero.last_canvas;
                let (x, y) = (c.x + c.w * fx, c.y + c.h * fy);
                if !select_box(&mut t, (x, y), (x + 90.0, y + 50.0)) || style_bar().is_empty() {
                    continue;
                }
                seen += 1;
                let corner = board_nav::corner(&t.hero, c);
                let map = board_nav::minimap_rect(corner);
                bad.extend(collisions(&t, &[map]).into_iter().map(|m| format!("{w} {fx},{fy}: {m}")));
            }
        }
    }
    assert!(seen > 0, "nenhuma caixa ficou seleccionada");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// ⭐ (b) Numa janela minúscula (a sala à direita da barra curta mais estreita que um grupo), a
/// barra de estilo não passa à esquerda por cima da barra curta, nem sai do quadro.
#[test]
fn in_a_tiny_window_the_style_bar_stays_right_of_the_toolbar() {
    let mut bad = Vec::new();
    for (w, h) in [(420.0, 560.0), (360.0, 480.0), (320.0, 420.0)] {
        let mut t = T::new();
        (t.vp_w, t.vp_h) = (w, h);
        t.paint();
        let c = t.hero.last_canvas;
        let tools = toolbar_rects(c);
        let right = tools.iter().map(|(_, r)| r.x + r.w).fold(0.0, f32::max);
        let x = right + (c.x + c.w - right) * 0.3;
        let y = c.y + c.h * 0.4;
        assert!(select_box(&mut t, (x, y), (x + 60.0, y + 40.0)), "{w}×{h}");
        let bar = style_bar();
        assert!(!bar.is_empty(), "{w}×{h}: sem barra");
        for (it, r) in &bar {
            if r.x <= right || r.x + r.w > c.x + c.w || r.y < c.y || r.y + r.h > c.y + c.h {
                bad.push(format!("{w}×{h}: {it:?} {r:?} (ferramentas até {right}, quadro {c:?})"));
            }
        }
        bad.extend(collisions(&t, &[]).into_iter().map(|m| format!("{w}×{h}: {m}")));
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
