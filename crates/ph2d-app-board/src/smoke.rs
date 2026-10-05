//! **As cenas de smoke do QUADRO** — `PH2D_BOARD_SMOKE=<n>`.
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | as abas: dois quadros já criados e o 1.º aberto, com uma grelha de rectângulos coloridos — a roda dá zoom à volta do cursor, arrastar move a vista, `Scene` devolve a cena |

use ph2d_board_model::{BoardOp, BoardSet, Element, ElementKind, Rgba};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_tokens::{ColorToken, Spacing};

/// O roteador desta família — o maior nível que o `match` de [`stage_armed_smokes`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_BOARD_SMOKE",
    max_level: 1,
}];

/// Encena o que o dono armou por variável de ambiente. Inerte sem ela.
pub fn stage_armed_smokes(hero: &mut HeroScreen) {
    let Some(level) = std::env::var("PH2D_BOARD_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
    else {
        return;
    };
    match level {
        1 => scene_two_boards(hero),
        _ => eprintln!("[board] PH2D_BOARD_SMOKE={level}: não há esta cena (1..=1)"),
    }
}

/// Cena 1 — dois quadros, o 1.º com uma grelha de 6×4 rectângulos nas cores de acento do tema.
fn scene_two_boards(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let first = set.create(default_name(1));
    set.create(default_name(2));
    let board = set.get_mut(first).expect("acabou de nascer");
    let tones = [
        ColorToken::Accent,
        ColorToken::Success,
        ColorToken::Warn,
        ColorToken::Danger,
    ];
    let cell = f64::from(Spacing::Xl4.px()) * 2.0;
    let gap = f64::from(Spacing::Xl2.px());
    for row in 0..4_u8 {
        for col in 0..6_u8 {
            let c = tones[usize::from((row + col) % 4)].resolve(hero.theme);
            let el = Element {
                id: board.doc.mint_id(),
                kind: ElementKind::Rect {
                    fill: Rgba([c.r, c.g, c.b, c.a]),
                },
                x: f64::from(col) * (cell + gap) - 3.0 * (cell + gap),
                y: f64::from(row) * (cell + gap) - 2.0 * (cell + gap),
                w: cell,
                h: cell,
                z: board.doc.z_on_top(),
                version: 0,
                nonce: 0,
                deleted: false,
            };
            BoardOp::Put(el).apply(&mut board.doc);
        }
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(first));
}

fn default_name(n: usize) -> String {
    ph2d_i18n::tr_with("board.tab.default_name", &[("n", &n)])
}
