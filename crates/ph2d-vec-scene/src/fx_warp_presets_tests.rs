//! Gates do **Warp** (Arc/Bulge/Wave/Fisheye/Rise). O oráculo é a APARÊNCIA — para onde a
//! silhueta de fato vai —, nunca a fórmula: um gate que repetisse a `deform` seria um espelho.

use super::{WarpSpec, WarpStyle, warp_contour};
use crate::effect::FxCtx;
use crate::{VecPath, VecVertex};

/// Um quadrado de lado 40 — números do produto, não `1.0`. Bbox `[0,40]²`, centro `(20,20)`.
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// O bbox das ÂNCORAS (as posições warpadas).
fn bbox(verts: &[VecVertex]) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    for v in verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (lo, hi)
}

/// Aplica um estilo com dobra `bend` ao quadrado e devolve os verts warpados.
fn warp(style: WarpStyle, bend: f64) -> Vec<VecVertex> {
    let p = square();
    let (out, _) = warp_contour(
        &p.verts,
        p.closed,
        &WarpSpec { style, bend },
        &FxCtx::of(&p),
    );
    out
}

/// **`bend == 0` é no-op BYTE-idêntico** — o que a pilha exige de todo efeito, senão não pode
/// saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
#[test]
fn the_neutral_warp_is_a_byte_identical_no_op() {
    let p = square();
    let c = FxCtx::of(&p);
    for &style in WarpStyle::ALL {
        let (out, closed) = warp_contour(&p.verts, p.closed, &WarpSpec { style, bend: 0.0 }, &c);
        assert_eq!(
            out,
            p.verts,
            "{}: bend 0 tem de devolver a fonte",
            style.label()
        );
        assert!(closed, "{}: não abre a forma", style.label());
    }
}

/// **Todo estilo DEFORMA a silhueta, e os cinco dão resultados DIFERENTES** — senão seriam
/// nomes diferentes para o mesmo efeito (ou para o gizmo, que uma escala uniforme seria).
///
/// ⚠️ Mutação: qualquer `deform` colapsar na identidade ⇒ o bbox não muda ⇒ RED; dois estilos
/// com a mesma fórmula ⇒ os verts coincidem ⇒ RED.
#[test]
fn every_style_bends_the_shape_and_the_styles_differ() {
    let src = bbox(&square().verts);
    let mut seen: Vec<Vec<VecVertex>> = Vec::new();
    for &style in WarpStyle::ALL {
        let out = warp(style, 50.0);
        assert_ne!(
            bbox(&out),
            src,
            "{}: bend 50 não mudou o bbox — a silhueta ficou igual",
            style.label()
        );
        for prev in &seen {
            assert_ne!(&out, prev, "dois estilos deram a MESMA geometria");
        }
        seen.push(out);
    }
}

/// **Arc ARQUEIA** — dobra positiva sobe o meio (o topo passa de `y=40`); negativa afunda-o.
/// O oráculo é o EXTREMO da silhueta, não a fórmula.
#[test]
fn the_arc_arches_the_shape() {
    let up = bbox(&warp(WarpStyle::Arc, 50.0));
    assert!(
        up.1[1] > 41.0,
        "Arc para cima devia subir o topo além de 40, deu {}",
        up.1[1]
    );
    let down = bbox(&warp(WarpStyle::Arc, -50.0));
    assert!(
        down.0[1] < -1.0,
        "Arc para baixo devia afundar a base abaixo de 0, deu {}",
        down.0[1]
    );
}

/// **Wave BALANÇA para os DOIS lados** — a onda (um seno ao longo de x) põe metade da forma
/// acima do topo e metade abaixo da base. É o que a distingue do Arc (que só sobe/desce junto).
#[test]
fn the_wave_swings_both_ways() {
    let (lo, hi) = bbox(&warp(WarpStyle::Wave, 50.0));
    assert!(
        hi[1] > 41.0,
        "a crista da onda devia passar de 40, deu {}",
        hi[1]
    );
    assert!(
        lo[1] < -1.0,
        "o vale da onda devia passar de 0, deu {}",
        lo[1]
    );
}

/// **O warp é independente da REAMOSTRAGEM** — o mesmo quadrado com âncoras a mais no meio de
/// cada aresta warpa para a MESMA silhueta. É a propriedade do arco que o Zig Zag pagou caro
/// para ter (*"duas formas que se veem iguais têm de se comportar igual"*), e o warp herda-a por
/// reamostrar por comprimento de arco.
#[test]
fn the_warp_is_resampling_independent() {
    // O mesmo quadrado, mas com o ponto médio de cada aresta como âncora extra (8 verts).
    let dense = VecPath {
        verts: [
            [0.0, 0.0],
            [20.0, 0.0],
            [40.0, 0.0],
            [40.0, 20.0],
            [40.0, 40.0],
            [20.0, 40.0],
            [0.0, 40.0],
            [0.0, 20.0],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    for &style in WarpStyle::ALL {
        let a = bbox(&warp(style, 40.0));
        let (out_d, _) = warp_contour(
            &dense.verts,
            dense.closed,
            &WarpSpec { style, bend: 40.0 },
            &FxCtx::of(&dense),
        );
        let b = bbox(&out_d);
        for k in 0..2 {
            assert!(
                (a.0[k] - b.0[k]).abs() < 0.5 && (a.1[k] - b.1[k]).abs() < 0.5,
                "{}: picar as arestas mudou a silhueta warpada ({a:?} vs {b:?}) — a densidade \
                 de âncoras está a vazar para o resultado",
                style.label()
            );
        }
    }
}

/// **A saída é lisa** (`Smooth`), e um contorno fechado continua fechado. Sem alças de
/// Catmull-Rom a poligonal densa leria como facetas.
#[test]
fn the_warp_output_is_smooth_and_stays_closed() {
    let out = warp(WarpStyle::Wave, 40.0);
    assert!(
        out.len() > 8,
        "a silhueta warpada é reamostrada densa, veio {}",
        out.len()
    );
    assert!(
        out.iter().all(|v| v.kind == crate::VertexKind::Smooth),
        "os vértices do warp têm de ser lisos"
    );
}
