//! **O que custa uma CENA de formas filtradas** — e não uma forma.
//!
//! ⚠️ **Todo número do plano 24 é de UMA pilha sobre UMA forma** (0,07 ms por degrau de borrão,
//! 0,004 ms por degrau pontual, 0,429 ms na pilha cheia). O eixo que o Enio nomeou — *"performance
//! em tempo real em runtime para games"* — é sobre uma CENA: dezenas de formas filtradas, todas
//! animando, logo todas **errando o memo** todo frame. O custo por-FORMA nunca foi medido, e é ele
//! que multiplica.
//!
//! Esta sonda encena o laço que o `fx_live::recook` roda de facto, na ordem dele:
//! por forma → rasterizar isolada num [`VelloPass`] scratch → correr a pilha para a textura de
//! saída daquela forma. O scratch é UM, partilhado; as saídas são persistentes, por forma.
//!
//! **As três perguntas, e por que são três:**
//! 1. O frame é LINEAR no número de formas, ou há custo super-linear escondido?
//! 2. O tamanho do scratch **varia por forma** (cada uma tem a sua caixa mais a margem da pilha).
//!    O [`VelloPass::ensure_size`] compara por IGUALDADE (`size == self.last_size`) ⇒ numa cena de
//!    formas de tamanhos diferentes ele **realoca a textura intermediária uma vez por forma**. O
//!    irmão dele, o `FxStackPass::ensure_work`, **CRESCE e guarda** (`>=`). Um dos dois está
//!    errado, e qual depende do preço — que é o que se mede aqui.
//! 3. Quanto do frame é o raster do Vello e quanto é a pilha? (decide onde uma wave de perf mexe)
//!
//! Rodar: `cargo test -p ph2d-render --test fx_scene_scale_cost -- --ignored --nocapture`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, VelloPass, make_output_texture};
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Point};
use vello::peniko::{Color, Fill};

mod fx_stack_common;
use fx_stack_common::{op, try_headless_gpu};

/// O formato de superfície que o `VelloPass` recebe — o scratch nunca apresenta, mas o construtor
/// dele monta o blitter, que precisa de um.
const SURFACE: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// Uma estrela preenchida que ocupa o scratch — a arte de UMA forma, isolada, como o
/// `draw_path_isolated` a entrega (transladada para a origem do scratch).
fn star(w: u32, h: u32) -> Scene {
    let (cx, cy) = (f64::from(w) * 0.5, f64::from(h) * 0.5);
    let r = f64::from(w.min(h)) * 0.42;
    let mut p = BezPath::new();
    for i in 0..10 {
        let a = std::f64::consts::TAU * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
        let rad = if i % 2 == 0 { r } else { r * 0.45 };
        let pt = Point::new(a.cos().mul_add(rad, cx), a.sin().mul_add(rad, cy));
        if i == 0 {
            p.move_to(pt);
        } else {
            p.line_to(pt);
        }
    }
    p.close_path();
    let mut scene = Scene::new();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(235, 175, 60, 255),
        None,
        &p,
    );
    scene
}

/// Os tamanhos de scratch das `n` formas de uma cena.
///
/// ⚠️ **`Varied` é o caso REAL e `Uniform` é o controle** — duas formas quaisquer de uma cena têm
/// caixas diferentes, e a margem que a pilha reserva depende dos parâmetros de cada uma. O par
/// existe para isolar o preço da REALOCAÇÃO: as duas rasterizam a mesma área total (os tamanhos
/// variados oscilam em torno de `SIDE`), então o que os separa é quantas vezes o scratch troca de
/// textura.
fn sizes(varied: bool, n: usize) -> Vec<(u32, u32)> {
    const SIDE: u32 = 256;
    const STEPS: [(u32, u32); 4] = [(232, 280), (256, 256), (280, 232), (248, 264)];
    (0..n)
        .map(|i| {
            if varied {
                STEPS[i % STEPS.len()]
            } else {
                (SIDE, SIDE)
            }
        })
        .collect()
}

/// Um frame: para cada forma, rasteriza isolada no scratch e corre a pilha para a saída dela.
/// É a ordem do `fx_live::recook_one`, sem o memo (uma cena animada erra o memo em todas).
#[allow(clippy::too_many_arguments)]
fn frame(
    gpu: &ph2d_gpu::GpuContext,
    scratch: &mut VelloPass,
    stack: &mut FxStackPass,
    outs: &[wgpu::Texture],
    art: &[Scene],
    dims: &[(u32, u32)],
    ops: &[FxOpGpu],
    run_stack: bool,
) {
    for (i, &(w, h)) in dims.iter().enumerate() {
        scratch
            .render_to_intermediate(gpu, &art[i], (w, h), Color::TRANSPARENT, false)
            .expect("scratch render");
        if run_stack {
            stack.run(
                gpu,
                scratch.intermediate_texture(),
                &outs[i],
                w,
                h,
                ops,
                &[],
            );
        }
    }
}

