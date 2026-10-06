//! Os gates do **`Style: Solid` na aguada** ([`super::watercolor_solido`], doc 46 §2-7).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// Um gesto à mão livre pelos `cantos`, `por_aresta` eventos por aresta, numa aguada azul de raio 4.
fn gesto(solid: bool, cantos: &[[f32; 2]], por_aresta: usize) -> PainterTool {
    let mut t = tool(128, PaintMedia::Watercolor, 4.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.style_solid = solid;
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
        }
    }
    t.on_canvas_pointer(cp(cantos[cantos.len() - 1], PointerPhase::Up));
    t
}

/// Os texels de um quadrado de `lado` px com canto em `(x0, y0)` que a aguada tingiu.
fn tingidos_em(t: &PainterTool, x0: usize, y0: usize, lado: usize) -> usize {
    (y0..y0 + lado)
        .flat_map(|y| (x0..x0 + lado).map(move |x| (y * 128 + x) * 4))
        .filter(|&i| t.canvas_rgba[i] < 240)
        .count()
}

/// **O SOLID ENCHE A AGUADA** — um laço fino à mão livre tinge a região que cerca (o meio do
/// quadrado fica a 30 px de todo o rastro).
#[test]
fn o_solid_enche_a_regiao_cercada_na_aguada() {
    let laco = [
        [24.0, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
    let linha = gesto(false, &laco, 10);
    let mancha = gesto(true, &laco, 10);
    assert_eq!(
        tingidos_em(&linha, 54, 54, 20),
        0,
        "controlo: sem Solid o miolo fica papel"
    );
    assert_eq!(
        tingidos_em(&mancha, 54, 54, 20),
        400,
        "o Solid não encheu o miolo da aguada"
    );
}

/// **A MANCHA DE UM QUADRO NÃO FICA NO SEGUINTE** — o caminho dá a volta ao quadrado e regressa
/// pelo meio, para a forma FINAL ter um entalhe (o triângulo entre o canto de partida e a perna que
/// volta) que as formas INTERMÉDIAS cobriam. Se a mancha provisória ficasse nos acumuladores da
/// aguada, o entalhe saía tingido.
#[test]
fn a_mancha_provisoria_nao_deixa_resto_na_aguada() {
    let volta = [
        [20.0, 20.0],
        [108.0, 20.0],
        [108.0, 108.0],
        [20.0, 108.0],
        [20.0, 60.0],
        [90.0, 60.0],
    ];
    let t = gesto(true, &volta, 12);
    // O entalhe: dentro do triângulo (20,20)–(20,60)–(90,60), longe do rastro (y 20 e 60) e da corda.
    assert!(
        tingidos_em(&t, 64, 80, 16) > 200,
        "controlo: a forma final tem de estar cheia"
    );
    assert_eq!(
        tingidos_em(&t, 27, 41, 8),
        0,
        "a mancha dos quadros intermédios ficou no entalhe da forma final"
    );
}

/// **A MANCHA NÃO PISCA QUANDO O TIQUE CARIMBA** (smoke do dono, 2026-10-04: *«esporadicamente a
/// área de preenchimento pisca»*). O lote do tique descasca a mancha, e o composite do quadro tem de
/// correr DEPOIS de ela voltar. Com a caneta parada o tique carimba sozinho — o `settle` do Space (o
/// pincel de fábrica) e o Airbrush, que carimba em todo tique, mesmo no quadro com movimento.
#[test]
fn a_mancha_nao_some_no_quadro_em_que_o_tique_carimba() {
    let laco = [
        [24.0, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
    // A rota congelada de um composite por evento (`wash.per_event`) compõe no próprio Move.
    for (metodo, por_evento) in [
        (StrokeMethod::Space, false),
        (StrokeMethod::Airbrush, false),
        (StrokeMethod::Space, true),
    ] {
        let mut t = tool(128, PaintMedia::Watercolor, 4.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        t.paint.brush.stroke_method = metodo;
        t.wash.per_event = por_evento;
        t.paint.brush.style_solid = true;
        t.on_canvas_pointer(cp(laco[0], PointerPhase::Down));
        for w in laco.windows(2) {
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
                // Um quadro por evento, como no app: o Airbrush só deixa tinta no tique.
                t.paint_tick(0.1);
            }
        }
        assert_eq!(
            tingidos_em(&t, 54, 54, 20),
            400,
            "{metodo:?} (por evento: {por_evento}): a mancha sumiu no quadro do movimento"
        );
        for quadro in 0..4 {
            t.paint_tick(0.25);
            assert_eq!(
                tingidos_em(&t, 54, 54, 20),
                400,
                "{metodo:?} (por evento: {por_evento}): a mancha sumiu no quadro parado {quadro}"
            );
        }
    }
}

/// **UMA ELIPSE EM SOLID É UM DISCO NA AGUADA** — o editor de forma reconstrói a aguada inteira a
/// cada quadro, e a região entra nela.
#[test]
fn uma_elipse_em_solid_e_um_disco_na_aguada() {
    let elipse = |solid: bool| {
        let mut t = tool(128, PaintMedia::Watercolor, 3.0);
        t.paint.brush.style_solid = solid;
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Up));
        t
    };
    assert_eq!(
        tingidos_em(&elipse(false), 54, 54, 20),
        0,
        "controlo: o anel tem o miolo vazio"
    );
    assert_eq!(
        tingidos_em(&elipse(true), 54, 54, 20),
        400,
        "a elipse em Solid não encheu o miolo na aguada"
    );
}

/// SONDA (relógio) — o report do dono, 2026-10-05: *«ficou lento numa mancha de 1000px»* (na
/// Aquarela, enquanto desenha). Um laço de 1000 px, com e sem Solid, em 2048² e 4096², intercalados.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_mancha_de_1000px_na_aguada -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico de relógio"]
fn diag_a_mancha_de_1000px_na_aguada() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg {}", carga.trim());
    for lado in [2048u32, 4096] {
        #[allow(clippy::cast_precision_loss)]
        let c = (lado / 2) as f32;
        let pt = |i: usize| {
            #[allow(clippy::cast_precision_loss)]
            let a = i as f32 / 120.0 * std::f32::consts::TAU;
            [c + 500.0 * a.cos(), c + 500.0 * a.sin()]
        };
        let variantes = [
            ("sem Solid", false, false),
            ("inteira", true, true),
            ("só o que muda", true, false),
        ];
        for rodada in 0..3 {
            for k in 0..variantes.len() {
                let (nome, solid, inteira) = variantes[(rodada + k) % variantes.len()];
                let mut t = tool(lado, PaintMedia::Watercolor, 20.0);
                t.set_brush_color_srgb8([30, 60, 220]);
                t.paint.brush.style_solid = solid;
                t.wash.mancha_inteira = inteira;
                t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
                let _ = (crate::wash_diag::take(), crate::wash_diag::take_mancha());
                let janelas0 = t.wash.composites;
                let (mut pior, t0) = (0.0f64, std::time::Instant::now());
                for i in 1..=120 {
                    let q = std::time::Instant::now();
                    t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
                    t.paint_tick(1.0 / 60.0);
                    pior = pior.max(q.elapsed().as_secs_f64() * 1e3);
                }
                let gesto = t0.elapsed().as_secs_f64() * 1e3 / 120.0;
                let (w, m) = (crate::wash_diag::take(), crate::wash_diag::take_mancha());
                let mut fases = format!(
                    "composite {:.3} ({:.0} px/quadro em {:.2} janelas de {:.3} ms) · carimbo {:.3} · pour {:.3}",
                    w.composite.avg_ms * w.composite.n as f64 / 120.0,
                    w.window_px_per_composite * w.composite.n as f64 / 120.0,
                    f64::from(t.wash.composites - janelas0) / 120.0,
                    w.composite.avg_ms * w.composite.n as f64
                        / f64::from((t.wash.composites - janelas0).max(1)),
                    w.stamp.avg_ms * w.stamp.n as f64 / 120.0,
                    w.pour.avg_ms * w.pour.n as f64 / 120.0,
                );
                for (nome, f) in crate::wash_diag::FASES_DA_MANCHA.iter().zip(&m) {
                    fases += &format!(" · {nome} {:.3}", f.avg_ms * f.n as f64 / 120.0);
                }
                eprintln!("    {lado}² {nome}: ms/quadro {fases}");
                let q = std::time::Instant::now();
                t.on_canvas_pointer(cp(pt(120), PointerPhase::Up));
                let soltar = q.elapsed().as_secs_f64() * 1e3;
                eprintln!(
                    "{lado}² rodada {rodada} {nome:<13}: gesto {gesto:>8.3} ms/quadro (pior {pior:>8.3}) · soltar {soltar:>8.3} ms"
                );
            }
        }
    }
}

/// Um gesto pelos `cantos` com um quadro por evento e `parados` quadros de caneta parada no fim;
/// devolve a tela de cada quadro (e a do pen-up) e o trabalho do composite (`wash.window_px`).
fn quadros(
    t: &mut PainterTool,
    cantos: &[[f32; 2]],
    por_aresta: usize,
    parados: usize,
) -> (Vec<Vec<u8>>, u64) {
    let mut telas = Vec::new();
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
            t.paint_tick(0.1);
            telas.push(t.canvas_rgba.to_vec());
        }
    }
    for _ in 0..parados {
        t.paint_tick(0.25);
        telas.push(t.canvas_rgba.to_vec());
    }
    t.on_canvas_pointer(cp(cantos[cantos.len() - 1], PointerPhase::Up));
    telas.push(t.canvas_rgba.to_vec());
    (telas, t.wash.window_px)
}

