//! Testes de `symbols.rs` — arquivo irmao (teto de 700 LOC por arquivo, HR-18).
//!
//! Ligado por `#[path]` no modulo pai, entao `use super::*` continua valendo.

use super::*;

const A: [f64; 2] = [-2.0, -1.0];
const B: [f64; 2] = [2.0, 1.0];
/// Quantas amostras por cúbica na medição da bbox VERDADEIRA (ver [`curve_bbox`]).
const SAMPLES: usize = 2048;

/// Toda forma, nos seus defaults (os mesmos de `ShapeKind::defaults`).
fn catalog() -> Vec<(&'static str, VecPath)> {
    vec![
        ("heart", heart(A, B, 0.2)),
        ("bolt", bolt(A, B, 0.4)),
        ("moon", moon(A, B, 0.45)),
        ("drop", drop(A, B, 0.5)),
        ("shield", shield(A, B, 0.6)),
        ("tag", tag(A, B, 0.25, 0.18)),
        ("gear", gear(A, B, 8.0, 0.2, 0.35)),
        ("cross", cross(A, B, 0.35)),
        ("check", check(A, B, 0.25)),
        ("banner", banner(A, B, 0.15)),
    ]
}

/// As mesmas formas nos EXTREMOS de cada parâmetro — é onde um clamp anti-transbordo
/// esquecido apareceria.
fn extremes() -> Vec<(&'static str, VecPath)> {
    vec![
        ("heart lo", heart(A, B, 0.0)),
        ("heart hi", heart(A, B, 0.5)),
        ("bolt lo", bolt(A, B, 0.05)),
        ("bolt hi", bolt(A, B, 0.8)),
        ("moon lo", moon(A, B, 0.05)),
        ("moon hi", moon(A, B, 0.95)),
        ("drop lo", drop(A, B, 0.1)),
        ("drop hi", drop(A, B, 0.9)),
        ("shield lo", shield(A, B, 0.0)),
        ("shield hi", shield(A, B, 1.0)),
        ("tag lo", tag(A, B, 0.05, 0.0)),
        ("tag hi", tag(A, B, 0.6, 0.4)),
        ("gear lo", gear(A, B, 4.0, 0.05, 0.0)),
        ("gear hi", gear(A, B, 30.0, 0.5, 0.8)),
        ("cross lo", cross(A, B, 0.05)),
        ("cross hi", cross(A, B, 0.95)),
        ("check lo", check(A, B, 0.05)),
        ("check hi", check(A, B, 0.6)),
        ("banner lo", banner(A, B, 0.02)),
        ("banner hi", banner(A, B, 0.4)),
    ]
}

/// A bbox **da curva**, amostrada — âncoras *e* o interior de cada cúbica. Uma Bézier
/// passeia fora do casco dos seus pontos de controle (o bico do coração e a barriga da
/// gota vivem justamente aí), então medir só âncoras SUBESTIMA a forma: o desenho vaza
/// do gizmo e o teste passa. Amostrar é um oráculo INDEPENDENTE do solver de extremos
/// que o [`fit`] usa — se os dois discordarem, o teste cai.
fn curve_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut eat = |verts: &[VecVertex], is_closed: bool| {
        let n = verts.len();
        if n == 0 {
            return;
        }
        let last = if is_closed { n } else { n - 1 };
        for i in 0..last {
            let (va, vb) = (&verts[i], &verts[(i + 1) % n]);
            for s in 0..=SAMPLES {
                let t = s as f64 / SAMPLES as f64;
                let q = crate::cubic_at(va.anchor, va.out_handle, vb.in_handle, vb.anchor, t);
                for k in 0..2 {
                    lo[k] = lo[k].min(q[k]);
                    hi[k] = hi[k].max(q[k]);
                }
            }
        }
    };
    eat(&p.verts, p.closed);
    for c in &p.subpaths {
        eat(&c.verts, c.closed);
    }
    (lo, hi)
}

fn uv(p: [f64; 2]) -> (f64, f64) {
    (p[0], 1.0 - p[1]) // a caixa [0,0]..[1,1] tem `Unit::p((u,v)) = [u, 1−v]`
}

