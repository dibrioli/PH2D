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

/// Aplica um estilo com dobra `bend` (sem distorção) ao quadrado e devolve os verts warpados.
fn warp(style: WarpStyle, bend: f64) -> Vec<VecVertex> {
    warp_spec(WarpSpec {
        style,
        bend,
        h_distort: 0.0,
        v_distort: 0.0,
    })
}

/// Aplica um `WarpSpec` inteiro ao quadrado — usado pelos gates de distorção, que precisam de
/// mais que a dobra.
fn warp_spec(spec: WarpSpec) -> Vec<VecVertex> {
    let p = square();
    let (out, _) = warp_contour(&p.verts, p.closed, &spec, &FxCtx::of(&p), None);
    out
}

/// **Os TRÊS controles em zero são no-op BYTE-idêntico** — o que a pilha exige de todo efeito,
/// senão não pode saltá-lo e o `Cow::Borrowed` morre (ADR-0132). ⚠️ Antes das perspectivas
/// bastava `bend == 0`; agora a dobra E as duas distorções têm de estar em zero, e este gate
/// prova que uma delas sozinha JÁ deforma (via os gates de keystone abaixo).
#[test]
fn the_neutral_warp_is_a_byte_identical_no_op() {
    let p = square();
    let c = FxCtx::of(&p);
    for &style in WarpStyle::ALL {
        let spec = WarpSpec {
            style,
            bend: 0.0,
            h_distort: 0.0,
            v_distort: 0.0,
        };
        let (out, closed) = warp_contour(&p.verts, p.closed, &spec, &c, None);
        assert_eq!(
            out,
            p.verts,
            "{}: os três em zero têm de devolver a fonte",
            style.label()
        );
        assert!(closed, "{}: não abre a forma", style.label());
    }
}

/// **A distorção HORIZONTAL faz o keystone** — com dobra zero, `h_distort > 0` alarga a borda de
/// CIMA contra a de baixo (o topo passa a ter mais x-extensão que a base). O oráculo é a razão
/// entre as larguras das duas metades — robusto à reamostragem, e a APARÊNCIA do keystone, não a
/// fórmula.
#[test]
fn the_horizontal_distortion_keystones_the_shape() {
    let out = warp_spec(WarpSpec {
        style: WarpStyle::Arc,
        bend: 0.0,
        h_distort: 60.0,
        v_distort: 0.0,
    });
    // Largura em x das âncoras de cada metade (acima/abaixo do centro y=20).
    let x_spread = |upper: bool| {
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for v in &out {
            if (v.anchor[1] >= 20.0) == upper {
                lo = lo.min(v.anchor[0]);
                hi = hi.max(v.anchor[0]);
            }
        }
        hi - lo
    };
    let (top, bot) = (x_spread(true), x_spread(false));
    assert!(
        top > bot * 1.5,
        "h_distort devia alargar o topo contra a base (keystone): topo {top:.2}, base {bot:.2}"
    );
}

/// **A distorção VERTICAL é o keystone do outro eixo** — `v_distort > 0` alonga a borda da
/// DIREITA contra a esquerda. Espelho exato do gate horizontal.
#[test]
fn the_vertical_distortion_keystones_the_shape() {
    let out = warp_spec(WarpSpec {
        style: WarpStyle::Arc,
        bend: 0.0,
        h_distort: 0.0,
        v_distort: 60.0,
    });
    let y_spread = |right: bool| {
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for v in &out {
            if (v.anchor[0] >= 20.0) == right {
                lo = lo.min(v.anchor[1]);
                hi = hi.max(v.anchor[1]);
            }
        }
        hi - lo
    };
    let (right, left) = (y_spread(true), y_spread(false));
    assert!(
        right > left * 1.5,
        "v_distort devia alongar a direita contra a esquerda: dir {right:.2}, esq {left:.2}"
    );
}

/// **As perspectivas COMPÕEM com a dobra** — Arc+Horizontal não é Arc sozinho nem Horizontal
/// sozinho: a perspectiva lê o ponto JÁ arqueado, então o resultado difere dos dois. É o
/// invariante que separa *compor* de *escolher entre*.
#[test]
fn the_distortion_composes_with_the_bend() {
    let bend_only = warp_spec(WarpSpec {
        style: WarpStyle::Arc,
        bend: 50.0,
        h_distort: 0.0,
        v_distort: 0.0,
    });
    let dist_only = warp_spec(WarpSpec {
        style: WarpStyle::Arc,
        bend: 0.0,
        h_distort: 50.0,
        v_distort: 0.0,
    });
    let both = warp_spec(WarpSpec {
        style: WarpStyle::Arc,
        bend: 50.0,
        h_distort: 50.0,
        v_distort: 0.0,
    });
    assert_ne!(
        both, bend_only,
        "compor com Horizontal tem de mudar o Arc dobrado"
    );
    assert_ne!(
        both, dist_only,
        "compor com a dobra tem de mudar o keystone"
    );
}

/// **Todo estilo DEFORMA a silhueta, e os NOVE dão resultados DIFERENTES** — senão seriam nomes
/// diferentes para o mesmo efeito.
///
/// ⚠️ O oráculo de "deformou" é *os VÉRTICES mudaram*, não o bbox: o **Squeeze** afina a cintura
/// e deixa as QUINAS onde estão, então o bbox fica igual enquanto a silhueta muda (um bbox como
/// oráculo o daria como inerte — falso). Mutação: `deform` colapsar na identidade ⇒ verts iguais
/// à fonte ⇒ RED; dois estilos com a mesma fórmula ⇒ os verts coincidem ⇒ RED.
#[test]
fn every_style_bends_the_shape_and_the_styles_differ() {
    let src = square();
    let mut seen: Vec<Vec<VecVertex>> = Vec::new();
    for &style in WarpStyle::ALL {
        let out = warp(style, 50.0);
        assert_ne!(
            out,
            src.verts,
            "{}: bend 50 não mudou os vértices — a silhueta ficou igual",
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
            &WarpSpec {
                style,
                bend: 40.0,
                h_distort: 0.0,
                v_distort: 0.0,
            },
            &FxCtx::of(&dense),
            None,
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
