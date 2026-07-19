//! Gates da **fatia R1** da wave `docs/Flip/10_regiao_por_curvas.md` — a fronteira do
//! preenchimento feita das CURVAS, não de um contorno vetorizado.
//!
//! Módulo irmão do `flip_fill_target_tests.rs` (teto de LOC do shell, 600), e separado por
//! ASSUNTO: lá mora o critério do `filled_shape_target` (BUGS #19), aqui a escolha de rota
//! e as quatro recusas que a protegem.

use crate::flip_fill::tests::style;
use crate::flip_fill::{fill_click, ring_area};
use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_tool_flip::{FillMode as ToolFillMode, FlipStyleSnapshot};
use ph2d_vec_scene::Xform;

/// A arte do smoke: uma GRADE **de mão** (trêmula).
///
/// ⚠️ **O tremor não é enfeite — sem ele a fixture não contém o fenômeno.** A 1ª versão
/// deste gate usava retas perfeitas alinhadas aos eixos, e o contorno VETORIZADO de um
/// quadrado assim já colapsa em 4 cantos exatos: as duas rotas dão o mesmo resultado, e a
/// mutação que **desliga a rota nova inteira** passava verde. É a arte de mão que separa
/// as duas — ali o traçado do raster produz dezenas de pontos que não estão sobre linha
/// nenhuma.
fn grid_drawing() -> FlipDrawing {
    let wob = |i: usize| ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let mut d = FlipDrawing::new();
    for (k, horiz, off) in [
        (0usize, true, 0.0f32),
        (1, true, 10.0),
        (2, false, 0.0),
        (3, false, 10.0),
    ] {
        let mut s = FlipStroke::new();
        for i in 0..24 {
            let t = -1.0 + i as f32 * (12.0 / 23.0);
            let j = wob(i + k * 37) * 0.5;
            s.push_point(Point {
                pos: if horiz {
                    Vec2::new(t, off + j)
                } else {
                    Vec2::new(off + j, t)
                },
                width: 0.4,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(s);
    }
    d
}

/// O preenchimento que o balde acabou de criar.
///
/// Qualquer traço com `fill` serve, não só o `hide_stroke`: o balde tem DUAS saídas — a
/// região nova (traço invisível) e a forma que pinta a si mesma (`filled_shape_target`
/// põe o `fill` no traço que já existia). Procurar só a primeira fazia o helper entrar em
/// pânico quando a outra rota atendia — um "não pintou" falso.
fn painted(d: &FlipDrawing) -> &FlipStroke {
    d.strokes
        .iter()
        .rev()
        .find(|s| s.fill.is_some())
        .expect("o balde criou um preenchimento")
}

/// A distância de `p` à linha mais próxima do desenho (segmentos, não vértices).
fn dist_to_art(d: &FlipDrawing, p: Vec2) -> f32 {
    let mut best = f32::MAX;
    for s in d.strokes.iter().filter(|s| !s.hide_stroke) {
        for (_, a, b) in s.segments() {
            let ab = Vec2::new(b.x - a.x, b.y - a.y);
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let t = if l2 <= 0.0 {
                0.0
            } else {
                (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
            };
            let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
            best = best.min((dx * dx + dy * dy).sqrt());
        }
    }
    best
}

/// Quantos pontos do anel **coincidem com um vértice de linha** — o discriminador.
///
/// ⚠️ A 1ª versão destes gates media *distância à linha*, e ela **não separa as rotas**:
/// o contorno vetorizado também cai sobre o eixo (é o BUGS #14), a 0,008 dele. Medido, a
/// distância máxima é 0,0000 na rota nova contra 0,0077 na velha — e qualquer limite entre
/// os dois é frágil. Já *coincidir com um vértice* dá **77 contra 0**: a rota nova É feita
/// dos vértices das linhas, a velha não tem nenhum deles. É a frase do Enio, literal.
fn on_vertex_count(art: &FlipDrawing, ring: &[Vec2]) -> usize {
    ring.iter()
        .filter(|p| {
            art.strokes.iter().any(|s| {
                s.positions()
                    .iter()
                    .any(|v| ((p.x - v.x).powi(2) + (p.y - v.y).powi(2)).sqrt() < 1e-4)
            })
        })
        .count()
}

/// **A malha do fill se apoia nos vértices das LINHAS** — o smoke do Enio, virado gate.
///
/// > *"a malha que o fill cria não usa os vertex das linhas para gerar a própria malha.
/// > Não se apoia no centro da linha onde os vertex estão. Diferente do Draw:Filled."*
///
/// O oráculo é o da wave: **todo ponto do anel está sobre uma linha**. O contorno
/// vetorizado não passa nisso — ele desliza entre os vértices trêmulos.
#[test]
fn the_fill_of_a_grid_cell_rides_the_lines_themselves() {
    let mut d = grid_drawing();
    let st = FlipStyleSnapshot {
        grow: 0.0,
        ..style(ToolFillMode::Paint)
    };
    fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");

    let art = grid_drawing(); // as linhas, sem o preenchimento
    let f = painted(&d);
    let on_vertex = on_vertex_count(&art, f.positions());
    assert!(
        on_vertex >= 60,
        "só {on_vertex} dos {} pontos do anel estão EM CIMA de um vértice de linha — \
         a malha do fill não se apoia nas linhas (a rota vetorizada dá ZERO; a das curvas, 77)",
        f.positions().len()
    );
    assert!(
        (ring_area(f.positions()).abs() - 100.0).abs() < 12.0,
        "a célula trêmula deveria ter ~100 de área; veio {}",
        ring_area(f.positions()).abs()
    );
}

/// **E a dilatação some, sozinha: nesta rota o balde É o Draw:Filled.**
///
/// A largura do contorno do fill é `2s`, onde `s` é o desvio até o eixo (a lei inteira —
/// ver o `dilate.rs`). Nesta rota o contorno **É** o eixo, feito dos próprios vértices das
/// linhas, então `s = 0` por construção e a dilatação vira exatamente **zero**: a cor
/// termina no eixo, que é onde o Draw:Filled a termina. Sem margem, sem compensação, sem
/// constante para calibrar. É a fatia R2 do plano acontecendo de graça.
///
/// ⚠️ Este gate afirmava `w = 0,4` (a espessura da linha) até 2026-07-18, quando a
/// medição contra o Draw:Filled condenou o termo `w`. A afirmação de hoje é mais forte, e
/// é a que fecha a wave: **nesta rota não há nada a dilatar.**
#[test]
fn on_the_curve_route_there_is_nothing_left_to_dilate() {
    let mut d = grid_drawing();
    let st = FlipStyleSnapshot {
        grow: 0.0,
        ..style(ToolFillMode::Paint)
    };
    fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");

    let f = painted(&d);
    for (i, w) in f.widths().iter().enumerate() {
        assert!(
            w.abs() < 0.02,
            "ponto {i}: a dilatação é {w}, e nesta rota ela tem de ser ZERO — o anel já \
             está sobre o eixo (0,4 = o termo `w` de volta, e a franja com ele)"
        );
    }
}

/// **Com um VÃO, a rota nova recusa e o raster assume** — a divisão de trabalho do §3.
///
/// Sem esta recusa o arranjo devolveria a face EXTERNA e a cor vazaria para a tela toda.
#[test]
fn a_gap_falls_back_to_the_raster_route() {
    let mut d = grid_drawing();
    let n = d.strokes[3].len();
    d.strokes[3].positions_mut()[n - 1] = Vec2::new(10.0, 4.0);

    let st = style(ToolFillMode::Paint);
    if fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).is_ok() {
        let a = ring_area(painted(&d).positions()).abs();
        assert!(
            a < 400.0,
            "a região saiu com área {a} — isso é a face EXTERNA, e ela pintaria a tela toda"
        );
    }
}

/// **O GROW continua funcionando** — a rota nova recusa quando ele está armado.
///
/// O Grow é um deslocamento A PARTIR DO EIXO; esta rota põe a fronteira exatamente NO
/// eixo. Não dá para honrar os dois, e aceitar seria **ignorar o slider em silêncio** —
/// que é precisamente o que acontecia até este gate existir (o `style` de teste já vinha
/// com `grow: 2.0` e a rota disparava assim mesmo).
#[test]
fn a_grow_setting_is_not_silently_thrown_away() {
    let route_used = |grow: f64| {
        let mut d = grid_drawing();
        let st = FlipStyleSnapshot {
            grow,
            ..style(ToolFillMode::Paint)
        };
        fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
        let art = grid_drawing();
        on_vertex_count(&art, painted(&d).positions())
    };
    assert!(
        route_used(0.0) >= 60,
        "sem Grow, a rota das curvas tem de disparar"
    );
    // ⚠️ Grow PEQUENO é o caso que morde: com Grow grande o abraço já recusa por
    // distância, mas com 1 ou 2 o deslocamento cabe na tolerância e o slider era
    // engolido em SILÊNCIO.
    for grow in [1.0, 2.0] {
        assert_eq!(
            route_used(grow),
            0,
            "com Grow {grow} a rota das curvas ainda disparou — e ela põe a fronteira \
             exatamente no eixo, logo o slider foi jogado fora sem avisar"
        );
    }
}

/// Régua: o que separa de fato as duas rotas.
#[test]
#[ignore = "régua — rode com --ignored --nocapture"]
fn measure_what_separates_the_two_routes() {
    for grow in [0.0f64, 1.0, 2.0, 4.0, 8.0] {
        let mut d = grid_drawing();
        let st = FlipStyleSnapshot {
            grow,
            ..style(ToolFillMode::Paint)
        };
        fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).expect("preenche");
        let art = grid_drawing();
        let f = painted(&d);
        let pts = f.positions();
        // quantos pontos do anel COINCIDEM com um vértice de linha
        let mut on_vertex = 0;
        for p in pts {
            let mut best = f32::MAX;
            for s in art.strokes.iter() {
                for v in s.positions() {
                    best = best.min(((p.x - v.x).powi(2) + (p.y - v.y).powi(2)).sqrt());
                }
            }
            if best < 1e-4 {
                on_vertex += 1;
            }
        }
        let mut worst = 0.0f32;
        for p in pts {
            worst = worst.max(dist_to_art(&art, *p));
        }
        println!(
            "grow {grow:>4} | {:>3} pontos | {on_vertex:>3} EM CIMA de vértice | dist máx {worst:.4} | área {:.2}",
            pts.len(),
            ring_area(pts).abs()
        );
    }
}

/// **O TRAP também não é engolido** — o abraço é quem o defende.
///
/// Duas câmaras ligadas por um pescoço estreito. Com o Trap armado, a bola não passa e o
/// **raster** confina a cor a uma câmara; o **arranjo**, que só enxerga topologia, vê as
/// duas como uma face só (o pescoço está aberto). As duas respostas são de regiões
/// DIFERENTES, e entregar a do arranjo pintaria a câmara que o usuário não pediu.
///
/// É o irmão do gate do Grow, e a razão de o abraço existir: sem ele, a fatia C1 inteira
/// (o trapped-ball) viraria decoração no dia em que a rota das curvas disparasse.
#[test]
fn the_trap_is_not_swallowed_by_the_curve_route() {
    let mut d = FlipDrawing::new();
    let line = |pts: &[(f32, f32)]| {
        let mut s = FlipStroke::new();
        for &(x, y) in pts {
            s.push_point(Point {
                pos: Vec2::new(x, y),
                width: 0.3,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s
    };
    // Caixa 20x10, com dois esporões que deixam um pescoço de ~1 unidade no meio.
    d.strokes.push(line(&[
        (0.0, 0.0),
        (20.0, 0.0),
        (20.0, 10.0),
        (0.0, 10.0),
        (0.0, 0.0),
    ]));
    d.strokes.push(line(&[(10.0, 0.0), (10.0, 4.5)]));
    d.strokes.push(line(&[(10.0, 10.0), (10.0, 5.5)]));

    // ⚠️ **Os números têm de ser os do PRODUTO.** O `trap` do painel vira raio em px de
    // BUFFER (`trap × precision`), e a grade aqui tem 100 px de buffer por unidade de
    // documento — então o pescoço de 1 unidade mede **100 px de buffer**. A 1ª versão
    // deste gate usou `trap: 2.0` "porque 2 > 1", a bola passou folgada e o gate acusou um
    // bug que não existia. É a lição do `feedback_test_with_product_numbers_not_convenient_ones`.
    let st = FlipStyleSnapshot {
        grow: 0.0,
        trap: 60.0, // 60 px de buffer > metade do pescoço (50)
        precision: 1.0,
        ..style(ToolFillMode::Paint)
    };
    let ok = fill_click(&mut d, &st, Vec2::new(5.0, 5.0), 0.01, &Xform::IDENTITY).is_ok();
    if ok {
        let a = ring_area(painted(&d).positions()).abs();
        assert!(
            a < 140.0,
            "a região saiu com área {a} — a caixa inteira tem 200, e o Trap deveria ter \
             confinado a cor a uma câmara (~100). A rota das curvas ignorou o slider."
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
/// **A tolerância do abraço segue a resolução ENTREGUE — senão a rota morre no zoom alto.**
///
/// O gate que faltava no smoke de 2026-07-18 (*"nenhuma melhoria e nenhuma mudança"*).
///
/// O `hug_tol` saía da precisão **pedida** (`style.precision / doc_per_px`) e era comparado
/// com um erro nascido da **entregue** (`Grid::new` capa o `scale` para caber no
/// `MAX_SIDE`). Enquanto a grade não satura os dois números coincidem e tudo funciona; a
/// partir dali a tolerância continua encolhendo contra um erro que ficou parado, e a rota
/// do arranjo **passa a recusar sempre, em silêncio**. Medido: acima de ~3200 px de arte
/// na tela com o default, e já em 1023 px com Precision 4,0.
///
/// A fixture tem de conter o fenômeno: um `doc_per_px` pequeno o bastante para a grade
/// SATURAR. Com a arte de ~12 unidades da grade e `MAX_SIDE = 4096`, basta pedir uma
/// precisão acima de ~340 px de buffer por unidade.
///
/// Mutação que sangra: voltar a dividir pela precisão pedida — a rota recusa e o anel
/// deixa de se apoiar nos vértices.
#[test]
fn the_hug_tolerance_follows_the_resolution_the_grid_delivered() {
    let art = grid_drawing();
    let st = FlipStyleSnapshot {
        grow: 0.0,
        ..style(ToolFillMode::Paint)
    };

    // `doc_per_px` minúsculo = câmera muito aproximada = precisão pedida altíssima.
    // (0,001 pede ~1600 px de buffer por unidade; a grade entrega ~340.)
    for doc_per_px in [0.01f32, 0.003, 0.001] {
        let mut d = art.clone();
        fill_click(
            &mut d,
            &st,
            Vec2::new(5.0, 5.0),
            doc_per_px,
            &Xform::IDENTITY,
        )
        .expect("preenche");
        let f = painted(&d);
        let on_vertex = on_vertex_count(&art, f.positions());
        assert!(
            on_vertex >= 60,
            "com doc_per_px={doc_per_px} a grade satura e so {on_vertex} pontos do anel \
             pousam num vertice de linha — a rota do arranjo se recusou EM SILENCIO \
             porque a tolerancia veio da precisao PEDIDA e o erro, da ENTREGUE"
        );
    }
}

/// A grade com uma FORMA SOLTA dentro da célula do meio — o donut.
fn grid_with_island() -> FlipDrawing {
    let mut d = grid_drawing();
    let mut island = FlipStroke::new();
    for i in 0..16 {
        let t = i as f32 / 16.0 * std::f32::consts::TAU;
        island.push_point(Point {
            pos: Vec2::new(5.0 + 2.0 * t.cos(), 5.0 + 2.0 * t.sin()),
            width: 0.4,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    island.closed = true;
    d.strokes.push(island);
    d
}

/// **O DONUT: a rota das curvas carrega o buraco.**
///
/// Era a recusa 1 (*"tem buraco? o raster que resolva"*), e ela existia porque o motor do
/// arranjo só devolvia o anel externo. Um buraco não é uma face que faltou achar: é uma
/// **componente conexa** que a caminhada de half-edge nunca vê, porque ela só anda por
/// arestas e componentes distintas não compartilham nenhuma.
///
/// O gate afirma as duas metades: a região SAI com um buraco, e o anel dele também se
/// apoia nos vértices das linhas (senão teríamos curado o anel externo e deixado o buraco
/// vetorizado — a mesma dessincronização, um nível abaixo).
#[test]
fn a_region_with_an_island_keeps_the_hole_and_rides_the_lines() {
    let art = grid_with_island();
    let mut d = art.clone();
    let st = FlipStyleSnapshot {
        grow: 0.0,
        ..style(ToolFillMode::Paint)
    };
    fill_click(&mut d, &st, Vec2::new(2.0, 2.0), 0.01, &Xform::IDENTITY).expect("preenche");

    let f = painted(&d);
    assert_eq!(
        f.holes.len(),
        1,
        "a celula com uma ilha dentro tem UM buraco; veio {} — sem ele a cor pinta por \
         cima da forma solta",
        f.holes.len()
    );
    let on_vertex = on_vertex_count(&art, &f.holes[0]);
    assert!(
        on_vertex >= 12,
        "so {on_vertex} dos {} pontos do BURACO pousam num vertice de linha — o anel \
         externo veio das curvas e o buraco ficou vetorizado",
        f.holes[0].len()
    );
}
