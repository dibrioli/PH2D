//! ⭐⭐⭐ **A MISTURA EM GRUPO PELA ROTA DO DISPOSITIVO** (B6 · P4) — a MESMA cena de estrelas
//! SOBREPOSTAS, com o `Output` a misturar, cozida pelas duas rotas da ponte e desenhada pela placa de
//! formas: as duas camadas comparam-se pixel a pixel, em cada modo × alcance × camada isolada, e as
//! duas pousas (P4 (b)) também. As IMAGENS de um grupo têm o irmão `mistura_imagens_tests`.
//!
//! ⚠️ As metades que a paridade sozinha não diz: o dispositivo tem de ter sido TOMADO (senão o gate
//! mede a CPU contra si mesma), as corridas têm de levar a mistura (senão o Normal das duas rotas
//! também empata — por isso a CPU é comparada também com o `Normal`, que tem de DIFERIR), e onde o
//! dispositivo recusa (o `Scene` fora da camada isolada) di-lo pelo nome.
//!
//! `#[ignore]`: precisa de adaptador real.
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib --release \
//!        mistura_tests -- --ignored --nocapture`

use super::mistura::RECUSA_SCENE_NO_DISPOSITIVO;
use super::{GpuOutcome, cook_gpu};
use crate::motion_shape_placa::gpu_tests::{LADO, camara, compara, le_a_camada};
use crate::motion_shape_placa::{
    Misturas, PlacaDeFormas, corridas_do_dispositivo, decide_o_quadro, desenha_o_quadro,
};
use crate::motion_state::MotionState;
use ph2d_eval_motion::BlendWith;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::{Edge, NodeId};

const TIQUE: u64 = 2;
const DT: f64 = 1.0 / 60.0;

fn liga(m: &mut MotionState, de: NodeId, para: NodeId, porta: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("fio");
}

/// `grelha 4×4 → duplicador ← estrela → rotate → tint [→ combine] → Output(tag, com)` — as estrelas
/// SOBREPÕEM-SE (o passo é menor que a estrela). Com `junta`, um `combine` antes da saída e uma
/// grelha de IMAGENS solta (o controlo); sem ele a cadeia é toda da placa, como no irmão.
fn estado(tag: u8, com: BlendWith, isolada: bool, junta: bool) -> (MotionState, NodeId, NodeId) {
    estado_de(4.0, false, tag, com, isolada, junta)
}

/// O [`estado`] com uma grelha `lado × lado` e, com `anima`, um `motion.wiggle` depois do `rotate`
/// (a sonda do preço: uma cena parada a CPU serve da memória e não coze nada).
pub(super) fn estado_de(
    lado: f32,
    anima: bool,
    tag: u8,
    com: BlendWith,
    isolada: bool,
    junta: bool,
) -> (MotionState, NodeId, NodeId) {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grade = g.add_node("motion.grid");
    for (k, v) in [
        ("rows", lado),
        ("cols", lado),
        ("gap_x", 0.8),
        ("gap_y", 0.8),
    ] {
        g.set_param(grade, k, v);
    }
    let forma = g.add_node("source.shape");
    let estrela = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
        .expect("a estrela");
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(forma, ph2d_node_motion_shape::param::KIND, estrela as f32);
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, 1.4);
    let dup = g.add_node("motion.duplicator");
    let roda = g.add_node("motion.rotate");
    g.set_param(roda, "angle", 23.0);
    // Um tom MÉDIO e meio transparente: o branco opaco satura e o Add leria-se como o Normal.
    let tinta = g.add_node("motion.tint");
    for (k, v) in [("r", 0.35), ("g", 0.45), ("b", 0.6), ("a", 0.7)] {
        g.set_param(tinta, k, v);
    }
    let combina = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    g.set_param(out, ph2d_eval_motion::SINK_BLEND_PARAM, f32::from(tag));
    let alcance = BlendWith::ALL.iter().position(|c| *c == com).unwrap_or(0);
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(out, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, alcance as f32);
    let imagens = g.add_node("motion.grid");
    for (k, v) in [("rows", 2.0), ("cols", 2.0), ("gap_x", 1.0), ("gap_y", 1.0)] {
        g.set_param(imagens, k, v);
    }
    if anima {
        let mexe = g.add_node("motion.wiggle");
        let fios = [(roda, mexe, 0u16), (mexe, tinta, 0)];
        for (de, para, porta) in fios {
            g.connect(Edge {
                from: (de, 0),
                to: (para, porta),
                delayed: false,
            })
            .expect("fio");
        }
    }
    let fios: &[(NodeId, NodeId, u16)] = if junta {
        &[
            (forma, dup, 0),
            (grade, dup, 1),
            (dup, roda, 0),
            (roda, tinta, 0),
            (tinta, combina, 0),
            (combina, out, 0),
        ]
    } else {
        &[
            (forma, dup, 0),
            (grade, dup, 1),
            (dup, roda, 0),
            (roda, tinta, 0),
            (tinta, out, 0),
        ]
    };
    for &(de, para, porta) in fios {
        // Com `anima` o `rotate` já vai ao `wiggle`, que vai à tinta.
        if anima && de == roda {
            continue;
        }
        liga(&mut m, de, para, porta);
    }
    m.sinks = vec![out];
    m.pump.isola_na_camada(isolada);
    (m, combina, imagens)
}

