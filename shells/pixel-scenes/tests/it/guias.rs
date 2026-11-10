//! **P13 U-Guias pelo caminho real**: View ▸ Guides põe as guias no documento (um passo), a alça
//! do ponto de fuga arrasta-se com o ponteiro (prévia sem passo; soltar = um passo, Ctrl+Z volta),
//! e o Shift da Linha e do lápis segue a guia — sem guias, o Shift de hoje, byte a byte.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_editor_core::pixel_panel::{PixelEditAction as E, PixelGuides as G, PixelRequest};
use ph2d_pixel_core::raster::{Cell, line_cells, snap_pixel_angle};
use ph2d_pixel_core::{Guias, Sprite};
use ph2d_vector::Point;
use winit::keyboard::KeyCode;

use crate::comum::{Palco, canvas};

const SHIFT: Mods = Mods {
    shift: true,
    ctrl: false,
    alt: false,
};

fn guias(p: &Palco) -> Guias {
    *p.sprite().guias()
}

/// O ponto de ecrã do ponto `q` da imagem (células contínuas).
fn ecra(p: &Palco, q: [f32; 2]) -> (f32, f32) {
    let m = input::to_screen(
        p.sim.world(),
        p.bits,
        p.app.doc(p.key).expect("doc"),
        &p.cam,
        crate::comum::WINDOW,
        crate::comum::PPM,
    )
    .expect("afim");
    let s = m * Point::new(f64::from(q[0]), f64::from(q[1]));
    (s.x as f32, s.y as f32)
}

fn arrasta_ecra(p: &mut Palco, de: (f32, f32), para: (f32, f32), m: Mods) {
    let c = canvas(&p.sim, &p.cam, p.bits);
    assert_eq!(
        input::press(&mut p.app, &c, de, Button::Primary, m),
        Press::Done
    );
    assert!(input::moved(&mut p.app, &c, para, m));
    assert!(input::release(&mut p.app));
    if p.app.live_shape().is_some() {
        p.tecla(KeyCode::Enter, false, false);
    }
    p.quadro();
}

fn arrasta_celulas(p: &mut Palco, o: Cell, d: Cell) {
    let (de, para) = (centro(p, o), centro(p, d));
    arrasta_ecra(p, de, para, SHIFT);
}

fn centro(p: &Palco, c: Cell) -> (f32, f32) {
    crate::comum::at(&p.app, &p.sim, &p.cam, p.bits, c.0, c.1)
}

/// As células que mudaram entre `antes` e o documento de agora.
fn pintadas(antes: &Sprite, p: &Palco) -> Vec<Cell> {
    let d = p.app.doc(p.key).expect("doc");
    let k = (d.layer, d.frame);
    let (w, h) = (antes.width() as i32, antes.height() as i32);
    let mut v = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if antes.value_at(k, x, y) != d.sprite.value_at(k, x, y) {
                v.push((x, y));
            }
        }
    }
    v
}

fn reta(a: Cell, b: Cell) -> Vec<Cell> {
    let mut v = Vec::new();
    line_cells(a.0, a.1, b.0, b.1, |x, y| v.push((x, y)));
    v.sort_by_key(|c| (c.1, c.0));
    v.dedup();
    v
}

/// A distância do ponto `pf` à reta pelos centros de `a` e `b`.
fn falha(a: Cell, b: Cell, pf: [f32; 2]) -> f64 {
    let c = |c: Cell| [f64::from(c.0) + 0.5, f64::from(c.1) + 0.5];
    let (a, b) = (c(a), c(b));
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let (px, py) = (f64::from(pf[0]) - a[0], f64::from(pf[1]) - a[1]);
    (dx * py - dy * px).abs() / dx.hypot(dy)
}

fn com_guias(k: G) -> Palco {
    let mut p = Palco::limpo();
    p.pede(PixelRequest::Edit(E::Guides(k)));
    p
}

#[test]
fn escolher_perspective_2_poe_as_guias_e_arrastar_a_alca_move_o_pf_num_passo() {
    let mut p = com_guias(G::Perspective2);
    let g = guias(&p);
    let (pf0, pf1) = (
        g.pontos_de_fuga[0].expect("PF 1"),
        g.pontos_de_fuga[1].expect("PF 2"),
    );
    assert!(g.pontos_de_fuga[2].is_none() && g.horizonte == Some(pf0[1]));
    assert_eq!(pf0[1], pf1[1], "os dois PF no horizonte");
    assert!(p.app.undo(p.key), "o menu foi um passo");
    assert_eq!(guias(&p), Guias::default());
    assert!(p.app.redo(p.key));
    assert_eq!(guias(&p), g);

    // Arrastar a alça do PF 1 três células para a direita e duas para baixo.
    let meio = |v: f32| (v * 2.0).round() / 2.0;
    let alvo = [meio(pf0[0]) + 3.0, meio(pf0[1]) + 2.0];
    let antes = p.sprite().clone();
    let c = canvas(&p.sim, &p.cam, p.bits);
    let (de, para) = (ecra(&p, pf0), ecra(&p, alvo));
    assert_eq!(
        input::press(&mut p.app, &c, de, Button::Primary, Mods::default()),
        Press::Done
    );
    assert!(input::moved(&mut p.app, &c, para, Mods::default()));
    assert_eq!(
        guias(&p),
        g,
        "durante o arrasto o documento não muda (prévia)"
    );
    assert!(input::release(&mut p.app));
    p.quadro();
    let n = guias(&p);
    assert_eq!(
        n.pontos_de_fuga[0],
        Some(alvo),
        "o PF foi para onde a mão o largou"
    );
    assert_eq!(
        n.pontos_de_fuga[1],
        Some([pf1[0], alvo[1]]),
        "o horizonte leva o outro PF"
    );
    assert_eq!(n.horizonte, Some(alvo[1]));
    assert!(pintadas(&antes, &p).is_empty(), "agarrar a alça não pinta");
    assert!(p.app.undo(p.key), "soltar foi UM passo");
    assert_eq!(guias(&p), g, "Ctrl+Z volta o PF");
}

