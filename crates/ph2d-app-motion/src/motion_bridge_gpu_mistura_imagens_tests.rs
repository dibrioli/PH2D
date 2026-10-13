//! ⭐⭐⭐ **AS IMAGENS DE UM GRUPO NA PLACA** (P4, doc 132 §4) — uma saída que mistura em grupo com
//! linhas de IMAGEM, cozida pelas duas rotas da ponte: a da CPU desenha-a na cena vectorial (o Vello,
//! `motion_shape_gen::encode` com a arte recortada do átlas, como o produto), a do dispositivo na placa
//! de formas (`motion_shape_placa::imagens`). As duas comparam-se pixel a pixel.
//!
//! ⚠️ As metades que a paridade sozinha não diz: o dispositivo tem de ter sido TOMADO (a rota diz
//! `device`, e as linhas chegaram ao buffer da placa), a mistura tem de se VER (contra a mesma cena em
//! `Normal`), e a fixtura tem de conter o que se mede — arte com cor a variar nos dois eixos (o
//! bilinear), um anel de MEIA alfa (a borda que a ordem ingénua da tinta vaza, doc 118 §7) e um canto
//! transparente; rotação; uma tinta de COR; cópias que se sobrepõem.
//!
//! `#[ignore]`: precisa de adaptador real.
//!   `PH2D_GPU=1 PH2D_GPU_ESPERA=3600 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib \
//!        --release mistura_imagens_tests -- --ignored --nocapture`

use super::mistura_tests::{coze, estado_de, marcha_a_cpu, pelo_dispositivo};
use crate::motion_shape_placa::gpu_tests::{LADO, camara, compara, fotografa};
use crate::motion_state::MotionState;
use ph2d_eval_motion::BlendWith;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::{Edge, NodeId};

const TIQUE: u64 = 2;
const DT: f64 = 1.0 / 60.0;
const CHAVE: u32 = 7;
const LADO_DA_ARTE: u32 = 16;

/// A arte `16×16` em alfa recta: cor a variar nos dois eixos, um anel de dois texels a MEIA alfa e um
/// canto `4×4` transparente.
fn arte() -> Vec<u8> {
    let mut v = Vec::with_capacity((LADO_DA_ARTE * LADO_DA_ARTE * 4) as usize);
    for y in 0..LADO_DA_ARTE {
        for x in 0..LADO_DA_ARTE {
            let anel = x < 2 || y < 2 || x >= LADO_DA_ARTE - 2 || y >= LADO_DA_ARTE - 2;
            let a = if x < 4 && y < 4 {
                0
            } else if anel {
                128
            } else {
                255
            };
            #[expect(clippy::cast_possible_truncation, reason = "bytes da fixtura")]
            v.extend_from_slice(&[
                (x * 15 + 10) as u8,
                (y * 15 + 10) as u8,
                (90 + (x + y) * 4) as u8,
                a,
            ]);
        }
    }
    v
}

/// O átlas de teste: o renderer (a porta das texturas da placa), o rectângulo da arte e os bytes do
/// nível 0 (a porta da arte da CPU, `motion_leaf_images::crop`).
pub(super) struct Atlas {
    pub(super) renderer: ph2d_render::SpriteRenderer,
    pub(super) uv: [f32; 4],
    lado: u32,
    px: Vec<u8>,
}

pub(super) fn atlas(gpu: &GpuContext) -> Atlas {
    let mut a = ph2d_render::TextureAtlas::new(gpu, 64);
    a.insert(gpu, CHAVE, LADO_DA_ARTE, LADO_DA_ARTE, &arte())
        .expect("a arte cabe no átlas");
    let renderer = ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, a, 8);
    let uv = renderer.atlas().region_uv(CHAVE);
    let (lado, _, px) = renderer.atlas().readback_mip(gpu, 0);
    Atlas {
        renderer,
        uv,
        lado,
        px,
    }
}

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