/// **A MANCHA REFEITA SÓ ONDE MUDOU É A DA CAIXA INTEIRA, AO BYTE** — o oráculo é a rota de antes
/// no mesmo processo (`wash.mancha_inteira`): os SEIS planos da aguada iguais em todo quadro, e a
/// tela igual no pen-up (que recompõe o traço inteiro). O caminho volta pelo meio (a mancha ENCOLHE
/// no entalhe), com fios (que caem sobre o papel sem a mancha) e com o Airbrush (o tique carimba
/// com a caneta parada).
///
/// ⚠️ **A tela A MEIO do gesto não é a régua, e está medido porquê:** com o Airbrush um pixel do
/// TRAÇO (fora da mancha) lia `116` na rota inteira e `117` na incremental — e `117` no mesmo gesto
/// SEM Solid. O composite lê uma entrada que o tique do Airbrush muda sem marcar o quadro, e a rota
/// inteira, ao recompor a caixa toda a cada quadro, refrescava-o por acaso. A incremental faz o que
/// o traço sem Solid faz.
#[test]
fn a_mancha_incremental_e_a_inteira_ao_byte() {
    let volta = [
        [20.0, 20.0],
        [108.0, 20.0],
        [108.0, 108.0],
        [20.0, 108.0],
        [20.0, 60.0],
        [90.0, 60.0],
    ];
    let planos = |t: &PainterTool| {
        [
            t.paint.stroke_coverage.clone(),
            t.paint.stroke_color.clone(),
            t.paint.stroke_density.clone(),
            t.paint.stroke_deplete.clone(),
            t.paint.stroke_deplete_prox.clone(),
            t.paint.wet_styles.owner.clone(),
        ]
    };
    use ph2d_painter_brush::line_kind::LineKind;
    // O Wire liga o carimbo de agora aos anteriores pelo PERCURSO: fios longos que atravessam a
    // borda suavizada da mancha LONGE da janela dos dabs — é só lá que a ordem fio × mancha se vê
    // (os dois têm a cor do pincel; na borda a cobertura do fio, «só no seco», difere do `max`).
    for (nome, metodo, fios) in [
        ("Space", StrokeMethod::Space, None),
        (
            "Space + Sketchy",
            StrokeMethod::Space,
            Some(LineKind::Sketchy),
        ),
        ("Space + Wire", StrokeMethod::Space, Some(LineKind::Wire)),
        ("Airbrush", StrokeMethod::Airbrush, None),
    ] {
        let corre = |inteira: bool| {
            let mut t = tool(128, PaintMedia::Watercolor, 4.0);
            t.set_brush_color_srgb8([30, 60, 220]);
            t.paint.brush.stroke_method = metodo;
            t.paint.brush.style_solid = true;
            if let Some(kind) = fios {
                t.paint.brush.line_kind = kind;
                t.paint.brush.sketchy_reach = 3.0;
                t.paint.brush.thread_opacity = 1.0;
            }
            t.wash.mancha_inteira = inteira;
            let mut por_quadro = Vec::new();
            t.on_canvas_pointer(cp(volta[0], PointerPhase::Down));
            for w in volta.windows(2) {
                for k in 1..=12 {
                    #[allow(clippy::cast_precision_loss)]
                    let f = k as f32 / 12.0;
                    t.on_canvas_pointer(cp(
                        [
                            w[0][0] + (w[1][0] - w[0][0]) * f,
                            w[0][1] + (w[1][1] - w[0][1]) * f,
                        ],
                        PointerPhase::Move,
                    ));
                    t.paint_tick(0.1);
                    por_quadro.push(planos(&t));
                }
            }
            for _ in 0..3 {
                t.paint_tick(0.25);
                por_quadro.push(planos(&t));
            }
            t.on_canvas_pointer(cp(volta[5], PointerPhase::Up));
            (por_quadro, t.canvas_rgba.to_vec())
        };
        let ((qa, ta), (qb, tb)) = (corre(true), corre(false));
        for (q, (a, b)) in qa.iter().zip(&qb).enumerate() {
            for k in 0..a.len() {
                let difere = a[k].iter().zip(&b[k]).filter(|(p, s)| p != s).count();
                assert_eq!(
                    difere, 0,
                    "{nome}: no quadro {q} o plano {k} da mancha incremental difere da inteira em \
                     {difere} bytes"
                );
            }
        }
        let difere = ta.iter().zip(&tb).filter(|(p, s)| p != s).count();
        assert_eq!(
            difere, 0,
            "{nome}: a tela do pen-up difere em {difere} bytes"
        );
        assert!(ta.iter().any(|&v| v < 240), "controlo: {nome} pintou");
    }
}

