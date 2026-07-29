//! **A rota paralela É a rota serial** — os gates de identidade da exceção de
//! `rayon` (ADR-0145).
//!
//! Cada um dos três passes row-paralelos é rodado DUAS vezes sobre o MESMO
//! estado, uma por rota, e todo plano que ele escreve é comparado **byte a
//! byte**. Não há tolerância: o corpo de cada linha é uma função só, então a
//! única diferença possível entre as rotas é *qual thread avaliou qual linha*,
//! e isso não pode mudar o que a linha responde.
//!
//! ⚠️ **O gate que impede o verde-sobre-nada:** se a fixture ficasse abaixo dos
//! pisos medidos as duas rotas seriam *a serial contra ela mesma* — verde
//! sobre uma paralelização que nunca rodou, exactamente a armadilha que o
//! `plane_copy` do Painter pagou. Daí o
//! [`the_fixture_actually_crosses_the_parallel_threshold`], que afirma a
//! premissa dos outros três em vez de assumi-la.

mod util;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::solver;
use util::drive_stroke;

const SIDE: usize = 1200;

/// Uma poça com água VIVA, velocidade e pigmento — o estado em que os três
/// passes têm algo a fazer. Duas chamadas produzem grids idênticos (o motor é
/// determinístico ponta a ponta), e os gates AFIRMAM isso antes de comparar
/// qualquer coisa.
fn puddle() -> Engine {
    let mut e = Engine::new(SIDE, SIDE);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    // Duas faixas cruzadas: dá faixa viva larga em muitas linhas (é a área que
    // decide a rota) e uma frente molhada em todas as direções.
    drive_stroke(&mut e, 200.0, 380.0, 1000.0, 420.0, 24.0, 2);
    drive_stroke(&mut e, 560.0, 120.0, 640.0, 1080.0, 24.0, 2);
    e
}

fn params(e: &Engine) -> Params {
    e.sim.gather_params(&e.tuning)
}

/// Toda diferença entre dois grids, nomeada. Vazio = byte-idênticos.
fn diff(a: &Grid, b: &Grid) -> Vec<String> {
    let mut out = Vec::new();
    let mut f32s = |name: &str, x: &[f32], y: &[f32]| {
        if x.len() != y.len() {
            out.push(format!("{name}: comprimentos {} vs {}", x.len(), y.len()));
            return;
        }
        let n = x
            .iter()
            .zip(y.iter())
            .filter(|(p, q)| p.to_bits() != q.to_bits())
            .count();
        if n > 0 {
            let (i, p, q) = x
                .iter()
                .zip(y.iter())
                .enumerate()
                .find(|(_, (p, q))| p.to_bits() != q.to_bits())
                .map(|(i, (p, q))| (i, *p, *q))
                .unwrap();
            out.push(format!(
                "{name}: {n} celulas diferem (1a em {i}: {p} vs {q})"
            ));
        }
    };
    f32s("film", &a.film, &b.film);
    f32s("susp", &a.susp, &b.susp);
    f32s("sett", &a.sett, &b.sett);
    f32s("vel_x", &a.vel_x, &b.vel_x);
    f32s("vel_y", &a.vel_y, &b.vel_y);
    f32s("flow_x", &a.flow_x, &b.flow_x);
    f32s("flow_y", &a.flow_y, &b.flow_y);
    if a.active != b.active {
        let n = a
            .active
            .iter()
            .zip(b.active.iter())
            .filter(|(p, q)| p != q)
            .count();
        out.push(format!("active: {n} celulas diferem"));
    }
    if a.wet != b.wet {
        out.push("wet: difere".into());
    }
    if a.row_lo != b.row_lo {
        out.push("row_lo: difere".into());
    }
    if a.row_hi != b.row_hi {
        out.push("row_hi: difere".into());
    }
    if a.live_lo != b.live_lo {
        out.push("live_lo: difere".into());
    }
    if a.live_hi != b.live_hi {
        out.push("live_hi: difere".into());
    }
    let bbox_a = (a.bx0, a.by0, a.bx1, a.by1, a.has_fluid);
    let bbox_b = (b.bx0, b.by0, b.bx1, b.by1, b.has_fluid);
    if bbox_a != bbox_b {
        out.push(format!("bbox: {bbox_a:?} vs {bbox_b:?}"));
    }
    out
}

