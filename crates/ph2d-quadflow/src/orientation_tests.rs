//! **OS GATES DO CAMPO DE ORIENTAÇÃO** (ADR-0160 §4, asserção A8 e vizinhas).
//!
//! ⚠️ **O oráculo NÃO é *"o campo parece bom"***. São quatro propriedades
//! executáveis: a simetria de 4 dobras da representação, a MONOTONIA da energia
//! (a régua da convergência), o ponto fixo sobre um plano, e o determinismo.
//!
//! ```text
//! cargo test -p ph2d-quadflow
//! ```

use ph2d_mesh::{Face, Mesh, shapes};

use super::{compat_orientation_extrinsic_4, solve_orientation};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Uma malha com curvatura de verdade nas duas direções — é ela que contém o
/// fenômeno que um campo cruzado existe para descrever.
///
/// ⚠️ **Um TORO e não uma esfera.** Sobre a esfera as duas curvaturas principais
/// são iguais em todo ponto, então **não há direção preferida** — o campo pode
/// pousar em qualquer lugar e ainda estar certo, e um gate sobre isso não
/// discrimina nada. O toro tem direções principais bem definidas (o meridiano e
/// o paralelo) e um gênero que não é o da esfera, que é o que a Q3 vai precisar.
fn fixture() -> Mesh {
    shapes::torus(48, 24, 1.0, 0.35)
}

/// ⭐ **A REPRESENTAÇÃO TEM SIMETRIA DE 4 DOBRAS** — a asserção A8 do ADR-0160.
///
/// Trocar o representante guardado por qualquer uma das outras três direções da
/// mesma cruz **não pode mudar o alinhamento medido**. É a propriedade que
/// define um campo 4-RoSy, e é a que uma soma ingênua (`q0 + q1`) destrói: dois
/// vértices que descrevem a MESMA grade com representantes girados de 90° somam
/// para um vetor a 45° de ambos, e a grade que sai não é a de nenhum dos dois.
#[test]
fn the_representation_is_four_fold_symmetric() {
    let n0 = [0.0, 0.0, 1.0];
    let n1 = [0.0, 0.0, 1.0];
    let q0 = [1.0, 0.0, 0.0];
    // Um campo vizinho a 20° — perto o bastante para o alinhamento ser
    // inequívoco, longe o bastante para não empatar.
    let (s, c) = (20.0f32.to_radians().sin(), 20.0f32.to_radians().cos());
    let q1 = [c, s, 0.0];

    let (a, b) = compat_orientation_extrinsic_4(q0, n0, q1, n1);
    let base = dot(a, b);
    assert!(
        base > 0.9,
        "os representantes escolhidos nao sao os mais proximos ({base})"
    );

    // As QUATRO direções da cruz de `q1`, uma a uma.
    let perp = cross(n1, q1);
    for (k, alt) in [
        q1,
        perp,
        [-q1[0], -q1[1], -q1[2]],
        [-perp[0], -perp[1], -perp[2]],
    ]
    .into_iter()
    .enumerate()
    {
        let (a2, b2) = compat_orientation_extrinsic_4(q0, n0, alt, n1);
        let got = dot(a2, b2);
        assert!(
            (got - base).abs() < 1.0e-5,
            "a direcao {k} da MESMA cruz mediu {got} contra {base}: a representacao nao e' 4-RoSy, \
             e uma soma ingenua de campos vizinhos vai cancelar"
        );
    }
}