/// Mede um frame em ms, com aquecimento e com o fim do trabalho da GPU esperado — um `submit`
/// volta antes de a placa ter feito coisa alguma.
#[allow(clippy::too_many_arguments)]
fn measure(
    gpu: &ph2d_gpu::GpuContext,
    scratch: &mut VelloPass,
    stack: &mut FxStackPass,
    outs: &[wgpu::Texture],
    art: &[Scene],
    dims: &[(u32, u32)],
    ops: &[FxOpGpu],
    run_stack: bool,
) -> f64 {
    for _ in 0..2 {
        frame(gpu, scratch, stack, outs, art, dims, ops, run_stack);
    }
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    const N: u32 = 8;
    let t = std::time::Instant::now();
    for _ in 0..N {
        frame(gpu, scratch, stack, outs, art, dims, ops, run_stack);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    t.elapsed().as_secs_f64() * 1000.0 / f64::from(N)
}

/// Monta os recursos persistentes de uma cena de `n` formas.
fn scene_res(gpu: &ph2d_gpu::GpuContext, dims: &[(u32, u32)]) -> (Vec<wgpu::Texture>, Vec<Scene>) {
    let outs = dims
        .iter()
        .map(|&(w, h)| make_output_texture(gpu, w, h))
        .collect();
    let art = dims.iter().map(|&(w, h)| star(w, h)).collect();
    (outs, art)
}

/// **A arte de `n` formas num ATLAS** — as mesmas estrelas, cada uma na sua célula, numa CENA só.
///
/// É o outro lado da pergunta que a tabela acima abriu: o raster é 63 % do frame, e a sonda mede
/// `n` renders do Vello. Um render do Vello tem custo FIXO (ele corre a cadeia de binning/tiling
/// inteira, mais um `submit`, seja qual for o conteúdo) — então *`n` renders de uma forma* e
/// *um render de `n` formas* percorrem a MESMA área e diferem exactamente por esse fixo, vezes
/// `n − 1`. A área total é idêntica de propósito: é o fixo que se quer isolar, não o preenchimento.
fn atlas(n: usize, cell: u32, cols: u32) -> (Scene, u32, u32) {
    let rows = (n as u32).div_ceil(cols);
    let (w, h) = (cols * cell, rows * cell);
    let one = star(cell, cell);
    let mut scene = Scene::new();
    for i in 0..n as u32 {
        let (cx, cy) = ((i % cols) * cell, (i / cols) * cell);
        scene.append(
            &one,
            Some(Affine::translate((f64::from(cx), f64::from(cy)))),
        );
    }
    (scene, w, h)
}

/// **O custo FIXO de um render do Vello** — o número que decide se vale a pena juntar as formas
/// filtradas numa passagem só.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn measure_what_a_vello_render_costs_before_it_draws_anything() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-fixo] sem adapter — skip");
        return;
    };
    const CELL: u32 = 256;
    let mut scratch = VelloPass::new(&gpu, SURFACE, (CELL, CELL)).expect("scratch");
    let mut stack = FxStackPass::new(&gpu);
    let outs: Vec<wgpu::Texture> = Vec::new();

    eprintln!("[fx-fixo] a MESMA area de arte, em N renders contra UM (ms):");
    eprintln!("[fx-fixo]   N | N renders | 1 render (atlas) | fixo por render");
    for n in [1usize, 4, 16, 32] {
        let dims = sizes(false, n);
        let (_o, art) = scene_res(&gpu, &dims);
        let many = measure(
            &gpu,
            &mut scratch,
            &mut stack,
            &outs,
            &art,
            &dims,
            &[],
            false,
        );
        let (big, aw, ah) = atlas(n, CELL, 8);
        let one = measure(
            &gpu,
            &mut scratch,
            &mut stack,
            &outs,
            std::slice::from_ref(&big),
            &[(aw, ah)],
            &[],
            false,
        );
        eprintln!(
            "[fx-fixo] {n:3} | {many:9.3} | {one:16.3} | {:15.3}",
            (many - one) / (n.max(2) - 1) as f64
        );
    }
}

