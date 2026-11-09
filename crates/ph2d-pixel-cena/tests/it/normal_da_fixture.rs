//! P14c-pixel 1b: a fixture `heroi_p14` leva a camada Normal Map — o hemisfério no recorte da cor,
//! nos 6 quadros: de frente no centro, a inclinar para fora nas bordas.

use ph2d_pixel_cena::fixture::{COR, heroi_p14};
use ph2d_pixel_core::{compose_normal, compose_rgba};

#[test]
fn a_normal_da_fixture_e_um_hemisferio_no_recorte_da_cor() {
    let s = heroi_p14();
    assert_eq!(s.layers().iter().filter(|l| l.normal).count(), 1);
    let w = s.width() as usize;
    assert_eq!(s.frames().len(), 6);
    for f in s.frames() {
        let n = compose_normal(&s, f.id);
        let mut c = Vec::new();
        compose_rgba(&s, f.id, None, &mut c);
        let at = |b: &[u8], x: usize, y: usize| -> [u8; 4] {
            let i = (y * w + x) * 4;
            [b[i], b[i + 1], b[i + 2], b[i + 3]]
        };
        let perto = |a: u8, b: u8| a.abs_diff(b) <= 12;
        let centro = at(&n, 8, 8);
        assert!(
            perto(centro[0], 128) && perto(centro[1], 128) && perto(centro[2], 255),
            "quadro {:?}: centro {centro:?}",
            f.id
        );
        // a borda direita da linha do meio: o último píxel com cor
        let borda = (0..w).rev().find(|&x| at(&c, x, 8)[3] != 0).expect("arte");
        assert!(borda > 8);
        let d = at(&n, borda, 8);
        assert!(d[0] > 128 + 64, "quadro {:?}: borda direita {d:?}", f.id);
        let e = at(&n, (0..w).find(|&x| at(&c, x, 8)[3] != 0).expect("arte"), 8);
        assert!(e[0] < 128 - 64, "quadro {:?}: borda esquerda {e:?}", f.id);
        // em cima, a normal olha para cima (y para cima)
        let topo = (0..w).find(|&y| at(&c, 8, y)[3] != 0).expect("arte");
        assert!(at(&n, 8, topo)[1] > 128 + 64);
        for y in 0..w {
            for x in 0..w {
                assert_eq!(
                    at(&n, x, y)[3],
                    at(&c, x, y)[3],
                    "o recorte da cor em ({x},{y})"
                );
                if at(&c, x, y)[3] != 0 {
                    assert_eq!(at(&c, x, y), COR, "a normal não entra na cor");
                }
            }
        }
    }
}
