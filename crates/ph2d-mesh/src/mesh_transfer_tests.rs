//! Gates da travessia de canais autorados.

use super::*;
use crate::shapes;

/// Pinta a máscara com uma RAMPA em `x` — um campo que um degrau por face
/// não reproduz, e por isso o oráculo distingue interpolar de arredondar.
fn ramp_mask(mesh: &mut Mesh) {
    let xs: Vec<f32> = mesh.positions().iter().map(|p| p[0]).collect();
    let (lo, hi) = xs
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    let span = (hi - lo).max(f32::MIN_POSITIVE);
    let m = mesh.masks_mut();
    for (i, x) in xs.iter().enumerate() {
        m[i] = (x - lo) / span;
    }
}

/// A rampa lida no ponto `p` — o oráculo, e ele NÃO conhece a transferência.
fn want_at(from: &Mesh, p: [f32; 3]) -> f32 {
    let xs: Vec<f32> = from.positions().iter().map(|q| q[0]).collect();
    let (lo, hi) = xs
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    ((p[0] - lo) / (hi - lo).max(f32::MIN_POSITIVE)).clamp(0.0, 1.0)
}

/// **A máscara sobrevive a uma malha inteiramente NOVA.**
///
/// ⚠️ O destino aqui não partilha um vértice com a fonte — é outra esfera, com
/// outra contagem e outra numeração. É o caso do remesh, e é onde uma
/// transferência por ÍNDICE (a que a fusão pode usar) daria lixo bem-formado.
#[test]
fn an_authored_mask_survives_a_topology_it_never_shared() {
    let mut from = shapes::uv_sphere(16, 24, 1.0);
    ramp_mask(&mut from);
    let mut to = shapes::uv_sphere(9, 13, 1.0);
    assert!(to.masks().is_none(), "o destino nasce sem plano");

    transfer_authored(&from, &mut to);

    let got = to.masks().expect("a máscara atravessou").to_vec();
    assert_eq!(got.len(), to.vert_count());
    let mut worst = 0.0f32;
    for (i, &p) in to.positions().iter().enumerate() {
        worst = worst.max((got[i] - want_at(&from, p)).abs());
    }
    // A tolerância é a discretização das duas esferas, não a da aritmética: o
    // ponto mais próximo cai num triângulo cujos vértices amostram a rampa.
    assert!(worst < 0.05, "a rampa chegou torta: pior desvio {worst}");
    // E ela não é uma constante — um `fill` com a média passaria no teste acima
    // se a tolerância fosse frouxa, e não passa neste.
    let (lo, hi) = got
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &m| (a.min(m), b.max(m)));
    assert!(
        hi - lo > 0.9,
        "a máscara saiu chapada ({lo}..{hi}): não é a rampa, é uma constante"
    );
}

/// **Sem plano na fonte, nada é escrito** — e a saída fica byte-idêntica.
///
/// ⚠️ É o CONTROLE que mantém o custo honesto: uma malha virgem não paga um
/// plano por vértice para guardar zeros, e o remesh de quem nunca mascarou
/// continua devolvendo o que devolvia.
#[test]
fn a_source_with_nothing_authored_writes_nothing() {
    let from = shapes::uv_sphere(12, 18, 1.0);
    let mut to = shapes::uv_sphere(7, 9, 1.0);
    let before = to.positions().to_vec();

    transfer_authored(&from, &mut to);

    assert!(
        to.masks().is_none(),
        "materializou um plano de máscara vazio"
    );
    assert!(to.colors().is_none(), "materializou um plano de cor vazio");
    assert_eq!(
        to.positions(),
        &before[..],
        "a geometria não é assunto daqui"
    );
}

/// **A COR viaja pela mesma porta** — ela é autorada como a máscara.
#[test]
fn an_authored_colour_travels_too() {
    let mut from = shapes::uv_sphere(14, 20, 1.0);
    for (i, c) in from.colors_mut().iter_mut().enumerate() {
        // Vermelho num hemisfério, azul no outro — um campo com FRONTEIRA.
        *c = if i % 2 == 0 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
    }
    let mut to = shapes::uv_sphere(8, 11, 1.0);
    transfer_authored(&from, &mut to);
    let got = to.colors().expect("a cor atravessou");
    assert_eq!(got.len(), to.vert_count());
    assert!(
        got.iter().any(|c| c[0] > 0.2) && got.iter().any(|c| c[2] > 0.2),
        "as duas cores autoradas têm de aparecer no destino"
    );
}

/// **O AO e a espessura NÃO viajam** — eles são MEDIÇÕES da geometria, e a
/// geometria mudou.
///
/// ⚠️ Este é o gate que separa a decisão de um esquecimento. Carregar uma
/// oclusão medida noutra forma é entregar ao artista um número errado sem
/// sintoma: a peça fica sombreada por uma malha que não existe mais.
#[test]
fn a_measurement_of_the_old_shape_does_not_travel() {
    let mut from = shapes::uv_sphere(12, 18, 1.0);
    ramp_mask(&mut from);
    from.put_ao(vec![0.5; from.vert_count()], false);
    from.put_thickness(vec![0.25; from.vert_count()]);

    let mut to = shapes::uv_sphere(7, 9, 1.0);
    transfer_authored(&from, &mut to);

    assert!(to.masks().is_some(), "o AUTORADO viaja — o controle");
    assert!(
        to.ao().is_none(),
        "a oclusão de outra forma chegou ao destino"
    );
    assert!(
        to.thickness().is_none(),
        "a espessura de outra forma chegou ao destino"
    );
}

