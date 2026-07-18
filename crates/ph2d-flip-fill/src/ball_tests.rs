//! Gates do trapped-ball.
//!
//! A fixture **contém o fenômeno**: uma caixa com um vão de largura conhecida. Um
//! quadrado fechado provaria só que o flood funciona — que já sabíamos —, e é assim que
//! um gate fica verde sobre a feature que não existe (disciplina de fixture: *um
//! fixture só prova o que ele contém*).

use super::{Grid, TrapBall};
use crate::raster::{BOUNDARY, FILLED};
use ph2d_core::Vec2;

/// Uma grade 40×40 com uma caixa (10..30) cujo lado de baixo tem um **vão de `gap`
/// unidades**, centrado. `gap = 0` = caixa fechada.
fn boxed_with_gap(gap: f32) -> Grid {
    let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(40.0, 40.0), 1.0, 0, 128);
    let (a, b) = (10.0, 30.0);
    let mid = (a + b) * 0.5;
    // Laterais e topo, inteiros.
    for (p, q) in [
        (Vec2::new(a, a), Vec2::new(a, b)),
        (Vec2::new(b, a), Vec2::new(b, b)),
        (Vec2::new(a, b), Vec2::new(b, b)),
    ] {
        g.stroke_capsule(p, q, 0.0);
    }
    // O lado de baixo, em dois pedaços, deixando o vão no meio.
    let half = gap * 0.5;
    g.stroke_capsule(Vec2::new(a, a), Vec2::new(mid - half, a), 0.0);
    g.stroke_capsule(Vec2::new(mid + half, a), Vec2::new(b, a), 0.0);
    g
}

fn interior_seed(g: &Grid) -> (usize, usize) {
    g.pixel_of(Vec2::new(20.0, 20.0)).expect("semente no miolo")
}

/// **A razão de a feature existir: a bola não passa por um vão mais estreito que ela.**
///
/// O par ausência+presença numa asserção só ([[feedback_absence_gate_needs_a_presence_sibling]]):
/// o flood comum **vaza** por este vão (o controle POSITIVO — sem ele o gate ficaria
/// verde numa caixa que nunca teve vão), e a bola **não vaza E cobre o miolo**. Só
/// "não vazou" ficaria verde com região vazia.
#[test]
fn the_ball_does_not_squeeze_through_a_gap_narrower_than_itself() {
    const GAP: f32 = 5.0;

    // Controle positivo: o flood de pixel escapa por aqui.
    let mut plain = boxed_with_gap(GAP);
    let seed = interior_seed(&plain);
    assert!(
        !plain.flood(seed, 0),
        "premissa do gate: o flood COMUM tem de vazar por um vao de {GAP} \
         (se ele segura, a fixture nao contem o fenomeno e o gate nao prova nada)"
    );

    // A bola de raio 4 não cabe no vão de 5 (precisaria de 2r = 8 ≤ 5).
    let g = boxed_with_gap(GAP);
    let ball = TrapBall::new(&g);
    let region = ball
        .region_from(interior_seed(&g), 4.0)
        .expect("a bola cabe no miolo de uma caixa de 20 de lado");

    assert!(
        !region.escaped,
        "a bola de raio 4 NAO pode atravessar um vao de {GAP}"
    );
    // ...e cobriu de fato o miolo (o irmão de PRESENÇA).
    let covered = region.mask.iter().filter(|m| **m).count();
    assert!(
        covered > 200,
        "a regiao tem de cobrir o miolo, cobriu so {covered} px \
         (uma regiao vazia tambem 'nao vaza')"
    );
}

/// **A bola não é mágica, e o gate diz isso.** Um vão mais largo que a bola vaza — é o
/// contrato honesto do controle (o artista sobe o raio), e é o que impede alguém de
/// "consertar" o trapped-ball para um clamp que segura tudo.
#[test]
fn a_gap_wider_than_the_ball_still_leaks() {
    let g = boxed_with_gap(12.0);
    let ball = TrapBall::new(&g);
    let region = ball
        .region_from(interior_seed(&g), 2.0)
        .expect("a bola de raio 2 cabe no miolo");
    assert!(
        region.escaped,
        "raio 2 (diametro 4) TEM de escapar por um vao de 12"
    );
}

/// A bola que **não cabe na semente** responde `None` — não um erro, e não uma região
/// vazia que o chamador confundiria com "nada aqui". É a informação de que o laço
/// best-first do paper precisa para baixar o raio.
#[test]
fn a_ball_too_fat_for_the_seed_says_so() {
    let g = boxed_with_gap(0.0);
    let ball = TrapBall::new(&g);
    // O miolo tem 20 de lado ⇒ raio 9 cabe, raio 40 não.
    assert!(ball.region_from(interior_seed(&g), 9.0).is_some());
    assert!(
        ball.region_from(interior_seed(&g), 40.0).is_none(),
        "uma bola maior que a regiao tem de dizer que nao cabe"
    );
}

/// **A região nunca aparece do lado de fora da tinta.**
///
/// Não é um clamp: é consequência de a região ser a união de bolas que *cabem* (uma
/// bola que cabe está inteira no papel). O gate mede o que o artista veria — cor
/// aparecendo fora da forma fechada — e não a regra interna.
#[test]
fn the_ball_never_lands_outside_the_shape() {
    let g = boxed_with_gap(3.0);
    let ball = TrapBall::new(&g);
    let region = ball.region_from(interior_seed(&g), 3.0).unwrap();
    // Um ponto claramente fora da caixa (a caixa vai de 10 a 30).
    for probe in [
        Vec2::new(2.0, 2.0),
        Vec2::new(38.0, 38.0),
        Vec2::new(20.0, 4.0), // logo abaixo do vao
    ] {
        let (x, y) = g.pixel_of(probe).unwrap();
        assert!(
            !region.mask[y * g.w + x],
            "a cor apareceu fora da forma, em {probe:?}"
        );
    }
}

