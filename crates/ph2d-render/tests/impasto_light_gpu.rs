//! Impasto light pass — GPU/CPU parity gates.
//!
//! `#[ignore]`d: they need a real device, so they run on a developer machine / the GPU CI lane, not the
//! headless CPU runners (which `None` out of `try_headless_gpu`).
//!
//! They reconcile [`ph2d_render::ImpastoLightPass`] against **the canonical CPU pass itself** —
//! `PainterTool::apply_impasto_light` — and not against a re-implementation of it. An oracle that
//! re-derives the thing it is testing agrees with the bug; this one asks the shipped pass what it draws.
//!
//! ## What "parity" means here, and what it deliberately does not
//!
//! Runtime output is **not** bit-identical across backends, and chasing that would be chasing a ghost:
//! `sqrt` and the four arithmetic ops are correctly rounded in both IEEE and WGSL, but a backend is free
//! to contract `a * b + c` into an FMA — which is *more* accurate, and therefore different. That is the
//! same caveat the layer compositor carries, and the same reason its Bloom gate accepts 5 bytes and its
//! Shadows/Highlights gate 4.
//!
//! So the exactness is placed where exactness is actually available:
//!
//! - the **literals** are pinned bit-identical by a CPU-only gate in `impasto_light.rs` (no device, runs
//!   everywhere);
//! - the specular LUT is **uploaded**, not recomputed, so the one transcendental in the model (`powf`)
//!   never runs on the device at all and cannot diverge;
//! - the store is quantised **explicitly** in the shader, so Rust's round-half-away and WGSL's freedom to
//!   round half-to-even do not cost a byte on every .5 boundary;
//! - the two **structural** contracts are `assert_eq!`, because they are early-outs and exactly
//!   reproducible: flat paint comes back byte-identical, bare paper is untouched.
//!
//! Run with:
//!   cargo test -p ph2d-render --test impasto_light_gpu -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_render::layer_compositor::Region as GpuRegion;
use ph2d_render::{ImpastoLamp as GpuLamp, ImpastoLightInput, ImpastoLightPass};
use ph2d_tool_painter::{ImpastoPlanes, PainterTool, Region};

/// 64 px: `64 * 4 = 256` bytes, exactly the readback row alignment, so the staging copy is a plain memcpy
/// and the gate is about shading rather than about padding arithmetic.
const W: u32 = 64;
const H: u32 = 64;

/// The largest per-channel byte difference this gate accepts between the CPU pass and the shader.
///
/// **Two, not zero, and not five.** Zero is unreachable in principle for the reason in the module docs
/// (FMA contraction is a backend's prerogative), even though this backend in fact delivers it. Five would
/// be slack borrowed from the Bloom gate, which earns it honestly: Bloom runs a separable blur over dozens
/// of taps, so its error accumulates. This pass is a handful of multiply-adds, a `sqrt`, a divide and two
/// table lookups per texel — its error has nowhere to accumulate.
const MAX_DELTA: i32 = 2;