/// **O FIO QUE CAI SOB A MANCHA SOBREVIVE QUANDO ELA ENCOLHE** — um fio cai a meio do gesto através
/// da mancha viva, longe da janela dos dabs; o caminho volta pelo meio e o entalhe tira a mancha de
/// cima de metade dele. O fio tem de ficar lá (ele é tinta CUMULATIVA, a mancha é que é provisória):
/// por isso o depósito dos fios descasca a mancha sob eles antes de cair. Sem isso o fio caía POR
/// CIMA da mancha e o entalhe, ao repor o papel sem ela, levava o fio junto.
#[test]
fn o_fio_sob_a_mancha_sobrevive_ao_entalhe() {
    use ph2d_painter_brush::thread_raster::{ThreadInk, threads_bbox};
    let volta = [
        [20.0, 20.0],
        [108.0, 20.0],
        [108.0, 108.0],
        [20.0, 108.0],
        [20.0, 60.0],
        [90.0, 60.0],
    ];
    let mut linhas = Vec::new();
    for (inteira, solid) in [(false, false), (false, true), (true, true)] {
        let mut t = tool(128, PaintMedia::Watercolor, 4.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        t.paint.brush.style_solid = solid;
        t.wash.mancha_inteira = inteira;
        t.on_canvas_pointer(cp(volta[0], PointerPhase::Down));
        for (perna, w) in volta.windows(2).enumerate() {
            if perna == 4 {
                // O fio: uma linha a y = 40 através da mancha viva (o entalhe vai de x 20 a 55 ali).
                let fio = [[24.0f32, 40.5, 100.0, 40.5]];
                let ink = ThreadInk {
                    width_px: 6.0,
                    opacity: 1.0,
                };
                let [bx, by, bw, bh] = threads_bbox(&fio, ink.width_px, 128, 128).expect("caixa");
                #[allow(clippy::cast_possible_truncation)]
                let rect = crate::tool::paint::Region {
                    x: bx as u32,
                    y: by as u32,
                    w: bw as u32,
                    h: bh as u32,
                };
                t.fios_na_aguada(&fio, ink, rect);
            }
            for k in 1..=12 {
                #[allow(clippy::cast_precision_loss)]
                let f = k as f32 / 12.0;
                t.on_canvas_pointer(cp(
                    [
                        w[0][0] + (w[1][0] - w[0][0]) * f,
                        w[0][1] + (w[1][1] - w[0][1]) * f,
                    ],
                    PointerPhase::Move,
                ));
                t.paint_tick(0.1);
            }
        }
        t.on_canvas_pointer(cp(volta[5], PointerPhase::Up));
        // O troço do fio que ficou no entalhe (x 28..44, longe da borda e do rastro).
        let tingidos = (28..44)
            .filter(|&x| t.canvas_rgba[(40 * 128 + x) * 4] < 240)
            .count();
        assert_eq!(
            tingidos, 16,
            "rota inteira = {inteira}, Solid = {solid}: o fio não está no entalhe"
        );
        linhas.push(
            (28..44)
                .flat_map(|x| t.canvas_rgba[(40 * 128 + x) * 4..(40 * 128 + x) * 4 + 4].to_vec())
                .collect::<Vec<u8>>(),
        );
    }
    // O fio no entalhe é o do mesmo gesto SEM Solid, ao byte, nas duas rotas.
    for (i, l) in linhas.iter().enumerate().skip(1) {
        assert_eq!(
            l, &linhas[0],
            "rota {i}: o fio que caiu sob a mancha não é o do gesto sem Solid"
        );
    }
}

/// **O QUADRO RECOMPÕE SÓ O QUE MUDOU** — num laço de 400 px o composite caminha uma fração dos
/// texels da rota inteira (contagem, não relógio: `wash.window_px`).
#[test]
fn o_quadro_da_mancha_recompoe_so_o_que_mudou() {
    let laco: Vec<[f32; 2]> = (0..=48)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            [256.0 + 200.0 * a.cos(), 256.0 + 200.0 * a.sin()]
        })
        .collect();
    let trabalho = |inteira: bool| {
        let mut t = tool(512, PaintMedia::Watercolor, 6.0);
        t.paint.brush.style_solid = true;
        t.wash.mancha_inteira = inteira;
        quadros(&mut t, &laco, 2, 0).1
    };
    let (inteira, so_o_que_mudou) = (trabalho(true), trabalho(false));
    #[allow(clippy::cast_precision_loss)]
    let razao = so_o_que_mudou as f64 / inteira as f64;
    eprintln!(
        "texels do composite: inteira {inteira} · só o que mudou {so_o_que_mudou} · {razao:.3}"
    );
    // Medido 2026-10-05 com as faixas fundidas: `0,332` (a caixa única dava `0,398`).
    assert!(
        razao < 0.35,
        "o composite ainda caminha a caixa inteira: {so_o_que_mudou} contra {inteira} ({razao:.3})"
    );
}