/// `grelha lado×lado de IMAGENS → rotate → tint [→ wiggle] → Output(tag, com)`, as imagens com a arte
/// do átlas e `0,9` de lado — SOBREPOSTAS (o passo é `0,6`).
pub(super) fn so_imagens(
    lado: f32,
    anima: bool,
    tag: u8,
    com: BlendWith,
    isolada: bool,
    uv: [f32; 4],
) -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grade = g.add_node("motion.grid");
    for (k, v) in [
        ("rows", lado),
        ("cols", lado),
        ("gap_x", 0.6),
        ("gap_y", 0.6),
    ] {
        g.set_param(grade, k, v);
    }
    let roda = g.add_node("motion.rotate");
    g.set_param(roda, "angle", 17.0);
    let tinta = g.add_node("motion.tint");
    for (k, v) in [("r", 0.9), ("g", 0.55), ("b", 0.35), ("a", 0.85)] {
        g.set_param(tinta, k, v);
    }
    let out = g.add_node("motion.output");
    g.set_param(out, ph2d_eval_motion::SINK_BLEND_PARAM, f32::from(tag));
    let alcance = BlendWith::ALL.iter().position(|c| *c == com).unwrap_or(0);
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(out, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, alcance as f32);
    liga(&mut m, grade, roda, 0);
    liga(&mut m, roda, tinta, 0);
    if anima {
        let mexe = m.doc.graph.add_node("motion.wiggle");
        liga(&mut m, tinta, mexe, 0);
        liga(&mut m, mexe, out, 0);
    } else {
        liga(&mut m, tinta, out, 0);
    }
    m.sinks = vec![out];
    m.pump.isola_na_camada(isolada);
    // ⚠️ A lei do dono cala uma corrente de POSIÇÕES sem aparência (`tem_aparencia`): aqui a imagem
    // é o ladrilho de omissão, e o gate mede o passe das IMAGENS, não a lei (que tem os gates dela).
    m.pump.define_a_lei(false);
    m.default_uv_rect = uv;
    m.default_size = [0.9, 0.9];
    m
}

/// O ramo das IMAGENS de uma cena com formas: `grelha → scale → tint`, pronto a ligar ao `combine`.
/// ⚠️ O `combine` enche de ZEROS a coluna que só uma entrada tem — sem o `scale` e o `tint` a imagem
/// chegaria com tamanho e alfa `0` (as estrelas trazem as duas colunas).
fn ramo_de_imagens(m: &mut MotionState, grade: NodeId, lado: f32) -> NodeId {
    let g = &mut m.doc.graph;
    let escala = g.add_node("motion.scale");
    g.set_param(escala, "amount", lado);
    let tinta = g.add_node("motion.tint");
    for (k, v) in [("r", 0.95), ("g", 0.6), ("b", 0.3), ("a", 0.9)] {
        g.set_param(tinta, k, v);
    }
    liga(m, grade, escala, 0);
    liga(m, escala, tinta, 0);
    tinta
}

/// A camada da rota da CPU: a cena vectorial do produto (`encode`, a arte pela porta do produto),
/// rasterizada pelo `VelloPass` do produto.
fn pelo_vello(gpu: &GpuContext, m: &MotionState, a: &Atlas) -> Vec<u8> {
    let mut cena = ph2d_vector::VectorScene::new();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    crate::motion_shape_gen::encode(
        &m.pump.vector_instances,
        &m.shape_store,
        &mut |_, uv| crate::motion_leaf_images::crop(a.lado, a.lado, &a.px, uv),
        camara(),
        Some(janela),
        ph2d_render::ImageFilterMode::Smooth,
        &mut cena,
    );
    let mut vp =
        ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("o VelloPass do produto nasce");
    vp.render_and_readback(gpu, cena.inner(), (LADO, LADO))
        .expect("o Vello desenha")
}

/// A CPU marcha até [`TIQUE`] e o Vello desenha.
fn pela_cpu(gpu: &GpuContext, mut m: MotionState, a: &Atlas) -> Vec<u8> {
    m.gpu_enabled = false;
    marcha_a_cpu(&mut m, TIQUE);
    assert!(
        m.pump
            .vector_instances
            .iter()
            .any(|i| i.geometry_id == 0 && i.mistura.tem_camada()),
        "a fixtura tem de levar imagens ao grupo da cena vectorial ({} vectoriais, {} sprites, {:?})",
        m.pump.vector_instances.len(),
        m.pump.instances.len(),
        m.pump.last_error()
    );
    pelo_vello(gpu, &m, a)
}

/// O dispositivo coze dois tiques e a placa desenha pelas portas do quadro.
fn pela_placa(gpu: &GpuContext, m: &mut MotionState, a: &Atlas) -> Vec<u8> {
    assert!(
        coze(m, gpu, TIQUE - 1),
        "o 1.º quadro vai à placa ({:?})",
        m.route_said
    );
    assert!(coze(m, gpu, TIQUE), "{:?}", m.route_said);
    let (camada, corridas, _) = pelo_dispositivo(gpu, m, &a.renderer);
    let imagens = m.gpu_cook.imagens_do_grupo().map_or(0, |i| i.len());
    assert!(imagens > 0, "as imagens do grupo chegaram à placa");
    assert!(corridas > 0, "as corridas levam a mistura e as imagens");
    camada
}

