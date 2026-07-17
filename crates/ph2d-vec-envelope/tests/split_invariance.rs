//! **O gate-mãe do ADR-0123: invariância à subdivisão.**
//!
//! Uma transformação correta é invariante a **como se subdivide a entrada**: partir uma cúbica em
//! duas e deformar as metades tem de dar a mesma curva que deformar a inteira. O caminho ingênuo
//! (mapear pontos de controle) **não** é — e é assim que se prova, sem implementação de referência
//! e sem golden image.
//!
//! O gate não é invenção nossa: é o repro do [bug aberto #10547 do Inkscape][1] contra o LPE de
//! perspectiva deles — *"The transformed bezier curve should stay the same [whether] it's composed
//! of one segment or multiple subsegments."*
//!
//! [1]: https://gitlab.com/inkscape/inbox/-/work_items/10547
//!
//! # Duas disciplinas de oráculo, e as duas custaram bug neste repo
//!
//! 1. **A distância é GEOMÉTRICA, nunca no mesmo `t`.** Comparar `A(t)` com `B(t)` superestima o
//!    erro em ordens de grandeza — o que domina é deriva de *parametrização*, que é invisível na
//!    tela. Modelamos a **aparência**, não a regra.
//! 2. **O fixture é CURVO** — e mais: tem de curvar *na direção que o mapa deforma*. O gate
//!    `an_isoparametric_line_hides_the_defect` prova isso, executável: o MESMO mutante, no MESMO
//!    mapa, numa reta isoparamétrica, **passa**. Quem "simplificar" o fixture derruba esse teste,
//!    e ele diz porquê.

use kurbo::{BezPath, CubicBez, ParamCurve, PathEl, Point};
use ph2d_vec_envelope::{Warp, warp_path};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

/// Tolerância do fit. Não é calibração: é o número que o gate PEDE ao motor, e contra o qual ele
/// cobra o resultado.
const ACCURACY: f64 = 1e-4;

// ── Os mapas de teste ────────────────────────────────────────────────────────────────────────

/// Afim: **o controle vermelho/verde.** Um afim comuta com a avaliação de Bézier, então o caminho
/// ingênuo e o nosso têm de concordar em epsilon de máquina. Se este teste falhar, o gate está a
/// medir a coisa errada e nada abaixo dele significa nada.
struct Affine {
    m: [[f64; 2]; 2],
    t: [f64; 2],
}

impl Warp for Affine {
    fn map(&self, p: [f64; 2]) -> [f64; 2] {
        [
            self.m[0][0] * p[0] + self.m[0][1] * p[1] + self.t[0],
            self.m[1][0] * p[0] + self.m[1][1] * p[1] + self.t[1],
        ]
    }
    fn jacobian(&self, _p: [f64; 2]) -> [[f64; 2]; 2] {
        self.m
    }
}

/// Bilinear: o mapa não-afim mais simples que existe — o termo `k·u·v` é tudo o que separa este
/// mapa de um afim, e é o que quebra o caminho ingênuo. É também a matemática de uma célula de
/// grade de envelope.
struct Bilinear {
    k: f64,
}

impl Warp for Bilinear {
    fn map(&self, p: [f64; 2]) -> [f64; 2] {
        [p[0] + self.k * p[0] * p[1], p[1]]
    }
    fn jacobian(&self, p: [f64; 2]) -> [[f64; 2]; 2] {
        [[1.0 + self.k * p[1], self.k * p[0]], [0.0, 1.0]]
    }
}

// ── Fixtures ─────────────────────────────────────────────────────────────────────────────────

/// Uma cúbica **muito curva** — um quarto de arco largo. É o fixture que exibe o defeito.
fn curved() -> CubicBez {
    CubicBez::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 4.0),
        Point::new(4.0, 6.0),
        Point::new(8.0, 6.0),
    )
}

/// Uma reta **ISOPARAMÉTRICA** deste mapa (`v` constante), na convenção (⅓, ⅔).
///
/// ⚠️ **Tem de ser isoparamétrica, e a primeira versão deste arquivo errou isto.** Eu usei uma reta
/// *qualquer* supondo que "reto esconde o defeito" — e o gate derrubou-me. Sob `W(u,v) =
/// (u + k·u·v, v)`, uma reta geral `(8t, 6t)` vira `(8t + 48k·t², 6t)`: uma **parábola**. O
/// ingênuo erra nela também.
///
/// Só `v = const` escapa: `W(u, v₀) = (u·(1 + k·v₀), v₀)` é **afim em u**. É a única família de
/// retas que este mapa preserva — e é por isso que este fixture esconde o bug.
fn isoparametric() -> CubicBez {
    let (a, d) = (Point::new(0.0, 3.0), Point::new(8.0, 3.0));
    let v = d - a;
    CubicBez::new(a, a + v / 3.0, a + v * (2.0 / 3.0), d)
}

