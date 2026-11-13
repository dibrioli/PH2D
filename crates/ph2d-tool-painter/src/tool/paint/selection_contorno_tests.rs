//! B20.11: o contorno das formigas é EXATAMENTE a fronteira 128 da máscara, sobre as arestas entre
//! texels — retângulo, elipse, elipse com buraco (Remove) e sela diagonal.

use super::{Polilinha, contornos_da_mascara};
use crate::tool::PainterTool;
use crate::tool::paint::selection_shapes::{SelectionEntry, SelectionShape};
use ph2d_editor_core::tool::RasterEditTool;
use std::collections::BTreeSet;

type Aresta = ((i64, i64), (i64, i64));

/// As arestas unitárias NÃO dirigidas entre um texel dentro e um fora (ou a borda), à força bruta.
fn fronteira(mask: &[u8], w: usize, h: usize) -> BTreeSet<Aresta> {
    let d = |x: i64, y: i64| {
        x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < h
            && mask[y as usize * w + x as usize] >= 128
    };
    let mut s = BTreeSet::new();
    for y in -1..=h as i64 {
        for x in -1..=w as i64 {
            if d(x, y) != d(x + 1, y) {
                s.insert(((x + 1, y), (x + 1, y + 1)));
            }
            if d(x, y) != d(x, y + 1) {
                s.insert(((x, y + 1), (x + 1, y + 1)));
            }
        }
    }
    s
}

/// As arestas unitárias que as polilinhas percorrem; cada lado é eixo-alinhado, de cantos inteiros,
/// sem dois lados seguidos colineares, e nenhuma aresta passa duas vezes.
fn percorridas(contornos: &[Polilinha]) -> BTreeSet<Aresta> {
    let mut s = BTreeSet::new();
    for p in contornos {
        assert!(p.len() >= 4, "um laço tem pelo menos 4 cantos: {p:?}");
        let n = p.len();
        for k in 0..n {
            let (a, b, c) = (p[k], p[(k + 1) % n], p[(k + 2) % n]);
            for q in [a, b] {
                assert!(
                    q[0].fract() == 0.0 && q[1].fract() == 0.0,
                    "canto fora da grelha {q:?}"
                );
            }
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            assert!(
                (dx == 0.0) != (dy == 0.0),
                "lado não eixo-alinhado {a:?}→{b:?}"
            );
            let (ex, ey) = (c[0] - b[0], c[1] - b[1]);
            assert!(dx * ey - dy * ex != 0.0, "colineares por juntar em {b:?}");
            // `f32::signum(0.0)` é 1: o lado vertical andava na diagonal sem fim (a corrida que pendurou).
            let sinal = |v: f32| i64::from(v > 0.0) - i64::from(v < 0.0);
            let (sx, sy) = (sinal(dx), sinal(dy));
            let (mut x, mut y) = (a[0] as i64, a[1] as i64);
            while (x, y) != (b[0] as i64, b[1] as i64) {
                let (nx, ny) = (x + sx, y + sy);
                let e = ((x.min(nx), y.min(ny)), (x.max(nx), y.max(ny)));
                assert!(s.insert(e), "aresta {e:?} percorrida duas vezes");
                (x, y) = (nx, ny);
            }
        }
    }
    s
}

fn confere(mask: &[u8], w: usize, h: usize) -> Vec<Polilinha> {
    let c = contornos_da_mascara(mask, w, h);
    assert_eq!(
        percorridas(&c),
        fronteira(mask, w, h),
        "o contorno não é a fronteira 128"
    );
    c
}

fn tela(s: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (s * s * 4) as usize], s, s);
    t
}

fn elipse(cx: f32, cy: f32, r: f32, op: u8) -> SelectionEntry {
    SelectionEntry {
        shape: SelectionShape::Ellipse {
            center: [cx, cy],
            u: [1.0, 0.0],
            rx: r,
            ry: r * 0.7,
        },
        op,
    }
}