/// As barras: as do irmão das formas (`mistura_tests`, a mesma barra de arredondamento).
const ALFA_MAX: u8 = 4;
const COR_MAX: u8 = 4;
const FORA_MAX: usize = 0;

/// ⭐⭐⭐ **A imagem de um grupo pelo dispositivo desenha o que a CPU desenha** — cada modo × os
/// alcances que a placa compõe (o `Copies` dentro e fora da camada isolada, o `Everything` dentro):
/// imagens SÓ, rodadas, tintadas a cor, sobrepostas. A barra é a das formas (`4 · 4 · 0`).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_imagem_de_um_grupo_pelo_dispositivo_desenha_o_que_a_cpu_desenha() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let a = atlas(&gpu);
    let mut comparados = 0;
    for tag in [1u8, 3, 4] {
        for (com, isolada) in [
            (BlendWith::Copies, false),
            (BlendWith::Copies, true),
            (BlendWith::Everything, true),
        ] {
            let caso = format!("tag {tag} {com:?} isolada={isolada}");
            let da_cpu = pela_cpu(&gpu, so_imagens(3.0, false, tag, com, isolada, a.uv), &a);
            // O CONTROLO: a mesma cena noutro modo tem de DIFERIR (a fixtura mostra a mistura).
            let outro = if tag == 1 { 4 } else { 1 };
            let controlo = pela_cpu(&gpu, so_imagens(3.0, false, outro, com, isolada, a.uv), &a);
            let mut m = so_imagens(3.0, false, tag, com, isolada, a.uv);
            let do_dev = pela_placa(&gpu, &mut m, &a);
            assert!(
                m.route_said.is_some_and(|r| r.starts_with("device")),
                "{caso}: {:?}",
                m.route_said
            );
            let (alfa, cor, nc, nd, fora) = compara(&da_cpu, &do_dev);
            let (_, cor_n, _, _, fora_n) = compara(&controlo, &da_cpu);
            fotografa(
                &format!("imagens_{tag}_{com:?}_{isolada}"),
                &da_cpu,
                &do_dev,
            );
            eprintln!(
                "{caso}: alfa max {alfa} · cor max {cor} · pixels {nc}/{nd} · > 16: {fora} \
                 (contra o modo {outro}: cor {cor_n}, > 16: {fora_n})"
            );
            assert!(nc > 3_000, "{caso}: a cena pinta ({nc} pixels)");
            assert!(
                alfa <= ALFA_MAX && cor <= COR_MAX,
                "{caso}: alfa {alfa} cor {cor}"
            );
            assert_eq!(fora, FORA_MAX, "{caso}: {fora} pixels desviam mais de 16");
            assert!(
                fora_n > 500,
                "{caso}: a mistura tem de se VER contra outro modo"
            );
            comparados += 1;
        }
    }
    assert_eq!(comparados, 9);
}