fn near(got: (f64, f64), want: (f64, f64), tol: f64, what: &str) {
    assert!(
        (got.0 - want.0).abs() < tol && (got.1 - want.1).abs() < tol,
        "{what}: {got:?} != {want:?} (tol {tol})"
    );
}

/// **A ORIENTAÇÃO, executável.** Cada uma destas caiu de verdade quando o módulo era
/// escrito em coordenadas de mundo: tudo o que é assimétrico em `y` nasceu espelhado
/// (coração de bico para cima, gota de bico para baixo, escudo invertido). São ordens
/// RELATIVAS no mundo (Y-para-cima), nunca constantes mágicas.
#[test]
fn the_asymmetric_symbols_are_right_side_up() {
    let lowest = |p: &VecPath| {
        p.verts.iter().map(|v| v.anchor).fold(
            [0.0, f64::MAX],
            |acc, q| if q[1] < acc[1] { q } else { acc },
        )
    };
    let highest = |p: &VecPath| {
        p.verts.iter().map(|v| v.anchor).fold(
            [0.0, f64::MIN],
            |acc, q| if q[1] > acc[1] { q } else { acc },
        )
    };

    // CORAÇÃO: o bico é o ponto mais baixo da CURVA INTEIRA (não só das âncoras — é essa
    // a diferença que faz a asserção morder: espelhado, o mais baixo passariam a ser os
    // lóbulos, que não são âncoras, e o bico viraria o ponto mais alto).
    let h = heart(A, B, 0.2);
    let (h_lo, h_hi) = curve_bbox(&h);
    let tip = lowest(&h);
    let cleft = h
        .verts
        .iter()
        .find(|v| (v.anchor[1] - tip[1]).abs() > 1e-9)
        .expect("o coracao tem vale e bico")
        .anchor;
    assert!(
        (tip[1] - h_lo[1]).abs() < 1e-6 && tip[0].abs() < 1e-9,
        "o bico do coracao e o ponto mais BAIXO da curva, no eixo: {tip:?} vs {h_lo:?}"
    );
    assert!(
        cleft[1] > tip[1] && cleft[1] < h_hi[1] - 1e-3,
        "o vale ({cleft:?}) afunda ENTRE o bico ({tip:?}) e o topo dos lobulos ({})",
        h_hi[1]
    );

    // GOTA: o bico é o ponto mais ALTO da CURVA e é uma QUINA (os dois flancos chegam
    // retos); o bulbo, redondo, fica embaixo — e o extremo de baixo NÃO é âncora nenhuma.
    // De ponta-cabeça, a âncora mais alta seria um vértice do arco, estritamente abaixo
    // da coroa do bulbo, e a primeira asserção cai.
    let d = drop(A, B, 0.5);
    let (_, d_hi) = curve_bbox(&d);
    let apex = highest(&d);
    assert!(
        (apex[1] - d_hi[1]).abs() < 1e-6 && apex[0].abs() < 1e-9,
        "o bico da gota e o ponto mais ALTO da curva, no eixo: {apex:?} vs {d_hi:?}"
    );
    let apex_v = d
        .verts
        .iter()
        .find(|v| v.anchor == apex)
        .expect("o apice e um vertice");
    assert!(
        apex_v.in_handle == apex_v.anchor && apex_v.out_handle == apex_v.anchor,
        "o bico da gota e QUINA: os flancos sao as tangentes, retas"
    );

    // ESCUDO: a ponta é o mais BAIXO, e o topo é RETO (duas quinas na altura máxima).
    let s = shield(A, B, 0.6);
    let point = lowest(&s);
    assert!(point[0].abs() < 1e-9, "a ponta do escudo fica no eixo");
    let top = s
        .verts
        .iter()
        .filter(|v| (v.anchor[1] - 1.0).abs() < 1e-9)
        .count();
    assert_eq!(top, 2, "o escudo tem o topo RETO (dois cantos no teto)");

    // LUA: as cornetas abrem para a DIREITA — são os dois pontos mais à direita, uma
    // em cima e outra embaixo; a barriga (o mais à esquerda) fica na meia-altura.
    let m = moon(A, B, 0.45);
    let horns: Vec<[f64; 2]> = m
        .verts
        .iter()
        .filter(|v| (v.anchor[0] - 2.0).abs() < 1e-6)
        .map(|v| v.anchor)
        .collect();
    assert_eq!(horns.len(), 2, "a lua tem DUAS cornetas na borda direita");
    assert!(
        horns.iter().any(|q| q[1] > 0.9) && horns.iter().any(|q| q[1] < -0.9),
        "uma corneta em cima e outra embaixo: {horns:?}"
    );
    let belly =
        m.verts.iter().map(|v| v.anchor).fold(
            [f64::MAX, 0.0],
            |acc, q| if q[0] < acc[0] { q } else { acc },
        );
    assert!(
        belly[1].abs() < 1e-6,
        "a barriga da lua fica na meia-altura, a ESQUERDA: {belly:?}"
    );

    // ETIQUETA: o bico é o ponto mais à ESQUERDA, na meia-altura.
    let t = tag(A, B, 0.25, 0.18);
    let nose =
        t.verts.iter().map(|v| v.anchor).fold(
            [f64::MAX, 0.0],
            |acc, q| if q[0] < acc[0] { q } else { acc },
        );
    assert!(
        nose[1].abs() < 1e-6,
        "o bico da etiqueta aponta para a ESQUERDA, na meia-altura: {nose:?}"
    );

    // CHECK: o cotovelo é o ponto mais BAIXO, e fica à ESQUERDA do centro.
    let c = check(A, B, 0.25);
    let el = lowest(&c);
    assert!(el[0] < 0.0, "o cotovelo do check fica a ESQUERDA: {el:?}");

    // RAIO: o bico de baixo é o mais BAIXO E o mais à direita (a diagonal do ECMA).
    let bt = bolt(A, B, 0.4);
    let strike = lowest(&bt);
    assert!(
        (strike[0] - 2.0).abs() < 1e-6,
        "o bico do raio bate no canto INFERIOR DIREITO: {strike:?}"
    );

    // BANNER: a dobra fica em CIMA (o topo alcança as duas bordas laterais) e o painel
    // pendura EMBAIXO (o fundo só existe no miolo).
    let bn = banner(A, B, 0.15);
    let top_xs: Vec<f64> = bn
        .verts
        .iter()
        .filter(|v| (v.anchor[1] - 1.0).abs() < 1e-6)
        .map(|v| v.anchor[0])
        .collect();
    assert!(
        top_xs.iter().any(|x| (*x + 2.0).abs() < 1e-6)
            && top_xs.iter().any(|x| (*x - 2.0).abs() < 1e-6),
        "as caudas do banner alcancam as duas bordas EM CIMA: {top_xs:?}"
    );
    for v in &bn.verts {
        if (v.anchor[1] + 1.0).abs() < 1e-6 {
            assert!(
                v.anchor[0].abs() < 1.2,
                "o fundo do banner e o PAINEL (miolo), nao a cauda: {:?}",
                v.anchor
            );
        }
    }
}