#[test]
fn o_contorno_das_formigas_e_a_fronteira_128_da_mascara() {
    // Retângulo pela porta do tool: um laço, os quatro cantos da caixa.
    let mut t = tela(96);
    t.set_rect_selection(10, 12, 60, 50);
    let c = t.selection_contornos().expect("seleção viva");
    assert_eq!(
        c.mascara,
        vec![vec![[10.0, 12.0], [70.0, 12.0], [70.0, 62.0], [10.0, 62.0]]],
        "o retângulo"
    );
    confere(&t.paint.selection_mask, 96, 96);

    // Elipse; e elipse com buraco (Remove): dois laços, o do buraco ao contrário.
    let mut t = tela(128);
    t.paint.selection_shapes = vec![elipse(64.0, 60.0, 50.0, 0)];
    t.recompose_selection_mask();
    assert_eq!(
        confere(&t.paint.selection_mask, 128, 128).len(),
        1,
        "a elipse"
    );
    t.paint.selection_shapes.push(elipse(60.0, 62.0, 18.0, 2));
    t.recompose_selection_mask();
    let c = confere(&t.paint.selection_mask, 128, 128);
    assert_eq!(c.len(), 2, "a elipse e o buraco");
    let area = |p: &Polilinha| {
        (0..p.len())
            .map(|k| {
                let (a, b) = (p[k], p[(k + 1) % p.len()]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum::<f32>()
    };
    assert!(
        area(&c[0]) * area(&c[1]) < 0.0,
        "o buraco anda ao contrário"
    );

    // Sela diagonal: dois texels que só se tocam no canto são dois laços; e a tela inteira.
    let sela = [255u8, 0, 0, 255];
    assert_eq!(confere(&sela, 2, 2).len(), 2, "a sela separa");
    let cheia = vec![255u8; 12];
    assert_eq!(
        confere(&cheia, 4, 3),
        vec![vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0]]]
    );
    assert!(contornos_da_mascara(&[0u8; 9], 3, 3).is_empty(), "vazia");
}

/// Casos-limite: cada um termina, cobre a fronteira inteira e não percorre aresta duas vezes.
#[test]
fn o_contorno_nos_casos_limite() {
    let m = |w: usize, h: usize, f: &dyn Fn(usize, usize) -> bool| {
        (0..w * h)
            .map(|i| if f(i % w, i / w) { 255u8 } else { 0 })
            .collect::<Vec<_>>()
    };
    // Texel isolado no meio, e encostado a cada canto da tela.
    let c = confere(&m(5, 5, &|x, y| (x, y) == (2, 2)), 5, 5);
    assert_eq!(
        c,
        vec![vec![[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0]]]
    );
    for (x0, y0) in [(0, 0), (4, 0), (4, 4), (0, 4)] {
        assert_eq!(confere(&m(5, 5, &|x, y| (x, y) == (x0, y0)), 5, 5).len(), 1);
    }
    assert_eq!(confere(&[255u8], 1, 1).len(), 1, "tela de 1 texel cheia");
    // Xadrez: cada texel dentro é um laço (as selas separam).
    assert_eq!(
        confere(&m(4, 4, &|x, y| (x + y) % 2 == 0), 4, 4).len(),
        8,
        "xadrez 4×4"
    );
    assert_eq!(
        confere(&[0u8, 255, 255, 0], 2, 2).len(),
        2,
        "a sela na outra diagonal"
    );
    // Buraco dentro de buraco: anel, buraco, ilha dentro do buraco = 3 laços.
    let ilha = m(9, 9, &|x, y| {
        let r = x.abs_diff(4).max(y.abs_diff(4));
        r == 4 || r == 3 || r == 0
    });
    assert_eq!(confere(&ilha, 9, 9).len(), 3, "buraco dentro de buraco");
    // Moldura encostada às 4 bordas: o laço de fora é a borda da tela.
    let moldura = m(6, 4, &|x, y| x == 0 || y == 0 || x == 5 || y == 3);
    let c = confere(&moldura, 6, 4);
    assert_eq!(c.len(), 2);
    assert!(
        c.contains(&vec![[0.0, 0.0], [6.0, 0.0], [6.0, 4.0], [0.0, 4.0]]),
        "{c:?}"
    );
    // Tamanho que não casa com a máscara: nada.
    assert!(contornos_da_mascara(&[255u8; 3], 2, 2).is_empty());
}

/// A guarda: o mesmo `Arc` da máscara devolve o MESMO contorno; os anéis do Offset entram nos níveis.
#[test]
fn o_contorno_guardado_troca_com_a_mascara_e_os_aneis() {
    let mut t = tela(96);
    t.set_rect_selection(10, 12, 60, 50);
    let a = t.selection_contornos().expect("viva");
    assert!(std::sync::Arc::ptr_eq(
        &a,
        &t.selection_contornos().expect("viva")
    ));
    assert!(a.niveis.is_empty(), "sem Offset, sem ciano");
    t.set_selection_offset(0.8);
    t.selection_offset_apply_keep();
    // O 1.º Apply & Keep só entra no modo anel (sem anel ainda); a 2.ª varrida desenha a linha viva.
    assert!(t.selection_contornos().expect("viva").niveis.is_empty());
    t.set_selection_offset(0.6);
    let b = t.selection_contornos().expect("viva");
    assert!(
        !b.niveis.is_empty(),
        "a linha viva do Offset tem contorno ciano"
    );
    t.selection_offset_apply_keep();
    let c = t.selection_contornos().expect("viva");
    assert!(!c.niveis.is_empty(), "o anel congelado continua ciano");
    t.clear_selection();
    assert!(t.selection_contornos().is_none(), "sem seleção, nada");
}

/// Sonda (B20.11): traçar o contorno de uma varinha mágica (Automatic) a 4K sobre ruído, e quantos
/// lados saem. `-- --ignored --nocapture`.
#[test]
#[ignore = "sonda de relógio"]
fn sonda_contorno_varinha_4k() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let (w, h) = (3840u32, 2160u32);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    let mut s = 0x2545_f491_u32;
    for _ in 0..w * h {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let v = if s.is_multiple_of(3) { 40 } else { 220 };
        px.extend_from_slice(&[v, v, v, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(px, w, h);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(0);
    let p = |ph| CanvasPointer {
        pos: [1920.0, 1080.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase: ph,
    };
    t.on_canvas_pointer(p(PointerPhase::Down));
    t.on_canvas_pointer(p(PointerPhase::Up));
    let mask = t.paint.selection_mask.clone();
    let dentro = mask.iter().filter(|&&m| m >= 128).count();
    let mut min = f64::INFINITY;
    let mut lados = 0;
    let mut lacos = 0;
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        let c = contornos_da_mascara(&mask, w as usize, h as usize);
        min = min.min(t0.elapsed().as_secs_f64() * 1e3);
        lados = c.iter().map(Vec::len).sum::<usize>();
        lacos = c.len();
    }
    eprintln!(
        "[contorno varinha 4K] {dentro} texels dentro · {lacos} laços · {lados} lados · {min:.1} ms (mín. de 5)"
    );
}
