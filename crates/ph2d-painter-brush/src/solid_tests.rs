//! Os gates da forma sólida ([`super`]). Filho por `#[path]` — o `mod tests` fica FILHO de `solid`,
//! então `use super::*` alcança os privados (o `accumulate_edge`).

use super::*;

/// Soma de cobertura, em unidades de PIXEL (o que a área do polígono deve dar).
fn total(cov: &[u8]) -> f64 {
    cov.iter().map(|&c| f64::from(c) / 255.0).sum()
}

/// Um retângulo axis-aligned com bordas FRACIONÁRIAS.
fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Vec<[f32; 2]>> {
    vec![vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]]
}

/// **A COBERTURA TOTAL É A ÁREA DO POLÍGONO** — o oráculo global de um rasterizador exato.
///
/// ⚠️ Ele é forte justamente por ser global: qualquer erro de repartição entre células de borda
/// **sobra ou falta** na soma. Um rasterizador binário (dentro/fora) passa perto num quadrado
/// inteiro e falha aqui em toda borda fracionária — que é por isso que as três fixturas têm
/// coordenada quebrada de propósito.
#[test]
fn the_total_coverage_is_the_polygons_area() {
    // 1) Retângulo fracionário: área conhecida ao dígito.
    let cov = fill_coverage(&rect(2.25, 3.5, 11.75, 9.25), 16, 16, [0.0, 0.0]);
    let want = f64::from((11.75f32 - 2.25) * (9.25 - 3.5));
    let got = total(&cov);
    assert!(
        (got - want).abs() < 0.02,
        "retangulo: cobertura {got:.4} contra area {want:.4}"
    );

    // 2) Triângulo — a metade da borda que um retângulo não exercita: uma diagonal.
    let tri = vec![vec![[1.0f32, 1.0], [14.0, 2.0], [3.0, 13.0]]];
    let cov = fill_coverage(&tri, 16, 16, [0.0, 0.0]);
    // Sapateiro.
    let want = f64::from(((14.0f32 - 1.0) * (13.0 - 1.0) - (3.0 - 1.0) * (2.0 - 1.0)).abs() / 2.0);
    let got = total(&cov);
    assert!(
        (got - want).abs() / want < 0.01,
        "triangulo: cobertura {got:.4} contra area {want:.4}"
    );

    // 3) Um disco de 128 lados — a borda curva, que é o caso do gesto real.
    let (cx, cy, r) = (32.0f32, 32.0, 24.0);
    let disc: Vec<[f32; 2]> = (0..128)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = i as f32 / 128.0 * std::f32::consts::TAU;
            [cx + r * a.cos(), cy + r * a.sin()]
        })
        .collect();
    let cov = fill_coverage(&[disc], 64, 64, [0.0, 0.0]);
    let want = f64::from(std::f32::consts::PI * r * r) * 0.9994; // área do 128-gono inscrito
    let got = total(&cov);
    assert!(
        (got - want).abs() / want < 0.005,
        "disco: cobertura {got:.4} contra area {want:.4}"
    );
}

