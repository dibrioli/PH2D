//! W25 — o que se pinta DEPOIS das gavetas (os menus da barra de estilo, o aviso do topo) não tapa o
//! chrome do quadro: a barra curta, as gavetas, os painéis da direita, a própria barra de estilo. A
//! verdade é o `hit_index` (o que o clique alcança), como em `board_bar_place_tests`.

use super::*;
use ph2d_board_model::{Head, ShapeType, Stamp};

/// Os painéis da direita abertos (molduras, comentários, minimapa).
fn open_right_panels(t: &mut T) {
    for b in [NavButton::Frames, NavButton::Comments, NavButton::Minimap] {
        t.hero
            .apply_event(WidgetEvent::Click(board_nav::button_node(b)));
    }
    t.paint();
}

/// O chrome aberto de cada caso: a direita, ou uma gaveta da barra curta.
const CHROME: [&str; 2] = ["direita", "formas"];

fn open_chrome(t: &mut T, which: &str) {
    match which {
        "direita" => open_right_panels(t),
        "formas" if !t.hero.documents.live.shapes_open => {
            tap(t, Item::MoreShapes);
        }
        "modelos" => {
            tap(t, super::super::super::mind::TEMPLATES);
        }
        _ => {}
    }
}

/// Down + Up pelo despacho real no centro do rectângulo que o pintor registou para `item`; `false` se não
/// está pintado ou outro o tapa (o `T::click_bar` acha-o varrendo o ecrã de 2 em 2 px — ~350 mil consultas
/// por clique, o que punha este ficheiro a 9 min).
fn tap(t: &mut T, item: Item) -> bool {
    let id = node(item);
    let Some(r) = t.hero.hit_index.rect_for(id) else {
        return false;
    };
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    if t.hero.hit_index.hit(x, y) != Some(id) {
        return false;
    }
    let arena = bumpalo::Bump::new();
    let mut ev = t
        .hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(t.hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        t.hero.apply_event(e);
    }
    t.paint();
    true
}

/// Os encontros de cada rectângulo de `mine` (no ecrã) com o resto do que o clique alcança, mais `extra`.
fn meetings(t: &T, mine: &[(String, Rect)], own: &[NodeId], extra: &[Rect]) -> Vec<String> {
    let table = items();
    let others: Vec<(String, Rect)> = t
        .hero
        .hit_index
        .iter_registrations()
        .filter(|(id, _)| !own.contains(id))
        .map(|(id, r)| {
            let named = table.iter().find(|it| node(**it) == id);
            (
                named.map_or_else(|| format!("{id:?}"), |it| format!("{it:?}")),
                r,
            )
        })
        .chain(extra.iter().map(|r| ("pintado".to_owned(), *r)))
        .collect();
    let mut out = Vec::new();
    for (it, r) in mine {
        for (who, o) in &others {
            if meets(*r, *o) {
                out.push(format!("{it} {r:?} sobre {who} {o:?}"));
            }
        }
    }
    out
}

/// O minimapa, se o caso o abriu (pinta-se sem se registar inteiro).
fn minimap(t: &T, which: &str) -> Vec<Rect> {
    let c = t.hero.last_canvas;
    let map = board_nav::minimap_rect(board_nav::corner(&t.hero, c));
    (which == "direita").then_some(map).into_iter().collect()
}

