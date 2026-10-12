//! **O degradê do Gradient Map, ASSADO** — a lei do [`super::bake_ramp`] e a medição que escolheu o
//! [`FxOp::RAMP_LUT_TEXELS`].

use super::*;
use ph2d_color::ramp_lut::bake;
use ph2d_color::ramp_lut::regua::{DISPLAY_STEP, SWEEP, corpus, erro_no_ecra, pior};
use ph2d_color::srgb::linear_to_srgb_unit;
use ph2d_color::{ColorRamp, GradientPreset, RampColorMode, RampInterp};

/// A reconstrução do `ramp_sample` do passe (`fx_stack_shader.rs`), a mesma aritmética.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "índices de uma tabela pequena"
)]
fn reconstruct(lut: &[[f32; 4]], t: f32) -> [f32; 4] {
    let x = t.clamp(0.0, 1.0) * (lut.len() - 1) as f32;
    let i = (x.floor() as usize).min(lut.len() - 2);
    let (c0, c1) = (lut[i], lut[i + 1]);
    std::array::from_fn(|c| c0[c] + (c1[c] - c0[c]) * (x - i as f32))
}

/// **A medição que escolheu a resolução** —
/// `bash scripts/cargo-test-narrow.sh ph2d-node-fx-filter measure_ramp_lut -- --nocapture`.
#[test]
fn measure_ramp_lut_in_the_screen_encoding() {
    eprintln!("passo do ecra' {DISPLAY_STEP:.5} (sRGB)");
    for n in [128usize, 256, 512, 1024, 2048, 4096] {
        eprintln!("{n:>5} texels:");
        for (nome, r) in corpus() {
            let (w, f) = erro_no_ecra(&r, n, reconstruct);
            eprintln!("    {nome:<24} {w:.5}  {:>7.3}%", f * 100.0);
        }
    }
    // ⛔ Recusa medida: guardar a tabela JÁ em sRGB não cura — o erro é das QUINAS das paradas,
    // não da curvatura (pior do corpus: 0,0478 a 512, 0,0117 a 2 048, os mesmos da tabela linear).
    eprintln!("-- a tabela guardada em sRGB (a alternativa recusada):");
    for n in [512usize, 2048] {
        let tela = |v: f32| linear_to_srgb_unit(v.clamp(0.0, 1.0));
        for (nome, r) in corpus() {
            let lut: Vec<[f32; 4]> = bake(&r, n)
                .iter()
                .map(|c| [tela(c[0]), tela(c[1]), tela(c[2]), c[3]])
                .collect();
            #[expect(clippy::cast_precision_loss, reason = "SWEEP <= 8192")]
            let worst = (0..=SWEEP).fold(0.0f32, |m, i| {
                let t = i as f32 / SWEEP as f32;
                let (got, want) = (reconstruct(&lut, t), r.eval(t));
                (0..3).fold(m, |m, c| m.max((got[c] - tela(want[c])).abs()))
            });
            eprintln!("    {n:>5} {nome:<24} {worst:.5}");
        }
    }
    assert!(
        pior(16, reconstruct) > DISPLAY_STEP,
        "controlo: a régua vê 16 texels"
    );
}

/// **A resolução que corre é a medida** — invisível no corpus inteiro, e metade dela visível (senão
/// pagávamos memória por nada).
#[test]
fn the_baked_ramp_is_invisible_and_half_of_it_is_not() {
    for (nome, r) in corpus() {
        let (w, _) = erro_no_ecra(&r, FxOp::RAMP_LUT_TEXELS, reconstruct);
        assert!(w <= DISPLAY_STEP, "{nome}: {w} acima do passo do ecrã");
    }
    let meia = pior(FxOp::RAMP_LUT_TEXELS / 2, reconstruct);
    assert!(
        meia > DISPLAY_STEP,
        "metade dos texels ({meia}) tem de se ver"
    );
}

/// `source.grid → fx.filter (Gradient Map) → motion.output`, com o degradê `texto`.
fn mapa(texto: Option<&str>) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let ids = ["source.grid", TYPE_NAME, "motion.output"].map(|t| g.add_node(t));
    for par in ids.windows(2) {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (par[0], 0),
            to: (par[1], 0),
            delayed: false,
        })
        .expect("liga");
    }
    let no = ids[1];
    g.set_param(no, param::KIND, f32::from(FxOp::GRADIENT_MAP));
    if let Some(t) = texto {
        g.set_text_param(no, param::RAMP.to_string(), t.to_string());
    }
    (g, no)
}