/// Monta duas poças, confere que nascem IDÊNTICAS, roda `f` com cada rota e
/// devolve a lista de divergências.
fn both_routes(f: impl Fn(&mut Grid, &Params, Rows)) -> Vec<String> {
    let mut a = puddle();
    let mut b = puddle();
    let pa = params(&a);
    let pb = params(&b);
    let pre = diff(a.active_grid(), b.active_grid());
    assert!(
        pre.is_empty(),
        "as duas poças já nascem diferentes — a fixture não é determinística, \
         e comparar depois do passe não provaria nada: {pre:?}"
    );
    // ⚠️ **A fixture é construída pela porta do PRODUTO** (`drive_stroke` chama
    // `step_simulation`), então uma rota paralela quebrada pode ENVENENAR a poça
    // em vez de aparecer na comparação: uma mutação que fazia o `reduce` devolver
    // a identidade deixava o rebuild chamar `empty_bbox`, as DUAS poças saíam sem
    // água, e comparar dois grids vazios era verde. A precondição é o que torna a
    // comparação não-vazia.
    assert!(
        a.active_grid().has_fluid,
        "a fixture perdeu a água ao ser construída — o passe não tem o que fazer, \
         e a identidade entre as rotas seria vácuo"
    );
    f(a.active_grid_mut(), &pa, Rows::Serial);
    f(b.active_grid_mut(), &pb, Rows::Parallel);
    diff(a.active_grid(), b.active_grid())
}

#[test]
fn the_fixture_actually_crosses_the_parallel_threshold() {
    let e = puddle();
    let g = e.active_grid();
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    assert!(
        g.has_fluid && rows > 1 && span > 1,
        "a fixture não tem água viva: has_fluid={} rows={rows} span={span}",
        g.has_fluid
    );
    // ⚠️ Os três pisos, não um: o do rebuild é 4× o dos outros (medido), então
    // uma fixture que passasse só pelo menor deixaria o gate do rebuild
    // comparando a rota serial contra ela mesma.
    for (nome, piso) in [
        ("jacobi", ph2d_wet_paint::par::MIN_CELLS_JACOBI),
        ("gather", ph2d_wet_paint::par::MIN_CELLS_GATHER),
        ("rebuild", ph2d_wet_paint::par::MIN_CELLS_REBUILD),
    ] {
        assert_eq!(
            Rows::pick(rows, span, piso),
            Rows::Parallel,
            "a janela da fixture ({rows}x{span} = {} células) fica abaixo do piso \
             do {nome} ({piso}), então o gate de identidade dele compararia a rota \
             serial contra ela mesma",
            rows * span
        );
    }
}

#[test]
fn the_parallel_projection_is_the_serial_projection_to_the_byte() {
    let d = both_routes(solver::project_rows);
    assert!(d.is_empty(), "o Jacobi divergiu entre as rotas: {d:?}");
}

#[test]
fn the_parallel_velocity_smoothing_is_the_serial_one_to_the_byte() {
    let d = both_routes(solver::smooth_velocity_rows);
    assert!(d.is_empty(), "o gather divergiu entre as rotas: {d:?}");
}

#[test]
fn the_parallel_active_rebuild_is_the_serial_rebuild_to_the_byte() {
    let d = both_routes(|g, _p, mode| solver::rebuild_active_region_rows(g, mode));
    assert!(
        d.is_empty(),
        "o rebuild divergiu entre as rotas: {d:?} — a saia é sequencial de \
         propósito, então uma divergência aqui é da limpeza, do scan ou do passe 1"
    );
}