/// **A BORDA INCLINADA CARREGA A ÁREA EXATA** — o gate da decisão §5.2 do plano 38
/// (*"a borda pode ser sólida e específica para essa tool; faça o melhor possível"*).
///
/// ⚠️ **A fixture TEM de ser inclinada.** Numa borda paralela a um eixo qualquer rasterizador
/// acerta — inclusive um binário —, e o gate ficaria verde sobre a coisa errada. O oráculo é uma
/// referência por supersample de 256×256 por pixel, três ordens de grandeza abaixo do que ela julga.
///
/// A W0 mediu o que o composite booleano de hoje entrega no mesmo teste: `SS = 3` erra mais de
/// 8/255 em **51% dos pixels de borda**. O número deste gate é o que substitui aquele.
#[test]
fn the_slanted_edge_carries_the_exact_area() {
    const W: usize = 48;
    // Um triângulo grande e magro: bordas em três inclinações diferentes, nenhuma axis-aligned.
    let tri = vec![vec![[3.3f32, 2.7], [44.1, 11.9], [9.4, 45.2]]];
    let cov = fill_coverage(&tri, W, W, [0.0, 0.0]);

    // Referência: 256×256 amostras por pixel, ponto-em-triângulo por sinal de área.
    const SS: usize = 256;
    let inside = |px: f32, py: f32| -> bool {
        let t = &tri[0];
        let sign =
            |a: [f32; 2], b: [f32; 2]| (px - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (py - b[1]);
        let (d1, d2, d3) = (sign(t[0], t[1]), sign(t[1], t[2]), sign(t[2], t[0]));
        let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
        !(neg && pos)
    };
    let mut worst = 0.0f64;
    let mut sum = 0.0f64;
    let mut edge = 0u32;
    for y in 0..W {
        for x in 0..W {
            let mut hits = 0u32;
            for j in 0..SS {
                for i in 0..SS {
                    #[allow(clippy::cast_precision_loss)]
                    let sx = x as f32 + (i as f32 + 0.5) / SS as f32;
                    #[allow(clippy::cast_precision_loss)]
                    let sy = y as f32 + (j as f32 + 0.5) / SS as f32;
                    hits += u32::from(inside(sx, sy));
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let want = f64::from(hits) / (SS * SS) as f64;
            if want <= 0.0 || want >= 1.0 {
                continue; // só a BORDA — no miolo e fora todos concordam por construção
            }
            edge += 1;
            let got = f64::from(cov[y * W + x]) / 255.0;
            let e = (got - want).abs() * 255.0;
            sum += e;
            worst = worst.max(e);
        }
    }
    assert!(
        edge > 60,
        "a fixture nao tem borda inclinada suficiente: {edge}"
    );
    let mean = sum / f64::from(edge);
    // A quantização sozinha já vale 0,5 nível; o que sobra é o erro do modelo.
    assert!(
        worst < 2.0 && mean < 0.6,
        "borda inclinada: erro medio {mean:.3} pior {worst:.3} niveis (de 255), em {edge} px"
    );
}

/// **UM LAÇO QUE SE CRUZA FICA CHEIO** — a regra não-zero, que é o que faz um gesto solto ler como
/// mancha em vez de xadrez.
///
/// ⚠️ A fixture é um laço que **atravessa a si mesmo** (um oito com as duas alças no mesmo sentido
/// de percurso): sob par-ímpar o miolo do cruzamento sairia VAZIO.
#[test]
fn a_self_crossing_loop_is_filled_not_checkered() {
    // Dois quadrados sobrepostos percorridos no MESMO sentido, ligados: a sobreposição tem winding 2.
    let lp = vec![vec![
        [4.0f32, 4.0],
        [20.0, 4.0],
        [20.0, 20.0],
        [12.0, 20.0],
        [12.0, 28.0],
        [28.0, 28.0],
        [28.0, 12.0],
        [20.0, 12.0],
        [20.0, 4.0],
        [36.0, 4.0],
        [36.0, 36.0],
        [4.0, 36.0],
    ]];
    let cov = fill_coverage(&lp, 40, 40, [0.0, 0.0]);
    // O miolo do cruzamento (winding 2) tem de estar CHEIO.
    let at = |x: usize, y: usize| cov[y * 40 + x];
    assert_eq!(at(16, 16), 255, "o cruzamento saiu vazado (par-impar?)");
    assert_eq!(at(8, 8), 255, "o miolo simples saiu vazado");
    assert_eq!(at(2, 2), 0, "vazou para fora do laco");
}

/// **UM GESTO QUE NÃO FECHA, FECHA SOZINHO** — a decisão §5.3 do plano 38, escrita onde ela mora.
///
/// ⚠️ E a consequência fica pinada junto: uma pincelada quase RETA vira uma *sliver* de área quase
/// zero. Não é defeito, é a lei — e o gate a afirma para ninguém a "consertar" com um limiar.
#[test]
fn the_gesture_closes_itself_and_a_straight_one_is_a_sliver() {
    // Três pontos que não voltam: o triângulo implícito tem área.
    let open = vec![vec![[4.0f32, 4.0], [28.0, 8.0], [8.0, 28.0]]];
    let cov = fill_coverage(&open, 32, 32, [0.0, 0.0]);
    assert!(
        total(&cov) > 200.0,
        "o gesto aberto nao fechou: cobertura {:.1}",
        total(&cov)
    );

    // Uma pincelada quase reta: fecha, e o que sobra é uma lasca.
    let straight = vec![vec![[2.0f32, 16.0], [30.0, 16.4], [16.0, 16.2]]];
    let cov = fill_coverage(&straight, 32, 32, [0.0, 0.0]);
    assert!(
        total(&cov) < 4.0,
        "a lasca deveria ser quase nada: cobertura {:.2}",
        total(&cov)
    );
}

/// **O DEGENERADO NÃO PANICA E NÃO PINTA** — laço de um ponto, lista vazia, janela nula.
#[test]
fn the_degenerate_cases_paint_nothing() {
    assert!(fill_coverage(&[], 8, 8, [0.0, 0.0]).iter().all(|&c| c == 0));
    assert!(
        fill_coverage(&[vec![[1.0, 1.0]]], 8, 8, [0.0, 0.0])
            .iter()
            .all(|&c| c == 0)
    );
    assert!(fill_coverage(&rect(0.0, 0.0, 4.0, 4.0), 0, 8, [0.0, 0.0]).is_empty());
    // Uma aresta HORIZONTAL sozinha não cruza linha de varredura nenhuma.
    let mut acc = vec![0.0f32; 9 * 8];
    accumulate_edge([1.0, 3.0], [7.0, 3.0], 9, 8, &mut acc);
    assert!(
        acc.iter().all(|&v| v == 0.0),
        "a aresta horizontal depositou massa"
    );
}

/// **A JANELA RECORTA, E O QUE FICA DE FORA NÃO VOLTA PELA ESQUERDA.**
///
/// ⚠️ Este é o gate da coluna de folga: sem ela o depósito da célula `x1` da última coluna cai na
/// coluna 0 da linha SEGUINTE, e a figura ganha um fantasma na borda esquerda — um defeito que a
/// soma de área não vê (a massa continua lá, só no lugar errado).
#[test]
fn the_window_clips_and_nothing_wraps_to_the_left() {
    // Um retângulo que sai pela direita da janela.
    let cov = fill_coverage(&rect(6.0, 2.0, 40.0, 10.0), 16, 16, [0.0, 0.0]);
    for y in 0..16 {
        assert_eq!(cov[y * 16], 0, "vazou para a coluna 0 na linha {y}");
    }
    assert_eq!(
        cov[5 * 16 + 15],
        255,
        "a figura devia encher ate' a borda direita"
    );
    // …e um que sai pela esquerda continua cheio à direita da fronteira.
    let cov = fill_coverage(&rect(-20.0, 2.0, 10.0, 10.0), 16, 16, [0.0, 0.0]);
    assert_eq!(
        cov[5 * 16],
        255,
        "a parte visivel do que entra pela esquerda sumiu"
    );
    assert_eq!(cov[5 * 16 + 14], 0, "vazou para a direita da figura");
}

/// **A CAIXA É A DOS LAÇOS, RECORTADA À TELA** — e uma figura inteiramente fora não devolve caixa.
#[test]
fn the_bbox_is_the_loops_clipped_to_the_canvas() {
    // A figura ocupa os pixels 10..=30 e 20..=40 — vinte e um de cada lado, não vinte e dois.
    let b = loops_bbox(&rect(10.2, 20.7, 30.9, 40.1), 100, 100).unwrap();
    assert_eq!(b, [10, 20, 21, 21], "caixa {b:?}");
    assert!(loops_bbox(&rect(-50.0, -50.0, -10.0, -10.0), 100, 100).is_none());
    assert!(loops_bbox(&[], 100, 100).is_none());
    // Recorta na tela em vez de estourar.
    let b = loops_bbox(&rect(90.0, 90.0, 500.0, 500.0), 100, 100).unwrap();
    assert_eq!(b, [90, 90, 10, 10], "caixa {b:?}");
}

/// **AS DUAS ROTAS DA SOMA CORRIDA ESCREVEM O MESMO** — a rede que torna o pool de threads uma
/// escolha de agendamento em vez de uma segunda resposta (ADR-0158).
///
/// ⚠️ **Ele existe porque nenhum outro gate alcança a rota paralela:** o piso do pool é ~131 k
/// texels e toda a fixture de Solid do repo roda muito abaixo dele, então a rota rápida shipava
/// **sem um único teste** — o mesmo buraco que o irmão dela no tool já tinha.
///
/// ⚠️ **A fixture tem de conter ARESTAS QUE SE CRUZAM**, senão o `nonzero` nunca soma duas
/// contribuições na mesma célula e a identidade sai verde por vácuo.
///
/// **Mutação que sangra:** trocar o `enumerate()` da rota paralela por um índice que não seja a
/// linha (a cobertura sai deslocada).
#[test]
fn both_walkers_of_the_running_sum_write_the_same_bytes() {
    // Uma estrela de cinco pontas: ela cruza a si mesma, então há células com duas derivadas.
    let star: Vec<[f32; 2]> = (0..5)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let a = k as f32 * 4.0 * std::f32::consts::PI / 5.0;
            [64.0 + 55.0 * a.cos(), 64.0 + 55.0 * a.sin()]
        })
        .collect();
    // …e um losango que a atravessa, para haver laços sobrepostos de sentidos diferentes.
    // ⚠️ **Ele é OBLÍQUO e de coordenada quebrada de propósito**: um retângulo alinhado ao eixo em
    // coordenada inteira não tem borda anti-aliased nenhuma, e o controle desta fixture nasceu
    // VERMELHO por isso (59 texels de meio-tom) — a identidade sairia verde sobre uma imagem
    // praticamente binária.
    let square = vec![[63.7, 12.3], [115.4, 64.1], [63.7, 115.8], [12.1, 64.1]];
    let loops = vec![star, square];
    let (w, h) = (128usize, 128usize);
    let serial = super::fill_coverage_routed(&loops, w, h, [0.0, 0.0], false);
    let parallel = super::fill_coverage_routed(&loops, w, h, [0.0, 0.0], true);
    let painted = serial.iter().filter(|v| **v > 0).count();
    assert!(
        painted > 4_000,
        "a fixture nao cobriu nada ({painted} texels): o oraculo nao mede nada"
    );
    let mid = serial
        .iter()
        .filter(|v| (1..255).contains(&i32::from(**v)))
        .count();
    assert!(
        mid > 200,
        "a fixture nao tem borda anti-aliased ({mid} texels de meio-tom): a identidade sairia \
         verde sobre uma imagem binaria"
    );
    let diff = serial
        .iter()
        .zip(parallel.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diff, 0,
        "as duas rotas da soma corrida divergem em {diff} texels — o pool deixou de ser so' \
         agendamento"
    );
}

/// **O QUE A SOMA CORRIDA COBRA, E O QUE O POOL COMPRA** — as duas rotas cronometradas
/// **costas-com-costas dentro da mesma corrida**, sobre a MESMA entrada.
///
/// ⚠️ **A forma do A/B é a lição do doc 28 §5.46:** esta workstation é compartilhada e o MESMO passe
/// foi medido entre 1,01 e 1,43 ms sem uma linha de código mudar. Comparar duas corridas atribuiria
/// a deriva da máquina ao ganho; comparar duas rotas dentro de uma corrida torna a carga um **fator
/// comum**.
///
/// Rodar: `cargo test -p ph2d-painter-brush --release measure_what_the_running_sum -- --ignored
/// --nocapture --test-threads=1`
#[test]
#[ignore = "medição, não gate — rode com --test-threads=1 na máquina calma"]
fn measure_what_the_running_sum_costs_by_both_routes() {
    // A rosácea que o produto de facto entrega: 12 cópias de um arco de ~1210 pontos, e 24 com o
    // Tiling — os números que o `measure_solid_cost` mediu na porta do artista.
    let arc: Vec<[f32; 2]> = (0..1210)
        .map(|k| {
            let a = k as f32 * 0.0052;
            [512.0 + 300.0 * a.cos(), 512.0 + 300.0 * a.sin()]
        })
        .collect();
    let rosette = |copies: usize| -> Vec<Vec<[f32; 2]>> {
        (0..copies)
            .map(|c| {
                let t = c as f32 * std::f32::consts::TAU / copies as f32;
                let (s, co) = (t.sin(), t.cos());
                arc.iter()
                    .map(|p| {
                        let (x, y) = (p[0] - 512.0, p[1] - 512.0);
                        [512.0 + x * co - y * s, 512.0 + x * s + y * co]
                    })
                    .collect()
            })
            .collect()
    };
    let tiny = vec![vec![[0.0, 0.0], [3.0, 0.0], [0.0, 3.0]]];
    println!("\n=== A SOMA CORRIDA: SERIAL x PARALELA, costas-com-costas (janela 1024x1024) ===\n");
    println!(
        "{:<22} {:>8} {:>10} {:>10} {:>8}",
        "conjunto", "pontos", "serial ms", "par ms", "ganho"
    );
    for (name, loops) in [
        ("piso de AREA", tiny),
        ("rosacea 12", rosette(12)),
        ("rosacea 24 (tiling)", rosette(24)),
    ] {
        let n = 8;
        let (mut ser, mut par) = (f64::MAX, f64::MAX);
        // Alternado e por MÍNIMO: cada rota vê as mesmas janelas de contenção.
        for _ in 0..n {
            let a = std::time::Instant::now();
            std::hint::black_box(super::fill_coverage_routed(
                &loops,
                1024,
                1024,
                [0.0, 0.0],
                false,
            ));
            ser = ser.min(a.elapsed().as_secs_f64() * 1e3);
            let b = std::time::Instant::now();
            std::hint::black_box(super::fill_coverage_routed(
                &loops,
                1024,
                1024,
                [0.0, 0.0],
                true,
            ));
            par = par.min(b.elapsed().as_secs_f64() * 1e3);
        }
        let pts: usize = loops.iter().map(Vec::len).sum();
        println!(
            "{name:<22} {pts:>8} {ser:>10.3} {par:>10.3} {:>7.2}x",
            ser / par
        );
    }
}
