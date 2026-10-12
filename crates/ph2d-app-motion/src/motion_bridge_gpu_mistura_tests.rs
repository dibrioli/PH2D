//! ⭐⭐⭐ **A MISTURA EM GRUPO PELA ROTA DO DISPOSITIVO** (B6) — a MESMA cena de estrelas
//! SOBREPOSTAS, com o `Output` a misturar, cozida pelas duas rotas da ponte e desenhada pela placa de
//! formas: as duas camadas comparam-se pixel a pixel, em cada modo × alcance × camada isolada.
//!
//! ⚠️ As metades que a paridade sozinha não diz: o dispositivo tem de ter sido TOMADO (senão o gate
//! mede a CPU contra si mesma), as corridas têm de levar a mistura (senão o Normal das duas rotas
//! também empata — por isso a CPU é comparada também com o `Normal`, que tem de DIFERIR), e onde a
//! placa da CPU recusa (o mundo por baixo) o dispositivo recusa pelo MESMO motivo.
//!
//! `#[ignore]`: precisa de adaptador real.
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib --release \
//!        mistura_tests -- --ignored --nocapture`

use super::mistura::{
    RECUSA_GRUPO_SEM_MEMORIA, RECUSA_IMAGEM_EM_GRUPO, RECUSA_MISTURA_PEDE_O_MUNDO,
};
use super::{GpuOutcome, cook_gpu};
use crate::motion_shape_placa::gpu_tests::{LADO, camara, compara, le_a_camada};
use crate::motion_shape_placa::{Misturas, PlacaDeFormas, corridas_do_dispositivo};
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

/// A camada da placa pela rota da CPU — `None` se a placa recusa (o Vello desenharia).
fn pela_cpu(gpu: &GpuContext, tag: u8, com: BlendWith, isolada: bool) -> Option<Vec<u8>> {
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
            le_a_camada(gpu, &placa)
        })
}

/// A camada da placa pela rota do DISPOSITIVO, com as corridas por faixa.
fn pelo_dispositivo(gpu: &GpuContext, m: &mut MotionState) -> (Vec<u8>, usize) {
    let corridas = corridas_do_dispositivo(m, Misturas::do_quadro(gpu, m.pump.camada_isolada()));
    let n_corridas = corridas.len();
    let formas = m.gpu_cook.formas().expect("o cozimento escreveu as formas");
    let n = formas.len();
    let sprites = ph2d_gpu_cook::read_instances(gpu, m.gpu_cook.instances().expect("sprites"));
    assert!(
        sprites
            .iter()
            .all(|s| s.size == [0.0, 0.0] && s.opacity == 0.0),
        "uma estrela no buffer das sprites misturaria em luz linear por baixo dela"
    );
    let mut placa = PlacaDeFormas::default();
    assert!(placa.decide_do_dispositivo(
        true,
        n,
        &m.formas_no_dispositivo,
        &m.shape_store,
        &mut m.placa_geometrias,
        camara(),
        corridas,
    ));
    let buffer = m.gpu_cook.formas().map(|f| f.buffer());
    placa
        .desenha(gpu, (LADO, LADO), &m.placa_geometrias, buffer)
        .expect("a placa do dispositivo desenha");
    (le_a_camada(gpu, &placa), n_corridas)
}

