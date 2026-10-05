//! Os gates do **Solid e dos fios no Wet Paint** ([`super::mascara`], doc 46 item 9).

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::line_kind::LineKind;

const LADO: u32 = 128;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn agua(raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    let b = BrushSpec {
        radius_px: raio,
        color: [0.1, 0.2, 0.8],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("wetpaint");
    t
}

/// Um gesto pelos `cantos`, `por_aresta` eventos por aresta, com um quadro por evento.
fn gesto(t: &mut PainterTool, cantos: &[[f32; 2]], por_aresta: usize) {
    t.on_canvas_pointer(cp(cantos[0], PointerPhase::Down));
    for w in cantos.windows(2) {
        for k in 1..=por_aresta {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / por_aresta as f32;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ],
                PointerPhase::Move,
            ));
            t.paint_tick(1.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(cantos[cantos.len() - 1], PointerPhase::Up));
}

/// Os pixels de um quadrado de `lado` com canto em `(x0, y0)` que a tinta tingiu.
fn tingidos_em(t: &PainterTool, x0: usize, y0: usize, lado: usize) -> usize {
    (y0..y0 + lado)
        .flat_map(|y| (x0..x0 + lado).map(move |x| (y * LADO as usize + x) * 4))
        .filter(|&i| t.canvas_rgba[i] < 240)
        .count()
}

/// O pigmento (suspenso + assente) médio das células de um quadrado de pixels.
fn pigmento_em(t: &PainterTool, x0: usize, y0: usize, lado: usize) -> f64 {
    let sess = t.paint.wetpaint.session.as_ref().expect("sessão viva");
    let g = &sess.engine.layers[sess.engine.active_layer].grid;
    let r = sess.ratio;
    let mut soma = 0.0;
    let mut n = 0.0;
    for y in y0..y0 + lado {
        for x in x0..x0 + lado {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let cx = grid_map::px_to_cell(x as f64 + 0.5, r).floor() as usize;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let cy = grid_map::px_to_cell(y as f64 + 0.5, r).floor() as usize;
            let i = cx + cy * g.s;
            soma += f64::from(g.susp[i] + g.sett[i]);
            n += 1.0;
        }
    }
    soma / n
}

const LACO: [[f32; 2]; 5] = [
    [24.0, 24.0],
    [104.0, 24.0],
    [104.0, 104.0],
    [24.0, 104.0],
    [24.0, 28.0],
];

/// **O SOLID ENCHE A REGIÃO NA ÁGUA** — o miolo do quadrado fica a 30 px de todo o rastro.
#[test]
fn o_solid_enche_a_regiao_na_agua() {
    let mut linha = agua(4.0);
    gesto(&mut linha, &LACO, 10);
    let mut mancha = agua(4.0);
    mancha.paint.brush.style_solid = true;
    gesto(&mut mancha, &LACO, 10);
    assert_eq!(
        tingidos_em(&linha, 54, 54, 20),
        0,
        "controlo: sem Solid o miolo fica papel"
    );
    assert_eq!(
        tingidos_em(&mancha, 54, 54, 20),
        400,
        "o Solid não encheu o miolo na água"
    );
}

/// **A MANCHA ENCHE AO SOLTAR** (escolha do dono, 2026-10-05) — durante o gesto a água mostra só o
/// traço; o pen-up enche a região (`enche_a_mancha_na_agua`).
#[test]
fn a_mancha_enche_ao_soltar() {
    let mut t = agua(4.0);
    t.paint.brush.style_solid = true;
    t.on_canvas_pointer(cp(LACO[0], PointerPhase::Down));
    for w in LACO.windows(2) {
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 10.0;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ],
                PointerPhase::Move,
            ));
            t.paint_tick(1.0 / 60.0);
        }
    }
    assert_eq!(
        tingidos_em(&t, 54, 54, 20),
        0,
        "a mancha apareceu durante o gesto"
    );
    assert!(
        tingidos_em(&t, 60, 22, 4) > 0,
        "controlo: o traço (a meio da aresta de cima, que já pousou) está na tela a meio do gesto"
    );
    t.on_canvas_pointer(cp(LACO[4], PointerPhase::Up));
    assert_eq!(
        tingidos_em(&t, 54, 54, 20),
        400,
        "o pen-up não encheu a região"
    );
}

/// **A REGIÃO É A DO GESTO INTEIRO** — o caminho volta pelo meio, e o entalhe que as formas
/// intermédias cobririam tem de sair limpo: a mancha é a forma do pen-up, nenhuma outra.
#[test]
fn a_mancha_provisoria_nao_deixa_resto_na_agua() {
    let volta = [
        [20.0, 20.0],
        [108.0, 20.0],
        [108.0, 108.0],
        [20.0, 108.0],
        [20.0, 60.0],
        [90.0, 60.0],
    ];
    let mut t = agua(4.0);
    t.paint.brush.style_solid = true;
    gesto(&mut t, &volta, 12);
    assert!(
        tingidos_em(&t, 64, 80, 16) > 200,
        "controlo: a forma final tem de estar cheia"
    );
    assert_eq!(
        tingidos_em(&t, 27, 41, 8),
        0,
        "a mancha dos quadros intermédios ficou no entalhe"
    );
}