/// **Cabe na caixa — e a PREENCHE.** Medido na curva de verdade (âncoras *mais* o
/// interior das cúbicas), nos defaults e nos EXTREMOS de cada parâmetro. É o contrato do
/// [`fit`]: nenhuma forma vaza do gizmo, nenhuma sobra dentro dele.
#[test]
fn every_symbol_fills_the_gesture_box_exactly() {
    for (name, p) in catalog().into_iter().chain(extremes()) {
        assert!(!p.verts.is_empty(), "{name}: cozinhou vazio");
        let (lo, hi) = curve_bbox(&p);
        for (k, (want_lo, want_hi)) in [(A[0], B[0]), (A[1], B[1])].into_iter().enumerate() {
            assert!(
                lo[k] >= want_lo - 1e-9 && hi[k] <= want_hi + 1e-9,
                "{name}: a curva VAZA da caixa no eixo {k}: [{}, {}]",
                lo[k],
                hi[k]
            );
            assert!(
                (lo[k] - want_lo).abs() < 1e-4 && (hi[k] - want_hi).abs() < 1e-4,
                "{name}: a curva SOBRA dentro da caixa no eixo {k}: [{}, {}] != [{want_lo}, {want_hi}]",
                lo[k],
                hi[k]
            );
        }
    }
}

