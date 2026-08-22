//! Os gates da ponte.
//!
//! ⚠️ **A pergunta que eles respondem não é "saiu um número", é "saiu uma DISTÂNCIA"** — porque é a
//! propriedade de distância, e não o valor, que faz a marcha de raios funcionar e a booleana ser
//! `min`/`max`.
//!
//! ⚠️ **E o campo tem DOIS regimes, com duas promessas diferentes** — medir os dois com a mesma
//! barra foi o primeiro erro desta wave, e ele reprovava código correto:
//!
//! | onde | promessa |
//! |---|---|
//! | dentro da caixa | é a distância, a menos de uma célula e do fator de [`CHAMFER_SAFETY`] |
//! | fora da caixa | é a distância **à caixa** — um minorante honesto, e nada mais |

use super::*;

/// A esfera de referência: uma malha, e a fórmula que ela aproxima.
fn sphere(radius: f32) -> (Mesh, impl Fn([f32; 3]) -> f32) {
    let mesh = ph2d_mesh::shapes::uv_sphere(64, 128, radius);
    (mesh, move |p: [f32; 3]| {
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - radius
    })
}

/// Uma varredura determinística por dentro da caixa. Sem `rand`, e reprodutível.
fn walk(f: &SampledField, n: usize, reach: f32) -> Vec<[f32; 3]> {
    let b = f.bounds();
    (0..n)
        .filter_map(|i| {
            let t = i as f32 / n as f32;
            let p = [
                (t * 37.0).sin() * reach,
                (t * 41.0).cos() * reach,
                (t * 43.0).sin() * reach,
            ];
            (distance_to_box(b, p) <= 0.0).then_some(p)
        })
        .collect()
}

/// ⭐ **Perto da superfície o campo É a distância da malha** — contra o ORÁCULO analítico.
///
/// ⚠️ **A barra é a célula, e é derivada e não escolhida.** Uma grade não sabe onde a superfície
/// está com mais precisão do que a distância entre duas amostras; exigir menos seria exigir que a
/// representação desobedecesse a si mesma. O gate mede a razão erro/célula e exige que ela **não
/// cresça** ao refinar, que é a afirmação com conteúdo.
///
/// ⚠️ **Só perto da superfície**, e a razão é a [`CHAMFER_SAFETY`]: o campo é deliberadamente um
/// **minorante**, então a `|d|` grande ele fica sistematicamente curto — o que é a promessa a
/// funcionar, não um erro. É o gate seguinte que mede aquele regime.
#[test]
fn near_the_surface_the_sampled_field_is_the_distance_of_the_mesh() {
    let (mesh, oracle) = sphere(0.6);
    let mut ratios = Vec::new();
    for res in [48u32, 96, 192] {
        let f = SampledField::from_mesh(&mesh, res).expect("a esfera não é vazia");
        let cell = f.cell();
        let mut worst = 0.0f32;
        let mut seen = 0usize;
        for p in walk(&f, 20_000, 0.8) {
            let truth = oracle(p);
            // A banda exata: onde a geometria de facto se decide.
            if truth.abs() > cell * 3.0 {
                continue;
            }
            seen += 1;
            worst = worst.max((f.at(p) - truth).abs());
        }
        assert!(seen > 50, "a fixture não visitou a casca: {seen} pontos");
        ratios.push(worst / cell);
    }
    for r in &ratios {
        assert!(
            *r < 1.0,
            "o pior erro é {r:.2} célula — uma grade não erra mais que a própria célula: {ratios:?}"
        );
    }
    assert!(
        ratios[2] <= ratios[0] * 1.2,
        "refinar não pode PIORAR a razão erro/célula: {ratios:?}"
    );
}

