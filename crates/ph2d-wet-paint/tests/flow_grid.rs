//! **A grade de FLUXO em ação** (plano 30, fases F2..F5).
//!
//! Os gates aqui respondem as três perguntas que separam *"a wave funciona"* de
//! *"a wave compila"*:
//!
//! 1. **`rf = 1` é o motor de hoje AO BYTE** — a rede de segurança. Sem ela
//!    nenhuma das outras afirmações é falsificável.
//! 2. **A água CORRE** com o fluxo grosso — um escorrido chega tão longe quanto
//!    chegava. É a metade que um refactor silenciosamente mata.
//! 3. **O pigmento fica FINO** — a razão de existir da wave. O contra-exemplo é
//!    o `Grid Size` que já shipou: ele também barateia o fluxo, e granula a
//!    borda fazendo isso.

mod util;

use ph2d_wet_paint::flow;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use util::{drive_stroke, sweep_nan};

const W: usize = 512;
const H: usize = 512;

/// Um traço horizontal molhado, com a gravidade que o chamador escolher.
///
/// ⚠️ **`8` px por frame, e o número é da FIXTURE, não do gosto:** a 40 px por
/// frame o depósito se amontoa no fim do traço (medido: 44% da tinta numa faixa
/// e 0% na primeira), porque a janela do trail do motor tem 123 células — o
/// item aberto `TRAIL_HALF` do doc 21. Uma fixture assim mede a janela do
/// trail, não a grade de fluxo.
fn scene(rf: usize, grav: f64) -> Engine {
    let mut e = Engine::with_flow_ratio(W, H, rf);
    e.sliders.water = 1.0;
    e.sliders.size = 0.8;
    e.sim.gravity_override = Some([0.0, grav]);
    drive_stroke(&mut e, 180.0, 80.0, 320.0, 80.0, 8.0, 0);
    for _ in 0..60 {
        e.step_simulation();
    }
    e
}

/// O CENTROIDE do filme em `y`, pesado pela massa — *onde a água está*.
fn film_centroid_y(g: &Grid) -> f64 {
    let (mut m, mut my) = (0.0f64, 0.0f64);
    for y in 1..=g.h as i32 {
        for x in 1..=g.w as i32 {
            let v = f64::from(g.film[x as usize + y as usize * g.s]);
            m += v;
            my += v * f64::from(y);
        }
    }
    if m > 0.0 { my / m } else { 0.0 }
}

/// **Quanto a GRAVIDADE levou a água para baixo** — o deslocamento do centroide
/// com gravidade menos o mesmo sem ela.
///
/// ⚠️ O alcance ABSOLUTO não serve de oráculo: a folha seca enquanto a água
/// corre, então a região molhada ENCOLHE mesmo com o campo de fluxo perfeito
/// (medido, sem gravidade: 106 → 81 em 60 passos). O que só existe se o campo
/// estiver vivo é a DIFERENÇA, e é ela que um refactor mata em silêncio.
///
/// ⚠️ **E a diferença é medida na MASSA, não na célula mais extrema acima de um
/// limiar — a primeira versão deste gate usava o [`wet_reach`] e passava por
/// SORTE.** A frente é uma estatística de UM valor extremo, e ela é caótica na
/// razão de fluxo: varrendo `rf` 1..8 o mesmo motor devolvia **27, 23, 36, 18,
/// 10, 14, 21** — 3,6× de amplitude *dentro do mesmo modelo*, com o rf=3 acima
/// do controle. Pelo centroide a mesma varredura é lisa (**20,1 · 12,0 · 22,3 ·
/// 13,5 · 13,7 · 14,3**) e a queda em `rf = 2` aparece **igual nos dois
/// modelos** (0,60 no Gauss-Seidel · 0,64 no independente de ordem) — isto é,
/// ela é da GRADE DE FLUXO, não do solver. A frente amplificava um 0,6
/// compartilhado em 0,85 contra 0,43, e foi assim que o gate reprovou uma
/// mudança de modelo por um motivo que não era o dela.
fn gravity_carry(rf: usize) -> f64 {
    film_centroid_y(scene(rf, 2.0).active_grid()) - film_centroid_y(scene(rf, 0.0).active_grid())
}

#[test]
fn the_flow_grid_is_smaller_and_the_pigment_grid_is_not() {
    // O que a wave É, em uma asserção: o pigmento NÃO encolhe.
    for rf in [1usize, 2, 4, 8] {
        let e = Engine::with_flow_ratio(W, H, rf);
        let g = e.active_grid();
        assert_eq!(
            (g.w, g.h),
            (W, H),
            "a grade FINA nao pode encolher (rf {rf})"
        );
        assert_eq!(g.film.len(), (W + 2) * (H + 2), "o film e fino (rf {rf})");
        assert_eq!(g.susp.len(), (W + 2) * (H + 2), "o susp e fino (rf {rf})");
        let (fw, fh) = flow::flow_dims(W, H, rf);
        assert_eq!((g.flow.w, g.flow.h), (fw, fh), "a grade de FLUXO (rf {rf})");
        assert_eq!(g.vel_x.len(), g.flow.cells, "vel mora no fluxo (rf {rf})");
        assert_eq!(g.flow_x.len(), g.flow.cells, "flow mora no fluxo (rf {rf})");
        // E o que ela COMPRA: `rf²` menos células de velocidade.
        if rf > 1 {
            let ratio = (W + 2) * (H + 2) / g.flow.cells;
            assert!(
                ratio >= rf * rf / 2,
                "a grade de fluxo mal encolheu: rf {rf} -> {ratio}x"
            );
        }
    }
}