/// **Determinismo.** Mesmos parâmetros ⇒ os MESMOS bytes, todo cozimento. É o que o gate
/// de replay-hash do CI compara — e o que faz undo/reload devolverem a mesma forma.
#[test]
fn cooking_twice_gives_byte_identical_geometry() {
    let bytes = |x: &VecPath| postcard::to_allocvec(x).expect("path serializa");
    let again = catalog().into_iter().chain(extremes());
    for ((name, first), (_, second)) in catalog().into_iter().chain(extremes()).zip(again) {
        assert_eq!(bytes(&first), bytes(&second), "{name}: cozimentos diferem");
    }
}

/// **Fidelidade ao padrão-ouro: o coração do ECMA-376.** Com o vale no default da espec
/// (`¼`), os seis pontos de controle têm de bater com a tabela "box-exact" — a que sai
/// de aplicar o ajuste à geometria crua da espec (que estoura a caixa em 0,36%).
#[test]
fn the_heart_reproduces_the_ecma_box_exact_table() {
    let h = heart([0.0, 0.0], [1.0, 1.0], 0.25);
    let tol = 1e-3;
    near(uv(h.verts[0].anchor), (0.5, 0.240_592), tol, "vale");
    near(
        uv(h.verts[0].out_handle),
        (0.706_825, -0.350_058),
        tol,
        "handle do lobulo direito",
    );
    near(
        uv(h.verts[1].in_handle),
        (1.513_445, 0.240_592),
        tol,
        "handle do flanco direito",
    );
    near(uv(h.verts[1].anchor), (0.5, 1.0), tol, "bico");
    near(
        uv(h.verts[1].out_handle),
        (-0.513_445, 0.240_592),
        tol,
        "handle do flanco esquerdo",
    );
    near(
        uv(h.verts[0].in_handle),
        (0.293_175, -0.350_058),
        tol,
        "handle do lobulo esquerdo",
    );
}

/// **Fidelidade ao padrão-ouro: os 11 vértices do `lightningBolt`.** No default o gerador
/// tem de devolver a tabela da espec — é o ponto de identidade da parametrização.
#[test]
fn the_bolt_reproduces_the_ecma_vertex_table() {
    let p = bolt([0.0, 0.0], [1.0, 1.0], BOLT_WAIST_CANON);
    assert_eq!(p.verts.len(), 11, "o raio do ECMA tem 11 vertices");
    for (i, v) in p.verts.iter().enumerate() {
        let want = (BOLT_TABLE[i].0 / BOLT_GRID, BOLT_TABLE[i].1 / BOLT_GRID);
        near(uv(v.anchor), want, 1e-3, &format!("vertice {i} do raio"));
    }
}

/// **Fidelidade ao padrão-ouro: os flancos da gota TANGENCIAM o bulbo.** O teste é o
/// discriminante: a reta ápice→T toca a elipse em UM ponto (raiz dupla). Ligar o ápice
/// ao extremo lateral do bulbo — o erro clássico, que o stencil de gota da maioria dos
/// apps comete — dá duas raízes, e o encontro sai com um vinco visível.
#[test]
fn the_drop_flanks_are_tangent_to_its_bulb() {
    for tip in [0.1_f64, 0.5, 0.9] {
        let ry = DROP_BULB_BLUNT + tip * (DROP_BULB_SHARP - DROP_BULB_BLUNT);
        let beta = (ry / (1.0 - ry)).acos();
        // Em espaço de autoria: ápice, ponto de tangência e a elipse do bulbo.
        let (ax, ay) = (0.5, 0.0);
        let (ex, ey) = (0.5, 1.0 - ry);
        let (tx, ty) = (0.5 + 0.5 * beta.sin(), (1.0 - ry) - ry * ry / (1.0 - ry));
        let (dx, dy) = (tx - ax, ty - ay);
        let qa = (dx / 0.5).powi(2) + (dy / ry).powi(2);
        let qb = 2.0 * ((ax - ex) * dx / 0.25 + (ay - ey) * dy / (ry * ry));
        let qc = ((ax - ex) / 0.5).powi(2) + ((ay - ey) / ry).powi(2) - 1.0;
        let disc = qb * qb - 4.0 * qa * qc;
        assert!(
            disc.abs() < 1e-9,
            "tip {tip}: discriminante {disc} != 0 — o flanco CORTA o bulbo em vez de \
             tangenciar"
        );
        // E o erro do desenho ingênuo (ligar ao equador) é grande o bastante para ver.
        assert!(
            ((1.0 - ry) - ty).abs() > 0.05,
            "tip {tip}: o ponto ingenuo estaria perto demais — o teste seria inutil"
        );
    }
}