/// ⭐ **Em toda a caixa o campo NUNCA superestima** — a propriedade que a marcha consome.
///
/// ⚠️ **É a metade que decide se um raio atravessa a peça.** Uma marcha de esfera avança pelo valor
/// do campo; se o valor for maior que a distância verdadeira, o passo salta por cima da superfície e
/// o raio sai pelo outro lado **sem a ver** — e o sintoma na tela é um buraco, não um erro.
#[test]
fn inside_the_box_the_field_never_overshoots() {
    let (mesh, oracle) = sphere(0.6);
    let f = SampledField::from_mesh(&mesh, 96).expect("esfera");
    let tol = f.cell();
    let mut worst_ratio = 0.0f32;
    for p in walk(&f, 20_000, 0.9) {
        let truth = oracle(p);
        let got = f.at(p);
        assert!(
            got.abs() <= truth.abs() + tol,
            "em {p:?}: o campo devolveu {got:.4} e a distância verdadeira é {truth:.4} — \
             superestimar é o erro que faz a marcha atravessar a peça"
        );
        if truth.abs() > tol * 4.0 {
            worst_ratio = worst_ratio.max(got.abs() / truth.abs());
        }
    }
    // ⚠️ **Controlo positivo do lado de baixo**: um campo que devolvesse zero em toda a parte
    // passaria o assert acima e nunca convergiria. O minorante tem de ser ÚTIL.
    assert!(
        worst_ratio > 0.7,
        "o minorante ficou frouxo demais ({worst_ratio:.3}) — a marcha andaria a passos inúteis"
    );
}

/// ⭐ **O SINAL está certo**: negativo dentro, positivo fora.
///
/// ⚠️ É o gate que separa "um campo" de "um campo do avesso". Uma malha voxelizada sem *flood fill*
/// devolve distância certa e sinal **sempre positivo** — e a peça inteira desapareceria da booleana
/// sem um único erro em lado nenhum.
#[test]
fn inside_is_negative_and_outside_is_positive() {
    let (mesh, _) = sphere(0.6);
    let f = SampledField::from_mesh(&mesh, 96).expect("esfera");
    assert!(f.at([0.0, 0.0, 0.0]) < -0.5, "o centro é o mais fundo");
    for p in [[0.3f32, 0.0, 0.0], [0.0, 0.0, -0.4], [0.2, 0.2, 0.2]] {
        assert!(f.at(p) < 0.0, "{p:?} está dentro da esfera de 0,6");
    }
    for p in [[0.65f32, 0.0, 0.0], [0.0, -0.7, 0.0], [0.5, 0.5, 0.5]] {
        assert!(f.at(p) > 0.0, "{p:?} está fora da esfera de 0,6");
    }
}

/// ⭐ **`‖∇f‖` é constante e CONHECIDO** — a segunda metade de *"isto é uma distância"*.
///
/// ⚠️ Um campo com o sinal certo e gradiente 0,5 faz a marcha andar **metade** do que podia; um com
/// 2,0 fá-la **atravessar**. Aqui o valor esperado não é 1: é `1 / CHAMFER_SAFETY`, porque a divisão
/// que torna o campo um minorante é uniforme — e o gate mede exatamente isso, em vez de aceitar
/// qualquer coisa perto de 1.
#[test]
fn the_sampled_field_marches_like_a_distance() {
    let (mesh, _) = sphere(0.6);
    let f = SampledField::from_mesh(&mesh, 128).expect("esfera");
    let eps = f.cell();
    let want = 1.0 / CHAMFER_SAFETY;
    let mut worst: f32 = 0.0;
    for i in 0..500 {
        let a = i as f32 * 0.0125;
        // A ~3 células fora da casca, que é onde a marcha de facto pergunta.
        let r = 0.6 + f.cell() * 3.0;
        let p = [r * a.cos(), r * (a * 0.7).sin(), r * a.sin() * 0.5];
        let d = |k: usize| {
            let (mut lo, mut hi) = (p, p);
            lo[k] -= eps;
            hi[k] += eps;
            (f.at(hi) - f.at(lo)) / (2.0 * eps)
        };
        let g = [d(0), d(1), d(2)];
        let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        worst = worst.max((n - want).abs());
    }
    assert!(
        worst < 0.2,
        "‖∇f‖ desviou {worst:.3} de {want:.3} perto da casca — a marcha ou anda de menos ou \
         atravessa"
    );
}