/// Junta dois paths abertos que partilham a âncora de junção.
///
/// **Não é `pop()` + `extend()`** — e a primeira versão deste arquivo era, e estava errada: o último
/// vértice da primeira metade carrega o `in_handle` da junção, então descartá-lo **apaga a última
/// cúbica**. O vértice de junção é um HÍBRIDO: o `in` vem de quem chega, o `out` de quem sai.
fn concat(mut a: VecPath, b: VecPath) -> VecPath {
    let Some(last) = a.verts.pop() else { return b };
    let mut it = b.verts.into_iter();
    let Some(first) = it.next() else {
        a.verts.push(last);
        return a;
    };
    a.verts.push(VecVertex {
        anchor: last.anchor,
        in_handle: last.in_handle,
        out_handle: first.out_handle,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    });
    a.verts.extend(it);
    a
}

/// Um path aberto de 2 vértices a partir de uma cúbica.
fn path_of(c: CubicBez) -> VecPath {
    let verts = vec![
        VecVertex {
            anchor: [c.p0.x, c.p0.y],
            in_handle: [c.p0.x, c.p0.y],
            out_handle: [c.p1.x, c.p1.y],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [c.p3.x, c.p3.y],
            in_handle: [c.p2.x, c.p2.y],
            out_handle: [c.p3.x, c.p3.y],
            kind: VertexKind::Corner,
            corner_radius: 0.0,
        },
    ];
    VecPath {
        verts,
        closed: false,
        ..Default::default()
    }
}

// ── O caminho INGÊNUO: o bug que o gate existe para pegar ────────────────────────────────────

/// `for v in verts { v = warp(v) }` — âncora e os dois handles, direto pelo mapa.
///
/// É o que o Blender, o Rive, o Skia (`morphpath`) e um dos LPEs do Inkscape fazem. Está aqui como
/// **mutante**: se o gate não o derrubar, o gate não vale nada.
fn warp_naive(path: &VecPath, warp: &impl Warp) -> VecPath {
    let mut out = path.clone();
    for v in &mut out.verts {
        v.anchor = warp.map(v.anchor);
        v.in_handle = warp.map(v.in_handle);
        v.out_handle = warp.map(v.out_handle);
    }
    out
}

// ── Medição ──────────────────────────────────────────────────────────────────────────────────

fn to_bez(p: &VecPath) -> BezPath {
    let mut b = BezPath::new();
    let Some(first) = p.verts.first() else {
        return b;
    };
    b.move_to(Point::new(first.anchor[0], first.anchor[1]));
    for w in p.verts.windows(2) {
        b.curve_to(
            Point::new(w[0].out_handle[0], w[0].out_handle[1]),
            Point::new(w[1].in_handle[0], w[1].in_handle[1]),
            Point::new(w[1].anchor[0], w[1].anchor[1]),
        );
    }
    b
}

/// Quão fino achatamos antes de medir — e este é o **piso do instrumento**: nenhuma asserção deste
/// arquivo pode afirmar precisão melhor que isto, porque abaixo daqui é o erro de corda do próprio
/// oráculo que está a ser medido, não o motor.
const FLATTEN_TOL: f64 = 1e-10;

/// `b` como polilinha.
///
/// ⚠️ **Por que não `PathSeg::nearest`: ele MENTE em cúbica reta.** A primeira versão deste arquivo
/// media com `nearest`, e ele devolve a distância até um **extremo** quando os pontos de controle
/// são colineares (a solve polinomial degenera) — medido: para um ponto EM CIMA do segmento, a
/// 1/8 do comprimento, ele devolveu 1.24 em vez de 0. Não é imprecisão, é resposta errada, e ela
/// teria feito este gate mentir exatamente no fixture reto.
///
/// Achatar tira o polinômio da jogada: distância ponto-segmento é fórmula fechada e não tem caso
/// degenerado.
fn polyline(b: &BezPath) -> Vec<Point> {
    let mut pts = Vec::new();
    kurbo::flatten(b.iter(), FLATTEN_TOL, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => pts.push(p),
        _ => {}
    });
    pts
}

fn dist_pt_seg(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.hypot2();
    if len2 <= 0.0 {
        return (p - a).hypot();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).hypot()
}

