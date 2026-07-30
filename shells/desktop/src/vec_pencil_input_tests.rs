//! **O estabilizador do lápis** — a mão filtrada, e a medição que escolhe o default.

use ph2d_tool_vector::params::PENCIL_STABILIZER_DEFAULT as DEFAULT_STABILIZER;

use super::PencilHand;

/// Unidades de mundo por px de tela (a régua da câmera default).
const PX_TO_WORLD: f64 = 0.0045;

/// O S ideal, em px de tela: a curva que a mão TENTA desenhar.
fn ideal_px(t: f64) -> (f32, f32) {
    (
        (60.0 + 300.0 * t) as f32,
        (200.0 + 90.0 * (t * std::f64::consts::TAU).sin()) as f32,
    )
}

/// A mão REAL: o S ideal mais um tremor determinístico de ±`amp` px.
///
/// ⚠️ Duas frequências incomensuráveis de propósito. Um tremor de uma senoide só é *suave*, e um
/// filtro passa-baixa remove-o quase perfeitamente — a fixture faria o estabilizador parecer melhor
/// do que ele é.
fn hand_px(n: usize, amp: f64) -> Vec<(f32, f32)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            let f = i as f64;
            let (x, y) = ideal_px(t);
            (
                x + (amp * (f * 1.9).sin()) as f32,
                y + (amp * (f * 2.7).cos()) as f32,
            )
        })
        .collect()
}