/// How many bytes may differ AT ALL, at any magnitude.
///
/// A magnitude bound alone is not enough, and a mutation proved it: dropping the `+ 0.5` from the
/// shader's `quantise` — turning round-half-away into truncation, the exact CPU/GPU rounding divergence
/// that function exists to abolish — moves a great many texels by ONE byte, and sailed straight under a
/// bound of 2. The two questions are different and both must be asked:
///
/// - *how far* did any texel drift (FMA contraction, a ULP, a genuinely different number);
/// - *how many* drifted at all (a systematic convention error, which is small per texel and everywhere).
///
/// Sixteen: comfortably above the zero this backend delivers and the stray ULP another might, and three
/// orders of magnitude below the thousands a convention error produces.
const MAX_DIFFERING_BYTES: usize = 16;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A canvas with real relief, real colour variety and a non-neutral material.
///
/// Every ingredient is load-bearing, and a fixture missing one would be green about nothing
/// ([[feedback_a_fixture_only_proves_what_it_contains]]):
/// - **crossing strokes** give slopes in every direction, including the steep walls at a stroke's cap,
///   which is where a normal is most sensitive;
/// - **a varied backdrop** matters because `albedo` is an INPUT to the shade, not only the thing it
///   multiplies — a metal's highlight takes the paint's colour;
/// - **a non-neutral material** is the only way the Wax and Metallic terms are anything but zero, and
///   those terms are precisely the ones that split the coloured path from the fast lane.
fn sculpted(shine: f32, rough: f32, metallic: f32, wax: f32) -> PainterTool {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    // A backdrop that varies in all three channels — a flat grey canvas would hide every albedo-dependent
    // term in the model.
    let mut px = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            px.extend_from_slice(&[(x * 4) as u8, (y * 4) as u8, 255 - (x * 2) as u8, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(px, W, H);
    t.toggle_brush_impasto(); // on (and this is the call that now defaults the falloff to Sphere)
    t.set_brush_impasto_depth(0.8);
    t.set_brush_size_px(9.0);
    t.set_brush_color_channel(0, 0.85);
    t.set_brush_color_channel(1, 0.30);
    t.set_brush_color_channel(2, 0.10);
    t.set_impasto_shine(shine);
    t.set_impasto_roughness(rough);
    t.set_impasto_metallic(metallic);
    t.set_impasto_wax(wax);
    t.set_impasto_wax_color([1.0, 0.7, 0.45]);
    for (a, b) in [
        ([12.0f32, 20.0f32], [52.0f32, 30.0f32]),
        ([30.0, 8.0], [34.0, 56.0]),
    ] {
        t.on_canvas_pointer(cp(a, PointerPhase::Down));
        t.on_canvas_pointer(cp(
            [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
            PointerPhase::Move,
        ));
        t.on_canvas_pointer(cp(b, PointerPhase::Move));
        t.on_canvas_pointer(cp(b, PointerPhase::Up));
    }
    t
}

/// The unlit pixels both paths are handed. Synthesised rather than composited on purpose: the light's
/// contract is `(rgba, planes) -> rgba`, and feeding both sides the identical buffer is what makes the
/// comparison about the shading and nothing else.
fn backdrop() -> Vec<u8> {
    let mut v = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            v.extend_from_slice(&[
                (20 + x * 3) as u8,
                (200 - y * 2) as u8,
                (60 + (x ^ y) * 2) as u8,
                255,
            ]);
        }
    }
    v
}

fn cpu_lit(t: &PainterTool, base: &[u8]) -> Vec<u8> {
    let mut buf = base.to_vec();
    t.apply_impasto_light(
        &mut buf,
        Region {
            x: 0,
            y: 0,
            w: W,
            h: H,
        },
    );
    buf
}

fn gpu_lit(gpu: &GpuContext, planes: &ImpastoPlanes, base: &[u8]) -> Vec<u8> {
    let src = make_src(gpu, base);
    let lamps: Vec<GpuLamp> = planes
        .lamps
        .iter()
        .map(|l| GpuLamp {
            dir: l.dir,
            half: l.half,
            tint: l.tint,
        })
        .collect();
    let mut pass = ImpastoLightPass::new(gpu);
    let input = ImpastoLightInput {
        width: planes.width,
        height: planes.height,
        region: GpuRegion::full(planes.width, planes.height),
        // The window the CPU folded. `impasto_gpu_planes` folds the whole canvas, so this reconciliation
        // runs the SEEDING upload — which is also the only one a parity gate may run, since a partial
        // upload's correctness is a statement about the frame BEFORE it and this fixture has none.
        plane_region: GpuRegion {
            x: planes.region.0,
            y: planes.region.1,
            w: planes.region.2,
            h: planes.region.3,
        },
        relief: &planes.relief,
        cover: &planes.cover,
        mat0: &planes.mat0,
        mat1: &planes.mat1,
        lamps: &lamps,
        spec_lut: planes.spec_lut,
        lut_width: planes.lut_width,
        rough_levels: planes.rough_levels,
    };
    let out = pass.run(gpu, &src, &input).expect("a well-formed dispatch");
    readback(gpu, out)
}

/// How far, where, and how many. Reported as well as asserted — a gate that only says "under the bar"
/// hides a drift walking toward it.
struct Drift {
    max: i32,
    at: usize,
    differing: usize,
}

fn compare(cpu: &[u8], gpu: &[u8]) -> Drift {
    let mut d = Drift {
        max: 0,
        at: 0,
        differing: 0,
    };
    for (i, (a, b)) in cpu.iter().zip(gpu).enumerate() {
        let delta = (i32::from(*a) - i32::from(*b)).abs();
        if delta > 0 {
            d.differing += 1;
        }
        if delta > d.max {
            d.max = delta;
            d.at = i;
        }
    }
    d
}

fn make_src(gpu: &GpuContext, bytes: &[u8]) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test impasto src"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    tex
}

