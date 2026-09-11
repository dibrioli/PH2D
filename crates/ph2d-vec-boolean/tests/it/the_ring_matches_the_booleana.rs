//! **O offset DIRETO (`offset_ring`) bate com a booleana (`offset_path`)** — a prova de que
//! trocar o motor do Contour vivo não muda o que se vê.
//!
//! O oráculo é a **área NonZero** (o que o Vello rasteriza), amostrada por winding — NÃO a área
//! de shoelace, que conta a auto-interseção com sinal e MENTE sobre o que o preenchimento faz. Foi
//! medindo por shoelace que uma primeira sonda reportou 6% de erro no côncavo sobre um método que
//! está a 0% (a discrepância era da régua, não do método).
//!
//! ⚠️ A fixture inclui uma ESTRELA (côncava) de propósito: é onde o offset se auto-cruza, o único
//! caso em que o direto e a booleana poderiam divergir — e o gate exige que NÃO divirjam.

use kurbo::{BezPath, Point, Shape};
use ph2d_vec_scene::{LineJoin, OffsetSide, VecPath, VecVertex};

fn poly(sides: usize, r: f64, inner: f64) -> VecPath {
    let n = if inner > 0.0 { sides * 2 } else { sides };
    let verts = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(u32::try_from(i).unwrap())
                / f64::from(u32::try_from(n).unwrap());
            let rr = if inner > 0.0 && i % 2 == 1 {
                r * inner
            } else {
                r
            };
            VecVertex::corner([2.5 + rr * a.cos(), rr * a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// A área que o rasterizador de fato preenche (winding NonZero != 0), amostrada numa grade fina.
fn nonzero_area(paths: &[VecPath]) -> f64 {
    // ⚠️ Respeita os HANDLES: as quinas Round do offset viram cúbicas, e medir só as âncoras
    // (LineTo) achataria os arcos e reportaria erro sobre um método correto (aconteceu).
    let mut bez = BezPath::new();
    for path in paths {
        let n = path.verts.len();
        if n < 3 {
            continue;
        }
        bez.move_to(Point::new(path.verts[0].anchor[0], path.verts[0].anchor[1]));
        for i in 0..n {
            let a = &path.verts[i];
            let b = &path.verts[(i + 1) % n];
            bez.curve_to(
                Point::new(a.out_handle[0], a.out_handle[1]),
                Point::new(b.in_handle[0], b.in_handle[1]),
                Point::new(b.anchor[0], b.anchor[1]),
            );
        }
        bez.close_path();
    }
    if bez.elements().len() < 3 {
        return 0.0;
    }
    let bb = bez.bounding_box();
    let (nx, ny) = (500usize, 500usize);
    let inside: u64 = (0..ny)
        .flat_map(|iy| (0..nx).map(move |ix| (ix, iy)))
        .filter(|&(ix, iy)| {
            let x = bb.x0 + (bb.x1 - bb.x0) * (f64::from(u32::try_from(ix).unwrap()) + 0.5) / 500.0;
            let y = bb.y0 + (bb.y1 - bb.y0) * (f64::from(u32::try_from(iy).unwrap()) + 0.5) / 500.0;
            bez.winding(Point::new(x, y)) != 0
        })
        .count() as u64;
    f64::from(u32::try_from(inside).unwrap()) * (bb.x1 - bb.x0) * (bb.y1 - bb.y0) / 250_000.0
}

#[test]
fn the_direct_ring_fills_the_same_area_as_the_booleana() {
    for (name, shape) in [
        ("hexágono", poly(6, 1.2, 0.0)),
        ("quadrado", poly(4, 1.2, 0.0)),
        ("estrela-5", poly(5, 1.2, 0.45)), // CÔNCAVA — o offset se auto-cruza
        ("estrela-8", poly(8, 1.2, 0.55)),
    ] {
        for d in [0.1, 0.3, 0.6] {
            let direct =
                ph2d_vec_boolean::offset_ring(&shape, d, LineJoin::Round, OffsetSide::Outer)
                    .expect("contorno único, Outer, d>0 está no domínio do offset direto");
            let boolean =
                ph2d_vec_boolean::offset_path(&shape, d, LineJoin::Round, OffsetSide::Outer);
            let (a_dir, a_bool) = (nonzero_area(&direct), nonzero_area(&boolean));
            // Onde a booleana panicou (área ~0), ela não é oráculo — o direto é a única resposta.
            if a_bool < 1e-3 {
                assert!(
                    a_dir > 0.0,
                    "{name} d={d}: a booleana panicou E o direto tambem falhou"
                );
                continue;
            }
            let err = (a_dir - a_bool).abs() / a_bool;
            assert!(
                err < 0.01,
                "{name} d={d}: area NonZero direto {a_dir:.3} vs booleana {a_bool:.3} = {:.1}% \
                 (o offset direto mudaria o que o artista ve)",
                err * 100.0
            );
        }
    }
}

/// **O offset direto é GROW-ONLY, e a booleana ENCOLHE** — o gate do report do Enio (*"Outer com
/// offset negativo cresce em vez de encolher"*).
///
/// O bug era o `offset_ring` **escolher o laço errado** ao encolher: o `kurbo::stroke` emite dois
/// contornos e a seleção antiga (por `area().abs()`) confundia a dilatação com a erosão numa forma
/// côncava — a estrela CRESCIA ao encolher. A cura estrutural é o direto **só CRESCER** (a
/// dilatação de Minkowski é robusta no kurbo; a erosão ele derruba a `|d|` grande, ver o doc de
/// `offset_ring`) e delegar TODA a erosão à booleana, que a resolve por `base ∖ banda` — o mesmo
/// caminho testado do Offset Path. Consistente (sem crossover direto↔booleana no meio de um
/// contour).
///
/// Duas metades:
/// 1. o direto **abstém** (`None`) em todo `d ≤ 0` — não há mais como ele mis-selecionar a erosão;
/// 2. a booleana ENCOLHE de verdade (área < a da fonte), inclusive na ESTRELA côncava, onde o bug
///    do sinal fazia crescer.
///
/// ⚠️ Mutação: `offset_ring` aceitar `d < 0` (voltar a escolher um laço ao encolher) ⇒ a metade 1
/// sangra. E `offset_path` devolver a dilatação ao encolher ⇒ a metade 2 sangra na estrela.
#[test]
fn the_direct_ring_is_grow_only_and_the_booleana_shrinks() {
    for (name, shape) in [
        ("hexágono", poly(6, 1.2, 0.0)),
        ("estrela-5", poly(5, 1.2, 0.45)), // CÔNCAVA — onde o sinal mentia
    ] {
        let src = nonzero_area(std::slice::from_ref(&shape));
        for d in [-0.1, -0.2] {
            // 1. o direto NÃO tenta encolher.
            assert!(
                ph2d_vec_boolean::offset_ring(&shape, d, LineJoin::Round, OffsetSide::Outer)
                    .is_none(),
                "{name} d={d}: o offset direto é grow-only, tinha de abster (`None`) ao encolher"
            );
            // 2. a booleana encolhe (área menor que a fonte), não cresce.
            let boolean =
                ph2d_vec_boolean::offset_path(&shape, d, LineJoin::Round, OffsetSide::Outer);
            let a = nonzero_area(&boolean);
            assert!(
                a < src * 0.98,
                "{name} d={d}: a booleana devia ENCOLHER, área {a:.3} vs fonte {src:.3} (o bug: \
                 crescia)"
            );
        }
    }
}

/// **O offset direto NUNCA panica** — a raiz do piscar. O `offset_path` (booleana) panica numa
/// fração grande das distâncias (medido: estrela+Bevel 118/200); o `offset_ring` cobre o mesmo
/// domínio sem tocar o `linesweeper`.
#[test]
fn the_direct_ring_never_panics_where_the_booleana_does() {
    let star = poly(5, 1.2, 0.45);
    let mut ok = 0;
    for i in 1..=200 {
        let d = f64::from(i) * 0.01;
        // Se panicasse, o teste morreria aqui (sem catch_unwind de propósito — é o ponto).
        let r = ph2d_vec_boolean::offset_ring(&star, d, LineJoin::Bevel, OffsetSide::Outer);
        assert!(r.is_some(), "contorno unico Outer devia estar no dominio");
        if !r.unwrap().is_empty() {
            ok += 1;
        }
    }
    assert!(
        ok > 150,
        "so {ok}/200 distancias produziram anel — o direto degenerou cedo demais"
    );
}

/// **Fora do domínio (compound), o offset direto se abstem** (`None`) para o chamador usar a
/// booleana — a porta que decide *"sei fazer isto?"*.
#[test]
fn the_direct_ring_abstains_on_a_compound() {
    let mut donut = poly(4, 1.5, 0.0);
    // um furo (subpath) torna-o compound
    donut
        .subpaths
        .push(ph2d_vec_scene::Contour::new_closed(poly(4, 0.6, 0.0).verts));
    assert!(
        ph2d_vec_boolean::offset_ring(&donut, 0.3, LineJoin::Round, OffsetSide::Outer).is_none(),
        "um compound tem furo que encolhe — fora do dominio do offset direto"
    );
}