/// A marcha da CPU até `ate` — o que a bomba faz no quadro em que a ponte cai (e o que deixa a
/// memória). ⚠️ O quadro SEGUINTE é o do dispositivo (`ate + 1`): a rota híbrida coze o prefixo
/// dos tiques que a bomba ainda deve, e no mesmo tique não deve nenhum.
pub(super) fn marcha_a_cpu(m: &mut MotionState, ate: u64) {
    crate::motion_externals::publish_all(m, ate as f64 * DT);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    for tick in m.pump.last_cooked_tick().map_or(0, |t| t + 1)..=ate {
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &m.sinks,
            tick,
            |t| t as f64 * DT,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
}

/// Um quadro da ponte no tique `t`; `Handled` só conta com o cozimento VIVO.
pub(super) fn coze(m: &mut MotionState, gpu: &GpuContext, t: u64) -> bool {
    crate::motion_externals::publish_all(m, t as f64 * DT);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    matches!(cook_gpu(m, gpu, t, DT, &scopes), GpuOutcome::Handled) && m.gpu_live
}

/// A camada da placa pela rota da CPU e a pousa dela — `None` se a placa recusa (o Vello desenharia).
fn pela_cpu(
    gpu: &GpuContext,
    tag: u8,
    com: BlendWith,
    isolada: bool,
) -> Option<(Vec<u8>, ph2d_render::Pousa)> {
    let (mut cpu, _, _) = estado(tag, com, isolada, false);
    cpu.gpu_enabled = false;
    marcha_a_cpu(&mut cpu, TIQUE);
    let mut placa = PlacaDeFormas::default();
    let misturas = Misturas::do_quadro(gpu, isolada);
    placa
        .decide_com_misturas(
            true,
            &cpu.pump.vector_instances,
            &cpu.shape_store,
            &mut cpu.placa_geometrias,
            camara(),
            misturas,
        )
        .then(|| {
            placa
                .desenha(gpu, (LADO, LADO), &cpu.placa_geometrias, None)
                .expect("a placa da CPU desenha");
            (le_a_camada(gpu, &placa), placa.pousa())
        })
}

/// A camada da placa pela rota do DISPOSITIVO, pelas portas do quadro do produto (a separação das
/// imagens do grupo, a decisão e o desenho) — com as corridas e a pousa.
pub(super) fn pelo_dispositivo(
    gpu: &GpuContext,
    m: &mut MotionState,
    renderer: &ph2d_render::SpriteRenderer,
) -> (Vec<u8>, usize, ph2d_render::Pousa) {
    m.gpu_cook.separa_as_imagens_do_grupo(gpu);
    let (corridas, _) =
        corridas_do_dispositivo(m, Misturas::do_quadro(gpu, m.pump.camada_isolada()));
    let n_corridas = corridas.len();
    if let Some(i) = m.gpu_cook.instances() {
        let sprites = ph2d_gpu_cook::read_instances(gpu, i);
        assert!(
            sprites
                .iter()
                .all(|s| s.size == [0.0, 0.0] && s.opacity == 0.0),
            "uma linha do grupo no buffer das sprites misturaria em luz linear por baixo dele"
        );
    }
    let mut placa = PlacaDeFormas::default();
    assert!(decide_o_quadro(&mut placa, gpu, m, (true, true), camara()));
    desenha_o_quadro(
        &mut placa,
        gpu,
        (LADO, LADO),
        m,
        renderer,
        ph2d_render::ImageFilterMode::Smooth,
    )
    .expect("a placa do dispositivo desenha");
    (le_a_camada(gpu, &placa), n_corridas, placa.pousa())
}

/// Um renderer com o átlas de teste (as texturas das imagens da placa vêm da porta dele).
pub(super) fn renderer(gpu: &GpuContext) -> ph2d_render::SpriteRenderer {
    ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::new(gpu, 64),
        8,
    )
}

