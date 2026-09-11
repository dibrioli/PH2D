//! **A silhueta de uma forma TRAÇADA é a borda EXTERNA da tinta, nunca a curva autorada.**
//!
//! Um traço centrado põe a curva do documento no MEIO da faixa de tinta. Quem alimenta um campo de
//! distância com essa curva planta a fronteira DENTRO da forma — e é exatamente por isso que o
//! `silhouette_segments` do `ph2d-vec-render` recusava (com razão) toda forma com traço, mandando-a
//! para o caminho do raster, cuja semente discreta desenha o pente que aparece no bevel.
//!
//! O oráculo aqui não conhece `silhouette_paths`: ele pergunta **onde a fronteira está** em relação
//! às âncoras que o artista desenhou, que é a propriedade que o defeito viola.

use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, StrokeSpec, VecPath, cook};

/// Metade da largura do traço, nas unidades da forma (bbox 4×4).
const HALF_WIDTH: f64 = 0.05;

fn star(points: f64) -> VecPath {
    let mut p = cook(
        ShapeKind::Star,
        [-2.0, -2.0],
        [2.0, 2.0],
        &[points, 0.45, 0.0],
    );
    p.fill = Some(Paint::Solid(Rgba8::new(160, 40, 200, 255)));
    p
}

fn stroked(points: f64) -> VecPath {
    let mut p = star(points);
    p.stroke = Some(StrokeSpec::new(
        Rgba8::new(255, 255, 255, 255),
        HALF_WIDTH * 2.0,
    ));
    p
}