/// ⭐⭐ **Imagens E formas no MESMO grupo** (a cena do smoke `=140`): a placa desenha as imagens da
/// corrida depois das formas dela — a mesma cor, porque a mistura de grupo é comutativa. A barra é a
/// das FORMAS contra o Vello (`a_rota_da_placa_desenha_o_que_a_cena_vello_desenha`: `100 · 100`, `5 %`
/// dos pixels — o quarto de pixel da borda curva das estrelas); a das imagens sozinhas é a de cima.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn imagens_e_formas_no_mesmo_grupo_pelo_dispositivo() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let a = atlas(&gpu);
    for tag in [1u8, 4] {
        let mista = |a: &Atlas| {
            let (mut m, combina, imagens) =
                estado_de(4.0, false, tag, BlendWith::Copies, false, true);
            let ramo = ramo_de_imagens(&mut m, imagens, 1.6);
            liga(&mut m, ramo, combina, 1);
            m.default_uv_rect = a.uv;
            m
        };
        let da_cpu = pela_cpu(&gpu, mista(&a), &a);
        // O CONTROLO: sem o ramo das imagens a cena tem de DIFERIR (as imagens pintam).
        let (mut so_formas, _, _) = estado_de(4.0, false, tag, BlendWith::Copies, false, true);
        so_formas.default_uv_rect = a.uv;
        so_formas.gpu_enabled = false;
        marcha_a_cpu(&mut so_formas, TIQUE);
        let sem_imagens = pelo_vello(&gpu, &so_formas, &a);
        let (_, _, _, _, fora_i) = compara(&sem_imagens, &da_cpu);
        assert!(fora_i > 500, "as imagens têm de se VER ({fora_i} pixels)");
        let mut m = mista(&a);
        let do_dev = pela_placa(&gpu, &mut m, &a);
        let (alfa, cor, nc, nd, fora) = compara(&da_cpu, &do_dev);
        fotografa(&format!("mista_{tag}"), &da_cpu, &do_dev);
        eprintln!("tag {tag}: alfa max {alfa} · cor max {cor} · pixels {nc}/{nd} · > 16: {fora}");
        assert!(nc > 10_000, "a cena pinta ({nc} pixels)");
        assert!(alfa <= 100 && cor <= 100, "alfa {alfa} cor {cor}");
        assert!(
            fora * 20 <= nc,
            "{fora} pixels desviam > 16 — acima de 5 % dos {nc}"
        );
    }
}

/// Uma cópia, NA FILA, do buffer inteiro — lida depois, sem parar o quadro (a régua não pode esperar
/// pela placa entre quadros: o produto não espera).
fn fotografia(gpu: &GpuContext, b: &wgpu::Buffer, n: u32) -> Option<(wgpu::Buffer, u32)> {
    if n == 0 {
        return None;
    }
    let bytes = u64::from(n) * std::mem::size_of::<ph2d_render::RenderInstance>() as u64;
    let s = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test fotografia do quadro"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_buffer_to_buffer(b, 0, &s, 0, bytes);
    gpu.queue.submit(Some(enc.finish()));
    Some((s, n))
}

/// As linhas VISÍVEIS de uma fotografia (tamanho e opacidade a não zero).
fn visiveis(gpu: &GpuContext, f: Option<&(wgpu::Buffer, u32)>) -> usize {
    let Some((s, _)) = f else { return 0 };
    s.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let linhas: Vec<ph2d_render::RenderInstance> =
        bytemuck::cast_slice(&s.slice(..).get_mapped_range()).to_vec();
    linhas
        .iter()
        .filter(|r| r.size != [0.0, 0.0] && r.opacity > 0.0)
        .count()
}

/// ⭐⭐⭐ **(c) A JANELA: nenhum quadro desenha a imagem de um grupo pelas sprites** — a porta do
/// quadro do PRODUTO (`coze_o_quadro`, com o `Poll` dele e NENHUMA espera entre quadros: as leituras da
/// régua são cópias na fila, mapeadas no fim). Três quadros só de formas no grupo; liga-se a grelha de
/// imagens ao `combine`; seis quadros. Em CADA um: a rota é o dispositivo, o buffer das sprites não
/// tem linha visível, e o da placa tem as quatro imagens.
///
/// Antes (B6): a imagem desenhava-se pelas sprites no quadro do fio e no seguinte — a vigia lia o
/// contador no quadro N+1 (ou N+2) e só então devolvia o grupo à CPU.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn nenhum_quadro_desenha_a_imagem_do_grupo_pelas_sprites() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    let a = atlas(&gpu);
    let (mut m, combina, imagens) = estado_de(4.0, false, 1, BlendWith::Copies, false, true);
    let ramo = ramo_de_imagens(&mut m, imagens, 1.2);
    m.default_uv_rect = a.uv;
    let mut playhead = ph2d_core::Playhead::new(DT);
    let mut quadros = Vec::new();
    for q in 1..=9u32 {
        if q == 4 {
            liga(&mut m, ramo, combina, 1);
        }
        playhead.seek(f64::from(q) * DT);
        crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, DT);
        let sprites = m
            .gpu_cook
            .instances()
            .and_then(|i| fotografia(&gpu, i.buffer(), i.len()));
        let placa = m
            .gpu_cook
            .imagens_do_grupo()
            .and_then(|i| fotografia(&gpu, i.buffer(), i.len()));
        quadros.push((q, m.route_said, m.gpu_live, sprites, placa));
    }
    let mut pelas_sprites = 0;
    for (q, rota, vivo, sprites, placa) in &quadros {
        let (s, p) = (
            visiveis(&gpu, sprites.as_ref()),
            visiveis(&gpu, placa.as_ref()),
        );
        eprintln!("quadro {q}: {rota:?} · sprites visíveis {s} · imagens na placa {p}");
        assert!(
            *vivo && rota.is_some_and(|r| r.starts_with("device")),
            "quadro {q}: a rota é o dispositivo ({rota:?})"
        );
        pelas_sprites += usize::from(s > 0);
        assert_eq!(
            p,
            if *q >= 4 { 4 } else { 0 },
            "quadro {q}: as imagens da placa"
        );
    }
    assert_eq!(
        pelas_sprites, 0,
        "quadros com a imagem do grupo pelas sprites"
    );
}

