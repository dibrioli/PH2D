//! Gates do [`TriEdges`].
//!
//! O oráculo da distância é uma **busca por força bruta** sobre o triângulo, não
//! uma segunda escrita do particionamento: sete regiões espelhadas num teste
//! concordariam com o produto exatamente onde ele erra.

use super::*;

/// O triângulo canônico: `s` corre em x, `t` corre em y, e a hipotenusa é
/// `x + y = 1`. Com ele, saber em que região um ponto cai é olhar as
/// coordenadas.
fn canonical() -> TriEdges {
    TriEdges::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
}

/// O menor `|p − q|²` sobre uma amostragem densa do triângulo. É um **limite
/// superior** da resposta exata, e o erro dele é o passo da amostragem.
fn brute_force_sq(p: [f32; 3], v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> f32 {
    const N: usize = 400;
    let mut best = f32::INFINITY;
    for i in 0..=N {
        let s = i as f32 / N as f32;
        for j in 0..=(N - i) {
            let t = j as f32 / N as f32;
            let q = [
                v1[0] + s * (v2[0] - v1[0]) + t * (v3[0] - v1[0]),
                v1[1] + s * (v2[1] - v1[1]) + t * (v3[1] - v1[1]),
                v1[2] + s * (v2[2] - v1[2]) + t * (v3[2] - v1[2]),
            ];
            let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2);
            if d < best {
                best = d;
            }
        }
    }
    best
}

/// As sete regiões, cada uma com um ponto que cai NELA. O nome de cada uma é o
/// do diagrama do Eberly que a referência reproduz no comentário.
const PROBES: [(&str, [f32; 3]); 7] = [
    ("0 — interior", [0.3, 0.3, 1.0]),
    ("1 — hipotenusa", [1.0, 1.0, 0.5]),
    ("2 — canto v3", [-0.5, 1.5, 0.2]),
    ("3 — aresta v1v3", [-1.0, 0.5, 0.0]),
    ("4 — canto v1", [-1.0, -1.0, 0.3]),
    ("5 — aresta v1v2", [0.5, -1.0, 0.0]),
    ("6 — canto v2", [1.5, -0.5, 0.4]),
];

#[test]
fn the_closest_point_matches_a_brute_force_search_over_the_triangle() {
    let (v1, v2, v3) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let tri = canonical();
    for (name, p) in PROBES {
        let (sq, _) = tri.closest_to(p);
        let brute = brute_force_sq(p, v1, v2, v3);
        // O exato nunca pode ser PIOR que uma amostra do próprio triângulo.
        assert!(
            sq <= brute + 1e-5,
            "região {name}: exato {sq:.6} > força bruta {brute:.6}"
        );
        // E não pode ser melhor do que a amostragem consegue enxergar.
        assert!(
            brute - sq < 2e-4,
            "região {name}: exato {sq:.6} longe demais da força bruta {brute:.6}"
        );
    }
}

#[test]
fn the_closest_point_lies_on_the_triangle() {
    let tri = canonical();
    for (name, p) in PROBES {
        let (sq, q) = tri.closest_to(p);
        // No triângulo canônico, "estar no triângulo" é `z == 0` e `(x, y)` no
        // simplexo — a forma mais barata de dizer que a resposta é um ponto DELE
        // e não um ponto qualquer do plano.
        assert!(q[2].abs() < 1e-5, "{name}: fora do plano ({})", q[2]);
        assert!(
            q[0] >= -1e-5 && q[1] >= -1e-5 && q[0] + q[1] <= 1.0 + 1e-5,
            "{name}: fora do simplexo ({}, {})",
            q[0],
            q[1]
        );
        // E a distância reportada é a distância ATÉ o ponto reportado — se as
        // duas saídas discordarem, quem consome uma delas está sozinho.
        let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2);
        assert!((d - sq).abs() < 1e-4, "{name}: {d:.6} != {sq:.6}");
    }
}

/// A região 6 tem gate próprio porque foi ali que o helper compartilhado da
/// hipotenusa **estava errado**: ela é a espelhada, e a forma quadrática casa
/// `a00` com `s` e `a11` com `t`, então trocar os dois papéis não é simetria
/// dela.
///
/// ⚠️ **E o triângulo canônico NÃO contém o fenômeno** — com `a01 = 0` o ramo
/// deslizante da região 6 é *inalcançável*: ele exige `s + t > det` e
/// `tmp1 > tmp0` ao mesmo tempo, e para o retângulo isóceles as duas
/// desigualdades se contradizem. Ali a região 6 sempre colapsa no canto `v2`.
/// Por isso a fixture é **enviesada** (`v3` puxado para longe): sem isso o gate
/// ficaria verde sobre o defeito.
///
/// ⚠️ E o oráculo tem de ser a **DISTÂNCIA**: o helper trocado devolve `(s, t)`
/// CERTOS e o quadrado errado (0.245 contra 1.225 nesta fixture), então um gate
/// que só olhasse a posição do ponto mais próximo passaria.
#[test]
fn the_mirrored_hypotenuse_region_is_not_the_shared_one() {
    let (v1, v2, v3) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]);
    let tri = TriEdges::new(v1, v2, v3);
    let p = [1.5, -0.2, 0.0];
    let (sq, q) = tri.closest_to(p);

    // O mínimo cai DENTRO do segmento v2→v3, não num canto — é isso que prova
    // que o ramo deslizante rodou.
    assert!(
        (q[0] - 1.15).abs() < 1e-4 && (q[1] - 0.15).abs() < 1e-4,
        "esperava (1.15, 0.15), veio ({:.4}, {:.4})",
        q[0],
        q[1]
    );
    let brute = brute_force_sq(p, v1, v2, v3);
    assert!(sq <= brute + 1e-5, "exato {sq:.6} > força bruta {brute:.6}");
    assert!(brute - sq < 2e-4, "exato {sq:.6} vs força bruta {brute:.6}");
    // O número literal, para a mutação ter onde sangrar sem depender da
    // resolução da amostragem.
    assert!((sq - 0.245).abs() < 1e-4, "{sq:.6}");
}