/// A distância de `p` à curva ideal, amostrada fina.
fn dist_to_ideal(p: (f32, f32)) -> f64 {
    (0..2001)
        .map(|k| {
            let (ix, iy) = ideal_px(f64::from(k) / 2000.0);
            let (dx, dy) = (f64::from(p.0 - ix), f64::from(p.1 - iy));
            (dx * dx + dy * dy).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

/// Corre a mão inteira por uma força de estabilização; devolve
/// `(tremor residual máx, atraso no fim, nós do traço)`.
fn run(strength: f32) -> (f64, f64, usize) {
    let raw = hand_px(240, 1.5);
    let mut hand = PencilHand::default();
    let mut pencil = ph2d_vec_edit::Pencil::default();
    let mut scene = ph2d_vec_scene::VecScene::new();

    hand.begin(raw[0]);
    let to_world = |p: (f32, f32)| [f64::from(p.0) * PX_TO_WORLD, f64::from(p.1) * PX_TO_WORLD];
    let id = pencil.on_press(&mut scene, to_world(raw[0]), PX_TO_WORLD);

    let mut worst = 0.0_f64;
    let mut last = raw[0];
    for &p in &raw[1..] {
        let f = hand.filter(p, strength);
        worst = worst.max(dist_to_ideal(f));
        pencil.on_drag(&mut scene, to_world(f));
        last = f;
    }
    let tail = raw[raw.len() - 1];
    let lag = f64::from((last.0 - tail.0).hypot(last.1 - tail.1));
    let nodes = scene.path(id).map_or(0, |p| p.verts.len());
    (worst, lag, nodes)
}

/// **A SONDA que escolhe o default.** `cargo test -p ph2d-host-desktop --bins
/// measure_pencil_stabilizer -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: escolhe o default de estabilização"]
fn measure_pencil_stabilizer() {
    eprintln!("stabilizer | tremor residual (px) | atraso final (px) | nós");
    for s in [0.0_f32, 0.25, 0.5, 0.75, 0.9, 0.97, 1.0] {
        let (tremor, lag, nodes) = run(s);
        eprintln!("      {s:.2} |              {tremor:6.2} |            {lag:6.2} | {nodes:3}");
    }
    let (raw_tremor, _, _) = run(0.0);
    eprintln!("(o tremor da mão CRUA é {raw_tremor:.2} px — é o que há para remover)");
}

/// **A SONDA da FAIXA de Fidelity**, com a mão realista e o estabilizador no default — que é o que
/// o artista de facto terá. `cargo test -p ph2d-host-desktop --bins measure_pencil_fidelity_range
/// -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: escolhe a faixa do slider de Fidelity"]
fn measure_pencil_fidelity_range() {
    let raw = hand_px(240, 1.5);
    eprintln!("fidelity (px) | nós | desvio máx da curva IDEAL (px)");
    for fid in [0.5_f64, 1.0, 2.0, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0] {
        let mut hand = PencilHand::default();
        let mut pencil = ph2d_vec_edit::Pencil::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        pencil.set_fidelity_px(fid);
        hand.begin(raw[0]);
        let to_world = |p: (f32, f32)| [f64::from(p.0) * PX_TO_WORLD, f64::from(p.1) * PX_TO_WORLD];
        let id = pencil.on_press(&mut scene, to_world(raw[0]), PX_TO_WORLD);
        for &p in &raw[1..] {
            let f = hand.filter(p, DEFAULT_STABILIZER);
            pencil.on_drag(&mut scene, to_world(f));
        }
        pencil.on_release(&mut scene);
        let nodes = scene.path(id).map_or(0, |p| p.verts.len());
        // A curva RESULTANTE, amostrada por cúbica, contra o S ideal — em px de tela.
        let verts = &scene.path(id).expect("o traço existe").verts;
        let mut worst = 0.0_f64;
        for i in 0..verts.len().saturating_sub(1) {
            let (p0, c1) = (verts[i].anchor, verts[i].out_handle);
            let (c2, p3) = (verts[i + 1].in_handle, verts[i + 1].anchor);
            for s in 0..=24 {
                let t = f64::from(s) / 24.0;
                let mt = 1.0 - t;
                let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
                let pt = [
                    p0[0] * a + c1[0] * b + c2[0] * c + p3[0] * d,
                    p0[1] * a + c1[1] * b + c2[1] * c + p3[1] * d,
                ];
                worst = worst.max(dist_to_ideal((
                    (pt[0] / PX_TO_WORLD) as f32,
                    (pt[1] / PX_TO_WORLD) as f32,
                )));
            }
        }
        eprintln!("       {fid:5.1} | {nodes:3} | {worst:6.2}");
    }
}

/// **Estabilização 0 é o ponteiro cru, ao bit** — o produto de antes desta wave.
#[test]
fn zero_stabilizer_is_the_raw_pointer() {
    let mut hand = PencilHand::default();
    hand.begin((10.0, 10.0));
    for p in [(11.0, 12.0), (40.0, 3.5), (-7.25, 90.125)] {
        assert_eq!(
            hand.filter(p, 0.0),
            p,
            "com o slider no mínimo o lápis TEM de ver o ponteiro cru"
        );
    }
}

/// **Estabilizar reduz o tremor** — e o preço é o atraso. As duas metades num gate só, porque um
/// "conserto" que só suavizasse mais (sem limite) passaria na primeira e é o que torna a ferramenta
/// inutilizável.
#[test]
fn stabilising_trades_tremor_for_lag() {
    let (raw_tremor, raw_lag, _) = run(0.0);
    let (soft_tremor, soft_lag, _) = run(0.75);
    assert!(
        soft_tremor < raw_tremor * 0.75,
        "estabilizar não removeu tremor: cru {raw_tremor:.2} px, filtrado {soft_tremor:.2} px"
    );
    assert!(
        soft_lag > raw_lag,
        "o ponto filtrado não atrasa — então ele não está a filtrar nada (cru {raw_lag:.2} px, \
         filtrado {soft_lag:.2} px)"
    );
}

/// **O default é o que a sonda mediu, e ele COMPRA o que diz comprar.**
///
/// Não é um pin de literal (esse não prova nada): o gate re-corre a medição e exige do default as
/// duas propriedades que o escolheram — a contagem de nós cai para uma que se pode editar à mão, e o
/// atraso fica abaixo de um quarto da largura de traço default (3 px), ou seja invisível.
#[test]
fn the_default_stabilizer_buys_an_editable_node_count_for_invisible_lag() {
    let (_, raw_lag, raw_nodes) = run(0.0);
    let (_, lag, nodes) = run(DEFAULT_STABILIZER);
    assert_eq!(raw_lag, 0.0, "a mão crua não pode atrasar");
    assert!(
        nodes * 4 < raw_nodes,
        "o default não corta a contagem de nós (crua {raw_nodes}, default {nodes}) — num lápis \
         VETORIAL o que o artista herda é a curva que vai editar"
    );
    assert!(
        lag < 3.0,
        "o atraso do default ({lag:.2} px) passou de um quarto da largura de traço default — o \
         traço termina visivelmente antes de onde a mão levantou"
    );
}

/// **Passado o joelho, o filtro come o DESENHO** — a propriedade não-monotónica que a sonda achou,
/// pinada para ninguém "melhorar" o default subindo-o.
#[test]
fn past_the_knee_the_filter_eats_the_drawing() {
    let (best, _, _) = run(0.75);
    let (maxed, _, _) = run(1.0);
    let (raw, _, _) = run(0.0);
    assert!(
        maxed > best * 2.0,
        "a 1,00 o tremor residual ({maxed:.2} px) devia ser MUITO pior que no joelho \
         ({best:.2} px) — se não é, a fixture deixou de conter o fenómeno"
    );
    assert!(
        maxed > raw,
        "a 1,00 o traço devia estar MAIS longe da curva pretendida que a mão crua \
         (cru {raw:.2} px, filtrado {maxed:.2} px): e' o sinal a ser comido, não o ruído"
    );
}

/// **As DUAS cópias do default de Fidelity concordam** — o gate que torna a duplicata segura.
///
/// O tool autora o valor (semeia o slider, inicializa o campo) e o motor tem o SEU default como
/// fallback para um chamador que nunca escolhe. As duas crates não se veem — a shell é a única que
/// vê ambas, então é aqui que a igualdade pode ser afirmada. Sem este gate, mexer num dos números
/// deixa o slider a dizer 2,0 e o motor a decimar noutra tolerância, **sem erro em lado nenhum**.
#[test]
fn the_two_fidelity_defaults_agree() {
    assert!(
        (ph2d_tool_vector::params::PENCIL_FIDELITY_DEFAULT_PX
            - ph2d_vec_edit::pencil::DEFAULT_FIDELITY_PX)
            .abs()
            < f64::EPSILON,
        "o default de Fidelity do TOOL ({}) diverge do fallback do MOTOR ({}) — o slider mostraria \
         um número e o decimador usaria outro",
        ph2d_tool_vector::params::PENCIL_FIDELITY_DEFAULT_PX,
        ph2d_vec_edit::pencil::DEFAULT_FIDELITY_PX
    );
}

/// **O default de Fidelity está DENTRO da faixa do slider** — um default fora dela é um knob que
/// nasce a mentir (o botão encosta numa ponta e o valor autorado é outro).
#[test]
fn the_fidelity_default_is_inside_the_slider_range() {
    use ph2d_tool_vector::params::{
        PENCIL_FIDELITY_DEFAULT_PX, PENCIL_FIDELITY_MAX_PX, PENCIL_FIDELITY_MIN_PX,
        fidelity_px_to_slider, slider_to_fidelity_px,
    };
    assert!(
        (PENCIL_FIDELITY_MIN_PX..=PENCIL_FIDELITY_MAX_PX).contains(&PENCIL_FIDELITY_DEFAULT_PX),
        "o default {PENCIL_FIDELITY_DEFAULT_PX} está fora de \
         {PENCIL_FIDELITY_MIN_PX}..={PENCIL_FIDELITY_MAX_PX}"
    );
    // E o par de mapeamentos é inverso um do outro nas duas pontas e no default.
    for px in [
        PENCIL_FIDELITY_MIN_PX,
        PENCIL_FIDELITY_DEFAULT_PX,
        PENCIL_FIDELITY_MAX_PX,
    ] {
        let back = slider_to_fidelity_px(fidelity_px_to_slider(px));
        assert!(
            (back - px).abs() < 1e-6,
            "o mapeamento do slider de Fidelity não é inverso: {px} px -> {back} px"
        );
    }
}