/// **O custo FIXO de uma corrida da PILHA** — o segundo eixo, e ele existe pela mesma razão do
/// primeiro: o `FxStackPass::run` termina em `queue.submit`, logo uma cena de `n` formas paga `n`
/// submissões também.
///
/// A medição isola o fixo pondo o degrau mais BARATO que existe (um pontual, ~0,004 ms a 512²,
/// logo ~0,001 a 256²) — com o ingest e o resolve, o trabalho real de uma corrida é ~0,003 ms.
/// Tudo acima disso é encode + bind groups + submissão.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn measure_what_a_stack_run_costs_before_it_filters_anything() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-pilha] sem adapter — skip");
        return;
    };
    const CELL: u32 = 256;
    let dims = sizes(false, 32);
    let (outs, _art) = scene_res(&gpu, &dims);
    let src = make_output_texture(&gpu, CELL, CELL);
    let mut stack = FxStackPass::new(&gpu);
    let cheap = [op(FxOp::COLOR_OVERLAY, 0.0, [0.2, 0.5, 1.0, 1.0])];

    let time = |stack: &mut FxStackPass, n: usize| -> f64 {
        for _ in 0..2 {
            for o in outs.iter().take(n) {
                stack.run(&gpu, &src, o, CELL, CELL, &cheap, &[]);
            }
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        const R: u32 = 8;
        let t = std::time::Instant::now();
        for _ in 0..R {
            for o in outs.iter().take(n) {
                stack.run(&gpu, &src, o, CELL, CELL, &cheap, &[]);
            }
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        t.elapsed().as_secs_f64() * 1000.0 / f64::from(R)
    };
    eprintln!("[fx-pilha] N corridas do degrau mais barato que existe (ms):");
    for n in [1usize, 4, 16, 32] {
        let ms = time(&mut stack, n);
        eprintln!(
            "[fx-pilha] {n:3} | {ms:7.3} | por corrida {:6.3}",
            ms / n as f64
        );
    }
}

/// Um frame do desenho NOVO: UM render do atlas, e depois a pilha de cada forma lendo a célula
/// dela. A grade é a que o empacotador da shell produz para células de tamanho igual.
#[allow(clippy::too_many_arguments)]
fn frame_batched(
    gpu: &ph2d_gpu::GpuContext,
    scratch: &mut VelloPass,
    stack: &mut FxStackPass,
    outs: &[wgpu::Texture],
    art: &Scene,
    aw: u32,
    ah: u32,
    cell: u32,
    cols: u32,
    ops: &[FxOpGpu],
) {
    scratch
        .render_to_intermediate(gpu, art, (aw, ah), Color::TRANSPARENT, false)
        .expect("atlas render");
    let src = scratch.intermediate_texture();
    for (i, out) in outs.iter().enumerate() {
        let i = i as u32;
        let org = [
            i32::try_from((i % cols) * cell).expect("cabe"),
            i32::try_from((i / cols) * cell).expect("cabe"),
        ];
        stack.run_from(gpu, src, org, out, cell, cell, ops, &[]);
    }
}

/// **O GANHO, medido ponta a ponta** — o mesmo trabalho pelo caminho de ontem e pelo de hoje.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn measure_what_the_atlas_saves() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-ganho] sem adapter — skip");
        return;
    };
    const CELL: u32 = 256;
    const COLS: u32 = 8;
    let ops = [
        FxOpGpu {
            offset_px: [6, 6],
            ..op(FxOp::DROP_SHADOW, 6.0, [0.0, 0.0, 0.0, 0.6])
        },
        op(FxOp::COLOR_OVERLAY, 0.0, [0.2, 0.5, 1.0, 1.0]),
    ];
    let mut scratch = VelloPass::new(&gpu, SURFACE, (CELL, CELL)).expect("scratch");
    let mut stack = FxStackPass::new(&gpu);

    eprintln!("[fx-ganho] o MESMO frame, pelos dois caminhos (ms):");
    eprintln!("[fx-ganho]   N | N renders (ontem) | 1 render (hoje) | ganho");
    for n in [1usize, 4, 16, 32] {
        let dims = sizes(false, n);
        let (outs, art) = scene_res(&gpu, &dims);
        let before = measure(
            &gpu,
            &mut scratch,
            &mut stack,
            &outs,
            &art,
            &dims,
            &ops,
            true,
        );
        let (big, aw, ah) = atlas(n, CELL, COLS);
        for _ in 0..2 {
            frame_batched(
                &gpu,
                &mut scratch,
                &mut stack,
                &outs,
                &big,
                aw,
                ah,
                CELL,
                COLS,
                &ops,
            );
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        const R: u32 = 8;
        let t = std::time::Instant::now();
        for _ in 0..R {
            frame_batched(
                &gpu,
                &mut scratch,
                &mut stack,
                &outs,
                &big,
                aw,
                ah,
                CELL,
                COLS,
                &ops,
            );
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        let after = t.elapsed().as_secs_f64() * 1000.0 / f64::from(R);
        eprintln!(
            "[fx-ganho] {n:3} | {before:17.3} | {after:15.3} | {:5.2}x",
            before / after.max(1e-6)
        );
    }
}

/// **A sonda.** Não afirma nada: imprime a tabela que decide se há wave de perf, e qual.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn measure_what_a_scene_of_filtered_shapes_costs() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-cena] sem adapter — skip");
        return;
    };
    // Uma pilha típica de artista: uma sombra (o degrau caro, com kernel) e um Color Overlay (o
    // pontual). Não é a pilha cheia — o que se mede aqui é o custo POR FORMA, e a pilha cheia já
    // tem número próprio.
    let ops = [
        FxOpGpu {
            offset_px: [6, 6],
            ..op(FxOp::DROP_SHADOW, 6.0, [0.0, 0.0, 0.0, 0.6])
        },
        op(FxOp::COLOR_OVERLAY, 0.0, [0.2, 0.5, 1.0, 1.0]),
    ];
    let mut scratch = VelloPass::new(&gpu, SURFACE, (256, 256)).expect("scratch");
    let mut stack = FxStackPass::new(&gpu);

    eprintln!("[fx-cena] o frame de N formas filtradas, todas errando o memo (ms):");
    eprintln!("[fx-cena]   N | uniforme | variado | so o raster (variado) | por forma (variado)");
    for n in [1usize, 4, 16, 32] {
        let du = sizes(false, n);
        let dv = sizes(true, n);
        let (ou, au) = scene_res(&gpu, &du);
        let (ov, av) = scene_res(&gpu, &dv);
        let uni = measure(&gpu, &mut scratch, &mut stack, &ou, &au, &du, &ops, true);
        let var = measure(&gpu, &mut scratch, &mut stack, &ov, &av, &dv, &ops, true);
        let raw = measure(&gpu, &mut scratch, &mut stack, &ov, &av, &dv, &ops, false);
        eprintln!(
            "[fx-cena] {n:3} | {uni:8.3} | {var:7.3} | {raw:21.3} | {:18.3}",
            var / n as f64
        );
    }
}