#[test]
fn a_point_on_the_triangle_is_at_zero_distance() {
    let tri = canonical();
    let (sq, q) = tri.closest_to([0.25, 0.25, 0.0]);
    assert!(sq < 1e-6, "{sq}");
    assert!((q[0] - 0.25).abs() < 1e-5 && (q[1] - 0.25).abs() < 1e-5);
}

/// **OS DOIS MÖLLER–TRUMBORE CONCORDAM NA ARESTA PARTILHADA** — e este gate
/// nasceu afirmando o CONTRÁRIO.
///
/// ⚠️ Ele chamava-se `the_lenient_ray_hits_the_shared_edge_that_the_strict_one_misses`
/// e pinava a estreiteza do irmão de picking como decisão, com a justificativa
/// do cabeçalho do módulo. A medição de 2026-08-16 derrubou a justificativa (ver
/// [`crate::ray::BARY_SLACK`]): recusar dos DOIS lados não é uma escolha
/// ambígua, é um buraco, e ele custava `1 em 6144` empurrões de um ULP a trocar
/// acerto por erro no pick.
///
/// **A metade que sobrevive é a que sempre importou** — *o ruído de `f32` na
/// borda não pode apagar uma superfície* —, e ela passou a ser exigida dos dois.
/// A mutação que re-estreita qualquer um deles sangra aqui.
#[test]
fn both_rays_hit_the_shared_edge_and_absorb_f32_noise() {
    // Dois triângulos que partilham a aresta de x = 0 a x = 1 em y = 0. Um raio
    // que sobe exatamente por cima dessa aresta tem de acertar ALGUÉM — no
    // voxelizador, errar os dois é um furo por onde o flood fill escapa.
    let a = TriEdges::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let origin = [0.5, 0.0, -1.0];
    let dir = [0.0, 0.0, 1.0];

    // O tolerante acerta: o ponto cai em t = 0, na fronteira exata.
    let lenient = a.ray_hit(origin, dir);
    assert!(lenient.is_some(), "o tolerante recusou a aresta partilhada");
    assert!((lenient.unwrap() - 1.0).abs() < 1e-4);

    // O de PICKING acerta o mesmo caso — o controle de que o par não divergiu
    // no lado fácil (t = 0 satisfaz qualquer das duas leis).
    let strict = crate::ray::ray_triangle(
        origin,
        dir,
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    assert!(strict.is_some(), "o de picking recusou a aresta partilhada");

    // ⚠️ **O caso que DECIDE**: uma barycêntrica em −3e-7 — dentro do ruído de
    // `f32`, fora do intervalo fechado. Era aqui que os dois divergiam, e é aqui
    // que a malha de QUADS vazava: cada face é partida em `(0,1,2)` e `(0,2,3)`,
    // que testam a diagonal com ordens de vértice diferentes, então uma recusa
    // por ruído acontece nos DOIS e o raio atravessa a superfície.
    let just_outside = [0.5, -3e-7, -1.0];
    assert!(
        a.ray_hit(just_outside, dir).is_some(),
        "o do remesh tem de absorver ruído de f32 na borda"
    );
    assert!(
        crate::ray::ray_triangle(
            just_outside,
            dir,
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0]
        )
        .is_some(),
        "o de PICKING também — recusar aqui é o buraco de 1-em-6144 ULPs"
    );
}

/// ⚠️ Pina a armadilha de porte: o épsilon da referência é um número de `f64`,
/// e copiá-lo daria uma função ESTRITA com um comentário dizendo o contrário.
#[test]
fn the_references_epsilon_would_be_inert_in_f32() {
    let js_epsilon: f32 = 1e-15;
    assert_eq!(
        1.0f32 + js_epsilon,
        1.0f32,
        "se isto deixar de valer, a folga pode voltar a ser a do original"
    );
    // A nossa, ao contrário, é observável.
    assert_ne!(1.0f32 + BARY_SLACK, 1.0f32);
}

#[test]
fn a_ray_running_along_the_triangles_plane_misses() {
    let tri = canonical();
    assert!(tri.ray_hit([-1.0, 0.25, 0.0], [1.0, 0.0, 0.0]).is_none());
}

#[test]
fn a_ray_pointing_away_from_the_triangle_misses() {
    let tri = canonical();
    assert!(tri.ray_hit([0.25, 0.25, 1.0], [0.0, 0.0, 1.0]).is_none());
}