/// ⭐ **FORA da caixa a resposta é a distância à CAIXA** — e ela é uma distância honesta.
///
/// ⚠️ **O `VoxelField::sample` devolve ali a DIAGONAL da grade.** Está certo para os consumidores
/// dele (oclusão e espessura só perguntam dentro) e é fatal para uma marcha: um passo do tamanho da
/// diagonal salta a peça inteira. O gate mede as duas metades que uma marcha exige: nunca
/// superestimar, e **crescer** ao afastar (senão o raio nunca chega).
#[test]
fn outside_the_box_the_distance_never_overshoots() {
    let (mesh, oracle) = sphere(0.6);
    let f = SampledField::from_mesh(&mesh, 96).expect("esfera");
    let far = f.bounds().max[0];
    assert!(far > 0.6, "a caixa tem de conter a esfera");
    let mut last = f32::NEG_INFINITY;
    for i in 1..40 {
        let r = far + i as f32 * 0.25;
        let p = [r, 0.0, 0.0];
        let d = f.at(p);
        assert!(
            d <= oracle(p) + 1e-4,
            "em r = {r}: o campo devolveu {d:.4} contra {:.4} de verdade",
            oracle(p)
        );
        assert!(
            d > last,
            "e a distância tem de CRESCER ao afastar: {d} depois de {last}"
        );
        last = d;
    }
}

/// ⭐ **A caixa sobra em relação à peça**, e é isso que deixa um filete existir.
///
/// ⚠️ Sem a folga da [`PAD_CELLS`] a grade encosta na malha com 1,51 células, e fora dela a resposta
/// é a distância à **caixa**. Um arredondamento entre a escultura e uma forma analítica acontece até
/// um raio para fora da escultura — e seria calculado contra a caixa, silenciosamente.
#[test]
fn the_box_leaves_room_for_a_fillet() {
    let (mesh, _) = sphere(0.6);
    let f = SampledField::from_mesh(&mesh, 96).expect("esfera");
    let margin = f.bounds().max[0] - 0.6;
    assert!(
        margin >= f.cell() * 8.0,
        "só {margin:.4} de folga ({:.1} células) — um filete maior que isso sai da caixa",
        margin / f.cell()
    );
}