/// A menor distância de `q` à poligonal fechada de `p` (âncoras; as formas aqui são poligonais).
fn dist_to_boundary(p: &VecPath, q: [f64; 2]) -> f64 {
    let mut best = f64::MAX;
    for c in std::iter::once(&p.verts[..]).chain(p.subpaths.iter().map(|c| &c.verts[..])) {
        for i in 0..c.len() {
            let (a, b) = (c[i].anchor, c[(i + 1) % c.len()].anchor);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let len2 = dx * dx + dy * dy;
            let t = if len2 > 0.0 {
                (((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (px, py) = (a[0] + t * dx - q[0], a[1] + t * dy - q[1]);
            best = best.min((px * px + py * py).sqrt());
        }
    }
    best
}

/// Quantas vezes um raio para +x cruza a fronteira de `p` — `ímpar` = `q` está DENTRO.
fn inside(p: &VecPath, q: [f64; 2]) -> bool {
    let mut crossings = 0usize;
    for c in std::iter::once(&p.verts[..]).chain(p.subpaths.iter().map(|c| &c.verts[..])) {
        for i in 0..c.len() {
            let (a, b) = (c[i].anchor, c[(i + 1) % c.len()].anchor);
            if (a[1] > q[1]) != (b[1] > q[1]) {
                let t = (q[1] - a[1]) / (b[1] - a[1]);
                if a[0] + t * (b[0] - a[0]) > q[0] {
                    crossings += 1;
                }
            }
        }
    }
    crossings % 2 == 1
}

/// **O gate.** A curva que o artista desenhou fica no INTERIOR da silhueta — a fronteira saiu de
/// cima dela e foi para a borda da tinta.
///
/// ⚠️ **A distância NÃO é uma barra uniforme, e a primeira versão deste gate errava por isso:** num
/// espeto o join é clampado (miter vira bevel), e a corda do bevel passa a **0,0101** da ponta numa
/// estrela de 12 pontas — legítimo, e é o que o rasterizador desenha também. *Dentro* vale em todo
/// ponto; *meia-largura de folga* só vale longe das quinas, e é por isso que os dois são medidos
/// separadamente (o irmão abaixo).
///
/// ⚠️ Mutação que sangra: `silhouette_paths` devolver `vec![path.clone()]` para uma forma traçada
/// (a resposta *"a silhueta é a própria curva"*). Aí toda âncora cai EM CIMA da fronteira e a
/// contagem de cruzamentos deixa de ser ímpar — a fronteira volta a passar pelo meio da forma.
#[test]
fn the_authored_curve_lies_inside_the_silhouette_never_on_its_boundary() {
    for points in [5.0_f64, 12.0] {
        let p = stroked(points);
        let sil = ph2d_vec_boolean::silhouette_paths(&p);
        assert!(
            !sil.is_empty(),
            "{points} pontas: a silhueta de uma forma tracada nao pode ser vazia"
        );
        for (i, v) in p.verts.iter().enumerate() {
            let hit = sil.iter().any(|r| inside(r, v.anchor));
            assert!(
                hit,
                "{points} pontas: a ancora {i} em {:?} nao esta DENTRO da silhueta \
                 - a fronteira ainda passa pela curva autorada",
                v.anchor
            );
        }
    }
}

/// **Longe das quinas a folga é a meia-largura EXATA.** É o número que o gate acima não pode
/// afirmar (o join clampa nos espetos) e sem o qual *"dentro"* seria satisfeito por uma silhueta
/// um micron maior que a curva.
#[test]
fn away_from_the_corners_the_clearance_is_exactly_half_the_stroke() {
    let p = stroked(5.0);
    let sil = ph2d_vec_boolean::silhouette_paths(&p);
    for i in 0..p.verts.len() {
        let (a, b) = (p.verts[i].anchor, p.verts[(i + 1) % p.verts.len()].anchor);
        let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        let d = sil
            .iter()
            .map(|r| dist_to_boundary(r, mid))
            .fold(f64::MAX, f64::min);
        assert!(
            (d - HALF_WIDTH).abs() < HALF_WIDTH * 0.05,
            "meio da aresta {i}: folga {d:.4}, esperado {HALF_WIDTH:.4}"
        );
    }
}

/// **O MIOLO da forma pertence à silhueta — não há fronteira no meio dela.**
///
/// ⚠️ Este gate nasceu de uma **mutação que sobreviveu aos outros três**: devolver só a tinta
/// (`return ink`, sem unir com o preenchimento) deixa um ANEL — e o furo do anel é uma fronteira
/// que corre pelo meio da forma, que é literalmente o defeito que a união existe para apagar. Os
/// outros gates não a viam porque medem perto da curva autorada, e a curva autorada fica DENTRO da
/// faixa de tinta nos dois casos.
///
/// O oráculo é o CENTRO: com o furo ele fica de fora (o raio cruza a borda externa e a interna, e
/// duas travessias é "fora"); com a união ele está dentro e a fronteira mais próxima está a um raio
/// interno de distância.
#[test]
fn the_middle_of_the_shape_belongs_to_the_silhouette_no_boundary_runs_through_it() {
    for points in [5.0_f64, 12.0] {
        let p = stroked(points);
        let sil = ph2d_vec_boolean::silhouette_paths(&p);
        let c = [0.0, 0.0];
        assert!(
            sil.iter().any(|r| inside(r, c)),
            "{points} pontas: o centro da forma nao esta na silhueta \
             - sobrou o furo do anel de tinta, e o campo o leria como fronteira"
        );
        let d = sil
            .iter()
            .map(|r| dist_to_boundary(r, c))
            .fold(f64::MAX, f64::min);
        assert!(
            d > 0.5,
            "{points} pontas: ha fronteira a {d:.4} do centro - a silhueta esta furada"
        );
    }
}

/// **Sem traço, a silhueta é a própria forma — ao BIT.** Sem isto a porta podia "resolver" o caso
/// que já estava certo e mover geometria que ninguém pediu para mover.
#[test]
fn a_shape_without_a_stroke_is_its_own_silhouette() {
    let p = star(5.0);
    let sil = ph2d_vec_boolean::silhouette_paths(&p);
    assert_eq!(sil.len(), 1, "sem traco a silhueta e um caminho so");
    assert_eq!(sil[0], p, "sem traco a silhueta e a PROPRIA forma, ao bit");
}

/// **Largura ZERO é "sem traço" — silhueta ao BIT, e sem sweep.**
///
/// Report do Enio (2026-07-27, 2ª rodada): *"para stroke maior que 0 funciona. Mas para stroke = 0
/// linhas aparecem"*. O slider chega a `0`, e `0` significa sem traço — mas `stroke.is_some()`
/// continua verdadeiro, então a tinta saía VAZIA e esta porta devolvia `Vec::new()`, que o chamador
/// lê como *"não sei responder"* e converte no caminho do raster.
///
/// ⚠️ O gate mede as DUAS metades: a resposta (a forma, ao bit) e o CUSTO (zero sweeps) — sem a
/// segunda, uma implementação que unisse a forma consigo mesma passaria.
#[test]
fn a_zero_width_stroke_is_no_stroke_at_all() {
    let mut p = star(5.0);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.0));
    let before = ph2d_vec_boolean::__sweep_calls();
    let sil = ph2d_vec_boolean::silhouette_paths(&p);
    assert_eq!(sil.len(), 1, "largura zero devolveu {} pecas", sil.len());
    assert_eq!(sil[0], p, "largura zero tem de devolver a forma AO BIT");
    assert_eq!(
        ph2d_vec_boolean::__sweep_calls(),
        before,
        "largura zero custou um sweep"
    );
}

/// **A silhueta CONTÉM a forma.** O gate acima diz que a fronteira se afastou das âncoras; este diz
/// para que LADO — sem ele, encolher o traço para dentro passaria pelos dois.
#[test]
fn the_silhouette_grows_the_shape_it_never_shrinks_it() {
    let p = stroked(5.0);
    let sil = ph2d_vec_boolean::silhouette_paths(&p);
    let radius = |q: &VecPath| {
        q.verts
            .iter()
            .chain(q.subpaths.iter().flat_map(|c| c.verts.iter()))
            .map(|v| v.anchor[0].hypot(v.anchor[1]))
            .fold(0.0, f64::max)
    };
    let src = radius(&p);
    let out = sil.iter().map(radius).fold(0.0, f64::max);
    assert!(
        out > src + HALF_WIDTH * 0.5,
        "a silhueta tem de CRESCER a forma (raio {src:.4} -> {out:.4})"
    );
}
