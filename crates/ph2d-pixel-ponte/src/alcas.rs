//! **As alças das guias no canvas** (P13 U-Guias): premir perto de um ponto de fuga ou da origem
//! da iso agarra-a; arrastar é PRÉVIA (nenhum passo); soltar devolve as guias novas para UM
//! `DocOp::SetGuias`. Estado do gesto (um por fio de UI), como o registo dos desenhados.

use std::cell::RefCell;

use ph2d_editor_core::pixel_panel::PixelGuides;
use ph2d_pixel_core::{DocOp, Guias, Op, Sprite};
use ph2d_pixel_guia::{Alca, alcas, guias_iniciais, move_alca};
use ph2d_vector::{Affine, Point};

/// O alcance do clique sobre uma alça, em píxeis de ecrã.
pub const ALCA_GRAB_PX: f64 = 10.0; // LITERAL-PX-OK: alcance do clique sobre a alça de uma guia

struct Arrasto {
    key: u32,
    alca: Alca,
    antes: Guias,
    agora: Guias,
    to_screen: Affine,
}

thread_local! {
    static ARRASTO: RefCell<Option<Arrasto>> = const { RefCell::new(None) };
}

/// As guias de omissão da escolha `k` do menu para um sprite `size`.
#[must_use]
pub fn do_menu(k: PixelGuides, size: (u32, u32)) -> Guias {
    match k {
        PixelGuides::None => Guias::default(),
        PixelGuides::Perspective1 => guias_iniciais(1, false, size),
        PixelGuides::Perspective2 => guias_iniciais(2, false, size),
        PixelGuides::Perspective3 => guias_iniciais(3, false, size),
        PixelGuides::Isometric => guias_iniciais(0, true, size),
    }
}

/// A alça de `g` sob o ponto de ecrã `p` (a mais perto, dentro de [`ALCA_GRAB_PX`]).
#[must_use]
pub fn sob(g: &Guias, to_screen: Affine, p: (f32, f32)) -> Option<Alca> {
    let p = Point::new(f64::from(p.0), f64::from(p.1));
    alcas(g)
        .into_iter()
        .map(|(a, c)| {
            let s = to_screen * Point::new(f64::from(c[0]), f64::from(c[1]));
            (a, s.distance(p))
        })
        .filter(|(_, d)| *d <= ALCA_GRAB_PX)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(a, _)| a)
}

/// Premir em `p`: agarra a alça sob ele (`true`) ou não faz nada (`false`).
pub fn agarra(key: u32, g: &Guias, to_screen: Affine, p: (f32, f32)) -> bool {
    let Some(alca) = sob(g, to_screen, p) else {
        return false;
    };
    ARRASTO.with_borrow_mut(|a| {
        *a = Some(Arrasto {
            key,
            alca,
            antes: *g,
            agora: *g,
            to_screen,
        });
    });
    true
}

/// Arrastar até `p` (ecrã): a prévia anda, ao meio píxel; `false` se nada está agarrado.
pub fn arrasta(p: (f32, f32)) -> bool {
    ARRASTO.with_borrow_mut(|a| {
        let Some(a) = a.as_mut() else {
            return false;
        };
        let c = a.to_screen.inverse() * Point::new(f64::from(p.0), f64::from(p.1));
        let meio = |v: f64| ((v * 2.0).round() / 2.0) as f32;
        a.agora = move_alca(&a.antes, a.alca, [meio(c.x), meio(c.y)]);
        true
    })
}

/// Soltar: `Some((key, passo))` se algo estava agarrado; `passo` = o `SetGuias` (`None` se a alça
/// não mexeu). `sprite` dá o documento da chave.
pub fn solta<'a>(sprite: impl FnOnce(u32) -> Option<&'a Sprite>) -> Option<(u32, Option<Op>)> {
    let a = ARRASTO.with_borrow_mut(Option::take)?;
    let op = (a.agora != a.antes)
        .then_some(a.agora)
        .zip(sprite(a.key))
        .and_then(|(g, s)| op(s, g));
    Some((a.key, op))
}

/// As guias a MOSTRAR do documento `key`: a prévia do arrasto, ou as do documento.
#[must_use]
pub fn vistas(key: u32, g: &Guias) -> Guias {
    ARRASTO.with_borrow(|a| match a {
        Some(a) if a.key == key => a.agora,
        _ => *g,
    })
}

/// O passo que põe as guias `g` no sprite `s` (`None` = já são essas: nenhum passo).
#[must_use]
pub fn op(s: &Sprite, g: Guias) -> Option<Op> {
    (*s.guias() != g).then(|| Op::Doc(Box::new(DocOp::SetGuias(g))))
}

/// O passo da escolha `k` de View ▸ Guides no sprite `s`.
#[must_use]
pub fn op_do_menu(s: &Sprite, k: PixelGuides) -> Option<Op> {
    op(s, do_menu(k, (s.width(), s.height())))
}
pub use ph2d_pixel_guia::alvo_do_lapis;