/// ⭐⭐⭐ **Cada modo × alcance × camada isolada: o dispositivo desenha o que a CPU desenha, ou
/// recusa pelo motivo dele.** Desde o P4 sem memória da CPU: o 1.º cozimento de um grupo já vai à
/// placa. Fora da camada isolada o `Everything` vai às DUAS placas e pousa com o modo (P4 (b)); o
/// `Scene` vai à da CPU e o dispositivo recusa-o pelo nome (`RECUSA_SCENE_NO_DISPOSITIVO`).
///
/// **A barra, MEDIDA** — ver a tabela no handoff B6 (`alfa`/`cor` máx. e pixels `> 16`).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_mistura_em_grupo_pelo_dispositivo_desenha_o_que_a_cpu_desenha() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let renderer = renderer(&gpu);
    let (normal, _) = pela_cpu(&gpu, 0, BlendWith::Copies, false).expect("o Normal vai à placa");
    let mut comparados = 0;
    for tag in [1u8, 3, 4] {
        for com in BlendWith::ALL {
            for isolada in [false, true] {
                let caso = format!("tag {tag} {com:?} isolada={isolada}");
                let (da_cpu, pousa_cpu) = pela_cpu(&gpu, tag, com, isolada)
                    .expect("desde o P4 a placa da CPU compõe-nas");
                let (mut m, _, _) = estado(tag, com, isolada, false);
                let em_grupo = super::sink_mistura_em_grupo(&m.doc.graph, m.sinks[0], isolada);
                let primeira = coze(&mut m, &gpu, TIQUE - 1);
                if com == BlendWith::Scene && !isolada {
                    assert!(!primeira, "{caso}");
                    assert_eq!(m.route_said, Some(RECUSA_SCENE_NO_DISPOSITIVO), "{caso}");
                    assert_ne!(
                        pousa_cpu,
                        ph2d_render::Pousa::Normal,
                        "{caso}: a CPU pousa-o"
                    );
                    continue;
                }
                assert!(
                    primeira,
                    "{caso}: o 1.º quadro já vai à placa ({:?})",
                    m.route_said
                );
                assert!(coze(&mut m, &gpu, TIQUE), "{caso}");
                let (do_dev, corridas, pousa_dev) = pelo_dispositivo(&gpu, &mut m, &renderer);
                assert_eq!(
                    corridas > 0,
                    em_grupo,
                    "{caso}: as corridas levam a mistura"
                );
                assert_eq!(pousa_dev, pousa_cpu, "{caso}: as duas rotas pousam igual");
                assert_eq!(
                    pousa_dev == ph2d_render::Pousa::Normal,
                    isolada || com == BlendWith::Copies,
                    "{caso}: fora da camada isolada o Everything pousa com o modo"
                );
                let (alfa, cor, nc, nd, fora) = compara(&da_cpu, &do_dev);
                let (_, cor_n, _, _, fora_n) = compara(&normal, &da_cpu);
                eprintln!(
                    "{caso}: alfa max {alfa} · cor max {cor} · pixels {nc}/{nd} · > 16: {fora} \
                     · pousa {pousa_dev:?} (contra o Normal: cor {cor_n}, > 16: {fora_n})"
                );
                assert!(nc > 5_000, "{caso}: a cena pinta ({nc} pixels)");
                assert!(
                    alfa <= ALFA_MAX && cor <= COR_MAX,
                    "{caso}: alfa {alfa} cor {cor}"
                );
                assert_eq!(fora, FORA_MAX, "{caso}: {fora} pixels desviam mais de 16");
                if em_grupo {
                    assert!(
                        fora_n > 1_000,
                        "{caso}: a mistura tem de se VER contra o Normal"
                    );
                }
                comparados += 1;
            }
        }
    }
    assert!(comparados >= 12, "a tabela mediu {comparados} casos");
}

/// As barras do veredito — as do irmão `formas_tests` (o MESMO passe e as MESMAS geometrias; só as
/// cópias diferem, no seno e na ordem das somas da posição).
const ALFA_MAX: u8 = 4;
const COR_MAX: u8 = 4;
const FORA_MAX: usize = 0;