/// ⭐ **A tabela É o `ColorRamp::eval`** — ao bit, com espaço, interpolação e alfa que a lei dos
/// stops não tem (HSV, `Ease`, três paradas). Controlos: sem texto, com lixo, ou num tipo que não lê
/// a rampa, não há tabela — e a `op` desenha os stops de fábrica.
#[test]
fn the_baked_ramp_is_what_the_colour_library_evaluates() {
    let mut ramp = GradientPreset::Rainbow.ramp();
    ramp.color_mode = RampColorMode::Hsv;
    ramp.interp = RampInterp::Ease;
    let texto = ph2d_color::serialize_gradient(&ramp);
    let lido = ph2d_color::parse_gradient(&texto).expect("o texto volta");
    let (g, no) = mapa(Some(&texto));
    let lut = bake_ramp(&g, no).expect("o Gradient Map com texto assa");
    assert_eq!(lut.len(), FxOp::RAMP_LUT_TEXELS);
    #[expect(clippy::cast_precision_loss, reason = "4096 amostras")]
    for (i, c) in lut.iter().enumerate() {
        let t = i as f32 / (lut.len() - 1) as f32;
        assert_eq!(
            c.map(f32::to_bits),
            lido.eval(t).map(f32::to_bits),
            "amostra {i}"
        );
    }
    assert_eq!(
        from_graph(&g)[0].ramp.as_deref(),
        Some(&lut[..]),
        "a pilha leva a tabela"
    );
    let (vazio, n) = mapa(None);
    assert!(bake_ramp(&vazio, n).is_none(), "sem texto");
    let (lixo, n) = mapa(Some("g9 isto nao e' um degrade"));
    assert!(bake_ramp(&lixo, n).is_none(), "texto que não parse");
    let (mut duo, n) = mapa(Some(&texto));
    duo.set_param(n, param::KIND, f32::from(FxOp::DUOTONE));
    assert!(bake_ramp(&duo, n).is_none(), "o Duotone não lê a rampa");
}

/// O degradê de fábrica do editor (`ColorRamp::default`, preto→branco linear) e os stops de fábrica
/// da `op` são a MESMA rampa — o cartão sem texto não mostra uma coisa e desenha outra.
#[test]
fn the_editors_default_is_the_ops_factory_ramp() {
    let editor = ColorRamp::default();
    let (g, no) = mapa(None);
    let (stops, pos, n) = op_of(&g, no).ramp_for_device();
    assert_eq!(n, 2);
    for (i, t) in [0.0f32, 1.0].into_iter().enumerate() {
        let lin = stops[i].map(ph2d_color::srgb::srgb_to_linear_unit);
        let want = editor.eval(t);
        assert!((pos[0][i] - t).abs() < 1e-6);
        for c in 0..3 {
            assert!((lin[c] - want[c]).abs() < 1e-6, "parada {i}, canal {c}");
        }
    }
}

/// O degradê assa-se UMA vez por texto: dois quadros com o mesmo texto partilham a tabela, o texto
/// mexido assa outra, e o que saiu do grafo sai da memória. Controlo: sem a memória assa sempre.
#[test]
fn the_ramp_bakes_once_per_text() {
    let texto = |p: GradientPreset| ph2d_color::serialize_gradient(&p.ramp());
    let (mut g, no) = mapa(Some(&texto(GradientPreset::Heat)));
    let mut rampas = RampasAssadas::default();
    let mut le = |g: &Graph| {
        from_graph_cached(g, &mut rampas)[0]
            .ramp
            .clone()
            .expect("assa")
    };
    let (a, b) = (le(&g), le(&g));
    assert!(Arc::ptr_eq(&a, &b), "o mesmo texto não reassa");
    g.set_text_param(no, param::RAMP.to_string(), texto(GradientPreset::Ice));
    let c = le(&g);
    assert!(!Arc::ptr_eq(&a, &c), "o texto mexido assa outra");
    assert_eq!(rampas.len(), 1, "o texto antigo saiu");
    let sem = from_graph(&g)[0].ramp.clone().expect("assa");
    assert!(
        !Arc::ptr_eq(&sem, &c) && sem == c,
        "controlo: sem a memória, outra tabela igual"
    );
}