/// Determinismo: a mesma bola no mesmo grid dá a MESMA região (HR-5). O flood tem uma
/// pilha, e uma ordem de visita dependente de hash daria máscaras diferentes entre
/// execuções sem nunca falhar um teste de forma.
#[test]
fn the_same_ball_gives_the_same_region() {
    let g = boxed_with_gap(4.0);
    let ball = TrapBall::new(&g);
    let a = ball.region_from(interior_seed(&g), 3.0).unwrap();
    let b = ball.region_from(interior_seed(&g), 3.0).unwrap();
    assert_eq!(a.mask, b.mask);
    assert_eq!(a.escaped, b.escaped);
}

/// **A janela da 2ª EDT é uma OTIMIZAÇÃO, não uma aproximação.**
///
/// A dilatação de volta roda só na bbox do núcleo folgada de `r`, porque um pixel mais
/// longe que `r` do núcleo está fora da região por definição. O gate compara com a
/// resposta de referência — a EDT na **grade inteira** — e exige igualdade **ao bit**.
/// Sem ele, o comentário que diz "é uma janela, não uma aproximação" é só uma promessa.
#[test]
fn windowing_the_dilation_gives_the_same_region_as_the_whole_grid() {
    for &(gap, r) in &[(0.0f32, 3.0f32), (4.0, 5.0), (7.0, 2.0), (0.0, 9.0)] {
        let g = boxed_with_gap(gap);
        let ball = TrapBall::new(&g);
        let seed = interior_seed(&g);
        let Some(got) = ball.region_from(seed, r) else {
            continue;
        };

        // Referência: a MESMA definição, sem janela nenhuma.
        let core = ball.core_mask_for_reference(seed, r).expect("nucleo");
        let full = crate::sq_distance_to_set(g.w, g.h, |i| core[i]);
        let rr = f64::from(r) * f64::from(r);
        let want: Vec<bool> = full.iter().map(|&d| f64::from(d) <= rr).collect();

        assert_eq!(
            got.mask, want,
            "a janela mudou a resposta (gap {gap}, raio {r})"
        );
    }
}

/// **A régua** (`--ignored --nocapture`): o tamanho REAL da grade e o custo do
/// trapped-ball com os números do PRODUTO, que é o que a CLAUDE.md §0.0 exige antes de
/// se escrever qualquer teto. Imprime a tabela; não afirma número nenhum (um gate de
/// wall-clock mediria o PERFIL do build, não o produto).
#[test]
#[ignore = "régua de medição — rode com --ignored --nocapture"]
fn measure_the_product_grid_and_ball_cost() {
    // Os números do produto, como o repo já os define em
    // `the_bucket_fills_at_the_real_camera_scale`: câmera default = 10 unidades de
    // mundo na altura de uma janela de 1080p, e `precision` = DEFAULT_PRECISION (1.6).
    let px_to_world = 10.0f32 / 1080.0;
    let precision_ui = 1.6f32;
    let precision = precision_ui / px_to_world; // px de buffer por unidade de doc

    println!("\n=== grade do balde com os numeros do PRODUTO ===");
    println!("px_to_world = {px_to_world:.6}  ·  precision(ui) = {precision_ui}");
    println!("=> {precision:.1} px de buffer por unidade de documento\n");
    println!(
        "{:>10} {:>10} {:>12} {:>10} {:>9} {:>9} {:>9}",
        "tela(px)", "mundo(u)", "grade", "Mpix", "EDT1(ms)", "resto(ms)", "total"
    );

    for &screen in &[512.0f32, 1080.0, 1920.0, 3840.0] {
        let world = screen * px_to_world;
        let mut g = Grid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(world, world),
            precision,
            20,
            4096,
        );
        // Uma caixa, para haver fronteira de verdade na EDT.
        let (a, b) = (world * 0.25, world * 0.75);
        for (p, q) in [
            (Vec2::new(a, a), Vec2::new(b, a)),
            (Vec2::new(b, a), Vec2::new(b, b)),
            (Vec2::new(b, b), Vec2::new(a, b)),
            (Vec2::new(a, b), Vec2::new(a, a)),
        ] {
            g.stroke_capsule(p, q, 0.0);
        }
        let mpix = (g.w * g.h) as f64 / 1.0e6;
        let seed = g.pixel_of(Vec2::new(world * 0.5, world * 0.5)).unwrap();

        // Repartido, porque "217 ms" não diz onde gastar o esforço: a 1ª EDT (o campo
        // de alcance) e a 2ª (a dilatação de volta) custam a MESMA coisa hoje, e só uma
        // delas precisa varrer a grade inteira.
        let t0 = std::time::Instant::now();
        let ball = TrapBall::new(&g);
        let edt1 = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = std::time::Instant::now();
        let _ = ball.region_from(seed, 8.0);
        let rest = t1.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{screen:>10.0} {world:>10.2} {:>5}x{:<6} {mpix:>10.2} {:>9.1} {rest:>9.1} {:>9.1}",
            g.w,
            g.h,
            edt1,
            edt1 + rest
        );
    }
    println!(
        "\n(o teto de lado e MAX_SIDE=4096; ao bater nele quem cede e a RESOLUCAO)\n\
         (ball = 2 EDTs + 1 flood, em DEBUG salvo --release)\n"
    );
    // Um uso do FILLED para o import não ficar órfão quando o gate muda de forma.
    let _ = FILLED | BOUNDARY;
}