/// **O ALINHAMENTO É SIMÉTRICO** — perguntar `(a, b)` ou `(b, a)` dá o mesmo
/// número.
///
/// ⚠️ Sem isto a energia dependeria da direção em que a aresta é percorrida, e a
/// soma sobre arestas dirigidas contaria coisas diferentes nas duas passagens —
/// uma régua que se move enquanto se mede.
#[test]
fn the_alignment_does_not_depend_on_which_side_asks() {
    let n0 = [0.0, 0.0, 1.0];
    let n1 = [0.0, 0.3, 0.954_f32];
    let q0 = [1.0, 0.0, 0.0];
    let q1 = [0.6, 0.8, 0.0];

    let (a, b) = compat_orientation_extrinsic_4(q0, n0, q1, n1);
    let (c, d) = compat_orientation_extrinsic_4(q1, n1, q0, n0);
    assert!(
        (dot(a, b).abs() - dot(c, d).abs()).abs() < 1.0e-6,
        "o alinhamento mudou ao trocar os lados: {} vs {}",
        dot(a, b).abs(),
        dot(c, d).abs()
    );
}

/// ⭐ **A ENERGIA NUNCA SOBE** — a régua da convergência (ADR-0160 §5, Q1).
///
/// ⚠️ **É o gate que mede a LEI, e não a aparência.** Uma suavização que
/// divergisse produziria um campo perfeitamente plausível de se olhar e uma
/// energia a crescer; e é a energia que a hierarquia multirresolução da Q2 vai
/// ter de melhorar — sem esta régua, aquela wave não teria contra o que se
/// comparar.
#[test]
fn the_energy_never_climbs() {
    let mesh = fixture();

    // ⚠️ **A TOLERÂNCIA É DERIVADA, NUNCA ESCOLHIDA** (`CLAUDE.md` §0.0). A
    // energia é uma soma de `n` termos, cada um vindo de produtos escalares em
    // `f32`, acumulada em `f64` — o erro absoluto do somatório é da ordem de
    // `n · ε(f32)`. Uma barra menor que isso reprova a ARITMÉTICA e chama-lhe
    // divergência: foi o que aconteceu na primeira corrida, em que 256 → 512
    // varreduras "subiu" **1,7e-5** sobre um campo que já tinha convergido.
    let edges: usize = (0..mesh.vert_count())
        .map(|v| mesh.adjacency().vert_verts.neighbours(v).len())
        .sum();
    let noise = edges as f64 * f64::from(f32::EPSILON);

    let mut last = f64::INFINITY;
    let mut seen = Vec::new();
    for it in [0usize, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512] {
        let f = solve_orientation(&mesh, it);
        let e = f.energy(&mesh);
        assert!(
            e <= last + noise,
            "a energia SUBIU de {last} para {e} em {it} varreduras (ruido {noise:e}): a suavizacao \
             esta' a divergir"
        );
        last = e;
        seen.push((it, e));
    }
    eprintln!("[quadflow] {edges} arestas dirigidas, ruido {noise:e}");
    for (it, e) in &seen {
        eprintln!("[quadflow]   {it:>3} varreduras -> energia {e:.4}");
    }

    // **CONVERGE**, e o gate diz o que isso quer dizer: as duas últimas medições
    // são a mesma dentro do ruído. Sem esta metade, um campo que descesse para
    // sempre — devagar, sem nunca parar — passaria pela monotonia acima.
    let (a, b) = (seen[seen.len() - 2].1, seen[seen.len() - 1].1);
    assert!(
        (a - b).abs() <= noise,
        "de {} para {} varreduras a energia ainda anda ({a} -> {b}): o campo nao convergiu",
        seen[seen.len() - 2].0,
        seen[seen.len() - 1].0
    );

    // **E a suavização de facto ANDA** — a semente não é o ponto fixo. O número
    // é o MEDIDO nesta fixture (toro 48×24), não um alvo escolhido: qualquer
    // regressão que faça o passe parar de andar cai aqui.
    let (seed, settled) = (seen[0].1, b);
    assert!(
        settled < seed * 0.9,
        "a suavizacao mal andou: semente {seed:.4} -> convergido {settled:.4}"
    );
}

