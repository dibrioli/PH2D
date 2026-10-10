//! Cena 10 — A EQUIPA E A SAÍDA (W8, `docs/MiroClone/06_equipa_e_saida.md` §5), módulo filho do roteador
//! [`super`]: um lançamento em três frames de TOPO (Problem · Ideas · Plan — os diapositivos), um
//! comentário do Rui por ler numa nota e um da Ana já resolvido, uma versão «Kick-off» guardada antes da
//! última ideia, e por baixo uma nota LARGA por lição na ORDEM REAL (comentar · versões · apresentar ·
//! exportar · abrir um `.excalidraw`). ⚠️ Sem frame à volta (os três deixariam de ser diapositivos): a
//! margem do que os painéis tapam é a caixa que a cena manda enquadrar (`board_nav::show_box`). O gate `every_team_lesson_does_what_it_says` conduz cada lição pelo ECRÃ.

use super::default_name;
use ph2d_board_model::{
    Anchor, Board, BoardOp, BoardSet, Connector, Element, ElementId, End, Pin, Rgba, RichText,
    Route, STICKY_COLORS, STICKY_SIDE, Shape, ShapeType, Style, ThreadId,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "só o gate da cena lê as lições")
)]
pub(crate) struct Lessons {
    pub frames: [ElementId; 3],
    pub builds: ElementId,
    pub rui: ThreadId,
    pub ana: ThreadId,
    pub lessons: [ElementId; 5],
    /// O que a cena enquadra ao nascer: tudo, mais a margem do que os painéis tapam.
    pub view: [f64; 4],
}

/// A margem do que os painéis e a barra de estado tapam. ⛔ A da cena 9 (330) não chega: ela é em
/// unidades do MUNDO e os painéis em px — a 1.ª foto desta cena (largura 2 490, zoom 0,73) tinha os dois
/// painéis a tapar ~440 de cada lado. Com a cena a 1 640 de largo e esta margem, o zoom de nascença é
/// ~0,76 e os painéis tapam ~408 < 450.
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
/// Os frames: lado a lado, com o vão de uma nota pequena entre as notas.
const FRAME: [f64; 2] = [520.0, 320.0];
const GAP: f64 = 40.0;
/// As notas dos frames (menores que a de nascença: três cabem num frame).
const SMALL: f64 = 150.0;
pub(super) const INK: Rgba = Rgba([30, 30, 30, 255]);

pub(super) fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

pub(super) fn put(b: &mut Board, el: Element) -> ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

pub(super) fn note(
    b: &mut Board,
    key: &str,
    color: usize,
    bx: [f64; 4],
    wide: bool,
    home: Option<ElementId>,
) -> ElementId {
    let c = Rgba(STICKY_COLORS[color]);
    let shape = Shape {
        kind: if wide {
            ShapeType::StickyWide
        } else {
            ShapeType::Sticky
        },
        style: Style::new(Some(c), None, c.readable_ink()),
        text: RichText::plain(&tr(key)),
    };
    let mut el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
    el.frame = home;
    put(b, el)
}

pub(super) fn boxed(b: &mut Board, key: &str, bx: [f64; 4], home: ElementId) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(Rgba(STICKY_COLORS[12])), Some(INK), INK),
        text: RichText::plain(&tr(key)),
    };
    let mut el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), shape, bx);
    el.frame = Some(home);
    put(b, el)
}

pub(super) fn arrow(b: &mut Board, from: ElementId, to: ElementId) {
    let c = Connector::new(
        End::Bound {
            target: from,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: to,
            anchor: Anchor::Center,
        },
        Route::Straight,
        Style::new(None, Some(INK), INK),
    );
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    put(b, el);
}