/// ⭐ **A PAREDE DA CAIXA NÃO É UMA SUPERFÍCIE** — o gate que faltava, e o smoke pagou por ele.
///
/// ⚠️ **Os dois gates vizinhos passavam sobre o código quebrado.** Um mede o regime de dentro, o
/// outro o de fora, e cada um estava certo no seu lado: por dentro o campo valia a distância à peça
/// (+0,10 junto da parede), por fora crescia a partir de zero. O que ninguém media era a **costura**
/// — e ali o valor caía de 0,10 para 0,000 num passo.
///
/// **Um campo que cai a zero numa superfície TEM uma superfície ali.** A marcha encontrava-a e
/// parava: a caixa da grade virava um cubo sólido na tela, com a peça a aparecer só por dentro do
/// furo que a booleana abrira (Enio, smoke de 21/08: *"um objeto texturizado dentro de um cubo
/// furado"*).
///
/// *Dois regimes certos não fazem um campo certo. Quem mede um de cada vez nunca vê a emenda.*
#[test]
fn there_is_no_false_surface_at_the_wall_of_the_box() {
    let (mesh, _) = sphere(0.6);
    // ⚠️ **A RESOLUÇÃO varre, e é isso que faz o gate provar alguma coisa.** A esfera sozinha, a uma
    // resolução só, passava sobre o código quebrado: o índice da parede caía exato ali. Quem
    // empurra o arredondamento para o outro lado é a combinação `dims`/`passo`, que muda a cada
    // resolução — medido, **4 das 20 abaixo** reprovavam (62, 68, 76, 77).
    for res in 60u32..80 {
        let f = SampledField::from_mesh(&mesh, res).expect("esfera");
        let b = f.bounds();
        let cell = f.cell();
        // Atravessa cada uma das seis paredes, de dentro para fora, em passos de meia célula.
        for axis in 0..3 {
            for side in [-1.0f32, 1.0] {
                let wall = if side < 0.0 { b.min[axis] } else { b.max[axis] };
                let mut worst = f32::INFINITY;
                let mut prev = f32::NAN;
                for k in -8..=8 {
                    let mut p = [0.0f32; 3];
                    p[axis] = (k as f32 * 0.5).mul_add(cell * side, wall);
                    let v = f.at(p);
                    worst = worst.min(v);
                    if prev.is_finite() {
                        // ⚠️ **Continuidade**: um salto maior que uma célula entre amostras a meia
                        // célula de distância é uma parede a nascer do nada.
                        assert!(
                            (v - prev).abs() < cell,
                            "res {res}, eixo {axis} lado {side}: salto de {:.4} entre amostras a \
                             {:.4} — uma descontinuidade que passa perto de zero é uma superfície",
                            (v - prev).abs(),
                            cell * 0.5
                        );
                    }
                    prev = v;
                }
                assert!(
                    worst > cell,
                    "res {res}, eixo {axis} lado {side}: o campo chegou a {worst:.4} na travessia \
                     da parede — a caixa está a virar sólido"
                );
            }
        }
    }
}

/// Uma malha vazia não é um campo, e um campo vazio leria como **espaço cheio**.
#[test]
fn an_empty_mesh_is_not_a_field() {
    assert!(SampledField::from_mesh(&Mesh::default(), 64).is_none());
}

/// ⚠️ **A sonda do custo** — a tabela que escolhe a [`DEFAULT_RESOLUTION`].
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_sampling_cost() {
    println!("triângulos | res | células |     MB |      ms | célula");
    for tris in [2_000usize, 50_000] {
        let mesh = ph2d_mesh::shapes::sphere_with_triangles(tris, 0.6);
        for res in [64u32, 96, 128, 192, 256] {
            let t0 = std::time::Instant::now();
            let f = SampledField::from_mesh(&mesh, res).expect("malha");
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            println!(
                "{:10} | {res:3} | {:8} | {:6.1} | {ms:7.1} | {:.5}",
                mesh.faces().len(),
                f.cell_count(),
                f.cell_count() as f64 * 4.0 / (1024.0 * 1024.0),
                f.cell()
            );
        }
    }
}

/// ⚠️ **A sonda que escreve a [`CHAMFER_SAFETY`]** — quanto o chanfro superestima a reta.
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_chamfer_overshoot() {
    let (mesh, oracle) = sphere(0.6);
    println!("res | célula  | razão máx chanfro/verdadeiro | onde");
    for res in [48u32, 96, 192] {
        let f = SampledField::from_mesh(&mesh, res).expect("esfera");
        let mut worst = 0.0f32;
        let mut at = [0.0f32; 3];
        for p in walk(&f, 40_000, 0.85) {
            let truth = oracle(p).abs();
            // Longe o bastante da casca para que o erro de célula não domine a razão.
            if truth < f.cell() * 4.0 {
                continue;
            }
            // ⚠️ Desfaz a divisão: o que se mede é o chanfro CRU contra a reta.
            let r = f.at(p).abs() * CHAMFER_SAFETY / truth;
            if r > worst {
                worst = r;
                at = p;
            }
        }
        println!(
            "{res:3} | {:.5} | {worst:28.4} | ({:.2},{:.2},{:.2})",
            f.cell(),
            at[0],
            at[1],
            at[2]
        );
    }
}