fn readback(gpu: &GpuContext, tex: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test impasto readback"),
        size: (padded as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("recv").expect("map");
    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (H as usize));
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

/// **The gate.** The shader draws what the CPU pass draws, over a canvas that actually contains relief,
/// colour variety and a non-neutral material.
///
/// The default rig is GREY, which means the CPU takes its achromatic fast lane (one channel computed
/// instead of three) while the shader takes its only lane, the coloured one. That is not an accident of
/// the fixture — it is the end-to-end check on the algebra that licensed a single-path shader
/// (`the_achromatic_fast_lane_is_the_coloured_one_to_the_bit`, in the tool crate).
#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn gpu_impasto_light_matches_the_cpu_pass() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let base = backdrop();
    for (name, shine, rough, metallic, wax) in [
        ("neutral-ish", 0.0, 0.5, 0.0, 0.0),
        ("glossy", 0.8, 0.15, 0.0, 0.0),
        ("metal", 0.7, 0.35, 1.0, 0.0),
        ("waxy", 0.4, 0.7, 0.0, 0.85),
        ("metal+wax", 0.9, 0.05, 0.6, 0.5),
    ] {
        let t = sculpted(shine, rough, metallic, wax);
        let planes = t
            .impasto_gpu_planes()
            .expect("the fixture is lit and sculpted");
        // The fixture must CONTAIN the phenomenon: a canvas the light never touches would pass this gate
        // by doing nothing on both sides.
        let touched = planes.cover.iter().filter(|c| **c > 0).count();
        assert!(
            touched > 300,
            "{name}: the fixture must carry paint (only {touched} texels do)"
        );
        let cpu = cpu_lit(&t, &base);
        assert_ne!(
            cpu, base,
            "{name}: …and the light must actually MOVE those pixels, or parity is vacuous"
        );
        let gpu_out = gpu_lit(&gpu, &planes, &base);
        let d = compare(&cpu, &gpu_out);
        eprintln!(
            "impasto light parity [{name}]: worst delta {} at byte {}, {} of {} bytes differ",
            d.max,
            d.at,
            d.differing,
            cpu.len()
        );
        assert!(
            d.max <= MAX_DELTA,
            "{name}: the shader drifted {} bytes from the CPU pass at byte {} \
             (cpu {} vs gpu {}) — bound is {MAX_DELTA}",
            d.max,
            d.at,
            cpu[d.at],
            gpu_out[d.at]
        );
        assert!(
            d.differing <= MAX_DIFFERING_BYTES,
            "{name}: {} bytes differ from the CPU pass (bound is {MAX_DIFFERING_BYTES}). \
             Small-but-everywhere is the signature of a systematic difference — a rounding \
             convention, an off-by-one index — not of floating-point noise",
            d.differing
        );
    }
}

/// **Structural contract 1: flat paint is byte-identical.** Exact, not epsilon. An absolute shading model
/// would darken the entire painting the moment the light came on — half the emboss filters ever written
/// have that bug — and this is the gate that would catch the shader growing one.
///
/// ## A survivor worth recording: the shader honours this TWICE
///
/// Neutering the early-out (`if ((dhx == 0 && dhy == 0) || body <= 0)` -> never taken, so flat paint runs
/// the full lamp loop) leaves this gate GREEN. That is not a hole, it is the design being true: on zero
/// slope `N.L` is exactly `L.z` for every lamp, so the diffuse equals the flat divisor term for term and
/// the ratio is exactly 1 — the relative-shading contract delivers byte-identity through the arithmetic,
/// and the branch is an optimisation sitting on top of it.
///
/// Two independent mechanisms mean no single mutation can kill this gate
/// ([[feedback_layered_defenses_need_per_layer_gates]]): the branch is covered by the parity gate (which
/// compares against a CPU pass that has the same branch), and the arithmetic is covered by this one with
/// the branch removed. Anyone who deletes the early-out for tidiness should know it costs speed and
/// nothing else; anyone who makes the shading absolute breaks this gate whether the branch is there or
/// not.
#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn the_shader_leaves_flat_paint_byte_identical() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let t = sculpted(0.8, 0.3, 0.4, 0.4);
    let mut planes = t.impasto_gpu_planes().expect("lit and sculpted");
    // Full coverage everywhere and a perfectly level relief: there is paint, it is simply not tilted.
    // (Coverage matters — zeroing it would test the OTHER contract and leave this one unmeasured.)
    planes.relief.fill(0.5);
    planes.cover.fill(255);
    let base = backdrop();
    let out = gpu_lit(&gpu, &planes, &base);
    assert_eq!(
        out, base,
        "level relief must multiply by exactly 1 and add exactly 0 — the relative-shading contract"
    );
}