/// **De que é feito o PRIMEIRO frame com FX** — o número que o smoke mostra uma vez e nunca mais.
///
/// ⚠️ Existe porque um custo de UMA VEZ é invisível em toda a tabela acima: elas aquecem antes de
/// medir, de propósito. O log do produto traz `recook 11,8 ms` no frame em que a cena arma os
/// filtros e `0,012 ms` quando nada muda — e a diferença tem de ser ATRIBUÍDA, não explicada.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn measure_what_the_first_fx_frame_pays_once() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-1o] sem adapter — skip");
        return;
    };
    const CELL: u32 = 256;
    let t = std::time::Instant::now();
    let mut scratch = VelloPass::new(&gpu, SURFACE, (CELL, CELL)).expect("scratch");
    let vello_new = t.elapsed().as_secs_f64() * 1000.0;

    let t = std::time::Instant::now();
    let mut stack = FxStackPass::new(&gpu);
    let stack_new = t.elapsed().as_secs_f64() * 1000.0;

    // As 15 texturas de saída de uma cena como a do smoke `=33`.
    let dims = sizes(false, 15);
    let t = std::time::Instant::now();
    let (outs, art) = scene_res(&gpu, &dims);
    let allocs = t.elapsed().as_secs_f64() * 1000.0;

    // O 1º frame de verdade: um render do atlas + as 15 pilhas, SEM aquecimento.
    let ops = [op(FxOp::BLUR, 6.0, [0.0, 0.0, 0.0, 1.0])];
    let (big, aw, ah) = atlas(15, CELL, 8);
    let t = std::time::Instant::now();
    frame_batched(
        &gpu,
        &mut scratch,
        &mut stack,
        &outs,
        &big,
        aw,
        ah,
        CELL,
        8,
        &ops,
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let first = t.elapsed().as_secs_f64() * 1000.0;

    // E o MESMO frame outra vez, agora quente.
    let t = std::time::Instant::now();
    frame_batched(
        &gpu,
        &mut scratch,
        &mut stack,
        &outs,
        &big,
        aw,
        ah,
        CELL,
        8,
        &ops,
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let warm = t.elapsed().as_secs_f64() * 1000.0;

    let _ = art;
    eprintln!("[fx-1o] `VelloPass::new` (o renderer scratch)  {vello_new:8.3} ms");
    eprintln!("[fx-1o] `FxStackPass::new` (os pipelines)      {stack_new:8.3} ms");
    eprintln!("[fx-1o] 15 texturas de saida                   {allocs:8.3} ms");
    eprintln!("[fx-1o] o 1o frame (atlas + 15 pilhas), frio   {first:8.3} ms");
    eprintln!("[fx-1o] o MESMO frame, quente                  {warm:8.3} ms");
    eprintln!(
        "[fx-1o] TOTAL de uma vez: {:.3} ms",
        vello_new + stack_new + allocs + (first - warm)
    );
}