/// **SOBRE UM PLANO o campo converge para uma direção ÚNICA.**
///
/// ⚠️ **É o caso em que a resposta certa é conhecida sem oráculo escrito à mão:**
/// num plano não há curvatura para preferir direção nenhuma, então o mínimo da
/// energia é o campo CONSTANTE — energia exatamente zero. Um campo que ficasse
/// com energia residual num plano estaria a errar no caso mais fácil que existe.
///
/// A fixture é a face de um cubo grande: normais todas iguais, e uma vizinhança
/// que não é degenerada.
#[test]
fn on_a_flat_patch_the_field_becomes_constant() {
    let mesh = grid(12, 12);
    let f = solve_orientation(&mesh, 64);
    let e = f.energy(&mesh);
    assert!(
        e < 1.0e-3,
        "o campo num plano guardou energia {e}: ele nao convergiu para a direcao unica que o plano \
         admite"
    );

    // E a direção é a MESMA em toda parte, a menos da cruz.
    let d0 = f.dir(0);
    let n0 = mesh.normals()[0];
    for v in 1..f.len() {
        let (a, b) = compat_orientation_extrinsic_4(d0, n0, f.dir(v), mesh.normals()[v]);
        assert!(
            dot(a, b) > 0.999,
            "o vertice {v} ficou a {:.4} do vertice 0 num plano",
            dot(a, b)
        );
    }
}

/// **DETERMINÍSTICO** (HR-5): duas corridas, a mesma malha, o mesmo campo ao bit.
///
/// ⚠️ **Sem isto o remesh nao e' reproduzivel:** o artista reabriria o projeto e
/// a malha sairia outra. É a razão de a semente ser derivada da normal em vez de
/// aleatória, e de o laço ter ordem de visita fixa.
#[test]
fn the_field_is_bit_reproducible() {
    let mesh = fixture();
    let a = solve_orientation(&mesh, 8);
    let b = solve_orientation(&mesh, 8);
    assert_eq!(a, b, "duas corridas deram campos diferentes");
}

/// **A SEMENTE JÁ É TANGENTE** — e continua tangente depois de suavizar.
///
/// ⚠️ Um campo que saísse do plano tangente descreveria uma grade que não vive na
/// superfície, e o defeito apareceria só na EXTRAÇÃO (Q3), a três ondas de
/// distância da causa.
#[test]
fn every_direction_stays_in_the_tangent_plane() {
    let mesh = fixture();
    for its in [0usize, 8] {
        let f = solve_orientation(&mesh, its);
        for v in 0..f.len() {
            let d = f.dir(v);
            let n = mesh.normals()[v];
            assert!(
                dot(d, n).abs() < 1.0e-3,
                "com {its} varreduras o vertice {v} tem campo fora do plano tangente ({})",
                dot(d, n)
            );
            let len = dot(d, d).sqrt();
            assert!(
                (len - 1.0).abs() < 1.0e-3,
                "o campo do vertice {v} nao e' unitario ({len})"
            );
        }
    }
}

/// Uma grade plana `w × h` de quads triangulados — o plano do gate acima.
///
/// ⚠️ Local a este arquivo e não uma `shape` nova: um plano não é uma primitiva
/// que o artista peça, e pô-lo na `ph2d-mesh` seria alargar a superfície pública
/// dela para servir um teste.
fn grid(w: usize, h: usize) -> Mesh {
    let mut positions = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            positions.push([x as f32, y as f32, 0.0]);
        }
    }
    // ⚠️ **QUADS e não triângulos**, e não é estética: a diagonal de um triângulo
    // é uma aresta a mais na adjacência, e ela puxaria o campo para 45° num
    // gate cujo ponto inteiro é *o plano não prefere direção nenhuma*.
    let mut faces = Vec::new();
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let i = (y * w + x) as u32;
            let (r, d) = (i + 1, i + w as u32);
            faces.push(Face::quad(i, d, d + 1, r));
        }
    }
    Mesh::from_parts(positions, faces).expect("a grade e' bem formada")
}