/// ⭐⭐⭐ **Cada modo × alcance × camada isolada: o dispositivo desenha o que a CPU desenha, ou
/// recusa pelo MESMO motivo.** Sem memória da CPU, o 1.º cozimento de um grupo cai (a CPU coze
/// primeiro); com ela, vai à placa com as corridas da mistura.
///
/// **A barra, MEDIDA** — ver a tabela no handoff B6 (`alfa`/`cor` máx. e pixels `> 16`).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_mistura_em_grupo_pelo_dispositivo_desenha_o_que_a_cpu_desenha() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let normal = pela_cpu(&gpu, 0, BlendWith::Copies, false).expect("o Normal vai à placa");
    let mut comparados = 0;
    for tag in [1u8, 3, 4] {
        for com in BlendWith::ALL {
            for isolada in [false, true] {
                let caso = format!("tag {tag} {com:?} isolada={isolada}");
                let da_cpu = pela_cpu(&gpu, tag, com, isolada);
                let (mut m, _, _) = estado(tag, com, isolada, false);
                let em_grupo = super::sink_mistura_em_grupo(&m.doc.graph, m.sinks[0], isolada);
                let primeira = coze(&mut m, &gpu, TIQUE - 1);
                let Some(da_cpu) = da_cpu else {
                    // A placa da CPU recusa (o mundo por baixo): o dispositivo também, pelo nome.
                    assert!(!primeira, "{caso}");
                    assert_eq!(m.route_said, Some(RECUSA_MISTURA_PEDE_O_MUNDO), "{caso}");
                    continue;
                };
                if em_grupo {
                    assert!(!primeira, "{caso}: sem memória da CPU");
                    assert_eq!(m.route_said, Some(RECUSA_GRUPO_SEM_MEMORIA), "{caso}");
                    marcha_a_cpu(&mut m, TIQUE - 1);
                    assert!(
                        coze(&mut m, &gpu, TIQUE),
                        "{caso}: com a memória da CPU o grupo de formas vai à placa ({:?})",
                        m.route_said
                    );
                } else {
                    assert!(primeira, "{caso}: sem grupo ({:?})", m.route_said);
                    assert!(coze(&mut m, &gpu, TIQUE), "{caso}");
                }
                let (do_dev, corridas) = pelo_dispositivo(&gpu, &mut m);
                assert_eq!(
                    corridas > 0,
                    em_grupo,
                    "{caso}: as corridas levam a mistura"
                );
                let (alfa, cor, nc, nd, fora) = compara(&da_cpu, &do_dev);
                let (_, cor_n, _, _, fora_n) = compara(&normal, &da_cpu);
                eprintln!(
                    "{caso}: alfa max {alfa} · cor max {cor} · pixels {nc}/{nd} · > 16: {fora} \
                     (contra o Normal: cor {cor_n}, > 16: {fora_n})"
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
    assert!(comparados >= 9, "a tabela mediu {comparados} casos");
}

/// ⛔ **O CONTROLO: uma imagem no sink que mistura em grupo fica na CPU e di-lo** — sem memória,
/// com memória (a corrente não tem `geometry_id`), e quando ela APARECE depois de o grupo ter ido à
/// placa: a memória da CPU é velha, o dispositivo coze, a vigia conta-a e o quadro seguinte cai.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn uma_imagem_no_sink_que_mistura_em_grupo_fica_na_cpu_e_di_lo() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    // (1) Só imagens no grupo: a grelha no lugar das estrelas.
    let (mut m, junta, imagens) = estado(1, BlendWith::Copies, false, true);
    let _ = m.doc.graph.disconnect(junta, 0);
    liga(&mut m, imagens, junta, 0);
    assert!(!coze(&mut m, &gpu, 1));
    assert_eq!(m.route_said, Some(RECUSA_GRUPO_SEM_MEMORIA), "sem memória");
    marcha_a_cpu(&mut m, 1);
    assert!(!coze(&mut m, &gpu, 2));
    assert_eq!(
        m.route_said,
        Some(RECUSA_IMAGEM_EM_GRUPO),
        "a memória mostra as imagens"
    );

    // (2) A imagem chega DEPOIS: o grupo de formas foi à placa; liga-se a grelha ao `combine`.
    let (mut m, junta, imagens) = estado(1, BlendWith::Copies, false, true);
    assert!(!coze(&mut m, &gpu, 1), "sem memória");
    marcha_a_cpu(&mut m, 1);
    assert!(
        coze(&mut m, &gpu, 2),
        "só formas: placa ({:?})",
        m.route_said
    );
    // A leitura do quadro anterior acaba (o produto recolhe-a no seguinte), e o quadro do fio
    // encomenda a SUA — um pedido em voo de cada vez.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    liga(&mut m, imagens, junta, 1);
    assert!(
        coze(&mut m, &gpu, 3),
        "a memória da CPU é de antes do fio: o dispositivo coze, e a vigia conta ({:?})",
        m.route_said
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert!(!coze(&mut m, &gpu, 4));
    assert_eq!(
        m.route_said,
        Some(RECUSA_IMAGEM_EM_GRUPO),
        "a vigia devolve-o à CPU"
    );
    // E a CPU, agora com memória fresca, segura-o.
    marcha_a_cpu(&mut m, 4);
    assert!(!coze(&mut m, &gpu, 5));
    assert_eq!(m.route_said, Some(RECUSA_IMAGEM_EM_GRUPO));
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
        assert!(!coze(&mut dev, &gpu, 1), "sem memória");
        marcha_a_cpu(&mut dev, 1);
        // O CONTROLO do preço do B6: o mesmo dispositivo em `Normal` — sem vigia nem corridas.
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
            let corridas = corridas_do_dispositivo(dev, misturas);
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
             · {:.1}× (a placa em Normal, sem vigia nem corridas: {min_nor:.3})",
            dev.route_said.unwrap_or(""),
            min_cpu / min_dev
        );
    }
}