/// Monta o lançamento em `b`.
pub(crate) fn build(b: &mut Board) -> Lessons {
    let names = [
        "board.smoke.team.problem",
        "board.smoke.team.ideas",
        "board.smoke.team.plan",
    ];
    let frames: [ElementId; 3] = std::array::from_fn(|i| {
        let x = i as f64 * (FRAME[0] + GAP);
        super::organize::frame(
            &mut b.doc,
            &tr(names[i]),
            [x, 0.0, FRAME[0], FRAME[1]],
            None,
        )
    });
    let (s, y0) = (SMALL, 90.0);
    let builds = note(
        b,
        "board.smoke.team.n_builds",
        1,
        [GAP, y0, s, s],
        false,
        Some(frames[0]),
    );
    note(
        b,
        "board.smoke.team.n_reviews",
        3,
        [2.0 * GAP + s, y0, s, s],
        false,
        Some(frames[0]),
    );
    let ix = FRAME[0] + GAP;
    let col = |k: f64| ix + 20.0 + k * (s + 15.0);
    note(
        b,
        "board.smoke.team.n_cache",
        5,
        [col(0.0), y0, s, s],
        false,
        Some(frames[1]),
    );
    note(
        b,
        "board.smoke.team.n_pair",
        12,
        [col(1.0), y0, s, s],
        false,
        Some(frames[1]),
    );
    let px = 2.0 * (FRAME[0] + GAP);
    let w = 120.0;
    let m = boxed(
        b,
        "board.smoke.team.measure",
        [px + 30.0, 130.0, w, 70.0],
        frames[2],
    );
    let c = boxed(
        b,
        "board.smoke.team.cache",
        [px + 200.0, 130.0, w, 70.0],
        frames[2],
    );
    let ship = boxed(
        b,
        "board.smoke.team.ship",
        [px + 370.0, 130.0, w, 70.0],
        frames[2],
    );
    arrow(b, m, c);
    arrow(b, c, ship);
    // R1 — a versão de antes da última ideia (restaurá-la mostra o quadro sem ela).
    // As horas são as de AGORA recuadas (a 1.ª foto dos estados lia «20736 d ago»: o instante 0 é 1970).
    let now = board_nav::team::now_unix();
    b.save_version(tr("board.smoke.team.version"), now - 26 * 3600);
    note(
        b,
        "board.smoke.team.n_small",
        8,
        [col(2.0), y0, s, s],
        false,
        Some(frames[1]),
    );
    // K2/K6 — o Rui comentou a nota dos builds (por ler); a Ana aprovou o plano (resolvido).
    let at = ph2d_board_model::Element::center(b.doc.get(builds).expect("acabou de nascer"));
    let rui = b
        .team
        .open(
            Pin::at(&b.doc, Some(builds), [at[0] + 45.0, at[1] - 45.0]),
            "Rui",
            &tr("board.smoke.team.c_rui"),
            now - 2 * 3600,
        )
        .expect("texto");
    let ship_at = b.doc.get(ship).expect("acabou de nascer").center();
    let ana = b
        .team
        .open(
            Pin::at(&b.doc, Some(ship), ship_at),
            "Ana",
            &tr("board.smoke.team.c_ana"),
            now - 5 * 3600,
        )
        .expect("texto");
    b.team.set_resolved(ana, true);
    // As lições, por baixo, na ordem real.
    let y = FRAME[1] + GAP;
    let side = STICKY_SIDE;
    let lessons: [ElementId; 5] = std::array::from_fn(|i| {
        let key = format!("board.smoke.team.l{}", i + 1);
        let x = i as f64 * (side + GAP);
        note(b, &key, 1, [x, y, side, side], false, None)
    });
    let right = 3.0 * FRAME[0] + 2.0 * GAP;
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(&tr("board.smoke.team.title")),
    };
    // O título à vista por cima dos frames (a 1.ª foto tinha-o no canto, debaixo do painel).
    // ⛔ Nada travado: o quadro marca um travado com um cadeado (a foto dos estados viu três soltos).
    let t = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, right, 40.0],
    );
    put(b, t);
    Lessons {
        frames,
        builds,
        rui,
        ana,
        lessons,
        view: [
            -MARGIN_X,
            -80.0 - MARGIN_Y,
            right + MARGIN_X,
            y + side + MARGIN_Y,
        ],
    }
}

/// A cena 10 no hero; devolve os ids dela (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_team(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let l = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, l.view);
    l
}

#[cfg(test)]
#[path = "smoke_team_tests.rs"]
mod tests;