#[test]
fn a_linha_com_shift_segue_a_guia() {
    let mut p = com_guias(G::Perspective2);
    let g = guias(&p);
    p.ferramenta(Tool::Line);
    let o = (10, 50);
    let mut visados = 0;
    for d in [(24, 44), (30, 58), (8, 62), (22, 34), (40, 47)] {
        let antes = p.sprite().clone();
        arrasta_celulas(&mut p, o, d);
        let cells = pintadas(&antes, &p);
        assert!(cells.contains(&o), "a linha parte da origem: {d:?}");
        let fim = *cells
            .iter()
            .max_by_key(|c| (c.0 - o.0).pow(2) + (c.1 - o.1).pow(2))
            .expect("pintou");
        assert_eq!(reta(o, fim), {
            let mut c = cells.clone();
            c.sort_by_key(|c| (c.1, c.0));
            c
        });
        let melhor = g
            .pontos_de_fuga
            .iter()
            .flatten()
            .map(|pf| falha(o, fim, *pf))
            .fold(f64::INFINITY, f64::min);
        let vertical = fim.0 == o.0;
        assert!(
            vertical || melhor <= 0.5,
            "{o:?}→{d:?} acabou em {fim:?}: falha {melhor} px do PF mais perto"
        );
        visados += usize::from(!vertical);
        assert!(p.app.undo(p.key));
    }
    assert!(visados >= 3, "o fixture visa os PF, não só a vertical");
}

#[test]
fn o_lapis_com_shift_segue_a_guia_e_sem_guias_e_o_de_hoje() {
    for k in [G::Perspective2, G::None] {
        let mut p = com_guias(k);
        p.ferramenta(Tool::Pencil);
        let (o, d) = ((10, 50), (24, 44));
        p.press(o, Mods::default());
        p.release();
        let antes = p.sprite().clone();
        p.press(d, SHIFT);
        p.release();
        let mut cells = pintadas(&antes, &p);
        cells.push(o); // a origem já estava pintada pelo 1.º clique
        cells.sort_by_key(|c| (c.1, c.0));
        if k == G::None {
            assert_eq!(
                cells,
                reta(o, d),
                "sem guias: a reta até ao ponteiro, sem estalo"
            );
        } else {
            let fim = ph2d_pixel_ponte_alvo(&p, o, d);
            assert_ne!(fim, d, "com guias o ponteiro foi guiado");
            assert_eq!(cells, reta(o, fim));
        }
    }
}

/// O destino guiado que o lápis tem de usar: o mesmo que a Linha pinta pelo gesto real.
fn ph2d_pixel_ponte_alvo(p: &Palco, o: Cell, d: Cell) -> Cell {
    let mut q = Palco::limpo();
    q.pede(PixelRequest::Edit(E::Guides(G::Perspective2)));
    assert_eq!(guias(&q), guias(p));
    q.ferramenta(Tool::Line);
    let antes = q.sprite().clone();
    arrasta_celulas(&mut q, o, d);
    *pintadas(&antes, &q)
        .iter()
        .max_by_key(|c| (c.0 - o.0).pow(2) + (c.1 - o.1).pow(2))
        .expect("pintou")
}

#[test]
fn sem_guias_o_shift_da_linha_e_o_de_hoje() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::Line);
    let o = (10, 50);
    for d in [(24, 44), (30, 58), (8, 62), (22, 34), (17, 53)] {
        let antes = p.sprite().clone();
        arrasta_celulas(&mut p, o, d);
        let mut cells = pintadas(&antes, &p);
        cells.sort_by_key(|c| (c.1, c.0));
        assert_eq!(cells, reta(o, snap_pixel_angle(o, d)), "{d:?}");
        assert!(p.app.undo(p.key));
    }
}

#[test]
fn o_iso_poe_a_grade_e_a_linha_vai_a_dois_por_um() {
    let mut p = com_guias(G::Isometric);
    let iso = guias(&p).iso.expect("iso");
    assert_eq!(iso.passo, ph2d_pixel_core::Iso2x1::PASSO);
    p.ferramenta(Tool::Line);
    let o = (4, 10);
    let antes = p.sprite().clone();
    arrasta_celulas(&mut p, o, (15, 14));
    let fim = *pintadas(&antes, &p)
        .iter()
        .max_by_key(|c| (c.0 - o.0).pow(2) + (c.1 - o.1).pow(2))
        .expect("pintou");
    let (dx, dy) = (fim.0 - o.0, fim.1 - o.1);
    assert_eq!(dx, 2 * dy, "2:1 exacto: {fim:?}");
}