/// ⭐ **O PREÇO das imagens de um grupo: a CPU (a bomba e o Vello) contra a placa** (`Add · Copies`,
/// a grelha a mexer com `wiggle`) — sonda manual, `--release --nocapture`. O quadro inteiro de cada
/// rota com a placa esperada no fim; blocos intercalados com a ordem rodada, o MÍNIMO por quadro, a
/// carga ao lado (MEDIR_VELOCIDADE.md).
#[test]
#[ignore = "sonda: manual — --release --nocapture, com placa"]
fn o_preco_das_imagens_de_um_grupo_cpu_contra_placa() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        return;
    };
    let carga = || {
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    println!("\ncarga: {}", carga());
    let espera = |gpu: &GpuContext| {
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    let a = atlas(&gpu);
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    let mut vp =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("vello");
    for lado in [32.0f32, 64.0, 128.0] {
        let mut cpu = so_imagens(lado, true, 1, BlendWith::Copies, false, a.uv);
        cpu.gpu_enabled = false;
        let mut dev = so_imagens(lado, true, 1, BlendWith::Copies, false, a.uv);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&cpu.doc.graph, &cpu.registry);
        let (mut t_cpu, mut t_dev) = (1u64, 1u64);
        let mut placa = crate::motion_shape_placa::PlacaDeFormas::default();
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
            let mut cena = ph2d_vector::VectorScene::new();
            crate::motion_shape_gen::encode(
                &cpu.pump.vector_instances,
                &cpu.shape_store,
                &mut |_, uv| crate::motion_leaf_images::crop(a.lado, a.lado, &a.px, uv),
                camara(),
                Some(janela),
                ph2d_render::ImageFilterMode::Smooth,
                &mut cena,
            );
            vp.render_to_intermediate(
                &gpu,
                cena.inner(),
                (LADO, LADO),
                ph2d_vector::Color::TRANSPARENT,
            )
            .expect("vello");
            espera(&gpu);
        };
        let mut quadro_dev = |dev: &mut MotionState| {
            assert!(coze(dev, &gpu, t_dev), "{:?}", dev.route_said);
            t_dev += 1;
            dev.gpu_cook.separa_as_imagens_do_grupo(&gpu);
            assert!(crate::motion_shape_placa::decide_o_quadro(
                &mut placa,
                &gpu,
                dev,
                (true, true),
                camara(),
            ));
            let _ = crate::motion_shape_placa::desenha_o_quadro(
                &mut placa,
                &gpu,
                (LADO, LADO),
                dev,
                &a.renderer,
                ph2d_render::ImageFilterMode::Smooth,
            );
            espera(&gpu);
        };
        for _ in 0..5 {
            quadro_cpu(&mut cpu);
            quadro_dev(&mut dev);
        }
        const BLOCOS: usize = 12;
        const QUADROS: u32 = 10;
        let mut min = [f64::MAX; 2];
        for b in 0..BLOCOS {
            for k in 0..2 {
                let v = (b + k) % 2;
                let inicio = std::time::Instant::now();
                for _ in 0..QUADROS {
                    if v == 0 {
                        quadro_cpu(&mut cpu);
                    } else {
                        quadro_dev(&mut dev);
                    }
                }
                min[v] = min[v].min(inicio.elapsed().as_secs_f64() * 1e3 / f64::from(QUADROS));
            }
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "lado inteiro"
        )]
        let n = (lado * lado) as u32;
        println!(
            "  {n:>6} imagens a mexer ({}) · Add Copies: CPU (bomba + Vello) {:.3} ms/quadro · placa {:.3} ms/quadro · {:.1}× · carga {}",
            dev.route_said.unwrap_or(""),
            min[0],
            min[1],
            min[0] / min[1],
            carga()
        );
    }
}