/// **Fidelidade ao padrão-ouro: as cornetas da lua são TANGENCIAIS.** Limbo e terminador
/// têm o mesmo semi-eixo vertical, então os dois chegam à corneta com tangente
/// HORIZONTAL: o ângulo entre eles é zero (agulha), e não os 20,6° do crescente do ECMA.
#[test]
fn the_moon_cusps_are_tangential() {
    for phase in [0.05_f64, 0.45, 0.95] {
        let m = moon(A, B, phase);
        let cusps: Vec<&VecVertex> = m
            .verts
            .iter()
            .filter(|v| (v.anchor[0] - 2.0).abs() < 1e-6)
            .collect();
        assert_eq!(cusps.len(), 2, "phase {phase}: duas cornetas");
        for c in cusps {
            assert!(
                (c.in_handle[1] - c.anchor[1]).abs() < 1e-9,
                "phase {phase}: a tangente que CHEGA na corneta nao e horizontal"
            );
            assert!(
                (c.out_handle[1] - c.anchor[1]).abs() < 1e-9,
                "phase {phase}: a tangente que SAI da corneta nao e horizontal"
            );
            // E os dois handles caem do MESMO lado — é uma cúspide, não uma junção lisa.
            assert!(
                (c.in_handle[0] - c.anchor[0]) * (c.out_handle[0] - c.anchor[0]) > 0.0,
                "phase {phase}: a corneta e uma CUSPIDE (os dois handles para dentro)"
            );
        }
    }
    assert!(!moon(A, B, 0.45).is_compound(), "a lua e UM contorno");
}

/// **As laterais do escudo encostam no ombro com tangente VERTICAL.** É a propriedade que
/// define a construção heráldica (o centro do arco está na horizontal do ponto de
/// encontro), e é ela que garante G1 com a lateral reta para QUALQUER raio.
#[test]
fn the_shield_sides_leave_the_shoulder_vertically() {
    for curve in [0.0_f64, 0.6, 1.0] {
        let s = shield(A, B, curve);
        // O ombro direito: a âncora na borda direita que NÃO está no teto.
        let shoulder = s
            .verts
            .iter()
            .find(|v| (v.anchor[0] - 2.0).abs() < 1e-9 && v.anchor[1] < 0.999)
            .expect("o ombro direito existe");
        assert!(
            (shoulder.out_handle[0] - shoulder.anchor[0]).abs() < 1e-9,
            "curve {curve}: a lateral sai do ombro na VERTICAL (handle {:?} vs ancora {:?})",
            shoulder.out_handle,
            shoulder.anchor
        );
    }
}

/// Engrenagem e etiqueta FURAM de verdade: o furo é um contorno próprio + `EvenOdd`. Sem
/// furo pedido não viram compound (nada de contorno degenerado dentro do documento).
#[test]
fn the_gear_and_the_tag_actually_punch_their_holes() {
    for (name, holed, solid) in [
        ("gear", gear(A, B, 8.0, 0.2, 0.4), gear(A, B, 8.0, 0.2, 0.0)),
        ("tag", tag(A, B, 0.25, 0.18), tag(A, B, 0.25, 0.0)),
    ] {
        assert!(holed.is_compound(), "{name}: o furo e um contorno proprio");
        assert_eq!(
            holed.fill_rule,
            crate::FillRule::EvenOdd,
            "{name}: sem EvenOdd o furo nao vaza"
        );
        assert!(!solid.is_compound(), "{name}: sem furo, sem compound");
    }
}