/// ⭐ **O PREÇO: o cozimento na CPU contra a placa**, para um sink que mistura em grupo (`Add` ·
/// `Copies`) — sonda manual, `--release --nocapture`. O quadro inteiro de cada rota (a marcha da
/// bomba ou o cozimento do dispositivo, a decisão da placa e o desenho), com a placa esperada no
/// fim; blocos intercalados com a ordem rodada, o MÍNIMO por quadro (MEDIR_VELOCIDADE.md).
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn o_preco_da_mistura_em_grupo_cpu_contra_placa() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    println!(
        "\ncarga: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let espera = |gpu: &GpuContext| {
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    for lado in [32.0f32, 64.0, 128.0] {
        let (mut cpu, _, _) = estado_de(lado, true, 1, BlendWith::Copies, false, false);
        cpu.gpu_enabled = false;
        let (mut dev, _, _) = estado_de(lado, true, 1, BlendWith::Copies, false, false);
        marcha_a_cpu(&mut dev, 1);
        // O CONTROLO do preço do B6: o mesmo dispositivo em `Normal` — sem corridas.
        let (mut nor, _, _) = estado_de(lado, true, 0, BlendWith::Copies, false, false);
        marcha_a_cpu(&mut nor, 1);
        let mut placa_cpu = PlacaDeFormas::default();
        let mut placa_dev = PlacaDeFormas::default();
        let misturas = Misturas::do_quadro(&gpu, false);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&cpu.doc.graph, &cpu.registry);
        let mut t_cpu = 2u64;
        let mut t_dev = 2u64;
        let mut t_nor = 2u64;
        let mut placa_nor = PlacaDeFormas::default();
        let mut quadro_cpu = |cpu: &mut MotionState| {
            crate::motion_externals::publish_all(cpu, t_cpu as f64 * DT);
            cpu.pump.advance_or_scrub_scoped(
                &cpu.doc.graph,
                &cpu.registry,
                &cpu.sinks,
                t_cpu,
                |t| t as f64 * DT,
                cpu.default_uv_rect,
                cpu.default_size,
                &scopes,
            );
            t_cpu += 1;
            assert!(placa_cpu.decide_com_misturas(
                true,
                &cpu.pump.vector_instances,
                &cpu.shape_store,
                &mut cpu.placa_geometrias,
                camara(),
                misturas,
            ));
            let _ = placa_cpu.desenha(&gpu, (LADO, LADO), &cpu.placa_geometrias, None);
            espera(&gpu);
        };
        let mut quadro_dev = |dev: &mut MotionState| {
            assert!(coze(dev, &gpu, t_dev), "{:?}", dev.route_said);
            t_dev += 1;
            let (corridas, _) = corridas_do_dispositivo(dev, misturas);
            let n = dev.gpu_cook.formas().map_or(0, |f| f.len());
            assert!(placa_dev.decide_do_dispositivo(
                true,
                n,
                &dev.formas_no_dispositivo,
                &dev.shape_store,
                &mut dev.placa_geometrias,
                camara(),
                corridas,
            ));
            let buffer = dev.gpu_cook.formas().map(|f| f.buffer());
            let _ = placa_dev.desenha(&gpu, (LADO, LADO), &dev.placa_geometrias, buffer);
            espera(&gpu);
        };
        let mut quadro_nor = |nor: &mut MotionState| {
            assert!(coze(nor, &gpu, t_nor), "{:?}", nor.route_said);
            t_nor += 1;
            let n = nor.gpu_cook.formas().map_or(0, |f| f.len());
            assert!(placa_nor.decide_do_dispositivo(
                true,
                n,
                &nor.formas_no_dispositivo,
                &nor.shape_store,
                &mut nor.placa_geometrias,
                camara(),
                Vec::new(),
            ));
            let buffer = nor.gpu_cook.formas().map(|f| f.buffer());
            let _ = placa_nor.desenha(&gpu, (LADO, LADO), &nor.placa_geometrias, buffer);
            espera(&gpu);
        };
        // Aquecimento: os pipelines e os buffers de cada rota.
        for _ in 0..5 {
            quadro_cpu(&mut cpu);
            quadro_dev(&mut dev);
            quadro_nor(&mut nor);
        }
        const BLOCOS: usize = 12;
        const QUADROS: u32 = 10;
        let mut min = [f64::MAX; 3];
        for b in 0..BLOCOS {
            for k in 0..3 {
                let v = (b + k) % 3;
                let inicio = std::time::Instant::now();
                for _ in 0..QUADROS {
                    match v {
                        0 => quadro_cpu(&mut cpu),
                        1 => quadro_dev(&mut dev),
                        _ => quadro_nor(&mut nor),
                    }
                }
                min[v] = min[v].min(inicio.elapsed().as_secs_f64() * 1e3 / f64::from(QUADROS));
            }
        }
        let [min_cpu, min_dev, min_nor] = min;
        let n = (lado * lado) as u32;
        println!(
            "  {n:>6} estrelas a mexer ({}) · Add Copies: CPU {min_cpu:.3} ms/quadro · placa {min_dev:.3} ms/quadro \
             · {:.1}× (a placa em Normal, sem corridas: {min_nor:.3})",
            dev.route_said.unwrap_or(""),
            min_cpu / min_dev
        );
    }
}