/// **O TRAÇO SEGUINTE NÃO DESFAZ A MANCHA** — o bracket do Solid descasca a cada lote; na água não há
/// rascunho a repor, e um descasque que repusesse alguma coisa apagaria a mancha do traço anterior.
#[test]
fn o_traco_seguinte_nao_desfaz_a_mancha() {
    let mut t = agua(4.0);
    t.paint.brush.style_solid = true;
    gesto(&mut t, &LACO, 10);
    assert_eq!(
        tingidos_em(&t, 54, 54, 20),
        400,
        "controlo: a mancha assentou"
    );
    gesto(&mut t, &[[10.0, 116.0], [118.0, 116.0]], 10);
    assert_eq!(
        tingidos_em(&t, 54, 54, 20),
        400,
        "o traço seguinte desfez a mancha assente"
    );
}

/// **OS FIOS PINTAM NA ÁGUA** — o Wire liga o carimbo de agora aos anteriores pelo percurso: a
/// teia cai entre os lados de um zigue-zague, onde o rastro não passa.
#[test]
fn os_fios_pintam_na_agua() {
    let zigue = [[20.0, 30.0], [108.0, 30.0], [20.0, 50.0], [108.0, 50.0]];
    let com = |fios: bool| {
        let mut t = agua(2.0);
        if fios {
            t.paint.brush.line_kind = LineKind::Wire;
            t.paint.brush.thread_opacity = 1.0;
        }
        gesto(&mut t, &zigue, 12);
        t.canvas_rgba.to_vec()
    };
    let (sem, com_fios) = (com(false), com(true));
    let mudam = sem
        .as_chunks::<4>()
        .0
        .iter()
        .zip(com_fios.as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    assert!(mudam > 100, "os fios não pintaram na água ({mudam} pixels)");
}

/// **A MANCHA TEM O CORPO DO TRAÇO** — o pigmento médio no miolo da região fica na ordem do miolo
/// de um traço do mesmo pincel (no espaçamento de fábrica, em três raios): a porta pousa a máscara
/// numa passada da lei do carimbo. ⚠️ O traço mede-se num gesto SEM Solid — a borda da mancha passa
/// pelo centro do rastro, e medi-lo no mesmo gesto somava metade da mancha ao número do traço.
#[test]
fn a_mancha_tem_o_corpo_do_traco() {
    for raio in [4.0f32, 8.0, 16.0] {
        let mut t = agua(raio);
        t.paint.brush.style_solid = true;
        gesto(&mut t, &LACO, 10);
        let miolo = pigmento_em(&t, 54, 54, 20);
        let mut so_traco = agua(raio);
        gesto(&mut so_traco, &LACO, 10);
        // A linha central da aresta de cima, longe dos cantos.
        let traco = (40..88)
            .step_by(8)
            .map(|x| pigmento_em(&so_traco, x, 23, 2))
            .sum::<f64>()
            / 6.0;
        let razao = miolo / traco.max(1e-9);
        eprintln!("raio {raio:>4}: mancha {miolo:>8.1} · traço {traco:>8.1} · {razao:.2}");
        assert!(
            (0.67..=1.5).contains(&razao),
            "raio {raio}: a mancha não tem o corpo do traço ({miolo:.1} contra {traco:.1}, razão \
             {razao:.2})"
        );
    }
}

/// SONDA (relógio) — o report do dono, 2026-10-05: *«ficou lento numa mancha de 1000px»*. Os três
/// momentos de um laço de 1000 px numa tela de 2048²: o gesto, o pen-up e a água depois dele.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_mancha_de_1000px -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico de relógio"]
fn diag_a_mancha_de_1000px() {
    const L: u32 = 4096;
    let pt = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let a = i as f32 / 120.0 * std::f32::consts::TAU;
        [2048.0 + 500.0 * a.cos(), 2048.0 + 500.0 * a.sin()]
    };
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg {}", carga.trim());
    for rodada in 0..3 {
        for solid in [rodada % 2 == 0, rodada % 2 != 0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
            t.paint.brush.radius_px = 20.0;
            t.paint.brush.color = [0.1, 0.2, 0.8];
            t.set_paint_tool_mode("wetpaint");
            t.paint.brush.style_solid = solid;
            t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
            let (mut pior, t0) = (0.0f64, std::time::Instant::now());
            for i in 1..=120 {
                let q = std::time::Instant::now();
                t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
                t.paint_tick(1.0 / 60.0);
                pior = pior.max(q.elapsed().as_secs_f64() * 1e3);
            }
            let gesto = t0.elapsed().as_secs_f64() * 1e3 / 120.0;
            let q = std::time::Instant::now();
            t.on_canvas_pointer(cp(pt(120), PointerPhase::Up));
            let soltar = q.elapsed().as_secs_f64() * 1e3;
            // A água depois: o PASSO da sim (síncrono — o worker esconde-o do quadro, mas a cadência
            // visual da água É a taxa de passos: acima de 25 ms ela anda em câmara lenta).
            let mut passos = Vec::new();
            for _ in 0..12 {
                let q = std::time::Instant::now();
                t.wet_step_sync(1);
                passos.push(q.elapsed().as_secs_f64() * 1e3);
            }
            passos.sort_by(f64::total_cmp);
            eprintln!(
                "rodada {rodada} solid {solid:<5}: gesto {gesto:>7.3} ms/quadro (pior {pior:>7.3}) · soltar {soltar:>8.3} ms · passo da água mín {:>7.3} med {:>7.3} ms",
                passos[0], passos[6]
            );
        }
    }
}