/// Um elemento da família `kind` nascido em `a`→`b` e seleccionado por um clique nele.
fn make(t: &mut T, kind: &str, a: (f32, f32), b: (f32, f32)) -> bool {
    use ph2d_board_edit::Tool;
    let (tool, grab) = match kind {
        "forma" => return select_box(t, a, b),
        // A seta nasce entre duas caixas (no vazio a ferramenta não a cria).
        "seta" => (Tool::Connector, ((a.0 + b.0) / 2.0, a.1 + 20.0)),
        _ => (Tool::Shape(ShapeType::Sticky), a),
    };
    let ends = [(a.0, a.1 + 20.0), (b.0, a.1 + 20.0)];
    if kind == "seta" {
        let c = t.hero.last_canvas;
        let on_board = |p: (f32, f32)| {
            p.0 > c.x && p.0 < c.x + c.w && t.hero.hit_index.hit(p.0, p.1).is_none()
        };
        if ![(a.0 - 20.0, a.1), (b.0 + 20.0, a.1 + 40.0)]
            .into_iter()
            .all(on_board)
        {
            return false;
        }
        select_box(t, (a.0 - 20.0, a.1), (a.0 + 20.0, a.1 + 40.0));
        select_box(t, (b.0 - 20.0, a.1), (b.0 + 20.0, a.1 + 40.0));
    }
    ed(t).tool = tool;
    if kind == "seta" {
        t.drag(ends[0], ends[1]);
    } else {
        t.drag(a, a);
    }
    ed(t).tool = Tool::Select;
    t.key(board_keys::BoardKey::Escape, NO_MODS, None);
    t.drag(grab, grab);
    ed(t).selection().len() == 1
}

/// O botão da barra que abre o menu de `kind`, e os itens do menu.
fn menu_of(kind: &str) -> (Item, Vec<Item>) {
    match kind {
        "forma" => (
            Item::Reshape,
            ShapeType::ALL.iter().map(|s| Item::Pick(*s)).collect(),
        ),
        "seta" => (
            Item::MoreHeads(1),
            Head::ALL.iter().map(|h| Item::Head(1, *h)).collect(),
        ),
        _ => (
            Item::Fac(FacItem::Stamps),
            (0..Stamp::ALL.len())
                .map(|i| Item::Fac(FacItem::Stamp(i)))
                .collect(),
        ),
    }
}