#[test]
fn the_water_still_runs_when_the_flow_is_coarse() {
    // ⚠️ O gate que um refactor mata em silêncio: a água pode ficar EXATAMENTE
    // onde foi pintada e todos os outros gates passam. O quanto a gravidade a
    // carrega é a grandeza que só existe se o campo de fluxo estiver vivo.
    let base = gravity_carry(1);
    assert!(
        base > 10.0,
        "o CONTROLE nao carregou a agua ({base:.2} celulas) — a fixture nao contem o fenomeno"
    );
    for rf in [2usize, 4] {
        let e = scene(rf, 2.0);
        sweep_nan(e.active_grid(), &format!("rf {rf}"));
        let carry = gravity_carry(rf);
        // A física muda (é outra discretização), mas a ORDEM DE GRANDEZA não
        // pode: metade do transporte já seria "a água parou".
        assert!(
            carry > base * 0.5,
            "a agua parou de correr com o fluxo grosso: rf {rf} carregou {carry:.2}, \
             o controle {base:.2}"
        );
    }
}

#[test]
fn the_paint_lands_where_the_brush_went_at_every_ratio() {
    // A tinta tem de estar SOB o traço em qualquer razão — se as portas
    // discordarem sobre o bloco, o campo sai deslocado e a poça anda de lado.
    for rf in [1usize, 2, 4, 8] {
        let e = scene(rf, 2.0);
        let g = e.active_grid();
        let mut mass = 0.0f64;
        let mut cx = 0.0f64;
        for y in 1..=g.h as i32 {
            for x in 1..=g.w as i32 {
                let m = f64::from(g.susp[x as usize + y as usize * g.s])
                    + f64::from(g.sett[x as usize + y as usize * g.s]);
                mass += m;
                cx += m * f64::from(x);
            }
        }
        assert!(mass > 0.0, "nada foi pintado (rf {rf})");
        let cx = cx / mass;
        // O traço vai de x=180 a x=320 ⇒ o centro de massa em x fica no meio.
        assert!(
            (cx - 250.0).abs() < 30.0,
            "a tinta saiu do lugar: rf {rf} centro em x = {cx:.1} (esperado ~250)"
        );
    }
}

/// Quantas colunas da faixa central do traço têm tinta, e em quantos PEDAÇOS
/// contíguos elas caem.
fn coverage(e: &Engine) -> (usize, usize) {
    let g = e.active_grid();
    let (mut painted, mut runs, mut prev) = (0usize, 0usize, false);
    for x in 190..=310usize {
        let i = x + 80 * g.s;
        let on = g.susp[i] > 0.0001 || g.sett[i] > 0.0001;
        if on {
            painted += 1;
            if !prev {
                runs += 1;
            }
        }
        prev = on;
    }
    (painted, runs)
}

#[test]
fn a_coarse_flow_still_reaches_every_fine_cell_of_the_stroke() {
    // ⚠️ O modo de falha que a AMOSTRA pode ter e a média não teria: se o
    // pigmento só se movesse nas células-probe, o traço sairia PICOTADO com o
    // período do bloco. O `advect` amostra o fluxo BILINEARMENTE justamente
    // para isso, e este gate é o que prova que ele o faz.
    //
    // ⚠️ **O oráculo é o CONTROLE, nunca uma contagem absoluta**, e o primeiro
    // corte deste gate errou exatamente aí: ele exigia 110 de 121 colunas — e a
    // razão 1 pinta **100**, com um vão de 17 células. O vão é a estrutura de
    // CERDAS do pincel, não a grade; uma barra absoluta media o pincel.
    let (base_painted, base_runs) = coverage(&scene(1, 2.0));
    for rf in [2usize, 4] {
        let (painted, runs) = coverage(&scene(rf, 2.0));
        assert!(
            painted + 10 >= base_painted,
            "o traco perdeu cobertura na grade grossa: rf {rf} pintou {painted}, o controle {base_painted}"
        );
        assert!(
            runs <= base_runs + 2,
            "o traco saiu picotado com o periodo do bloco: rf {rf} em {runs} pedacos, o controle em {base_runs}"
        );
    }
}

#[test]
fn the_blow_tool_does_not_get_rf_squared_stronger() {
    // O sopro soma um impulso por PIXEL do carimbo; sem a lei da posse, `rf²`
    // pixels somariam no mesmo alvo e o vento sairia 16× forte a rf=4.
    let mut speeds = Vec::new();
    for rf in [1usize, 4] {
        let mut e = Engine::with_flow_ratio(W, H, rf);
        e.sliders.water = 1.0;
        drive_stroke(&mut e, 200.0, 200.0, 300.0, 200.0, 40.0, 4);
        e.tool = ph2d_wet_paint::painter::Tool::Blow;
        drive_stroke(&mut e, 240.0, 180.0, 260.0, 180.0, 10.0, 2);
        let g = e.active_grid();
        let peak = g
            .vel_x
            .iter()
            .chain(g.vel_y.iter())
            .fold(0.0f32, |a, v| a.max(v.abs()));
        speeds.push(f64::from(peak));
    }
    assert!(
        speeds[1] <= speeds[0] * 4.0 + 0.05,
        "o sopro ficou desproporcional na grade grossa: rf1 {:.4} -> rf4 {:.4}",
        speeds[0],
        speeds[1]
    );
}