/// **A MANCHA QUE ENCOLHE DEVOLVE O PAPEL FORA DELA** — o registo da mancha só cresce, e quando a
/// região nova é menor que ele as linhas do registo fora dela têm de voltar ao papel sem mancha. Os
/// seis planos depois de um quadrado grande e um pequeno são os de só o pequeno. ⚠️ A borda de baixo
/// é FRACCIONÁRIA: a última linha do pequeno tem cobertura, e é ela que fica no rascunho da linha.
#[test]
fn a_mancha_que_encolhe_devolve_o_papel_fora_dela() {
    let quadrado = |a: f32, b: f32| vec![vec![[a, a], [b, a], [b, b], [a, b]]];
    let prepara = || {
        let mut t = tool(128, PaintMedia::Watercolor, 4.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([66.0, 64.0], PointerPhase::Move));
        t.paint_tick(0.1);
        t
    };
    let planos = |t: &PainterTool| {
        [
            t.paint.stroke_coverage.clone(),
            t.paint.stroke_color.clone(),
            t.paint.stroke_density.clone(),
            t.paint.wet_styles.owner.clone(),
        ]
    };
    let mut a = prepara();
    a.atualiza_a_mancha(&quadrado(10.0, 110.0));
    let grande = planos(&a);
    a.atualiza_a_mancha(&quadrado(40.0, 80.5));
    let mut b = prepara();
    b.atualiza_a_mancha(&quadrado(40.0, 80.5));
    assert_ne!(
        grande[0],
        planos(&b)[0],
        "controlo: o quadrado grande pintou"
    );
    for (k, (p, q)) in planos(&a).iter().zip(planos(&b).iter()).enumerate() {
        let difere = p.iter().zip(q).filter(|(x, y)| x != y).count();
        assert_eq!(
            difere, 0,
            "plano {k}: a mancha que encolheu deixou {difere} bytes"
        );
    }
}