/// ⭐ (b) O menu `kind` da barra de estilo — o «Forma», as pontas, os carimbos — aberto com uma gaveta ou os
/// painéis da direita abertos, em sítios do quadro de cada canto: nenhum botão do menu cai sobre outro
/// clicável (as gavetas, os painéis, a barra curta, as outras linhas da própria barra de estilo). A grelha é
/// a que apanhou as mutações (prova W25: o menu ancorado ao botão, e sem o chrome) — cada caso custa ~10
/// pinturas em debug, e 216 casos levavam 9 min.
fn no_menu_covers_the_chrome(kind: &str) {
    let (opener, menu) = menu_of(kind);
    let own: Vec<NodeId> = menu.iter().map(|it| node(*it)).collect();
    let (mut bad, mut seen) = (Vec::new(), 0);
    for w in [1366.0, 800.0] {
        for which in CHROME {
            for (fx, fy) in [(0.2, 0.15), (0.6, 0.7), (0.8, 0.82), (0.88, 0.5)] {
                let mut t = T::new();
                t.vp_w = w;
                t.paint();
                if which == "direita" {
                    open_chrome(&mut t, which);
                }
                let (x, y) = t.at(fx, fy);
                if !make(&mut t, kind, (x, y), (x + 120.0, y + 50.0)) {
                    continue;
                }
                if which != "direita" {
                    open_chrome(&mut t, which);
                }
                if !tap(&mut t, opener) {
                    continue;
                }
                let mine: Vec<(String, Rect)> = menu
                    .iter()
                    .filter_map(|it| {
                        t.hero
                            .hit_index
                            .rect_for(node(*it))
                            .map(|r| (format!("{it:?}"), r))
                    })
                    .collect();
                if mine.is_empty() {
                    continue;
                }
                seen += 1;
                bad.extend(
                    meetings(&t, &mine, &own, &minimap(&t, which))
                        .into_iter()
                        .map(|m| format!("{w} {kind} {which} {fx},{fy}: {m}")),
                );
            }
        }
    }
    assert!(seen >= 4, "o menu de {kind} abriu só {seen} vezes");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_reshape_menu_never_covers_the_chrome() {
    no_menu_covers_the_chrome("forma");
}

#[test]
fn the_heads_menu_never_covers_the_chrome() {
    no_menu_covers_the_chrome("seta");
}

#[test]
fn the_stamps_menu_never_covers_the_chrome() {
    no_menu_covers_the_chrome("nota");
}

/// ⭐ (a) O aviso do topo (o com botão, W23 B15, e o da recusa) com cada gaveta ou os painéis da direita
/// abertos, da janela larga à estreita: não toca a barra curta, uma gaveta nem um painel enquanto houver no
/// quadro um lugar livre que o leve (procurado à força, [`free_spot`]); antes ficava sempre no topo e tapava
/// o painel «Frames» com lugar livre ao lado.
#[test]
fn the_top_notice_is_never_under_the_chrome() {
    let (mut bad, mut no_room) = (Vec::new(), Vec::new());
    let mut seen = 0;
    for w in [1366.0, 800.0, 600.0] {
        for which in ["nada", "direita", "formas", "modelos", "notas", "caneta"] {
            for alert in [true, false] {
                let mut t = T::new();
                t.vp_w = w;
                t.paint();
                match which {
                    "notas" => {
                        tap(
                            &mut t,
                            Item::Tool(ph2d_board_edit::Tool::Shape(ShapeType::Sticky)),
                        );
                    }
                    "caneta" => {
                        tap(&mut t, super::super::super::pen::PEN_TOOL);
                    }
                    _ => open_chrome(&mut t, which),
                }
                let live = &mut t.hero.documents.live;
                if alert {
                    live.alert = Some(crate::documents::alert::Alert::new(
                        "Rui mentioned you: @Ana look at the frame".into(),
                        None,
                    ));
                } else {
                    live.notice = Some(("board.images.unreadable", std::time::Instant::now()));
                }
                t.paint();
                let Some(r) = crate::screens::hero::board_alert::BANNER.with(std::cell::Cell::get)
                else {
                    continue;
                };
                seen += 1;
                // O que se pinta sem se registar inteiro: o minimapa e o corpo dos painéis.
                let painted =
                    crate::screens::hero::board_nav::fit::painted(&t.hero.documents.live.nav);
                let extra: Vec<Rect> = minimap(&t, which)
                    .into_iter()
                    .chain(painted.into_iter().filter(|p| *p != r))
                    .collect();
                let hits = meetings(&t, &[("aviso".into(), r)], &[], &extra);
                if hits.is_empty() {
                    continue;
                }
                // Tapar é o último recurso: só quando NENHUM lugar do quadro leva o aviso.
                match free_spot(&t, (r.w, r.h), &extra) {
                    Some(spot) => bad
                        .extend(hits.into_iter().map(|m| {
                            format!("{w} {which} alert={alert}: {m} (livre em {spot:?})")
                        })),
                    None => no_room.push(format!("{w} {which}")),
                }
            }
        }
    }
    eprintln!("sem lugar livre (o aviso tapa, por cima): {no_room:?}");
    assert!(seen > 0, "nenhum aviso pintado");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Um lugar `w×h` do quadro, à folga `Spacing::Lg` de todo o clicável e de `extra` — procurado à força,
/// de 4 em 4 px, sem a conta do produto.
fn free_spot(t: &T, (w, h): (f32, f32), extra: &[Rect]) -> Option<(f32, f32)> {
    let c = t.hero.last_canvas;
    let m = ph2d_tokens::Spacing::Lg.px();
    let taken: Vec<Rect> = t
        .hero
        .hit_index
        .iter_registrations()
        .map(|(_, r)| r)
        .chain(extra.iter().copied())
        .map(|r| Rect::new(r.x - m, r.y - m, r.w + 2.0 * m, r.h + 2.0 * m))
        .collect();
    let mut y = c.y;
    while y + h <= c.y + c.h {
        let mut x = c.x;
        while x + w <= c.x + c.w {
            let spot = Rect::new(x, y, w, h);
            if !taken.iter().any(|k| meets(spot, *k)) {
                return Some((x, y));
            }
            x += 4.0;
        }
        y += 4.0;
    }
    None
}