/// O furo da etiqueta fica DENTRO do material — inclusive quando o bico é curto e o
/// pedido é grande. É um clamp de contenção de verdade (não de bbox): o ilhó furando a
/// borda descaracterizaria a forma.
#[test]
fn the_tag_eyelet_never_breaches_the_outline() {
    for (point, hole) in [(0.05_f64, 0.4_f64), (0.25, 0.18), (0.6, 0.4), (0.1, 0.3)] {
        let t = tag(A, B, point, hole);
        let eye = t.subpaths.first().expect("com hole > 0 o ilho existe");
        let p = point.clamp(0.05, 0.60);
        let n = eye.verts.len();
        // A elipse do furo passeia ENTRE as âncoras — checar só as quatro pontas do eixo
        // deixaria a diagonal (que é justamente a aresta do bico) escapar.
        for i in 0..n {
            let (va, vb) = (&eye.verts[i], &eye.verts[(i + 1) % n]);
            for s in 0..=256 {
                let q = crate::cubic_at(
                    va.anchor,
                    va.out_handle,
                    vb.in_handle,
                    vb.anchor,
                    f64::from(s) / 256.0,
                );
                // De volta a `uv` (a caixa do teste tem hw = 2, hh = 1).
                let (u, vv) = ((q[0] + 2.0) / 4.0, 0.5 - q[1] * 0.5);
                let half = 0.5 * (u / p).min(1.0); // meia-altura da etiqueta em `u`
                assert!(
                    u > 0.0 && (vv - 0.5).abs() <= half + 1e-9,
                    "point {point}, hole {hole}: o ilho furou a borda em {q:?}"
                );
            }
        }
    }
}

/// A cruz tem 12 quinas (o contorno em degrau) — é o que a separa de um retângulo.
#[test]
fn the_cross_has_twelve_corners() {
    assert_eq!(cross(A, B, 0.3).verts.len(), 12);
}

/// **A dobra do banner é uma DOBRA, não um buraco.** O `moveTo/lnTo/arcTo` do `ribbon` do
/// ECMA desenha só as faces da FRENTE: copiado ao pé da letra, ele deixa a face dobrada
/// (a `darkenLess`) vazia, e a fita sai furada no meio da dobra — foi o que a primeira
/// versão fez, e só o desenho mostrou. Aqui o contorno externo é a silhueta da UNIÃO.
///
/// O teste é um ray-cast no contorno externo: o miolo da face dobrada tem de estar
/// DENTRO, e o entalhe acima do painel (que é o que faz a cauda ler como mais alta que o
/// painel) tem de continuar FORA — senão a "correção" teria sido tapar tudo.
#[test]
fn the_banner_fold_is_a_fold_not_a_hole() {
    // Cruzamentos de um raio horizontal para a direita, no contorno externo achatado.
    fn inside(p: &VecPath, q: [f64; 2]) -> bool {
        let mut pts: Vec<[f64; 2]> = Vec::new();
        let n = p.verts.len();
        for i in 0..n {
            let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
            for s in 0..64 {
                pts.push(crate::cubic_at(
                    a.anchor,
                    a.out_handle,
                    b.in_handle,
                    b.anchor,
                    f64::from(s) / 64.0,
                ));
            }
        }
        let m = pts.len();
        (0..m)
            .filter(|&i| {
                let (c, d) = (pts[i], pts[(i + 1) % m]);
                (c[1] > q[1]) != (d[1] > q[1]) && {
                    let t = (q[1] - c[1]) / (d[1] - c[1]);
                    c[0] + t * (d[0] - c[0]) > q[0]
                }
            })
            .count()
            % 2
            == 1
    }

    let bn = banner(A, B, 0.15);
    // `uv` → mundo na caixa do teste (hw = 2, hh = 1).
    let w = |u: f64, v: f64| [4.0 * u - 2.0, 1.0 - 2.0 * v];
    let rx = (BANNER_RX * 0.5).clamp(0.006, 0.055); // aspect da caixa = 0.5
    let ry = 0.25 * BANNER_FOLD;
    let x4 = 0.25 + 3.0 * rx;

    for (name, q) in [
        ("miolo da face dobrada, a esquerda", w(x4, 3.0 * ry)),
        ("miolo da face dobrada, a direita", w(1.0 - x4, 3.0 * ry)),
        ("o painel", w(0.5, 0.6)),
        ("a cauda, a direita do V", w(0.2, 0.2)),
    ] {
        assert!(inside(&bn, q), "{name} ({q:?}) tem de estar DENTRO da fita");
    }
    for (name, q) in [
        ("o entalhe acima do painel", w(0.5, 0.02)),
        ("o V da cauda esquerda", w(0.02, 0.4167)),
        ("abaixo da cauda, fora do painel", w(0.1, 0.95)),
    ] {
        assert!(!inside(&bn, q), "{name} ({q:?}) tem de estar FORA da fita");
    }
}