/// Distância **geométrica** máxima de `a` até `b`: para cada amostra de `a`, o ponto mais próximo
/// em `b` — **não** o de mesmo `t`. Unidirecional de propósito; os chamadores medem nos dois
/// sentidos quando importa.
fn max_dist(a: &BezPath, b: &BezPath, samples: usize) -> f64 {
    let poly = polyline(b);
    assert!(
        poly.len() >= 2,
        "polilinha degenerada: o oráculo não tem o que medir"
    );
    let mut worst: f64 = 0.0;
    for seg in a.segments() {
        for i in 0..=samples {
            let p = seg.eval(i as f64 / samples as f64);
            let d = poly
                .windows(2)
                .map(|w| dist_pt_seg(p, w[0], w[1]))
                .fold(f64::INFINITY, f64::min);
            worst = worst.max(d);
        }
    }
    worst
}

// ── Os gates ─────────────────────────────────────────────────────────────────────────────────

/// **O CONTROLE.** Sob um afim, o ingênuo está CERTO — e o nosso tem de concordar com ele em
/// epsilon de máquina. Sem este teste verde, os outros não têm significado.
#[test]
fn an_affine_warp_agrees_with_the_naive_path() {
    let w = Affine {
        m: [[1.3, -0.4], [0.25, 0.9]],
        t: [3.0, -2.0],
    };
    let src = path_of(curved());

    let ours = to_bez(&warp_path(&src, &w, ACCURACY));
    let naive = to_bez(&warp_naive(&src, &w));

    let d = max_dist(&ours, &naive, 64).max(max_dist(&naive, &ours, 64));
    // 1e-8 é ~100x o piso do oráculo (`FLATTEN_TOL`) e ~10.000x abaixo da `ACCURACY` que pedimos ao
    // fit. Sob afim o motor não deveria errar NADA, e a margem confirma que não erra — o resíduo é
    // do instrumento. Apertar mais seria medir o achatamento, não o código.
    assert!(
        d < 1e-8,
        "afim: o nosso e o ingênuo têm de coincidir (afim comuta com Bézier); divergiram {d:.3e}"
    );
}

/// **O GATE-MÃE.** Partir a fonte não pode mudar a curva deformada.
#[test]
fn splitting_a_segment_does_not_change_the_warped_curve() {
    let w = Bilinear { k: 0.08 };
    let c = curved();

    let whole = to_bez(&warp_path(&path_of(c), &w, ACCURACY));

    let split = to_bez(&concat(
        warp_path(&path_of(c.subsegment(0.0..0.5)), &w, ACCURACY),
        warp_path(&path_of(c.subsegment(0.5..1.0)), &w, ACCURACY),
    ));

    let d = max_dist(&whole, &split, 128).max(max_dist(&split, &whole, 128));
    assert!(
        d < 4.0 * ACCURACY,
        "invariância à subdivisão: inteira vs partida divergiram {d:.3e} (tol {:.3e})",
        4.0 * ACCURACY
    );
}

/// **A PROVA DE MUTAÇÃO.** O gate acima só vale se derrubar o bug que existe para pegar.
///
/// O ingênuo erra por **6% da extensão da forma** aqui — não é roundoff, é outra curva.
#[test]
fn the_naive_control_point_warp_fails_split_invariance() {
    let w = Bilinear { k: 0.08 };
    let c = curved();

    let whole = to_bez(&warp_naive(&path_of(c), &w));

    let split = to_bez(&concat(
        warp_naive(&path_of(c.subsegment(0.0..0.5)), &w),
        warp_naive(&path_of(c.subsegment(0.5..1.0)), &w),
    ));

    let d = max_dist(&whole, &split, 128).max(max_dist(&split, &whole, 128));
    assert!(
        d > 100.0 * ACCURACY,
        "o ingênuo TEM de falhar este gate — se ele passa, o gate não morde. Divergiu só {d:.3e}"
    );
}

/// **DISCIPLINA DE FIXTURE, executável.** O mesmo mutante, o mesmo mapa, a mesma extensão — e numa
/// reta **isoparamétrica** ele **passa**.
///
/// Não basta "não ser polígono": a fonte tem de curvar **na direção que o mapa deforma**. Este mapa
/// preserva `v = const`, então essa família de retas esconde o bug — e uma reta *qualquer* NÃO
/// esconde (vira parábola). A lasca do Build ensinou a primeira metade disto; esta ensina a
/// segunda.
#[test]
fn an_isoparametric_line_hides_the_defect() {
    let w = Bilinear { k: 0.08 };
    let c = isoparametric();

    let whole = to_bez(&warp_naive(&path_of(c), &w));

    let split = to_bez(&concat(
        warp_naive(&path_of(c.subsegment(0.0..0.5)), &w),
        warp_naive(&path_of(c.subsegment(0.5..1.0)), &w),
    ));

    let d = max_dist(&whole, &split, 128).max(max_dist(&split, &whole, 128));
    assert!(
        d < 100.0 * ACCURACY,
        "reta isoparamétrica: o ingênuo deveria ESCAPAR aqui (é o ponto do teste), mas divergiu {d:.3e}"
    );
}
