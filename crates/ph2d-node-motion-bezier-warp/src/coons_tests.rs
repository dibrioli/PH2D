//! Os gates da geometria — a identidade, a fronteira, e a divergência do irmão.

use super::*;

const EPS: f32 = 1e-6;

fn close(a: P2, b: P2, eps: f32) -> bool {
    (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
}

/// **AS TANGENTES NOS TERÇOS DÃO A RECTA — POR IDENTIDADE, NÃO POR APROXIMAÇÃO.**
///
/// ⚠️ É a base de tudo o resto: se este degrau fosse aproximado, o nó no neutro
/// moveria as peças por um ε e o *"o default é a identidade"* seria falso.
#[test]
fn a_cubic_with_thirds_tangents_is_exactly_the_straight_segment() {
    let (a, b) = ([-2.0, 1.0], [3.0, -4.0]);
    let [t0, t1] = thirds(a, b);
    for k in 0..=20 {
        let t = k as f32 / 20.0;
        let got = bezier(a, t0, t1, b, t);
        let want = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        assert!(close(got, want, 1e-5), "t={t}: {got:?} vs {want:?}");
    }
}

/// **O PATCH NEUTRO É A IDENTIDADE DO QUADRADO UNITÁRIO.**
#[test]
fn the_unit_boundary_gives_the_identity_patch() {
    let b = Boundary::unit();
    for iu in 0..=10 {
        for iv in 0..=10 {
            let (u, v) = (iu as f32 / 10.0, iv as f32 / 10.0);
            let got = coons(&b, u, v);
            assert!(close(got, [u, v], 1e-5), "({u},{v}) → {got:?}");
        }
    }
}

/// **A FRONTEIRA DO PATCH É EXACTAMENTE AS CURVAS DADAS** — a propriedade que define
/// um Coons, e a razão de a borda desenhada pelo artista ser a borda que aparece.
///
/// ⚠️ Sem este gate, um patch que apenas *interpolasse os quatro cantos* passaria em
/// tudo o resto e a borda sairia arqueada de outra maneira — que é precisamente o
/// modo de falha que a mistura bilinear tem.
#[test]
fn the_patch_boundary_is_the_authored_curves() {
    let mut b = Boundary::unit();
    // Uma barriga para fora no topo e uma para dentro à esquerda.
    b.tangent[TOP][0][1] += 0.5;
    b.tangent[TOP][1][1] += 0.5;
    b.tangent[LEFT][0][0] += 0.3;
    b.tangent[LEFT][1][0] += 0.3;
    for k in 0..=10 {
        let t = k as f32 / 10.0;
        // v = 1 é o TOPO, percorrido TL → TR.
        let want_top = bezier(
            b.corner[TL],
            b.tangent[TOP][0],
            b.tangent[TOP][1],
            b.corner[TR],
            t,
        );
        assert!(
            close(coons(&b, t, 1.0), want_top, 1e-5),
            "topo em t={t}: {:?} vs {want_top:?}",
            coons(&b, t, 1.0)
        );
        // u = 0 é a ESQUERDA, percorrida BL → TL.
        let want_left = bezier(
            b.corner[BL],
            b.tangent[LEFT][0],
            b.tangent[LEFT][1],
            b.corner[TL],
            t,
        );
        assert!(
            close(coons(&b, 0.0, t), want_left, 1e-5),
            "esquerda em t={t}: {:?} vs {want_left:?}",
            coons(&b, 0.0, t)
        );
    }
    // E o CONTROLE: a barriga de facto existe (senão o gate acima seria sobre a
    // identidade e não provaria nada).
    let mid_top = coons(&b, 0.5, 1.0);
    assert!(
        mid_top[1] > 1.3,
        "o topo tem de arquear para fora: {mid_top:?}"
    );
}

/// **O COONS DE UM QUAD DE LADOS RECTOS É O BILINEAR — E O BILINEAR NÃO É A
/// HOMOGRAFIA.**
///
/// ⚠️ **É este gate que justifica o nó existir.** A célula P1 da folha 04 concluiu
/// que o *Bezier Warp* não pode ser um param do `motion.four_point_warp`, e a razão é
/// aritmética: com as tangentes nos terços a fronteira vira um quadrilátero de lados
/// rectos, e ali o Coons é o mapa **bilinear** — que concorda com a homografia
/// projectiva nos quatro CANTOS e diverge no interior. Um nó que reduzisse à
/// homografia seria o irmão com mais knobs.
///
/// A prova é local e não precisa de importar o irmão (esta é uma crate-folha): a
/// **projectividade preserva rectas**, então basta mostrar que o bilinear NÃO
/// preserva a diagonal do quadrado — a mesma afirmação, medida de dentro.
#[test]
fn the_straight_edged_patch_is_bilinear_and_bends_the_interior_lines() {
    // Um quad em que o canto superior-direito foi puxado: lados rectos, não-afim.
    let mut b = Boundary::unit();
    b.corner[TR] = [2.0, 1.6];
    b.tangent[TOP] = thirds(b.corner[TL], b.corner[TR]);
    b.tangent[RIGHT] = thirds(b.corner[TR], b.corner[BR]);
    // Os quatro cantos saem exactos (a propriedade que os dois mapas partilham).
    assert!(close(coons(&b, 0.0, 0.0), b.corner[BL], EPS));
    assert!(close(coons(&b, 1.0, 1.0), b.corner[TR], EPS));
    // A DIAGONAL `u = v` do quadrado: o bilinear arqueia-a. Medimos a distância do
    // ponto médio à recta que une as duas pontas da diagonal mapeada.
    let (a, c) = (coons(&b, 0.0, 0.0), coons(&b, 1.0, 1.0));
    let m = coons(&b, 0.5, 0.5);
    let d = [c[0] - a[0], c[1] - a[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let cross = ((m[0] - a[0]) * d[1] - (m[1] - a[1]) * d[0]).abs() / len;
    assert!(
        cross > 0.02,
        "o bilinear tem de ENTORTAR a diagonal (desvio {cross}) — se ele a mantivesse \
         recta, este nó seria o `motion.four_point_warp` com mais knobs"
    );
    // ⚠️ E o CONTROLE que impede o gate de passar por um bug: num quad AFIM (um
    // paralelogramo) o bilinear É afim, e aí a diagonal fica recta. Sem esta metade,
    // um patch quebrado entortaria tudo e ainda passaria acima.
    let mut par = Boundary::unit();
    for c in &mut par.corner {
        *c = [c[0] * 2.0 + c[1] * 0.5, c[1] * 1.5];
    }
    par.tangent[TOP] = thirds(par.corner[TL], par.corner[TR]);
    par.tangent[RIGHT] = thirds(par.corner[TR], par.corner[BR]);
    par.tangent[BOTTOM] = thirds(par.corner[BR], par.corner[BL]);
    par.tangent[LEFT] = thirds(par.corner[BL], par.corner[TL]);
    let (a, c) = (coons(&par, 0.0, 0.0), coons(&par, 1.0, 1.0));
    let m = coons(&par, 0.5, 0.5);
    let d = [c[0] - a[0], c[1] - a[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let cross = ((m[0] - a[0]) * d[1] - (m[1] - a[1]) * d[0]).abs() / len;
    assert!(
        cross < 1e-5,
        "controle: num paralelogramo o bilinear é AFIM e a diagonal fica recta ({cross})"
    );
}

/// **CADA CANTO SAI NO SEU CANTO** — a checagem de sentido que apanha um índice
/// trocado.
///
/// ⚠️ Um par de lados invertido ainda interpola a fronteira e ainda dá a identidade
/// no neutro; o que ele faz é **cruzar** o patch, e isso lê-se como *"o warp está a
/// torcer"* em vez de *"um índice está trocado"*.
#[test]
fn every_corner_of_the_square_lands_on_its_own_corner() {
    let mut b = Boundary::unit();
    b.corner = [[-1.0, 5.0], [7.0, 6.0], [8.0, -2.0], [0.0, -3.0]];
    b.tangent[TOP] = thirds(b.corner[TL], b.corner[TR]);
    b.tangent[RIGHT] = thirds(b.corner[TR], b.corner[BR]);
    b.tangent[BOTTOM] = thirds(b.corner[BR], b.corner[BL]);
    b.tangent[LEFT] = thirds(b.corner[BL], b.corner[TL]);
    assert!(close(coons(&b, 0.0, 1.0), b.corner[TL], 1e-5), "TL");
    assert!(close(coons(&b, 1.0, 1.0), b.corner[TR], 1e-5), "TR");
    assert!(close(coons(&b, 1.0, 0.0), b.corner[BR], 1e-5), "BR");
    assert!(close(coons(&b, 0.0, 0.0), b.corner[BL], 1e-5), "BL");
}
