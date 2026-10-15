//! Os gates da cena `=142`: monta pelo roteador, a saída da galáxia é `Scene` em `Screen` com as
//! IMAGENS antes das ESTRELAS no `combine`, e vai à PLACA por um desenho ordenado.

use super::*;
use crate::motion_state::MotionState;

fn cena() -> (MotionState, Cena) {
    let mut m = MotionState::new();
    let c = monta(&mut m.doc, &m.registry).expect("a cena monta e valida");
    m.sinks = vec![c.fundo, c.saida];
    (m, c)
}

/// ⭐ **A cena monta pelo ROTEADOR** (o caminho do `PH2D_GPU_COOK_DEMO=142`): o fundo e a galáxia.
#[test]
fn a_cena_142_monta_pelo_roteador() {
    let m = MotionState::new();
    let mut doc = ph2d_motion_doc::MotionDoc::new();
    let (sinks, legenda) = crate::motion_demo_legend::monta("142", &mut doc, &m.registry);
    assert_eq!(sinks.len(), 2, "o fundo e a galaxia");
    assert_eq!(legenda.len(), 1, "uma ficha no canvas");
    assert!(crate::motion_state::demo_router::is_cycle_scene("142"));
}

/// ⭐ **O `Scene` e a ORDEM**: a galáxia mistura com o cenário em `Screen` com o alcance `Scene` (o
/// fundo não); e no `combine` as IMAGENS entram pela porta 0 e as ESTRELAS pela 1 — a ordem das
/// linhas que põe as estrelas por cima. Trocadas, a cena ensinaria o contrário.
#[test]
fn a_cena_142_e_um_scene_com_as_imagens_antes_das_estrelas() {
    let (m, c) = cena();
    let g = &m.doc.graph;
    assert_eq!(
        ph2d_eval_motion::sink_blend_with(g, c.saida),
        BlendWith::Scene
    );
    assert_eq!(ph2d_eval_motion::sink_style(g, c.saida).blend, MISTURA);
    assert_eq!(
        ph2d_eval_motion::sink_blend_with(g, c.fundo),
        BlendWith::Everything
    );
    assert_eq!(
        ph2d_eval_motion::sink_style(g, c.fundo).blend,
        0,
        "o fundo em Normal"
    );
    let quem = |porta: u16| {
        g.edges()
            .iter()
            .find(|e| e.to == (c.junta, porta))
            .map(|e| e.from.0)
    };
    assert_eq!(quem(0), Some(c.imagens), "as imagens entram PRIMEIRO");
    assert_eq!(quem(1), Some(c.estrelas), "as estrelas DEPOIS (por cima)");
    let tipo = |n| g.node(n).map(|i| i.type_name.clone());
    assert_eq!(tipo(c.imagens).as_deref(), Some("motion.tint"));
    assert_eq!(tipo(c.estrelas).as_deref(), Some("motion.duplicator"));
}

/// ⭐⭐ **A cena vai à PLACA** pela porta do quadro do produto: a rota é o dispositivo desde o 1.º
/// quadro, as imagens da galáxia chegam ao buffer da placa (uma por ponto), e a decisão da placa
/// desenha o `Scene` por uma corrida ORDENADA (as duas médias num desenho, na ordem das linhas).
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_cena_142_vai_a_placa_por_uma_corrida_ordenada() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a rota do device");
    };
    let (mut m, _) = cena();
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    for q in 1..=4u32 {
        playhead.seek(f64::from(q) / 60.0);
        crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, 1.0 / 60.0);
        assert!(
            m.gpu_live && m.route_said.is_some_and(|r| r.starts_with("device")),
            "quadro {q}: {:?}",
            m.route_said
        );
    }
    let imagens = m
        .gpu_cook
        .imagens_do_grupo()
        .expect("as imagens foram à placa");
    let linhas = ph2d_gpu_cook::read_imagens_do_grupo(&gpu, imagens);
    let visiveis = linhas.iter().filter(|r| r.size != [0.0, 0.0]).count();
    let galaxia = (super::super::ritmo_demo::LADO * super::super::ritmo_demo::LADO) as usize;
    assert_eq!(visiveis, galaxia, "um quadrado de imagem por ponto");
    let mut placa = crate::motion_shape_placa::PlacaDeFormas::default();
    assert!(crate::motion_shape_placa::decide_o_quadro(
        &mut placa,
        &gpu,
        &mut m,
        (true, true),
        crate::motion_shape_placa::gpu_tests::camara(),
    ));
    assert!(
        placa.corridas_do_quadro().iter().any(|c| c.ordenada()),
        "o Scene com imagens desenha-se por uma corrida ORDENADA"
    );
}

/// ⭐ **A LEGENDA NÃO TAPA A GALÁXIA, em nenhum quadro.** A galáxia é SIMULADA (o redemoinho e o íman
/// mexem-na), logo a borda de baixo mede-se ao longo de duas voltas (`2 · CICLO`), e a ficha (11 px,
/// ~0,35 m de altura na câmara da cena, fotografado) pousa abaixo dela. Achado na foto de 15/10: a
/// `-3,2` herdada da `=140` (duas galáxias menores) tapava a ponta de baixo do losango.
#[test]
fn a_legenda_da_cena_142_pousa_abaixo_da_galaxia_em_todos_os_quadros() {
    const MEIA_FICHA: f32 = 0.35;
    let (mut m, c) = cena();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut baixo = f32::INFINITY;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos inteiros"
    )]
    let tiques = (2.0 * super::super::ritmo_demo::CICLO * 60.0) as u64;
    for t in 0..=tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        let s = m.pump.cook.peek(c.saida).expect("a saida cozeu")[0]
            .as_stream()
            .clone();
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s.get("P") else {
            panic!("a saida tem posicoes");
        };
        // O quadrado pode girar: a meia-diagonal cobre qualquer ângulo.
        let fundo = p.iter().map(|q| q[1]).fold(f32::INFINITY, f32::min)
            - IMAGEM * std::f32::consts::FRAC_1_SQRT_2;
        baixo = baixo.min(fundo);
    }
    let ficha = captions()[0].world[1];
    eprintln!("cena 142: borda de baixo {baixo:.3} m · ficha a {ficha:.3} m");
    assert!(baixo.is_finite(), "a galaxia tem pontos");
    assert!(
        ficha + MEIA_FICHA <= baixo,
        "a legenda (y = {ficha}) tapa a galaxia, cuja borda de baixo chega a {baixo:.3}"
    );
}