/// **O destino pode estar LONGE da fonte, e a resposta continua a mais próxima.**
///
/// ⚠️ **É este gate que dá dentes ao rejeito barato.** Com o destino POUSADO
/// sobre a fonte o primeiro candidato já dá distância ~0, o `melhor` é
/// desprezível e o termo cruzado do rejeito (`2·r·√melhor`) some — então uma
/// versão agressiva demais dele passa nos outros gates sem mudar uma resposta.
/// Aqui o melhor vale ~0,04 e o raio de um triângulo é da mesma ordem: a faixa
/// onde o termo decide existe, e um rejeito sem ele descarta o vencedor.
///
/// ⚠️ E o caso é REAL: a extração do remesh emite vértices a até uma célula da
/// superfície, e numa resolução grossa uma célula é uma fração visível do
/// modelo.
#[test]
fn a_destination_that_floats_off_the_source_still_gets_the_nearest_value() {
    let mut from = shapes::uv_sphere(16, 24, 1.0);
    ramp_mask(&mut from);
    // Uma casca 20% maior — todo vértice a ~0,2 da superfície de onde vem.
    let mut to = shapes::uv_sphere(11, 15, 1.2);

    transfer_authored(&from, &mut to);

    let got = to.masks().expect("a máscara atravessou");
    // O oráculo é a rampa lida na DIREÇÃO do vértice — a projeção radial é o
    // ponto mais próximo numa esfera concêntrica, e é geometria, não a função
    // sob teste.
    let mut worst = 0.0f32;
    for (i, &p) in to.positions().iter().enumerate() {
        let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        let on_source = [p[0] / n, p[1] / n, p[2] / n];
        worst = worst.max((got[i] - want_at(&from, on_source)).abs());
    }
    assert!(
        worst < 0.06,
        "o valor não é o do ponto mais próximo: pior desvio {worst}"
    );
}

/// **O rejeito barato é EXATO** — a força bruta é o oráculo.
///
/// ⚠️ **Uma amostra não decide isto, e a tentativa está registrada:** a mutação
/// que aperta o rejeito (tirar o termo cruzado `2·r·√melhor`) sobreviveu a
/// TRÊS fixtures de aparência — inclusive uma com o destino flutuando longe da
/// fonte. A janela em que ele erra é estreita **e depende da ORDEM** em que o
/// octree devolve as faces, então procurá-la por amostragem é procurar uma
/// coincidência.
///
/// O oráculo honesto é o outro: testar TODOS os triângulos, sem rejeito nenhum,
/// e exigir o mesmo valor. Qualquer descarte indevido escolhe um triângulo
/// estritamente mais longe, e o valor interpolado ali é outro.
#[test]
fn the_cheap_reject_never_drops_the_winner() {
    // ⚠️ **A fonte é GROSSA de propósito.** O termo cruzado só decide quando o
    // raio da esfera envolvente de um triângulo é comparável à distância até a
    // superfície — numa malha fina o centroide e o ponto mais próximo quase
    // coincidem, e a folga do rejeito nunca é exercitada.
    let mut from = shapes::uv_sphere(4, 6, 1.0);
    ramp_mask(&mut from);
    // Três distâncias: pousado, perto e longe — o `melhor` de partida muda de
    // ordem de grandeza, que é o eixo em que o termo cruzado decide.
    for radius in [1.0f32, 1.02, 1.05, 1.3] {
        let mut to = shapes::uv_sphere(17, 23, radius);
        transfer_authored(&from, &mut to);
        let got = to.masks().expect("a máscara atravessou").to_vec();

        let mut tris: Vec<[u32; 3]> = Vec::new();
        from.triangle_indices(&mut tris);
        let pos = from.positions();
        let src = from.masks().expect("a fonte foi mascarada");
        for (i, &p) in to.positions().iter().enumerate() {
            // FORÇA BRUTA: todo triângulo, sem octree e sem rejeito.
            let mut best = (f32::INFINITY, 0.0f32);
            for &[a, b, c] in &tris {
                let e = TriEdges::new(pos[a as usize], pos[b as usize], pos[c as usize]);
                let (sq, s, t) = e.closest_bary(p);
                if sq < best.0 {
                    let w = [1.0 - s - t, s, t];
                    best = (
                        sq,
                        w[0] * src[a as usize] + w[1] * src[b as usize] + w[2] * src[c as usize],
                    );
                }
            }
            assert!(
                (got[i] - best.1).abs() < 1e-5,
                "raio {radius}, vértice {i}: o rejeito descartou o vencedor \
                 ({} contra {} da força bruta)",
                got[i],
                best.1
            );
        }
    }
}