/// **Structural contract 2: bare paper gets exactly nothing.** The pass multiplies a composited pixel,
/// and at a stroke's translucent edge that pixel is mostly paper showing through. Shading it in full
/// means bleaching the paper — the halo this weighting exists to kill.
#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn the_shader_does_not_touch_bare_paper() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let t = sculpted(1.0, 0.1, 0.5, 0.6);
    let mut planes = t.impasto_gpu_planes().expect("lit and sculpted");
    // Relief left exactly as sculpted — steep, real, everywhere the strokes went. Only the PAINT is gone.
    // Relief over zero coverage must not light: it is a shape with nothing standing on it.
    planes.cover.fill(0);
    let base = backdrop();
    let out = gpu_lit(&gpu, &planes, &base);
    assert_eq!(
        out, base,
        "relief over zero coverage is a no-op: the light modulates paint that is there, \
         it does not invent it"
    );
}

/// **Um upload PARCIAL escreve a janela onde ela está, em qualquer largura.**
///
/// O report do smoke (Enio, 2026-07-25) é um retângulo branco que aparece **pintando** — e um bloco
/// retangular de branco é o que se vê quando os planos recebem LIXO ali: normais absurdas, a luz
/// estoura. O suspeito é a forma do upload que esta wave introduziu, a única com origem não-zero e
/// buffer empacotado por região: `bytes_per_row = region.w x texel`, um passo ARBITRÁRIO.
///
/// O repo já tinha a forma segura e ela é outra — `layer_compositor`'s `upload_slice_region` mantém o
/// buffer de tela CHEIA e usa `offset` + o passo da linha inteira. Se o passo arbitrário for o defeito,
/// ele aparece em larguras que não são múltiplos convenientes, e é exatamente isso que os gates de
/// paridade não cobriam: eles sobem a tela inteira.
///
/// Então: para várias larguras (ímpar, primo, não-alinhada), sobe a janela e confronta com o que o
/// upload de tela CHEIA desenha nos mesmos texels. Divergir é o bug reportado.
#[test]
#[ignore = "requires a GPU adapter"]
fn a_partial_plane_upload_lands_where_it_belongs_at_any_width() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let t = sculpted(0.6, 0.4, 0.0, 0.0);
    let base = backdrop();
    let whole = t.impasto_gpu_planes().expect("planos");
    let full_lit = gpu_lit(&gpu, &whole, &base);

    // Larguras deliberadamente inconvenientes: 1 texel de linha (17 e 33 bytes num plano rgba8) nunca é
    // múltiplo de 256, que é o alinhamento que uma cópia buffer->textura exigiria.
    for w in [17u32, 31, 33, 63, 100] {
        let win = (
            3u32,
            5u32,
            w.min(whole.width - 3),
            11u32.min(whole.height - 5),
        );
        let part = t.impasto_gpu_planes_in(win).expect("a janela tem planos");
        assert_eq!(part.region, win, "a janela reporta onde vai (w={w})");

        // Um passe NOVO por largura: ele precisa da SEMEADURA cheia antes do parcial (é o contrato que
        // `planes_seeded` guarda), e reusar um passe entre larguras esconderia uma janela que só
        // funciona porque a anterior já escreveu ali.
        let mut pass = ImpastoLightPass::new(&gpu);
        let src = make_src(&gpu, &base);
        let lamps: Vec<GpuLamp> = whole
            .lamps
            .iter()
            .map(|l| GpuLamp {
                dir: l.dir,
                half: l.half,
                tint: l.tint,
            })
            .collect();
        fn mk<'a>(p: &'a ImpastoPlanes, lamps: &'a [GpuLamp]) -> ImpastoLightInput<'a> {
            ImpastoLightInput {
                width: p.width,
                height: p.height,
                region: GpuRegion::full(p.width, p.height),
                plane_region: GpuRegion {
                    x: p.region.0,
                    y: p.region.1,
                    w: p.region.2,
                    h: p.region.3,
                },
                relief: &p.relief,
                cover: &p.cover,
                mat0: &p.mat0,
                mat1: &p.mat1,
                lamps,
                spec_lut: p.spec_lut,
                lut_width: p.lut_width,
                rough_levels: p.rough_levels,
            }
        }
        // Semeia cheio, depois re-sobe SÓ a janela (com os mesmos valores) — o resultado tem de ficar
        // igual ao de tela cheia. Se o passo arbitrário desalinhar, a janela vira lixo e diverge.
        pass.run(&gpu, &src, &mk(&whole, &lamps))
            .expect("semeadura");
        let out = pass.run(&gpu, &src, &mk(&part, &lamps)).expect("janela");
        let got = readback(&gpu, out);
        let d = compare(&full_lit, &got);
        assert!(
            d.max <= MAX_DELTA && d.differing <= MAX_DIFFERING_BYTES,
            "janela de {w} texels de largura divergiu do upload cheio: pior {} em {} bytes \
             (um bloco retangular de lixo é o report do smoke)",
            d.max,
            d.differing
        );
    }
}