/// **E ela é mais RÁPIDA** — o único modo de falha que a identidade não vê.
///
/// Os três gates acima ficariam verdes se a "rota paralela" fosse um alias da
/// serial: eles comparam RESULTADOS, e os resultados são iguais por desenho. O
/// que só um relógio distingue precisa de um gate de relógio.
///
/// ⚠️ **Redutor = MÍNIMO, e aqui isso é o certo:** toda amostra faz o mesmo
/// trabalho (o estado é restaurado antes de cada uma), então a variação é só
/// carga de máquina, e carga de máquina só sabe deixar mais lento. A barra é
/// 1,30× contra 4,35×/5,26×/2,30× medidos nesta janela — 77% de margem no mais
/// fraco, para o gate não se dissolver numa máquina carregada.
#[test]
#[cfg_attr(debug_assertions, ignore = "gate de relogio: precisa de --release")]
fn the_parallel_walk_is_actually_faster() {
    const REPS: usize = 7;
    const BAR: f64 = 1.30;
    let mut e = Engine::new(2048, 2048);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    drive_stroke(&mut e, 300.0, 700.0, 1750.0, 800.0, 24.0, 2);
    drive_stroke(&mut e, 900.0, 200.0, 1100.0, 1850.0, 24.0, 2);
    let p = params(&e);
    let g0 = e.active_grid();
    let (rows, span) = (
        (g0.by1 - g0.by0 + 1).max(0) as usize,
        (g0.bx1 - g0.bx0 + 1).max(0) as usize,
    );
    let cells = rows * span;
    for piso in [
        ph2d_wet_paint::par::MIN_CELLS_JACOBI,
        ph2d_wet_paint::par::MIN_CELLS_GATHER,
        ph2d_wet_paint::par::MIN_CELLS_REBUILD,
    ] {
        assert!(
            cells >= piso,
            "a janela ({cells}) fica abaixo do piso {piso} — o produto não tomaria \
             a rota paralela aqui, e o gate mediria o que ninguém roda"
        );
    }
    let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
    let mut fastest = |mode: Rows, which: usize| -> f64 {
        let mut best = f64::INFINITY;
        for _ in 0..REPS {
            let g = e.active_grid_mut();
            ph2d_wet_paint::grid::restore_grid(g, &snap);
            let t = std::time::Instant::now();
            match which {
                0 => solver::project_rows(g, &p, mode),
                1 => solver::smooth_velocity_rows(g, &p, mode),
                _ => solver::rebuild_active_region_rows(g, mode),
            }
            best = best.min(t.elapsed().as_secs_f64() * 1e3);
        }
        best
    };
    for (which, nome) in ["project", "smooth_velocity", "rebuild_active_region"]
        .into_iter()
        .enumerate()
    {
        let ser = fastest(Rows::Serial, which);
        let par = fastest(Rows::Parallel, which);
        let ratio = ser / par.max(1e-9);
        assert!(
            ratio >= BAR,
            "{nome}: serial {ser:.3} ms / paralelo {par:.3} ms = {ratio:.2}x, \
             abaixo da barra {BAR:.2}x — a rota paralela parou de comprar tempo"
        );
        println!("    {nome:<24} {ser:7.3} -> {par:7.3} ms   {ratio:5.2}x");
    }
}

/// A caminhada paralela repetida dá SEMPRE o mesmo resultado.
///
/// ⚠️ Não é redundante com os três acima: eles provam *paralelo == serial* numa
/// corrida; este prova que o agendamento não é uma entrada. Um race benigno na
/// maioria das corridas passaria naqueles e falharia aqui.
#[test]
fn the_parallel_walk_does_not_depend_on_the_scheduling() {
    for pass in 0..6 {
        let mut a = puddle();
        let mut b = puddle();
        let (pa, pb) = (params(&a), params(&b));
        for _ in 0..3 {
            solver::project_rows(a.active_grid_mut(), &pa, Rows::Parallel);
            solver::smooth_velocity_rows(a.active_grid_mut(), &pa, Rows::Parallel);
            solver::rebuild_active_region_rows(a.active_grid_mut(), Rows::Parallel);
            solver::project_rows(b.active_grid_mut(), &pb, Rows::Parallel);
            solver::smooth_velocity_rows(b.active_grid_mut(), &pb, Rows::Parallel);
            solver::rebuild_active_region_rows(b.active_grid_mut(), Rows::Parallel);
        }
        let d = diff(a.active_grid(), b.active_grid());
        assert!(d.is_empty(), "corrida {pass}: o agendamento vazou: {d:?}");
    }
}
